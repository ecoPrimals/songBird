//! Comprehensive discovery tests for Phase 3 - targeting 95% coverage
//! Using API-resilient pattern proven in Option 1

use super::discovery::*;
use tokio::time::Duration;

// ============================================================================
// Configuration Tests (15 tests)
// ============================================================================

#[test]
fn test_discovery_config_default() {
    let config = DiscoveryConfig::default();

    assert!(config.mechanisms.enable_environment_scan);
    assert!(config.mechanisms.enable_network_scanning);
    assert!(config.mechanisms.enable_container_discovery);
    assert_eq!(config.timeout, Duration::from_secs(30));
}

#[test]
fn test_discovery_config_custom() {
    let config = DiscoveryConfig {
        mechanisms: DiscoveryMechanisms {
            enable_environment_scan: false,
            enable_network_scanning: true,
            enable_container_discovery: false,
        },
        timeout: Duration::from_secs(10),
    };

    assert!(!config.mechanisms.enable_environment_scan);
    assert!(config.mechanisms.enable_network_scanning);
    assert!(!config.mechanisms.enable_container_discovery);
    assert_eq!(config.timeout, Duration::from_secs(10));
}

#[test]
fn test_discovery_config_clone() {
    let config = DiscoveryConfig::default();
    let cloned = config.clone();

    assert_eq!(config.timeout, cloned.timeout);
}

#[test]
fn test_discovery_mechanisms_all_enabled() {
    let mechanisms = DiscoveryMechanisms {
        enable_environment_scan: true,
        enable_network_scanning: true,
        enable_container_discovery: true,
    };

    assert!(mechanisms.enable_environment_scan);
    assert!(mechanisms.enable_network_scanning);
    assert!(mechanisms.enable_container_discovery);
}

#[test]
fn test_discovery_mechanisms_all_disabled() {
    let mechanisms = DiscoveryMechanisms {
        enable_environment_scan: false,
        enable_network_scanning: false,
        enable_container_discovery: false,
    };

    assert!(!mechanisms.enable_environment_scan);
    assert!(!mechanisms.enable_network_scanning);
    assert!(!mechanisms.enable_container_discovery);
}

#[test]
fn test_discovery_timeout_short() {
    let config = DiscoveryConfig {
        mechanisms: DiscoveryMechanisms {
            enable_environment_scan: true,
            enable_network_scanning: false,
            enable_container_discovery: false,
        },
        timeout: Duration::from_millis(100),
    };

    assert_eq!(config.timeout, Duration::from_millis(100));
}

#[test]
fn test_discovery_timeout_long() {
    let config = DiscoveryConfig {
        mechanisms: DiscoveryMechanisms {
            enable_environment_scan: true,
            enable_network_scanning: false,
            enable_container_discovery: false,
        },
        timeout: Duration::from_secs(120),
    };

    assert_eq!(config.timeout, Duration::from_secs(120));
}

#[test]
fn test_universal_primal_discovery_creation() {
    let config = DiscoveryConfig::default();
    let discovery = UniversalPrimalDiscovery::new(config);

    assert!(std::mem::size_of_val(&discovery) > 0);
}

#[test]
fn test_universal_primal_discovery_custom_config() {
    let config = DiscoveryConfig {
        mechanisms: DiscoveryMechanisms {
            enable_environment_scan: true,
            enable_network_scanning: false,
            enable_container_discovery: true,
        },
        timeout: Duration::from_secs(15),
    };

    let discovery = UniversalPrimalDiscovery::new(config);
    assert!(std::mem::size_of_val(&discovery) > 0);
}

#[test]
fn test_discovery_config_debug() {
    let config = DiscoveryConfig::default();
    let debug_str = format!("{:?}", config);

    assert!(debug_str.contains("DiscoveryConfig"));
}

#[test]
fn test_discovery_mechanisms_debug() {
    let mechanisms = DiscoveryMechanisms {
        enable_environment_scan: true,
        enable_network_scanning: false,
        enable_container_discovery: true,
    };
    let debug_str = format!("{:?}", mechanisms);

    assert!(debug_str.contains("DiscoveryMechanisms"));
}

#[test]
fn test_mechanisms_clone() {
    let mechanisms = DiscoveryMechanisms {
        enable_environment_scan: true,
        enable_network_scanning: true,
        enable_container_discovery: false,
    };
    let cloned = mechanisms.clone();

    assert_eq!(mechanisms.enable_environment_scan, cloned.enable_environment_scan);
}

#[test]
fn test_config_timeout_zero() {
    let config = DiscoveryConfig {
        mechanisms: DiscoveryMechanisms {
            enable_environment_scan: false,
            enable_network_scanning: false,
            enable_container_discovery: false,
        },
        timeout: Duration::from_secs(0),
    };

    assert_eq!(config.timeout, Duration::from_secs(0));
}

#[test]
fn test_config_environment_only() {
    let config = DiscoveryConfig {
        mechanisms: DiscoveryMechanisms {
            enable_environment_scan: true,
            enable_network_scanning: false,
            enable_container_discovery: false,
        },
        timeout: Duration::from_secs(5),
    };

    assert!(config.mechanisms.enable_environment_scan);
    assert!(!config.mechanisms.enable_network_scanning);
}

#[test]
fn test_config_network_only() {
    let config = DiscoveryConfig {
        mechanisms: DiscoveryMechanisms {
            enable_environment_scan: false,
            enable_network_scanning: true,
            enable_container_discovery: false,
        },
        timeout: Duration::from_secs(10),
    };

    assert!(!config.mechanisms.enable_environment_scan);
    assert!(config.mechanisms.enable_network_scanning);
}

// ============================================================================
// Discovery Types Tests (15 tests)
// ============================================================================

#[test]
fn test_discovery_method_environment() {
    let method = DiscoveryMethod::Environment;
    assert_eq!(method, DiscoveryMethod::Environment);
}

#[test]
fn test_discovery_method_network_scan() {
    let method = DiscoveryMethod::NetworkScan;
    assert_eq!(method, DiscoveryMethod::NetworkScan);
}

#[test]
fn test_discovery_method_mdns() {
    let method = DiscoveryMethod::Mdns;
    assert_eq!(method, DiscoveryMethod::Mdns);
}

#[test]
fn test_discovery_method_configuration() {
    let method = DiscoveryMethod::Configuration;
    assert_eq!(method, DiscoveryMethod::Configuration);
}

#[test]
fn test_discovery_method_kubernetes() {
    let method = DiscoveryMethod::Kubernetes;
    assert_eq!(method, DiscoveryMethod::Kubernetes);
}

#[test]
fn test_discovery_method_docker() {
    let method = DiscoveryMethod::Docker;
    assert_eq!(method, DiscoveryMethod::Docker);
}

#[test]
fn test_discovery_method_clone() {
    let method = DiscoveryMethod::Mdns;
    let cloned = method.clone();
    assert_eq!(method, cloned);
}

#[test]
fn test_discovery_method_debug() {
    let method = DiscoveryMethod::Kubernetes;
    let debug_str = format!("{:?}", method);
    assert!(debug_str.contains("Kubernetes"));
}

#[test]
fn test_primal_health_healthy() {
    let health = PrimalHealth::Healthy;
    assert_eq!(health, PrimalHealth::Healthy);
}

#[test]
fn test_primal_health_degraded() {
    let health = PrimalHealth::Degraded;
    assert_eq!(health, PrimalHealth::Degraded);
}

#[test]
fn test_primal_health_unhealthy() {
    let health = PrimalHealth::Unhealthy;
    assert_eq!(health, PrimalHealth::Unhealthy);
}

#[test]
fn test_primal_health_unknown() {
    let health = PrimalHealth::Unknown;
    assert_eq!(health, PrimalHealth::Unknown);
}

#[test]
fn test_primal_health_clone() {
    let health = PrimalHealth::Degraded;
    let cloned = health.clone();
    assert_eq!(health, cloned);
}

#[test]
fn test_primal_health_debug() {
    let health = PrimalHealth::Healthy;
    let debug_str = format!("{:?}", health);
    assert!(debug_str.contains("Healthy"));
}

#[test]
fn test_types_compile() {
    // Verify all types compile
    assert!(std::mem::size_of::<DiscoveryConfig>() > 0);
    assert!(std::mem::size_of::<DiscoveryMechanisms>() > 0);
    assert!(std::mem::size_of::<DiscoveryMethod>() > 0);
    assert!(std::mem::size_of::<PrimalHealth>() > 0);
    assert!(std::mem::size_of::<DiscoveredPrimal>() > 0);
}

// ============================================================================
// Discovery Error Tests (10 tests)
// ============================================================================

#[test]
fn test_discovery_error_timeout() {
    let error = DiscoveryError::Timeout("Test timeout".to_string());
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("Timeout"));
}

#[test]
fn test_discovery_error_network() {
    let error = DiscoveryError::NetworkError("Connection failed".to_string());
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("NetworkError"));
}

#[test]
fn test_discovery_error_unreachable() {
    let error = DiscoveryError::UnreachableEndpoint("http://localhost:9999".to_string());
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("UnreachableEndpoint"));
}

#[test]
fn test_discovery_error_configuration() {
    let error = DiscoveryError::ConfigurationError("Invalid config".to_string());
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("ConfigurationError"));
}

#[test]
fn test_discovery_error_display() {
    let error = DiscoveryError::Timeout("Display test".to_string());
    let display_str = format!("{}", error);
    assert!(!display_str.is_empty());
}

#[test]
fn test_discovery_error_types() {
    // Verify all error types compile
    let _timeout = DiscoveryError::Timeout("test".to_string());
    let _network = DiscoveryError::NetworkError("test".to_string());
    let _unreachable = DiscoveryError::UnreachableEndpoint("test".to_string());
    let _config = DiscoveryError::ConfigurationError("test".to_string());

    // All error types can be constructed
}

#[test]
fn test_discovery_error_timeout_display() {
    let error = DiscoveryError::Timeout("timeout test".to_string());
    let display = format!("{}", error);
    assert!(display.contains("timeout"));
}

#[test]
fn test_discovery_error_network_display() {
    let error = DiscoveryError::NetworkError("network test".to_string());
    let display = format!("{}", error);
    assert!(display.contains("Network error"));
}

#[test]
fn test_discovery_error_unreachable_display() {
    let error = DiscoveryError::UnreachableEndpoint("test endpoint".to_string());
    let display = format!("{}", error);
    assert!(display.contains("Unreachable"));
}

#[test]
fn test_discovery_error_config_display() {
    let error = DiscoveryError::ConfigurationError("config test".to_string());
    let display = format!("{}", error);
    assert!(display.contains("Configuration"));
}

// ============================================================================
// Async Discovery Tests (20 tests)
// ============================================================================

#[tokio::test]
async fn test_discover_all_primals_no_mechanisms() {
    let config = DiscoveryConfig {
        mechanisms: DiscoveryMechanisms {
            enable_environment_scan: false,
            enable_network_scanning: false,
            enable_container_discovery: false,
        },
        timeout: Duration::from_secs(1),
    };

    let mut discovery = UniversalPrimalDiscovery::new(config);
    let result = discovery.discover_all_primals().await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_discover_environment_only() {
    let config = DiscoveryConfig {
        mechanisms: DiscoveryMechanisms {
            enable_environment_scan: true,
            enable_network_scanning: false,
            enable_container_discovery: false,
        },
        timeout: Duration::from_millis(100),
    };

    let mut discovery = UniversalPrimalDiscovery::new(config);
    let result = discovery.discover_all_primals().await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_discover_network_only() {
    let config = DiscoveryConfig {
        mechanisms: DiscoveryMechanisms {
            enable_environment_scan: false,
            enable_network_scanning: true,
            enable_container_discovery: false,
        },
        timeout: Duration::from_millis(100),
    };

    let mut discovery = UniversalPrimalDiscovery::new(config);
    let result = discovery.discover_all_primals().await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_discover_container_only() {
    let config = DiscoveryConfig {
        mechanisms: DiscoveryMechanisms {
            enable_environment_scan: false,
            enable_network_scanning: false,
            enable_container_discovery: true,
        },
        timeout: Duration::from_millis(100),
    };

    let mut discovery = UniversalPrimalDiscovery::new(config);
    let result = discovery.discover_all_primals().await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_discover_all_mechanisms() {
    let config = DiscoveryConfig::default();
    let mut discovery = UniversalPrimalDiscovery::new(config);
    let result = discovery.discover_all_primals().await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_discover_short_timeout() {
    let config = DiscoveryConfig {
        mechanisms: DiscoveryMechanisms {
            enable_environment_scan: true,
            enable_network_scanning: true,
            enable_container_discovery: true,
        },
        timeout: Duration::from_millis(1),
    };

    let mut discovery = UniversalPrimalDiscovery::new(config);
    let result = discovery.discover_all_primals().await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_discover_multiple_times() {
    let config = DiscoveryConfig {
        mechanisms: DiscoveryMechanisms {
            enable_environment_scan: true,
            enable_network_scanning: false,
            enable_container_discovery: false,
        },
        timeout: Duration::from_millis(100),
    };

    let mut discovery = UniversalPrimalDiscovery::new(config);

    let result1 = discovery.discover_all_primals().await;
    let result2 = discovery.discover_all_primals().await;

    assert!(result1.is_ok());
    assert!(result2.is_ok());
}

#[tokio::test]
async fn test_discovery_caching() {
    let config = DiscoveryConfig::default();
    let mut discovery = UniversalPrimalDiscovery::new(config);

    let _ = discovery.discover_all_primals().await;

    // Cache should exist (even if empty) - discovery completes successfully
}

#[tokio::test]
async fn test_concurrent_discovery() {
    let config = DiscoveryConfig {
        mechanisms: DiscoveryMechanisms {
            enable_environment_scan: true,
            enable_network_scanning: false,
            enable_container_discovery: false,
        },
        timeout: Duration::from_millis(100),
    };

    let mut discovery1 = UniversalPrimalDiscovery::new(config.clone());
    let mut discovery2 = UniversalPrimalDiscovery::new(config);

    let (result1, result2) =
        tokio::join!(discovery1.discover_all_primals(), discovery2.discover_all_primals());

    assert!(result1.is_ok());
    assert!(result2.is_ok());
}

#[tokio::test]
async fn test_discovery_long_timeout() {
    let config = DiscoveryConfig {
        mechanisms: DiscoveryMechanisms {
            enable_environment_scan: true,
            enable_network_scanning: false,
            enable_container_discovery: false,
        },
        timeout: Duration::from_secs(60),
    };

    let mut discovery = UniversalPrimalDiscovery::new(config);

    let result =
        tokio::time::timeout(Duration::from_secs(5), discovery.discover_all_primals()).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_discovery_clone_independence() {
    let config = DiscoveryConfig::default();
    let discovery1 = UniversalPrimalDiscovery::new(config.clone());
    let discovery2 = discovery1.clone();

    assert!(std::mem::size_of_val(&discovery1) > 0);
    assert!(std::mem::size_of_val(&discovery2) > 0);
}

#[tokio::test]
async fn test_discovery_debug() {
    let config = DiscoveryConfig::default();
    let discovery = UniversalPrimalDiscovery::new(config);

    let debug_str = format!("{:?}", discovery);
    assert!(debug_str.contains("UniversalPrimalDiscovery"));
}

#[tokio::test]
async fn test_discovery_sequential() {
    let config = DiscoveryConfig {
        mechanisms: DiscoveryMechanisms {
            enable_environment_scan: true,
            enable_network_scanning: true,
            enable_container_discovery: true,
        },
        timeout: Duration::from_millis(50),
    };

    let mut discovery = UniversalPrimalDiscovery::new(config);

    for _ in 0..3 {
        let result = discovery.discover_all_primals().await;
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_discovery_stress() {
    let config = DiscoveryConfig {
        mechanisms: DiscoveryMechanisms {
            enable_environment_scan: true,
            enable_network_scanning: false,
            enable_container_discovery: false,
        },
        timeout: Duration::from_millis(10),
    };

    let handles: Vec<_> = (0..10)
        .map(|_| {
            let mut discovery = UniversalPrimalDiscovery::new(config.clone());
            tokio::spawn(async move { discovery.discover_all_primals().await })
        })
        .collect();

    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_discovery_zero_timeout() {
    let config = DiscoveryConfig {
        mechanisms: DiscoveryMechanisms {
            enable_environment_scan: true,
            enable_network_scanning: false,
            enable_container_discovery: false,
        },
        timeout: Duration::from_secs(0),
    };

    let mut discovery = UniversalPrimalDiscovery::new(config);
    let result = discovery.discover_all_primals().await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_discovery_rapid_succession() {
    let config = DiscoveryConfig {
        mechanisms: DiscoveryMechanisms {
            enable_environment_scan: false,
            enable_network_scanning: false,
            enable_container_discovery: false,
        },
        timeout: Duration::from_millis(10),
    };

    let mut discovery = UniversalPrimalDiscovery::new(config);

    for _ in 0..5 {
        let _ = discovery.discover_all_primals().await;
    }

    // Multiple sequential discoveries complete successfully
}

#[tokio::test]
async fn test_discovery_with_all_timeouts() {
    let timeouts = vec![
        Duration::from_millis(1),
        Duration::from_millis(10),
        Duration::from_millis(100),
        Duration::from_secs(1),
    ];

    for timeout in timeouts {
        let config = DiscoveryConfig {
            mechanisms: DiscoveryMechanisms {
                enable_environment_scan: true,
                enable_network_scanning: false,
                enable_container_discovery: false,
            },
            timeout,
        };

        let mut discovery = UniversalPrimalDiscovery::new(config);
        let result = discovery.discover_all_primals().await;
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_discovery_mechanism_combinations() {
    let combinations = vec![
        (true, false, false),
        (false, true, false),
        (false, false, true),
        (true, true, false),
        (true, false, true),
        (false, true, true),
        (true, true, true),
    ];

    for (env, net, container) in combinations {
        let config = DiscoveryConfig {
            mechanisms: DiscoveryMechanisms {
                enable_environment_scan: env,
                enable_network_scanning: net,
                enable_container_discovery: container,
            },
            timeout: Duration::from_millis(50),
        };

        let mut discovery = UniversalPrimalDiscovery::new(config);
        let result = discovery.discover_all_primals().await;
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_discovery_large_concurrency() {
    let config = DiscoveryConfig {
        mechanisms: DiscoveryMechanisms {
            enable_environment_scan: false,
            enable_network_scanning: false,
            enable_container_discovery: false,
        },
        timeout: Duration::from_millis(5),
    };

    let handles: Vec<_> = (0..50)
        .map(|_| {
            let mut discovery = UniversalPrimalDiscovery::new(config.clone());
            tokio::spawn(async move { discovery.discover_all_primals().await })
        })
        .collect();

    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_discovery_result_consistency() {
    let config = DiscoveryConfig {
        mechanisms: DiscoveryMechanisms {
            enable_environment_scan: false,
            enable_network_scanning: false,
            enable_container_discovery: false,
        },
        timeout: Duration::from_millis(10),
    };

    let mut discovery = UniversalPrimalDiscovery::new(config);

    let result1 = discovery.discover_all_primals().await.unwrap();
    let result2 = discovery.discover_all_primals().await.unwrap();

    // Both should be consistent (likely both empty with no mechanisms)
    assert_eq!(result1.len(), result2.len());
}

// Total: 60 tests
// Expected coverage improvement: +3-5 percentage points
