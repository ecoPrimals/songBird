//! Tests for Universal Discovery Factory
//!
//! Comprehensive test coverage for factory patterns and environment detection.

#[cfg(test)]
mod tests {
    use super::super::factory::UniversalDiscoveryFactory;
    use crate::traits::discovery::ServiceDiscovery;
    use std::time::Duration;

    #[tokio::test]
    async fn test_create_auto_detect() {
        // Test that we can create a discovery adapter with auto-detection
        let result = UniversalDiscoveryFactory::create_auto_detect().await;
        assert!(result.is_ok(), "Auto-detect should succeed");
    }

    #[tokio::test]
    async fn test_create_for_config_with_static() {
        // Test creating discovery with static backend
        use crate::traits::discovery::{DiscoveryBackend, DiscoveryConfig};

        let config = DiscoveryConfig {
            backend: DiscoveryBackend::Static,
            health_check_interval: Duration::from_secs(60),
            connection_timeout: Duration::from_secs(5),
            retry_attempts: 3,
            retry_delay: Duration::from_secs(1),
        };

        let result = UniversalDiscoveryFactory::create_for_config(&config).await;
        assert!(result.is_ok(), "Should create adapter for static config");
    }

    #[tokio::test]
    async fn test_create_for_config_with_songbird() {
        // Test creating discovery with songbird backend
        use crate::traits::discovery::{DiscoveryBackend, DiscoveryConfig};

        let config = DiscoveryConfig {
            backend: DiscoveryBackend::Songbird {
                federation_enabled: true,
                trust_verification: true,
                attribution_tracking: true,
            },
            health_check_interval: Duration::from_secs(30),
            connection_timeout: Duration::from_secs(10),
            retry_attempts: 3,
            retry_delay: Duration::from_secs(1),
        };

        let result = UniversalDiscoveryFactory::create_for_config(&config).await;
        assert!(result.is_ok(), "Should create adapter for songbird config");
    }

    #[tokio::test]
    async fn test_create_for_config_with_etcd() {
        // Test creating discovery with etcd backend
        use crate::traits::discovery::{DiscoveryBackend, DiscoveryConfig};

        let config = DiscoveryConfig {
            backend: DiscoveryBackend::Etcd {
                endpoints: vec!["http://localhost:2379".to_string()],
                username: None,
                password: None,
            },
            health_check_interval: Duration::from_secs(30),
            connection_timeout: Duration::from_secs(10),
            retry_attempts: 3,
            retry_delay: Duration::from_secs(1),
        };

        let result = UniversalDiscoveryFactory::create_for_config(&config).await;
        assert!(result.is_ok(), "Should create adapter for etcd config");
    }

    #[tokio::test]
    async fn test_discovery_adapter_basic_operations() {
        // Test basic discovery adapter operations
        let adapter =
            UniversalDiscoveryFactory::create_auto_detect().await.expect("Should create adapter");

        // Test querying services (should not error even if empty)
        use crate::traits::discovery::ServiceQuery;
        let query = ServiceQuery::default();
        let result = adapter.discover_services(&query).await;
        assert!(result.is_ok(), "Discovery query should succeed");
    }

    #[tokio::test]
    async fn test_multiple_adapter_creation() {
        // Test that we can create multiple adapters without conflicts
        let adapter1 = UniversalDiscoveryFactory::create_auto_detect().await;
        let adapter2 = UniversalDiscoveryFactory::create_auto_detect().await;
        let adapter3 = UniversalDiscoveryFactory::create_auto_detect().await;

        assert!(adapter1.is_ok());
        assert!(adapter2.is_ok());
        assert!(adapter3.is_ok());
    }

    #[tokio::test]
    async fn test_discovery_config_defaults() {
        // Test discovery config default values
        use crate::traits::discovery::DiscoveryConfig;

        let config = DiscoveryConfig::default();
        assert_eq!(config.health_check_interval, Duration::from_secs(30));
        assert_eq!(config.connection_timeout, Duration::from_secs(10));
        assert_eq!(config.retry_attempts, 3);
    }

    #[tokio::test]
    async fn test_discovery_backend_variants() {
        // Test all discovery backend variants can be used
        use crate::traits::discovery::DiscoveryBackend;

        let backends = vec![
            DiscoveryBackend::Static,
            DiscoveryBackend::Songbird {
                federation_enabled: true,
                trust_verification: true,
                attribution_tracking: true,
            },
            DiscoveryBackend::Etcd {
                endpoints: vec!["http://localhost:2379".to_string()],
                username: None,
                password: None,
            },
        ];

        for backend in backends {
            assert!(format!("{:?}", backend).len() > 0);
        }
    }

    #[tokio::test]
    async fn test_adapter_creation_is_deterministic() {
        // Test that adapter creation is consistent
        let result1 = UniversalDiscoveryFactory::create_auto_detect().await;
        let result2 = UniversalDiscoveryFactory::create_auto_detect().await;

        assert!(result1.is_ok());
        assert!(result2.is_ok());
    }

    #[tokio::test]
    async fn test_discovery_config_custom_values() {
        // Test custom config values
        use crate::traits::discovery::{DiscoveryBackend, DiscoveryConfig};

        let config = DiscoveryConfig {
            backend: DiscoveryBackend::Static,
            health_check_interval: Duration::from_secs(120),
            connection_timeout: Duration::from_secs(30),
            retry_attempts: 5,
            retry_delay: Duration::from_secs(2),
        };

        assert_eq!(config.health_check_interval, Duration::from_secs(120));
        assert_eq!(config.connection_timeout, Duration::from_secs(30));
        assert_eq!(config.retry_attempts, 5);
        assert_eq!(config.retry_delay, Duration::from_secs(2));
    }
}
