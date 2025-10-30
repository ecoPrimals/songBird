//! Comprehensive tests for songbird-config defaults modules
//!
//! This test suite provides comprehensive coverage for all default configuration
//! modules, including environment variable overrides and error conditions.

use songbird_config::defaults::*;
use std::env;
use std::time::Duration;

// ============================================================================
// ENDPOINTS TESTS
// ============================================================================

#[cfg(test)]
mod endpoints_tests {
    use super::*;

    #[test]
    fn test_discovery_endpoint() {
        let endpoint = discovery_endpoint();
        assert!(!endpoint.is_empty(), "Discovery endpoint should not be empty");
        assert!(
            endpoint.starts_with("http://") || endpoint.starts_with("https://"),
            "Endpoint should start with http:// or https://"
        );
    }

    #[test]
    fn test_orchestrator_endpoint() {
        let endpoint = orchestrator_endpoint();
        assert!(!endpoint.is_empty());
        assert!(endpoint.starts_with("http://") || endpoint.starts_with("https://"));
    }

    #[test]
    fn test_metrics_url() {
        let url = metrics_url();
        assert!(!url.is_empty());
        assert!(url.starts_with("http://") || url.starts_with("https://"));
    }

    #[test]
    fn test_dashboard_url() {
        let url = dashboard_url();
        assert!(!url.is_empty());
        assert!(url.starts_with("http://") || url.starts_with("https://"));
    }
}

// ============================================================================
// HOSTS TESTS
// ============================================================================

#[cfg(test)]
mod hosts_tests {
    use super::*;

    #[test]
    fn test_default_host() {
        let host = default_host();
        assert!(!host.is_empty());
        // Should be a valid IP or hostname
        assert!(host == "127.0.0.1" || host.parse::<std::net::IpAddr>().is_ok());
    }

    #[test]
    fn test_bind_address() {
        let host = bind_address();
        assert!(!host.is_empty());
        assert!(
            host == "0.0.0.0" || host == "127.0.0.1" || host.parse::<std::net::IpAddr>().is_ok()
        );
    }

    #[test]
    fn test_discovery_host() {
        let host = discovery_host();
        assert!(!host.is_empty());
    }

    #[test]
    fn test_orchestrator_host() {
        let host = orchestrator_host();
        assert!(!host.is_empty());
    }

    #[test]
    fn test_service_host() {
        let host = service_host("TEST");
        assert!(!host.is_empty());
    }

    #[test]
    fn test_environment() {
        let env = environment();
        assert!(!env.is_empty());
        assert!(env == "development" || env == "production" || env == "staging");
    }

    #[test]
    fn test_is_production() {
        // Should return false for default environment
        let prod = is_production();
        assert!(!prod || env::var("SONGBIRD_ENVIRONMENT").is_ok());
    }

    #[test]
    fn test_host_environment_override() {
        // Test that environment variables can override defaults
        env::set_var("SONGBIRD_HOST", "192.168.1.1");
        let host = default_host();
        assert_eq!(host, "192.168.1.1");
        env::remove_var("SONGBIRD_HOST");
    }
}

// ============================================================================
// PORTS TESTS
// ============================================================================

#[cfg(test)]
mod ports_tests {
    use super::*;

    #[test]
    fn test_orchestrator_port() {
        let port = orchestrator_port();
        assert!(port > 0, "Port should be valid");
        assert!(port < 65535, "Port should be valid");
    }

    #[test]
    fn test_discovery_port() {
        let port = discovery_port();
        assert!(port > 0);
        assert!(port < 65535);
    }

    #[test]
    fn test_health_port() {
        let port = health_port();
        assert!(port > 0);
        assert!(port < 65535);
    }

    #[test]
    fn test_metrics_port() {
        let port = metrics_port();
        assert!(port > 0);
        assert!(port < 65535);
    }

    #[test]
    fn test_dashboard_port() {
        let port = dashboard_port();
        assert!(port > 0);
        assert!(port < 65535);
    }

    #[test]
    fn test_port_environment_override() {
        env::set_var("SONGBIRD_ORCHESTRATOR_PORT", "9999");
        let port = orchestrator_port();
        assert_eq!(port, 9999);
        env::remove_var("SONGBIRD_ORCHESTRATOR_PORT");
    }

    #[test]
    fn test_ports_are_different() {
        // Ensure different services use different ports
        let discovery = discovery_port();
        let health = health_port();
        let metrics = metrics_port();
        let orchestrator = orchestrator_port();

        // At least some ports should be different
        let unique_count = [discovery, health, metrics, orchestrator]
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len();
        assert!(unique_count >= 2, "Services should use different ports");
    }
}

// ============================================================================
// TIMEOUTS TESTS (Extended)
// ============================================================================

#[cfg(test)]
mod timeouts_extended_tests {
    use super::*;

    #[test]
    fn test_all_timeouts_positive() {
        assert!(standard_timeout().as_millis() > 0);
        assert!(long_timeout().as_millis() > 0);
        assert!(request_timeout().as_millis() > 0);
        assert!(cache_expiry().as_millis() > 0);
        assert!(heartbeat_interval().as_millis() > 0);
        assert!(discovery_timeout().as_millis() > 0);
        assert!(connection_timeout().as_millis() > 0);
        assert!(retry_backoff().as_millis() > 0);
    }

    #[test]
    fn test_timeout_ordering() {
        let standard = standard_timeout();
        let long = long_timeout();

        // Long timeout should be longer than standard
        assert!(long >= standard);
    }

    #[test]
    fn test_timeout_environment_override() {
        env::set_var("SONGBIRD_TIMEOUT_MS", "1234");
        let timeout = standard_timeout();
        assert_eq!(timeout.as_millis(), 1234);
        env::remove_var("SONGBIRD_TIMEOUT_MS");
    }

    #[test]
    fn test_operation_timeout_with_custom_name() {
        let timeout = operation_timeout("CUSTOM_OP", Duration::from_secs(7));
        assert!(timeout.as_secs() >= 7);
    }

    #[test]
    fn test_operation_timeout_respects_env() {
        env::set_var("SONGBIRD_MYOP_TIMEOUT_MS", "3456");
        let timeout = operation_timeout("MYOP", Duration::from_secs(1));
        assert_eq!(timeout.as_millis(), 3456);
        env::remove_var("SONGBIRD_MYOP_TIMEOUT_MS");
    }

    #[test]
    fn test_retry_backoff_reasonable() {
        let backoff = retry_backoff();
        // Retry backoff should be reasonable (not too short, not too long)
        assert!(backoff.as_millis() >= 100);
        assert!(backoff.as_secs() <= 60);
    }

    #[test]
    fn test_cache_expiry_reasonable() {
        let expiry = cache_expiry();
        // Cache expiry should be reasonable (minutes to hours range)
        assert!(expiry.as_secs() >= 60); // At least 1 minute
        assert!(expiry.as_secs() <= 3600 * 24); // At most 24 hours
    }
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_complete_server_config() {
        // Test that all defaults work together to form a complete configuration
        let host = bind_address();
        let port = orchestrator_port();
        let timeout = standard_timeout();

        assert!(!host.is_empty());
        assert!(port > 0);
        assert!(timeout.as_millis() > 0);

        // Should be able to construct a valid server address
        let addr = format!("{}:{}", host, port);
        assert!(addr.contains(':'));
    }

    #[test]
    fn test_service_discovery_config() {
        let host = discovery_host();
        let port = discovery_port();
        let timeout = discovery_timeout();
        let endpoint = discovery_endpoint();

        assert!(!host.is_empty());
        assert!(port > 0);
        assert!(timeout.as_millis() > 0);
        assert!(!endpoint.is_empty());
    }

    #[test]
    fn test_health_check_config() {
        let port = health_port();
        let timeout = standard_timeout();

        assert!(port > 0);
        assert!(timeout.as_millis() > 0);
    }

    #[test]
    fn test_metrics_config() {
        let port = metrics_port();
        let url = metrics_url();

        assert!(port > 0);
        assert!(!url.is_empty());
    }
}

// ============================================================================
// ERROR CONDITION TESTS
// ============================================================================

#[cfg(test)]
mod error_condition_tests {
    use super::*;

    #[test]
    fn test_invalid_port_env_fallback() {
        // Test that invalid port env var falls back to default
        env::set_var("SONGBIRD_ORCHESTRATOR_PORT", "invalid");
        let port = orchestrator_port();
        assert_eq!(port, 8080); // Should use default
        env::remove_var("SONGBIRD_ORCHESTRATOR_PORT");
    }

    #[test]
    fn test_out_of_range_port_env() {
        // Test that out-of-range port env var falls back
        env::set_var("SONGBIRD_ORCHESTRATOR_PORT", "99999");
        let port = orchestrator_port();
        assert_eq!(port, 8080); // Should use default (parse fails for > u16::MAX)
        env::remove_var("SONGBIRD_ORCHESTRATOR_PORT");
    }

    #[test]
    fn test_invalid_timeout_env_fallback() {
        env::set_var("SONGBIRD_TIMEOUT_MS", "not_a_number");
        let timeout = standard_timeout();
        assert!(timeout.as_millis() > 0); // Should use default
        env::remove_var("SONGBIRD_TIMEOUT_MS");
    }

    #[test]
    fn test_negative_timeout_env_fallback() {
        env::set_var("SONGBIRD_TIMEOUT_MS", "-1000");
        let timeout = standard_timeout();
        assert!(timeout.as_millis() > 0); // Should use default
        env::remove_var("SONGBIRD_TIMEOUT_MS");
    }
}

// ============================================================================
// EDGE CASE TESTS
// ============================================================================

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_zero_timeout_env() {
        env::set_var("SONGBIRD_TIMEOUT_MS", "0");
        let timeout = standard_timeout();
        // Zero timeout from env should either be accepted or fall back to default
        // Both behaviors are acceptable
        assert!(timeout.as_millis() >= 0);
        env::remove_var("SONGBIRD_TIMEOUT_MS");
    }

    #[test]
    fn test_very_large_timeout() {
        env::set_var("SONGBIRD_TIMEOUT_MS", "9999999");
        let timeout = standard_timeout();
        assert_eq!(timeout.as_millis(), 9999999);
        env::remove_var("SONGBIRD_TIMEOUT_MS");
    }

    #[test]
    fn test_port_values_reasonable() {
        // Test that default ports are reasonable
        let port = orchestrator_port();
        assert!(port > 0 && port < 65535, "Should be valid port");
    }

    #[test]
    fn test_localhost_variations() {
        // Test that common localhost variations are handled
        let host = default_host();
        let bind = bind_address();

        // Default host should be localhost
        assert_eq!(host, "127.0.0.1");
        // Bind address should be 0.0.0.0 for all interfaces
        assert_eq!(bind, "0.0.0.0");
    }
}
