//! HTTP Baseline Benchmark
//!
//! Measures performance of plain HTTP REST API calls to Songbird

mod common;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use common::{measure_latency, print_banner, BenchmarkConfig, BenchmarkResults};
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(name = "HTTP Baseline Benchmark")]
#[command(about = "Benchmark HTTP REST API performance")]
struct Args {
    /// Target Songbird URL
    #[arg(short, long, default_value = "https://localhost:8443")]
    target: String,

    /// Number of requests to send
    #[arg(short, long, default_value = "10000")]
    num_requests: usize,

    /// Warmup requests (not counted)
    #[arg(short, long, default_value = "100")]
    warmup: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    print_banner("🌐 HTTP BASELINE BENCHMARK");

    let config = BenchmarkConfig {
        name: "HTTP".to_string(),
        target_url: args.target.clone(),
        num_requests: args.num_requests,
        concurrent_connections: 1,
        warmup_requests: args.warmup,
    };

    println!("{}:", "Configuration".bright_yellow().bold());
    println!("  Target:       {}", config.target_url);
    println!("  Requests:     {}", config.num_requests);
    println!("  Warmup:       {}", config.warmup_requests);
    println!();

    // Create HTTP client with connection pooling
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true) // For self-signed certs
        .pool_max_idle_per_host(10)
        .build()?;

    // Warmup
    println!("{}", "Warming up...".bright_blue());
    for _ in 0..config.warmup_requests {
        let _ = client.get(format!("{}/health", config.target_url)).send().await;
    }
    println!("{}", "✅ Warmup complete".bright_green());
    println!();

    // Run benchmark
    println!("{}", "🚀 Starting benchmark...".bright_blue().bold());
    println!();

    let mut results = BenchmarkResults::new("HTTP".to_string(), config.num_requests);
    let start_time = Instant::now();

    for i in 0..config.num_requests {
        if i > 0 && i % 1000 == 0 {
            println!(
                "  Progress: {}/{} ({:.1}%)",
                i,
                config.num_requests,
                (i as f64 / config.num_requests as f64) * 100.0
            );
        }

        let client_clone = client.clone();
        let url = format!("{}/health", config.target_url);

        let (result, latency_us) = measure_latency(|| async move {
            client_clone
                .get(&url)
                .send()
                .await?
                .error_for_status()
                .map_err(|e| anyhow::anyhow!(e))?;
            Ok(())
        })
        .await;

        match result {
            Ok(_) => results.record_success(latency_us),
            Err(_) => results.record_failure(),
        }
    }

    let total_duration = start_time.elapsed();
    results.finalize(total_duration);

    // Print results
    results.print_summary();

    // Save results
    let results_json = results.to_json();
    std::fs::write("results_http.json", serde_json::to_string_pretty(&results_json)?)?;
    println!("📊 Results saved to: {}", "results_http.json".bright_cyan());
    println!();

    Ok(())
}
