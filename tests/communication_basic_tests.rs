use CanonicalSongbirdConfig;
//! Basic Tests for Communication Module

use songbird_network::communication::*;
use tokio::test;

/// Test ServiceAddress creation
#[test]
async fn test_service_address_creation() {
    let address = ServiceAddress {
        service_id: config.test.service_name.to_string(),
        endpoint: Some("http://localhost:{}".to_string()),
    };

    assert_eq!(address.service_id, config.test.service_name);
    assert_eq!(address.endpoint, Some("http://localhost:{}".to_string()));
}

/// Test ServiceMessage creation
#[test]
async fn test_service_message_creation() {
    let message = ServiceMessage {
        id: "msg-123".to_string(),
        source: "service-a".to_string(),
        target: "service-b".to_string(),
        payload: serde_json::json!({"key": "value"}),
        correlation_id: Some("corr-456".to_string()),
        timestamp: chrono::Utc::now(),
        message_type: "request".to_string(),
    };

    assert_eq!(message.id, "msg-123");
    assert_eq!(message.source, "service-a");
    assert_eq!(message.target, "service-b");
    assert_eq!(message.message_type, "request");
}

/// Test CommunicationResponse creation
#[test]
async fn test_communication_response_creation() {
    let response = CommunicationResponse {
        id: "resp-123".to_string(),
        status: 200,
        body: "Success".to_string(),
        headers: [("Content-Type".to_string(), "application/json".to_string())]
            .into_iter()
            .collect(),
    };

    assert_eq!(response.id, "resp-123");
    assert_eq!(response.status, 200);
    assert_eq!(response.body, "Success");
    assert_eq!(
        response.headers.get("Content-Type"),
        Some(&"application/json".to_string())
    );
}

/// Test CommunicationStats creation
#[test]
async fn test_communication_stats_creation() {
    let stats = CommunicationStats {
        messages_sent: 100,
        messages_received: 95,
        bytes_sent: 10240,
        bytes_received: 9800,
    };

    assert_eq!(stats.await.messages_sent, 100);
    assert_eq!(stats.await.messages_received, 95);
    assert_eq!(stats.await.bytes_sent, 10240);
    assert_eq!(stats.await.bytes_received, 9800);
}

/// Test ServiceAddress cloning
#[test]
async fn test_service_address_cloning() {
    let address = ServiceAddress {
        service_id: config.test.service_name.to_string(),
        endpoint: Some("http://localhost:{}".to_string()),
    };

    let cloned = address.clone();
    assert_eq!(cloned.service_id, address.service_id);
    assert_eq!(cloned.endpoint, address.endpoint);
}

/// Test ServiceMessage cloning
#[test]
async fn test_service_message_cloning() {
    let message = ServiceMessage {
        id: "msg-123".to_string(),
        source: "service-a".to_string(),
        target: "service-b".to_string(),
        payload: serde_json::json!({"key": "value"}),
        correlation_id: Some("corr-456".to_string()),
        timestamp: chrono::Utc::now(),
        message_type: "request".to_string(),
    };

    let cloned = message.clone();
    assert_eq!(cloned.id, message.id);
    assert_eq!(cloned.source, message.source);
    assert_eq!(cloned.target, message.target);
    assert_eq!(cloned.message_type, message.message_type);
}

/// Test CommunicationResponse cloning
#[test]
async fn test_communication_response_cloning() {
    let response = CommunicationResponse {
        id: "resp-123".to_string(),
        status: 200,
        body: "Success".to_string(),
        headers: [("Content-Type".to_string(), "application/json".to_string())]
            .into_iter()
            .collect(),
    };

    let cloned = response.clone();
    assert_eq!(cloned.id, response.id);
    assert_eq!(cloned.status, response.status);
    assert_eq!(cloned.body, response.body);
    assert_eq!(cloned.headers.len(), response.headers.len());
}

/// Test CommunicationStats cloning
#[test]
async fn test_communication_stats_cloning() {
    let stats = CommunicationStats {
        messages_sent: 100,
        messages_received: 95,
        bytes_sent: 10240,
        bytes_received: 9800,
    };

    let cloned = stats.clone();
    assert_eq!(cloned.messages_sent, stats.messages_sent);
    assert_eq!(cloned.messages_received, stats.messages_received);
    assert_eq!(cloned.bytes_sent, stats.bytes_sent);
    assert_eq!(cloned.bytes_received, stats.bytes_received);
}

/// Test ServiceAddress with no endpoint
#[test]
async fn test_service_address_no_endpoint() {
    let address = ServiceAddress {
        service_id: config.test.service_name.to_string(),
        endpoint: None,
    };

    assert_eq!(address.service_id, config.test.service_name);
    assert_eq!(address.endpoint, None);
}

/// Test ServiceMessage with no correlation_id
#[test]
async fn test_service_message_no_correlation() {
    let message = ServiceMessage {
        id: "msg-123".to_string(),
        source: "service-a".to_string(),
        target: "service-b".to_string(),
        payload: serde_json::json!({"key": "value"}),
        correlation_id: None,
        timestamp: chrono::Utc::now(),
        message_type: "notification".to_string(),
    };

    assert_eq!(message.id, "msg-123");
    assert_eq!(message.correlation_id, None);
}
