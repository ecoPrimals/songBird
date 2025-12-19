//! tarpc Single Connection Benchmark
//!
//! Measures performance of a single tarpc connection to Songbird
//! Real implementation using binary protocol

mod common;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use common::{measure_latency, print_banner, BenchmarkConfig, BenchmarkResults};
use futures::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tarpc::{client, context, tokio_serde::formats::Bincode};

#[derive(Parser, Debug)]
#[command(name = "tarpc Single Connection Benchmark")]
#[command(about = "Benchmark single tarpc connection performance")]
struct Args {
    /// Target Songbird tarpc address
    #[arg(short, long, default_value = "localhost:8091")]
    target: String,

    /// Number of requests to send
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

    print_banner("⚡ tarpc SINGLE CONNECTION BENCHMARK");

    let config = BenchmarkConfig {
        name: "tarpc-single".to_string(),
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

    // Connect to tarpc server
    println!("{}", "Connecting to tarpc server...".bright_blue());

    let transport = tarpc::serde_transport::tcp::connect(&config.target_url, Bincode::default);

    // Try to connect with timeout
    let transport = tokio::time::timeout(std::time::Duration::from_secs(5), transport.fuse())
        .await
        .map_err(|_| anyhow::anyhow!("Connection timeout"))?
        .map_err(|e| anyhow::anyhow!("Connection failed: {}", e))?;

    let client = SongbirdRpcClient::new(client::Config::default(), transport).spawn();

    println!("{}", "✅ Connected".bright_green());
    println!();

    // Warmup
    println!("{}", "Warming up...".bright_blue());
    for _ in 0..config.warmup_requests {
        let _ = client.health(context::current()).await;
    }
    println!("{}", "✅ Warmup complete".bright_green());
    println!();

    // Run benchmark
    println!("{}", "🚀 Starting benchmark...".bright_blue().bold());
    println!();

    let mut results = BenchmarkResults::new("tarpc (single)".to_string(), config.num_requests);
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

        let (result, latency_us) = measure_latency(|| async move {
            client_clone.health(context::current()).await.map_err(|e| anyhow::anyhow!(e))?;
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
    std::fs::write("results_tarpc_single.json", serde_json::to_string_pretty(&results_json)?)?;
    println!("📊 Results saved to: {}", "results_tarpc_single.json".bright_cyan());
    println!();

    Ok(())
}
