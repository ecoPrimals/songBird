//! Production System Demonstration
//!
//! This example demonstrates the complete production-ready Universal System
//! with service discovery, capability routing, and health monitoring.

use songbird_universal::{
    IntegratedUniversalSystem, UniversalSystemConfig, ServiceDiscoveryConfig,
    CapabilityType, ServiceHealth
};
use tokio;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("🚀 Starting Production Universal System Demo");

    // Create system configuration
    let config = UniversalSystemConfig {
        discovery: ServiceDiscoveryConfig {
            discovery_interval_secs: 30,
            health_check_timeout_secs: 5,
            max_services: 50,
            enable_env_discovery: true,
            enable_network_discovery: false, // Disabled for security
        },
        enable_health_monitoring: true,
        health_check_interval_secs: 30,
        enable_auto_registration: true,
    };

    // Create and initialize the integrated system
    let system = IntegratedUniversalSystem::new(config);
    system.initialize().await?;

    info!("✅ System initialized successfully");

    // Demonstrate manual service registration
    demo_service_registration(&system).await?;

    // Demonstrate capability routing
    demo_capability_routing(&system).await?;

    // Demonstrate system status monitoring
    demo_system_monitoring(&system).await?;

    // Demonstrate concurrent operations
    demo_concurrent_operations(&system).await?;

    info!("🎉 Production System Demo completed successfully!");

    Ok(())
}

/// Demonstrate manual service registration
async fn demo_service_registration(system: &IntegratedUniversalSystem) -> Result<(), Box<dyn std::error::Error>> {
    info!("📝 Demonstrating Service Registration");

    // Register various services
    let services = vec![
        ("beardog-security", "http://localhost:8080", vec!["security", "authentication", "encryption"]),
        ("toadstool-compute", "http://localhost:8081", vec!["compute", "processing", "analysis"]),
        ("nestgate-storage", "http://localhost:8082", vec!["storage", "persistence", "backup"]),
        ("squirrel-ai", "http://localhost:8083", vec!["ai", "ml", "inference", "training"]),
    ];

    for (name, endpoint, capabilities) in services {
        let service_id = system
            .register_service(
                name.to_string(),
                endpoint.to_string(),
                capabilities.iter().map(|s| s.to_string()).collect(),
            )
            .await?;

        info!("✅ Registered service '{}' with ID: {}", name, service_id);
    }

    Ok(())
}

/// Demonstrate capability routing
async fn demo_capability_routing(system: &IntegratedUniversalSystem) -> Result<(), Box<dyn std::error::Error>> {
    info!("🎯 Demonstrating Capability Routing");

    // Test various capability requests
    let test_requests = vec![
        ("security", "encrypt", serde_json::json!({
            "data": "sensitive_information",
            "algorithm": "AES-256-GCM",
            "key_id": "production-key-001"
        })),
        ("compute", "analyze", serde_json::json!({
            "dataset": "performance_metrics",
            "analysis_type": "statistical",
            "parameters": {"confidence": 0.95}
        })),
        ("storage", "store", serde_json::json!({
            "document_id": "doc_12345",
            "content": "important_data",
            "metadata": {"version": 1, "encrypted": true}
        })),
        ("ai", "infer", serde_json::json!({
            "model_id": "production-model-v2",
            "input_data": [1.0, 2.5, 3.8, 4.2],
            "inference_type": "classification"
        })),
    ];

    for (capability, operation, parameters) in test_requests {
        let response = system
            .route_capability_request(capability, operation, &parameters)
            .await?;

        if response.success {
            info!("✅ Successfully routed {} -> {}", capability, operation);
            if let Some(data) = response.data {
                if let Some(service) = data.get("service") {
                    info!("   └─ Handled by service: {}", service.get("name").unwrap_or(&serde_json::Value::String("unknown".to_string())));
                }
            }
        } else {
            info!("❌ Failed to route {} -> {}", capability, operation);
        }
    }

    Ok(())
}

/// Demonstrate system status monitoring
async fn demo_system_monitoring(system: &IntegratedUniversalSystem) -> Result<(), Box<dyn std::error::Error>> {
    info!("📊 Demonstrating System Monitoring");

    let status = system.get_system_status().await?;

    info!("System Status Report:");
    info!("  📈 Total Services: {}", status.total_discovered_services);
    info!("  🎯 Total Capabilities: {}", status.total_capabilities);
    info!("  🟢 Active Providers: {}", status.active_providers);
    info!("  💓 System Health: {:?}", status.system_health);
    
    info!("  Provider Health Summary:");
    for (name, priority) in status.provider_health_summary {
        let health_indicator = match priority {
            1 => "🟢 Healthy",
            2 => "🟡 Degraded", 
            _ => "🔴 Unhealthy",
        };
        info!("    {} - {}", name, health_indicator);
    }

    info!("  Health Distribution:");
    for (health, count) in status.health_distribution {
        info!("    {:?}: {} services", health, count);
    }

    Ok(())
}

/// Demonstrate concurrent operations
async fn demo_concurrent_operations(system: &IntegratedUniversalSystem) -> Result<(), Box<dyn std::error::Error>> {
    info!("⚡ Demonstrating Concurrent Operations");

    let mut handles = vec![];

    // Create 10 concurrent requests
    for i in 0..10 {
        let system_clone = system;
        let handle = tokio::spawn(async move {
            let test_data = serde_json::json!({
                "request_id": format!("concurrent_req_{}", i),
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "payload": format!("test_data_{}", i)
            });

            system_clone
                .route_capability_request("compute", "concurrent_test", &test_data)
                .await
        });
        handles.push(handle);
    }

    // Wait for all requests to complete
    let mut successful = 0;
    let mut failed = 0;

    for (i, handle) in handles.into_iter().enumerate() {
        match handle.await {
            Ok(Ok(response)) => {
                if response.success {
                    successful += 1;
                } else {
                    failed += 1;
                }
            }
            _ => {
                failed += 1;
                info!("❌ Request {} failed", i);
            }
        }
    }

    info!("Concurrent Operations Results:");
    info!("  ✅ Successful: {}", successful);
    info!("  ❌ Failed: {}", failed);
    info!("  📊 Success Rate: {:.1}%", (successful as f64 / (successful + failed) as f64) * 100.0);

    Ok(())
}

/// Demonstrate environment-based service discovery
#[allow(dead_code)]
async fn demo_environment_discovery() -> Result<(), Box<dyn std::error::Error>> {
    info!("🌍 Environment-based Discovery Demo");
    info!("Set these environment variables to test automatic discovery:");
    info!("  export BEARDOG_ENDPOINT=http://localhost:8080");
    info!("  export TOADSTOOL_ENDPOINT=http://localhost:8081");
    info!("  export NESTGATE_ENDPOINT=http://localhost:8082");
    info!("  export SQUIRREL_ENDPOINT=http://localhost:8083");
    info!("");
    info!("Or use generic service patterns:");
    info!("  export SERVICE_1_NAME=custom-service");
    info!("  export SERVICE_1_ENDPOINT=http://localhost:9000");
    info!("  export SERVICE_1_CAPABILITIES=custom,demo,test");

    Ok(())
}

/// Performance benchmarking
#[allow(dead_code)]
async fn benchmark_system_performance(system: &IntegratedUniversalSystem) -> Result<(), Box<dyn std::error::Error>> {
    info!("⚡ Performance Benchmark");

    let start = std::time::Instant::now();
    let mut successful_requests = 0;

    // Run 100 requests and measure performance
    for i in 0..100 {
        let test_data = serde_json::json!({
            "benchmark_id": i,
            "data": format!("benchmark_payload_{}", i)
        });

        if let Ok(response) = system
            .route_capability_request("compute", "benchmark", &test_data)
            .await
        {
            if response.success {
                successful_requests += 1;
            }
        }
    }

    let elapsed = start.elapsed();
    let requests_per_second = (successful_requests as f64) / elapsed.as_secs_f64();

    info!("Performance Results:");
    info!("  📊 Total Requests: 100");
    info!("  ✅ Successful: {}", successful_requests);
    info!("  ⏱️  Total Time: {:.2}s", elapsed.as_secs_f64());
    info!("  🚀 Requests/sec: {:.2}", requests_per_second);
    info!("  ⚡ Avg Latency: {:.2}ms", (elapsed.as_millis() as f64) / 100.0);

    Ok(())
} 