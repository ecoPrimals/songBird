// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Tower Atomic / `WireGuard` parity benchmark harness.
//!
//! Measures latency (TCP RTT), connection setup time, and throughput
//! for both Tower Atomic stack and `WireGuard` baseline. Outputs structured
//! JSON for `primalSpring` consumption.

use anyhow::{Context, Result, anyhow};
use clap::{Args, ValueEnum};
use serde::Serialize;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

/// Benchmark mode — which transport stack to measure.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum BenchmarkMode {
    /// Measure the Tower Atomic stack (`songBird` mesh TCP/TURN).
    TowerAtomic,
    /// Measure `WireGuard` baseline (TCP over WG interface).
    Wireguard,
}

/// CLI arguments for `songbird benchmark`.
#[derive(Args, Debug, Clone)]
pub struct BenchmarkArgs {
    /// Transport stack to benchmark.
    #[arg(long, value_enum, default_value = "tower-atomic")]
    pub mode: BenchmarkMode,

    /// Peer address to benchmark against (ip:port).
    /// For tower-atomic: the peer's mesh port (default 7700).
    /// For wireguard: the peer's WG IP + any TCP port (e.g. 10.13.37.2:7700).
    #[arg(long)]
    pub peer: String,

    /// Duration of the throughput test.
    #[arg(long, default_value = "10s", value_parser = parse_duration)]
    pub duration: Duration,

    /// Number of latency probes to send.
    #[arg(long, default_value_t = 50)]
    pub probes: u32,

    /// Output format.
    #[arg(long, default_value = "json")]
    pub output: OutputFormat,

    /// Timeout for individual operations.
    #[arg(long, default_value = "5s", value_parser = parse_duration)]
    pub timeout: Duration,
}

/// Output format for benchmark results.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    /// JSON (machine-readable, for primalSpring).
    Json,
    /// Human-readable text summary.
    Text,
}

/// Full benchmark report.
#[derive(Debug, Serialize)]
#[allow(missing_docs)]
pub struct BenchmarkReport {
    pub mode: String,
    pub peer: String,
    pub timestamp: String,
    pub latency: LatencyReport,
    pub setup: SetupReport,
    pub throughput: ThroughputReport,
}

/// Latency measurement results.
#[derive(Debug, Serialize)]
#[allow(missing_docs)]
pub struct LatencyReport {
    pub probes_sent: u32,
    pub probes_ok: u32,
    pub min_ms: f64,
    pub avg_ms: f64,
    pub max_ms: f64,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub jitter_ms: f64,
}

/// Connection setup time results.
#[derive(Debug, Serialize)]
#[allow(missing_docs)]
pub struct SetupReport {
    pub attempts: u32,
    pub min_ms: f64,
    pub avg_ms: f64,
    pub max_ms: f64,
}

/// Throughput measurement results.
#[derive(Debug, Serialize)]
#[allow(missing_docs)]
pub struct ThroughputReport {
    pub duration_ms: u64,
    pub bytes_sent: u64,
    pub throughput_mbps: f64,
}

/// Run the benchmark with the given arguments.
pub async fn run_benchmark(args: &BenchmarkArgs) -> Result<()> {
    let peer_addr: SocketAddr =
        args.peer.parse().with_context(|| format!("Invalid peer address: {}", args.peer))?;

    let mode_str = match args.mode {
        BenchmarkMode::TowerAtomic => "tower-atomic",
        BenchmarkMode::Wireguard => "wireguard",
    };

    eprintln!("songbird benchmark — mode: {mode_str}, peer: {peer_addr}");
    eprintln!("  latency probes: {}, throughput duration: {:?}", args.probes, args.duration);
    eprintln!();

    // Phase 1: Connection setup measurement
    eprintln!("[1/3] Measuring connection setup time...");
    let setup = measure_setup(peer_addr, args.timeout, 10).await?;

    // Phase 2: Latency measurement (TCP RTT)
    eprintln!("[2/3] Measuring latency ({} probes)...", args.probes);
    let latency = measure_latency(peer_addr, args.timeout, args.probes).await?;

    // Phase 3: Throughput measurement
    eprintln!("[3/3] Measuring throughput ({:?})...", args.duration);
    let throughput = measure_throughput(peer_addr, args.timeout, args.duration).await?;

    let report = BenchmarkReport {
        mode: mode_str.to_string(),
        peer: peer_addr.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        latency,
        setup,
        throughput,
    };

    match args.output {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        OutputFormat::Text => {
            print_text_report(&report);
        }
    }

    Ok(())
}

/// Measure TCP connection setup time.
async fn measure_setup(
    peer: SocketAddr,
    op_timeout: Duration,
    attempts: u32,
) -> Result<SetupReport> {
    let mut times = Vec::with_capacity(attempts as usize);

    for _ in 0..attempts {
        let start = Instant::now();
        if let Ok(Ok(stream)) = timeout(op_timeout, TcpStream::connect(peer)).await {
            times.push(start.elapsed());
            drop(stream);
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    if times.is_empty() {
        return Err(anyhow!("All connection attempts to {peer} failed"));
    }

    let times_ms: Vec<f64> = times.iter().map(|d| d.as_secs_f64() * 1000.0).collect();
    #[expect(clippy::cast_possible_truncation, reason = "attempt count fits u32")]
    let attempts = times.len() as u32;
    #[expect(clippy::cast_precision_loss, reason = "timing stats — precision loss negligible")]
    let avg = times_ms.iter().sum::<f64>() / times_ms.len() as f64;
    Ok(SetupReport {
        attempts,
        min_ms: times_ms.iter().copied().reduce(f64::min).unwrap_or(0.0),
        avg_ms: avg,
        max_ms: times_ms.iter().copied().reduce(f64::max).unwrap_or(0.0),
    })
}

/// Measure TCP round-trip latency via JSON-RPC health.ping.
async fn measure_latency(
    peer: SocketAddr,
    op_timeout: Duration,
    probes: u32,
) -> Result<LatencyReport> {
    let mut times = Vec::with_capacity(probes as usize);
    let ping_req = br#"{"jsonrpc":"2.0","method":"health.ping","params":{},"id":1}"#;

    for _ in 0..probes {
        let start = Instant::now();
        let result = timeout(op_timeout, async {
            let mut stream = TcpStream::connect(peer).await?;
            stream.write_all(ping_req).await?;
            stream.write_all(b"\n").await?;
            let mut buf = vec![0u8; 1024];
            let _n = stream.read(&mut buf).await?;
            Ok::<_, std::io::Error>(())
        })
        .await;

        if matches!(result, Ok(Ok(()))) {
            times.push(start.elapsed());
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    if times.is_empty() {
        return Err(anyhow!("All latency probes to {peer} failed"));
    }

    let mut times_ms: Vec<f64> = times.iter().map(|d| d.as_secs_f64() * 1000.0).collect();
    times_ms.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let len = times_ms.len();
    #[expect(clippy::cast_precision_loss, reason = "timing stats")]
    let avg = times_ms.iter().sum::<f64>() / len as f64;
    let jitter = if len > 1 {
        let diffs: Vec<f64> = times_ms.windows(2).map(|w| (w[1] - w[0]).abs()).collect();
        #[expect(clippy::cast_precision_loss, reason = "timing stats")]
        let j = diffs.iter().sum::<f64>() / diffs.len() as f64;
        j
    } else {
        0.0
    };

    #[expect(clippy::cast_possible_truncation, reason = "probe count fits u32")]
    let probes_ok = len as u32;
    Ok(LatencyReport {
        probes_sent: probes,
        probes_ok,
        min_ms: times_ms[0],
        avg_ms: avg,
        max_ms: times_ms[len - 1],
        p50_ms: percentile(&times_ms, 50.0),
        p95_ms: percentile(&times_ms, 95.0),
        p99_ms: percentile(&times_ms, 99.0),
        jitter_ms: jitter,
    })
}

/// Measure throughput by streaming data over TCP.
async fn measure_throughput(
    peer: SocketAddr,
    op_timeout: Duration,
    duration: Duration,
) -> Result<ThroughputReport> {
    let mut stream = timeout(op_timeout, TcpStream::connect(peer))
        .await
        .map_err(|_| anyhow!("Connection timeout for throughput test"))?
        .map_err(|e| anyhow!("TCP connect failed: {e}"))?;

    // Send a benchmark-mode signal so the peer knows to echo/sink data.
    // For now, just blast data and measure write throughput (the peer will
    // RST or accept — we measure our send rate as a lower bound).
    let chunk = vec![0xABu8; 65_536]; // 64 KiB chunks
    let start = Instant::now();
    let mut bytes_sent: u64 = 0;

    while start.elapsed() < duration {
        if matches!(timeout(Duration::from_secs(2), stream.write_all(&chunk)).await, Ok(Ok(()))) {
            bytes_sent += chunk.len() as u64;
        } else {
            break;
        }
    }

    let elapsed = start.elapsed();
    #[expect(
        clippy::cast_precision_loss,
        reason = "throughput calculation — precision loss negligible at these magnitudes"
    )]
    let throughput_mbps = if elapsed.as_secs_f64() > 0.0 {
        (bytes_sent as f64 * 8.0) / (elapsed.as_secs_f64() * 1_000_000.0)
    } else {
        0.0
    };

    #[expect(clippy::cast_possible_truncation, reason = "benchmark duration < u64::MAX ms")]
    let duration_ms = elapsed.as_millis() as u64;
    Ok(ThroughputReport {
        duration_ms,
        bytes_sent,
        throughput_mbps,
    })
}

fn percentile(sorted: &[f64], pct: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    #[expect(clippy::cast_precision_loss, reason = "percentile index calculation")]
    let raw_idx = (pct / 100.0) * (sorted.len() - 1) as f64;
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "index from positive float"
    )]
    let idx = raw_idx.round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn print_text_report(report: &BenchmarkReport) {
    println!("═══════════════════════════════════════════════════");
    println!("  Songbird Benchmark Report");
    println!("  Mode: {}  |  Peer: {}", report.mode, report.peer);
    println!("  Time: {}", report.timestamp);
    println!("═══════════════════════════════════════════════════");
    println!();
    println!("  ┌─ Setup ({} attempts)", report.setup.attempts);
    println!(
        "  │  min: {:.2} ms  avg: {:.2} ms  max: {:.2} ms",
        report.setup.min_ms, report.setup.avg_ms, report.setup.max_ms
    );
    println!("  │");
    println!(
        "  ├─ Latency ({}/{} probes OK)",
        report.latency.probes_ok, report.latency.probes_sent
    );
    println!(
        "  │  min: {:.2} ms  avg: {:.2} ms  max: {:.2} ms",
        report.latency.min_ms, report.latency.avg_ms, report.latency.max_ms
    );
    println!(
        "  │  p50: {:.2} ms  p95: {:.2} ms  p99: {:.2} ms",
        report.latency.p50_ms, report.latency.p95_ms, report.latency.p99_ms
    );
    println!("  │  jitter: {:.2} ms", report.latency.jitter_ms);
    println!("  │");
    #[expect(clippy::cast_precision_loss, reason = "display formatting only")]
    let dur_secs = report.throughput.duration_ms as f64 / 1000.0;
    println!("  └─ Throughput ({dur_secs:.1}s)");
    println!(
        "     sent: {} bytes  rate: {:.2} Mbps",
        report.throughput.bytes_sent, report.throughput.throughput_mbps
    );
    println!();
}

fn parse_duration(s: &str) -> Result<Duration, String> {
    let s = s.trim();
    if let Some(secs) = s.strip_suffix('s') {
        secs.parse::<f64>()
            .map(Duration::from_secs_f64)
            .map_err(|e| format!("Invalid duration: {e}"))
    } else if let Some(ms) = s.strip_suffix("ms") {
        ms.parse::<u64>().map(Duration::from_millis).map_err(|e| format!("Invalid duration: {e}"))
    } else {
        s.parse::<f64>()
            .map(Duration::from_secs_f64)
            .map_err(|_| format!("Invalid duration '{s}' — use '10s' or '500ms'"))
    }
}
