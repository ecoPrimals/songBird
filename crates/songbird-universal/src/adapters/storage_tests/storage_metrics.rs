// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::super::*;
use songbird_types::{SongbirdError, SongbirdResult};

#[test]
fn test_storage_metrics_calculations() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000, // 1TB
        used_bytes: 250_000_000_000,             // 250GB
        available_bytes: 750_000_000_000,        // 750GB
        object_count: 1_500,
        avg_read_latency_ms: 15.0,
        avg_write_latency_ms: 25.0,
        timestamp: chrono::Utc::now(),
    };

    assert!((metrics.usage_percent() - 25.0).abs() < 0.1);
    assert!(!metrics.is_nearly_full());
    assert!(!metrics.is_high_latency());
    assert_eq!(metrics.health_status(), StorageHealth::Healthy);
}

#[test]
fn test_storage_nearly_full() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 960_000_000_000, // 96%
        available_bytes: 40_000_000_000,
        object_count: 50_000,
        avg_read_latency_ms: 20.0,
        avg_write_latency_ms: 600.0, // High write latency
        timestamp: chrono::Utc::now(),
    };

    assert!(metrics.is_nearly_full());
    assert_eq!(metrics.health_status(), StorageHealth::Critical);
}

#[test]
fn test_storage_warning() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 870_000_000_000, // 87%
        available_bytes: 130_000_000_000,
        object_count: 25_000,
        avg_read_latency_ms: 120.0, // High read latency
        avg_write_latency_ms: 180.0,
        timestamp: chrono::Utc::now(),
    };

    assert!(!metrics.is_nearly_full());
    assert!(metrics.is_high_latency());
    assert_eq!(metrics.health_status(), StorageHealth::Warning);
}

#[test]
fn test_storage_health_equality() {
    assert_eq!(StorageHealth::Healthy, StorageHealth::Healthy);
    assert_eq!(StorageHealth::Warning, StorageHealth::Warning);
    assert_eq!(StorageHealth::Critical, StorageHealth::Critical);
    assert_ne!(StorageHealth::Healthy, StorageHealth::Warning);
    assert_ne!(StorageHealth::Warning, StorageHealth::Critical);
}

#[test]
fn test_storage_usage_zero_capacity() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 0,
        used_bytes: 0,
        available_bytes: 0,
        object_count: 0,
        avg_read_latency_ms: 10.0,
        avg_write_latency_ms: 20.0,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(metrics.usage_percent(), 0.0);
}

#[test]
fn test_nearly_full_boundary() {
    // Just below threshold (90%)
    let metrics_below = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 900_000_000_000,
        available_bytes: 100_000_000_000,
        object_count: 10_000,
        avg_read_latency_ms: 15.0,
        avg_write_latency_ms: 25.0,
        timestamp: chrono::Utc::now(),
    };
    assert!(!metrics_below.is_nearly_full());

    // Just above threshold
    let metrics_above = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 901_000_000_000,
        available_bytes: 99_000_000_000,
        object_count: 10_000,
        avg_read_latency_ms: 15.0,
        avg_write_latency_ms: 25.0,
        timestamp: chrono::Utc::now(),
    };
    assert!(metrics_above.is_nearly_full());
}

#[test]
fn test_high_read_latency_boundary() {
    // Just below threshold
    let metrics_below = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 250_000_000_000,
        available_bytes: 750_000_000_000,
        object_count: 1_500,
        avg_read_latency_ms: 100.0,
        avg_write_latency_ms: 50.0,
        timestamp: chrono::Utc::now(),
    };
    assert!(!metrics_below.is_high_latency());

    // Just above threshold
    let metrics_above = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 250_000_000_000,
        available_bytes: 750_000_000_000,
        object_count: 1_500,
        avg_read_latency_ms: 100.1,
        avg_write_latency_ms: 50.0,
        timestamp: chrono::Utc::now(),
    };
    assert!(metrics_above.is_high_latency());
}

#[test]
fn test_high_write_latency_boundary() {
    // Just below threshold
    let metrics_below = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 250_000_000_000,
        available_bytes: 750_000_000_000,
        object_count: 1_500,
        avg_read_latency_ms: 50.0,
        avg_write_latency_ms: 200.0,
        timestamp: chrono::Utc::now(),
    };
    assert!(!metrics_below.is_high_latency());

    // Just above threshold
    let metrics_above = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 250_000_000_000,
        available_bytes: 750_000_000_000,
        object_count: 1_500,
        avg_read_latency_ms: 50.0,
        avg_write_latency_ms: 200.1,
        timestamp: chrono::Utc::now(),
    };
    assert!(metrics_above.is_high_latency());
}

#[test]
fn test_health_status_critical_high_usage() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 960_000_000_000, // 96%
        available_bytes: 40_000_000_000,
        object_count: 50_000,
        avg_read_latency_ms: 20.0,
        avg_write_latency_ms: 50.0,
        timestamp: chrono::Utc::now(),
    };
    assert_eq!(metrics.health_status(), StorageHealth::Critical);
}

#[test]
fn test_health_status_critical_high_write_latency() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 500_000_000_000,
        available_bytes: 500_000_000_000,
        object_count: 10_000,
        avg_read_latency_ms: 20.0,
        avg_write_latency_ms: 501.0,
        timestamp: chrono::Utc::now(),
    };
    assert_eq!(metrics.health_status(), StorageHealth::Critical);
}

#[test]
fn test_health_status_warning_high_usage() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 860_000_000_000, // 86%
        available_bytes: 140_000_000_000,
        object_count: 25_000,
        avg_read_latency_ms: 20.0,
        avg_write_latency_ms: 50.0,
        timestamp: chrono::Utc::now(),
    };
    assert_eq!(metrics.health_status(), StorageHealth::Warning);
}

#[test]
fn test_health_status_warning_high_latency() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 500_000_000_000,
        available_bytes: 500_000_000_000,
        object_count: 10_000,
        avg_read_latency_ms: 120.0,
        avg_write_latency_ms: 50.0,
        timestamp: chrono::Utc::now(),
    };
    assert_eq!(metrics.health_status(), StorageHealth::Warning);
}

#[test]
fn test_health_status_boundary_95_usage() {
    // Exactly at critical threshold
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 950_000_000_000, // 95%
        available_bytes: 50_000_000_000,
        object_count: 40_000,
        avg_read_latency_ms: 20.0,
        avg_write_latency_ms: 50.0,
        timestamp: chrono::Utc::now(),
    };
    assert_eq!(metrics.health_status(), StorageHealth::Warning);
}

#[test]
fn test_health_status_boundary_500ms_write() {
    // Exactly at critical threshold
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 500_000_000_000,
        available_bytes: 500_000_000_000,
        object_count: 10_000,
        avg_read_latency_ms: 20.0,
        avg_write_latency_ms: 500.0,
        timestamp: chrono::Utc::now(),
    };
    assert_eq!(metrics.health_status(), StorageHealth::Warning);
}

#[test]
fn test_storage_metrics_serialization() -> SongbirdResult<()> {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 250_000_000_000,
        available_bytes: 750_000_000_000,
        object_count: 1_500,
        avg_read_latency_ms: 15.0,
        avg_write_latency_ms: 25.0,
        timestamp: chrono::Utc::now(),
    };

    let json = serde_json::to_string(&metrics).map_err(|e| {
        SongbirdError::configuration(format!("Serialization should succeed: {}", e))
    })?;
    assert!(json.contains("total_capacity_bytes"));
    assert!(json.contains("object_count"));
    Ok(())
}

#[test]
fn test_storage_health_serialization() -> SongbirdResult<()> {
    assert_eq!(
        serde_json::to_string(&StorageHealth::Healthy).map_err(|e| {
            SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: format!("Serialization failed: {}", e),
                debug_info: None,
            }
        })?,
        "\"Healthy\""
    );
    assert_eq!(
        serde_json::to_string(&StorageHealth::Warning).map_err(|e| {
            SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: format!("Serialization failed: {}", e),
                debug_info: None,
            }
        })?,
        "\"Warning\""
    );
    assert_eq!(
        serde_json::to_string(&StorageHealth::Critical).map_err(|e| {
            SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: format!("Serialization failed: {}", e),
                debug_info: None,
            }
        })?,
        "\"Critical\""
    );
    Ok(())
}

#[test]
fn test_storage_metrics_perfect_conditions() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 100_000_000_000, // 10%
        available_bytes: 900_000_000_000,
        object_count: 500,
        avg_read_latency_ms: 5.0,
        avg_write_latency_ms: 10.0,
        timestamp: chrono::Utc::now(),
    };

    assert!(!metrics.is_nearly_full());
    assert!(!metrics.is_high_latency());
    assert_eq!(metrics.health_status(), StorageHealth::Healthy);
}

#[test]
fn test_storage_metrics_all_zero() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 0,
        used_bytes: 0,
        available_bytes: 0,
        object_count: 0,
        avg_read_latency_ms: 0.0,
        avg_write_latency_ms: 0.0,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(metrics.usage_percent(), 0.0);
    assert!(!metrics.is_nearly_full());
    assert!(!metrics.is_high_latency());
    assert_eq!(metrics.health_status(), StorageHealth::Healthy);
}

#[test]
fn test_storage_100_percent_full() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 1_000_000_000_000, // 100%
        available_bytes: 0,
        object_count: 100_000,
        avg_read_latency_ms: 50.0,
        avg_write_latency_ms: 100.0,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(metrics.usage_percent(), 100.0);
    assert!(metrics.is_nearly_full());
    assert_eq!(metrics.health_status(), StorageHealth::Critical);
}
