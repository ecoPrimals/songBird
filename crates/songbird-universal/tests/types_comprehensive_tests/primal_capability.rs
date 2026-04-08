// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal::types::*;
use std::collections::HashMap;

// ============================================================================
// PRIMAL CAPABILITY TESTS
// ============================================================================

#[test]
fn test_primal_capability_creation() {
    let mut params = HashMap::new();
    params.insert("key".to_string(), serde_json::json!("value"));

    let cap = PrimalCapability {
        capability_type: "compute".to_string(),
        version: "1.0.0".to_string(),
        parameters: params,
        qos_metrics: QosMetrics::default(),
    };

    assert_eq!(cap.capability_type, "compute");
    assert_eq!(cap.version, "1.0.0");
    assert_eq!(cap.parameters.len(), 1);
}

#[test]
fn test_primal_capability_clone() -> SongbirdResult<()> {
    let cap1 = PrimalCapability {
        capability_type: "network".to_string(),
        version: "2.0.0".to_string(),
        parameters: HashMap::new(),
        qos_metrics: QosMetrics::default(),
    };

    let cap2 = cap1.clone();
    assert_eq!(cap1.capability_type, cap2.capability_type);
    Ok(())
}

#[test]
fn test_primal_capability_serialization() -> SongbirdResult<()> {
    let cap = PrimalCapability {
        capability_type: "storage".to_string(),
        version: "1.0.0".to_string(),
        parameters: HashMap::new(),
        qos_metrics: QosMetrics::default(),
    };

    let json = serde_json::to_string(&cap)
        .map_err(|_e| SongbirdError::configuration("Failed to serialize"))?;
    let deserialized: PrimalCapability = serde_json::from_str(&json)
        .map_err(|_e| SongbirdError::configuration("Failed to deserialize"))?;

    assert_eq!(deserialized.capability_type, cap.capability_type);
    Ok(())
}
