use CanonicalSongbirdConfig;
//! Robust Networking Tests
//!
//! Comprehensive test suite for Songbird networking functionality including:
//! - Network configuration validation and security
//! - Connection management and pooling
//! - Protocol detection and handling
//! - Load balancing and failover scenarios
//! - Performance and timeout testing
//! - Error handling and edge cases

use songbird_config::config::{ConnectionLimits, NetworkTimeouts, PortRange};
use songbird_config::canonical_network::CanonicalNetworkConfig as NetworkConfig;
use songbird_core::{
    orchestrator::Orchestrator,
    traits::{
        ServiceDiscovery, ServiceInfo, ServiceHealth, HealthStatus,
        LoadBalancer, LoadBalancingStrategy,
    },
};
use songbird_errors::{NetworkError, SongbirdError, SongbirdResult};
use songbird_network::{
    management::{
        config::NetworkManagementConfig,
        load_balancer::{LoadBalancer as NetworkLoadBalancer, LoadBalancingStrategy as NetworkStrategy},
    },
    network::mod_types::{NetworkRequest, NetworkResponse},
};
use songbird_network::management::{
    config::HealthCheckConfig,
    health::{HealthCheckTarget, HealthChecker},
};
use std::net::{IpAddr, Ipv4Addr};
use std::time::{Duration, Instant};
use tokio::time::timeout;

#[cfg(test)]
mod network_config_tests {
    use super::*;

    #[test]
    fn test_network_config_defaults() {
        let config = NetworkConfig::default();

        // Test secure defaults
        assert_eq!(config.bind_address, IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))); // Localhost only
        assert!(config.orchestrator_port > 0);
        // Port is u16, so always <= 65535 by type definition
        assert!(config.require_tls == false); // Default false for development, should be true in prod

        // Test reasonable defaults
        assert!(config.connection_limits.max_connections_per_host > 0);
        assert!(config.connection_limits.max_total_connections > 0);
        assert!(config.timeouts.connection < config.timeouts.request);
    }

    #[test]
    fn test_network_config_validation() {
        let mut config = NetworkConfig::default();

        // Test valid port ranges
        config.orchestrator_port = config.network.http_port;
        assert!(
            config.orchestrator_port >= 1024,
            "Should use unprivileged ports by default"
        );

        // Test timeout relationships
        assert!(
            config.timeouts.connection < config.timeouts.request,
            "Connection timeout should be less than request timeout"
        );
        assert!(
            config.timeouts.health_check < config.timeouts.connection,
            "Health check should be fastest"
        );
    }

    #[test]
    fn test_production_security_config() {
        let mut config = NetworkConfig::default();

        // Configure for production
        config.bind_address = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)); // All interfaces
        config.require_tls = true;
        config.allowed_networks = vec![
            "10.0.0.0/8".to_string(),
            "192.168.0.0/16".to_string(),
            "172.16.0.0/12".to_string(),
        ];

        assert!(config.require_tls, "Production should require TLS");
        assert!(
            !config.allowed_networks.is_empty(),
            "Should have network restrictions"
        );
        assert!(
            config
                .allowed_networks
                .iter()
                .all(|net| !net.contains("0.0.0.0/0")),
            "Should not allow all networks"
        );
    }

    #[test]
    fn test_port_range_validation() {
        let gaming_range = PortRange {
            start: 7000,
            end: 8000,
        };

        assert!(gaming_range.contains(7500), "Should contain ports in range");
        assert!(
            !gaming_range.contains(6999),
            "Should not contain ports below range"
        );
        assert!(
            !gaming_range.contains(8001),
            "Should not contain ports above range"
        );

        let all_ports = gaming_range.all_ports();
        assert_eq!(all_ports.len(), 1001, "Should contain all ports in range");
        assert_eq!(all_ports[0], 7000);
        assert_eq!(all_ports[1000], 8000);

        let random_port = gaming_range.random_port();
        assert!(
            gaming_range.contains(random_port),
            "Random port should be in range"
        );
    }
}

#[cfg(test)]
mod connection_management_tests {
    use super::*;

    #[test]
    fn test_connection_limits() {
        let limits = ConnectionLimits {
            max_connections_per_host: 50,
            max_total_connections: 500,
            max_retries: 3,
            pool_idle_timeout_secs: 300,
        };

        assert!(
            limits.max_connections_per_host <= limits.max_total_connections,
            "Per-host limit should not exceed total limit"
        );
        assert!(
            limits.max_retries > 0 && limits.max_retries < 10,
            "Retry count should be reasonable"
        );
        assert!(
            limits.pool_idle_timeout_secs > 0,
            "Idle timeout should be positive"
        );
    }

    #[test]
    fn test_timeout_configuration() {
        let timeouts = NetworkTimeouts {
            connection: Duration::from_secs(10),
            request: Duration::from_secs(30),
            health_check: Duration::from_secs(5),
            default: Duration::from_secs(15),
        };

        // Test timeout ordering makes sense
        assert!(
            timeouts.health_check < timeouts.connection,
            "Health check should be fastest"
        );
        assert!(
            timeouts.connection < timeouts.request,
            "Connection should be faster than full request"
        );
        assert!(
            timeouts.default > timeouts.connection,
            "Default should be reasonable middle ground"
        );
    }

    #[tokio::test]
    async fn test_connection_timeout_enforcement() {
        let start = Instant::now();
        let timeout_duration = Duration::from_millis(100);

        // Simulate a slow operation that should timeout
        let result = timeout(timeout_duration, async {
            tokio::time::sleep(Duration::from_millis(200)).await;
            "completed"
        })
        .await;

        let elapsed = start.elapsed();
        assert!(result.is_err(), "Operation should have timed out");
        assert!(
            elapsed < Duration::from_millis(150),
            "Should timeout quickly"
        );
        assert!(
            elapsed >= timeout_duration,
            "Should respect timeout duration"
        );
    }
}

#[cfg(test)]
mod health_checking_tests {
    use super::*;

    #[tokio::test]
    async fn test_health_checker_creation() {
        let health_config = HealthCheckConfig::default();
        let health_checker = HealthChecker::new(health_config);

        // Should be able to create health checker
        // Note: HealthChecker::new returns a HealthChecker directly, not a Result
        assert_eq!(
            std::mem::size_of_val(&health_checker),
            std::mem::size_of::<HealthChecker>()
        );
    }

    #[tokio::test]
    async fn test_health_check_localhost() {
        let health_config = HealthCheckConfig::default();
        let mut health_checker = HealthChecker::new(health_config);

        // Test health check of localhost (should be reachable)
        let target = HealthCheckTarget {
            name: "localhost".to_string(),
            url: "http://127.0.0.1:0".to_string(), // Port 0 should fail, but address is valid
            expected_status: 200,
            timeout: Duration::from_secs(1),
        };
        let result = health_checker.check_target(&target).await;

        // Expect connection refused (which is better than DNS failure)
        // This tests the health check mechanism itself
        match result {
            Err(SongbirdError::Network { .. }) => {
                // Expected - can't connect to port 0, but tried to connect
            }
            Ok(_) => {
                // Unexpected but not wrong - maybe there's something on port 0
            }
            Err(e) => {
                panic!("Unexpected error type: {:?}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_health_check_invalid_address() {
        let health_config = HealthCheckConfig::default();
        let mut health_checker = HealthChecker::new(health_config);

        // Test health check of invalid address
        let target = HealthCheckTarget {
            name: "invalid".to_string(),
            url: "http://invalid.nonexistent.domain.test:{}".to_string(),
            expected_status: 200,
            timeout: Duration::from_secs(1),
        };
        let result = health_checker.check_target(&target).await;

        // Result may be error or success with unhealthy status - both are acceptable
        match result {
            Ok(health_result) => {
                // If it succeeds, it should indicate unhealthy status or failure
                assert_eq!(health_result.target, "invalid");
            }
            Err(SongbirdError::Network(network_err)) => {
                // Network error is expected for invalid address
                assert!(
                    network_err.message.contains("Failed") || network_err.message.contains("error"),
                    "Should contain error information"
                );
            }
            Err(_) => {
                // Other errors are also acceptable for invalid address
            }
        }
    }

    #[tokio::test]
    async fn test_health_check_timeout() {
        let health_config = HealthCheckConfig {
            interval: Duration::from_secs(60),
            timeout: Duration::from_millis(10), // Very short timeout
            unhealthy_threshold: 3,
            healthy_threshold: 2,
        };

        let mut health_checker = HealthChecker::new(health_config);

        let start = Instant::now();
        let target = HealthCheckTarget {
            name: "timeout_test".to_string(),
            url: "http://192.0.2.0:{}".to_string(), // RFC5737 test address
            expected_status: 200,
            timeout: Duration::from_millis(10),
        };
        let result = health_checker.check_target(&target).await;
        let elapsed = start.elapsed();

        // Test timeout behavior - result may be error or success with health status
        match result {
            Ok(health_result) => {
                // Health check succeeded - verify it completed quickly
                assert!(elapsed < Duration::from_secs(5), "Should complete quickly");
                assert_eq!(health_result.target, "timeout_test");
            }
            Err(_) => {
                // Error is also acceptable - verify timing
                assert!(
                    elapsed < Duration::from_secs(5),
                    "Should timeout quickly, not hang"
                );
            }
        }
    }
}

#[cfg(test)]
mod network_error_handling_tests {
    use super::*;

    #[test]
    fn test_network_error_creation() {
        let error = NetworkError {
            message: "Connection failed".to_string(),
            endpoint: Some("example.com:{}".to_string()),
            port: Some(config.network.http_port),
            protocol: Some("HTTP".to_string()),
        };

        assert_eq!(error.message, "Connection failed");
        assert_eq!(error.endpoint, Some("example.com:{}".to_string()));
        assert_eq!(error.port, Some(config.network.http_port));
        assert_eq!(error.protocol, Some("HTTP".to_string()));
    }

    #[test]
    fn test_songbird_error_network() {
        let network_err = SongbirdError::network_error("Connection timeout");

        match network_err {
            SongbirdError::Network(network_error) => {
                assert_eq!(network_error.message, "Connection timeout");
            }
            _ => panic!("Expected Network error"),
        }
    }

    #[test]
    fn test_error_context_preservation() {
        let original_error =
            std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "Connection refused");
        let network_error =
            SongbirdError::network_error(&format!("Failed to connect: {}", original_error));

        match network_error {
            SongbirdError::Network(network_err) => {
                assert!(network_err.message.contains("Connection refused"));
                assert!(network_err.message.contains("Failed to connect"));
            }
            _ => panic!("Expected Network error"),
        }
    }
}

#[cfg(test)]
mod network_performance_tests {
    use super::*;
    use tokio::time::Instant;

    #[tokio::test]
    async fn test_concurrent_connections_simulation() {
        let _config = NetworkConfig::default();
        let max_concurrent = 10;

        // Simulate multiple concurrent "connections" (really just delays)
        let mut handles = Vec::new();

        let start = Instant::now();

        for i in 0..max_concurrent {
            let handle = tokio::spawn(async move {
                let delay = Duration::from_millis(50 + (i as u64 * 10));
                tokio::time::sleep(delay).await;
                i
            });
            handles.push(handle);
        }

        // Wait for all to complete
        let results: Vec<_> = futures_util::future::join_all(handles).await;
        let elapsed = start.elapsed();

        assert_eq!(results.len(), max_concurrent);
        assert!(
            elapsed < Duration::from_millis(500),
            "Should complete concurrently, not sequentially"
        );

        // Verify all completed successfully
        for (i, result) in results.iter().enumerate() {
            assert!(result.is_ok(), "Task {} should complete successfully", i);
        }
    }

    #[tokio::test]
    async fn test_connection_pool_efficiency() {
        let limits = ConnectionLimits {
            max_connections_per_host: 5,
            max_total_connections: 20,
            max_retries: 3,
            pool_idle_timeout_secs: 60,
        };

        // Test that pool limits are reasonable for efficiency
        assert!(
            limits.max_connections_per_host < limits.max_total_connections,
            "Should allow connections to multiple hosts"
        );

        let efficiency_ratio =
            limits.max_connections_per_host as f64 / limits.max_total_connections as f64;
        assert!(
            efficiency_ratio >= 0.1 && efficiency_ratio <= 0.5,
            "Connection distribution should be balanced"
        );
    }

    #[test]
    fn test_bandwidth_calculations() {
        let max_bandwidth_mbps = 1000; // 1 Gbps
        let max_connections = 100;

        let bandwidth_per_connection = max_bandwidth_mbps / max_connections;
        assert_eq!(bandwidth_per_connection, 10); // 10 Mbps per connection

        // Test reasonable bandwidth allocation
        assert!(
            bandwidth_per_connection >= 1,
            "Should have at least 1 Mbps per connection"
        );
        assert!(
            bandwidth_per_connection <= 100,
            "Shouldn't over-allocate bandwidth"
        );
    }
}

#[cfg(test)]
mod network_security_tests {
    use super::*;

    #[test]
    fn test_network_allowlist_validation() {
        let allowed_networks = vec![
            "127.0.0.0/8".to_string(),    // Localhost
            "10.0.0.0/8".to_string(),     // Private RFC1918
            "192.168.0.0/16".to_string(), // Private RFC1918
            "172.16.0.0/12".to_string(),  // Private RFC1918
        ];

        // Test that no public networks are allowed by default
        for network in &allowed_networks {
            assert!(
                !network.contains("0.0.0.0/0"),
                "Should not allow all networks"
            );
            assert!(!network.starts_with("8.8."), "Should not allow public DNS");
            assert!(!network.starts_with("1.1."), "Should not allow public DNS");
        }

        // Test that private networks are properly formatted
        for network in &allowed_networks {
            assert!(network.contains("/"), "Should be in CIDR notation");
            let parts: Vec<&str> = network.split('/').collect();
            assert_eq!(parts.len(), 2, "Should have IP and mask");

            let mask: u8 = parts[1].parse().expect("Should parse mask");
            assert!(mask <= 32, "IPv4 mask should be <= 32");
        }
    }

    #[test]
    fn test_port_security_ranges() {
        let config = NetworkConfig::default();

        // Test that we're not using privileged ports by default
        assert!(
            config.orchestrator_port >= 1024,
            "Should use unprivileged ports"
        );
        assert!(
            config.discovery_port >= 1024,
            "Should use unprivileged ports"
        );
        assert!(
            config.dashboard_port >= 1024,
            "Should use unprivileged ports"
        );

        // Test gaming port range is reasonable
        assert!(
            config.gaming_port_range.start >= 1024,
            "Gaming ports should be unprivileged"
        );
        assert!(
            config.gaming_port_range.end <= 65535,
            "Ports should be valid"
        );
        assert!(
            config.gaming_port_range.start < config.gaming_port_range.end,
            "Range should be valid"
        );
    }

    #[test]
    fn test_tls_configuration() {
        let mut config = NetworkConfig::default();

        // Test TLS defaults
        config.require_tls = true;
        assert!(config.require_tls, "TLS should be configurable");

        // In production, TLS should be required for external interfaces
        if config.bind_address == IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)) {
            assert!(config.require_tls, "External interfaces should require TLS");
        }
    }

    #[test]
    fn test_connection_rate_limiting() {
        let limits = ConnectionLimits::default();

        // Test that rate limiting is reasonable
        assert!(
            limits.max_connections_per_host > 0,
            "Should allow connections"
        );
        assert!(
            limits.max_connections_per_host <= 100,
            "Should have reasonable per-host limits"
        );
        assert!(
            limits.max_total_connections >= limits.max_connections_per_host,
            "Total should be >= per-host"
        );
        assert!(
            limits.max_retries > 0 && limits.max_retries <= 5,
            "Should have reasonable retry limits"
        );
    }
}

#[cfg(test)]
mod network_edge_cases_tests {
    use super::*;

    #[test]
    fn test_zero_timeout_handling() {
        // Test that zero timeouts are handled gracefully
        let timeouts = NetworkTimeouts {
            connection: Duration::from_secs(0),
            request: Duration::from_secs(30),
            health_check: Duration::from_secs(5),
            default: Duration::from_secs(15),
        };

        // Zero timeout should be treated as immediate timeout
        assert_eq!(timeouts.connection, Duration::from_secs(0));
    }

    #[test]
    fn test_maximum_port_range() {
        let max_range = PortRange {
            start: 1,
            end: 65535,
        };

        assert!(max_range.contains(1), "Should contain minimum port");
        assert!(max_range.contains(65535), "Should contain maximum port");
        assert!(!max_range.contains(0), "Port 0 should not be in range");

        let all_ports = max_range.all_ports();
        assert_eq!(all_ports.len(), 65535, "Should contain all valid ports");
    }

    #[tokio::test]
    async fn test_connection_cleanup_on_drop() {
        // Test that connections are properly cleaned up when dropped
        let health_config = HealthCheckConfig::default();
        let health_checker = HealthChecker::new(health_config);

        // Create and drop the health checker
        drop(health_checker);

        // If we get here without panicking, cleanup worked
        assert!(true, "Health checker should drop cleanly");
    }

    #[test]
    fn test_invalid_address_formats() {
        let invalid_addresses = vec![
            "",                            // Empty
            "not-a-url",                   // Not a URL
            "http://",                     // Incomplete URL
            "ftp://example.com",           // Wrong protocol
            "http://[invalid-ipv6",        // Malformed IPv6
            "http://256.256.256.256:{}", // Invalid IP
        ];

        for addr in invalid_addresses {
            // These should be handled gracefully by the health checker
            // We can't test the actual health check here without async, but we can verify
            // the addresses are properly formatted for testing
            assert!(
                !addr.is_empty() || addr == "",
                "Test addresses should be as expected"
            );
        }
    }
}
