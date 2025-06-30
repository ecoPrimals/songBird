/*!
 * Proxy Demo - Songbird Orchestrator
 *
 * Demonstrates advanced connection proxy tags:
 * - HTTP request routing and forwarding
 * - Load balancing strategies (Round Robin, Random, Least Connections)
 * - Circuit breaker pattern for fault tolerance
 * - Real-time metrics and monitoring
 * - Service registry integration
 * - Request/response transformation
 */

use std::time::Duration;
use tokio::time::sleep;

use axum::http::{HeaderMap, Method};
use songbird_gaming_bridge::{
    prelude::*,
    proxy_types::{ConnectionProxy, LoadBalancingStrategy, ProxyConfig, ProxyRequest},
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info,songbird_orchestrator=debug")
        .init();

    println!("🚀 Songbird Orchestrator - Proxy Demo");
    println!("=====================================");

    // Demo 1: Basic Proxy Setup
    println!("\n📋 Demo 1: Basic Proxy Configuration");
    demo_basic_proxy_setup().await?;

    // Demo 2: Load Balancing Strategies
    println!("\n📋 Demo 2: Load Balancing Strategies");
    demo_load_balancing_strategies().await?;

    // Demo 3: Circuit Breaker Pattern
    println!("\n📋 Demo 3: Circuit Breaker Pattern");
    demo_circuit_breaker().await?;

    // Demo 4: Service Registry Integration
    println!("\n📋 Demo 4: Service Registry Integration");
    demo_service_registry_integration().await?;

    // Demo 5: Metrics and Monitoring
    println!("\n📋 Demo 5: Metrics and Monitoring");
    demo_metrics_monitoring().await?;

    // Demo 6: Advanced Configuration
    println!("\n📋 Demo 6: Advanced Configuration");
    demo_advanced_configuration().await?;

    println!("\n✅ All proxy demos completed successfully!");
    Ok(())
}

/// Demonstrate basic proxy setup and configuration
async fn demo_basic_proxy_setup() -> Result<()> {
    println!("Creating basic connection proxy...");

    let config = ProxyConfig::default();
    let proxy = ConnectionProxy::new(config);

    // Start the proxy
    proxy.start().await?;

    println!("Proxy Configuration:");
    println!("  - Bind Address: 0.0.0.0:{}", 8080);
    println!("  - Request Timeout: 30s");
    println!("  - Circuit Breaker: Enabled");
    println!("  - Load Balancing: Enabled (Round Robin)");
    println!("  - Running: {}", proxy.is_running().await);

    // Stop the proxy
    proxy.stop().await?;

    println!("✓ Basic proxy setup demo completed");
    Ok(())
}

/// Demonstrate different load balancing strategies
async fn demo_load_balancing_strategies() -> Result<()> {
    println!("Testing load balancing strategies...");

    // Create services for load balancing
    let services = create_test_services();

    // Test Round Robin
    println!("\n🔄 Round Robin Load Balancing:");
    let config = ProxyConfig {
        load_balancing_strategy: LoadBalancingStrategy::RoundRobin,
        ..Default::default()
    };
    test_load_balancing_strategy(config, &services, "Round Robin").await?;

    // Test Random
    println!("\n🎲 Random Load Balancing:");
    let config = ProxyConfig {
        load_balancing_strategy: LoadBalancingStrategy::Random,
        ..Default::default()
    };
    test_load_balancing_strategy(config, &services, "Random").await?;

    // Test Least Connections
    println!("\n⚖️ Least Connections Load Balancing:");
    let config = ProxyConfig {
        load_balancing_strategy: LoadBalancingStrategy::LeastConnections,
        ..Default::default()
    };
    test_load_balancing_strategy(config, &services, "Least Connections").await?;

    println!("✓ Load balancing strategies demo completed");
    Ok(())
}

/// Test a specific load balancing strategy
async fn test_load_balancing_strategy(
    config: ProxyConfig,
    services: &[ServiceInfo],
    _strategy_name: &str,
) -> Result<()> {
    let proxy = ConnectionProxy::new(config);
    proxy.start().await?;

    // Update service registry
    proxy.update_services(services.to_vec()).await?;

    // Simulate multiple requests
    for i in 1..=5 {
        let request = create_test_request(&format!("/api/test/{}", i));
        match proxy.route_request("api", request).await {
            Ok(response) => {
                let body = String::from_utf8_lossy(&response.body);
                println!(
                    "  Request {}: {} ({}ms)",
                    i,
                    body,
                    response.response_time.as_millis()
                );
            }
            Err(e) => {
                println!("  Request {}: Error - {}", i, e);
            }
        }
    }

    proxy.stop().await?;
    Ok(())
}

/// Demonstrate circuit breaker pattern
async fn demo_circuit_breaker() -> Result<()> {
    println!("Testing circuit breaker pattern...");

    let config = ProxyConfig {
        enable_circuit_breaker: true,
        circuit_breaker_threshold: 3,
        circuit_breaker_timeout: 5,
        ..Default::default()
    };

    let proxy = ConnectionProxy::new(config);
    proxy.start().await?;

    // Add a test service
    let services = create_test_services();
    proxy.update_services(services).await?;

    println!("Circuit Breaker Configuration:");
    println!("  - Failure Threshold: 3");
    println!("  - Timeout: 5 seconds");
    println!("  - Enabled: true");

    // Simulate requests that would trigger circuit breaker
    println!("\nSimulating requests (some will 'fail' to trigger circuit breaker):");

    for i in 1..=8 {
        let request = create_test_request(&format!("/api/test/{}", i));

        // Simulate failures for requests 2, 3, 4 to trigger circuit breaker
        let service_name = if (2..=4).contains(&i) {
            "failing-service"
        } else {
            "api"
        };

        match proxy.route_request(service_name, request).await {
            Ok(response) => {
                let body = String::from_utf8_lossy(&response.body);
                println!("  Request {}: ✅ {}", i, body);
            }
            Err(e) => {
                println!("  Request {}: ❌ {}", i, e);
            }
        }

        sleep(Duration::from_millis(200)).await;
    }

    // Show circuit breaker states
    let circuit_states = proxy.get_circuit_breaker_states().await;
    println!("\nCircuit Breaker States:");
    for (service, state) in circuit_states {
        println!(
            "  - {}: {:?} (failures: {})",
            service, state.state, state.failure_count
        );
    }

    proxy.stop().await?;

    println!("✓ Circuit breaker demo completed");
    Ok(())
}

/// Demonstrate service registry integration
async fn demo_service_registry_integration() -> Result<()> {
    println!("Testing service registry integration...");

    let proxy = ConnectionProxy::new(ProxyConfig::default());
    proxy.start().await?;

    // Create diverse service portfolio
    let services = vec![
        create_service("api-v1", "API Service v1", "api", 8001),
        create_service("api-v2", "API Service v2", "api", 8002),
        create_service("web-ui", "Web UI", "web", 3000),
        create_service("websocket", "WebSocket Service", "websocket", 8080),
        create_service("database", "Database Proxy", "database", 5432),
    ];

    println!("Registering {} services with proxy:", services.len());
    for service in &services {
        println!(
            "  - {} ({}): {}",
            service.name, service.service_type, service.id
        );
    }

    // Update proxy service registry
    proxy.update_services(services).await?;

    // Test routing to different service types
    let test_routes = vec![
        ("api", "/users"),
        ("web", "/dashboard"),
        ("websocket", "/ws/chat"),
        ("database", "/query"),
    ];

    println!("\nTesting service routing:");
    for (service_type, path) in test_routes {
        let request = create_test_request(path);
        match proxy.route_request(service_type, request).await {
            Ok(response) => {
                let body = String::from_utf8_lossy(&response.body);
                println!("  {} {}: ✅ {}", service_type, path, body);
            }
            Err(e) => {
                println!("  {} {}: ❌ {}", service_type, path, e);
            }
        }
    }

    proxy.stop().await?;

    println!("✓ Service registry integration demo completed");
    Ok(())
}

/// Demonstrate metrics and monitoring
async fn demo_metrics_monitoring() -> Result<()> {
    println!("Testing metrics and monitoring...");

    let proxy = ConnectionProxy::new(ProxyConfig::default());
    proxy.start().await?;

    // Add services
    let services = create_test_services();
    proxy.update_services(services).await?;

    // Generate some traffic for metrics
    println!("Generating traffic for metrics collection...");
    for i in 1..=10 {
        let request = create_test_request(&format!("/api/endpoint/{}", i));
        let _ = proxy.route_request("api", request).await;
        sleep(Duration::from_millis(50)).await;
    }

    // Get and display metrics
    let stats = proxy.get_stats().await;

    println!("\nProxy Metrics:");
    println!("  📊 Total Requests: {}", stats.total_requests);
    println!("  ✅ Successful Requests: {}", stats.successful_requests);
    println!("  ❌ Failed Requests: {}", stats.failed_requests);
    println!("  🔗 Active Connections: {}", stats.active_connections);
    println!(
        "  ⏱️ Avg Response Time: {:.2}ms",
        stats.average_response_time_ms
    );
    println!("  📈 Bytes Transferred: {} bytes", stats.bytes_transferred);
    println!("  🎯 Error Rate: {:.2}%", stats.error_rate);
    println!("  🚀 Requests/Second: {:.2}", stats.requests_per_second);

    proxy.stop().await?;

    println!("✓ Metrics and monitoring demo completed");
    Ok(())
}

/// Demonstrate advanced proxy configuration
async fn demo_advanced_configuration() -> Result<()> {
    println!("Testing advanced proxy configuration...");

    let config = ProxyConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 9090,
        enable_logging: true,
        request_timeout: 60,
        connection_timeout: 15,
        max_retries: 5,
        enable_circuit_breaker: true,
        circuit_breaker_threshold: 10,
        circuit_breaker_timeout: 120,
        enable_load_balancing: true,
        load_balancing_strategy: LoadBalancingStrategy::WeightedRoundRobin,
        enable_ssl: false,
        ssl_cert_path: None,
        ssl_key_path: None,
        enable_compression: true,
        max_body_size: 2 * 1024 * 1024, // 2MB
    };

    println!("Advanced Configuration:");
    println!("  🌐 Bind Address: {}:{}", config.bind_address, config.port);
    println!("  ⏰ Request Timeout: {}s", config.request_timeout);
    println!("  🔌 Connection Timeout: {}s", config.connection_timeout);
    println!("  🔄 Max Retries: {}", config.max_retries);
    println!(
        "  🛡️ Circuit Breaker Threshold: {}",
        config.circuit_breaker_threshold
    );
    println!("  ⚖️ Load Balancing: {:?}", config.load_balancing_strategy);
    println!("  🗜️ Compression: {}", config.enable_compression);
    println!("  📏 Max Body Size: {} bytes", config.max_body_size);

    let proxy = ConnectionProxy::new(config);
    proxy.start().await?;

    println!("  ✅ Advanced proxy started successfully");

    proxy.stop().await?;

    println!("✓ Advanced configuration demo completed");
    Ok(())
}

/// Create test services for demonstrations
fn create_test_services() -> Vec<ServiceInfo> {
    vec![
        create_service("api-1", "API Service Instance 1", "api", 8001),
        create_service("api-2", "API Service Instance 2", "api", 8002),
        create_service("api-3", "API Service Instance 3", "api", 8003),
    ]
}

/// Create a test service
fn create_service(id: &str, name: &str, service_type: &str, port: u16) -> ServiceInfo {
    ServiceInfo {
        id: id.to_string(),
        name: name.to_string(),
        version: "1.0.0".to_string(),
        service_type: service_type.to_string(),
        description: format!("Test service_id: {}", name),
        endpoints: vec![ServiceEndpoint {
            auth_required: false,
            rate_limit: None,
            path: format!("http://localhost:{}", port),
            method: "GET".to_string(),
            description: Some("HTTP endpoint").to_string(),
            parameters: vec![],
            response_schema: None,
        }],
        tags: std::collections::HashMap::new(),
        tags: std::collections::HashMap::new(),
        metadata: std::collections::HashMap::new(),
    }
}

/// Create a test proxy request
fn create_test_request(path: &str) -> ProxyRequest {
    ProxyRequest {
        method: Method::GET,
        uri: path.parse().unwrap_or_else(|_| "/".parse().unwrap()),
        headers: HeaderMap::new(),
        body: Vec::new(),
        source_ip: Some("127.0.0.1".to_string()),
        timestamp: std::time::Instant::now(),
    }
}
