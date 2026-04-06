// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals
//! Real HTTP Integration Tests for Capability-Based Adapters
//!
//! 🍼 MIGRATED: These tests now use capability-based adapters instead of hardcoded primal names
//!
//! These tests verify that adapters can communicate with actual HTTP endpoints.
//! They use mock HTTP servers to simulate capability providers.
//!
//! **Migration Notes**:
//! - BearDogSecurityAdapter → SecurityCapabilityAdapter
//! - NestGateStorageAdapter → StorageCapabilityAdapter  
//! - SquirrelAIAdapter → AiCapabilityAdapter
//! - ToadStoolMetricsAdapter → ComputeCapabilityAdapter

#![cfg(test)]

use songbird_universal::adapters::{
    security::SecurityCapabilityAdapter, storage::StorageCapabilityAdapter,
    ai::AiCapabilityAdapter, compute::ComputeCapabilityAdapter,
};
use std::time::Duration;
// Removed unused: use tokio::time::sleep;

// ⚠️ DEPRECATED: Legacy primal-specific adapters
#[allow(deprecated)]
use songbird_universal::adapters::{
    BearDogSecurityAdapter, NestGateStorageAdapter, SquirrelAIAdapter, ToadStoolMetricsAdapter,
};

/// Test helper to check if a port is available
async fn port_available(port: u16) -> bool {
    tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port))
        .await
        .is_ok()
}

// ============================================================================
// COMPUTE CAPABILITY ADAPTER TESTS (was toadstool)
// ============================================================================

#[tokio::test]
async fn test_compute_adapter_creation_and_config() {
    // 🍼 MIGRATED: Test capability-based compute adapter
    let adapter = ComputeCapabilityAdapter::new("http://localhost:8080".to_string())
        .expect("Adapter creation should succeed");

    assert_eq!(adapter.endpoint(), "http://localhost:8080");

    // Test: Adapter accepts custom timeout
    let adapter_with_timeout = ComputeCapabilityAdapter::new("http://localhost:8080".to_string())
        .expect("Adapter creation should succeed")
        .with_timeout(Duration::from_secs(10));

    assert_eq!(adapter_with_timeout.endpoint(), "http://localhost:8080");
}

// ⚠️ DEPRECATED: Legacy adapter type — backward compatibility coverage
#[tokio::test]
#[allow(deprecated)]
async fn test_backward_compat_legacy_toadstool_metrics_adapter_creation_and_config() {
    let adapter = ToadStoolMetricsAdapter::new("http://localhost:8080".to_string())
        .expect("Adapter creation should succeed");
    assert_eq!(adapter.endpoint(), "http://localhost:8080");
}

#[tokio::test]
async fn test_compute_adapter_network_error_handling() {
    // 🍼 MIGRATED: Test capability-based error handling
    let adapter = ComputeCapabilityAdapter::new("http://localhost:59999".to_string())
        .expect("Adapter creation should succeed");

    let result = adapter.collect_metrics().await;

    // Should return an error (network error), not panic
    assert!(
        result.is_err(),
        "Should return error for unreachable endpoint"
    );
}

// ⚠️ DEPRECATED: Legacy adapter type — backward compatibility coverage
#[tokio::test]
#[allow(deprecated)]
async fn test_backward_compat_legacy_toadstool_metrics_adapter_network_errors() {
    let adapter = ToadStoolMetricsAdapter::new("http://localhost:59999".to_string())
        .expect("Adapter creation should succeed");
    let result = adapter.collect_metrics().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_compute_metrics_validation() {
    // 🍼 MIGRATED: Test capability-based metrics
    use songbird_universal::adapters::compute::ComputeMetrics;

    let metrics = ComputeMetrics {
        cpu_usage_percent: 75.0,
        memory_usage_bytes: 3_000_000_000,     // 3GB
        memory_available_bytes: 5_000_000_000, // 5GB
        active_containers: 10,
        queued_jobs: 5,
        performance_score: 0.85,
        timestamp: chrono::Utc::now(),
    };

    // Validate calculations
    assert_eq!(metrics.total_memory_bytes(), 8_000_000_000);
    assert!((metrics.memory_usage_percent() - 37.5).abs() < 0.1);
    assert!(!metrics.is_high_load());

    use songbird_universal::adapters::compute::HealthStatus;
    assert_eq!(metrics.health_status(), HealthStatus::Healthy);
}

// ⚠️ DEPRECATED: Legacy adapter type — backward compatibility coverage
#[tokio::test]
#[allow(deprecated)]
async fn test_backward_compat_legacy_toadstool_metrics_validation() {
    use songbird_universal::adapters::toadstool::ComputeMetrics;
    let metrics = ComputeMetrics {
        cpu_usage_percent: 75.0,
        memory_usage_bytes: 3_000_000_000,
        memory_available_bytes: 5_000_000_000,
        active_containers: 10,
        queued_jobs: 5,
        performance_score: 0.85,
        timestamp: chrono::Utc::now(),
    };
    assert_eq!(metrics.total_memory_bytes(), 8_000_000_000);
}

// ============================================================================
// SECURITY CAPABILITY ADAPTER TESTS (was beardog)
// ============================================================================

#[tokio::test]
async fn test_security_adapter_creation_and_config() {
    // 🍼 MIGRATED: Test capability-based security adapter
    let adapter = SecurityCapabilityAdapter::new("http://localhost:8081".to_string())
        .expect("Adapter creation should succeed");

    assert_eq!(adapter.endpoint(), "http://localhost:8081");

    // Test: Adapter accepts custom timeout
    let adapter_with_timeout = SecurityCapabilityAdapter::new("http://localhost:8081".to_string())
        .expect("Adapter creation should succeed")
        .with_timeout(Duration::from_secs(15));

    assert_eq!(adapter_with_timeout.endpoint(), "http://localhost:8081");
}

// ⚠️ DEPRECATED: Legacy adapter type — backward compatibility coverage
#[tokio::test]
#[allow(deprecated)]
async fn test_backward_compat_legacy_beardog_security_adapter_creation_and_config() {
    let adapter = BearDogSecurityAdapter::new("http://localhost:8081".to_string())
        .expect("Adapter creation should succeed");
    assert_eq!(adapter.endpoint(), "http://localhost:8081");
}

#[tokio::test]
#[allow(deprecated)]
async fn test_backward_compat_legacy_beardog_security_adapter_network_errors() {
    // Test: Adapter handles unreachable endpoints gracefully
    let adapter = BearDogSecurityAdapter::new("http://localhost:59998".to_string())
        .expect("Adapter creation should succeed");

    let result = adapter.collect_metrics().await;

    // Should return an error (network error), not panic
    assert!(
        result.is_err(),
        "Should return error for unreachable endpoint"
    );
}

#[tokio::test]
async fn test_beardog_security_metrics_validation() {
    // Test: Security metrics structure and calculations
    use songbird_universal::adapters::beardog::{SecurityHealth, SecurityMetrics};

    let metrics = SecurityMetrics {
        active_sessions: 50,
        failed_auth_attempts: 10,
        blocked_ips: 2,
        security_score: 0.95,
        timestamp: chrono::Utc::now(),
    };

    // Validate security status
    assert!(!metrics.is_under_attack());
    assert_eq!(metrics.health_status(), SecurityHealth::Healthy);

    // Test: Under attack detection
    let attacked_metrics = SecurityMetrics {
        active_sessions: 100,
        failed_auth_attempts: 150,
        blocked_ips: 60,
        security_score: 0.45,
        timestamp: chrono::Utc::now(),
    };

    assert!(attacked_metrics.is_under_attack());
    assert_eq!(attacked_metrics.health_status(), SecurityHealth::Critical);
}

#[tokio::test]
#[allow(deprecated)]
async fn test_legacy_beardog_auth_result_types() {
    // Test: AuthResult enum variants
    use songbird_universal::adapters::beardog::AuthResult;

    assert_eq!(AuthResult::Authorized, AuthResult::Authorized);
    assert_ne!(AuthResult::Authorized, AuthResult::Unauthorized);
    assert_eq!(AuthResult::Expired, AuthResult::Expired);
    assert_eq!(AuthResult::Invalid, AuthResult::Invalid);
}

// ============================================================================
// NESTGATE STORAGE ADAPTER TESTS
// ============================================================================

#[tokio::test]
#[allow(deprecated)]
async fn test_backward_compat_legacy_nestgate_storage_adapter_creation_and_config() {
    // Test: Adapter can be created with custom configuration
    let adapter = NestGateStorageAdapter::new("http://localhost:8082".to_string())
        .expect("Adapter creation should succeed");

    assert_eq!(adapter.endpoint(), "http://localhost:8082");

    // Test: Adapter accepts custom timeout
    let adapter_with_timeout = NestGateStorageAdapter::new("http://localhost:8082".to_string())
        .expect("Adapter creation should succeed")
        .with_timeout(Duration::from_secs(10));

    assert_eq!(adapter_with_timeout.endpoint(), "http://localhost:8082");
}

#[tokio::test]
#[allow(deprecated)]
async fn test_backward_compat_legacy_nestgate_storage_adapter_network_errors() {
    // Test: Adapter handles unreachable endpoints gracefully
    let adapter = NestGateStorageAdapter::new("http://localhost:59997".to_string())
        .expect("Adapter creation should succeed");

    let result = adapter.collect_metrics().await;

    // Should return an error (network error), not panic
    assert!(
        result.is_err(),
        "Should return error for unreachable endpoint"
    );
}

#[tokio::test]
#[allow(deprecated)]
async fn test_legacy_nestgate_storage_metrics_validation() {
    // Test: Storage metrics structure and calculations
    use songbird_universal::adapters::nestgate::{StorageHealth, StorageMetrics};

    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000, // 1TB
        used_bytes: 250_000_000_000,             // 250GB
        available_bytes: 750_000_000_000,        // 750GB
        object_count: 1_500,
        avg_read_latency_ms: 15.0,
        avg_write_latency_ms: 25.0,
        timestamp: chrono::Utc::now(),
    };

    // Validate calculations
    assert!((metrics.usage_percent() - 25.0).abs() < 0.1);
    assert!(!metrics.is_nearly_full());
    assert!(!metrics.is_high_latency());
    assert_eq!(metrics.health_status(), StorageHealth::Healthy);

    // Test: Nearly full detection
    let full_metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 960_000_000_000, // 96%
        available_bytes: 40_000_000_000,
        object_count: 50_000,
        avg_read_latency_ms: 20.0,
        avg_write_latency_ms: 600.0,
        timestamp: chrono::Utc::now(),
    };

    assert!(full_metrics.is_nearly_full());
    assert_eq!(full_metrics.health_status(), StorageHealth::Critical);
}

// ============================================================================
// SQUIRREL AI ADAPTER TESTS
// ============================================================================

#[tokio::test]
#[allow(deprecated)]
async fn test_backward_compat_legacy_squirrel_ai_adapter_creation_and_config() {
    // Test: Adapter can be created with custom configuration
    let adapter = SquirrelAIAdapter::new("http://localhost:8083".to_string())
        .expect("Adapter creation should succeed");

    assert_eq!(adapter.endpoint(), "http://localhost:8083");

    // Test: Adapter accepts custom timeout (AI ops may need longer)
    let adapter_with_timeout = SquirrelAIAdapter::new("http://localhost:8083".to_string())
        .expect("Adapter creation should succeed")
        .with_timeout(Duration::from_secs(20));

    assert_eq!(adapter_with_timeout.endpoint(), "http://localhost:8083");
}

#[tokio::test]
#[allow(deprecated)]
async fn test_backward_compat_legacy_squirrel_ai_adapter_network_errors() {
    // Test: Adapter handles unreachable endpoints gracefully
    let adapter = SquirrelAIAdapter::new("http://localhost:59996".to_string())
        .expect("Adapter creation should succeed");

    let result = adapter.collect_metrics().await;

    // Should return an error (network error), not panic
    assert!(
        result.is_err(),
        "Should return error for unreachable endpoint"
    );
}

#[tokio::test]
#[allow(deprecated)]
async fn test_legacy_squirrel_ai_metrics_validation() {
    // Test: AI metrics structure and calculations
    use songbird_universal::adapters::squirrel::{AIHealth, AIMetrics};

    let metrics = AIMetrics {
        active_models: 3,
        total_requests: 1_500,
        avg_latency_ms: 250.0,
        accuracy_score: 0.92,
        gpu_utilization_percent: 45.0,
        timestamp: chrono::Utc::now(),
    };

    // Validate AI status
    assert!(!metrics.is_high_gpu_load());
    assert!(!metrics.is_high_latency());
    assert_eq!(metrics.health_status(), AIHealth::Healthy);

    // Test: Overloaded detection
    let overloaded_metrics = AIMetrics {
        active_models: 20,
        total_requests: 50_000,
        avg_latency_ms: 2500.0,
        accuracy_score: 0.88,
        gpu_utilization_percent: 99.0,
        timestamp: chrono::Utc::now(),
    };

    assert!(overloaded_metrics.is_high_gpu_load());
    assert!(overloaded_metrics.is_high_latency());
    assert_eq!(overloaded_metrics.health_status(), AIHealth::Overloaded);
}

#[tokio::test]
#[allow(deprecated)]
async fn test_legacy_squirrel_model_types() {
    // Test: ModelType enum variants
    use songbird_universal::adapters::squirrel::ModelType;

    assert_eq!(ModelType::Llm, ModelType::Llm);
    assert_ne!(ModelType::Llm, ModelType::Vision);
    assert_eq!(ModelType::Audio, ModelType::Audio);
    assert_eq!(ModelType::Embedding, ModelType::Embedding);
}

// ============================================================================
// CROSS-ADAPTER INTEGRATION TESTS
// ============================================================================

#[tokio::test]
#[allow(deprecated)]
async fn test_all_adapters_can_be_created_simultaneously() {
    // Test: All adapters can coexist and be created together
    let legacy_compute = ToadStoolMetricsAdapter::new("http://localhost:8080".to_string())
        .expect("legacy compute adapter should be created");

    let legacy_security = BearDogSecurityAdapter::new("http://localhost:8081".to_string())
        .expect("legacy security adapter should be created");

    let legacy_storage = NestGateStorageAdapter::new("http://localhost:8082".to_string())
        .expect("legacy storage adapter should be created");

    let legacy_ai = SquirrelAIAdapter::new("http://localhost:8083".to_string())
        .expect("legacy AI adapter should be created");

    // Verify all have correct endpoints
    assert_eq!(legacy_compute.endpoint(), "http://localhost:8080");
    assert_eq!(legacy_security.endpoint(), "http://localhost:8081");
    assert_eq!(legacy_storage.endpoint(), "http://localhost:8082");
    assert_eq!(legacy_ai.endpoint(), "http://localhost:8083");
}

#[tokio::test]
#[allow(deprecated)]
async fn test_adapters_handle_concurrent_failures_gracefully() {
    // Test: Multiple adapters failing simultaneously doesn't cause cascading issues
    let legacy_compute = ToadStoolMetricsAdapter::new("http://localhost:59999".to_string())
        .expect("Adapter creation should succeed");

    let legacy_security = BearDogSecurityAdapter::new("http://localhost:59998".to_string())
        .expect("Adapter creation should succeed");

    let legacy_storage = NestGateStorageAdapter::new("http://localhost:59997".to_string())
        .expect("Adapter creation should succeed");

    let legacy_ai = SquirrelAIAdapter::new("http://localhost:59996".to_string())
        .expect("Adapter creation should succeed");

    // All should fail gracefully (return errors, not panic)
    let results = tokio::join!(
        legacy_compute.collect_metrics(),
        legacy_security.collect_metrics(),
        legacy_storage.collect_metrics(),
        legacy_ai.collect_metrics()
    );

    assert!(results.0.is_err(), "legacy compute adapter should error");
    assert!(results.1.is_err(), "legacy security adapter should error");
    assert!(results.2.is_err(), "legacy storage adapter should error");
    assert!(results.3.is_err(), "legacy AI adapter should error");
}

#[tokio::test]
#[allow(deprecated)]
async fn test_adapter_timeout_behavior() {
    // Test: Adapters respect configured timeouts
    let short_timeout_adapter =
        ToadStoolMetricsAdapter::new("http://localhost:59999".to_string())
            .expect("Adapter creation should succeed")
            .with_timeout(Duration::from_millis(100)); // Very short timeout

    let start = std::time::Instant::now();
    let result = short_timeout_adapter.collect_metrics().await;
    let elapsed = start.elapsed();

    // Should fail quickly (within ~500ms including overhead)
    assert!(result.is_err(), "Should timeout");
    assert!(
        elapsed < Duration::from_secs(1),
        "Should timeout quickly, took {:?}",
        elapsed
    );
}

// ============================================================================
// CAPABILITY TRAIT TESTS
// ============================================================================

#[tokio::test]
#[allow(deprecated)]
async fn test_compute_metrics_provider_trait() {
    // Test: ToadStool implements the ComputeMetricsProvider trait
    use songbird_universal::adapters::toadstool::ComputeMetricsProvider;

    let adapter = ToadStoolMetricsAdapter::new("http://localhost:59999".to_string())
        .expect("Adapter creation should succeed");

    // Should be usable through trait
    let result = adapter.collect_compute_metrics().await;
    assert!(result.is_err(), "Should error for unreachable endpoint");
}

#[tokio::test]
#[allow(deprecated)]
async fn test_security_provider_trait() {
    // Test: BearDog implements the SecurityProvider trait
    use songbird_universal::adapters::beardog::SecurityProvider;

    let adapter = BearDogSecurityAdapter::new("http://localhost:59998".to_string())
        .expect("Adapter creation should succeed");

    // Should be usable through trait
    let result = adapter.collect_security_metrics().await;
    assert!(result.is_err(), "Should error for unreachable endpoint");
}

#[tokio::test]
#[allow(deprecated)]
async fn test_storage_provider_trait() {
    // Test: NestGate implements the StorageProvider trait
    use songbird_universal::adapters::nestgate::StorageProvider;

    let adapter = NestGateStorageAdapter::new("http://localhost:59997".to_string())
        .expect("Adapter creation should succeed");

    // Should be usable through trait
    let result = adapter.collect_storage_metrics().await;
    assert!(result.is_err(), "Should error for unreachable endpoint");
}

#[tokio::test]
#[allow(deprecated)]
async fn test_ai_provider_trait() {
    // Test: Squirrel implements the AIProvider trait
    use songbird_universal::adapters::squirrel::AIProvider;

    let adapter = SquirrelAIAdapter::new("http://localhost:59996".to_string())
        .expect("Adapter creation should succeed");

    // Should be usable through trait
    let result = adapter.collect_ai_metrics().await;
    assert!(result.is_err(), "Should error for unreachable endpoint");
}

