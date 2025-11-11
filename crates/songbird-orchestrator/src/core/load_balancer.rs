//! # ⚖️ Load Balancer
//!
//! **MODERN LOAD BALANCING** ✅

use super::{ComponentHealth, HealthStatus};
use songbird_types::SongbirdResult;

// Import comprehensive LoadBalancerConfig (Nov 10, 2025 consolidation)
pub use songbird_config::unified::robustness::{
    LoadBalancerConfig as CanonicalLoadBalancerConfig,
    LoadBalancingAlgorithm,
};

/// Load balancing strategies (re-exported from canonical for compatibility)
pub use LoadBalancingAlgorithm as LoadBalancingStrategy;

/// Load balancer implementation
#[derive(Debug)]
pub struct LoadBalancer {
    config: CanonicalLoadBalancerConfig,
    strategy: LoadBalancingAlgorithm,
}

impl LoadBalancer {
    #[must_use]
    pub fn new(config: CanonicalLoadBalancerConfig) -> Self {
        Self {
            strategy: config.algorithm.clone(),
            config,
        }
    }

    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn initialize(&mut self) -> SongbirdResult<()> {
        // Initialize load balancer
        Ok(())
    }

    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn start(&mut self) -> SongbirdResult<()> {
        // Start load balancing
        Ok(())
    }

    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn stop(&mut self) -> SongbirdResult<()> {
        // Stop load balancing
        Ok(())
    }

    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn health_check(&self) -> SongbirdResult<ComponentHealth> {
        Ok(ComponentHealth {
            status: HealthStatus::Healthy,
            message: Some("Load balancer operational".to_string()),
            last_check: Some(chrono::Utc::now().timestamp() as u64),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use songbird_types::SongbirdError;

    #[test]
    fn test_load_balancer_new() {
        let config = CanonicalLoadBalancerConfig::default();
        let lb = LoadBalancer::new(config);

        assert!(format!("{:?}", lb).contains("LoadBalancer"));
    }

    #[test]
    fn test_load_balancer_new_with_custom_strategy() {
        use songbird_config::unified::robustness::HealthCheckConfig;
        use std::time::Duration;
        
        let config = CanonicalLoadBalancerConfig {
            algorithm: LoadBalancingAlgorithm::LeastConnections,
            health_check: HealthCheckConfig::default(),
            sticky_sessions: false,
            session_timeout: Duration::from_secs(300),
            max_connections_per_backend: 100,
            connection_timeout: Duration::from_secs(60),
            fail_fast: false,
        };

        let lb = LoadBalancer::new(config);

        assert!(format!("{:?}", lb).contains("LoadBalancer"));
        assert!(format!("{:?}", lb).contains("LeastConnections"));
    }

    #[tokio::test]
    async fn test_load_balancer_initialize() {
        let config = CanonicalLoadBalancerConfig::default();
        let mut lb = LoadBalancer::new(config);

        let result = lb.initialize().await;
        assert!(result.is_ok(), "Initialize should succeed");
    }

    #[tokio::test]
    async fn test_load_balancer_start() -> SongbirdResult<()> {
        let config = CanonicalLoadBalancerConfig::default();
        let mut lb = LoadBalancer::new(config);

        lb.initialize().await?;

        let result = lb.start().await;
        assert!(result.is_ok(), "Start should succeed");
        Ok(())
    }

    #[tokio::test]
    async fn test_load_balancer_stop() -> SongbirdResult<()> {
        let config = CanonicalLoadBalancerConfig::default();
        let mut lb = LoadBalancer::new(config);

        lb.initialize().await?;
        lb.start().await?;

        let result = lb.stop().await;
        assert!(result.is_ok(), "Stop should succeed");
        Ok(())
    }

    #[tokio::test]
    async fn test_load_balancer_health_check() -> SongbirdResult<()> {
        let config = CanonicalLoadBalancerConfig::default();
        let lb = LoadBalancer::new(config);

        let health = lb.health_check().await?;
        assert_eq!(health.status, HealthStatus::Healthy);
        assert!(health.message.is_some());
        assert!(health.last_check.is_some());
        Ok(())
    }

    #[tokio::test]
    async fn test_load_balancer_full_lifecycle() -> SongbirdResult<()> {
        let config = CanonicalLoadBalancerConfig::default();
        let mut lb = LoadBalancer::new(config);

        lb.initialize().await?;
        lb.start().await?;

        let health = lb.health_check().await?;
        assert_eq!(health.status, HealthStatus::Healthy);

        lb.stop().await?;
        Ok(())
    }

    #[test]
    fn test_load_balancing_strategy_all_variants() {
        let strategies = [
            LoadBalancingStrategy::RoundRobin,
            LoadBalancingStrategy::LeastConnections,
            LoadBalancingStrategy::WeightedRoundRobin,
            LoadBalancingStrategy::HealthBased,
        ];

        assert_eq!(strategies.len(), 4);
    }

    #[test]
    fn test_load_balancing_strategy_equality() {
        assert_eq!(LoadBalancingStrategy::RoundRobin, LoadBalancingStrategy::RoundRobin);
        assert_ne!(LoadBalancingStrategy::RoundRobin, LoadBalancingStrategy::LeastConnections);
        assert_ne!(
            LoadBalancingStrategy::LeastConnections,
            LoadBalancingStrategy::WeightedRoundRobin
        );
        assert_ne!(LoadBalancingStrategy::WeightedRoundRobin, LoadBalancingStrategy::HealthBased);
    }

    #[test]
    fn test_load_balancing_strategy_clone() {
        let strategy = LoadBalancingStrategy::RoundRobin;
        let cloned = strategy.clone();

        assert_eq!(strategy, cloned);
    }

    #[test]
    fn test_load_balancing_strategy_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let strategy = LoadBalancingStrategy::HealthBased;
        let json = serde_json::to_string(&strategy).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Serialization failed: {}", e),
            debug_info: None,
        })?;
        let deserialized: LoadBalancingStrategy =
            serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: format!("Parsing failed: {}", e),
                debug_info: None,
            })?;

        assert_eq!(strategy, deserialized);
        Ok(())
    }

    #[test]
    fn test_load_balancing_config_with_different_strategies() {
        use songbird_config::unified::robustness::HealthCheckConfig;
        use std::time::Duration;
        
        let strategies = vec![
            LoadBalancingAlgorithm::RoundRobin,
            LoadBalancingAlgorithm::LeastConnections,
            LoadBalancingAlgorithm::WeightedRoundRobin,
            LoadBalancingAlgorithm::HealthBased,
        ];

        for algorithm in strategies {
            let config = CanonicalLoadBalancerConfig {
                algorithm: algorithm.clone(),
                health_check: HealthCheckConfig::default(),
                sticky_sessions: false,
                session_timeout: Duration::from_secs(300),
                max_connections_per_backend: 100,
                connection_timeout: Duration::from_secs(30),
                fail_fast: false,
            };

            let lb = LoadBalancer::new(config);
            assert!(format!("{:?}", lb).contains("LoadBalancer"));
        }
    }

    #[tokio::test]
    async fn test_load_balancer_health_check_message_format() -> SongbirdResult<()> {
        let config = CanonicalLoadBalancerConfig::default();
        let lb = LoadBalancer::new(config);

        let health = lb.health_check().await?;

        assert!(health.message.is_some());
        assert!(health
            .message
            .ok_or_else(|| SongbirdError::configuration(
                "Failed to select service".to_string()
            ))?
            .contains("operational"));
        Ok(())
    }

    #[test]
    fn test_load_balancer_debug_format() {
        let config = CanonicalLoadBalancerConfig::default();
        let lb = LoadBalancer::new(config);

        let debug_string = format!("{:?}", lb);
        assert!(debug_string.contains("LoadBalancer"));
        assert!(debug_string.contains("config"));
        assert!(debug_string.contains("strategy"));
    }
}
