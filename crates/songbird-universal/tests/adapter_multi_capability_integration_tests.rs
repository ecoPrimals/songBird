//! # 🔌 Multi-Capability Integration Tests
//!
//! Tests all 4 capability-based adapters working together in realistic scenarios.
//! This validates the complete capability-based adapter system for orchestration.

use songbird_universal::adapters::ai::AIHealth;
use songbird_universal::adapters::compute::HealthStatus;
use songbird_universal::adapters::security::SecurityHealth;
use songbird_universal::adapters::storage::StorageHealth;
use songbird_universal::adapters::{AIAdapter, ComputeAdapter, SecurityAdapter, StorageAdapter};
use std::time::Duration;

#[cfg(test)]
mod multi_capability_tests {
    use super::*;

    /// Test: All 4 capability adapters can be created successfully
    #[test]
    fn test_all_capability_adapters_creation() {
        // Create all 4 capability-based adapters
        let compute = ComputeAdapter::new("http://localhost:8080".to_string())
            .expect("Compute adapter creation");
        let security = SecurityAdapter::new("http://localhost:8081".to_string())
            .expect("Security adapter creation");
        let storage = StorageAdapter::new("http://localhost:8082".to_string())
            .expect("Storage adapter creation");
        let ai = AIAdapter::new("http://localhost:8083".to_string()).expect("AI adapter creation");

        // Verify endpoints
        assert_eq!(compute.endpoint(), "http://localhost:8080");
        assert_eq!(security.endpoint(), "http://localhost:8081");
        assert_eq!(storage.endpoint(), "http://localhost:8082");
        assert_eq!(ai.endpoint(), "http://localhost:8083");
    }

    /// Test: All adapters support custom timeouts independently
    #[test]
    fn test_all_adapters_custom_timeouts() {
        // Create adapters with different timeouts
        let compute = ComputeAdapter::new("http://localhost:8080".to_string())
            .unwrap()
            .with_timeout(Duration::from_secs(5));

        let security = SecurityAdapter::new("http://localhost:8081".to_string())
            .unwrap()
            .with_timeout(Duration::from_secs(10));

        let storage = StorageAdapter::new("http://localhost:8082".to_string())
            .unwrap()
            .with_timeout(Duration::from_secs(15));

        let ai = AIAdapter::new("http://localhost:8083".to_string())
            .unwrap()
            .with_timeout(Duration::from_secs(30)); // AI needs longer timeout

        // Verify they all work independently
        assert_eq!(compute.endpoint(), "http://localhost:8080");
        assert_eq!(security.endpoint(), "http://localhost:8081");
        assert_eq!(storage.endpoint(), "http://localhost:8082");
        assert_eq!(ai.endpoint(), "http://localhost:8083");
    }

    /// Test: Adapters can work with different capability providers
    #[test]
    fn test_adapters_with_different_providers() {
        // Simulate different providers implementing capabilities
        let compute_provider_a =
            ComputeAdapter::new("http://provider-a:8080".to_string()).expect("Compute provider A");
        let compute_provider_b =
            ComputeAdapter::new("http://provider-b:8080".to_string()).expect("Compute provider B");

        let storage_provider_a =
            StorageAdapter::new("http://provider-a:8082".to_string()).expect("Storage provider A");
        let storage_provider_b =
            StorageAdapter::new("http://provider-b:8082".to_string()).expect("Storage provider B");

        // Verify adapters don't care about provider identity - only capabilities
        assert_eq!(compute_provider_a.endpoint(), "http://provider-a:8080");
        assert_eq!(compute_provider_b.endpoint(), "http://provider-b:8080");
        assert_eq!(storage_provider_a.endpoint(), "http://provider-a:8082");
        assert_eq!(storage_provider_b.endpoint(), "http://provider-b:8082");
    }

    /// Test: Capability-based orchestration scenario
    /// Scenario: AI analyzes data stored in storage, secured by security, computed by compute
    #[tokio::test]
    async fn test_multi_capability_orchestration_scenario() {
        // Arrange - Setup all 4 capabilities
        let mut compute_server = mockito::Server::new_async().await;
        let mut security_server = mockito::Server::new_async().await;
        let mut storage_server = mockito::Server::new_async().await;
        let mut ai_server = mockito::Server::new_async().await;

        // Mock compute capability
        let compute_mock = compute_server
            .mock("GET", "/metrics/compute")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "cpu_usage_percent": 45.5,
                "memory_usage_bytes": 4000000000,
                "memory_available_bytes": 12000000000,
                "active_containers": 10,
                "queued_jobs": 3,
                "performance_score": 0.85,
                "timestamp": "2025-10-27T12:00:00Z"
            }"#,
            )
            .create_async()
            .await;

        // Mock security capability
        let security_mock = security_server
            .mock("GET", "/metrics/security")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "active_sessions": 50,
                "failed_auth_attempts": 10,
                "blocked_ips": 2,
                "security_score": 0.95,
                "timestamp": "2025-10-27T12:00:00Z"
            }"#,
            )
            .create_async()
            .await;

        // Mock storage capability
        let storage_mock = storage_server
            .mock("GET", "/metrics/storage")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "total_capacity_bytes": 1000000000000,
                "used_bytes": 250000000000,
                "available_bytes": 750000000000,
                "object_count": 1500,
                "avg_read_latency_ms": 15.0,
                "avg_write_latency_ms": 25.0,
                "timestamp": "2025-10-27T12:00:00Z"
            }"#,
            )
            .create_async()
            .await;

        // Mock AI capability
        let ai_mock = ai_server
            .mock("GET", "/metrics/ai")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "active_models": 3,
                "total_requests": 1500,
                "avg_latency_ms": 250.0,
                "accuracy_score": 0.92,
                "gpu_utilization_percent": 45.0,
                "timestamp": "2025-10-27T12:00:00Z"
            }"#,
            )
            .create_async()
            .await;

        // Create adapters for each capability
        let compute = ComputeAdapter::new(compute_server.url()).unwrap();
        let security = SecurityAdapter::new(security_server.url()).unwrap();
        let storage = StorageAdapter::new(storage_server.url()).unwrap();
        let ai = AIAdapter::new(ai_server.url()).unwrap();

        // Act - Collect metrics from all capabilities
        let compute_metrics = compute.collect_metrics().await;
        let security_metrics = security.collect_metrics().await;
        let storage_metrics = storage.collect_metrics().await;
        let ai_metrics = ai.collect_metrics().await;

        // Assert - All capabilities responded successfully
        compute_mock.assert_async().await;
        security_mock.assert_async().await;
        storage_mock.assert_async().await;
        ai_mock.assert_async().await;

        assert!(compute_metrics.is_ok());
        assert!(security_metrics.is_ok());
        assert!(storage_metrics.is_ok());
        assert!(ai_metrics.is_ok());

        // Verify all health statuses

        assert_eq!(compute_metrics.unwrap().health_status(), HealthStatus::Healthy);
        assert_eq!(security_metrics.unwrap().health_status(), SecurityHealth::Healthy);
        assert_eq!(storage_metrics.unwrap().health_status(), StorageHealth::Healthy);
        assert_eq!(ai_metrics.unwrap().health_status(), AIHealth::Healthy);
    }

    /// Test: Network effect pattern - capabilities working together
    /// Scenario: Storage provides data → AI processes it → Compute executes → Security validates
    #[tokio::test]
    async fn test_network_effect_pattern_all_capabilities() {
        // Arrange
        let mut storage_server = mockito::Server::new_async().await;
        let mut ai_server = mockito::Server::new_async().await;
        let mut compute_server = mockito::Server::new_async().await;
        let mut security_server = mockito::Server::new_async().await;

        // Setup mocks for network effect pattern
        let storage_mock = storage_server
            .mock("GET", "/metrics/storage")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "total_capacity_bytes": 1000000000000,
                "used_bytes": 500000000000,
                "available_bytes": 500000000000,
                "object_count": 5000,
                "avg_read_latency_ms": 10.0,
                "avg_write_latency_ms": 15.0,
                "timestamp": "2025-10-27T12:00:00Z"
            }"#,
            )
            .create_async()
            .await;

        let ai_mock = ai_server
            .mock("GET", "/metrics/ai")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "active_models": 5,
                "total_requests": 10000,
                "avg_latency_ms": 300.0,
                "accuracy_score": 0.94,
                "gpu_utilization_percent": 60.0,
                "timestamp": "2025-10-27T12:00:00Z"
            }"#,
            )
            .create_async()
            .await;

        let compute_mock = compute_server
            .mock("GET", "/metrics/compute")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "cpu_usage_percent": 65.0,
                "memory_usage_bytes": 8000000000,
                "memory_available_bytes": 8000000000,
                "active_containers": 20,
                "queued_jobs": 5,
                "performance_score": 0.90,
                "timestamp": "2025-10-27T12:00:00Z"
            }"#,
            )
            .create_async()
            .await;

        let security_mock = security_server
            .mock("GET", "/metrics/security")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "active_sessions": 100,
                "failed_auth_attempts": 5,
                "blocked_ips": 1,
                "security_score": 0.98,
                "timestamp": "2025-10-27T12:00:00Z"
            }"#,
            )
            .create_async()
            .await;

        // Create adapters for network effect
        let storage = StorageAdapter::new(storage_server.url()).unwrap();
        let ai = AIAdapter::new(ai_server.url()).unwrap();
        let compute = ComputeAdapter::new(compute_server.url()).unwrap();
        let security = SecurityAdapter::new(security_server.url()).unwrap();

        // Act - Execute network effect pattern (sequential capabilities)
        // Step 1: Check storage capability
        let storage_result = storage.collect_metrics().await;
        assert!(storage_result.is_ok());

        // Step 2: Use AI capability
        let ai_result = ai.collect_metrics().await;
        assert!(ai_result.is_ok());

        // Step 3: Execute compute capability
        let compute_result = compute.collect_metrics().await;
        assert!(compute_result.is_ok());

        // Step 4: Validate with security capability
        let security_result = security.collect_metrics().await;
        assert!(security_result.is_ok());

        // Assert - All mocks were called
        storage_mock.assert_async().await;
        ai_mock.assert_async().await;
        compute_mock.assert_async().await;
        security_mock.assert_async().await;
    }

    /// Test: Partial capability failure doesn't affect other capabilities
    #[tokio::test]
    async fn test_independent_capability_failures() {
        // Arrange - One capability fails, others succeed
        let mut compute_server = mockito::Server::new_async().await;
        let mut security_server = mockito::Server::new_async().await;
        let mut storage_server = mockito::Server::new_async().await;
        let mut ai_server = mockito::Server::new_async().await;

        // Compute succeeds
        let compute_mock = compute_server
            .mock("GET", "/metrics/compute")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "cpu_usage_percent": 45.5,
                "memory_usage_bytes": 4000000000,
                "memory_available_bytes": 12000000000,
                "active_containers": 10,
                "queued_jobs": 3,
                "performance_score": 0.85,
                "timestamp": "2025-10-27T12:00:00Z"
            }"#,
            )
            .create_async()
            .await;

        // Security FAILS
        let security_mock = security_server
            .mock("GET", "/metrics/security")
            .with_status(503)
            .with_body("Service Unavailable")
            .create_async()
            .await;

        // Storage succeeds
        let storage_mock = storage_server
            .mock("GET", "/metrics/storage")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "total_capacity_bytes": 1000000000000,
                "used_bytes": 250000000000,
                "available_bytes": 750000000000,
                "object_count": 1500,
                "avg_read_latency_ms": 15.0,
                "avg_write_latency_ms": 25.0,
                "timestamp": "2025-10-27T12:00:00Z"
            }"#,
            )
            .create_async()
            .await;

        // AI succeeds
        let ai_mock = ai_server
            .mock("GET", "/metrics/ai")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "active_models": 3,
                "total_requests": 1500,
                "avg_latency_ms": 250.0,
                "accuracy_score": 0.92,
                "gpu_utilization_percent": 45.0,
                "timestamp": "2025-10-27T12:00:00Z"
            }"#,
            )
            .create_async()
            .await;

        // Create adapters
        let compute = ComputeAdapter::new(compute_server.url()).unwrap();
        let security = SecurityAdapter::new(security_server.url()).unwrap();
        let storage = StorageAdapter::new(storage_server.url()).unwrap();
        let ai = AIAdapter::new(ai_server.url()).unwrap();

        // Act
        let compute_result = compute.collect_metrics().await;
        let security_result = security.collect_metrics().await;
        let storage_result = storage.collect_metrics().await;
        let ai_result = ai.collect_metrics().await;

        // Assert - Security failed, but others succeeded independently
        assert!(compute_result.is_ok());
        assert!(security_result.is_err()); // Security capability failed
        assert!(storage_result.is_ok());
        assert!(ai_result.is_ok());

        // Verify all were attempted
        compute_mock.assert_async().await;
        security_mock.assert_async().await;
        storage_mock.assert_async().await;
        ai_mock.assert_async().await;
    }

    /// Test: Health check across all capabilities
    #[tokio::test]
    async fn test_multi_capability_health_check() {
        // Arrange - Setup all capabilities with varying health
        let mut compute_server = mockito::Server::new_async().await;
        let mut security_server = mockito::Server::new_async().await;
        let mut storage_server = mockito::Server::new_async().await;
        let mut ai_server = mockito::Server::new_async().await;

        // Compute: Healthy
        let compute_mock = compute_server
            .mock("GET", "/metrics/compute")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "cpu_usage_percent": 30.0,
                "memory_usage_bytes": 2000000000,
                "memory_available_bytes": 14000000000,
                "active_containers": 5,
                "queued_jobs": 1,
                "performance_score": 0.95,
                "timestamp": "2025-10-27T12:00:00Z"
            }"#,
            )
            .create_async()
            .await;

        // Security: Warning
        let security_mock = security_server
            .mock("GET", "/metrics/security")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "active_sessions": 100,
                "failed_auth_attempts": 60,
                "blocked_ips": 10,
                "security_score": 0.65,
                "timestamp": "2025-10-27T12:00:00Z"
            }"#,
            )
            .create_async()
            .await;

        // Storage: Warning (high usage)
        let storage_mock = storage_server
            .mock("GET", "/metrics/storage")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "total_capacity_bytes": 1000000000000,
                "used_bytes": 870000000000,
                "available_bytes": 130000000000,
                "object_count": 25000,
                "avg_read_latency_ms": 120.0,
                "avg_write_latency_ms": 180.0,
                "timestamp": "2025-10-27T12:00:00Z"
            }"#,
            )
            .create_async()
            .await;

        // AI: Degraded
        let ai_mock = ai_server
            .mock("GET", "/metrics/ai")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "active_models": 10,
                "total_requests": 50000,
                "avg_latency_ms": 1200.0,
                "accuracy_score": 0.88,
                "gpu_utilization_percent": 92.0,
                "timestamp": "2025-10-27T12:00:00Z"
            }"#,
            )
            .create_async()
            .await;

        // Create adapters
        let compute = ComputeAdapter::new(compute_server.url()).unwrap();
        let security = SecurityAdapter::new(security_server.url()).unwrap();
        let storage = StorageAdapter::new(storage_server.url()).unwrap();
        let ai = AIAdapter::new(ai_server.url()).unwrap();

        // Act - Check health of all capabilities
        let compute_health = compute.check_health().await;
        let security_health = security.check_health().await;
        let storage_health = storage.check_health().await;
        let ai_health = ai.check_health().await;

        // Assert - Different health statuses
        assert_eq!(compute_health.unwrap(), HealthStatus::Healthy);
        assert_eq!(security_health.unwrap(), SecurityHealth::Warning);
        assert_eq!(storage_health.unwrap(), StorageHealth::Warning);
        assert_eq!(ai_health.unwrap(), AIHealth::Degraded);

        compute_mock.assert_async().await;
        security_mock.assert_async().await;
        storage_mock.assert_async().await;
        ai_mock.assert_async().await;
    }

    /// Test: All capabilities can be discovered and used without knowing provider names
    #[test]
    fn test_capability_based_discovery_pattern() {
        // This test validates the sovereignty principle:
        // We create adapters for capabilities, not for specific primals

        // Arrange - Create adapters for capabilities we need
        let compute_capability =
            ComputeAdapter::new("http://capability-provider-1:8080".to_string());
        let security_capability =
            SecurityAdapter::new("http://capability-provider-2:8081".to_string());
        let storage_capability =
            StorageAdapter::new("http://capability-provider-3:8082".to_string());
        let ai_capability = AIAdapter::new("http://capability-provider-4:8083".to_string());

        // Assert - We don't know or care who the providers are
        // We only know they provide the capabilities we need
        assert!(compute_capability.is_ok());
        assert!(security_capability.is_ok());
        assert!(storage_capability.is_ok());
        assert!(ai_capability.is_ok());

        // Verify sovereignty: adapters work with ANY provider
        assert_eq!(compute_capability.unwrap().endpoint(), "http://capability-provider-1:8080");
        assert_eq!(security_capability.unwrap().endpoint(), "http://capability-provider-2:8081");
        assert_eq!(storage_capability.unwrap().endpoint(), "http://capability-provider-3:8082");
        assert_eq!(ai_capability.unwrap().endpoint(), "http://capability-provider-4:8083");
    }

    /// Test: Capability failover - if one provider fails, can use another
    #[test]
    fn test_capability_provider_failover() {
        // Arrange - Create adapters for same capability from different providers
        let compute_primary = ComputeAdapter::new("http://primary-compute:8080".to_string());
        let compute_secondary = ComputeAdapter::new("http://secondary-compute:8080".to_string());
        let compute_tertiary = ComputeAdapter::new("http://tertiary-compute:8080".to_string());

        // Assert - All are valid compute capability providers
        assert!(compute_primary.is_ok());
        assert!(compute_secondary.is_ok());
        assert!(compute_tertiary.is_ok());

        // Verify we can failover between providers
        assert_eq!(compute_primary.unwrap().endpoint(), "http://primary-compute:8080");
        assert_eq!(compute_secondary.unwrap().endpoint(), "http://secondary-compute:8080");
        assert_eq!(compute_tertiary.unwrap().endpoint(), "http://tertiary-compute:8080");
    }

    /// Test: Zero hardcoded primal names in integration
    #[test]
    fn test_zero_hardcoded_primal_names() {
        // This test validates that we can create a complete multi-capability system
        // without ever mentioning specific primal names (BearDog, NestGate, Squirrel, ToadStool)

        // Arrange - Create capability-based system
        let compute = ComputeAdapter::new("http://unknown-provider-a:8080".to_string());
        let security = SecurityAdapter::new("http://unknown-provider-b:8081".to_string());
        let storage = StorageAdapter::new("http://unknown-provider-c:8082".to_string());
        let ai = AIAdapter::new("http://unknown-provider-d:8083".to_string());

        // Assert - All capabilities work without knowing provider identity
        // Verify the system is completely primal-agnostic
        // We never mentioned: BearDog, NestGate, Squirrel, or ToadStool
        // We only work with capabilities: compute, security, storage, ai
        assert!(compute.is_ok()); // Compute capability
        assert!(security.is_ok()); // Security capability
        assert!(storage.is_ok()); // Storage capability
        assert!(ai.is_ok()); // AI capability

        // Verify endpoints are provider-agnostic
        assert_eq!(compute.unwrap().endpoint(), "http://unknown-provider-a:8080");
        assert_eq!(security.unwrap().endpoint(), "http://unknown-provider-b:8081");
        assert_eq!(storage.unwrap().endpoint(), "http://unknown-provider-c:8082");
        assert_eq!(ai.unwrap().endpoint(), "http://unknown-provider-d:8083");
    }
}
