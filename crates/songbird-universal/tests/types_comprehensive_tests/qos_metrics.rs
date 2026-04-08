// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal::types::*;

// ============================================================================
// QOS METRICS TESTS
// ============================================================================

#[test]
fn test_qos_metrics_default() {
    let qos = QosMetrics::default();
    assert_eq!(qos.latency_ms, None);
    assert_eq!(qos.throughput_ops_sec, None);
    assert_eq!(qos.availability, None);
    assert_eq!(qos.reliability, None);
}

#[test]
fn test_qos_metrics_with_values() {
    let qos = QosMetrics {
        latency_ms: Some(50.0),
        throughput_ops_sec: Some(1000.0),
        availability: Some(0.99),
        reliability: Some(0.999),
    };

    assert_eq!(qos.latency_ms, Some(50.0));
    assert_eq!(qos.throughput_ops_sec, Some(1000.0));
    assert_eq!(qos.availability, Some(0.99));
    assert_eq!(qos.reliability, Some(0.999));
}

#[test]
fn test_qos_metrics_clone() -> SongbirdResult<()> {
    let qos1 = QosMetrics {
        latency_ms: Some(100.0),
        throughput_ops_sec: Some(500.0),
        availability: Some(0.95),
        reliability: Some(0.98),
    };
    let qos2 = qos1.clone();

    assert_eq!(qos1.latency_ms, qos2.latency_ms);
    Ok(())
}

#[test]
fn test_qos_metrics_serialization() -> SongbirdResult<()> {
    let qos = QosMetrics {
        latency_ms: Some(75.0),
        throughput_ops_sec: Some(800.0),
        availability: Some(0.97),
        reliability: Some(0.99),
    };

    let json = serde_json::to_string(&qos)
        .map_err(|_e| SongbirdError::configuration("Failed to serialize"))?;
    let deserialized: QosMetrics = serde_json::from_str(&json)
        .map_err(|_e| SongbirdError::configuration("Failed to deserialize"))?;

    assert_eq!(deserialized.latency_ms, qos.latency_ms);
    Ok(())
}
