//! Common utilities for Albatross benchmarks

use colored::Colorize;
use hdrhistogram::Histogram;
use std::time::{Duration, Instant};

/// Benchmark configuration
#[derive(Debug, Clone)]
#[allow(dead_code)] // Used by benchmark binaries
pub struct BenchmarkConfig {
    pub name: String,
    pub target_url: String,
    pub num_requests: usize,
    pub concurrent_connections: usize,
    pub warmup_requests: usize,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            name: "Benchmark".to_string(),
            target_url: "http://localhost:8443".to_string(),
            num_requests: 10_000,
            concurrent_connections: 1,
            warmup_requests: 100,
        }
    }
}

/// Benchmark results
#[derive(Debug)]
pub struct BenchmarkResults {
    pub protocol: String,
    pub total_requests: usize,
    pub successful_requests: usize,
    pub failed_requests: usize,
    pub total_duration: Duration,
    pub requests_per_second: f64,
    pub latency_histogram: Histogram<u64>,
}

impl BenchmarkResults {
    pub fn new(protocol: String, total_requests: usize) -> Self {
        Self {
            protocol,
            total_requests,
            successful_requests: 0,
            failed_requests: 0,
            total_duration: Duration::default(),
            requests_per_second: 0.0,
            latency_histogram: Histogram::<u64>::new(3).unwrap(),
        }
    }

    pub fn record_success(&mut self, latency_us: u64) {
        self.successful_requests += 1;
        self.latency_histogram.record(latency_us).ok();
    }

    pub fn record_failure(&mut self) {
        self.failed_requests += 1;
    }

    pub fn finalize(&mut self, duration: Duration) {
        self.total_duration = duration;
        self.requests_per_second = self.successful_requests as f64 / duration.as_secs_f64();
    }

    pub fn print_summary(&self) {
        println!("\n{}", "═".repeat(70).bright_cyan());
        println!(
            "{}",
            format!("  {} BENCHMARK RESULTS", self.protocol.to_uppercase()).bright_white().bold()
        );
        println!("{}", "═".repeat(70).bright_cyan());
        println!();

        // Basic stats
        println!("{}:", "Request Statistics".bright_yellow().bold());
        println!("  Total Requests:     {}", self.total_requests.to_string().bright_white());
        println!(
            "  Successful:         {} ({}%)",
            self.successful_requests.to_string().bright_green(),
            format!(
                "{:.1}",
                (self.successful_requests as f64 / self.total_requests as f64) * 100.0
            )
            .bright_green()
        );
        println!("  Failed:             {}", self.failed_requests.to_string().bright_red());
        println!("  Total Duration:     {:.2}s", self.total_duration.as_secs_f64());
        println!();

        // Throughput
        println!("{}:", "Throughput".bright_yellow().bold());
        println!(
            "  Requests/Second:    {} req/s",
            format!("{:.0}", self.requests_per_second).bright_green().bold()
        );
        println!();

        // Latency percentiles
        println!("{}:", "Latency Percentiles (microseconds)".bright_yellow().bold());
        println!(
            "  Min:                {}μs",
            self.latency_histogram.min().to_string().bright_white()
        );
        println!(
            "  p50 (median):       {}μs",
            self.latency_histogram.value_at_quantile(0.50).to_string().bright_white()
        );
        println!(
            "  p75:                {}μs",
            self.latency_histogram.value_at_quantile(0.75).to_string().bright_white()
        );
        println!(
            "  p90:                {}μs",
            self.latency_histogram.value_at_quantile(0.90).to_string().bright_white()
        );
        println!(
            "  p95:                {}μs",
            self.latency_histogram.value_at_quantile(0.95).to_string().bright_white()
        );
        println!(
            "  p99:                {}μs",
            self.latency_histogram.value_at_quantile(0.99).to_string().bright_white()
        );
        println!(
            "  Max:                {}μs",
            self.latency_histogram.max().to_string().bright_white()
        );
        println!(
            "  Mean:               {}μs",
            format!("{:.0}", self.latency_histogram.mean()).bright_white()
        );
        println!(
            "  Std Dev:            {}μs",
            format!("{:.0}", self.latency_histogram.stdev()).bright_white()
        );
        println!();

        println!("{}", "═".repeat(70).bright_cyan());
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "protocol": self.protocol,
            "total_requests": self.total_requests,
            "successful_requests": self.successful_requests,
            "failed_requests": self.failed_requests,
            "total_duration_secs": self.total_duration.as_secs_f64(),
            "requests_per_second": self.requests_per_second,
            "latency_us": {
                "min": self.latency_histogram.min(),
                "p50": self.latency_histogram.value_at_quantile(0.50),
                "p75": self.latency_histogram.value_at_quantile(0.75),
                "p90": self.latency_histogram.value_at_quantile(0.90),
                "p95": self.latency_histogram.value_at_quantile(0.95),
                "p99": self.latency_histogram.value_at_quantile(0.99),
                "max": self.latency_histogram.max(),
                "mean": self.latency_histogram.mean(),
                "stddev": self.latency_histogram.stdev(),
            }
        })
    }
}

/// Measure latency of an async operation
#[allow(dead_code)] // Utility function for future benchmarks
pub async fn measure_latency<F, Fut, T>(f: F) -> (Result<T, anyhow::Error>, u64)
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, anyhow::Error>>,
{
    let start = Instant::now();
    let result = f().await;
    let latency_us = start.elapsed().as_micros() as u64;
    (result, latency_us)
}

/// Print a centered banner
pub fn print_banner(title: &str) {
    let width = 70;
    let padding = (width - title.len() - 2) / 2;
    println!();
    println!("{}", "╔".to_string() + &"═".repeat(width) + "╗");
    println!(
        "║{}{}{}",
        " ".repeat(padding),
        title.bright_white().bold(),
        " ".repeat(width - padding - title.len()) + "║"
    );
    println!("{}", "╚".to_string() + &"═".repeat(width) + "╝");
    println!();
}
