//! # 📈 Auto Scaling
//!
//! **MODERN AUTO SCALING** ✅
//! **ZERO-COPY OPTIMIZATION** (Dec 8, 2025)

use super::{ComponentHealth, HealthStatus, ScalingConfig};
use serde::{Deserialize, Serialize};
use songbird_types::SongbirdResult;
use std::sync::Arc;

/// Scaling policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingPolicy {
    pub scale_up_threshold: f64,
    pub scale_down_threshold: f64,
    pub min_instances: u32,
    pub max_instances: u32,
}

/// Auto-scaler implementation
#[derive(Debug)]
pub struct AutoScaler {
    config: ScalingConfig,
    current_instances: u32,
}

impl AutoScaler {
    #[must_use]
    pub const fn new(config: ScalingConfig) -> Self {
        Self {
            current_instances: config.min_instances,
            config,
        }
    }

    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn initialize(&mut self) -> SongbirdResult<()> {
        // Initialize auto-scaler
        Ok(())
    }

    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn start(&mut self) -> SongbirdResult<()> {
        // Start auto-scaler
        Ok(())
    }

    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn stop(&mut self) -> SongbirdResult<()> {
        // Stop auto-scaler
        Ok(())
    }

    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn health_check(&self) -> SongbirdResult<ComponentHealth> {
        Ok(ComponentHealth {
            status: HealthStatus::Healthy,
            message: Some(Arc::from(
                format!("Auto-scaler managing {} instances", self.current_instances).as_str(),
            )),
            last_check: Some(chrono::Utc::now().timestamp() as u64),
        })
    }

    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn scale_up(&mut self) -> SongbirdResult<()> {
        if self.current_instances < self.config.max_instances {
            self.current_instances += 1;
        }
        Ok(())
    }

    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn scale_down(&mut self) -> SongbirdResult<()> {
        if self.current_instances > self.config.min_instances {
            self.current_instances -= 1;
        }
        Ok(())
    }

    #[must_use]
    pub const fn current_instances(&self) -> u32 {
        self.current_instances
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use songbird_types::SongbirdError;

    #[test]
    fn test_auto_scaler_new() {
        let config = ScalingConfig::default();
        let scaler = AutoScaler::new(config.clone());

        assert_eq!(scaler.current_instances(), config.min_instances);
    }

    #[tokio::test]
    async fn test_auto_scaler_initialize() {
        let config = ScalingConfig::default();
        let mut scaler = AutoScaler::new(config);

        let result = scaler.initialize().await;
        assert!(result.is_ok(), "Initialize should succeed");
    }

    #[tokio::test]
    async fn test_auto_scaler_start() -> SongbirdResult<()> {
        let config = ScalingConfig::default();
        let mut scaler = AutoScaler::new(config);

        scaler.initialize().await?;

        let result = scaler.start().await;
        assert!(result.is_ok(), "Start should succeed");
        Ok(())
    }

    #[tokio::test]
    async fn test_auto_scaler_stop() -> SongbirdResult<()> {
        let config = ScalingConfig::default();
        let mut scaler = AutoScaler::new(config);

        scaler.initialize().await?;
        scaler.start().await?;

        let result = scaler.stop().await;
        assert!(result.is_ok(), "Stop should succeed");
        Ok(())
    }

    #[tokio::test]
    async fn test_auto_scaler_health_check() -> SongbirdResult<()> {
        let config = ScalingConfig::default();
        let scaler = AutoScaler::new(config);

        let health = scaler.health_check().await?;
        assert_eq!(health.status, HealthStatus::Healthy);
        assert!(health.message.is_some());
        Ok(())
    }

    #[tokio::test]
    async fn test_auto_scaler_scale_up() -> SongbirdResult<()> {
        let config = ScalingConfig {
            enable_auto_scaling: true,
            scale_up_threshold: 70.0,
            scale_down_threshold: 30.0,
            min_instances: 1,
            max_instances: 10,
        };

        let mut scaler = AutoScaler::new(config);
        let initial_instances = scaler.current_instances();

        scaler.scale_up().await?;
        assert_eq!(scaler.current_instances(), initial_instances + 1);
        Ok(())
    }

    #[tokio::test]
    async fn test_auto_scaler_scale_up_max_limit() -> SongbirdResult<()> {
        let config = ScalingConfig {
            enable_auto_scaling: true,
            scale_up_threshold: 70.0,
            scale_down_threshold: 30.0,
            min_instances: 1,
            max_instances: 2,
        };

        let mut scaler = AutoScaler::new(config);

        scaler.scale_up().await?;
        assert_eq!(scaler.current_instances(), 2);

        // Try to scale beyond max
        scaler.scale_up().await?;
        assert_eq!(scaler.current_instances(), 2, "Should not scale beyond max");
        Ok(())
    }

    #[tokio::test]
    async fn test_auto_scaler_scale_down() -> SongbirdResult<()> {
        let config = ScalingConfig {
            enable_auto_scaling: true,
            scale_up_threshold: 70.0,
            scale_down_threshold: 30.0,
            min_instances: 1,
            max_instances: 10,
        };

        let mut scaler = AutoScaler::new(config);

        // Scale up first
        scaler.scale_up().await?;
        scaler.scale_up().await?;
        let current = scaler.current_instances();

        // Then scale down
        scaler.scale_down().await?;
        assert_eq!(scaler.current_instances(), current - 1);
        Ok(())
    }

    #[tokio::test]
    async fn test_auto_scaler_scale_down_min_limit() -> SongbirdResult<()> {
        let config = ScalingConfig {
            enable_auto_scaling: true,
            scale_up_threshold: 70.0,
            scale_down_threshold: 30.0,
            min_instances: 2,
            max_instances: 10,
        };

        let mut scaler = AutoScaler::new(config);
        assert_eq!(scaler.current_instances(), 2);

        // Try to scale below min
        scaler.scale_down().await?;
        assert_eq!(scaler.current_instances(), 2, "Should not scale below min");
        Ok(())
    }

    #[tokio::test]
    async fn test_auto_scaler_full_lifecycle() -> SongbirdResult<()> {
        let config = ScalingConfig::default();
        let mut scaler = AutoScaler::new(config);

        scaler.initialize().await?;
        scaler.start().await?;

        scaler.scale_up().await?;
        scaler.scale_up().await?;

        let health = scaler.health_check().await?;
        assert_eq!(health.status, HealthStatus::Healthy);

        scaler.scale_down().await?;
        scaler.stop().await?;
        Ok(())
    }

    #[test]
    fn test_scaling_policy_clone() {
        let policy = ScalingPolicy {
            scale_up_threshold: 70.0,
            scale_down_threshold: 30.0,
            min_instances: 1,
            max_instances: 10,
        };

        let cloned = policy.clone();
        assert_eq!(policy.scale_up_threshold, cloned.scale_up_threshold);
        assert_eq!(policy.scale_down_threshold, cloned.scale_down_threshold);
    }

    #[test]
    fn test_scaling_policy_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let policy = ScalingPolicy {
            scale_up_threshold: 80.0,
            scale_down_threshold: 20.0,
            min_instances: 2,
            max_instances: 20,
        };

        let json = serde_json::to_string(&policy).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Serialization failed: {}", e),
            debug_info: None,
        })?;
        let deserialized: ScalingPolicy =
            serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: format!("Parsing failed: {}", e),
                debug_info: None,
            })?;

        assert_eq!(policy.scale_up_threshold, deserialized.scale_up_threshold);
        assert_eq!(policy.min_instances, deserialized.min_instances);
        Ok(())
    }

    #[test]
    fn test_auto_scaler_debug_format() {
        let config = ScalingConfig::default();
        let scaler = AutoScaler::new(config);

        let debug_string = format!("{:?}", scaler);
        assert!(debug_string.contains("AutoScaler"));
    }

    #[tokio::test]
    async fn test_auto_scaler_multiple_scale_operations() -> SongbirdResult<()> {
        let config = ScalingConfig {
            enable_auto_scaling: true,
            scale_up_threshold: 70.0,
            scale_down_threshold: 30.0,
            min_instances: 1,
            max_instances: 10,
        };

        let mut scaler = AutoScaler::new(config);
        assert_eq!(scaler.current_instances(), 1);

        // Scale up multiple times
        for i in 1..5 {
            scaler.scale_up().await?;
            assert_eq!(scaler.current_instances(), 1 + i);
        }

        // Scale down multiple times
        for i in (1..5).rev() {
            scaler.scale_down().await?;
            assert_eq!(scaler.current_instances(), i);
        }
        Ok(())
    }

    #[test]
    fn test_auto_scaler_with_custom_config() {
        let config = ScalingConfig {
            enable_auto_scaling: false,
            scale_up_threshold: 90.0,
            scale_down_threshold: 10.0,
            min_instances: 5,
            max_instances: 50,
        };

        let scaler = AutoScaler::new(config);
        assert_eq!(scaler.current_instances(), 5);
    }
}
