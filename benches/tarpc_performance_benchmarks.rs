/// Performance Benchmarks for tarpc vs JSON-RPC vs HTTP
///
/// These benchmarks measure and compare the performance of different
/// protocol implementations to validate the 100x performance claim.
///
/// Version: 0.2.1
/// Last Updated: November 11, 2025

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;

// ============================================================================
// Common Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServiceInfo {
    name: String,
    address: String,
    port: u16,
    capabilities: Vec<String>,
    metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FederationStatus {
    total_services: usize,
    total_peers: usize,
    uptime_seconds: u64,
    version: String,
}

// ============================================================================
// HTTP/REST Benchmark
// ============================================================================

async fn benchmark_http_health_check() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client
        .get("http://localhost:8080/health")
        .timeout(Duration::from_secs(5))
        .send()
        .await?;
    let _ = response.text().await?;
    Ok(())
}

fn bench_http_protocol(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("http_health_check", |b| {
        b.to_async(&rt).iter(|| async {
            let _ = benchmark_http_health_check().await;
        });
    });
}

// ============================================================================
// JSON-RPC Benchmark
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: serde_json::Value,
    id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    result: Option<serde_json::Value>,
    error: Option<serde_json::Value>,
    id: i64,
}

async fn benchmark_jsonrpc_health_check() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "health".to_string(),
        params: serde_json::json!({}),
        id: 1,
    };
    
    let response = client
        .post("http://localhost:8080/jsonrpc")
        .json(&request)
        .timeout(Duration::from_secs(5))
        .send()
        .await?;
    
    let _: JsonRpcResponse = response.json().await?;
    Ok(())
}

fn bench_jsonrpc_protocol(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("jsonrpc_health_check", |b| {
        b.to_async(&rt).iter(|| async {
            let _ = benchmark_jsonrpc_health_check().await;
        });
    });
}

// ============================================================================
// tarpc Benchmark
// ============================================================================

// Note: These would be defined in a separate module in a real implementation
use tarpc::{client, context};
use tarpc::tokio_serde::formats::Bincode;

#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
pub enum ServiceError {
    #[error("Registration failed: {0}")]
    RegistrationFailed(String),
    #[error("Discovery failed: {0}")]
    DiscoveryFailed(String),
    #[error("Status failed: {0}")]
    StatusFailed(String),
}

#[tarpc::service]
pub trait SongbirdFederation {
    async fn health_check() -> Result<bool, ServiceError>;
}

async fn benchmark_tarpc_health_check() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:8091";
    let transport = tarpc::serde_transport::tcp::connect(addr, Bincode::default).await?;
    let client = SongbirdFederationClient::new(client::Config::default(), transport).spawn();
    
    let _ = client.health_check(context::current()).await??;
    Ok(())
}

fn bench_tarpc_protocol(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("tarpc_health_check", |b| {
        b.to_async(&rt).iter(|| async {
            let _ = benchmark_tarpc_health_check().await;
        });
    });
}

// ============================================================================
// Comparative Benchmarks
// ============================================================================

fn bench_protocol_comparison(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("protocol_comparison");
    
    // Configure measurement settings for accurate results
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(100);
    
    group.bench_function(BenchmarkId::new("HTTP", "health_check"), |b| {
        b.to_async(&rt).iter(|| async {
            let _ = benchmark_http_health_check().await;
        });
    });
    
    group.bench_function(BenchmarkId::new("JSON-RPC", "health_check"), |b| {
        b.to_async(&rt).iter(|| async {
            let _ = benchmark_jsonrpc_health_check().await;
        });
    });
    
    group.bench_function(BenchmarkId::new("tarpc", "health_check"), |b| {
        b.to_async(&rt).iter(|| async {
            let _ = benchmark_tarpc_health_check().await;
        });
    });
    
    group.finish();
}

// ============================================================================
// Throughput Benchmarks
// ============================================================================

async fn measure_throughput_http(duration: Duration) -> usize {
    let client = reqwest::Client::new();
    let start = std::time::Instant::now();
    let mut count = 0;
    
    while start.elapsed() < duration {
        if let Ok(_) = benchmark_http_health_check().await {
            count += 1;
        }
    }
    
    count
}

async fn measure_throughput_jsonrpc(duration: Duration) -> usize {
    let start = std::time::Instant::now();
    let mut count = 0;
    
    while start.elapsed() < duration {
        if let Ok(_) = benchmark_jsonrpc_health_check().await {
            count += 1;
        }
    }
    
    count
}

async fn measure_throughput_tarpc(duration: Duration) -> usize {
    let start = std::time::Instant::now();
    let mut count = 0;
    
    while start.elapsed() < duration {
        if let Ok(_) = benchmark_tarpc_health_check().await {
            count += 1;
        }
    }
    
    count
}

fn bench_throughput_comparison(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("throughput_comparison");
    
    // Measure requests per second for each protocol
    let duration = Duration::from_secs(1);
    
    group.bench_function(BenchmarkId::new("HTTP", "requests_per_sec"), |b| {
        b.to_async(&rt).iter(|| async {
            black_box(measure_throughput_http(duration).await)
        });
    });
    
    group.bench_function(BenchmarkId::new("JSON-RPC", "requests_per_sec"), |b| {
        b.to_async(&rt).iter(|| async {
            black_box(measure_throughput_jsonrpc(duration).await)
        });
    });
    
    group.bench_function(BenchmarkId::new("tarpc", "requests_per_sec"), |b| {
        b.to_async(&rt).iter(|| async {
            black_box(measure_throughput_tarpc(duration).await)
        });
    });
    
    group.finish();
}

// ============================================================================
// Concurrent Request Benchmarks
// ============================================================================

async fn benchmark_concurrent_http(concurrency: usize) -> Duration {
    let start = std::time::Instant::now();
    let mut handles = vec![];
    
    for _ in 0..concurrency {
        let handle = tokio::spawn(async move {
            benchmark_http_health_check().await
        });
        handles.push(handle);
    }
    
    for handle in handles {
        let _ = handle.await;
    }
    
    start.elapsed()
}

async fn benchmark_concurrent_jsonrpc(concurrency: usize) -> Duration {
    let start = std::time::Instant::now();
    let mut handles = vec![];
    
    for _ in 0..concurrency {
        let handle = tokio::spawn(async move {
            benchmark_jsonrpc_health_check().await
        });
        handles.push(handle);
    }
    
    for handle in handles {
        let _ = handle.await;
    }
    
    start.elapsed()
}

async fn benchmark_concurrent_tarpc(concurrency: usize) -> Duration {
    let start = std::time::Instant::now();
    let mut handles = vec![];
    
    for _ in 0..concurrency {
        let handle = tokio::spawn(async move {
            benchmark_tarpc_health_check().await
        });
        handles.push(handle);
    }
    
    for handle in handles {
        let _ = handle.await;
    }
    
    start.elapsed()
}

fn bench_concurrent_requests(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("concurrent_requests");
    
    for concurrency in [1, 10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("HTTP", concurrency),
            concurrency,
            |b, &concurrency| {
                b.to_async(&rt).iter(|| async move {
                    black_box(benchmark_concurrent_http(concurrency).await)
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("JSON-RPC", concurrency),
            concurrency,
            |b, &concurrency| {
                b.to_async(&rt).iter(|| async move {
                    black_box(benchmark_concurrent_jsonrpc(concurrency).await)
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("tarpc", concurrency),
            concurrency,
            |b, &concurrency| {
                b.to_async(&rt).iter(|| async move {
                    black_box(benchmark_concurrent_tarpc(concurrency).await)
                });
            },
        );
    }
    
    group.finish();
}

// ============================================================================
// Benchmark Groups
// ============================================================================

criterion_group!(
    benches,
    bench_http_protocol,
    bench_jsonrpc_protocol,
    bench_tarpc_protocol,
    bench_protocol_comparison,
    bench_throughput_comparison,
    bench_concurrent_requests
);

criterion_main!(benches);

