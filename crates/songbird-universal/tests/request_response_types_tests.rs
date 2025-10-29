//! Comprehensive tests for Universal Request and Response Types
//!
//! Tests all request/response structures, security contexts, and status enums

use serde_json::json;
use songbird_universal::types::{
    LoadBalancingConfig, LoadBalancingStrategy, ProtocolCharacteristics, ResponseStatus,
    UniversalEvent, UniversalRequest, UniversalResponse,
};
use std::collections::HashMap;

// ============================================================================
// UniversalRequest Tests
// ============================================================================

#[test]
fn test_universal_request_creation() {
    let request = UniversalRequest {
        request_id: "req-123".to_string(),
        source: "test-client".to_string(),
        target: "test-service".to_string(),
        action: "execute".to_string(),
        parameters: HashMap::new(),
        security_context: None,
    };

    assert_eq!(request.request_id, "req-123");
    assert_eq!(request.source, "test-client");
    assert_eq!(request.target, "test-service");
    assert_eq!(request.action, "execute");
    assert!(request.parameters.is_empty());
    assert!(request.security_context.is_none());
}

#[test]
fn test_universal_request_with_parameters() {
    let mut params = HashMap::new();
    params.insert("param1".to_string(), json!("value1"));
    params.insert("param2".to_string(), json!(42));

    let request = UniversalRequest {
        request_id: "req-123".to_string(),
        source: "test-client".to_string(),
        target: "test-service".to_string(),
        action: "execute".to_string(),
        parameters: params.clone(),
        security_context: None,
    };

    assert_eq!(request.parameters.len(), 2);
    assert_eq!(request.parameters.get("param1"), Some(&json!("value1")));
    assert_eq!(request.parameters.get("param2"), Some(&json!(42)));
}

#[test]
fn test_universal_request_clone() {
    let request = UniversalRequest {
        request_id: "req-123".to_string(),
        source: "test-client".to_string(),
        target: "test-service".to_string(),
        action: "execute".to_string(),
        parameters: HashMap::new(),
        security_context: None,
    };

    let cloned = request.clone();
    assert_eq!(request.request_id, cloned.request_id);
    assert_eq!(request.source, cloned.source);
}

#[test]
fn test_universal_request_debug() {
    let request = UniversalRequest {
        request_id: "req-123".to_string(),
        source: "test-client".to_string(),
        target: "test-service".to_string(),
        action: "execute".to_string(),
        parameters: HashMap::new(),
        security_context: None,
    };

    let debug_str = format!("{request:?}");
    assert!(debug_str.contains("req-123"));
    assert!(debug_str.contains("test-client"));
}

#[test]
fn test_universal_request_serialization() {
    let request = UniversalRequest {
        request_id: "req-123".to_string(),
        source: "test-client".to_string(),
        target: "test-service".to_string(),
        action: "execute".to_string(),
        parameters: HashMap::new(),
        security_context: None,
    };

    let serialized = serde_json::to_string(&request).unwrap();
    let deserialized: UniversalRequest = serde_json::from_str(&serialized).unwrap();

    assert_eq!(request.request_id, deserialized.request_id);
    assert_eq!(request.source, deserialized.source);
}

// ============================================================================
// UniversalResponse Tests
// ============================================================================

#[test]
fn test_universal_response_success() {
    let response = UniversalResponse {
        request_id: "req-123".to_string(),
        status: ResponseStatus::Success,
        data: Some(json!({"result": "ok"})),
        error: None,
        metadata: HashMap::new(),
    };

    assert_eq!(response.request_id, "req-123");
    assert_eq!(response.status, ResponseStatus::Success);
    assert!(response.data.is_some());
    assert!(response.error.is_none());
}

#[test]
fn test_universal_response_failed() {
    let response = UniversalResponse {
        request_id: "req-123".to_string(),
        status: ResponseStatus::Failed,
        data: None,
        error: Some("Operation failed".to_string()),
        metadata: HashMap::new(),
    };

    assert_eq!(response.status, ResponseStatus::Failed);
    assert!(response.data.is_none());
    assert_eq!(response.error, Some("Operation failed".to_string()));
}

#[test]
fn test_universal_response_with_metadata() {
    let mut metadata = HashMap::new();
    metadata.insert("server".to_string(), "server-1".to_string());
    metadata.insert("duration_ms".to_string(), "42".to_string());

    let response = UniversalResponse {
        request_id: "req-123".to_string(),
        status: ResponseStatus::Success,
        data: None,
        error: None,
        metadata: metadata.clone(),
    };

    assert_eq!(response.metadata.len(), 2);
    assert_eq!(response.metadata.get("server"), Some(&"server-1".to_string()));
}

#[test]
fn test_universal_response_clone() {
    let response = UniversalResponse {
        request_id: "req-123".to_string(),
        status: ResponseStatus::Success,
        data: Some(json!({"result": "ok"})),
        error: None,
        metadata: HashMap::new(),
    };

    let cloned = response.clone();
    assert_eq!(response.request_id, cloned.request_id);
    assert_eq!(response.status, cloned.status);
}

#[test]
fn test_universal_response_serialization() {
    let response = UniversalResponse {
        request_id: "req-123".to_string(),
        status: ResponseStatus::Success,
        data: Some(json!({"result": "ok"})),
        error: None,
        metadata: HashMap::new(),
    };

    let serialized = serde_json::to_string(&response).unwrap();
    let deserialized: UniversalResponse = serde_json::from_str(&serialized).unwrap();

    assert_eq!(response.request_id, deserialized.request_id);
    assert_eq!(response.status, deserialized.status);
}

// ============================================================================
// ResponseStatus Tests
// ============================================================================

#[test]
fn test_response_status_default() {
    let status = ResponseStatus::default();
    assert_eq!(status, ResponseStatus::Success);
}

#[test]
fn test_response_status_equality() {
    assert_eq!(ResponseStatus::Success, ResponseStatus::Success);
    assert_eq!(ResponseStatus::Partial, ResponseStatus::Partial);
    assert_eq!(ResponseStatus::Failed, ResponseStatus::Failed);
    assert_eq!(ResponseStatus::Timeout, ResponseStatus::Timeout);
    assert_eq!(ResponseStatus::NotFound, ResponseStatus::NotFound);
}

#[test]
fn test_response_status_inequality() {
    assert_ne!(ResponseStatus::Success, ResponseStatus::Failed);
    assert_ne!(ResponseStatus::Partial, ResponseStatus::Timeout);
    assert_ne!(ResponseStatus::Failed, ResponseStatus::NotFound);
}

#[test]
fn test_response_status_clone() {
    let status = ResponseStatus::Success;
    let cloned = status.clone();
    assert_eq!(status, cloned);
}

#[test]
fn test_response_status_debug() {
    let status = ResponseStatus::Success;
    let debug_str = format!("{status:?}");
    assert!(debug_str.contains("Success"));
}

#[test]
fn test_response_status_all_variants() {
    let statuses = vec![
        ResponseStatus::Success,
        ResponseStatus::Partial,
        ResponseStatus::Failed,
        ResponseStatus::Timeout,
        ResponseStatus::NotFound,
    ];

    assert_eq!(statuses.len(), 5);
    for status in &statuses {
        // Ensure all can be cloned and debugged
        let _ = status.clone();
        let _ = format!("{status:?}");
    }
}

// ============================================================================
// UniversalEvent Tests
// ============================================================================

#[test]
fn test_universal_event_creation() {
    let event = UniversalEvent {
        event_id: "evt-123".to_string(),
        event_type: "system.update".to_string(),
        source: "test-service".to_string(),
        timestamp: chrono::Utc::now(),
        payload: json!({"data": "test"}),
    };

    assert_eq!(event.event_id, "evt-123");
    assert_eq!(event.event_type, "system.update");
    assert_eq!(event.source, "test-service");
}

#[test]
fn test_universal_event_clone() {
    let event = UniversalEvent {
        event_id: "evt-123".to_string(),
        event_type: "system.update".to_string(),
        source: "test-service".to_string(),
        timestamp: chrono::Utc::now(),
        payload: json!({"data": "test"}),
    };

    let cloned = event.clone();
    assert_eq!(event.event_id, cloned.event_id);
    assert_eq!(event.event_type, cloned.event_type);
}

#[test]
fn test_universal_event_serialization() {
    let event = UniversalEvent {
        event_id: "evt-123".to_string(),
        event_type: "system.update".to_string(),
        source: "test-service".to_string(),
        timestamp: chrono::Utc::now(),
        payload: json!({"data": "test"}),
    };

    let serialized = serde_json::to_string(&event).unwrap();
    let deserialized: UniversalEvent = serde_json::from_str(&serialized).unwrap();

    assert_eq!(event.event_id, deserialized.event_id);
    assert_eq!(event.event_type, deserialized.event_type);
}

// ============================================================================
// ProtocolCharacteristics Tests
// ============================================================================

#[test]
fn test_protocol_characteristics_http() {
    let protocol = ProtocolCharacteristics {
        protocol_name: "HTTP".to_string(),
        version: "1.1".to_string(),
        max_message_size: 1024 * 1024, // 1MB
        supports_streaming: false,
        security_features: vec!["TLS".to_string()],
    };

    assert_eq!(protocol.protocol_name, "HTTP");
    assert_eq!(protocol.version, "1.1");
    assert_eq!(protocol.max_message_size, 1024 * 1024);
    assert!(!protocol.supports_streaming);
    assert_eq!(protocol.security_features.len(), 1);
}

#[test]
fn test_protocol_characteristics_websocket() {
    let protocol = ProtocolCharacteristics {
        protocol_name: "WebSocket".to_string(),
        version: "13".to_string(),
        max_message_size: 10 * 1024 * 1024, // 10MB
        supports_streaming: true,
        security_features: vec!["TLS".to_string(), "WSS".to_string()],
    };

    assert_eq!(protocol.protocol_name, "WebSocket");
    assert!(protocol.supports_streaming);
    assert_eq!(protocol.security_features.len(), 2);
}

#[test]
fn test_protocol_characteristics_clone() {
    let protocol = ProtocolCharacteristics {
        protocol_name: "HTTP".to_string(),
        version: "2.0".to_string(),
        max_message_size: 1024,
        supports_streaming: true,
        security_features: vec![],
    };

    let cloned = protocol.clone();
    assert_eq!(protocol.protocol_name, cloned.protocol_name);
    assert_eq!(protocol.version, cloned.version);
}

// ============================================================================
// LoadBalancingConfig Tests
// ============================================================================

#[test]
fn test_load_balancing_config_default() {
    let config = LoadBalancingConfig::default();
    // LoadBalancingStrategy doesn't implement PartialEq, so just verify it exists
    let _ = config.strategy;
    assert_eq!(config.health_check_interval.as_secs(), 30);
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.timeout.as_secs(), 10);
}

#[test]
fn test_load_balancing_config_custom() {
    let config = LoadBalancingConfig {
        strategy: LoadBalancingStrategy::LeastConnections,
        health_check_interval: std::time::Duration::from_secs(60),
        max_retries: 5,
        timeout: std::time::Duration::from_secs(30),
    };

    // LoadBalancingStrategy doesn't implement PartialEq, so just verify it exists
    let _ = config.strategy;
    assert_eq!(config.health_check_interval.as_secs(), 60);
    assert_eq!(config.max_retries, 5);
    assert_eq!(config.timeout.as_secs(), 30);
}

#[test]
fn test_load_balancing_config_clone() {
    let config = LoadBalancingConfig::default();
    let cloned = config.clone();
    // LoadBalancingStrategy doesn't implement PartialEq, so just verify clone works
    let _ = cloned.strategy;
    assert_eq!(config.max_retries, cloned.max_retries);
}

// ============================================================================
// LoadBalancingStrategy Tests
// ============================================================================

#[test]
fn test_load_balancing_strategy_creation() {
    // LoadBalancingStrategy doesn't implement PartialEq, so just verify variants exist
    let strategies = [
        LoadBalancingStrategy::RoundRobin,
        LoadBalancingStrategy::LeastConnections,
        LoadBalancingStrategy::Random,
        LoadBalancingStrategy::WeightedRoundRobin,
    ];

    assert_eq!(strategies.len(), 4);
}

#[test]
fn test_load_balancing_strategy_clone() {
    let strategy = LoadBalancingStrategy::RoundRobin;
    let cloned = strategy.clone();
    // Just verify cloning works
    let _ = format!("{cloned:?}");
}

#[test]
fn test_load_balancing_strategy_debug() {
    let strategy = LoadBalancingStrategy::RoundRobin;
    let debug_str = format!("{strategy:?}");
    assert!(!debug_str.is_empty());
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_request_response_roundtrip() {
    let request = UniversalRequest {
        request_id: "req-123".to_string(),
        source: "test-client".to_string(),
        target: "test-service".to_string(),
        action: "execute".to_string(),
        parameters: HashMap::new(),
        security_context: None,
    };

    let response = UniversalResponse {
        request_id: request.request_id.clone(),
        status: ResponseStatus::Success,
        data: Some(json!({"result": "ok"})),
        error: None,
        metadata: HashMap::new(),
    };

    assert_eq!(request.request_id, response.request_id);
}

#[test]
fn test_multiple_response_statuses() {
    let statuses = vec![
        (ResponseStatus::Success, "Success"),
        (ResponseStatus::Partial, "Partial"),
        (ResponseStatus::Failed, "Failed"),
        (ResponseStatus::Timeout, "Timeout"),
        (ResponseStatus::NotFound, "NotFound"),
    ];

    for (status, expected) in statuses {
        let debug_str = format!("{status:?}");
        assert!(debug_str.contains(expected));
    }
}

#[test]
fn test_event_with_complex_payload() {
    let payload = json!({
        "user_id": 123,
        "action": "login",
        "metadata": {
            "ip": "192.168.1.1",
            "user_agent": "Mozilla/5.0"
        }
    });

    let event = UniversalEvent {
        event_id: "evt-123".to_string(),
        event_type: "user.login".to_string(),
        source: "auth-service".to_string(),
        timestamp: chrono::Utc::now(),
        payload: payload.clone(),
    };

    assert_eq!(event.payload, payload);
}
