//! Metrics Capability Adapter Demonstration
//!
//! This example demonstrates Songbird's capability-based metrics ingestion
//! from other primals like ToadStool, BearDog, NestGate, and Squirrel.
//!
//! ## Architecture
//!
//! Songbird does NOT implement metrics collection directly. Instead, we use
//! capability-based adapters to ingest metrics from specialized primals:
//!
//! - 🍄 ToadStool → Compute metrics (CPU, memory, containers)  
//! - 🐕 BearDog → Security metrics (threats, auth, compliance)
//! - 🏠 NestGate → Storage metrics (disk usage, I/O, capacity)
//! - 🐿️ Squirrel → AI metrics (model inference, training jobs)
//!
//! This maintains clear separation of concerns and leverages each primal's expertise.

use songbird_core::metrics::{HttpMetricsCapabilityAdapter, MetricsCapabilityAdapter};
use songbird_errors::Result;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt().with_env_filter("info").init();

    info!("🎼 Songbird Metrics Capability Adapter Demo");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Create metrics capability adapter with auto-discovery
    let metrics_adapter = create_metrics_adapter().await?;

    // Demonstrate individual metrics collection
    demonstrate_individual_metrics_collection(&metrics_adapter).await;

    // Demonstrate aggregated metrics collection
    demonstrate_aggregated_metrics_collection(&metrics_adapter).await;

    // Demonstrate metrics-aware load balancing
    demonstrate_metrics_aware_load_balancing(metrics_adapter).await;

    info!("✅ Demo completed successfully!");
    Ok(())
}

/// Create a metrics capability adapter with automatic primal discovery
async fn create_metrics_adapter() -> Result<Arc<dyn MetricsCapabilityAdapter>> {
    info!("🔍 Creating metrics capability adapter with auto-discovery...");

    // Try auto-discovery first
    match HttpMetricsCapabilityAdapter::with_auto_discovery().await {
        Ok(adapter) => {
            info!("✅ Auto-discovery successful - found ecosystem primals!");
            Ok(Arc::new(adapter))
        }
        Err(e) => {
            warn!("⚠️  Auto-discovery failed: {}", e);
            info!("🔧 Falling back to manual endpoint configuration...");

            // Create with explicit endpoints for development/testing
            let adapter = HttpMetricsCapabilityAdapter::with_endpoints(
                Some("http://localhost:8082".to_string()),  // ToadStool
                Some("https://localhost:8443".to_string()), // BearDog
                Some("http://localhost:8080".to_string()),  // NestGate
                Some("http://localhost:8083".to_string()),  // Squirrel
            );

            info!("✅ Metrics adapter configured with manual endpoints");
            Ok(Arc::new(adapter))
        }
    }
}

/// Demonstrate collecting metrics from individual primals
async fn demonstrate_individual_metrics_collection(adapter: &Arc<dyn MetricsCapabilityAdapter>) {
    info!("\n📊 Demonstrating Individual Metrics Collection");
    info!("─────────────────────────────────────────────────");

    // Try to collect compute metrics from ToadStool
    info!("🍄 Attempting to collect compute metrics from ToadStool...");
    match adapter.collect_compute_metrics().await {
        Ok(metrics) => {
            info!("✅ ToadStool compute metrics collected:");
            info!("   • CPU Usage: {:.1}%", metrics.cpu_usage_percent);
            info!(
                "   • Memory Used: {} MB",
                metrics.memory_usage_bytes / 1024 / 1024
            );
            info!(
                "   • Memory Available: {} MB",
                metrics.memory_available_bytes / 1024 / 1024
            );
            info!("   • Active Containers: {}", metrics.active_containers);
            info!("   • Queued Jobs: {}", metrics.queued_jobs);
            info!("   • Performance Score: {:.2}", metrics.performance_score);
            info!(
                "   • Zero-Copy Ops/sec: {}",
                metrics.zero_copy_operations_per_sec
            );
        }
        Err(e) => {
            warn!("⚠️  ToadStool metrics unavailable: {}", e);
            info!("   → This is expected if ToadStool is not running");
        }
    }

    // Try to collect security metrics from BearDog
    info!("\n🐕 Attempting to collect security metrics from BearDog...");
    match adapter.collect_security_metrics().await {
        Ok(metrics) => {
            info!("✅ BearDog security metrics collected:");
            info!("   • Threat Level: {:?}", metrics.threat_level);
            info!("   • Active Sessions: {}", metrics.active_sessions);
            info!(
                "   • Failed Auth Attempts: {}",
                metrics.failed_auth_attempts
            );
            info!(
                "   • Encryption Ops/sec: {}",
                metrics.encryption_operations_per_sec
            );
            info!("   • Compliance Score: {:.2}", metrics.compliance_score);
            info!("   • Security Events: {}", metrics.security_events_count);
        }
        Err(e) => {
            warn!("⚠️  BearDog metrics unavailable: {}", e);
            info!("   → This is expected if BearDog is not running");
        }
    }

    // Try to collect storage metrics from NestGate
    info!("\n🏠 Attempting to collect storage metrics from NestGate...");
    match adapter.collect_storage_metrics().await {
        Ok(metrics) => {
            info!("✅ NestGate storage metrics collected:");
            info!(
                "   • Total Capacity: {} GB",
                metrics.total_capacity_bytes / 1024 / 1024 / 1024
            );
            info!(
                "   • Used Capacity: {} GB",
                metrics.used_capacity_bytes / 1024 / 1024 / 1024
            );
            info!(
                "   • Available Capacity: {} GB",
                metrics.available_capacity_bytes / 1024 / 1024 / 1024
            );
            info!("   • Read Ops/sec: {}", metrics.read_ops_per_sec);
            info!("   • Write Ops/sec: {}", metrics.write_ops_per_sec);
            info!("   • Active Backups: {}", metrics.active_backups);
        }
        Err(e) => {
            warn!("⚠️  NestGate metrics unavailable: {}", e);
            info!("   → This is expected if NestGate is not running");
        }
    }

    // Try to collect AI metrics from Squirrel
    info!("\n🐿️ Attempting to collect AI metrics from Squirrel...");
    match adapter.collect_ai_metrics().await {
        Ok(metrics) => {
            info!("✅ Squirrel AI metrics collected:");
            info!(
                "   • Inference Requests/sec: {:.2}",
                metrics.inference_requests_per_sec
            );
            info!(
                "   • Avg Inference Latency: {:.1}ms",
                metrics.average_inference_latency_ms
            );
            info!("   • Active Models: {}", metrics.active_models);
            info!("   • Queued Jobs: {}", metrics.queued_jobs);
            if let Some(gpu) = metrics.gpu_utilization_percent {
                info!("   • GPU Utilization: {:.1}%", gpu);
            }
            info!(
                "   • Active Agent Sessions: {}",
                metrics.active_agent_sessions
            );
            info!(
                "   • Processing Throughput: {:.2}",
                metrics.processing_throughput_score
            );
        }
        Err(e) => {
            warn!("⚠️  Squirrel metrics unavailable: {}", e);
            info!("   → This is expected if Squirrel is not running");
        }
    }
}

/// Demonstrate collecting aggregated metrics from all available primals
async fn demonstrate_aggregated_metrics_collection(adapter: &Arc<dyn MetricsCapabilityAdapter>) {
    info!("\n🌐 Demonstrating Aggregated Metrics Collection");
    info!("────────────────────────────────────────────────");

    info!("🔄 Collecting metrics from all available primals...");
    let start_time = std::time::Instant::now();

    match adapter.collect_all_metrics().await {
        Ok(aggregated) => {
            let duration = start_time.elapsed();
            info!(
                "✅ Aggregated metrics collected in {}ms",
                duration.as_millis()
            );
            info!(
                "   • Collection Duration: {}ms",
                aggregated.collection_duration_ms
            );

            // Report what metrics were successfully collected
            let mut collected_count = 0;
            if aggregated.compute.is_some() {
                info!("   ✓ Compute metrics available");
                collected_count += 1;
            }
            if aggregated.security.is_some() {
                info!("   ✓ Security metrics available");
                collected_count += 1;
            }
            if aggregated.storage.is_some() {
                info!("   ✓ Storage metrics available");
                collected_count += 1;
            }
            if aggregated.ai.is_some() {
                info!("   ✓ AI metrics available");
                collected_count += 1;
            }

            info!("📊 Total metrics sources: {}/4", collected_count);

            if collected_count == 0 {
                info!("💡 No primals are currently running - this is normal in a dev environment");
                info!("   To see real metrics, start ToadStool, BearDog, NestGate, or Squirrel");
            }
        }
        Err(e) => {
            error!("❌ Failed to collect aggregated metrics: {}", e);
        }
    }
}

/// Demonstrate metrics-aware load balancing using real metrics
async fn demonstrate_metrics_aware_load_balancing(
    metrics_adapter: Arc<dyn MetricsCapabilityAdapter>,
) {
    info!("\n⚖️  Demonstrating Metrics-Aware Load Balancing");
    info!("─────────────────────────────────────────────────");

    info!("🏗️ This would integrate with our load balancer to make");
    info!("   routing decisions based on real metrics from ToadStool:");
    info!("   • Route to primals with available CPU");
    info!("   • Avoid overloaded primals");
    info!("   • Consider queue depth for load balancing");
    info!("   • Use ToadStool's performance scores");

    // Simulate periodic metrics collection for load balancing
    info!("\n🔄 Simulating periodic metrics collection...");
    for i in 1..=3 {
        info!("📊 Metrics collection cycle {} of 3", i);

        match metrics_adapter.collect_compute_metrics().await {
            Ok(metrics) => {
                info!(
                    "   → CPU: {:.1}%, Memory: {} MB, Containers: {}, Queue: {}",
                    metrics.cpu_usage_percent,
                    metrics.memory_usage_bytes / 1024 / 1024,
                    metrics.active_containers,
                    metrics.queued_jobs
                );

                // Simulate load balancing decision
                let load_score = calculate_load_score(&metrics);
                info!("   → Load Score: {:.2}/1.0 (lower is better)", load_score);

                if load_score < 0.5 {
                    info!("   ✅ Primal has good capacity - route requests here");
                } else if load_score < 0.8 {
                    info!("   ⚠️  Primal is moderately loaded - route carefully");
                } else {
                    info!("   🚫 Primal is overloaded - avoid routing here");
                }
            }
            Err(_) => {
                info!("   → No compute metrics available (ToadStool not running)");
                info!("   → Would fallback to default routing strategy");
            }
        }

        if i < 3 {
            sleep(Duration::from_secs(2)).await;
        }
    }

    info!("✅ Metrics-aware load balancing demonstration completed");
    info!("💡 In production, this runs continuously to optimize routing");
}

/// Calculate a load score from compute metrics (0.0 = no load, 1.0 = max load)
fn calculate_load_score(metrics: &songbird_core::metrics::ComputeMetrics) -> f64 {
    // Combine multiple factors into a single load score
    let cpu_score = metrics.cpu_usage_percent / 100.0;
    let memory_score = if metrics.memory_available_bytes > 0 {
        metrics.memory_usage_bytes as f64
            / (metrics.memory_usage_bytes + metrics.memory_available_bytes) as f64
    } else {
        1.0 // Assume high load if we can't determine memory usage
    };
    let queue_score = (metrics.queued_jobs as f64 / 50.0).min(1.0); // Normalize to 50 max jobs

    // Weight the factors (CPU and memory are most important)
    (cpu_score * 0.4) + (memory_score * 0.4) + (queue_score * 0.2)
}
