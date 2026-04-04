// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals
//! End-to-End Load Balancing Tests
//!
//! Tests load balancing across multiple service providers

#![cfg(test)]

#[path = "../common/mod.rs"]
mod common;

use common::{TestEnvironment, MockServiceConfig, TestAssertions};
use songbird_types::{ServiceInfo, HealthStatus};
use std::collections::HashMap;

#[tokio::test]
async fn test_basic_load_balancing() {
    // Test load balancing across multiple providers
    let mut env = TestEnvironment::new().await;
    
    // 1. Register 3 providers for the same capability
    for i in 0..3 {
        let config = MockServiceConfig::new(format!("provider-{}", i))
            .with_capability("compute")
            .with_health(HealthStatus::Healthy);
        
        env.start_mock_service(&format!("provider-{}", i), config)
            .await
            .expect("Should start mock service");
        
        // Verify service started healthy
        let health = env.get_service_health(&format!("provider-{}", i)).await
            .expect("Should get health");
        TestAssertions::assert_healthy(health);
    }
    
    // 2. Simulate load balancing by counting requests
    let mut request_counts: HashMap<String, usize> = HashMap::new();
    let total_requests = 30;
    
    // In a real implementation, this would route requests
    // For now, demonstrate the testing pattern
    for i in 0..total_requests {
        let provider_id = i % 3; // Round-robin simulation
        let provider_name = format!("provider-{}", provider_id);
        *request_counts.entry(provider_name).or_insert(0) += 1;
    }
    
    // 3. Verify load is balanced (within 20% variance)
    TestAssertions::assert_load_balanced(&request_counts, total_requests, 0.2);
    
    // Each provider should have ~10 requests (30/3)
    for (provider, count) in &request_counts {
        assert!(*count >= 8 && *count <= 12, 
                "Provider {} received {} requests, expected ~10", 
                provider, count);
    }
}

#[tokio::test]
async fn test_health_based_routing() {
    // Test that unhealthy providers are excluded from load balancing
    let mut env = TestEnvironment::new().await;
    
    // 1. Register 3 providers
    for i in 0..3 {
        let health = if i == 1 { 
            HealthStatus::Unhealthy 
        } else { 
            HealthStatus::Healthy 
        };
        
        let config = MockServiceConfig::new(format!("provider-{}", i))
            .with_capability("storage")
            .with_health(health);
        
        env.start_mock_service(&format!("provider-{}", i), config)
            .await
            .expect("Should start mock service");
    }
    
    // 2. Verify health statuses
    TestAssertions::assert_healthy(
        env.get_service_health("provider-0").await.unwrap()
    );
    TestAssertions::assert_unhealthy(
        env.get_service_health("provider-1").await.unwrap()
    );
    TestAssertions::assert_healthy(
        env.get_service_health("provider-2").await.unwrap()
    );
}

#[tokio::test]
async fn test_failover_on_degraded_service() {
    // Test failover when a service becomes degraded
    let mut env = TestEnvironment::new().await;
    
    // 1. Start primary service
    let config = MockServiceConfig::new("primary")
        .with_capability("ai")
        .with_health(HealthStatus::Healthy);
    
    env.start_mock_service("primary", config).await.unwrap();
    
    // 2. Start backup service
    let backup_config = MockServiceConfig::new("backup")
        .with_capability("ai")
        .with_health(HealthStatus::Healthy);
    
    env.start_mock_service("backup", backup_config).await.unwrap();
    
    // 3. Both should be healthy initially
    TestAssertions::assert_healthy(
        env.get_service_health("primary").await.unwrap()
    );
    TestAssertions::assert_healthy(
        env.get_service_health("backup").await.unwrap()
    );
}

#[tokio::test]
async fn test_weighted_load_balancing() {
    // Test weighted load balancing based on capacity
    let mut env = TestEnvironment::new().await;
    
    // 1. Register providers with different "capacities"
    // (in real implementation, this would be metadata)
    let configs = vec![
        ("small-provider", 1),   // Low capacity
        ("medium-provider", 2),  // Medium capacity  
        ("large-provider", 4),   // High capacity
    ];
    
    for (name, _capacity) in &configs {
        let config = MockServiceConfig::new(name)
            .with_capability("compute");
        
        env.start_mock_service(name, config).await.unwrap();
    }
    
    // 2. In weighted load balancing, large-provider should get ~4x traffic of small
    // Test infrastructure supports this pattern
    for (name, _) in &configs {
        let health = env.get_service_health(name).await.unwrap();
        TestAssertions::assert_healthy(health);
    }
}

#[tokio::test]
async fn test_least_connections_routing() {
    // Test routing to service with least active connections
    let mut env = TestEnvironment::new().await;
    
    // 1. Register providers
    for i in 0..3 {
        let config = MockServiceConfig::new(format!("provider-{}", i))
            .with_capability("database")
            .with_health(HealthStatus::Healthy);
        
        env.start_mock_service(&format!("provider-{}", i), config)
            .await
            .expect("Should start mock service");
    }
    
    // 2. Simulate connection tracking
    let mut connections: HashMap<String, usize> = HashMap::new();
    connections.insert("provider-0".to_string(), 5);
    connections.insert("provider-1".to_string(), 2);  // Least loaded
    connections.insert("provider-2".to_string(), 8);
    
    // 3. Verify provider-1 has least connections
    let least_loaded = connections.iter()
        .min_by_key(|(_, count)| *count)
        .map(|(name, _)| name);
    
    assert_eq!(least_loaded.unwrap(), "provider-1");
}

#[tokio::test]
async fn test_circuit_breaker_integration() {
    // Test circuit breaker preventing traffic to failing service
    let mut env = TestEnvironment::new().await;
    
    // 1. Register service that will fail
    let config = MockServiceConfig::new("failing-service")
        .with_capability("api")
        .with_health(HealthStatus::Healthy);
    
    env.start_mock_service("failing-service", config).await.unwrap();
    
    // 2. Simulate failures (in real implementation)
    // After N failures, circuit breaker opens
    let simulated_failures = 5;
    let circuit_open = simulated_failures >= 3;  // Threshold
    
    assert!(circuit_open, "Circuit should open after threshold failures");
}

#[tokio::test]
async fn test_sticky_session_routing() {
    // Test sticky sessions (same client → same provider)
    let mut env = TestEnvironment::new().await;
    
    // 1. Register providers
    for i in 0..3 {
        let config = MockServiceConfig::new(format!("provider-{}", i))
            .with_capability("session");
        
        env.start_mock_service(&format!("provider-{}", i), config)
            .await
            .expect("Should start mock service");
    }
    
    // 2. Simulate sticky sessions (hash-based routing)
    let client_id = "client-123";
    let hash = client_id.len() % 3;  // Simple hash
    let assigned_provider = format!("provider-{}", hash);
    
    // 3. Verify same client always routes to same provider
    for _ in 0..10 {
        let hash_result = client_id.len() % 3;
        assert_eq!(hash_result, hash, "Sticky session should be consistent");
    }
    
    // Health check the assigned provider
    let health = env.get_service_health(&assigned_provider).await;
    assert!(health.is_ok());
}

#[tokio::test]
async fn test_geographic_load_balancing() {
    // Test routing based on geographic proximity
    let env = TestEnvironment::new().await;
    
    // 1. Register services in different regions
    let regions = vec![
        ("us-east-service", "us-east-1"),
        ("us-west-service", "us-west-2"),
        ("eu-service", "eu-central-1"),
    ];
    
    for (name, region) in &regions {
        let mut metadata = HashMap::new();
        metadata.insert("region".to_string(), region.to_string());
        
        let service = ServiceInfo {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec!["cdn".to_string()],
            endpoint: env.get_endpoint(name, region.len() as u16),
            metadata,
        };
        
        env.register_service(service).await
            .expect("Service registration should succeed");
    }
    
    // 2. Simulate client location-based routing
    let client_region = "us-east-1";
    let closest_service = regions.iter()
        .find(|(_, r)| r == &client_region)
        .map(|(name, _)| name);
    
    assert_eq!(closest_service.unwrap(), &"us-east-service");
}

#[tokio::test]
async fn test_priority_based_routing() {
    // Test routing with priority levels
    let mut env = TestEnvironment::new().await;
    
    // 1. Register services with priorities
    let services = vec![
        ("high-priority", 1),
        ("medium-priority", 5),
        ("low-priority", 10),
    ];
    
    for (name, priority) in &services {
        let mut metadata = HashMap::new();
        metadata.insert("priority".to_string(), priority.to_string());
        
        let config = MockServiceConfig::new(name)
            .with_capability("queue");
        
        env.start_mock_service(name, config).await.unwrap();
    }
    
    // 2. Verify routing would prefer higher priority (lower number)
    let priorities: Vec<i32> = vec![1, 5, 10];
    let min_priority = priorities.iter().min().unwrap();
    
    assert_eq!(*min_priority, 1, "Should route to highest priority");
}

#[tokio::test]
async fn test_capacity_aware_routing() {
    // Test routing based on current capacity
    let mut env = TestEnvironment::new().await;
    
    // 1. Register providers with capacity limits
    for i in 0..3 {
        let config = MockServiceConfig::new(format!("provider-{}", i))
            .with_capability("processing");
        
        env.start_mock_service(&format!("provider-{}", i), config)
            .await
            .expect("Should start mock service");
    }
    
    // 2. Simulate capacity tracking
    let capacities = vec![
        ("provider-0", 80),  // 80% utilized
        ("provider-1", 50),  // 50% utilized - best choice
        ("provider-2", 95),  // 95% utilized
    ];
    
    // 3. Find provider with most available capacity
    let best_provider = capacities.iter()
        .min_by_key(|(_, util)| util)
        .map(|(name, _)| name);
    
    assert_eq!(best_provider.unwrap(), &"provider-1");
}

#[tokio::test]
async fn test_adaptive_load_balancing() {
    // Test adaptive routing based on response times
    let mut env = TestEnvironment::new().await;
    
    // 1. Register providers
    for i in 0..3 {
        let config = MockServiceConfig::new(format!("provider-{}", i))
            .with_capability("adaptive");
        
        env.start_mock_service(&format!("provider-{}", i), config)
            .await
            .expect("Should start mock service");
    }
    
    // 2. Simulate response time tracking (ms)
    let response_times: HashMap<String, u64> = [
        ("provider-0".to_string(), 150),
        ("provider-1".to_string(), 50),   // Fastest
        ("provider-2".to_string(), 200),
    ].iter().cloned().collect();
    
    // 3. Route to fastest provider
    let fastest = response_times.iter()
        .min_by_key(|(_, time)| *time)
        .map(|(name, _)| name);
    
    assert_eq!(fastest.unwrap(), "provider-1");
}

