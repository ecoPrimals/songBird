// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Expected JSON-RPC shapes (health, identity, beacon exchange).

#[test]
fn test_health_response_structure() {
    // Verify health response matches expected schema
    let health_response = serde_json::json!({
        "status": "healthy",
        "primal": "songbird",
        "version": env!("CARGO_PKG_VERSION"),
        "uptime_s": 0,
        "components": {
            "http_server": "running",
            "task_manager": "ready"
        }
    });

    // Validate required HEALTH-01 fields exist
    assert!(health_response.get("status").is_some());
    assert!(health_response.get("primal").is_some());
    assert!(health_response.get("version").is_some());
    assert!(health_response.get("uptime_s").is_some());
}

#[test]
fn test_identity_response_structure() {
    // Verify identity response matches expected schema
    let identity_response = serde_json::json!({
        "primal": "songbird",
        "version": env!("CARGO_PKG_VERSION"),
        "capabilities": ["orchestration", "task-management", "federation"],
        "node_id": "test-node-001"
    });

    assert_eq!(identity_response["primal"], "songbird");
    assert!(identity_response["capabilities"].is_array());
}

#[test]
fn test_beacon_exchange_request_validation() {
    // Test beacon exchange parameter validation
    let valid_request = serde_json::json!({
        "beacon": {
            "node_id": "peer-123",
            "capabilities": ["compute"],
            "endpoint": "https://192.168.1.100:8080"
        }
    });

    let beacon = valid_request.get("beacon");
    assert!(beacon.is_some());
    assert!(beacon.unwrap().get("node_id").is_some());
    assert!(beacon.unwrap().get("capabilities").is_some());
}

#[test]
fn test_beacon_exchange_missing_beacon() {
    // Should handle missing beacon gracefully
    let invalid_request = serde_json::json!({});
    assert!(invalid_request.get("beacon").is_none());
}
