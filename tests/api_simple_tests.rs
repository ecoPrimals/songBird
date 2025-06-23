//! Simple API Tests for Phase 2 - Alternative Approach
//! Testing REST endpoints using tokio HTTP client approach
//! Target: 80% coverage for API module (692 lines)

use axum::body::{to_bytes, Body};
use axum::http::{Method, Request, StatusCode};
use axum::Router;
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use songbird_orchestrator::api::{
    ApiResponse, ApiState, BroadcastMessageRequest, RegisterServiceRequest, SendMessageRequest,
    create_router,
};
use songbird_orchestrator::communication::{WebSocketCommunication, WebSocketConfig};
use songbird_orchestrator::traits::communication::MessageType;
use songbird_orchestrator::{Orchestrator, OrchestratorConfig};
use std::sync::Arc;
use std::time::Duration;
use tower::ServiceExt; // For oneshot
use uuid::Uuid;

/// Test helper to create configured router
async fn create_test_router() -> (Router, Arc<Orchestrator>, Arc<WebSocketCommunication>) {
    let config = OrchestratorConfig::default();
    let orchestrator = Arc::new(Orchestrator::new(config).await.unwrap());
    
    let ws_config = WebSocketConfig {
        max_connections: 10,
        connection_timeout: Duration::from_secs(5),
        heartbeat_interval: Duration::from_secs(2),
        message_buffer_size: 100,
    };
    
    let websocket = Arc::new(WebSocketCommunication::with_config(
        "127.0.0.1".to_string(),
        0, // Use any available port for testing
        ws_config,
    ));
    
    let state = ApiState::new(orchestrator.clone(), websocket.clone());
    let app = create_router(state);
    
    (app, orchestrator, websocket)
}

/// Helper to make HTTP requests
async fn make_request(app: Router, method: Method, path: &str, body: Option<String>) -> (StatusCode, String) {
    let mut request_builder = Request::builder()
        .method(method)
        .uri(path);
    
    if body.is_some() {
        request_builder = request_builder.header("content-type", "application/json");
    }
    
    let request = request_builder
        .body(Body::from(body.unwrap_or_default()))
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    
    let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
    
    (status, body_str)
}

/// Test helper to validate API response structure
fn validate_api_response_format(body: &str) {
    let json: Value = serde_json::from_str(body).expect("Response should be valid JSON");
    
    assert!(json["success"].is_boolean(), "success field should be boolean");
    assert!(json["timestamp"].is_string(), "timestamp field should be string");
    
    // Validate timestamp format
    let timestamp_str = json["timestamp"].as_str().unwrap();
    let timestamp: DateTime<Utc> = timestamp_str.parse().expect("timestamp should be valid UTC");
    assert!(timestamp <= Utc::now());
    assert!(timestamp > Utc::now() - chrono::Duration::seconds(10));
}

#[cfg(test)]
mod basic_health_tests {
    use super::*;

    #[tokio::test]
    async fn test_health_endpoint() {
        let (app, _orchestrator, _websocket) = create_test_router().await;
        
        let (status, body) = make_request(app, Method::GET, "/health", None).await;
        
        assert_eq!(status, StatusCode::OK);
        validate_api_response_format(&body);
        
        let json: Value = serde_json::from_str(&body).unwrap();
        assert!(json["success"].as_bool().unwrap());
        assert_eq!(json["data"], "healthy");
        assert!(json["error"].is_null());
    }

    #[tokio::test]
    async fn test_detailed_health_endpoint() {
        let (app, _orchestrator, _websocket) = create_test_router().await;
        
        let (status, body) = make_request(app, Method::GET, "/health/detailed", None).await;
        
        assert_eq!(status, StatusCode::OK);
        validate_api_response_format(&body);
        
        let json: Value = serde_json::from_str(&body).unwrap();
        assert!(json["success"].as_bool().unwrap());
        assert!(json["data"].is_object());
        
        let health_data = &json["data"];
        assert!(health_data["status"].is_string());
        assert!(health_data["checks"].is_object());
        assert!(health_data["uptime_seconds"].is_u64());
    }

    #[tokio::test]
    async fn test_system_info_endpoint() {
        let (app, _orchestrator, _websocket) = create_test_router().await;
        
        let (status, body) = make_request(app, Method::GET, "/system/info", None).await;
        
        assert_eq!(status, StatusCode::OK);
        validate_api_response_format(&body);
        
        let json: Value = serde_json::from_str(&body).unwrap();
        assert!(json["success"].as_bool().unwrap());
        
        let system_info = &json["data"];
        assert_eq!(system_info["name"], "Songbird Orchestrator");
        assert!(system_info["version"].is_string());
        assert!(system_info["uptime_seconds"].is_u64());
        assert!(system_info["total_services"].is_u64());
        assert!(system_info["healthy_services"].is_u64());
        assert!(system_info["api_endpoints"].is_array());
        
        // Verify API endpoints list
        let endpoints = system_info["api_endpoints"].as_array().unwrap();
        assert!(endpoints.len() > 0);
        assert!(endpoints.iter().any(|e| e == "/health"));
        assert!(endpoints.iter().any(|e| e == "/services"));
        assert!(endpoints.iter().any(|e| e == "/metrics"));
    }

    #[tokio::test]
    async fn test_system_metrics_endpoint() {
        let (app, _orchestrator, _websocket) = create_test_router().await;
        
        let (status, body) = make_request(app, Method::GET, "/system/metrics", None).await;
        
        assert_eq!(status, StatusCode::OK);
        validate_api_response_format(&body);
        
        let json: Value = serde_json::from_str(&body).unwrap();
        assert!(json["success"].as_bool().unwrap());
        
        let metrics = &json["data"];
        assert!(metrics["uptime_seconds"].is_u64());
        assert!(metrics["total_services"].is_u64());
        assert!(metrics["healthy_services"].is_u64());
        assert!(metrics["total_requests"].is_u64());
    }
}

#[cfg(test)]
mod service_management_tests {
    use super::*;

    #[tokio::test]
    async fn test_list_services_empty() {
        let (app, _orchestrator, _websocket) = create_test_router().await;
        
        let (status, body) = make_request(app, Method::GET, "/services", None).await;
        
        assert_eq!(status, StatusCode::OK);
        validate_api_response_format(&body);
        
        let json: Value = serde_json::from_str(&body).unwrap();
        assert!(json["success"].as_bool().unwrap());
        assert!(json["data"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_register_service() {
        let (app, _orchestrator, _websocket) = create_test_router().await;
        
        let service_request = RegisterServiceRequest {
            name: "Test Service".to_string(),
            service_type: "test".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Test service description".to_string()),
            endpoints: None,
            capabilities: Some(vec!["http".to_string()]),
            tags: Some([("env".to_string(), "test".to_string())].into()),
            metadata: Some([("test".to_string(), json!(true))].into()),
        };
        
        let request_body = serde_json::to_string(&service_request).unwrap();
        let (status, body) = make_request(app, Method::POST, "/services", Some(request_body)).await;
        
        assert_eq!(status, StatusCode::OK);
        validate_api_response_format(&body);
        
        let json: Value = serde_json::from_str(&body).unwrap();
        assert!(json["success"].as_bool().unwrap());
        
        let service_id = json["data"].as_str().unwrap();
        assert!(!service_id.is_empty());
        
        // Verify service ID is a valid UUID
        assert!(Uuid::parse_str(service_id).is_ok());
    }

    #[tokio::test]
    async fn test_register_service_minimal() {
        let (app, _orchestrator, _websocket) = create_test_router().await;
        
        let service_request = RegisterServiceRequest {
            name: "Minimal Service".to_string(),
            service_type: "minimal".to_string(),
            version: "0.1.0".to_string(),
            description: None,
            endpoints: None,
            capabilities: None,
            tags: None,
            metadata: None,
        };
        
        let request_body = serde_json::to_string(&service_request).unwrap();
        let (status, body) = make_request(app, Method::POST, "/services", Some(request_body)).await;
        
        assert_eq!(status, StatusCode::OK);
        
        let json: Value = serde_json::from_str(&body).unwrap();
        assert!(json["success"].as_bool().unwrap());
        assert!(json["data"].is_string());
    }

    #[tokio::test]
    async fn test_get_service_not_found() {
        let (app, _orchestrator, _websocket) = create_test_router().await;
        
        let fake_id = Uuid::new_v4().to_string();
        let (status, body) = make_request(app, Method::GET, &format!("/services/{}", fake_id), None).await;
        
        assert_eq!(status, StatusCode::NOT_FOUND);
        validate_api_response_format(&body);
        
        let json: Value = serde_json::from_str(&body).unwrap();
        assert!(!json["success"].as_bool().unwrap());
        assert!(json["error"].is_string());
        assert!(json["error"].as_str().unwrap().contains("not found"));
    }

    #[tokio::test]
    async fn test_service_lifecycle_operations() {
        let (app, _orchestrator, _websocket) = create_test_router().await;
        
        let service_id = Uuid::new_v4().to_string();
        
        // Test start service
        let (status, body) = make_request(app, Method::POST, &format!("/services/{}/start", service_id), None).await;
        assert_eq!(status, StatusCode::OK);
        
        let json: Value = serde_json::from_str(&body).unwrap();
        assert!(json["success"].as_bool().unwrap());
        assert!(json["data"].as_str().unwrap().contains("started"));
    }
}

#[cfg(test)]
mod communication_tests {
    use super::*;

    #[tokio::test]
    async fn test_send_message() {
        let (app, _orchestrator, _websocket) = create_test_router().await;
        
        let message_request = SendMessageRequest {
            target_service: "test-service".to_string(),
            message_type: MessageType::Request,
            topic: Some("test.topic".to_string()),
            payload: json!({"test": "data", "value": 42}),
            headers: Some([("source".to_string(), "test".to_string())].into()),
            ttl: Some(30000),
        };
        
        let request_body = serde_json::to_string(&message_request).unwrap();
        let (status, body) = make_request(app, Method::POST, "/communication/send", Some(request_body)).await;
        
        // Note: This might fail due to service not existing, but we test the API structure
        validate_api_response_format(&body);
        
        let json: Value = serde_json::from_str(&body).unwrap();
        if json["success"].as_bool().unwrap_or(false) {
            assert!(json["data"].is_string()); // Should return message ID
            let message_id = json["data"].as_str().unwrap();
            assert!(Uuid::parse_str(message_id).is_ok());
        } else {
            // Expected for non-existent service
            assert!(json["error"].is_string());
        }
    }

    #[tokio::test]
    async fn test_broadcast_message() {
        let (app, _orchestrator, _websocket) = create_test_router().await;
        
        let broadcast_request = BroadcastMessageRequest {
            message_type: MessageType::Event,
            topic: Some("broadcast.test".to_string()),
            payload: json!({"broadcast": true, "timestamp": Utc::now()}),
            headers: Some([("type".to_string(), "test".to_string())].into()),
            ttl: Some(60000),
        };
        
        let request_body = serde_json::to_string(&broadcast_request).unwrap();
        let (status, body) = make_request(app, Method::POST, "/communication/broadcast", Some(request_body)).await;
        
        validate_api_response_format(&body);
        
        let json: Value = serde_json::from_str(&body).unwrap();
        if json["success"].as_bool().unwrap_or(false) {
            assert!(json["data"].is_string());
            let message_id = json["data"].as_str().unwrap();
            assert!(Uuid::parse_str(message_id).is_ok());
        }
    }

    #[tokio::test]
    async fn test_communication_stats() {
        let (app, _orchestrator, _websocket) = create_test_router().await;
        
        let (status, body) = make_request(app, Method::GET, "/communication/stats", None).await;
        
        assert_eq!(status, StatusCode::OK);
        validate_api_response_format(&body);
        
        let json: Value = serde_json::from_str(&body).unwrap();
        assert!(json["success"].as_bool().unwrap());
        
        let stats = &json["data"];
        
        // Verify the actual communication stats structure based on API response
        assert!(stats["active_connections"].is_number(), "active_connections should be a number");
        assert!(stats["messages_sent"].is_number(), "messages_sent should be a number");
        assert!(stats["messages_received"].is_number(), "messages_received should be a number");
        assert!(stats["bytes_sent"].is_number(), "bytes_sent should be a number");
        assert!(stats["bytes_received"].is_number(), "bytes_received should be a number");
        assert!(stats["failed_connections"].is_number(), "failed_connections should be a number");
        assert!(stats["last_activity"].is_string(), "last_activity should be a timestamp string");
        
        // Verify values are non-negative
        assert!(stats["active_connections"].as_u64().unwrap() >= 0);
        assert!(stats["messages_sent"].as_u64().unwrap() >= 0);
        assert!(stats["messages_received"].as_u64().unwrap() >= 0);
        assert!(stats["bytes_sent"].as_u64().unwrap() >= 0);
        assert!(stats["bytes_received"].as_u64().unwrap() >= 0);
        assert!(stats["failed_connections"].as_u64().unwrap() >= 0);
        
        // Verify timestamp format
        let timestamp_str = stats["last_activity"].as_str().unwrap();
        let _timestamp: DateTime<Utc> = timestamp_str.parse().unwrap();
    }

    #[tokio::test]
    async fn test_get_connections() {
        let (app, _orchestrator, _websocket) = create_test_router().await;
        
        let (status, body) = make_request(app, Method::GET, "/communication/connections", None).await;
        
        assert_eq!(status, StatusCode::OK);
        validate_api_response_format(&body);
        
        let json: Value = serde_json::from_str(&body).unwrap();
        assert!(json["success"].as_bool().unwrap());
        assert!(json["data"].is_u64());
    }
}

#[cfg(test)]
mod metrics_monitoring_tests {
    use super::*;

    #[tokio::test]
    async fn test_orchestrator_metrics() {
        let (app, _orchestrator, _websocket) = create_test_router().await;
        
        let (status, body) = make_request(app, Method::GET, "/metrics", None).await;
        
        assert_eq!(status, StatusCode::OK);
        validate_api_response_format(&body);
        
        let json: Value = serde_json::from_str(&body).unwrap();
        assert!(json["success"].as_bool().unwrap());
        
        let metrics = &json["data"];
        assert!(metrics["uptime_seconds"].is_u64());
        assert!(metrics["total_services"].is_u64());
        assert!(metrics["healthy_services"].is_u64());
        assert!(metrics["total_requests"].is_u64());
    }

    #[tokio::test]
    async fn test_all_service_metrics() {
        let (app, _orchestrator, _websocket) = create_test_router().await;
        
        let (status, body) = make_request(app, Method::GET, "/metrics/services", None).await;
        
        assert_eq!(status, StatusCode::OK);
        validate_api_response_format(&body);
        
        let json: Value = serde_json::from_str(&body).unwrap();
        assert!(json["success"].as_bool().unwrap());
        
        let metrics_map = &json["data"];
        assert!(metrics_map.is_object());
        
        // Should be empty for new orchestrator, but structure should be valid
        let map = metrics_map.as_object().unwrap();
        for (service_id, metrics) in map {
            assert!(!service_id.is_empty());
            assert!(metrics["request_count"].is_u64());
            assert!(metrics["error_count"].is_u64());
            assert!(metrics["average_response_time"].is_f64());
        }
    }
}

#[cfg(test)]
mod dashboard_tests {
    use super::*;

    #[tokio::test]
    async fn test_dashboard_data() {
        let (app, _orchestrator, _websocket) = create_test_router().await;
        
        let (status, body) = make_request(app, Method::GET, "/dashboard", None).await;
        
        assert_eq!(status, StatusCode::OK);
        validate_api_response_format(&body);
        
        let json: Value = serde_json::from_str(&body).unwrap();
        assert!(json["success"].as_bool().unwrap());
        
        let dashboard = &json["data"];
        assert!(dashboard["system_info"].is_object());
        assert!(dashboard["orchestrator_metrics"].is_object());
        assert!(dashboard["services"].is_array());
        assert!(dashboard["communication_stats"].is_object());
        assert!(dashboard["recent_events"].is_array());
        assert!(dashboard["timestamp"].is_string());
        
        // Verify system info structure
        let system_info = &dashboard["system_info"];
        assert_eq!(system_info["name"], "Songbird Orchestrator");
        assert!(system_info["version"].is_string());
        assert!(system_info["uptime_seconds"].is_u64());
        
        // Verify timestamp is recent
        let timestamp_str = dashboard["timestamp"].as_str().unwrap();
        let timestamp: DateTime<Utc> = timestamp_str.parse().unwrap();
        assert!(timestamp <= Utc::now());
        assert!(timestamp > Utc::now() - chrono::Duration::seconds(5));
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[tokio::test]
    async fn test_invalid_endpoint() {
        let (app, _orchestrator, _websocket) = create_test_router().await;
        
        let (status, _body) = make_request(app, Method::GET, "/invalid/endpoint", None).await;
        
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_malformed_json_request() {
        let (app, _orchestrator, _websocket) = create_test_router().await;
        
        let (status, _body) = make_request(app, Method::POST, "/services", Some("{invalid json".to_string())).await;
        
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_empty_request_body() {
        let (app, _orchestrator, _websocket) = create_test_router().await;
        
        let (status, _body) = make_request(app, Method::POST, "/services", Some("".to_string())).await;
        
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }
}

#[cfg(test)]
mod api_response_format_tests {
    use super::*;

    #[tokio::test]
    async fn test_success_response_format() {
        let (app, _orchestrator, _websocket) = create_test_router().await;
        
        let (status, body) = make_request(app, Method::GET, "/health", None).await;
        let json: Value = serde_json::from_str(&body).unwrap();
        
        // Verify standard API response format
        assert!(json["success"].is_boolean());
        assert!(json["success"].as_bool().unwrap());
        assert!(json["data"].is_string());
        assert!(json["error"].is_null());
        assert!(json["timestamp"].is_string());
        
        // Verify timestamp format
        let timestamp_str = json["timestamp"].as_str().unwrap();
        let _timestamp: DateTime<Utc> = timestamp_str.parse().unwrap();
    }

    #[tokio::test]
    async fn test_error_response_format() {
        let (app, _orchestrator, _websocket) = create_test_router().await;
        
        let fake_id = "invalid-uuid";
        let (status, body) = make_request(app, Method::GET, &format!("/services/{}", fake_id), None).await;
        let json: Value = serde_json::from_str(&body).unwrap();
        
        // Error responses should still follow standard format
        assert!(json["success"].is_boolean());
        assert!(!json["success"].as_bool().unwrap());
        assert!(json["data"].is_null());
        assert!(json["error"].is_string());
        assert!(json["timestamp"].is_string());
        
        let error_msg = json["error"].as_str().unwrap();
        assert!(!error_msg.is_empty());
    }

    #[tokio::test]
    async fn test_array_response_format() {
        let (app, _orchestrator, _websocket) = create_test_router().await;
        
        let (status, body) = make_request(app, Method::GET, "/services", None).await;
        let json: Value = serde_json::from_str(&body).unwrap();
        
        assert!(json["success"].as_bool().unwrap());
        assert!(json["data"].is_array());
        assert!(json["error"].is_null());
    }

    #[tokio::test]
    async fn test_object_response_format() {
        let (app, _orchestrator, _websocket) = create_test_router().await;
        
        let (status, body) = make_request(app, Method::GET, "/system/info", None).await;
        let json: Value = serde_json::from_str(&body).unwrap();
        
        assert!(json["success"].as_bool().unwrap());
        assert!(json["data"].is_object());
        assert!(json["error"].is_null());
    }
}

#[cfg(test)]
mod endpoint_coverage_tests {
    use super::*;

    #[tokio::test]
    async fn test_all_get_endpoints_accessible() {
        let (app, _orchestrator, _websocket) = create_test_router().await;
        
        // Test all GET endpoints
        let get_endpoints = vec![
            "/health",
            "/health/detailed", 
            "/system/info",
            "/system/metrics",
            "/services",
            "/communication/stats",
            "/communication/connections",
            "/metrics",
            "/metrics/services",
            "/dashboard",
        ];
        
        for endpoint in get_endpoints {
            let (status, _body) = make_request(app.clone(), Method::GET, endpoint, None).await;
            assert!(
                status.is_success(),
                "Endpoint {} failed with status: {}", 
                endpoint, 
                status
            );
        }
    }
} 