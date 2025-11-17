//! Comprehensive tests for auto-scaling functionality

use songbird_orchestrator::core::scaling::{AutoScaler, ScalingPolicy};
use songbird_orchestrator::core::{HealthStatus, ScalingConfig};
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_types::{SongbirdError, SongbirdResult};

#[tokio::test]
async fn test_auto_scaler_creation() -> SongbirdResult<()> {
    let config = ScalingConfig::default();
    let scaler = AutoScaler::new(config);
    assert!(format!("{scaler:?}").contains("AutoScaler"));
    Ok(())
}

#[tokio::test]
async fn test_auto_scaler_initialize() {
    let config = ScalingConfig::default();
    let mut scaler = AutoScaler::new(config);
    let result = scaler.initialize().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_auto_scaler_start_stop() -> SongbirdResult<()> {
    let config = ScalingConfig::default();
    let mut scaler = AutoScaler::new(config);

    let start_result = scaler.start().await;
    assert!(start_result.is_ok());

    let stop_result = scaler.stop().await;
    assert!(stop_result.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_auto_scaler_health_check() -> SongbirdResult<()> {
    let config = ScalingConfig::default();
    let scaler = AutoScaler::new(config);

    let health = scaler.health_check().await;
    assert!(health.is_ok());

    let health_status =
        health.ok_or_else(|| SongbirdError::configuration("Failed health check".to_string()))?;
    assert_eq!(health_status.status, HealthStatus::Healthy);
    assert!(health_status.message.is_some());
    assert!(health_status.last_check.is_some());
    Ok(())
}

#[test]
fn test_scaling_config_default() {
    let config = ScalingConfig::default();
    assert!(config.enable_auto_scaling);
    assert_eq!(config.scale_up_threshold, 70.0);
    assert_eq!(config.scale_down_threshold, 30.0);
    assert_eq!(config.min_instances, 1);
    assert_eq!(config.max_instances, 10);
}

#[test]
fn test_scaling_config_custom() {
    let config = ScalingConfig {
        enable_auto_scaling: false,
        scale_up_threshold: 80.0,
        scale_down_threshold: 20.0,
        min_instances: 2,
        max_instances: 20,
    };

    assert!(!config.enable_auto_scaling);
    assert_eq!(config.scale_up_threshold, 80.0);
    assert_eq!(config.scale_down_threshold, 20.0);
    assert_eq!(config.min_instances, 2);
    assert_eq!(config.max_instances, 20);
}

#[test]
fn test_scaling_config_thresholds_valid() -> SongbirdResult<()> {
    let config = ScalingConfig::default();
    assert!(
        config.scale_up_threshold > config.scale_down_threshold,
        "Scale up threshold should be higher than scale down threshold"
    );
    Ok(())
}

#[test]
fn test_scaling_config_instances_valid() -> SongbirdResult<()> {
    let config = ScalingConfig::default();
    assert!(
        config.min_instances <= config.max_instances,
        "Min instances should be <= max instances"
    );
    assert!(config.min_instances > 0, "Min instances should be positive");
    Ok(())
}

#[test]
fn test_scaling_config_serialization() -> SongbirdResult<()> {
    let config = ScalingConfig::default();
    let json = serde_json::to_string(&config)
        .map_err(|e| SongbirdError::configuration(format!("Failed to serialize: {}", e)))?;
    let deserialized: ScalingConfig =
        serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Failed to deserialize: {}", e),
            debug_info: None,
        })?;

    assert_eq!(config.enable_auto_scaling, deserialized.enable_auto_scaling);
    assert_eq!(config.scale_up_threshold, deserialized.scale_up_threshold);
    assert_eq!(config.scale_down_threshold, deserialized.scale_down_threshold);
    Ok(())
}

#[test]
fn test_scaling_policy_creation() -> SongbirdResult<()> {
    let policy = ScalingPolicy {
        scale_up_threshold: 75.0,
        scale_down_threshold: 25.0,
        min_instances: 2,
        max_instances: 15,
    };

    assert_eq!(policy.scale_up_threshold, 75.0);
    assert_eq!(policy.scale_down_threshold, 25.0);
    assert_eq!(policy.min_instances, 2);
    assert_eq!(policy.max_instances, 15);
    Ok(())
}

#[test]
fn test_scaling_policy_serialization() -> SongbirdResult<()> {
    let policy = ScalingPolicy {
        scale_up_threshold: 80.0,
        scale_down_threshold: 20.0,
        min_instances: 1,
        max_instances: 20,
    };

    let json = serde_json::to_string(&policy)
        .map_err(|e| SongbirdError::configuration(format!("Failed to serialize: {}", e)))?;
    let deserialized: ScalingPolicy =
        serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Failed to deserialize: {}", e),
            debug_info: None,
        })?;

    assert_eq!(policy.scale_up_threshold, deserialized.scale_up_threshold);
    assert_eq!(policy.scale_down_threshold, deserialized.scale_down_threshold);
    assert_eq!(policy.min_instances, deserialized.min_instances);
    Ok(())
}

#[tokio::test]
async fn test_auto_scaler_lifecycle() -> SongbirdResult<()> {
    let config = ScalingConfig::default();
    let mut scaler = AutoScaler::new(config);

    // Initialize
    assert!(scaler.initialize().await.is_ok());

    // Start
    assert!(scaler.start().await.is_ok());

    // Health check
    let health = scaler.health_check().await;
    assert!(health.is_ok());
    assert_eq!(
        health.ok_or_else(|| SongbirdError::configuration(format!("Error: {}", e)))?.status,
        HealthStatus::Healthy
    );

    // Stop
    assert!(scaler.stop().await.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_auto_scaler_with_scaling_disabled() -> SongbirdResult<()> {
    let config = ScalingConfig {
        enable_auto_scaling: false,
        scale_up_threshold: 70.0,
        scale_down_threshold: 30.0,
        min_instances: 1,
        max_instances: 10,
    };

    let mut scaler = AutoScaler::new(config);
    assert!(scaler.initialize().await.is_ok());
    assert!(scaler.start().await.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_auto_scaler_health_message() -> SongbirdResult<()> {
    let config = ScalingConfig::default();
    let scaler = AutoScaler::new(config);

    let health = scaler
        .health_check()
        .await
        .map_err(|e| SongbirdError::configuration("Failed to start orchestrator".to_string()))?;
    let message = health
        .message
        .ok_or_else(|| SongbirdError::configuration("Failed health check".to_string()))?;
    assert!(message.contains("Auto-scaler") || message.contains("instances"));
    Ok(())
}

#[test]
fn test_scaling_config_clone() {
    let config = ScalingConfig::default();
    let cloned = config.clone();

    assert_eq!(config.enable_auto_scaling, cloned.enable_auto_scaling);
    assert_eq!(config.scale_up_threshold, cloned.scale_up_threshold);
    assert_eq!(config.min_instances, cloned.min_instances);
}

#[test]
fn test_scaling_policy_clone() -> SongbirdResult<()> {
    let policy = ScalingPolicy {
        scale_up_threshold: 70.0,
        scale_down_threshold: 30.0,
        min_instances: 1,
        max_instances: 10,
    };

    let cloned = policy.clone();
    assert_eq!(policy.scale_up_threshold, cloned.scale_up_threshold);
    assert_eq!(policy.scale_down_threshold, cloned.scale_down_threshold);
    assert_eq!(policy.min_instances, cloned.min_instances);
    Ok(())
}

#[tokio::test]
async fn test_auto_scaler_operations_are_async() -> SongbirdResult<()> {
    let config = ScalingConfig::default();
    let mut scaler = AutoScaler::new(config);

    let start_time = std::time::Instant::now();
    scaler.initialize().await.map_err(|e| {
        SongbirdError::configuration("Failed to initialize orchestrator".to_string())
    })?;
    scaler.start().await.map_err(|e| {
        SongbirdError::configuration("Failed to initialize orchestrator".to_string())
    })?;
    let elapsed = start_time.elapsed();

    // Operations should complete quickly
    assert!(elapsed.as_millis() < 100);
    Ok(())
}

#[test]
fn test_scaling_thresholds_are_percentages() {
    let config = ScalingConfig::default();

    assert!(config.scale_up_threshold >= 0.0);
    assert!(config.scale_up_threshold <= 100.0);
    assert!(config.scale_down_threshold >= 0.0);
    assert!(config.scale_down_threshold <= 100.0);
}

#[test]
fn test_scaling_config_reasonable_defaults() {
    let config = ScalingConfig::default();

    // Thresholds should leave a reasonable gap
    let gap = config.scale_up_threshold - config.scale_down_threshold;
    assert!(gap >= 20.0, "Should have at least 20% gap between thresholds");

    // Max instances should be reasonable
    assert!(config.max_instances >= config.min_instances * 2, "Max should be at least 2x min");
}

#[tokio::test]
async fn test_auto_scaler_with_custom_thresholds() {
    let config = ScalingConfig {
        enable_auto_scaling: true,
        scale_up_threshold: 85.0,
        scale_down_threshold: 15.0,
        min_instances: 3,
        max_instances: 30,
    };

    let mut scaler = AutoScaler::new(config);
    assert!(scaler.initialize().await.is_ok());
    assert!(scaler.start().await.is_ok());
    assert!(scaler.stop().await.is_ok());
}

#[test]
fn test_scaling_policy_thresholds_valid() {
    let policy = ScalingPolicy {
        scale_up_threshold: 70.0,
        scale_down_threshold: 30.0,
        min_instances: 1,
        max_instances: 10,
    };

    assert!(policy.scale_up_threshold > policy.scale_down_threshold);
    assert!(policy.min_instances <= policy.max_instances);
}

#[test]
fn test_scaling_config_is_production_ready() {
    let config = ScalingConfig::default();

    // Verify defaults are appropriate for production
    assert!(config.scale_up_threshold <= 90.0, "Scale up shouldn't be too aggressive");
    assert!(config.scale_down_threshold >= 20.0, "Scale down shouldn't be too aggressive");
    assert!(config.min_instances >= 1, "Should have at least one instance");
}

#[tokio::test]
async fn test_auto_scaler_multiple_lifecycle_cycles() {
    let config = ScalingConfig::default();
    let mut scaler = AutoScaler::new(config);

    // First cycle
    assert!(scaler.initialize().await.is_ok());
    assert!(scaler.start().await.is_ok());
    assert!(scaler.stop().await.is_ok());

    // Can perform multiple stop/start cycles
    assert!(scaler.start().await.is_ok());
    assert!(scaler.stop().await.is_ok());
}
