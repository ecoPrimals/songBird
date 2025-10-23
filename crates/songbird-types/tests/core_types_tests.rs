//! Core Types Tests
//!
//! Comprehensive tests for fundamental Songbird types

use songbird_types::{
    CanonicalAddress, CanonicalEndpoint, CanonicalNodeType, CanonicalRequest, CanonicalResponse,
    SongbirdError,
};

#[cfg(test)]
mod endpoint_tests {
    use super::*;

    #[test]
    fn test_endpoint_creation() {
        let endpoint = CanonicalEndpoint::new("localhost", 8080, "http");

        assert_eq!(endpoint.host, "localhost");
        assert_eq!(endpoint.port, 8080);
        assert_eq!(endpoint.protocol, "http");
        assert_eq!(endpoint.path, None);
    }

    #[test]
    fn test_endpoint_with_path() {
        let mut endpoint = CanonicalEndpoint::new("localhost", 8080, "http");
        let _ = endpoint.with_path("/api/v1");

        assert_eq!(endpoint.path, Some("/api/v1".to_string()));
    }

    #[test]
    fn test_endpoint_url_without_path() {
        let endpoint = CanonicalEndpoint::new("example.com", 443, "https");
        assert_eq!(endpoint.url(), "https://example.com:443");
    }

    #[test]
    fn test_endpoint_url_with_path() {
        let mut endpoint = CanonicalEndpoint::new("example.com", 443, "https");
        let _ = endpoint.with_path("/api/health");

        assert_eq!(endpoint.url(), "https://example.com:443/api/health");
    }

    #[test]
    fn test_endpoint_is_available() {
        let valid_endpoint = CanonicalEndpoint::new("localhost", 8080, "http");
        assert!(valid_endpoint.is_available());

        let invalid_endpoint = CanonicalEndpoint::new("", 0, "");
        assert!(!invalid_endpoint.is_available());
    }

    #[test]
    fn test_endpoint_display() {
        let endpoint = CanonicalEndpoint::new("localhost", 9090, "http");
        let display_str = format!("{endpoint}");
        assert_eq!(display_str, "http://localhost:9090");
    }

    #[test]
    fn test_endpoint_clone() {
        let endpoint1 = CanonicalEndpoint::new("localhost", 8080, "http");
        let endpoint2 = endpoint1.clone();

        assert_eq!(endpoint1, endpoint2);
    }

    #[test]
    fn test_endpoint_serialization() {
        let endpoint = CanonicalEndpoint::new("localhost", 8080, "http");

        let json = serde_json::to_string(&endpoint);
        assert!(json.is_ok());

        let json_str = json.expect("Test: JSON serialization should succeed");
        let deserialized: Result<CanonicalEndpoint, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());
        assert_eq!(deserialized.expect("Test: JSON deserialization should succeed"), endpoint);
    }

    #[test]
    fn test_endpoint_different_protocols() {
        let http = CanonicalEndpoint::new("localhost", 80, "http");
        let https = CanonicalEndpoint::new("localhost", 443, "https");
        let grpc = CanonicalEndpoint::new("localhost", 50051, "grpc");

        assert_eq!(http.protocol, "http");
        assert_eq!(https.protocol, "https");
        assert_eq!(grpc.protocol, "grpc");
    }

    #[test]
    fn test_endpoint_different_ports() {
        let port_8080 = CanonicalEndpoint::new("localhost", 8080, "http");
        let port_8081 = CanonicalEndpoint::new("localhost", 8081, "http");

        assert_ne!(port_8080, port_8081);
    }
}

#[cfg(test)]
mod address_tests {
    use super::*;

    #[test]
    fn test_address_creation() {
        let addr = CanonicalAddress::new("localhost", 8080, "http");

        assert!(addr.street.is_some());
        assert!(addr.addr_type.is_some());
    }

    #[test]
    fn test_address_default() {
        let addr = CanonicalAddress::default();

        assert!(addr.street.is_none());
        assert!(addr.city.is_none());
        assert!(addr.state.is_none());
        assert!(addr.country.is_none());
        assert!(addr.postal_code.is_none());
    }

    #[test]
    fn test_address_with_type() {
        let mut addr = CanonicalAddress::default();
        let _ = addr.with_type("datacenter");

        assert_eq!(addr.addr_type, Some("datacenter".to_string()));
    }

    #[test]
    fn test_address_clone() {
        let addr1 = CanonicalAddress::new("localhost", 8080, "http");
        let addr2 = addr1.clone();

        assert_eq!(addr1.street, addr2.street);
        assert_eq!(addr1.addr_type, addr2.addr_type);
    }

    #[test]
    fn test_address_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let addr = CanonicalAddress::default();

        let json = serde_json::to_string(&addr);
        assert!(json.is_ok());

        let deserialized: Result<CanonicalAddress, _> =
            serde_json::from_str(&json.map_err(|e| {
                SongbirdError::configuration(format!("Test operation failed: {e}"))
            })?);
        assert!(deserialized.is_ok());
        Ok(())
    }
}

#[cfg(test)]
mod node_type_tests {
    use super::*;

    #[test]
    fn test_node_type_creation() {
        let tower = CanonicalNodeType::Tower;
        let edge = CanonicalNodeType::Edge;
        let gateway = CanonicalNodeType::Gateway;
        let storage = CanonicalNodeType::Storage;
        let coordinator = CanonicalNodeType::Coordinator;

        // Verify all variants can be created
        assert_eq!(format!("{tower:?}"), "Tower");
        assert_eq!(format!("{edge:?}"), "Edge");
        assert_eq!(format!("{gateway:?}"), "Gateway");
        assert_eq!(format!("{storage:?}"), "Storage");
        assert_eq!(format!("{coordinator:?}"), "Coordinator");
    }

    #[test]
    fn test_node_type_default() {
        let default = CanonicalNodeType::default();
        assert_eq!(default, CanonicalNodeType::Edge);
    }

    #[test]
    fn test_node_type_display() {
        let tower = CanonicalNodeType::Tower;
        assert_eq!(tower.to_string(), "Tower");

        let edge = CanonicalNodeType::Edge;
        assert_eq!(edge.to_string(), "Edge");
    }

    #[test]
    fn test_node_type_from_str() -> Result<(), Box<dyn std::error::Error>> {
        use std::str::FromStr;

        let tower = CanonicalNodeType::from_str("tower");
        assert!(tower.is_ok());
        assert_eq!(
            tower
                .map_err(|e| SongbirdError::configuration(format!("Test operation failed: {e}")))?,
            CanonicalNodeType::Tower
        );

        let gateway = CanonicalNodeType::from_str("GATEWAY");
        assert!(gateway.is_ok());
        assert_eq!(
            gateway
                .map_err(|e| SongbirdError::configuration(format!("Test operation failed: {e}")))?,
            CanonicalNodeType::Gateway
        );
        Ok(())
    }

    #[test]
    fn test_node_type_clone() {
        let node1 = CanonicalNodeType::Coordinator;
        let node2 = node1;

        assert_eq!(node1, node2);
    }

    #[test]
    fn test_node_type_equality() {
        let tower1 = CanonicalNodeType::Tower;
        let tower2 = CanonicalNodeType::Tower;
        let edge = CanonicalNodeType::Edge;

        assert_eq!(tower1, tower2);
        assert_ne!(tower1, edge);
    }

    #[test]
    fn test_node_type_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let node = CanonicalNodeType::Gateway;

        let json = serde_json::to_string(&node);
        assert!(json.is_ok());

        let deserialized: Result<CanonicalNodeType, _> =
            serde_json::from_str(&json.map_err(|e| {
                SongbirdError::configuration(format!("Test operation failed: {e}"))
            })?);
        assert!(deserialized.is_ok());
        assert_eq!(
            deserialized
                .map_err(|e| SongbirdError::configuration(format!("Test operation failed: {e}")))?,
            node
        );
        Ok(())
    }
}

#[cfg(test)]
mod request_tests {
    use super::*;

    #[test]
    fn test_request_creation() {
        let payload = serde_json::json!({"action": "get_health"});
        let request = CanonicalRequest::new("health_check", payload);

        assert_eq!(request.operation, "health_check");
        assert!(!request.request_id.is_empty());
        assert!(request.metadata.is_empty());
    }

    #[test]
    fn test_request_with_metadata() {
        let payload = serde_json::json!({"data": "test"});
        let mut request = CanonicalRequest::new("test_operation", payload);
        request.with_metadata("user_id", "12345");
        request.with_metadata("source", "api");

        assert_eq!(request.metadata.len(), 2);
        assert_eq!(request.metadata.get("user_id"), Some(&"12345".to_string()));
        assert_eq!(request.metadata.get("source"), Some(&"api".to_string()));
    }

    #[test]
    fn test_request_with_complex_payload() {
        let payload = serde_json::json!({
            "items": [1, 2, 3],
            "metadata": {"version": "1.0"},
            "status": "active"
        });

        let request = CanonicalRequest::new("process_items", payload);
        assert_eq!(request.operation, "process_items");
        assert!(request.payload.is_object());
    }

    #[test]
    fn test_request_has_unique_ids() {
        let payload = serde_json::json!({});
        let request1 = CanonicalRequest::new("test", payload.clone());
        let request2 = CanonicalRequest::new("test", payload);

        // IDs should be unique
        assert_ne!(request1.request_id, request2.request_id);
    }

    #[test]
    fn test_request_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let payload = serde_json::json!({"test": "data"});
        let request = CanonicalRequest::new("test_op", payload);

        let json = serde_json::to_string(&request);
        assert!(json.is_ok());

        let deserialized: Result<CanonicalRequest, _> =
            serde_json::from_str(&json.map_err(|e| {
                SongbirdError::configuration(format!("Test operation failed: {e}"))
            })?);
        assert!(deserialized.is_ok());
        Ok(())
    }

    #[test]
    fn test_request_clone() {
        let payload = serde_json::json!({"test": "data"});
        let request1 = CanonicalRequest::new("test", payload);
        let request2 = request1.clone();

        assert_eq!(request1.request_id, request2.request_id);
        assert_eq!(request1.operation, request2.operation);
    }
}

#[cfg(test)]
mod response_tests {
    use super::*;

    #[test]
    fn test_response_success_creation() {
        let data = serde_json::json!({"message": "operation completed"});
        let response = CanonicalResponse::success("request-123", data);

        assert_eq!(response.request_id, "request-123");
        assert_eq!(response.status, "success");
        assert!(response.data.is_some());
        assert!(response.error_message.is_none());
    }

    #[test]
    fn test_response_error_creation() {
        let response = CanonicalResponse::error("request-456", "Operation failed");

        assert_eq!(response.request_id, "request-456");
        assert_eq!(response.status, "error");
        assert!(response.data.is_none());
        assert_eq!(response.error_message, Some("Operation failed".to_string()));
    }

    #[test]
    fn test_response_is_success() {
        let success = CanonicalResponse::success("req-1", serde_json::json!({}));
        let error = CanonicalResponse::error("req-2", "failed");

        assert!(success.is_success());
        assert!(!error.is_success());
    }

    #[test]
    fn test_response_with_complex_data() {
        let data = serde_json::json!({
            "results": [
                {"id": 1, "name": "Item 1"},
                {"id": 2, "name": "Item 2"}
            ],
            "total": 2,
            "page": 1
        });

        let response = CanonicalResponse::success("req-789", data);
        assert!(response.is_success());
        assert!(response.data.is_some());
    }

    #[test]
    fn test_response_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let response = CanonicalResponse::success("req-1", serde_json::json!({"test": "data"}));

        let json = serde_json::to_string(&response);
        assert!(json.is_ok());

        let deserialized: Result<CanonicalResponse, _> =
            serde_json::from_str(&json.map_err(|e| {
                SongbirdError::configuration(format!("Test operation failed: {e}"))
            })?);
        assert!(deserialized.is_ok());
        Ok(())
    }

    #[test]
    fn test_response_clone() {
        let response1 = CanonicalResponse::success("req-1", serde_json::json!({}));
        let response2 = response1.clone();

        assert_eq!(response1.request_id, response2.request_id);
        assert_eq!(response1.status, response2.status);
    }

    #[test]
    fn test_response_error_no_data() {
        let response = CanonicalResponse::error("req-1", "Network timeout");

        assert!(!response.is_success());
        assert!(response.data.is_none());
        assert!(response.error_message.is_some());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_endpoint_and_request_integration() {
        let endpoint = CanonicalEndpoint::new("api.example.com", 443, "https");
        let payload = serde_json::json!({"endpoint": endpoint.url()});
        let request = CanonicalRequest::new("connect", payload);

        // Verify we can use both types together
        assert!(endpoint.is_available());
        assert_eq!(request.operation, "connect");
    }

    #[test]
    fn test_request_response_flow() {
        // Create a request
        let payload = serde_json::json!({"action": "fetch_data"});
        let request = CanonicalRequest::new("fetch", payload);
        let request_id = request.request_id.clone();

        // Create corresponding success response
        let response_data = serde_json::json!({"data": [1, 2, 3]});
        let response = CanonicalResponse::success(&request_id, response_data);

        assert_eq!(request.request_id, response.request_id);
        assert!(response.is_success());
    }

    #[test]
    fn test_request_response_error_flow() {
        // Create a request
        let payload = serde_json::json!({"action": "delete"});
        let request = CanonicalRequest::new("delete", payload);
        let request_id = request.request_id.clone();

        // Create corresponding error response
        let response = CanonicalResponse::error(&request_id, "Permission denied");

        assert_eq!(request.request_id, response.request_id);
        assert!(!response.is_success());
        assert!(response.error_message.is_some());
    }

    #[test]
    fn test_types_can_be_cloned_and_compared() {
        let endpoint1 = CanonicalEndpoint::new("localhost", 8080, "http");
        let endpoint2 = endpoint1.clone();

        assert_eq!(endpoint1, endpoint2);
    }

    #[test]
    fn test_types_can_be_debugged() {
        let endpoint = CanonicalEndpoint::new("localhost", 8080, "http");
        let debug_str = format!("{endpoint:?}");

        assert!(debug_str.contains("localhost"));
        assert!(debug_str.contains("8080"));

        let node = CanonicalNodeType::Gateway;
        let node_debug = format!("{node:?}");
        assert!(node_debug.contains("Gateway"));
    }

    #[test]
    fn test_all_types_serialize() {
        // Test that all core types can be serialized
        let endpoint = CanonicalEndpoint::new("localhost", 8080, "http");
        assert!(serde_json::to_string(&endpoint).is_ok());

        let address = CanonicalAddress::default();
        assert!(serde_json::to_string(&address).is_ok());

        let node = CanonicalNodeType::Tower;
        assert!(serde_json::to_string(&node).is_ok());

        let request = CanonicalRequest::new("test", serde_json::json!({}));
        assert!(serde_json::to_string(&request).is_ok());

        let response = CanonicalResponse::success("req-1", serde_json::json!({}));
        assert!(serde_json::to_string(&response).is_ok());
    }
}
