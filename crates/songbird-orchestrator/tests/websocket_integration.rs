// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::ignore_without_reason,
    clippy::unreadable_literal,
    clippy::no_effect_underscore_binding,
    clippy::float_cmp,
    clippy::default_trait_access,
    clippy::needless_collect,
    clippy::unused_async,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::items_after_statements,
    clippy::unnecessary_wraps,
    clippy::used_underscore_binding,
    clippy::struct_excessive_bools,
    clippy::similar_names,
    clippy::significant_drop_tightening,
    clippy::struct_field_names,
    clippy::match_same_arms,
    clippy::future_not_send,
    reason = "integration tests: strict clippy matches crate [lints] policy"
)]
// Allow unwrap/expect in tests - idiomatic for test code
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions and harness ergonomics"
)]

/// WebSocket Integration Tests
///
/// Comprehensive tests for WebSocket server and event broadcasting system.
///
/// Version: 0.2.1
/// Last Updated: November 11, 2025 - Phase 4
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore] // Requires running server
async fn test_websocket_connection() {
    // Connect to WebSocket server
    let url = "ws://localhost:8080/api/ws/ws";

    let result = tokio::time::timeout(Duration::from_secs(5), connect_async(url)).await;

    assert!(result.is_ok(), "Connection should succeed");

    if let Ok(Ok((ws_stream, _))) = result {
        let (mut write, mut read) = ws_stream.split();

        // Should receive welcome message
        if let Some(Ok(Message::Text(msg))) = read.next().await {
            let data: serde_json::Value =
                serde_json::from_str(&msg).expect("should parse valid input");
            assert_eq!(data["type"], "ack");
        }

        // Clean up
        let _ = write.close().await;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore] // Requires running server
async fn test_ping_pong() {
    let url = "ws://localhost:8080/api/ws/ws";
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    let (mut write, mut read) = ws_stream.split();

    // Skip welcome message
    let _ = read.next().await;

    // Send ping
    let ping_msg = json!({
        "type": "ping",
        "data": "test"
    });

    write.send(Message::Text(ping_msg.to_string())).await.expect("test precondition");

    // Wait for pong
    if let Some(Ok(Message::Text(msg))) = read.next().await {
        let data: serde_json::Value = serde_json::from_str(&msg).expect("should parse valid input");
        assert_eq!(data["type"], "pong");
        assert_eq!(data["data"], "test");
    } else {
        panic!("Did not receive pong");
    }

    let _ = write.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore] // Requires running server
async fn test_query_status() {
    let url = "ws://localhost:8080/api/ws/ws";
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    let (mut write, mut read) = ws_stream.split();

    // Skip welcome message
    let _ = read.next().await;

    // Query status
    let query_msg = json!({
        "type": "query_status"
    });

    write.send(Message::Text(query_msg.to_string())).await.expect("test precondition");

    // Wait for response
    if let Some(Ok(Message::Text(msg))) = read.next().await {
        let data: serde_json::Value = serde_json::from_str(&msg).expect("should parse valid input");
        assert_eq!(data["type"], "federation_status");
        assert!(data["total_services"].is_number());
        assert!(data["total_peers"].is_number());
        assert!(data["uptime_seconds"].is_number());
    } else {
        panic!("Did not receive status response");
    }

    let _ = write.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore] // Requires running server
async fn test_query_services() {
    let url = "ws://localhost:8080/api/ws/ws";
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    let (mut write, mut read) = ws_stream.split();

    // Skip welcome message
    let _ = read.next().await;

    // Query services
    let query_msg = json!({
        "type": "query_services",
        "capabilities": ["ml"]
    });

    write.send(Message::Text(query_msg.to_string())).await.expect("test precondition");

    // Wait for response
    if let Some(Ok(Message::Text(msg))) = read.next().await {
        let data: serde_json::Value = serde_json::from_str(&msg).expect("should parse valid input");
        assert_eq!(data["type"], "service_list");
        assert!(data["services"].is_array());
    } else {
        panic!("Did not receive service list response");
    }

    let _ = write.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore] // Requires running server
async fn test_subscription() {
    let url = "ws://localhost:8080/api/ws/ws";
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    let (mut write, mut read) = ws_stream.split();

    // Skip welcome message
    let _ = read.next().await;

    // Subscribe to events
    let subscribe_msg = json!({
        "type": "subscribe",
        "events": ["service_update", "health_update"]
    });

    write.send(Message::Text(subscribe_msg.to_string())).await.expect("test precondition");

    // Wait for ack
    if let Some(Ok(Message::Text(msg))) = read.next().await {
        let data: serde_json::Value = serde_json::from_str(&msg).expect("should parse valid input");
        assert_eq!(data["type"], "ack");
    } else {
        panic!("Did not receive subscription ack");
    }

    let _ = write.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore] // Requires running server
async fn test_unsubscribe() {
    let url = "ws://localhost:8080/api/ws/ws";
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    let (mut write, mut read) = ws_stream.split();

    // Skip welcome message
    let _ = read.next().await;

    // Subscribe first
    let subscribe_msg = json!({
        "type": "subscribe",
        "events": ["service_update"]
    });
    write.send(Message::Text(subscribe_msg.to_string())).await.expect("test precondition");
    let _ = read.next().await; // Skip ack

    // Unsubscribe
    let unsubscribe_msg = json!({
        "type": "unsubscribe",
        "events": ["service_update"]
    });

    write.send(Message::Text(unsubscribe_msg.to_string())).await.expect("test precondition");

    // Wait for ack
    if let Some(Ok(Message::Text(msg))) = read.next().await {
        let data: serde_json::Value = serde_json::from_str(&msg).expect("should parse valid input");
        assert_eq!(data["type"], "ack");
    } else {
        panic!("Did not receive unsubscription ack");
    }

    let _ = write.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore] // Requires running server
async fn test_invalid_message() {
    let url = "ws://localhost:8080/api/ws/ws";
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    let (mut write, mut read) = ws_stream.split();

    // Skip welcome message
    let _ = read.next().await;

    // Send invalid JSON
    write.send(Message::Text("invalid json".to_string())).await.expect("test precondition");

    // Should receive error response
    if let Some(Ok(Message::Text(msg))) = read.next().await {
        let data: serde_json::Value = serde_json::from_str(&msg).expect("should parse valid input");
        assert_eq!(data["type"], "error");
        assert!(data["message"].as_str().expect("test precondition").contains("Invalid"));
    } else {
        panic!("Did not receive error response");
    }

    let _ = write.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore] // Requires running server
async fn test_binary_message_rejected() {
    let url = "ws://localhost:8080/api/ws/ws";
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    let (mut write, mut read) = ws_stream.split();

    // Skip welcome message
    let _ = read.next().await;

    // Send binary message
    write.send(Message::Binary(vec![1, 2, 3, 4])).await.expect("test precondition");

    // Should receive error response
    if let Some(Ok(Message::Text(msg))) = read.next().await {
        let data: serde_json::Value = serde_json::from_str(&msg).expect("should parse valid input");
        assert_eq!(data["type"], "error");
        assert!(data["message"].as_str().expect("test precondition").contains("Binary"));
    } else {
        panic!("Did not receive error response");
    }

    let _ = write.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore] // Requires running server
async fn test_multiple_clients() {
    let url = "ws://localhost:8080/api/ws/ws";

    // Connect multiple clients
    let (ws1, _) = connect_async(url).await.expect("Client 1 failed to connect");
    let (ws2, _) = connect_async(url).await.expect("Client 2 failed to connect");

    let (mut write1, mut read1) = ws1.split();
    let (mut write2, mut read2) = ws2.split();

    // Skip welcome messages
    let _ = read1.next().await;
    let _ = read2.next().await;

    // Both clients query status
    let query = json!({"type": "query_status"});

    write1.send(Message::Text(query.to_string())).await.expect("test precondition");
    write2.send(Message::Text(query.to_string())).await.expect("test precondition");

    // Both should receive responses
    if let Some(Ok(Message::Text(msg1))) = read1.next().await {
        let data: serde_json::Value =
            serde_json::from_str(&msg1).expect("should parse valid input");
        assert_eq!(data["type"], "federation_status");
    } else {
        panic!("Client 1 did not receive response");
    }

    if let Some(Ok(Message::Text(msg2))) = read2.next().await {
        let data: serde_json::Value =
            serde_json::from_str(&msg2).expect("should parse valid input");
        assert_eq!(data["type"], "federation_status");
    } else {
        panic!("Client 2 did not receive response");
    }

    let _ = write1.close().await;
    let _ = write2.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore] // Requires running server
async fn test_connection_close() {
    let url = "ws://localhost:8080/api/ws/ws";
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    let (mut write, _read) = ws_stream.split();

    // Close connection gracefully
    let result = write.close().await;
    assert!(result.is_ok(), "Close should succeed");
}

// Event broadcasting tests (unit tests in events.rs)
// These tests would require triggering actual events from the server,
// which is better tested in the events.rs module directly.

#[cfg(test)]
mod event_tests {
    use songbird_orchestrator::server::events::{Event, EventBroadcaster, EventType};

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_event_broadcaster_creation() {
        let broadcaster = EventBroadcaster::new();
        assert_eq!(broadcaster.subscriber_count().await, 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_event_broadcaster_subscribe() {
        let broadcaster = EventBroadcaster::new();

        let _rx = broadcaster
            .subscribe("test-client".to_string(), vec!["service_update".to_string()])
            .await;

        assert_eq!(broadcaster.subscriber_count().await, 1);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_event_broadcaster_broadcast() {
        let broadcaster = EventBroadcaster::new();

        let mut rx = broadcaster
            .subscribe("test-client".to_string(), vec!["service_update".to_string()])
            .await;

        let event = Event::service_update(
            "test-service".to_string(),
            "running".to_string(),
            "localhost:8080".to_string(),
        );

        broadcaster.broadcast(event.clone()).await;

        // Receive event
        let received = rx.recv().await.expect("test precondition");
        assert_eq!(received.event_type, "service_update");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_event_broadcaster_unsubscribe() {
        let broadcaster = EventBroadcaster::new();

        let _rx = broadcaster
            .subscribe(
                "test-client".to_string(),
                vec!["service_update".to_string(), "health_update".to_string()],
            )
            .await;

        assert_eq!(broadcaster.subscriber_count().await, 1);

        broadcaster.unsubscribe("test-client", vec!["service_update".to_string()]).await;

        // Still subscribed to health_update
        assert_eq!(broadcaster.subscriber_count().await, 1);

        broadcaster.unsubscribe("test-client", vec!["health_update".to_string()]).await;

        // Now fully unsubscribed
        assert_eq!(broadcaster.subscriber_count().await, 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_event_type_conversion() {
        assert_eq!(EventType::ServiceUpdate.as_str(), "service_update");
        assert_eq!(EventType::HealthUpdate.as_str(), "health_update");
        assert_eq!(EventType::FederationStatus.as_str(), "federation_status");

        assert_eq!(EventType::from_str("service_update"), Some(EventType::ServiceUpdate));
        assert_eq!(EventType::from_str("invalid"), None);
    }
}
