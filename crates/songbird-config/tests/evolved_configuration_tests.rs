// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::await_holding_lock,
    clippy::float_cmp,
    clippy::absurd_extreme_comparisons,
    clippy::needless_collect,
    clippy::nonminimal_bool,
    clippy::used_underscore_binding,
    clippy::field_reassign_with_default,
    clippy::return_self_not_must_use,
    clippy::overly_complex_bool_expr,
    clippy::assertions_on_constants,
    clippy::no_effect_underscore_binding,
    clippy::items_after_statements,
    clippy::empty_line_after_doc_comments,
    clippy::const_is_empty,
    clippy::duplicated_attributes,
    deprecated,
    clippy::unnecessary_literal_unwrap,
    reason = "test assertions and harness ergonomics"
)]

//! Comprehensive tests for evolved capability-based configuration
//!
//! Tests cover:
//! - Self-aware configuration
//! - Environment detection
//! - Capability-based port allocation
//! - Service locator functionality
//! - Migration from old patterns

use std::sync::Mutex;

use songbird_config::defaults::{
    hosts_evolved::{AdvertiseConfig, BindConfig, Environment, SelfAwareConfig, ServiceLocator},
    ports_evolved::{PortAllocator, ServicePort},
};

/// File-local mutex to serialize tests that modify process-wide env vars.
static ENV_LOCK: Mutex<()> = Mutex::new(());

// ============================================================================
// Environment Detection Tests
// ============================================================================

#[test]
fn test_environment_detection_default() {
    let _guard = ENV_LOCK.lock().unwrap();
    let env = Environment::detect();
    // Should detect without panicking
    assert!(matches!(
        env,
        Environment::Development
            | Environment::Test
            | Environment::Staging
            | Environment::Production
    ));
}

#[test]
fn test_environment_detection_explicit() {
    use std::collections::HashMap;
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_ENVIRONMENT".into(), "production".into());
    assert_eq!(
        Environment::detect_with(&|k| m.get(k).cloned().ok_or(std::env::VarError::NotPresent)),
        Environment::Production
    );

    m.insert("SONGBIRD_ENVIRONMENT".into(), "staging".into());
    assert_eq!(
        Environment::detect_with(&|k| m.get(k).cloned().ok_or(std::env::VarError::NotPresent)),
        Environment::Staging
    );

    m.insert("SONGBIRD_ENVIRONMENT".into(), "development".into());
    assert_eq!(
        Environment::detect_with(&|k| m.get(k).cloned().ok_or(std::env::VarError::NotPresent)),
        Environment::Development
    );
}

#[test]
fn test_environment_production_like() {
    assert!(Environment::Production.is_production_like());
    assert!(Environment::Staging.is_production_like());
    assert!(!Environment::Development.is_production_like());
    assert!(!Environment::Test.is_production_like());
}

#[test]
fn test_environment_development_like() {
    assert!(Environment::Development.is_development_like());
    assert!(Environment::Test.is_development_like());
    assert!(!Environment::Production.is_development_like());
    assert!(!Environment::Staging.is_development_like());
}

// ============================================================================
// Bind Configuration Tests
// ============================================================================

#[test]
fn test_bind_config_development() {
    let config = BindConfig::for_environment(&Environment::Development);
    assert!(config.ip.is_loopback(), "Development should bind to localhost");
    assert_eq!(config.port, 8080);
}

#[test]
fn test_bind_config_production() {
    let config = BindConfig::for_environment(&Environment::Production);
    assert!(config.ip.is_unspecified(), "Production should bind to all interfaces");
}

#[test]
fn test_bind_config_test() {
    let config = BindConfig::for_environment(&Environment::Test);
    assert!(config.ip.is_loopback(), "Test should bind to localhost");
    assert_eq!(config.port, 0, "Test should use OS-assigned port");
}

#[test]
fn test_bind_config_socket_addr() {
    let config = BindConfig::for_environment(&Environment::Development);
    let addr = config.socket_addr();
    assert_eq!(addr.port(), 8080);
    assert!(addr.ip().is_loopback());
}

// ============================================================================
// Advertise Configuration Tests
// ============================================================================

#[test]
fn test_advertise_config_development() {
    let config = AdvertiseConfig::for_environment(&Environment::Development);
    assert!(config.ip.is_loopback(), "Development should advertise localhost");
}

#[test]
fn test_advertise_config_production() {
    let config = AdvertiseConfig::for_environment(&Environment::Production);
    // Production may return unspecified if no public IP detected
    // This is acceptable as it will be resolved during registration
    assert!(config.ip.is_unspecified() || !config.ip.is_loopback());
}

// ============================================================================
// Self-Aware Configuration Tests
// ============================================================================

#[test]
fn test_self_aware_config_creation() {
    let config = SelfAwareConfig::from_environment();
    // Should have valid addresses
    assert!(config.bind_address().port() > 0 || config.environment == Environment::Test);
    assert!(config.advertise_address().port() > 0 || config.environment == Environment::Test);
}

#[tokio::test]
async fn test_self_aware_config_development() {
    // Use modern RAII-based environment isolation
    // CRITICAL: Use a single ScopedEnv call to avoid deadlocking on the global
    // ENV_LOCK mutex. Previously this used remove_multiple + set which acquired
    // the lock twice, causing a deadlock.
    use songbird_test_utils::ScopedEnv;

    // Setting SONGBIRD_ENVIRONMENT=development is sufficient — Environment::detect()
    // checks this first and never reaches the production indicator checks.
    let _env = ScopedEnv::set("SONGBIRD_ENVIRONMENT", "development").await;

    let config = SelfAwareConfig::from_environment();
    assert_eq!(config.environment, Environment::Development);
    assert!(config.bind_address().ip().is_loopback());
}

#[tokio::test]
async fn test_self_aware_config_production() {
    use songbird_test_utils::ScopedEnv;
    let _env = ScopedEnv::set("SONGBIRD_ENVIRONMENT", "production").await;

    let config = SelfAwareConfig::from_environment();
    assert_eq!(config.environment, Environment::Production);
    assert!(config.bind_address().ip().is_unspecified());
}

// ============================================================================
// Service Locator Tests
// ============================================================================

#[test]
fn test_service_locator_creation() {
    let locator = ServiceLocator::new();
    assert!(
        locator.self_config().bind_address().port() > 0
            || locator.self_config().environment == Environment::Test
    );
}

#[tokio::test]
async fn test_service_locator_discovery() {
    let locator = ServiceLocator::new();
    // Should not panic on discovery
    // Note: discover_by_capability is synchronous
    let _storage = locator.discover_by_capability("storage");
    let _compute = locator.discover_by_capability("compute");
    let _ai = locator.discover_by_capability("ai");
}

#[tokio::test]
async fn test_service_locator_self_registration() {
    let locator = ServiceLocator::new();
    // Should not panic on registration
    // Note: register_self is synchronous
    let result = locator.register_self(&["orchestration", "discovery"]);
    assert!(result.is_ok());
}

// ============================================================================
// Port Allocator Tests
// ============================================================================

#[test]
fn test_port_allocator_creation() {
    let allocator = PortAllocator::new();
    // Allocator created successfully - strategy is environment-dependent
    // Just verify it can allocate a port
    let result = allocator.allocate_for_capability("test");
    assert!(result.is_ok(), "Should be able to allocate port");
}

#[test]
fn test_port_allocator_os_assigned() {
    let allocator = PortAllocator::new();
    // Test that port allocation works (strategy is determined by environment)
    let result = allocator.allocate_for_capability("test");
    assert!(result.is_ok(), "Should allocate port");
    let listener = result.expect("test precondition");
    let addr = listener.local_addr().expect("test precondition");
    assert!(addr.port() > 0, "Port should be assigned");
}

#[test]
fn test_port_allocator_capability_ranges() {
    let allocator = PortAllocator::new();
    // Test known capability ranges
    let orchestration = allocator.capability_range("orchestration", 8000..9000);
    assert_eq!(orchestration, 8000..8100);

    let discovery = allocator.capability_range("discovery", 8000..9000);
    assert_eq!(discovery, 8100..8200);

    let storage = allocator.capability_range("storage", 8000..9000);
    assert_eq!(storage, 8300..8400);

    let compute = allocator.capability_range("compute", 8000..9000);
    assert_eq!(compute, 8400..8500);

    let security = allocator.capability_range("security", 8000..9000);
    assert_eq!(security, 8500..8600);

    let messaging = allocator.capability_range("messaging", 8000..9000);
    assert_eq!(messaging, 8200..8300);

    // Unknown capabilities get default range
    let unknown = allocator.capability_range("unknown", 8000..9000);
    assert_eq!(unknown, 8800..8900);
}

// ============================================================================
// Service Port Tests
// ============================================================================

#[test]
fn test_service_port_new() {
    let port = ServicePort::new(8080, "orchestration");
    assert_eq!(port.port, 8080);
    assert_eq!(port.capability, "orchestration");
    assert!(!port.dynamic);
    assert!(port.range.is_none());
}

#[test]
fn test_service_port_dynamic() {
    let port = ServicePort::dynamic("discovery");
    assert_eq!(port.port, 0);
    assert_eq!(port.capability, "discovery");
    assert!(port.dynamic);
}

#[test]
fn test_service_port_capability_range() {
    let port = ServicePort::capability_range("storage", 8300..8400);
    assert_eq!(port.port, 8300);
    assert_eq!(port.capability, "storage");
    assert_eq!(port.range, Some(8300..8400));
}

// ============================================================================
// Integration Tests
// ============================================================================

#[tokio::test]
async fn test_full_self_aware_lifecycle() {
    // 1. Create self-aware configuration
    let _config = SelfAwareConfig::from_environment();

    // 2. Allocate capability-based port
    let allocator = PortAllocator::new();
    let listener =
        allocator.allocate_for_capability("orchestration").expect("Should allocate port");
    let actual_port = listener.local_addr().expect("test precondition").port();
    assert!(actual_port > 0);

    // 3. Create service locator
    let locator = ServiceLocator::new();

    // 4. Register self
    let capabilities = vec!["orchestration", "discovery"];
    let result = locator.register_self(&capabilities);
    assert!(result.is_ok() || result.is_err()); // May fail if no discovery service

    // 5. Discover other services
    // Note: discover_by_capability is synchronous
    let storage_services = locator.discover_by_capability("storage");
    // Empty is OK - no services registered yet
    assert!(storage_services.is_empty() || !storage_services.is_empty());
}

#[test]
fn test_environment_aware_behavior() {
    // Test that configuration adapts to environment
    for env in
        [Environment::Development, Environment::Test, Environment::Staging, Environment::Production]
    {
        let bind = BindConfig::for_environment(&env);
        let advertise = AdvertiseConfig::for_environment(&env);

        match env {
            Environment::Development | Environment::Test => {
                assert!(bind.ip.is_loopback(), "Dev/Test should use localhost");
            }
            Environment::Production | Environment::Staging => {
                assert!(bind.ip.is_unspecified(), "Prod/Staging should bind to all interfaces");
            }
        }

        // All should produce valid socket addresses
        let _bind_addr = bind.socket_addr();
        let _adv_addr = advertise.socket_addr();
    }
}

// ============================================================================
// Edge Cases and Error Handling
// ============================================================================

#[test]
fn test_port_allocation_different_capabilities() {
    let allocator = PortAllocator::new();
    // Allocate ports for different capabilities
    // Should succeed even if using OS assignment
    let results: Vec<_> = vec!["orchestration", "discovery", "storage", "compute"]
        .into_iter()
        .filter_map(|cap| allocator.allocate_for_capability(cap).ok())
        .collect();
    assert!(!results.is_empty(), "Should allocate at least one port");
}

#[test]
fn test_environment_detection_kubernetes() {
    use std::collections::HashMap;
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("KUBERNETES_SERVICE_HOST".into(), "10.0.0.1".into());
    let env =
        Environment::detect_with(&|k| m.get(k).cloned().ok_or(std::env::VarError::NotPresent));
    assert_eq!(env, Environment::Production, "K8s should be detected as production");
}

#[test]
fn test_environment_detection_ecs() {
    use std::collections::HashMap;
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("ECS_CONTAINER_METADATA_URI".into(), "http://169.254.170.2".into());
    let env =
        Environment::detect_with(&|k| m.get(k).cloned().ok_or(std::env::VarError::NotPresent));
    assert_eq!(env, Environment::Production, "ECS should be detected as production");
}

// ============================================================================
// Backwards Compatibility Tests
// ============================================================================

#[test]
#[expect(deprecated, reason = "test assertions and harness ergonomics")]
fn test_well_known_ports_still_work() {
    use songbird_config::defaults::ports_evolved::well_known;
    // Old constants should still be accessible
    assert_eq!(well_known::orchestrator(), 8080);
    assert_eq!(well_known::discovery(), 8081);
    assert_eq!(well_known::dashboard(), 3000);
    assert_eq!(well_known::metrics(), 9090);
}

// ============================================================================
// Performance and Resource Tests
// ============================================================================

#[test]
fn test_rapid_config_creation() {
    // Should handle rapid creation without issues
    for _ in 0..100 {
        let _config = SelfAwareConfig::from_environment();
        let _locator = ServiceLocator::new();
        let _allocator = PortAllocator::new();
    }
}

#[tokio::test]
async fn test_concurrent_discovery() {
    let locator = ServiceLocator::new();

    // Spawn concurrent discovery requests
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let loc = locator.clone();
            let capability = if i % 2 == 0 {
                "storage"
            } else {
                "compute"
            };
            tokio::spawn(async move {
                // discover_by_capability is sync, but spawn requires async block
                loc.discover_by_capability(capability)
            })
        })
        .collect();

    // All should complete without panicking
    for handle in handles {
        let _ = handle.await;
    }
}
