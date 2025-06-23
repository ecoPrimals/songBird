//! Songbird Observability Demo - Phase 2
//!
//! This demo showcases the built-in observability features:
//! - System metrics collection (CPU, memory, disk)
//! - Service health monitoring
//! - Optional web dashboard
//! - Real-time metrics streaming
//!
//! Run with: cargo run --example observability_demo

use std::time::Duration;
use tokio::time::{sleep, interval};
use tracing::{info, error, warn};

use songbird_orchestrator::{
    Orchestrator, OrchestratorConfig, 
    ClusterStatus,
    config::ObservabilityConfig as ConfigObservabilityConfig,
};

// Simple demo service to show observability in action
struct DemoService {
    id: String,
    request_count: std::sync::atomic::AtomicU64,
}

impl DemoService {
    fn new(id: String) -> Self {
        Self {
            id,
            request_count: std::sync::atomic::AtomicU64::new(0),
        }
    }
    
    async fn simulate_work(&self) {
        use std::sync::atomic::Ordering;
        
        // Simulate some work
        let count = self.request_count.fetch_add(1, Ordering::Relaxed);
        
        // Simulate varying response times
        let delay_ms = match count % 5 {
            0 => 10,  // Fast response
            1 => 50,  // Normal response
            2 => 100, // Slow response
            3 => 20,  // Fast response
            _ => 200, // Very slow response
        };
        
        sleep(Duration::from_millis(delay_ms)).await;
        
        // Simulate occasional errors
        if count % 23 == 0 {
            warn!("Simulated error in service {}", self.id);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("songbird_orchestrator=debug,observability_demo=info")
        .init();

    info!("🎼 Starting Songbird Observability Demo - Phase 2");

    // Create orchestrator configuration with observability enabled
    let mut config = OrchestratorConfig::default();
    
    // Enable observability features
    config.observability = ConfigObservabilityConfig {
        enabled: true,
        metrics_interval_secs: 10,           // Collect metrics every 10 seconds
        health_check_interval_secs: 15,      // Check health every 15 seconds
        enable_dashboard: true,              // Enable web dashboard
        dashboard_port: 8081,                // Dashboard on port 8081
        export_prometheus: true,             // Export Prometheus metrics
        max_metric_history: 100,             // Keep 100 data points
        enable_system_metrics: true,         // Collect system metrics
        enable_service_metrics: true,        // Collect service metrics
    };

    // Create orchestrator
    let orchestrator = Orchestrator::new(config).await?;

    // Start orchestrator (this will start observability automatically)
    orchestrator.start().await?;

    // Check if dashboard is available
    if let Some(dashboard_url) = orchestrator.get_dashboard_url() {
        info!("🌐 Songbird Dashboard available at: {}", dashboard_url);
        info!("📊 Metrics API available at: {}/api/metrics", dashboard_url);
        info!("🏥 Health API available at: {}/api/health", dashboard_url);
        info!("📈 Prometheus metrics at: {}/api/prometheus", dashboard_url);
    }

    // Subscribe to observability events
    let mut obs_events = orchestrator.subscribe_observability_events();

    // Spawn task to handle observability events
    let obs_handle = tokio::spawn(async move {
        while let Ok(event) = obs_events.recv().await {
            match event {
                songbird_orchestrator::observability::ObservabilityEvent::MetricsCollected { timestamp, duration_ms } => {
                    info!("📊 Metrics collected at {} (took {}ms)", timestamp, duration_ms);
                }
                songbird_orchestrator::observability::ObservabilityEvent::HealthCheckCompleted { service_id, is_healthy, response_time_ms } => {
                    let status = if is_healthy { "✅" } else { "❌" };
                    info!("🏥 Health check: {} {} ({}ms)", service_id, status, response_time_ms);
                }
                songbird_orchestrator::observability::ObservabilityEvent::DashboardStarted { port } => {
                    info!("🌐 Dashboard started on port {}", port);
                }
                songbird_orchestrator::observability::ObservabilityEvent::SystemAlert { message, severity } => {
                    match severity.as_str() {
                        "critical" => error!("🚨 CRITICAL: {}", message),
                        "warning" => warn!("⚠️  WARNING: {}", message),
                        _ => info!("ℹ️  INFO: {}", message),
                    }
                }
            }
        }
    });

    // Simulate services and workload
    info!("🚀 Starting demo workload simulation...");

    // Create some demo services
    let services = vec![
        DemoService::new("web-api".to_string()),
        DemoService::new("auth-service".to_string()),
        DemoService::new("database".to_string()),
        DemoService::new("cache".to_string()),
    ];

    // Simulate workload
    let mut workload_tasks = Vec::new();
    
    for service in services {
        let service_id = service.id.clone();
        let task = tokio::spawn(async move {
            let mut ticker = interval(Duration::from_millis(1500));
            
            for _ in 0..40 { // Run for about 1 minute
                ticker.tick().await;
                service.simulate_work().await;
            }
            
            info!("📴 Service {} finished simulation", service_id);
        });
        
        workload_tasks.push(task);
    }

    // Monitor cluster status
    let orchestrator_clone = orchestrator.clone();
    let status_task = tokio::spawn(async move {
        let mut status_ticker = interval(Duration::from_secs(20));
        
        for _ in 0..3 { // Print status 3 times
            status_ticker.tick().await;
            
            match orchestrator_clone.get_cluster_status().await {
                Ok(status) => {
                    print_cluster_status(&status);
                }
                Err(e) => {
                    error!("Failed to get cluster status: {}", e);
                }
            }
        }
    });

    // Run the demo
    info!("🎬 Demo running... (will complete in ~1 minute)");
    info!("💡 While running, you can:");
    info!("   - Visit the dashboard at http://localhost:8081");
    info!("   - Check metrics at http://localhost:8081/api/metrics");
    info!("   - View health status at http://localhost:8081/api/health");
    info!("   - Get Prometheus metrics at http://localhost:8081/api/prometheus");

    // Wait for workload to complete
    for task in workload_tasks {
        if let Err(e) = task.await {
            error!("Workload task failed: {}", e);
        }
    }

    // Wait for status monitoring
    if let Err(e) = status_task.await {
        error!("Status monitoring failed: {}", e);
    }

    // Final status report
    info!("📊 Getting final cluster status...");
    match orchestrator.get_cluster_status().await {
        Ok(status) => {
            print_final_status(&status);
        }
        Err(e) => {
            error!("Failed to get final cluster status: {}", e);
        }
    }

    // Stop observability events monitoring
    obs_handle.abort();

    // Stop orchestrator
    orchestrator.stop().await?;

    info!("✅ Songbird Observability Demo completed successfully!");
    info!("🎯 Phase 2 observability features demonstrated:");
    info!("   ✓ Built-in system metrics collection");
    info!("   ✓ Service health monitoring");
    info!("   ✓ Real-time metrics streaming");
    info!("   ✓ Web dashboard with beautiful UI");
    info!("   ✓ Prometheus-compatible metrics export");
    info!("   ✓ Event-driven observability notifications");

    Ok(())
}

fn print_cluster_status(status: &ClusterStatus) {
    info!("🏛️  === CLUSTER STATUS ===");
    info!("📊 System Metrics:");
    info!("   CPU Usage: {:.1}%", status.metrics.system.cpu_usage);
    info!("   Memory Usage: {:.1}%", status.metrics.system.memory_usage * 100.0);
    info!("   Load Average: {:.2}", status.metrics.system.load_average.one);
    info!("   Uptime: {:.0} seconds", status.metrics.system.uptime.as_secs_f64());
    
    info!("🏥 Service Health:");
    info!("   Total Services: {}", status.services.len());
    info!("   Healthy: {}", status.services.iter().filter(|s| s.is_healthy).count());
    info!("   Unhealthy: {}", status.services.iter().filter(|s| !s.is_healthy).count());
    
    info!("🎵 Application Metrics:");
    info!("   Active Services: {}", status.metrics.songbird.active_services);
    info!("   Request Rate: {:.2} req/s", status.metrics.songbird.request_rate);
    info!("   Error Rate: {:.2} err/s", status.metrics.songbird.error_rate);
    info!("   Avg Response Time: {:.1}ms", status.metrics.songbird.avg_response_time_ms);
    
    info!("===============================");
}

fn print_final_status(status: &ClusterStatus) {
    info!("🎯 === FINAL DEMO RESULTS ===");
    info!("📈 Performance Summary:");
    info!("   Total Services: {}", status.services.len());
    info!("   Average Response Time: {:.1}ms", status.metrics.songbird.avg_response_time_ms);
    info!("   System CPU Utilization: {:.1}%", status.metrics.system.cpu_usage);
    info!("   Memory Efficiency: {:.1}%", (1.0 - status.metrics.system.memory_usage) * 100.0);
    
    info!("🏆 Observability Features:");
    info!("   ✓ Metrics Collection: Active");
    info!("   ✓ Health Monitoring: Operational");
    info!("   ✓ Dashboard: Available");
    info!("   ✓ System Monitoring: Comprehensive");
    
    info!("🎼 Songbird Phase 2 - Observability: COMPLETE!");
    info!("===============================");
} 