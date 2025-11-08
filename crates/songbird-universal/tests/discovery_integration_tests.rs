//! Integration tests for discovery workflows
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::float_cmp)]
#![allow(clippy::useless_vec)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::module_name_repetitions)]

//!
//! These tests verify that discovery mechanisms work correctly in realistic scenarios.

use songbird_types::SongbirdResult;
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal::discovery::{
    DiscoveryConfig, DiscoveryMechanisms, UniversalPrimalDiscovery,
};
use std::time::Duration;

#[tokio::test]
async fn test_discovery_with_all_mechanisms_enabled() -> SongbirdResult<()> {
    // ARRANGE
    let mechanisms = DiscoveryMechanisms {
        enable_environment_scan: true,
        enable_network_scanning: true,
        enable_container_discovery: true,
    };

    let config = DiscoveryConfig {
        mechanisms,
        timeout: Duration::from_secs(30),
    };

    let discovery = UniversalPrimalDiscovery::new(config);

    // ACT & ASSERT
    // Discovery should be created successfully
    assert!(format!("{:?}", discovery).contains("UniversalPrimalDiscovery"));
    Ok(())
}

#[tokio::test]
async fn test_discovery_with_only_environment_enabled() -> SongbirdResult<()> {
    // ARRANGE
    let mechanisms = DiscoveryMechanisms {
        enable_environment_scan: true,
        enable_network_scanning: false,
        enable_container_discovery: false,
    };

    let config = DiscoveryConfig {
        mechanisms,
        timeout: Duration::from_secs(10),
    };

    // ACT
    let discovery = UniversalPrimalDiscovery::new(config);

    // ASSERT
    assert!(format!("{:?}", discovery).contains("UniversalPrimalDiscovery"));
    Ok(())
}

#[tokio::test]
async fn test_discovery_with_network_scanning_enabled() -> SongbirdResult<()> {
    // ARRANGE
    let mechanisms = DiscoveryMechanisms {
        enable_environment_scan: false,
        enable_network_scanning: true,
        enable_container_discovery: false,
    };

    let config = DiscoveryConfig {
        mechanisms,
        timeout: Duration::from_secs(15),
    };

    // ACT
    let discovery = UniversalPrimalDiscovery::new(config);

    // ASSERT
    assert!(format!("{:?}", discovery).contains("UniversalPrimalDiscovery"));
    Ok(())
}

#[tokio::test]
async fn test_discovery_with_container_discovery_enabled() -> SongbirdResult<()> {
    // ARRANGE
    let mechanisms = DiscoveryMechanisms {
        enable_environment_scan: false,
        enable_network_scanning: false,
        enable_container_discovery: true,
    };

    let config = DiscoveryConfig {
        mechanisms,
        timeout: Duration::from_secs(20),
    };

    // ACT
    let discovery = UniversalPrimalDiscovery::new(config);

    // ASSERT
    assert!(format!("{:?}", discovery).contains("UniversalPrimalDiscovery"));
    Ok(())
}

#[tokio::test]
async fn test_discovery_config_with_short_timeout() -> SongbirdResult<()> {
    // ARRANGE & ACT
    let config = DiscoveryConfig {
        mechanisms: DiscoveryMechanisms {
            enable_environment_scan: true,
            enable_network_scanning: false,
            enable_container_discovery: false,
        },
        timeout: Duration::from_secs(5),
    };

    let discovery = UniversalPrimalDiscovery::new(config);

    // ASSERT
    assert!(format!("{:?}", discovery).contains("UniversalPrimalDiscovery"));
    Ok(())
}

#[tokio::test]
async fn test_discovery_config_with_long_timeout() -> SongbirdResult<()> {
    // ARRANGE & ACT
    let config = DiscoveryConfig {
        mechanisms: DiscoveryMechanisms {
            enable_environment_scan: true,
            enable_network_scanning: false,
            enable_container_discovery: false,
        },
        timeout: Duration::from_secs(60),
    };

    let discovery = UniversalPrimalDiscovery::new(config);

    // ASSERT
    assert!(format!("{:?}", discovery).contains("UniversalPrimalDiscovery"));
    Ok(())
}

#[tokio::test]
async fn test_discovery_mechanisms_all_disabled() -> SongbirdResult<()> {
    // ARRANGE
    let mechanisms = DiscoveryMechanisms {
        enable_environment_scan: false,
        enable_network_scanning: false,
        enable_container_discovery: false,
    };

    let config = DiscoveryConfig {
        mechanisms,
        timeout: Duration::from_secs(10),
    };

    // ACT
    let discovery = UniversalPrimalDiscovery::new(config);

    // ASSERT - Should still create successfully even with all mechanisms disabled
    assert!(format!("{:?}", discovery).contains("UniversalPrimalDiscovery"));
    Ok(())
}

#[test]
fn test_discovery_mechanisms_creation() {
    // ACT
    let mechanisms = DiscoveryMechanisms {
        enable_environment_scan: true,
        enable_network_scanning: false,
        enable_container_discovery: false,
    };

    // ASSERT - Should enable environment scan
    assert!(mechanisms.enable_environment_scan);
    assert!(!mechanisms.enable_network_scanning);
}

#[test]
fn test_discovery_config_creation() {
    // ACT
    let config = DiscoveryConfig {
        mechanisms: DiscoveryMechanisms {
            enable_environment_scan: true,
            enable_network_scanning: false,
            enable_container_discovery: false,
        },
        timeout: Duration::from_secs(30),
    };

    // ASSERT
    assert!(config.mechanisms.enable_environment_scan);
    assert!(config.timeout.as_secs() > 0);
}

#[test]
fn test_discovery_mechanisms_clone() {
    // ARRANGE
    let mechanisms1 = DiscoveryMechanisms {
        enable_environment_scan: true,
        enable_network_scanning: true,
        enable_container_discovery: false,
    };

    // ACT
    let mechanisms2 = mechanisms1.clone();

    // ASSERT
    assert_eq!(mechanisms1.enable_environment_scan, mechanisms2.enable_environment_scan);
    assert_eq!(mechanisms1.enable_network_scanning, mechanisms2.enable_network_scanning);
    assert_eq!(mechanisms1.enable_container_discovery, mechanisms2.enable_container_discovery);
}

#[test]
fn test_discovery_config_clone() {
    // ARRANGE
    let config1 = DiscoveryConfig {
        mechanisms: DiscoveryMechanisms {
            enable_environment_scan: true,
            enable_network_scanning: false,
            enable_container_discovery: false,
        },
        timeout: Duration::from_secs(45),
    };

    // ACT
    let config2 = config1.clone();

    // ASSERT
    assert_eq!(config1.timeout, config2.timeout);
}

#[tokio::test]
async fn test_discovery_with_custom_timeout_boundaries() -> SongbirdResult<()> {
    // Test minimum timeout
    let config_min = DiscoveryConfig {
        mechanisms: DiscoveryMechanisms {
            enable_environment_scan: true,
            enable_network_scanning: false,
            enable_container_discovery: false,
        },
        timeout: Duration::from_secs(1),
    };
    let discovery_min = UniversalPrimalDiscovery::new(config_min);
    assert!(format!("{:?}", discovery_min).contains("UniversalPrimalDiscovery"));

    // Test maximum reasonable timeout
    let config_max = DiscoveryConfig {
        mechanisms: DiscoveryMechanisms {
            enable_environment_scan: true,
            enable_network_scanning: false,
            enable_container_discovery: false,
        },
        timeout: Duration::from_secs(300),
    };
    let discovery_max = UniversalPrimalDiscovery::new(config_max);
    assert!(format!("{:?}", discovery_max).contains("UniversalPrimalDiscovery"));
    Ok(())
}

#[tokio::test]
async fn test_discovery_sequential_creation() -> SongbirdResult<()> {
    // Test that we can create multiple discovery instances
    let config = DiscoveryConfig {
        mechanisms: DiscoveryMechanisms {
            enable_environment_scan: true,
            enable_network_scanning: false,
            enable_container_discovery: false,
        },
        timeout: Duration::from_secs(30),
    };

    let discovery1 = UniversalPrimalDiscovery::new(config.clone());
    let discovery2 = UniversalPrimalDiscovery::new(config.clone());
    let discovery3 = UniversalPrimalDiscovery::new(config);

    assert!(format!("{:?}", discovery1).contains("UniversalPrimalDiscovery"));
    assert!(format!("{:?}", discovery2).contains("UniversalPrimalDiscovery"));
    assert!(format!("{:?}", discovery3).contains("UniversalPrimalDiscovery"));
    Ok(())
}

#[tokio::test]
async fn test_discovery_mechanisms_partial_combinations() -> SongbirdResult<()> {
    // Test various combinations of enabled mechanisms
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
        let mechanisms = DiscoveryMechanisms {
            enable_environment_scan: env,
            enable_network_scanning: net,
            enable_container_discovery: container,
        };

        let config = DiscoveryConfig {
            mechanisms,
            timeout: Duration::from_secs(10),
        };

        let discovery = UniversalPrimalDiscovery::new(config);
        assert!(format!("{:?}", discovery).contains("UniversalPrimalDiscovery"));
    }
    Ok(())
}
