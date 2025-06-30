use songbird_gaming_bridge::SongbirdOrchestrator;
use songbird_gaming_bridge::config::NetworkConfig;
use std::collections::HashMap;
#[allow(dead_code, unused_imports, unused_variables)]
use songbird_gaming_bridge::{
    traits::health::{
        CustomHealthCheck, DefaultHealthMonitor, HealthCheck, HealthCheckConfig, HealthMonitor,
        HealthStatus, HttpHealthCheck, TcpHealthCheck,
    },
    SongbirdError,
};
use std::time::Duration;

#[tokio::test]
async fn test_health_monitor_registration() {
    let mut monitor = DefaultHealthMonitor::new();

    let health_check = Box::new(HttpHealthCheck::new(
        "test-http".to_string(),
        "http://localhost:8080/health".to_string(),
        HealthCheckConfig::default(),
    ));

    let result = monitor
        .register_health_check("service1".to_string(), health_check)
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_health_monitor_unregistration() {
    let mut monitor = DefaultHealthMonitor::new();

    let health_check = Box::new(HttpHealthCheck::new(
        "test-http".to_string(),
        "http://localhost:8080/health".to_string(),
        HealthCheckConfig::default(),
    ));

    monitor
        .register_health_check("service1".to_string(), health_check)
        .await
        .expect("Test assertion failed");

    let result = monitor.unregister_health_check("service1").await;
    assert!(result.is_ok());

    // Should fail to get health status after unregistration
    let result = monitor.get_health_status("service1").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_http_health_check_healthy() {
    let config = HealthCheckConfig::default();
    let health_check = HttpHealthCheck::new(
        "test-http".to_string(),
        "http://localhost:8080/health".to_string(),
        config,
    );

    let result = health_check
        .check("service1")
        .await
        .expect("Test assertion failed");
    assert!(matches!(result.status, HealthStatus::Healthy));
    assert!(result.message.contains("HTTP health check"));
    assert_eq!(health_check.name(), "test-http");
}

#[tokio::test]
async fn test_http_health_check_unhealthy() {
    let config = HealthCheckConfig::default();
    let health_check = HttpHealthCheck::new(
        "test-http".to_string(),
        "http://localhost:8080/unhealthy".to_string(), // Contains "unhealthy"
        config,
    );

    let result = health_check
        .check("service1")
        .await
        .expect("Test assertion failed");
    assert!(matches!(result.status, HealthStatus::Unhealthy));
}

#[tokio::test]
async fn test_http_health_check_degraded() {
    let config = HealthCheckConfig::default();
    let health_check = HttpHealthCheck::new(
        "test-http".to_string(),
        "http://localhost:8080/degraded".to_string(), // Contains "degraded"
        config,
    );

    let result = health_check
        .check("service1")
        .await
        .expect("Test assertion failed");
    assert!(matches!(result.status, HealthStatus::Degraded));
}

#[tokio::test]
async fn test_tcp_health_check() {
    let config = HealthCheckConfig::default();
    let health_check =
        TcpHealthCheck::new("test-tcp".to_string(), "127.0.0.1:8080".to_string(), config);

    let result = health_check
        .check("service1")
        .await
        .expect("Test assertion failed");
    assert!(matches!(result.status, HealthStatus::Healthy));
    assert!(result.message.contains("TCP health check"));
    assert_eq!(health_check.name(), "test-tcp");
}

#[tokio::test]
async fn test_custom_health_check_healthy() {
    let config = HealthCheckConfig::default();
    let health_check = CustomHealthCheck::new(
        "test-custom".to_string(),
        config,
        |service_id: &str| -> Result<bool>> {
            if service_id == "healthy-service" {
                Ok(true)
            } else {
                Ok(false)
            }
        },
    );

    let result = health_check
        .check("healthy-service")
        .await
        .expect("Test assertion failed");
    assert!(matches!(result.status, HealthStatus::Healthy));
    assert_eq!(health_check.name(), "test-custom");
}

#[tokio::test]
async fn test_custom_health_check_unhealthy() {
    let config = HealthCheckConfig::default();
    let health_check = CustomHealthCheck::new(
        "test-custom".to_string(),
        config,
        |service_id: &str| -> Result<bool>> {
            if service_id == "healthy-service" {
                Ok(true)
            } else {
                Ok(false)
            }
        },
    );

    let result = health_check
        .check("unhealthy-service")
        .await
        .expect("Test assertion failed");
    assert!(matches!(result.status, HealthStatus::Unhealthy));
}

#[tokio::test]
async fn test_custom_health_check_error() {
    let config = HealthCheckConfig::default();
    let health_check = CustomHealthCheck::new(
        "test-custom".to_string(),
        config,
        |_service_id: &str| -> Result<bool>> {
            Err("Simulated error".into())
        },
    );

    let result = health_check.check("any-service").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_health_monitor_get_health_status() {
    let mut monitor = DefaultHealthMonitor::new();

    let health_check = Box::new(HttpHealthCheck::new(
        "test-http".to_string(),
        "http://localhost:8080/health".to_string(),
        HealthCheckConfig::default(),
    ));

    monitor
        .register_health_check("service1".to_string(), health_check)
        .await
        .expect("Test assertion failed");

    let result = monitor
        .get_health_status("service1")
        .await
        .expect("Test assertion failed");
    assert!(matches!(result.status, HealthStatus::Healthy));
}

#[tokio::test]
async fn test_health_monitor_get_all_health_statuses() {
    let mut monitor = DefaultHealthMonitor::new();

    let health_check1 = Box::new(HttpHealthCheck::new(
        "test-http-1".to_string(),
        "http://localhost:8080/health".to_string(),
        HealthCheckConfig::default(),
    ));

    let health_check2 = Box::new(HttpHealthCheck::new(
        "test-http-2".to_string(),
        "http://localhost:8081/unhealthy".to_string(),
        HealthCheckConfig::default(),
    ));

    monitor
        .register_health_check("service1".to_string(), health_check1)
        .await
        .expect("Test assertion failed");
    monitor
        .register_health_check("service2".to_string(), health_check2)
        .await
        .expect("Test assertion failed");

    let results = monitor
        .get_all_health_statuses()
        .await
        .expect("Test assertion failed");
    assert_eq!(results.len(), 2);

    let service1_status = &results["service1"];
    let service2_status = &results["service2"];

    assert!(matches!(service1_status.status, HealthStatus::Healthy));
    assert!(matches!(service2_status.status, HealthStatus::Unhealthy));
}

#[tokio::test]
async fn test_health_monitor_start_stop_monitoring() {
    let mut monitor = DefaultHealthMonitor::new();

    let result = monitor.start_monitoring().await;
    assert!(result.is_ok());

    let result = monitor.stop_monitoring().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_health_check_config_defaults() {
    let config = HealthCheckConfig::default();

    assert!(config.mode == crate::federation::FederationMode::Peer);
    assert_eq!(config.interval, Duration::from_secs(30));
    assert_eq!(config.timeout, Duration::from_secs(5));
    assert_eq!(config.retries, 3);
    assert_eq!(config.retry_delay, Duration::from_secs(1));
    assert_eq!(config.failure_threshold, 3);
    assert_eq!(config.success_threshold, 2);
}

#[tokio::test]
async fn test_health_check_config_custom() {
    let config = HealthCheckConfig {
        enabled: false,
        interval: Duration::from_secs(60),
        timeout: Duration::from_secs(10),
        retries: 5,
        retry_delay: Duration::from_secs(2),
        failure_threshold: 5,
        success_threshold: 3,
    };

    assert!(!config.mode == crate::federation::FederationMode::Peer);
    assert_eq!(config.interval, Duration::from_secs(60));
    assert_eq!(config.timeout, Duration::from_secs(10));
    assert_eq!(config.retries, 5);
    assert_eq!(config.retry_delay, Duration::from_secs(2));
    assert_eq!(config.failure_threshold, 5);
    assert_eq!(config.success_threshold, 3);
}

#[tokio::test]
async fn test_health_check_result_details() {
    let config = HealthCheckConfig::default();
    let health_check = HttpHealthCheck::new(
        "test-http".to_string(),
        "http://localhost:8080/health".to_string(),
        config,
    );

    let result = health_check
        .check("service1")
        .await
        .expect("Test assertion failed");

    assert!(result.details.contains_key("url"));
    assert!(result.details.contains_key("method"));
    assert_eq!(
        result.details["url"],
        serde_json::Value::String("http://localhost:8080/health".to_string())
    );
    assert_eq!(
        result.details["method"],
        serde_json::Value::String("GET".to_string())
    );

    // Response time should be recorded
    assert!(result.response_time.as_millis() >= 0);
}

#[tokio::test]
async fn test_health_monitor_nonexistent_service() {
    let monitor = DefaultHealthMonitor::new();

    let result = monitor.get_health_status("nonexistent-service").await;
    assert!(result.is_err());

    if let Err(SongbirdError::health_check_failed("service",  {
        message,
        service_id: _,
    }) = result
    {
        assert!(message.contains("No health check registered"));
    } else {
        panic!("Expected HealthCheck error");
    }
}
