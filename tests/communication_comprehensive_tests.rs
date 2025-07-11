use songbird_lib::communication::{
    CommunicationLayer, CommunicationResponse, HttpCommunication,
    InMemoryCommunication, ServiceAddress, ServiceMessage, WebSocketCommunication,
};
use songbird_lib::errors::Result;
use std::collections::HashMap;
use chrono::Utc;

#[tokio::test]
async fn test_service_address_creation() -> Result<()> {
    // Test service address creation with all fields
    let addr = ServiceAddress {
        service_id: "test-service".to_string(),
        endpoint: Some("/api/v1".to_string()),
    };
    
    assert_eq!(addr.service_id, "test-service");
    assert_eq!(addr.endpoint, Some("/api/v1".to_string()));
    
    Ok(())
}

#[tokio::test]
async fn test_service_address_minimal() -> Result<()> {
    // Test service address creation with minimal fields
    let addr = ServiceAddress {
        service_id: "minimal-service".to_string(),
        endpoint: None,
    };
    
    assert_eq!(addr.service_id, "minimal-service");
    assert!(addr.endpoint.is_none());
    
    Ok(())
}

#[tokio::test]
async fn test_service_message_creation() -> Result<()> {
    // Test service message creation
    let message = ServiceMessage {
        id: "msg-123".to_string(),
        source: "sender".to_string(),
        target: "receiver".to_string(),
        payload: serde_json::json!({"key": "value"}),
        correlation_id: Some("corr-123".to_string()),
        timestamp: Utc::now(),
        message_type: "request".to_string(),
    };
    
    assert_eq!(message.id, "msg-123");
    assert_eq!(message.source, "sender");
    assert_eq!(message.target, "receiver");
    assert_eq!(message.message_type, "request");
    assert_eq!(message.correlation_id, Some("corr-123".to_string()));
    
    Ok(())
}

#[tokio::test]
async fn test_communication_response_creation() -> Result<()> {
    // Test using the correct CommunicationResponse structure
    let response = CommunicationResponse {
        id: "resp-123".to_string(),
        status: 200,
        body: "Operation completed successfully".to_string(),
        headers: HashMap::new(),
    };
    
    assert_eq!(response.id, "resp-123");
    assert_eq!(response.status, 200);
    assert_eq!(response.body, "Operation completed successfully");
    
    Ok(())
}

#[tokio::test]
async fn test_in_memory_communication_basic() -> Result<()> {
    let in_memory_comm = InMemoryCommunication::new();
    
    let dest_addr = ServiceAddress {
        service_id: "destination".to_string(),
        endpoint: None,
    };
    
    let message = ServiceMessage {
        id: "test-msg".to_string(),
        source: "source".to_string(),
        target: "destination".to_string(),
        payload: serde_json::json!({"test": "data"}),
        correlation_id: None,
        timestamp: Utc::now(),
        message_type: "request".to_string(),
    };
    
    let response = in_memory_comm.send_message(dest_addr, message).await?;
    assert_eq!(response.id, "memory-response");
    assert_eq!(response.status, 200);
    
    Ok(())
}

#[tokio::test]
async fn test_http_communication_basic() -> Result<()> {
    let http_comm = HttpCommunication::new("http://localhost:8080".to_string())?;
    
    let address = ServiceAddress {
        service_id: "test-service".to_string(),
        endpoint: Some("/test".to_string()),
    };
    
    let message = ServiceMessage {
        id: "http-test-msg".to_string(),
        source: "client".to_string(),
        target: "test-service".to_string(),
        payload: serde_json::json!({"test": "data"}),
        correlation_id: None,
        timestamp: Utc::now(),
        message_type: "request".to_string(),
    };
    
    let response = http_comm.send_message(address, message).await?;
    assert_eq!(response.id, "http-response");
    assert_eq!(response.status, 200);
    
    Ok(())
}

#[tokio::test]
async fn test_websocket_communication_basic() -> Result<()> {
    let ws_comm = WebSocketCommunication::new("localhost".to_string(), 8080);
    
    let address = ServiceAddress {
        service_id: "ws-service".to_string(),
        endpoint: None,
    };
    
    let message = ServiceMessage {
        id: "ws-test-msg".to_string(),
        source: "client".to_string(),
        target: "ws-service".to_string(),
        payload: serde_json::json!({"event": "test"}),
        correlation_id: None,
        timestamp: Utc::now(),
        message_type: "event".to_string(),
    };
    
    let response = ws_comm.send_message(address, message).await?;
    assert_eq!(response.id, "ws-response");
    assert_eq!(response.status, 200);
    
    Ok(())
}

#[tokio::test]
async fn test_communication_stats() -> Result<()> {
    let comm = InMemoryCommunication::new();
    let stats = comm.get_stats().await?;
    
    assert_eq!(stats.messages_sent, 0);
    assert_eq!(stats.messages_received, 0);
    assert_eq!(stats.bytes_sent, 0);
    assert_eq!(stats.bytes_received, 0);
    
    Ok(())
}

#[tokio::test]
async fn test_communication_connection_status() -> Result<()> {
    let comm = InMemoryCommunication::new();
    
    assert!(comm.is_connected().await);
    
    comm.connect().await?;
    assert!(comm.is_connected().await);
    
    comm.disconnect().await?;
    // For in-memory, we always return true for is_connected
    assert!(comm.is_connected().await);
    
    Ok(())
}

#[tokio::test]
async fn test_message_types() -> Result<()> {
    let message_types = vec![
        "request",
        "response", 
        "event",
        "command",
        "notification",
    ];
    
    for msg_type in message_types {
        let message = ServiceMessage {
            id: "test-msg".to_string(),
            source: "sender".to_string(),
            target: "receiver".to_string(),
            payload: serde_json::json!({}),
            correlation_id: None,
            timestamp: Utc::now(),
            message_type: msg_type.to_string(),
        };
        
        assert_eq!(message.message_type, msg_type);
    }
    
    Ok(())
} 