//! tarpc Multiplexed Connections Benchmark
//!
//! Measures performance of multiple concurrent tarpc connections
//! This demonstrates Albatross: linear scaling with negligible overhead

mod common;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use common::{print_banner, BenchmarkConfig, BenchmarkResults};
// futures prelude imported for stream operations
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tarpc::{client, context, tokio_serde::formats::Bincode};
use tokio::sync::Semaphore;

#[derive(Parser, Debug)]
#[command(name = "tarpc Multiplexed Connections Benchmark")]
#[command(about = "Benchmark multiplexed tarpc connections")]
struct Args {
    /// Target Songbird instances (comma-separated)
    #[arg(short, long, default_value = "localhost:8091,localhost:8092,localhost:8093")]
    targets: String,

    /// Number of concurrent connections per target
    #[arg(short, long, default_value = "10")]
    connections_per_target: usize,

    /// Number of requests to send (total across all connections)
    #[arg(short, long, default_value = "10000")]
    num_requests: usize,

    /// Warmup requests (not counted)
    #[arg(short, long, default_value = "100")]
    warmup: usize,
}

// Match the tarpc service trait from songbird-orchestrator
#[tarpc::service]
trait SongbirdRpc {
    async fn discover(capability: String) -> Vec<ServiceInfo>;
    async fn discover_all() -> Vec<ServiceInfo>;
    async fn register(registration: ServiceRegistration) -> RegistrationResult;
    async fn unregister(service_id: String) -> RegistrationResult;
    async fn health() -> HealthStatus;
    async fn version() -> VersionInfo;
    async fn protocols() -> Vec<ProtocolInfo>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServiceInfo {
    id: String,
    capability: String,
    endpoint: String,
    status: String,
    metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServiceRegistration {
    service_id: String,
    capability: String,
    endpoint: String,
    metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RegistrationResult {
    success: bool,
    message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HealthStatus {
    status: String,
    version: String,
    uptime_seconds: u64,
    services_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VersionInfo {
    version: String,
    protocol: String,
    capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProtocolInfo {
    name: String,
    port: u16,
    status: String,
    path: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    print_banner("🦅 tarpc MULTIPLEXED BENCHMARK");

    let targets: Vec<String> = args.targets.split(',').map(|s| s.trim().to_string()).collect();
    let total_connections = targets.len() * args.connections_per_target;

    let config = BenchmarkConfig {
        name: "tarpc-multiplex".to_string(),
        target_url: args.targets.clone(),
        num_requests: args.num_requests,
        concurrent_connections: total_connections,
        warmup_requests: args.warmup,
    };

    println!("{}:", "Configuration".bright_yellow().bold());
    println!("  Targets:      {} instances", targets.len());
    println!(
        "  Connections:  {} per target ({} total)",
        args.connections_per_target, total_connections
    );
    println!("  Requests:     {}", config.num_requests);
    println!("  Warmup:       {}", config.warmup_requests);
    println!();

    // Connect to all tarpc servers
    println!("{}", "Connecting to tarpc servers...".bright_blue());

    let mut clients = Vec::new();
    for target in &targets {
        for i in 0..args.connections_per_target {
            let transport = tarpc::serde_transport::tcp::connect(target, Bincode::default)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to connect to {}: {}", target, e))?;

            let client = SongbirdRpcClient::new(client::Config::default(), transport).spawn();
            clients.push(client);

            if (i + 1) % 5 == 0 || i == args.connections_per_target - 1 {
                println!("  {} → {}/{} connections", target, i + 1, args.connections_per_target);
            }
        }
    }

    println!("{}", format!("✅ {} connections established", total_connections).bright_green());
    println!();

    // Warmup
    println!("{}", "Warming up...".bright_blue());
    for client in &clients {
        let _ = client.health(context::current()).await;
    }
    println!("{}", "✅ Warmup complete".bright_green());
    println!();

    // Run benchmark with round-robin across clients
    println!("{}", "🚀 Starting benchmark...".bright_blue().bold());
    println!("   Using round-robin load balancing across {} connections", total_connections);
    println!();

    let mut results =
        BenchmarkResults::new(format!("tarpc ({}x)", total_connections), config.num_requests);
    let start_time = Instant::now();

    // Limit concurrent requests
    let semaphore = Arc::new(Semaphore::new(total_connections));
    let mut tasks = Vec::new();

    for i in 0..config.num_requests {
        if i > 0 && i % 1000 == 0 {
            println!(
                "  Progress: {}/{} ({:.1}%)",
                i,
                config.num_requests,
                (i as f64 / config.num_requests as f64) * 100.0
            );
        }

        // Round-robin client selection
        let client = clients[i % clients.len()].clone();
        let semaphore = semaphore.clone();

        let task = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();

            let start = Instant::now();
            let result = client.health(context::current()).await;
            let latency_us = start.elapsed().as_micros() as u64;

            (result.is_ok(), latency_us)
        });

        tasks.push(task);
    }

    // Collect all results
    for task in tasks {
        let (success, latency_us) = task.await?;
        if success {
            results.record_success(latency_us);
        } else {
            results.record_failure();
        }
    }

    let total_duration = start_time.elapsed();
    results.finalize(total_duration);

    // Print results
    results.print_summary();

    // Print scaling analysis
    println!();
    println!("{}", "═".repeat(70).bright_cyan());
    println!("{}", "  SCALING ANALYSIS".bright_white().bold());
    println!("{}", "═".repeat(70).bright_cyan());
    println!();
    println!("  Connections:        {}", total_connections);
    println!(
        "  Throughput:         {} req/s",
        format!("{:.0}", results.requests_per_second).bright_green().bold()
    );
    println!("  Latency (p50):      {}μs", results.latency_histogram.value_at_quantile(0.50));
    println!("  Latency (p99):      {}μs", results.latency_histogram.value_at_quantile(0.99));
    println!();
    println!(
        "  Per-connection:     {} req/s",
        format!("{:.0}", results.requests_per_second / total_connections as f64).bright_yellow()
    );
    println!();

    // Save results
    let results_json = results.to_json();
    std::fs::write("results_tarpc_multiplex.json", serde_json::to_string_pretty(&results_json)?)?;
    println!("📊 Results saved to: {}", "results_tarpc_multiplex.json".bright_cyan());
    println!();

    Ok(())
}
