// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, reason = "test assertions")]

use super::*;
use chrono::Utc;
use songbird_types::SongbirdError;

#[tokio::test]
async fn test_metrics_collector_new() {
    let collector = MetricsCollector::new();
    assert_eq!(collector.get_collection_count(), 0);
}

#[tokio::test]
async fn test_metrics_collector_default() {
    let collector = MetricsCollector::default();
    assert_eq!(collector.get_collection_count(), 0);
}

#[tokio::test]
async fn test_collect_all_metrics() -> Result<()> {
    let collector = MetricsCollector::new();
    let result = collector.collect_all_metrics().await;

    assert!(result.is_ok());
    let metrics =
        result.map_err(|e| SongbirdError::configuration(format!("Operation failed: {e}")))?;
    assert!((metrics.system.cpu_usage - 0.0).abs() < f64::EPSILON);
    assert_eq!(metrics.songbird.active_services, 0);
    assert_eq!(collector.get_collection_count(), 1);
    Ok(())
}

#[tokio::test]
async fn test_get_current_snapshot_after_collection() -> Result<()> {
    let collector = MetricsCollector::new();

    // First collection
    let _ = collector.collect_all_metrics().await;

    // Get snapshot
    let result = collector.get_current_snapshot().await;
    assert!(result.is_ok());
    let snapshot =
        result.map_err(|e| SongbirdError::configuration(format!("Operation failed: {e}")))?;
    assert!((snapshot.system.cpu_usage - 0.0).abs() < f64::EPSILON);
    Ok(())
}

#[tokio::test]
async fn test_get_current_snapshot_without_prior_collection() {
    let collector = MetricsCollector::new();

    // Get snapshot without prior collection - should auto-collect
    let result = collector.get_current_snapshot().await;
    assert!(result.is_ok());
    assert_eq!(collector.get_collection_count(), 1);
}

#[tokio::test]
async fn test_get_current_metrics_alias() -> Result<()> {
    let collector = MetricsCollector::new();

    let result = collector.get_current_metrics().await;
    assert!(result.is_ok());
    let metrics =
        result.map_err(|e| SongbirdError::configuration(format!("Operation failed: {e}")))?;
    assert!((metrics.system.cpu_usage - 0.0).abs() < f64::EPSILON);
    Ok(())
}

#[tokio::test]
async fn test_collection_count_increments() {
    let collector = MetricsCollector::new();

    assert_eq!(collector.get_collection_count(), 0);
    let _ = collector.collect_all_metrics().await;
    assert_eq!(collector.get_collection_count(), 1);
    let _ = collector.collect_all_metrics().await;
    assert_eq!(collector.get_collection_count(), 2);
    let _ = collector.collect_all_metrics().await;
    assert_eq!(collector.get_collection_count(), 3);
}

#[tokio::test]
async fn test_export_prometheus_format() -> Result<()> {
    let collector = MetricsCollector::new();
    let _ = collector.collect_all_metrics().await;

    let result = collector.export_prometheus().await;
    assert!(result.is_ok());

    let output =
        result.map_err(|e| SongbirdError::configuration(format!("Operation failed: {e}")))?;
    assert!(output.contains("songbird_cpu_usage_percent"));
    assert!(output.contains("songbird_memory_usage_ratio"));
    assert!(output.contains("songbird_active_services"));
    assert!(output.contains("# HELP"));
    assert!(output.contains("# TYPE"));
    Ok(())
}

#[tokio::test]
async fn test_export_prometheus_without_prior_collection() {
    let collector = MetricsCollector::new();

    // Export should auto-collect if needed
    let result = collector.export_prometheus().await;
    assert!(result.is_ok());
    assert_eq!(collector.get_collection_count(), 1);
}

#[tokio::test]
async fn test_last_collection_time() {
    let collector = MetricsCollector::new();
    let time = collector.last_collection_time();
    assert!(time.is_some());
}

#[tokio::test]
async fn test_metrics_snapshot_contains_timestamp() -> Result<()> {
    let collector = MetricsCollector::new();
    let before = Utc::now();
    let metrics = collector
        .collect_all_metrics()
        .await
        .map_err(|e| SongbirdError::configuration(format!("Operation failed: {e}")))?;
    let after = Utc::now();
    assert!(metrics.timestamp >= before && metrics.timestamp <= after);
    Ok(())
}

#[tokio::test]
async fn test_metrics_snapshot_system_metrics() -> Result<()> {
    let collector = MetricsCollector::new();
    let metrics = collector
        .collect_all_metrics()
        .await
        .map_err(|e| SongbirdError::configuration(format!("Operation failed: {e}")))?;

    assert!((metrics.system.cpu_usage - 0.0).abs() < f64::EPSILON);
    assert!((metrics.system.memory_usage - 0.0).abs() < f64::EPSILON);
    assert!((metrics.system.disk_usage - 0.0).abs() < f64::EPSILON);
    assert_eq!(metrics.system.network_io.bytes_in, 0);
    assert_eq!(metrics.system.network_io.bytes_out, 0);
    Ok(())
}

#[tokio::test]
async fn test_metrics_snapshot_application_metrics() -> Result<()> {
    let collector = MetricsCollector::new();
    let metrics = collector
        .collect_all_metrics()
        .await
        .map_err(|e| SongbirdError::configuration(format!("Operation failed: {e}")))?;

    assert_eq!(metrics.songbird.active_services, 0);
    assert!((metrics.songbird.request_rate - 0.0).abs() < f64::EPSILON);
    assert!((metrics.songbird.error_rate - 0.0).abs() < f64::EPSILON);
    assert!((metrics.songbird.avg_response_time_ms - 0.0).abs() < f64::EPSILON);
    Ok(())
}

#[tokio::test]
async fn test_metrics_snapshot_clone() -> Result<()> {
    let collector = MetricsCollector::new();
    let metrics = collector
        .collect_all_metrics()
        .await
        .map_err(|e| SongbirdError::configuration(format!("Operation failed: {e}")))?;

    let cloned = metrics.clone();
    assert!((metrics.system.cpu_usage - cloned.system.cpu_usage).abs() < f64::EPSILON);
    assert_eq!(metrics.songbird.active_services, cloned.songbird.active_services);
    Ok(())
}

#[tokio::test]
async fn test_metrics_snapshot_serialization() -> Result<()> {
    let collector = MetricsCollector::new();
    let metrics = collector
        .collect_all_metrics()
        .await
        .map_err(|e| SongbirdError::configuration(format!("Operation failed: {e}")))?;

    let serialized = serde_json::to_string(&metrics).map_err(|e| SongbirdError::Serialization {
        format: Some("JSON".to_string()),
        message: format!("Serialization failed: {e}"),
        debug_info: None,
    })?;
    let deserialized: MetricsSnapshot =
        serde_json::from_str(&serialized).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Parsing failed: {e}"),
            debug_info: None,
        })?;

    assert!((metrics.system.cpu_usage - deserialized.system.cpu_usage).abs() < f64::EPSILON);
    assert_eq!(metrics.songbird.active_services, deserialized.songbird.active_services);
    Ok(())
}

#[tokio::test]
async fn test_application_metrics_serialization() -> Result<()> {
    let app_metrics = ApplicationMetrics {
        active_services: 5,
        request_rate: 100.5,
        error_rate: 0.01,
        avg_response_time_ms: 25.3,
    };

    let serialized =
        serde_json::to_string(&app_metrics).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Serialization failed: {e}"),
            debug_info: None,
        })?;
    let deserialized: ApplicationMetrics =
        serde_json::from_str(&serialized).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Parsing failed: {e}"),
            debug_info: None,
        })?;

    assert_eq!(app_metrics.active_services, deserialized.active_services);
    assert!((app_metrics.request_rate - deserialized.request_rate).abs() < f64::EPSILON);
    Ok(())
}

#[tokio::test]
async fn test_concurrent_collections() -> Result<()> {
    let collector = std::sync::Arc::new(MetricsCollector::new());

    let mut handles = vec![];
    for _ in 0..10 {
        let collector_clone = std::sync::Arc::clone(&collector);
        let handle = tokio::spawn(async move { collector_clone.collect_all_metrics().await });
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.await.map_err(|e| {
            SongbirdError::configuration(format!("Failed to join concurrent metrics task: {e}"))
        })?;
        assert!(result.is_ok());
    }

    assert_eq!(collector.get_collection_count(), 10);
    Ok(())
}

#[tokio::test]
async fn test_metrics_collector_debug() {
    let collector = MetricsCollector::new();
    let debug_str = format!("{collector:?}");
    assert!(debug_str.contains("MetricsCollector"));
}

#[tokio::test]
async fn test_metrics_snapshot_debug() -> Result<()> {
    let collector = MetricsCollector::new();
    let metrics = collector
        .collect_all_metrics()
        .await
        .map_err(|e| SongbirdError::configuration(format!("Operation failed: {e}")))?;
    let debug_str = format!("{metrics:?}");
    assert!(debug_str.contains("MetricsSnapshot"));
    Ok(())
}

#[tokio::test]
async fn test_prometheus_export_format_correctness() -> Result<()> {
    let collector = MetricsCollector::new();
    let output = collector
        .export_prometheus()
        .await
        .map_err(|e| SongbirdError::configuration(format!("Operation failed: {e}")))?;

    // Check format: metric name followed by value
    let mut found_metric_line = false;

    for line in output.lines() {
        if line.starts_with("songbird_cpu_usage_percent") && !line.starts_with('#') {
            found_metric_line = true;
            // Should have format: metric_name value
            assert_eq!(line.split_whitespace().count(), 2);
        }
    }

    assert!(found_metric_line, "Should have at least one metric line");
    Ok(())
}

#[tokio::test]
async fn test_get_current_snapshot_uses_cache_without_extra_collection() {
    let collector = MetricsCollector::new();
    collector.collect_all_metrics().await.unwrap();
    assert_eq!(collector.get_collection_count(), 1);

    let _ = collector.get_current_snapshot().await.unwrap();
    let _ = collector.get_current_snapshot().await.unwrap();
    assert_eq!(collector.get_collection_count(), 1);
}

#[tokio::test]
async fn test_get_current_metrics_matches_snapshot_when_cached() -> Result<()> {
    let collector = MetricsCollector::new();
    collector.collect_all_metrics().await.unwrap();

    let snap = collector.get_current_snapshot().await?;
    let alias = collector.get_current_metrics().await?;
    assert_eq!(snap.timestamp, alias.timestamp);
    assert_eq!(snap.collection_duration_ms, alias.collection_duration_ms);
    assert_eq!(snap.songbird.active_services, alias.songbird.active_services);
    Ok(())
}

#[tokio::test]
async fn test_export_prometheus_does_not_increment_when_snapshot_cached() {
    let collector = MetricsCollector::new();
    collector.collect_all_metrics().await.unwrap();
    assert_eq!(collector.get_collection_count(), 1);

    let _ = collector.export_prometheus().await.unwrap();
    let _ = collector.export_prometheus().await.unwrap();
    assert_eq!(collector.get_collection_count(), 1);
}

#[tokio::test]
async fn test_collect_all_metrics_sets_collection_duration_ms() -> Result<()> {
    let collector = MetricsCollector::new();
    let metrics = collector.collect_all_metrics().await?;
    assert_eq!(metrics.collection_duration_ms, 1);
    Ok(())
}

#[tokio::test]
async fn test_collect_all_metrics_network_io_packets() -> Result<()> {
    let collector = MetricsCollector::new();
    let metrics = collector.collect_all_metrics().await?;
    assert_eq!(metrics.system.network_io.packets_in, 0);
    assert_eq!(metrics.system.network_io.packets_out, 0);
    Ok(())
}

#[tokio::test]
async fn test_metrics_snapshot_top_level_timestamp_recent() -> Result<()> {
    let collector = MetricsCollector::new();
    let before = Utc::now();
    let metrics = collector.collect_all_metrics().await?;
    let after = Utc::now();
    assert!(metrics.timestamp >= before && metrics.timestamp <= after);
    Ok(())
}

#[tokio::test]
async fn test_application_metrics_debug() {
    let app = ApplicationMetrics {
        active_services: 3,
        request_rate: 42.0,
        error_rate: 0.05,
        avg_response_time_ms: 12.5,
    };
    let dbg = format!("{app:?}");
    assert!(dbg.contains("ApplicationMetrics"));
    assert!(dbg.contains("active_services"));
}

#[tokio::test]
async fn test_application_metrics_full_roundtrip_json() -> Result<()> {
    let app = ApplicationMetrics {
        active_services: 7,
        request_rate: 99.25,
        error_rate: 0.002,
        avg_response_time_ms: 8.75,
    };
    let json = serde_json::to_string(&app).map_err(|e| SongbirdError::Serialization {
        format: Some("JSON".to_string()),
        message: format!("Serialization failed: {e}"),
        debug_info: None,
    })?;
    let back: ApplicationMetrics =
        serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Parsing failed: {e}"),
            debug_info: None,
        })?;
    assert_eq!(app.active_services, back.active_services);
    assert!((app.request_rate - back.request_rate).abs() < f64::EPSILON);
    assert!((app.error_rate - back.error_rate).abs() < f64::EPSILON);
    assert!((app.avg_response_time_ms - back.avg_response_time_ms).abs() < f64::EPSILON);
    Ok(())
}

#[tokio::test]
async fn test_prometheus_export_lines_match_snapshot_values() -> Result<()> {
    let collector = MetricsCollector::new();
    collector.collect_all_metrics().await?;
    let out = collector.export_prometheus().await?;

    assert!(out.contains("songbird_cpu_usage_percent 0"));
    assert!(out.contains("songbird_memory_usage_ratio 0"));
    assert!(out.contains("songbird_active_services 0"));
    Ok(())
}

#[tokio::test]
async fn test_stored_snapshot_overwritten_on_each_collect() -> Result<()> {
    let collector = MetricsCollector::new();
    let first = collector.collect_all_metrics().await.unwrap();
    let second = collector.collect_all_metrics().await.unwrap();

    assert!(second.timestamp >= first.timestamp);
    let cached = collector.get_current_snapshot().await.unwrap();
    assert_eq!(cached.timestamp, second.timestamp);
    assert_eq!(collector.get_collection_count(), 2);
    Ok(())
}

#[tokio::test]
async fn test_last_collection_time_is_recent_utc() {
    let collector = MetricsCollector::new();
    let before = Utc::now();
    let t = collector.last_collection_time().unwrap();
    let after = Utc::now();
    assert!(t >= before && t <= after);
}

/// Both nested and top-level timestamps fall within the same wall-clock window as `collect_all_metrics`.
#[tokio::test]
async fn test_collect_timestamps_bounded_by_wall_clock() -> Result<()> {
    let collector = MetricsCollector::new();
    let before = Utc::now();
    let metrics = collector.collect_all_metrics().await?;
    let after = Utc::now();
    assert!(metrics.timestamp >= before && metrics.timestamp <= after);
    assert!(metrics.system.timestamp >= before && metrics.system.timestamp <= after);
    Ok(())
}

/// JSON roundtrip preserves all `network_io` counters on [`super::super::SystemMetrics`].
#[tokio::test]
async fn test_metrics_snapshot_json_preserves_network_io() -> Result<()> {
    let collector = MetricsCollector::new();
    let metrics = collector.collect_all_metrics().await?;
    let json = serde_json::to_string(&metrics)?;
    let back: MetricsSnapshot = serde_json::from_str(&json)?;
    assert_eq!(back.system.network_io.bytes_in, metrics.system.network_io.bytes_in);
    assert_eq!(back.system.network_io.bytes_out, metrics.system.network_io.bytes_out);
    assert_eq!(back.system.network_io.packets_in, metrics.system.network_io.packets_in);
    assert_eq!(back.system.network_io.packets_out, metrics.system.network_io.packets_out);
    Ok(())
}

#[tokio::test]
async fn test_prometheus_export_exactly_three_metric_value_lines() -> Result<()> {
    let collector = MetricsCollector::new();
    let out = collector.export_prometheus().await?;
    let value_lines = out.lines().filter(|l| !l.is_empty() && !l.starts_with('#')).count();
    assert_eq!(value_lines, 3);
    Ok(())
}

#[tokio::test]
async fn test_prometheus_export_document_order_cpu_memory_services() -> Result<()> {
    let collector = MetricsCollector::new();
    let out = collector.export_prometheus().await?;
    let cpu = out.find("songbird_cpu_usage_percent").unwrap();
    let mem = out.find("songbird_memory_usage_ratio").unwrap();
    let svc = out.find("songbird_active_services").unwrap();
    assert!(cpu < mem && mem < svc);
    Ok(())
}

#[tokio::test]
async fn test_application_metrics_clone() {
    let app = ApplicationMetrics {
        active_services: 11,
        request_rate: 1.25,
        error_rate: 0.5,
        avg_response_time_ms: 100.0,
    };
    let c = app.clone();
    assert_eq!(app.active_services, c.active_services);
    assert!((app.request_rate - c.request_rate).abs() < f64::EPSILON);
}

#[tokio::test]
async fn test_arc_shared_metrics_collector_sees_same_snapshot() -> Result<()> {
    use std::sync::Arc;

    let c = Arc::new(MetricsCollector::new());
    let c2 = Arc::clone(&c);
    c.collect_all_metrics().await.unwrap();
    assert_eq!(c2.get_collection_count(), 1);
    let s = c2.get_current_snapshot().await.unwrap();
    assert_eq!(s.collection_duration_ms, 1);
    assert_eq!(s.songbird.active_services, 0);
    Ok(())
}

#[tokio::test]
async fn test_metrics_snapshot_json_has_expected_top_level_keys() -> Result<()> {
    let collector = MetricsCollector::new();
    let metrics = collector.collect_all_metrics().await?;
    let v: serde_json::Value = serde_json::to_value(&metrics)?;
    assert!(v.get("system").is_some());
    assert!(v.get("songbird").is_some());
    assert!(v.get("collection_duration_ms").is_some());
    assert!(v.get("timestamp").is_some());
    Ok(())
}

#[test]
fn test_metrics_collector_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<MetricsCollector>();
    assert_send_sync::<MetricsSnapshot>();
    assert_send_sync::<ApplicationMetrics>();
}

#[tokio::test]
async fn test_direct_collect_matches_get_snapshot_after_auto_collect() -> Result<()> {
    let collector = MetricsCollector::new();
    let a = collector.collect_all_metrics().await?;
    let b = collector.get_current_snapshot().await?;
    assert_eq!(a.timestamp, b.timestamp);
    assert_eq!(a.collection_duration_ms, b.collection_duration_ms);
    assert_eq!(a.songbird.active_services, b.songbird.active_services);
    Ok(())
}

#[tokio::test]
async fn test_prometheus_values_parse_as_f64() -> Result<()> {
    let collector = MetricsCollector::new();
    collector.collect_all_metrics().await?;
    let out = collector.export_prometheus().await?;
    for line in out.lines() {
        if line.starts_with("songbird_cpu_usage_percent ") {
            let v: f64 = line.split_whitespace().nth(1).unwrap().parse().unwrap();
            assert!((v - 0.0).abs() < f64::EPSILON);
        }
        if line.starts_with("songbird_memory_usage_ratio ") {
            let v: f64 = line.split_whitespace().nth(1).unwrap().parse().unwrap();
            assert!((v - 0.0).abs() < f64::EPSILON);
        }
        if line.starts_with("songbird_active_services ") {
            let v: f64 = line.split_whitespace().nth(1).unwrap().parse().unwrap();
            assert!((v - 0.0).abs() < f64::EPSILON);
        }
    }
    Ok(())
}

#[tokio::test]
async fn test_prometheus_export_three_help_and_three_type_lines() -> Result<()> {
    let collector = MetricsCollector::new();
    let out = collector.export_prometheus().await?;
    let help = out.lines().filter(|l| l.starts_with("# HELP ")).count();
    let type_lines = out.lines().filter(|l| l.starts_with("# TYPE ")).count();
    assert_eq!(help, 3);
    assert_eq!(type_lines, 3);
    Ok(())
}

#[tokio::test]
async fn test_metrics_snapshot_json_roundtrip_preserves_all_application_fields() -> Result<()> {
    let collector = MetricsCollector::new();
    let metrics = collector.collect_all_metrics().await?;
    let json = serde_json::to_string(&metrics)?;
    let back: MetricsSnapshot = serde_json::from_str(&json)?;
    assert!((metrics.songbird.request_rate - back.songbird.request_rate).abs() < f64::EPSILON);
    assert!((metrics.songbird.error_rate - back.songbird.error_rate).abs() < f64::EPSILON);
    assert!(
        (metrics.songbird.avg_response_time_ms - back.songbird.avg_response_time_ms).abs()
            < f64::EPSILON
    );
    Ok(())
}

#[tokio::test]
async fn test_default_collector_matches_new_for_initial_state() {
    let a = MetricsCollector::new();
    let b = MetricsCollector::default();
    assert_eq!(a.get_collection_count(), b.get_collection_count());
}
