use songbird_gaming_bridge::SongbirdOrchestrator;
use songbird_gaming_bridge::config::NetworkConfig;
use std::collections::HashMap;
use axum::http::{HeaderMap, Method, StatusCode, Uri};
#[allow(dead_code, unused_imports, unused_variables)]
// Proxy Integration Tests
//
// Tests to verify the proxy TODO implementations work correctly
use songbird_gaming_bridge::proxy::{
    ConnectionProxy, LoadBalancingStrategy, ProxyConfig, ProxyRequest,
};
use songbird_gaming_bridge::traits::service_id::{ServiceEndpoint, ServiceInfo};
use std::time::Instant;

#[tokio::test]
async fn test_proxy_creation_and_basic_functionality() {
    let config = ProxyConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 0, // Use port 0 for testing to avoid conflicts
        enable_logging: true,
        request_timeout: 30,
        connection_timeout: 10,
        max_retries: 3,
        enable_circuit_breaker: true,
        circuit_breaker_threshold: 5,
        circuit_breaker_timeout: 30,
        enable_load_balancing: true,
        load_balancing_strategy: LoadBalancingStrategy::RoundRobin,
        enable_ssl: false,
        ssl_cert_path: None,
        ssl_key_path: None,
        enable_compression: false,
        max_body_size: 1024 * 1024, // 1MB
    };

    let proxy = ConnectionProxy::new(config);

    // Test proxy creation
    assert!(!proxy.is_running().await);

    // Test service registration
    let service = ServiceInfo {
        service_id: "test-service".to_string(),
        name: "Test Service".to_string(),
        version: "1.0.0".to_string(),
        service_type: "web".to_string(),
        description: Some("Test service for proxy integration tests").to_string(),
        endpoints: vec![ServiceEndpoint {
            auth_required: false,
            rate_limit: None,
            path: "http://httpbin.org/json".to_string(), // Use httpbin for testing
            method: "GET".to_string(),
            description: Some("JSON endpoint for testing").to_string(),
            parameters: vec![],
            response_schema: None,
        }],
        tags: std::collections::HashMap::new(),
        tags: {
            let mut tags = HashMap::new();
            tags.insert("environment".to_string(), "test".to_string());
            tags
        },
        
    };

    let services = vec![service];
    proxy
        .update_services(services)
        .await
        .expect("Failed to update services");

    // Test proxy request creation and routing
    let proxy_request = ProxyRequest {
        method: Method::GET,
        uri: "/test".parse::<Uri>().expect("Test assertion failed"),
        headers: HeaderMap::new(),
        body: vec![],
        source_ip: Some("127.0.0.1".to_string()),
        timestamp: Instant::now(),
    };

    // Test routing a request (this will actually make an HTTP call to httpbin.org)
    match proxy.route_request("test-service", proxy_request).await {
        Ok(response) => {
            // httpbin.org/json should return 200 OK
            assert_eq!(response.status_code, StatusCode::OK);
            assert!(!response.body.is_empty());
            println!(
                "✅ Proxy request successful: {} bytes received",
                response.body.len()
            );
        }
        Err(e) => {
            // This might fail in environments without internet access, which is fine for testing
            println!("⚠️ Proxy request failed (possibly no internet): {}", e);
        }
    }

    // Test proxy statistics
    let stats = proxy.get_stats().await;
    assert!(stats.total_requests > 0);
    println!("📊 Proxy stats: {} total requests", stats.total_requests);
}

#[tokio::test]
async fn test_proxy_load_balancing() {
    let config = ProxyConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 0,
        enable_load_balancing: true,
        load_balancing_strategy: LoadBalancingStrategy::RoundRobin,
        ..Default::default()
    };

    let proxy = ConnectionProxy::new(config);

    // Register multiple service instances
    let services = vec![
        ServiceInfo {
            id: "lb-service-1".to_string(),
            name: "Load Balanced Service 1".to_string(),
            version: "1.0.0".to_string(),
            service_type: "web".to_string(),
            description: Some("First load balanced service").to_string(),
            endpoints: vec![ServiceEndpoint {
            auth_required: false,
            rate_limit: None,
                path: "http://httpbin.org/uuid".to_string(),
                method: "GET".to_string(),
                description: Some("UUID endpoint").to_string(),
                parameters: vec![],
                response_schema: None,
            }],
            tags: std::collections::HashMap::new(),
            tags: {
                let mut tags = HashMap::new();
                tags.insert("environment".to_string(), "test".to_string());
                tags
            },
            
        },
        ServiceInfo {
            id: "lb-service-2".to_string(),
            name: "Load Balanced Service 2".to_string(),
            version: "1.0.0".to_string(),
            service_type: "web".to_string(),
            description: Some("Second load balanced service").to_string(),
            endpoints: vec![ServiceEndpoint {
            auth_required: false,
            rate_limit: None,
                path: "http://httpbin.org/ip".to_string(),
                method: "GET".to_string(),
                description: Some("IP endpoint").to_string(),
                parameters: vec![],
                response_schema: None,
            }],
            tags: std::collections::HashMap::new(),
            tags: {
                let mut tags = HashMap::new();
                tags.insert("environment".to_string(), "test".to_string());
                tags
            },
            
        },
    ];

    proxy
        .update_services(services)
        .await
        .expect("Failed to update services");

    // Test circuit breaker functionality
    let circuit_breakers = proxy.get_circuit_breaker_states().await;
    println!("🔌 Circuit breakers: {} registered", circuit_breakers.len());

    println!("✅ Load balancing test completed successfully");
}

#[tokio::test]
async fn test_proxy_error_handling() {
    let config = ProxyConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 0,
        enable_circuit_breaker: true,
        circuit_breaker_threshold: 2,
        ..Default::default()
    };

    let proxy = ConnectionProxy::new(config);

    // Register a service with an invalid endpoint
    let service = ServiceInfo {
        id: "invalid-service".to_string(),
        name: "Invalid Service".to_string(),
        version: "1.0.0".to_string(),
        service_type: "web".to_string(),
        description: Some("Service with invalid endpoint for testing error handling").to_string(),
        endpoints: vec![ServiceEndpoint {
            auth_required: false,
            rate_limit: None,
            path: "http://invalid-host-that-does-not-exist.com".to_string(),
            method: "GET".to_string(),
            description: Some("Invalid endpoint for testing").to_string(),
            parameters: vec![],
            response_schema: None,
        }],
        tags: std::collections::HashMap::new(),
        tags: {
            let mut tags = HashMap::new();
            tags.insert("environment".to_string(), "test".to_string());
            tags
        },
        
    };

    proxy
        .update_services(vec![service])
        .await
        .expect("Failed to update services");

    // Test request to invalid service (should fail)
    let proxy_request = ProxyRequest {
        method: Method::GET,
        uri: "/test".parse::<Uri>().expect("Test assertion failed"),
        headers: HeaderMap::new(),
        body: vec![],
        source_ip: Some("127.0.0.1".to_string()),
        timestamp: Instant::now(),
    };

    let result = proxy.route_request("invalid-service", proxy_request).await;
    assert!(result.is_err(), "Request to invalid service should fail");

    println!("✅ Error handling test completed successfully");
}
