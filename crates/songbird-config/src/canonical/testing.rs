//! Test fixtures for canonical configuration types
//!
//! Provides pre-configured test instances of canonical config types
//! to simplify testing across the codebase.

use super::*;
use std::net::IpAddr;
use std::time::Duration;

/// Create a minimal test configuration suitable for unit tests
///
/// This configuration has minimal dependencies and fast timeouts,
/// making it ideal for unit testing individual components.
pub fn minimal_test_config() -> MinimalTestConfig {
    MinimalTestConfig {
        network: minimal_network_config(),
        performance: minimal_performance_config(),
    }
}

/// Create a full test configuration suitable for integration tests
///
/// This configuration includes all subsystems configured for
/// realistic testing scenarios.
pub fn full_test_config() -> FullTestConfig {
    FullTestConfig {
        network: test_network_config(),
        security: test_security_config(),
        discovery: test_discovery_config(),
        observability: test_observability_config(),
        performance: test_performance_config(),
    }
}

/// Create a production-like test configuration
///
/// This configuration mirrors production settings but with
/// test-specific overrides (shorter timeouts, etc.)
pub fn production_test_config() -> FullTestConfig {
    FullTestConfig {
        network: production_network_config(),
        security: production_security_config(),
        discovery: production_discovery_config(),
        observability: production_observability_config(),
        performance: production_performance_config(),
    }
}

// Network configurations

fn minimal_network_config() -> network::CanonicalNetworkConfig {
    network::CanonicalNetworkConfig {
        bind_address: "127.0.0.1".parse().unwrap(),
        production_bind_address: "127.0.0.1".parse().unwrap(),
        port: 8080,
        discovery_port: 8081,
        external_address: Some("http://localhost:8080".to_string()),
        ..Default::default()
    }
}

fn test_network_config() -> network::CanonicalNetworkConfig {
    network::CanonicalNetworkConfig {
        bind_address: "127.0.0.1".parse().unwrap(),
        production_bind_address: "127.0.0.1".parse().unwrap(),
        port: 8080,
        discovery_port: 8081,
        external_address: Some("http://localhost:8080".to_string()),
        timeout: Duration::from_secs(30),
        connection_timeout: Duration::from_secs(10),
        ..Default::default()
    }
}

fn production_network_config() -> network::CanonicalNetworkConfig {
    network::CanonicalNetworkConfig {
        bind_address: "0.0.0.0".parse().unwrap(),
        production_bind_address: "0.0.0.0".parse().unwrap(),
        port: 8080,
        discovery_port: 8081,
        external_address: None, // Will be detected
        timeout: Duration::from_secs(60),
        connection_timeout: Duration::from_secs(30),
        ..Default::default()
    }
}

// Security configurations

fn test_security_config() -> security::UniversalSecurityConfig {
    security::UniversalSecurityConfig {
        capability_requirements: security::SecurityCapabilityRequirements {
            authentication: security::AuthenticationLevel::Optional,
            encryption: security::EncryptionLevel::Optional,
            access_control: security::AccessControlLevel::Basic,
        },
        authentication: test_authentication_config(),
        encryption: test_encryption_config(),
        access_control: test_access_control_config(),
        ..Default::default()
    }
}

fn production_security_config() -> security::UniversalSecurityConfig {
    security::UniversalSecurityConfig {
        capability_requirements: security::SecurityCapabilityRequirements {
            authentication: security::AuthenticationLevel::Required,
            encryption: security::EncryptionLevel::Required,
            access_control: security::AccessControlLevel::Advanced,
        },
        authentication: production_authentication_config(),
        encryption: production_encryption_config(),
        access_control: production_access_control_config(),
        ..Default::default()
    }
}

fn test_authentication_config() -> security::AuthenticationConfig {
    security::AuthenticationConfig {
        enabled: false, // Simplified for testing
        provider: None,
        token: security::TokenConfig {
            expiration_secs: 3600,
            refresh_enabled: false,
        },
        session: security::SessionConfig {
            timeout_secs: 3600,
            max_sessions_per_user: 5,
        },
    }
}

fn production_authentication_config() -> security::AuthenticationConfig {
    security::AuthenticationConfig {
        enabled: true,
        provider: Some("oauth2".to_string()),
        token: security::TokenConfig {
            expiration_secs: 900, // 15 minutes
            refresh_enabled: true,
        },
        session: security::SessionConfig {
            timeout_secs: 3600,
            max_sessions_per_user: 3,
        },
    }
}

fn test_encryption_config() -> security::EncryptionConfig {
    security::EncryptionConfig {
        enabled: false, // Simplified for testing
        algorithm: "aes256".to_string(),
        key_management: security::KeyManagementConfig {
            rotation_interval_secs: 86400, // 1 day
            auto_rotation: false,
        },
        transport: security::TransportEncryptionConfig {
            require_tls: false,
            min_tls_version: "1.2".to_string(),
        },
    }
}

fn production_encryption_config() -> security::EncryptionConfig {
    security::EncryptionConfig {
        enabled: true,
        algorithm: "aes256-gcm".to_string(),
        key_management: security::KeyManagementConfig {
            rotation_interval_secs: 604800, // 7 days
            auto_rotation: true,
        },
        transport: security::TransportEncryptionConfig {
            require_tls: true,
            min_tls_version: "1.3".to_string(),
        },
    }
}

fn test_access_control_config() -> security::AccessControlConfig {
    security::AccessControlConfig {
        enabled: false, // Simplified for testing
        rbac: security::RbacConfig {
            enabled: false,
            default_role: "user".to_string(),
        },
        abac: security::AbacConfig {
            enabled: false,
            policy_enforcement_point: "local".to_string(),
        },
    }
}

fn production_access_control_config() -> security::AccessControlConfig {
    security::AccessControlConfig {
        enabled: true,
        rbac: security::RbacConfig {
            enabled: true,
            default_role: "guest".to_string(),
        },
        abac: security::AbacConfig {
            enabled: true,
            policy_enforcement_point: "centralized".to_string(),
        },
    }
}

// Discovery configurations

fn test_discovery_config() -> discovery::DiscoveryConfig {
    discovery::DiscoveryConfig {
        service_discovery: discovery::ServiceDiscoveryConfig {
            enabled: true,
            interval_secs: 5,
            timeout_secs: 2,
        },
        capability_discovery: discovery::CapabilityDiscoveryConfig {
            enabled: true,
            scan_interval_secs: 10,
        },
        network_discovery: discovery::NetworkDiscoveryConfig {
            enabled: false, // Disabled for unit tests
            broadcast_interval_secs: 30,
        },
    }
}

fn production_discovery_config() -> discovery::DiscoveryConfig {
    discovery::DiscoveryConfig {
        service_discovery: discovery::ServiceDiscoveryConfig {
            enabled: true,
            interval_secs: 30,
            timeout_secs: 10,
        },
        capability_discovery: discovery::CapabilityDiscoveryConfig {
            enabled: true,
            scan_interval_secs: 60,
        },
        network_discovery: discovery::NetworkDiscoveryConfig {
            enabled: true,
            broadcast_interval_secs: 60,
        },
    }
}

// Observability configurations

fn test_observability_config() -> observability::UnifiedObservabilityConfig {
    observability::UnifiedObservabilityConfig {
        dashboard: observability::DashboardConfig {
            enabled: false, // Disabled for tests
            port: 3000,
            bind_address: "127.0.0.1".to_string(),
        },
        logging: observability::LoggingConfig {
            enabled: true,
            level: "debug".to_string(),
            format: "text".to_string(),
            rotation: observability::LogRotationConfig {
                enabled: false,
                max_size_mb: 100,
                max_age_days: 7,
            },
        },
        tracing: observability::TracingConfig {
            enabled: false, // Disabled for unit tests
            endpoint: None,
            sample_rate: 1.0,
        },
    }
}

fn production_observability_config() -> observability::UnifiedObservabilityConfig {
    observability::UnifiedObservabilityConfig {
        dashboard: observability::DashboardConfig {
            enabled: true,
            port: 3000,
            bind_address: "0.0.0.0".to_string(),
        },
        logging: observability::LoggingConfig {
            enabled: true,
            level: "info".to_string(),
            format: "json".to_string(),
            rotation: observability::LogRotationConfig {
                enabled: true,
                max_size_mb: 1000,
                max_age_days: 30,
            },
        },
        tracing: observability::TracingConfig {
            enabled: true,
            endpoint: Some("http://jaeger:14268/api/traces".to_string()),
            sample_rate: 0.1, // 10% sampling in production
        },
    }
}

// Performance configurations

fn minimal_performance_config() -> performance::PerformanceConfig {
    performance::PerformanceConfig {
        thread_pool_size: 2, // Minimal for tests
        max_blocking_threads: 2,
        stack_size_kb: 2048,
        cache: performance::CacheConfig {
            enabled: false,
            ttl_secs: 60,
            max_entries: 100,
        },
        metrics: performance::MetricsConfig {
            enabled: false,
            interval_secs: 60,
        },
        benchmark: performance::BenchmarkConfig {
            enabled: false,
            iterations: 100,
        },
    }
}

fn test_performance_config() -> performance::PerformanceConfig {
    performance::PerformanceConfig {
        thread_pool_size: 4,
        max_blocking_threads: 4,
        stack_size_kb: 2048,
        cache: performance::CacheConfig {
            enabled: true,
            ttl_secs: 30, // Short TTL for tests
            max_entries: 1000,
        },
        metrics: performance::MetricsConfig {
            enabled: true,
            interval_secs: 10, // Frequent for tests
        },
        benchmark: performance::BenchmarkConfig {
            enabled: false,
            iterations: 100,
        },
    }
}

fn production_performance_config() -> performance::PerformanceConfig {
    performance::PerformanceConfig {
        thread_pool_size: num_cpus::get(),
        max_blocking_threads: 512,
        stack_size_kb: 2048,
        cache: performance::CacheConfig {
            enabled: true,
            ttl_secs: 300, // 5 minutes
            max_entries: 10000,
        },
        metrics: performance::MetricsConfig {
            enabled: true,
            interval_secs: 60,
        },
        benchmark: performance::BenchmarkConfig {
            enabled: false,
            iterations: 1000,
        },
    }
}

// Config container types

/// Minimal test configuration with only essential subsystems
#[derive(Debug, Clone)]
pub struct MinimalTestConfig {
    pub network: network::CanonicalNetworkConfig,
    pub performance: performance::PerformanceConfig,
}

/// Full test configuration with all subsystems
#[derive(Debug, Clone)]
pub struct FullTestConfig {
    pub network: network::CanonicalNetworkConfig,
    pub security: security::UniversalSecurityConfig,
    pub discovery: discovery::DiscoveryConfig,
    pub observability: observability::UnifiedObservabilityConfig,
    pub performance: performance::PerformanceConfig,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_config_creation() {
        let config = minimal_test_config();
        assert_eq!(config.network.port, 8080);
        assert_eq!(config.performance.thread_pool_size, 2);
    }

    #[test]
    fn test_full_config_creation() {
        let config = full_test_config();
        assert_eq!(config.network.port, 8080);
        assert!(config.discovery.service_discovery.enabled);
    }

    #[test]
    fn test_production_config_creation() {
        let config = production_test_config();
        assert!(config.security.authentication.enabled);
        assert!(config.security.encryption.enabled);
    }
}

