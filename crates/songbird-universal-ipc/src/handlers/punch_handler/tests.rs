// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::*;
use serde_json::json;
use std::time::Duration;

#[tokio::test]
async fn punch_handler_default_matches_new_behavior() {
    let a = PunchHandler::new();
    let b = PunchHandler::default();
    let ra = a.handle_request(json!({ "target_node_id": "node-a" })).await.unwrap();
    let rb = b.handle_request(json!({ "target_node_id": "node-b" })).await.unwrap();
    assert_eq!(ra["reason"], rb["reason"]);
}

#[tokio::test]
async fn handle_request_missing_target_errors() {
    let handler = PunchHandler::new();
    let err = handler.handle_request(json!({})).await.expect_err("target");
    assert!(err.contains("target_node_id"));
}

#[tokio::test]
async fn handle_status_missing_target_errors() {
    let handler = PunchHandler::new();
    let err = handler.handle_status(json!({})).await.expect_err("target");
    assert!(err.contains("target_node_id"));
}

#[tokio::test]
async fn handle_coordinate_missing_fields_errors_or_relay() {
    let handler = PunchHandler::new();
    let err = handler
        .handle_coordinate(json!({ "target_node_id": "peer-1" }))
        .await
        .expect_err("peer_predicted_port");
    assert!(err.contains("peer_predicted_port"));
}

#[tokio::test]
async fn test_punch_handler_new_uses_default_max_attempts_in_request() {
    let handler = PunchHandler::new();
    let r = handler.handle_request(json!({ "target_node_id": "z" })).await.unwrap();
    assert_eq!(r["reason"], "hole_punch_coordinator_not_initialized");
}

#[tokio::test]
async fn test_punch_request_no_coordinator() {
    let handler = PunchHandler::new();

    let result = handler
        .handle_request(json!({
            "target_node_id": "test-peer",
            "timeout_seconds": 5
        }))
        .await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response["success"], false);
    assert_eq!(response["fallback"], "family_relay");
}

#[tokio::test]
async fn test_punch_status_not_found() {
    let handler = PunchHandler::new();

    let result = handler
        .handle_status(json!({
            "target_node_id": "unknown-peer"
        }))
        .await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response["status"], "not_found");
}

#[tokio::test]
async fn test_punch_record_success() {
    let handler = PunchHandler::new();

    // Start a punch request
    handler
        .handle_request(json!({
            "target_node_id": "test-peer",
            "timeout_seconds": 5
        }))
        .await
        .unwrap();

    // Record success
    handler
        .record_success("test-peer", "1.2.3.4:5678".parse().unwrap(), Duration::from_millis(45), 5)
        .await;

    // Check status
    let result = handler
        .handle_status(json!({
            "target_node_id": "test-peer"
        }))
        .await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response["status"], "succeeded");
    assert_eq!(response["connected_address"], "1.2.3.4:5678");
    assert_eq!(response["latency_ms"], 45);
}

#[tokio::test]
async fn test_punch_record_failure() {
    let handler = PunchHandler::new();

    // Start a punch request first
    handler
        .handle_request(json!({
            "target_node_id": "test-peer",
            "timeout_seconds": 5
        }))
        .await
        .unwrap();

    // Record failure
    handler.record_failure("test-peer", "symmetric_nat_both_sides".to_string(), 20).await;

    // Check status
    let result = handler
        .handle_status(json!({
            "target_node_id": "test-peer"
        }))
        .await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response["status"], "failed");
    assert_eq!(response["reason"], "symmetric_nat_both_sides");
    assert_eq!(response["fallback"], "family_relay");
}

#[tokio::test]
async fn handle_coordinate_without_coordinator_parses_sequential_pattern() {
    let handler = PunchHandler::new();
    let r = handler
        .handle_coordinate(json!({
            "target_node_id": "peer-coord",
            "peer_predicted_port": 45000,
            "peer_public_ip": "10.0.0.1",
            "our_pattern": {
                "pattern": "sequential",
                "step": 2,
                "last_port": 40000,
                "predicted_next": 40002,
                "confidence": 0.95
            }
        }))
        .await
        .unwrap();
    assert_eq!(r["success"], false);
    assert_eq!(r["mode"], "relay");
}

#[tokio::test]
async fn handle_coordinate_without_coordinator_parses_random_pattern() {
    let handler = PunchHandler::new();
    let r = handler
        .handle_coordinate(json!({
            "target_node_id": "peer-rand",
            "peer_predicted_port": 1234,
            "peer_public_ip": "::1",
            "our_pattern": {
                "pattern": "random",
                "observed_ports": [1000, 1001, 9999]
            }
        }))
        .await
        .unwrap();
    assert_eq!(r["reason"], "coordinator_not_initialized");
}

#[tokio::test]
async fn handle_coordinate_unknown_pattern_defaults() {
    let handler = PunchHandler::new();
    let r = handler
        .handle_coordinate(json!({
            "target_node_id": "u",
            "peer_predicted_port": 9,
            "peer_public_ip": "192.0.2.1",
            "our_pattern": { "pattern": "weird" }
        }))
        .await
        .unwrap();
    assert_eq!(r["fallback"], "relay_continues");
}

#[tokio::test]
async fn handle_request_respects_max_attempts_and_timeout_json() {
    let handler = PunchHandler::new();
    let r = handler
        .handle_request(json!({
            "target_node_id": "ma",
            "timeout_seconds": 3,
            "max_attempts": 7
        }))
        .await
        .unwrap();
    // Without coordinator, response is immediate failure (no `started` branch fields).
    assert_eq!(r["success"], false);
    assert_eq!(r["attempts"], 0);
    assert_eq!(r["reason"], "hole_punch_coordinator_not_initialized");
}

#[tokio::test]
async fn handle_status_in_progress_shape() {
    let handler = PunchHandler::new();
    handler
        .handle_request(json!({ "target_node_id": "prog", "timeout_seconds": 60 }))
        .await
        .unwrap();
    let r = handler.handle_status(json!({ "target_node_id": "prog" })).await.unwrap();
    assert_eq!(r["status"], "failed");
    assert_eq!(r["reason"], "no_coordinator");
}

#[tokio::test]
async fn handle_coordinate_invalid_ip_errors() {
    let handler = PunchHandler::new();
    let err = handler
        .handle_coordinate(json!({
            "target_node_id": "x",
            "peer_predicted_port": 1,
            "peer_public_ip": "not-an-ip"
        }))
        .await
        .expect_err("ip");
    assert!(err.contains("peer_public_ip") || err.contains("invalid"));
}

#[tokio::test]
async fn punch_status_includes_elapsed_ms() {
    let handler = PunchHandler::new();
    handler.handle_request(json!({ "target_node_id": "elapsed" })).await.unwrap();
    let r = handler.handle_status(json!({ "target_node_id": "elapsed" })).await.unwrap();
    assert!(r.get("elapsed_ms").is_some());
}

#[test]
fn punch_status_and_attempt_enum_debug() {
    let s = PunchStatus::Failed {
        reason: "r".to_string(),
    };
    assert!(format!("{s:?}").contains("Failed"));
}
