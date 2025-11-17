//! Comprehensive tests for performance monitoring

use songbird_orchestrator::core::performance::{PerformanceMetrics, PerformanceMonitor};
use songbird_orchestrator::core::{HealthStatus, PerformanceConfig};
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;

#[tokio::test]
async fn test_performance_monitor_creation() -> SongbirdResult<()> {
    let config = PerformanceConfig::default();
    let monitor = PerformanceMonitor::new(config);
    assert!(format!("{monitor:?}").contains("PerformanceMonitor"));
    Ok(())
}

#[tokio::test]
async fn test_performance_monitor_initialize() {
    let config = PerformanceConfig::default();
    let mut monitor = PerformanceMonitor::new(config);
    let result = monitor.initialize().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_performance_monitor_start_stop() -> SongbirdResult<()> {
    let config = PerformanceConfig::default();
    let mut monitor = PerformanceMonitor::new(config);

    let start_result = monitor.start().await;
    assert!(start_result.is_ok());

    let stop_result = monitor.stop().await;
    assert!(stop_result.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_performance_monitor_health_check() -> SongbirdResult<()> {
    let config = PerformanceConfig::default();
    let monitor = PerformanceMonitor::new(config);

    let health = monitor.health_check().await;
    assert!(health.is_ok());

    let health_status =
        health.ok_or_else(|| SongbirdError::configuration("Failed health check".to_string()))?;
    assert_eq!(health_status.status, HealthStatus::Healthy);
    assert!(health_status.message.is_some());
    assert!(health_status.last_check.is_some());
    Ok(())
}

#[tokio::test]
async fn test_performance_monitor_get_metrics() {
    let config = PerformanceConfig::default();
    let monitor = PerformanceMonitor::new(config);

    let metrics = monitor.get_metrics();
    assert_eq!(metrics.cpu_usage, 0.0);
    assert_eq!(metrics.memory_usage, 0.0);
    assert_eq!(metrics.response_time, 0.0);
    assert_eq!(metrics.throughput, 0.0);
    assert_eq!(metrics.error_rate, 0.0);
}

#[test]
fn test_performance_metrics_creation() -> SongbirdResult<()> {
    let metrics = PerformanceMetrics {
        cpu_usage: 45.5,
        memory_usage: 60.2,
        response_time: 150.0,
        throughput: 1000.0,
        error_rate: 0.01,
    };

    assert_eq!(metrics.cpu_usage, 45.5);
    assert_eq!(metrics.memory_usage, 60.2);
    assert_eq!(metrics.response_time, 150.0);
    assert_eq!(metrics.throughput, 1000.0);
    assert_eq!(metrics.error_rate, 0.01);
    Ok(())
}

#[test]
fn test_performance_metrics_serialization() -> SongbirdResult<()> {
    let metrics = PerformanceMetrics {
        cpu_usage: 50.0,
        memory_usage: 70.0,
        response_time: 200.0,
        throughput: 500.0,
        error_rate: 0.05,
    };

    let json = serde_json::to_string(&metrics)
        .map_err(|e| SongbirdError::configuration(format!("Failed to serialize: {}", e)))?;
    let deserialized: PerformanceMetrics =
        serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Failed to deserialize: {}", e),
            debug_info: None,
        })?;

    assert_eq!(metrics.cpu_usage, deserialized.cpu_usage);
    assert_eq!(metrics.memory_usage, deserialized.memory_usage);
    assert_eq!(metrics.response_time, deserialized.response_time);
    Ok(())
}

#[test]
fn test_performance_config_default() {
    let config = PerformanceConfig::default();
    assert_eq!(config.metrics_interval, 60);
    assert!(config.enable_benchmarking);
    assert!(config.alert_thresholds.contains_key("cpu_usage"));
    assert!(config.alert_thresholds.contains_key("memory_usage"));
    assert!(config.alert_thresholds.contains_key("response_time"));
}

#[test]
fn test_performance_config_custom() -> SongbirdResult<()> {
    let mut thresholds = HashMap::new();
    thresholds.insert("custom_metric".to_string(), 99.0);

    let config = PerformanceConfig {
        metrics_interval: 30,
        alert_thresholds: thresholds.clone(),
        enable_benchmarking: false,
    };

    assert_eq!(config.metrics_interval, 30);
    assert!(!config.enable_benchmarking);
    assert_eq!(config.alert_thresholds.get("custom_metric"), Some(&99.0));
    Ok(())
}

#[test]
fn test_performance_config_serialization() -> SongbirdResult<()> {
    let config = PerformanceConfig::default();
    let json = serde_json::to_string(&config)
        .map_err(|e| SongbirdError::configuration(format!("Failed to serialize: {}", e)))?;
    let deserialized: PerformanceConfig =
        serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Failed to deserialize: {}", e),
            debug_info: None,
        })?;

    assert_eq!(config.metrics_interval, deserialized.metrics_interval);
    assert_eq!(config.enable_benchmarking, deserialized.enable_benchmarking);
    Ok(())
}

#[tokio::test]
async fn test_performance_monitor_lifecycle() -> SongbirdResult<()> {
    let config = PerformanceConfig::default();
    let mut monitor = PerformanceMonitor::new(config);

    // Initialize
    assert!(monitor.initialize().await.is_ok());

    // Start
    assert!(monitor.start().await.is_ok());

    // Check metrics
    let metrics = monitor.get_metrics();
    assert!(metrics.cpu_usage >= 0.0);

    // Health check
    let health = monitor.health_check().await;
    assert!(health.is_ok());
    assert_eq!(
        health.ok_or_else(|| SongbirdError::configuration(format!("Error: {}", e)))?.status,
        HealthStatus::Healthy
    );

    // Stop
    assert!(monitor.stop().await.is_ok());
    Ok(())
}

#[test]
fn test_performance_metrics_clone() -> SongbirdResult<()> {
    let metrics = PerformanceMetrics {
        cpu_usage: 10.0,
        memory_usage: 20.0,
        response_time: 30.0,
        throughput: 40.0,
        error_rate: 0.5,
    };

    let cloned = metrics.clone();
    assert_eq!(metrics.cpu_usage, cloned.cpu_usage);
    assert_eq!(metrics.memory_usage, cloned.memory_usage);
    assert_eq!(metrics.response_time, cloned.response_time);
    assert_eq!(metrics.throughput, cloned.throughput);
    assert_eq!(metrics.error_rate, cloned.error_rate);
    Ok(())
}

#[tokio::test]
async fn test_performance_monitor_health_message() -> SongbirdResult<()> {
    let config = PerformanceConfig::default();
    let monitor = PerformanceMonitor::new(config);

    let health = monitor
        .health_check()
        .await
        .map_err(|e| SongbirdError::configuration("Failed health check".to_string()))?;
    let message = health
        .message
        .ok_or_else(|| SongbirdError::configuration("Failed health check".to_string()))?;
    assert!(message.contains("Performance"));
    assert!(message.contains("monitor"));
    Ok(())
}

#[test]
fn test_performance_config_thresholds() -> SongbirdResult<()> {
    let config = PerformanceConfig::default();

    // Check default thresholds are reasonable
    let cpu_threshold = config.alert_thresholds.get("cpu_usage");
    assert!(cpu_threshold.is_some());
    assert!(
        *cpu_threshold.ok_or_else(|| SongbirdError::configuration(format!("Error: {}", e)))? > 0.0
    );
    assert!(
        *cpu_threshold.ok_or_else(|| SongbirdError::configuration(format!("Error: {}", e)))?
            <= 100.0
    );

    let memory_threshold = config.alert_thresholds.get("memory_usage");
    assert!(memory_threshold.is_some());
    assert!(
        *memory_threshold.ok_or_else(|| SongbirdError::configuration(format!("Error: {}", e)))?
            > 0.0
    );
    assert!(
        *memory_threshold.ok_or_else(|| SongbirdError::configuration(format!("Error: {}", e)))?
            <= 100.0
    );
    Ok(())
}

#[test]
fn test_performance_metrics_all_fields_initialized() -> SongbirdResult<()> {
    let config = PerformanceConfig::default();
    let monitor = PerformanceMonitor::new(config);
    let metrics = monitor.get_metrics();

    // All fields should be initialized (even if to 0.0)
    assert!(metrics.cpu_usage.is_finite());
    assert!(metrics.memory_usage.is_finite());
    assert!(metrics.response_time.is_finite());
    assert!(metrics.throughput.is_finite());
    assert!(metrics.error_rate.is_finite());
    Ok(())
}

#[tokio::test]
async fn test_performance_monitor_operations_are_async() -> SongbirdResult<()> {
    let config = PerformanceConfig::default();
    let mut monitor = PerformanceMonitor::new(config);

    let start_time = std::time::Instant::now();
    monitor.initialize().await.map_err(|e| {
        SongbirdError::configuration("Failed to initialize orchestrator".to_string())
    })?;
    monitor.start().await.map_err(|e| {
        SongbirdError::configuration("Failed to initialize orchestrator".to_string())
    })?;
    let elapsed = start_time.elapsed();

    // Operations should complete quickly
    assert!(elapsed.as_millis() < 100);
    Ok(())
}

#[test]
fn test_performance_config_benchmarking_flag() {
    let config_enabled = PerformanceConfig {
        metrics_interval: 60,
        alert_thresholds: HashMap::new(),
        enable_benchmarking: true,
    };
    assert!(config_enabled.enable_benchmarking);

    let config_disabled = PerformanceConfig {
        metrics_interval: 60,
        alert_thresholds: HashMap::new(),
        enable_benchmarking: false,
    };
    assert!(!config_disabled.enable_benchmarking);
}

#[tokio::test]
async fn test_performance_monitor_with_custom_config() {
    let mut thresholds = HashMap::new();
    thresholds.insert("cpu_usage".to_string(), 90.0);

    let config = PerformanceConfig {
        metrics_interval: 120,
        alert_thresholds: thresholds,
        enable_benchmarking: false,
    };

    let mut monitor = PerformanceMonitor::new(config);
    assert!(monitor.initialize().await.is_ok());
    assert!(monitor.start().await.is_ok());
    assert!(monitor.stop().await.is_ok());
}
