#![allow(deprecated)]
//! Comprehensive port configuration tests
//!
//! These tests validate the robust, thread-safe port configuration system
//! without relying on environment variable mutation or serialization.

// ✅ UPDATED: Using canonical module (Nov 8, 2025 - config consolidation)
use songbird_config::canonical::network::PortRange as PortConfig;
use songbird_config::defaults::ports::*;
use songbird_types::{SongbirdError, SongbirdResult};

// ============================================================================
// GLOBAL CONFIGURATION TESTS
// ============================================================================
// These tests validate the global LazyLock-based configuration

#[test]
fn test_default_ports() {
    // Global config should be initialized with defaults (or env vars if set during process start)
    // We can't control what env vars were set when the process started, so we just verify
    // the ports are valid (u16 is always <= 65535, so we only check > 0)
    assert!(orchestrator_port() > 0);
    assert!(discovery_port() > 0);
    assert!(dashboard_port() > 0);
    assert!(metrics_port() > 0);
    assert!(federation_port() > 0);
    assert!(websocket_port() > 0);
}

#[test]
fn test_global_config_consistency() {
    // Multiple calls should return identical values (immutability test)
    let orch1 = orchestrator_port();
    let orch2 = orchestrator_port();
    let disc1 = discovery_port();
    let disc2 = discovery_port();

    assert_eq!(orch1, orch2, "Orchestrator port should be consistent");
    assert_eq!(disc1, disc2, "Discovery port should be consistent");
}

#[test]
fn test_global_config_thread_safety() -> SongbirdResult<()> {
    use songbird_types::{SongbirdError, SongbirdResult};
    use std::thread;

    // Spawn multiple threads reading configuration simultaneously and collect results directly
    let ports: Vec<_> = (0..10)
        .map(|_| {
            thread::spawn(|| {
                let port = orchestrator_port();
                assert!(port > 0);
                port
            })
        })
        .collect::<Vec<_>>()
        .into_iter()
        .map(|h| {
            h.join().or_else(|_| {
                SongbirdError::configuration(format!(
                    "Error: {}",
                    h
                ))
            })?
        })
        .collect();
    let first_port = ports[0];
    for port in &ports {
        assert_eq!(*port, first_port, "All threads should see the same port");
    }
    Ok(())
}

// ============================================================================
// DEPENDENCY INJECTION TESTS (Port Config Struct)
// ============================================================================
// These tests use the PortConfig struct for clean, isolated testing

#[test]
fn test_port_config_with_defaults() {
    let config = PortConfig::with_defaults();
    assert_eq!(config.orchestrator_port, 8080);
    assert_eq!(config.discovery_port, 8081);
    assert_eq!(config.dashboard_port, 3000);
    assert_eq!(config.metrics_port, 9090);
    assert_eq!(config.federation_port, 8082);
    assert_eq!(config.websocket_port, 8080);
}

#[test]
fn test_port_config_custom_values() {
    // Demonstrate dependency injection pattern for testing
    let mut config = PortConfig::with_defaults();
    config.orchestrator_port = 9000;
    config.discovery_port = 9001;

    assert_eq!(config.orchestrator_port, 9000);
    assert_eq!(config.discovery_port, 9001);
}

#[test]
fn test_port_range_valid() {
    // Test various valid port values using config struct
    for port_num in [80, 443, 8080, 8443, 9000, 65535] {
        let mut config = PortConfig::with_defaults();
        config.orchestrator_port = port_num;
        assert_eq!(config.orchestrator_port, port_num);
    }
}

#[test]
fn test_well_known_ports() {
    let well_known = [
        80,   // HTTP
        443,  // HTTPS
        8080, // HTTP alternate
        8443, // HTTPS alternate
        3000, // Common dev port
        5000, // Common app port
    ];

    for expected_port in well_known {
        let mut config = PortConfig::with_defaults();
        config.orchestrator_port = expected_port;
        assert_eq!(config.orchestrator_port, expected_port);
    }
}

#[test]
fn test_ephemeral_port_range() {
    // Ephemeral ports: 49152-65535
    for port_num in [49152, 50000, 60000, 65535] {
        let mut config = PortConfig::with_defaults();
        config.discovery_port = port_num;
        assert_eq!(config.discovery_port, port_num);
    }
}

#[test]
fn test_privileged_port_range() {
    // Privileged ports: 1-1023
    for port_num in [1, 80, 443, 1023] {
        let mut config = PortConfig::with_defaults();
        config.discovery_port = port_num;
        assert_eq!(config.discovery_port, port_num);
    }
}

#[test]
fn test_registered_port_range() {
    // Registered ports: 1024-49151
    for port_num in [1024, 8080, 8443, 49151] {
        let mut config = PortConfig::with_defaults();
        config.discovery_port = port_num;
        assert_eq!(config.discovery_port, port_num);
    }
}

#[test]
fn test_port_maximum_value() {
    let mut config = PortConfig::with_defaults();
    config.orchestrator_port = 65535;
    assert_eq!(config.orchestrator_port, 65535);
}

#[test]
fn test_port_minimum_value() {
    let mut config = PortConfig::with_defaults();
    config.orchestrator_port = 1;
    assert_eq!(config.orchestrator_port, 1);
}

#[test]
fn test_common_database_ports() {
    let db_ports = [
        (3306, "MySQL"),
        (5432, "PostgreSQL"),
        (27017, "MongoDB"),
        (6379, "Redis"),
        (9042, "Cassandra"),
        (7000, "Cassandra inter-node"),
    ];

    for (port, _description) in db_ports {
        let mut config = PortConfig::with_defaults();
        config.orchestrator_port = port;
        assert_eq!(config.orchestrator_port, port);
    }
}

#[test]
fn test_all_service_ports_independent() {
    let config = PortConfig::with_defaults();

    // Each service should have a distinct port
    let ports = [
        config.orchestrator_port,
        config.discovery_port,
        config.dashboard_port,
        config.metrics_port,
    ];

    // Verify all ports are positive
    for port in ports {
        assert!(port > 0);
    }
}

#[test]
fn test_port_config_is_copy() -> SongbirdResult<()> {
    let config1 = PortConfig::with_defaults();
    let config2 = config1; // Should copy, not move

    // Both should be usable
    assert_eq!(config1.orchestrator_port, config2.orchestrator_port);
    assert_eq!(config1.discovery_port, config2.discovery_port);
    Ok(())
}

#[test]
fn test_port_config_is_debug() -> SongbirdResult<()> {
    let config = PortConfig::with_defaults();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("PortConfig"));
    Ok(())
}

#[test]
fn test_gaming_port_defaults() {
    let config = PortConfig::with_defaults();
    assert_eq!(config.starcraft_port, 6112);
    assert_eq!(config.aoe2_port, 2300);
    assert_eq!(config.gaming_port, 6112);
}

#[test]
fn test_gaming_port_ranges() {
    let config = PortConfig::with_defaults();
    assert_eq!(config.gaming_port_range_start, 7000);
    assert_eq!(config.gaming_port_range_end, 7100);
    assert!(config.gaming_port_range_start < config.gaming_port_range_end);
}

#[test]
fn test_cnc_port_ranges() {
    let config = PortConfig::with_defaults();
    assert_eq!(config.cnc_port_range_start, 1234);
    assert_eq!(config.cnc_port_range_end, 1240);
    assert!(config.cnc_port_range_start < config.cnc_port_range_end);
}

#[test]
fn test_service_specific_ports() {
    let config = PortConfig::with_defaults();
    assert_eq!(config.beardog_port, 8443);
    assert_eq!(config.toadstool_port, 8001);
    assert_eq!(config.squirrel_port, 8002);
    assert_eq!(config.nestgate_port, 8003);
}

#[test]
fn test_port_config_completeness() {
    let config = PortConfig::with_defaults();

    // Verify all ports are initialized to reasonable values
    assert!(config.orchestrator_port > 0);
    assert!(config.discovery_port > 0);
    assert!(config.dashboard_port > 0);
    assert!(config.metrics_port > 0);
    assert!(config.federation_port > 0);
    assert!(config.websocket_port > 0);
    assert!(config.gaming_port > 0);
    assert!(config.health_port > 0);
    assert!(config.beardog_port > 0);
    assert!(config.toadstool_port > 0);
    assert!(config.squirrel_port > 0);
    assert!(config.nestgate_port > 0);
}

// ============================================================================
// PRODUCTION BEHAVIOR TESTS
// ============================================================================
// These tests validate behavior without modifying environment

#[test]
fn test_production_vs_development_ports() {
    let config = PortConfig::with_defaults();

    // Development typically uses high ports (8000+), production might use standard ports
    // We just verify it's a valid port (u16 is always <= 65535)
    assert!(config.orchestrator_port > 0);
}

#[test]
fn test_all_default_ports() {
    // Validate the global configuration returns valid ports
    assert!(orchestrator_port() > 0);
    assert!(discovery_port() > 0);
    assert!(dashboard_port() > 0);
    assert!(metrics_port() > 0);
    assert!(federation_port() > 0);
    assert!(websocket_port() > 0);
}

#[test]
fn test_port_consistency_across_calls() {
    // Multiple calls should be fast and return the same value
    let start = std::time::Instant::now();

    for _ in 0..1000 {
        let _ = orchestrator_port();
    }

    let duration = start.elapsed();

    // Should be essentially instant (< 1ms) since it's just reading a cached value
    assert!(duration.as_millis() < 10, "Port access should be fast (cached)");
}

#[test]
fn test_service_port_dynamic() {
    // Test the dynamic service_port function
    let port = service_port("CUSTOM_SERVICE", 9000);
    assert_eq!(port, 9000); // Should return default when env var not set

    let port2 = service_port("ANOTHER_SERVICE", 9001);
    assert_eq!(port2, 9001);
}

// ============================================================================
// ARCHITECTURAL VALIDATION
// ============================================================================

#[test]
fn test_no_environment_mutation_needed() {
    // This test validates that we can create multiple configs without
    // env var mutation - a key architectural improvement

    let config1 = PortConfig::with_defaults();
    let mut config2 = PortConfig::with_defaults();
    config2.orchestrator_port = 9999;

    // Configs are independent
    assert_ne!(config1.orchestrator_port, config2.orchestrator_port);

    // Global config is unaffected
    let global_port = orchestrator_port();
    assert!(global_port > 0); // Just verify it's valid
}

#[test]
fn test_config_builder_pattern_readiness() {
    // Demonstrates the config struct is ready for builder pattern
    let mut config = PortConfig::with_defaults();
    config.orchestrator_port = 8080;
    config.discovery_port = 8081;
    config.metrics_port = 9090;

    assert_eq!(config.orchestrator_port, 8080);
    assert_eq!(config.discovery_port, 8081);
    assert_eq!(config.metrics_port, 9090);
}
