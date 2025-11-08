//! Metrics integration tests for comprehensive coverage

use songbird_observability::observability::metrics::*;
use songbird_observability::observability::*;
use songbird_types::SongbirdResult;

#[tokio::test]
async fn test_metrics_collector_default() {
    let collector1 = MetricsCollector::default();
    let collector2 = MetricsCollector::new();

    assert_eq!(collector1.get_collection_count(), collector2.get_collection_count());
}

#[tokio::test]
async fn test_metrics_collector_multiple_collections() {
    let collector = MetricsCollector::new();

    for _ in 0..5 {
        let result = collector.collect_all_metrics().await;
        assert!(result.is_ok());
    }

    assert_eq!(collector.get_collection_count(), 5);
}

#[tokio::test]
async fn test_metrics_snapshot_structure() {
    let collector = MetricsCollector::new();
    let result = collector.collect_all_metrics().await;

    assert!(result.is_ok());
    let snapshot = result.unwrap();

    // Verify snapshot structure (unsigned types are always >= 0, so we verify they exist)
    assert!(snapshot.system.cpu_usage >= 0.0);
    assert!(snapshot.system.memory_usage >= 0.0);
    // active_services and collection_duration_ms are unsigned, so >= 0 is always true
    // We just verify the snapshot has these fields by accessing them
    let _ = snapshot.songbird.active_services;
    let _ = snapshot.collection_duration_ms;
}

#[tokio::test]
async fn test_prometheus_export_format() {
    let collector = MetricsCollector::new();
    collector.collect_all_metrics().await.unwrap();

    let export = collector.export_prometheus().await.unwrap();

    // Verify Prometheus format
    assert!(export.contains("# HELP"));
    assert!(export.contains("# TYPE"));
    assert!(export.contains("songbird_"));
}

#[tokio::test]
async fn test_metrics_last_collection_time() {
    let collector = MetricsCollector::new();

    let time_before = collector.last_collection_time();
    assert!(time_before.is_some());

    collector.collect_all_metrics().await.unwrap();

    let time_after = collector.last_collection_time();
    assert!(time_after.is_some());
}

#[tokio::test]
async fn test_system_metrics_validation() {
    use chrono::Utc;
    use songbird_observability::observability::NetworkIO;

    let metrics = SystemMetrics {
        cpu_usage: 45.5,
        memory_usage: 0.75,
        disk_usage: 0.50,
        network_io: NetworkIO {
            bytes_in: 100_000,
            bytes_out: 50_000,
            packets_in: 1000,
            packets_out: 500,
        },
        timestamp: Utc::now(),
    };

    assert!((metrics.cpu_usage - 45.5).abs() < 0.001);
    assert!((metrics.memory_usage - 0.75).abs() < 0.001);
    assert!((metrics.disk_usage - 0.50).abs() < 0.001);
}

#[tokio::test]
async fn test_application_metrics_validation() {
    let metrics = ApplicationMetrics {
        active_services: 10,
        request_rate: 100.5,
        error_rate: 0.01,
        avg_response_time_ms: 25.3,
    };

    assert_eq!(metrics.active_services, 10);
    assert!(metrics.request_rate > 100.0);
    assert!(metrics.error_rate < 1.0);
    assert!(metrics.avg_response_time_ms > 0.0);
}

#[tokio::test]
async fn test_observability_manager_lifecycle() {
    let manager = ObservabilityManager::new();

    let start_result = manager.start().await;
    assert!(start_result.is_ok());

    let stop_result = manager.stop().await;
    assert!(stop_result.is_ok());
}

#[tokio::test]
async fn test_observability_manager_get_metrics() {
    let manager = ObservabilityManager::new();
    manager.start().await.unwrap();

    let metrics = manager.get_metrics().await;
    assert!(metrics.is_ok());

    let system_metrics = metrics.unwrap();
    assert!(system_metrics.cpu_usage >= 0.0);
}

#[tokio::test]
async fn test_health_check_recording() {
    let manager = ObservabilityManager::new();
    manager.start().await.unwrap();

    let result =
        manager.record_health_check("test-service".to_string(), HealthStatus::Healthy, 100).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_health_check_degraded() {
    let manager = ObservabilityManager::new();
    manager.start().await.unwrap();

    let result = manager
        .record_health_check("degraded-service".to_string(), HealthStatus::Degraded, 500)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_health_check_unhealthy() {
    let manager = ObservabilityManager::new();
    manager.start().await.unwrap();

    let result = manager
        .record_health_check("unhealthy-service".to_string(), HealthStatus::Unhealthy, 1000)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_event_subscription_system() {
    let manager = ObservabilityManager::new();

    let receiver1 = manager.subscribe_to_events().await;
    let receiver2 = manager.subscribe_to_events().await;

    // Multiple subscribers can be created
    drop(receiver1);
    drop(receiver2);
}

#[tokio::test]
async fn test_cluster_status_reporting() {
    let manager = ObservabilityManager::new();
    manager.start().await.unwrap();

    let cluster_status = manager.get_cluster_status().await;
    assert!(cluster_status.is_ok());
}

#[tokio::test]
async fn test_observability_manager_default_creation() {
    let manager1 = ObservabilityManager::new();
    let manager2 = ObservabilityManager::default();

    // Both should start successfully
    assert!(manager1.start().await.is_ok());
    assert!(manager2.start().await.is_ok());
}

#[tokio::test]
async fn test_metrics_collection_with_delays() {
    let collector = MetricsCollector::new();

    collector.collect_all_metrics().await.unwrap();
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    collector.collect_all_metrics().await.unwrap();
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    collector.collect_all_metrics().await.unwrap();

    assert_eq!(collector.get_collection_count(), 3);
}

#[tokio::test]
async fn test_service_health_tracking() {
    let manager = ObservabilityManager::new();
    manager.start().await.unwrap();

    // Record multiple health checks for same service
    for i in 0..5 {
        let health_status = if i % 2 == 0 {
            HealthStatus::Healthy
        } else {
            HealthStatus::Degraded
        };

        manager
            .record_health_check("test-service".to_string(), health_status, (i + 1) * 100)
            .await
            .unwrap();
    }

    // All recordings should succeed
}

#[tokio::test]
async fn test_multiple_services_health_tracking() -> SongbirdResult<()> {
    let manager = ObservabilityManager::new();
    manager.start().await.unwrap();

    let services = vec!["service1", "service2", "service3", "service4"];

    for service in services {
        manager.record_health_check(service.to_string(), HealthStatus::Healthy, 100).await.unwrap();
    }
    Ok(())
}

#[test]
fn test_health_status_clone() -> SongbirdResult<()> {
    let status = HealthStatus::Healthy;
    let cloned = status.clone();

    assert!(matches!(status, HealthStatus::Healthy));
    assert!(matches!(cloned, HealthStatus::Healthy));
    Ok(())
}

#[test]
fn test_health_status_debug() -> SongbirdResult<()> {
    let statuses = vec![HealthStatus::Healthy, HealthStatus::Degraded, HealthStatus::Unhealthy];

    for status in statuses {
        let debug_str = format!("{status:?}");
        assert!(!debug_str.is_empty());
    }
    Ok(())
}

#[test]
fn test_system_metrics_clone() {
    use chrono::Utc;
    use songbird_observability::observability::NetworkIO;

    let metrics = SystemMetrics {
        cpu_usage: 50.0,
        memory_usage: 0.75,
        disk_usage: 0.60,
        network_io: NetworkIO {
            bytes_in: 100_000,
            bytes_out: 50_000,
            packets_in: 1000,
            packets_out: 500,
        },
        timestamp: Utc::now(),
    };

    let cloned = metrics.clone();
    assert!((metrics.cpu_usage - cloned.cpu_usage).abs() < 0.001);
    assert!((metrics.memory_usage - cloned.memory_usage).abs() < 0.001);
}

#[test]
fn test_application_metrics_clone() {
    let metrics = ApplicationMetrics {
        active_services: 10,
        request_rate: 100.5,
        error_rate: 0.01,
        avg_response_time_ms: 25.3,
    };

    let cloned = metrics.clone();
    assert_eq!(metrics.active_services, cloned.active_services);
    assert!((metrics.request_rate - cloned.request_rate).abs() < 0.001);
}
