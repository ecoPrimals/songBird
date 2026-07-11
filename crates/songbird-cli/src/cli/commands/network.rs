// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Network probing commands — real TCP RTT measurement against mesh peers.
//!
//! Parses `SONGBIRD_PEERS` environment variable to discover target peers
//! and measures actual round-trip latency via TCP connect probes.

#![allow(missing_docs, reason = "network clap enums document flags inline")]

use crate::errors::SongbirdResult;
use clap::Subcommand;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

const PROBE_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Subcommand)]
pub enum NetworkCommand {
    /// Probe mesh peers for real latency measurement
    #[command(about = "Probe mesh peer latency via TCP connect")]
    Test {
        /// Target specific peer (node_id or address)
        #[arg(long)]
        target: Option<String>,

        /// Number of probe iterations
        #[arg(long, default_value = "10")]
        iterations: u32,
    },

    /// Monitor network health continuously
    #[command(about = "Monitor mesh peer health in real-time")]
    Monitor {
        /// Probe interval in seconds
        #[arg(long, default_value = "5")]
        interval: u64,

        /// Run until interrupted
        #[arg(long)]
        continuous: bool,
    },

    /// Diagnose network connectivity
    #[command(about = "Diagnose mesh connectivity and port reachability")]
    Diagnose {
        /// Run comprehensive checks (DNS, ports, sockets)
        #[arg(long)]
        comprehensive: bool,
    },
}

/// Handle network commands
pub async fn handle_network_command(command: NetworkCommand) -> SongbirdResult<()> {
    match command {
        NetworkCommand::Test { target, iterations } => probe_peers(target, iterations).await,
        NetworkCommand::Monitor {
            interval,
            continuous,
        } => monitor_peers(interval, continuous).await,
        NetworkCommand::Diagnose { comprehensive } => diagnose_connectivity(comprehensive).await,
    }
}

/// Peer endpoint parsed from `SONGBIRD_PEERS`.
#[derive(Debug)]
struct PeerTarget {
    node_id: String,
    addr: SocketAddr,
}

/// Parse `SONGBIRD_PEERS` env var (format: `node_id@host:port,...`).
fn resolve_peers(filter: Option<&str>) -> Result<Vec<PeerTarget>, String> {
    let raw = songbird_process_env::var("SONGBIRD_PEERS")
        .map_err(|_| String::from("SONGBIRD_PEERS not set — no mesh peers configured"))?;
    parse_peers_value(&raw, filter)
}

/// Pure parser for the `SONGBIRD_PEERS` format — testable without global state.
fn parse_peers_value(raw: &str, filter: Option<&str>) -> Result<Vec<PeerTarget>, String> {
    if raw.trim().is_empty() {
        return Err(String::from(
            "SONGBIRD_PEERS is empty — configure mesh peers first",
        ));
    }

    let mut peers = Vec::new();
    for entry in raw.split(',') {
        let entry = entry.trim();
        if entry.is_empty() {
            continue;
        }
        let Some((node_id, addr_str)) = entry.split_once('@') else {
            continue;
        };
        let Ok(addr) = addr_str.parse::<SocketAddr>() else {
            continue;
        };
        if let Some(f) = filter
            && node_id != f && addr_str != f
        {
            continue;
        }
        peers.push(PeerTarget {
            node_id: node_id.to_string(),
            addr,
        });
    }

    if peers.is_empty() {
        return Err(if filter.is_some() {
            String::from("No matching peer found in SONGBIRD_PEERS")
        } else {
            String::from("No valid peers parsed from SONGBIRD_PEERS")
        });
    }

    Ok(peers)
}

/// Measure TCP connect RTT to an address.
async fn tcp_probe(addr: SocketAddr) -> Result<Duration, String> {
    let start = Instant::now();
    tokio::time::timeout(PROBE_TIMEOUT, tokio::net::TcpStream::connect(addr))
        .await
        .map_err(|_| format!("timeout ({PROBE_TIMEOUT:?})"))?
        .map_err(|e| format!("connect failed: {e}"))?;
    Ok(start.elapsed())
}

/// Probe peers with real TCP latency measurement.
async fn probe_peers(target: Option<String>, iterations: u32) -> SongbirdResult<()> {
    let peers = match resolve_peers(target.as_deref()) {
        Ok(p) => p,
        Err(e) => {
            println!("error: {e}");
            println!("hint: set SONGBIRD_PEERS=node@host:port,... to configure mesh targets");
            return Ok(());
        }
    };

    println!(
        "Probing {} peer(s), {} iterations each:",
        peers.len(),
        iterations
    );
    println!();

    for peer in &peers {
        println!("  {} ({})", peer.node_id, peer.addr);
        let mut rtts: Vec<Duration> = Vec::with_capacity(iterations as usize);
        let mut failures = 0u32;

        for _ in 0..iterations {
            match tcp_probe(peer.addr).await {
                Ok(rtt) => rtts.push(rtt),
                Err(_) => failures += 1,
            }
        }

        if rtts.is_empty() {
            println!("    unreachable ({failures}/{iterations} probes failed)");
        } else {
            let stats = compute_stats(&rtts);
            println!(
                "    latency: min={:.1}ms avg={:.1}ms max={:.1}ms jitter={:.1}ms loss={}/{iterations}",
                stats.min, stats.avg, stats.max, stats.jitter, failures
            );
        }
        println!();
    }

    Ok(())
}

/// Monitor peers in a loop with real probes.
async fn monitor_peers(interval: u64, continuous: bool) -> SongbirdResult<()> {
    let peers = match resolve_peers(None) {
        Ok(p) => p,
        Err(e) => {
            println!("error: {e}");
            return Ok(());
        }
    };

    let interval_dur = Duration::from_secs(interval);
    let cycles = if continuous { u32::MAX } else { 1 };

    for cycle in 0..cycles {
        if cycle > 0 {
            tokio::time::sleep(interval_dur).await;
        }

        let now = chrono::Utc::now().format("%H:%M:%S");
        println!("[{now}] Mesh health:");

        for peer in &peers {
            match tcp_probe(peer.addr).await {
                Ok(rtt) => {
                    let ms = rtt.as_secs_f64() * 1000.0;
                    println!("  {} ({}) — {ms:.1}ms", peer.node_id, peer.addr);
                }
                Err(e) => {
                    println!("  {} ({}) — DOWN ({e})", peer.node_id, peer.addr);
                }
            }
        }
        println!();
    }

    Ok(())
}

/// Diagnose connectivity: check IPC socket, peer reachability, DNS.
async fn diagnose_connectivity(comprehensive: bool) -> SongbirdResult<()> {
    println!("Network diagnostics:");
    println!();

    // 1. Check IPC socket
    let biomeos_dir = songbird_process_env::var("BIOMEOS_SOCKET_DIR")
        .map(std::path::PathBuf::from)
        .or_else(|_| {
            songbird_process_env::var("XDG_RUNTIME_DIR").map(|xdg| {
                std::path::PathBuf::from(xdg)
                    .join(songbird_types::defaults::paths::BIOMEOS_RUNTIME_SUBDIR)
            })
        })
        .unwrap_or_else(|_| std::env::temp_dir());

    let socket_path =
        biomeos_dir.join(format!("{}.sock", songbird_types::primal_names::SELF_NAME));
    if socket_path.exists() {
        println!("  IPC socket: OK ({})", socket_path.display());
    } else {
        println!("  IPC socket: NOT FOUND ({})", socket_path.display());
        println!("    hint: is the songBird orchestrator running?");
    }

    // 2. Check mesh peers
    match resolve_peers(None) {
        Ok(peers) => {
            println!("  Configured peers: {}", peers.len());
            for peer in &peers {
                match tcp_probe(peer.addr).await {
                    Ok(rtt) => {
                        let ms = rtt.as_secs_f64() * 1000.0;
                        println!("    {} ({}) — reachable ({ms:.1}ms)", peer.node_id, peer.addr);
                    }
                    Err(e) => {
                        println!("    {} ({}) — UNREACHABLE ({e})", peer.node_id, peer.addr);
                    }
                }
            }
        }
        Err(e) => {
            println!("  Mesh peers: {e}");
        }
    }

    // 3. Comprehensive: DNS resolution check
    if comprehensive {
        println!();
        println!("  DNS resolution:");
        let test_hosts = ["dns.google", "one.one.one.one"];
        for host in test_hosts {
            match tokio::net::lookup_host(format!("{host}:443")).await {
                Ok(mut addrs) => {
                    if let Some(addr) = addrs.next() {
                        println!("    {host} -> {addr} OK");
                    } else {
                        println!("    {host} -> no addresses");
                    }
                }
                Err(e) => println!("    {host} -> FAILED ({e})"),
            }
        }

        // Federation port check
        let fed_port = songbird_process_env::var("SONGBIRD_FEDERATION_PORT")
            .unwrap_or_else(|_| String::from("7700"));
        println!();
        println!("  Federation port: {fed_port}");
        let bind_test: SocketAddr = format!("0.0.0.0:{fed_port}")
            .parse()
            .unwrap_or_else(|_| SocketAddr::from(([0, 0, 0, 0], 7700)));
        match tokio::net::TcpListener::bind(bind_test).await {
            Ok(_) => println!("    bind test: OK (port available)"),
            Err(e) => println!("    bind test: IN USE or blocked ({e})"),
        }
    }

    println!();
    println!("Done.");
    Ok(())
}

struct ProbeStats {
    min: f64,
    avg: f64,
    max: f64,
    jitter: f64,
}

/// Compute min/avg/max/jitter (all in milliseconds) from a series of RTT durations.
fn compute_stats(rtts: &[Duration]) -> ProbeStats {
    let ms_values: Vec<f64> = rtts.iter().map(|d| d.as_secs_f64() * 1000.0).collect();
    let min = ms_values.iter().copied().reduce(f64::min).unwrap_or(0.0);
    let max = ms_values.iter().copied().reduce(f64::max).unwrap_or(0.0);
    let avg = ms_values.iter().sum::<f64>() / ms_values.len() as f64;

    let jitter = if ms_values.len() > 1 {
        ms_values.iter().map(|v| (v - avg).abs()).sum::<f64>() / ms_values.len() as f64
    } else {
        0.0
    };

    ProbeStats {
        min,
        avg,
        max,
        jitter,
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn compute_stats_single_sample() {
        let rtts = vec![Duration::from_millis(42)];
        let stats = compute_stats(&rtts);
        assert!((stats.avg - 42.0).abs() < 0.5);
        assert!((stats.jitter).abs() < 0.01);
    }

    #[test]
    fn compute_stats_multiple_samples() {
        let rtts = vec![
            Duration::from_millis(10),
            Duration::from_millis(20),
            Duration::from_millis(30),
        ];
        let stats = compute_stats(&rtts);
        assert!((stats.min - 10.0).abs() < 0.5);
        assert!((stats.max - 30.0).abs() < 0.5);
        assert!((stats.avg - 20.0).abs() < 0.5);
        assert!(stats.jitter > 0.0);
    }

    #[test]
    fn parse_empty_value_errors() {
        let result = parse_peers_value("", None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty"));
    }

    #[test]
    fn parse_valid_entries() {
        let peers = parse_peers_value("gate-a@192.168.1.1:7700,gate-b@192.168.1.2:7700", None).unwrap();
        assert_eq!(peers.len(), 2);
        assert_eq!(peers[0].node_id, "gate-a");
        assert_eq!(peers[1].node_id, "gate-b");
    }

    #[test]
    fn parse_filters_by_node_id() {
        let peers = parse_peers_value(
            "gate-a@192.168.1.1:7700,gate-b@192.168.1.2:7700",
            Some("gate-b"),
        ).unwrap();
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0].node_id, "gate-b");
    }

    #[test]
    fn parse_skips_malformed_entries() {
        let peers = parse_peers_value(
            "good@10.0.0.1:7700,bad-no-at-sign,also-bad@not-a-socket",
            None,
        ).unwrap();
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0].node_id, "good");
    }

    #[test]
    fn parse_filter_no_match_errors() {
        let result = parse_peers_value("gate-a@10.0.0.1:7700", Some("nonexistent"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No matching"));
    }
}
