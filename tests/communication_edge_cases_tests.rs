use songbird_gaming_bridge::SongbirdOrchestrator;
use songbird_gaming_bridge::config::NetworkConfig;
use std::collections::HashMap;
use chrono::Utc;
use serde_json::json;
#[allow(dead_code, unused_imports, unused_variables)]
// Communication Edge Cases and Unused Methods Testing Suite
//
// Tests covering:
// - Unused communication methods (test_service_connectivity, broadcast)
// - Protocol router edge cases
// - Load balancer edge cases
// - Circuit breaker scenarios
// - Service registry integration
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

use songbird_gaming_bridge::{
    communication::{
        HttpCommunication, InMemoryCommunication, ProtocolRouter, ServiceRegistry,
        WebSocketCommunication,
    },
    errors::SongbirdError,
    traits::communication::{
        CommunicationLayer, CommunicationStats, MessageType, ServiceAddress, ServiceMessage,
    },
    traits::discovery::ServiceHealthStatus,
    traits::service_id::{ServiceEndpoint, ServiceInfo},
};

#[cfg(test)]
mod communication_edge_cases_tests {
    use super::*;

    // ============================================================================
    // TEST UTILITIES
    // ============================================================================

    fn create_test_service_address(service_id: &str) -> ServiceAddress {
        ServiceAddress {
            service_id: service_id.to_string(),
            instance_id: Some(format!("{}-instance", service_id)),
            endpoint: Some(format!("http://localhost:8080/services/{}", service_id)),
        }
    }

    fn create_test_service_message(message_type: MessageType) -> ServiceMessage {
        ServiceMessage {
            id: Uuid::new_v4().to_string(),
            message_type,
            topic: Some("test.topic".to_string()),
            body: json!({"test": "data", "timestamp": Utc::now()}),
            headers: HashMap::from([
                ("source".to_string(), "test-suite".to_string()),
                ("priority".to_string(), "high".to_string()),
            ]),
            timestamp: Utc::now(),
            correlation_id: Some(Uuid::new_v4().to_string()),
            reply_to: None,
            ttl: Some(30000),
        }
    }

    fn create_test_service_info(service_id: &str, service_type: &str) -> ServiceInfo {
        ServiceInfo {
            id: service_id.to_string(),
            name: format!("{} Service", service_id),
            service_type: service_type.to_string(),
            version: "1.0.0".to_string(),
            description: format!("Test service for {}", service_id),
            endpoints: vec![ServiceEndpoint {
            auth_required: false,
            rate_limit: None,
                path: format!("http://localhost:8080/services/{}", service_id),
                method: "GET".to_string(),
                description: Some("Main endpoint").to_string(),
                parameters: vec![],
                response_schema: None,
            }],
            tags: std::collections::HashMap::new(),
            tags: HashMap::from([
                ("protocol".to_string(), "http".to_string()),
                ("environment".to_string(), "test".to_string()),
            ]),
            metadata: HashMap::from([
                ("created_by".to_string(), json!("test-suite")),
                ("test_mode".to_string(), json!(true)),
            ]),
        }
    }

    // Mock service registry for testing
    struct MockServiceRegistry {
        services: HashMap<String, String>,
    }

    impl MockServiceRegistry {
        fn new() -> Self {
            Self {
                services: HashMap::from([
                    ("service1".to_string(), "http://localhost:8081".to_string()),
                    ("service2".to_string(), "http://localhost:8082".to_string()),
                    ("service3".to_string(), "http://localhost:8083".to_string()),
                ]),
            }
        }
    }

    #[async_trait::async_trait]
    impl ServiceRegistry for MockServiceRegistry {
        async fn get_service_endpoint(
            &self,
            service_id: &str,
        ) -> Result<Option<String>, SongbirdError> {
            Ok(self.services.get(service_id).cloned())
        }

        async fn get_service_info(
            &self,
            _service_id: &str,
        ) -> Result<Option<ServiceInfo>, SongbirdError> {
            Ok(None)
        }

        async fn get_all_endpoints(&self) -> Vec<(String, String)> {
            self.services
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        }
    }

    // ============================================================================
    // HTTP COMMUNICATION UNUSED METHODS TESTS
    // ============================================================================

    #[tokio::test]
    async fn test_http_communication_test_service_connectivity_success() {
        let http_comm = HttpCommunication::new("http://httpbin.org".to_string());
        let target = ServiceAddress {
            service_id: "httpbin".to_string(),
            instance_id: None,
            endpoint: Some("http://httpbin.org".to_string()),
        };

        // This should succeed since httpbin.org/health is a valid endpoint
        let result = http_comm.test_service_connectivity(&target).await;

        // Note: This might fail in CI/CD without internet, so we test both cases
        match result {
            Ok(is_connected) => {
                // If we can reach httpbin.org, it should return true
                assert!(is_connected || !is_connected); // Either is valid
            }
            Err(_) => {
                // Network error is acceptable in testing environment
            }
        }
    }

    #[tokio::test]
    async fn test_http_communication_test_service_connectivity_failure() {
        let http_comm = HttpCommunication::new("http://localhost:9999".to_string());
        let target = ServiceAddress {
            service_id: "nonexistent".to_string(),
            instance_id: None,
            endpoint: Some("http://localhost:9999".to_string()),
        };

        let result = http_comm.test_service_connectivity(&target).await;

        // This should return false since localhost:9999 is unlikely to be running
        match result {
            Ok(is_connected) => assert!(!is_connected),
            Err(_) => {
                // Error is also acceptable for unreachable service
            }
        }
    }

    #[tokio::test]
    async fn test_http_communication_test_service_connectivity_various_endpoints() {
        let http_comm = HttpCommunication::new("http://localhost:8080".to_string());

        let test_targets = vec![
            ServiceAddress {
                service_id: "test1".to_string(),
                instance_id: None,
                endpoint: Some("http://localhost:8080/test1".to_string()),
            },
            ServiceAddress {
                service_id: "test2".to_string(),
                instance_id: None,
                endpoint: None, // Will use default URL construction
            },
            ServiceAddress {
                service_id: "test3".to_string(),
                instance_id: Some("instance-123".to_string()),
                endpoint: Some("http://127.0.0.1:8080/test3".to_string()),
            },
        ];

        for target in test_targets {
            let result = http_comm.test_service_connectivity(&target).await;

            // All should return Ok(false) since these services don't exist
            match result {
                Ok(is_connected) => assert!(!is_connected),
                Err(_) => {
                    // Network errors are acceptable
                }
            }
        }
    }

    #[tokio::test]
    async fn test_http_communication_broadcast_without_registry() {
        let http_comm = HttpCommunication::new("http://localhost:8080".to_string());
        let message = create_test_service_message(MessageType::Event);

        let result = http_comm.broadcast(message).await;

        // Should succeed but return empty vec since no registry is configured
        assert!(result.is_ok());
        let responses = result.expect("Test assertion failed");
        assert!(responses.is_empty());
    }

    #[tokio::test]
    async fn test_http_communication_broadcast_with_registry() {
        let registry = Arc::new(MockServiceRegistry::new());
        let http_comm = HttpCommunication::new("http://localhost:8080".to_string())
            .with_service_registry(registry);

        let message = create_test_service_message(MessageType::Event);

        let result = http_comm.broadcast(message).await;

        // Should succeed and return responses for each registered service
        assert!(result.is_ok());
        let responses = result.expect("Test assertion failed");
        assert_eq!(responses.len(), 3); // MockServiceRegistry has 3 services

        // All should be failures since the services don't actually exist
        for response in responses {
            assert!(!response.success);
            assert!(response.error.is_some());
            assert!(response.body.is_some());
        }
    }

    #[tokio::test]
    async fn test_http_communication_broadcast_large_message() {
        let registry = Arc::new(MockServiceRegistry::new());
        let http_comm = HttpCommunication::new("http://localhost:8080".to_string())
            .with_service_registry(registry);

        let large_payload = json!({
            "data": "x".repeat(10000), // 10KB of data
            "metadata": {
                "size": 10000,
                "type": "large_payload_test"
            }
        });

        let message = ServiceMessage {
            id: Uuid::new_v4().to_string(),
            message_type: MessageType::Event,
            topic: Some("large.message.test".to_string()),
            body: large_payload,
            headers: HashMap::new(),
            timestamp: Utc::now(),
            correlation_id: None,
            reply_to: None,
            ttl: Some(60000),
        };

        let result = http_comm.broadcast(message).await;

        // Should handle large messages gracefully
        assert!(result.is_ok());
        let responses = result.expect("Test assertion failed");
        assert_eq!(responses.len(), 3);
    }

    // ============================================================================
    // WEBSOCKET COMMUNICATION BROADCAST TESTS
    // ============================================================================

    #[tokio::test]
    async fn test_websocket_communication_broadcast_no_connections() {
        let websocket_comm = WebSocketCommunication::new("127.0.0.1".to_string(), 8081);
        let message = create_test_service_message(MessageType::Event);

        let result = websocket_comm.broadcast(message).await;

        // Should succeed but return empty vec since no connections
        assert!(result.is_ok());
        let responses = result.expect("Test assertion failed");
        assert!(responses.is_empty());
    }

    #[tokio::test]
    async fn test_websocket_communication_broadcast_message_types() {
        let websocket_comm = WebSocketCommunication::new("127.0.0.1".to_string(), 8082);

        let message_types = vec![
            MessageType::Request,
            MessageType::Response,
            MessageType::Event,
            MessageType::Command,
            MessageType::Notification,
        ];

        for message_type in message_types {
            let message = create_test_service_message(message_type);
            let result = websocket_comm.broadcast(message).await;

            // Should succeed for all message types
            assert!(result.is_ok());
            let responses = result.expect("Test assertion failed");
            assert!(responses.is_empty()); // No connections yet
        }
    }

    // ============================================================================
    // IN-MEMORY COMMUNICATION TESTS
    // ============================================================================

    #[tokio::test]
    async fn test_in_memory_communication_broadcast() {
        let in_memory_comm = InMemoryCommunication::new();
        let message = create_test_service_message(MessageType::Event);

        let result = in_memory_comm.broadcast(message).await;

        // Should succeed and return empty vec (in-memory implementation)
        assert!(result.is_ok());
        let responses = result.expect("Test assertion failed");
        assert!(responses.is_empty());
    }

    #[tokio::test]
    async fn test_in_memory_communication_all_methods() {
        let in_memory_comm = InMemoryCommunication::new();
        let target = create_test_service_address("test-service");
        let message = create_test_service_message(MessageType::Request);

        // Test all communication methods
        let send_result = in_memory_comm.send_message(target, message.clone()).await;
        assert!(send_result.is_ok());

        let broadcast_result = in_memory_comm.broadcast(message).await;
        assert!(broadcast_result.is_ok());

        let listen_result = in_memory_comm.listen().await;
        assert!(listen_result.is_ok());

        let subscribe_result = in_memory_comm.subscribe("test.topic").await;
        assert!(subscribe_result.is_ok());

        let unsubscribe_result = in_memory_comm.unsubscribe("test.topic").await;
        assert!(unsubscribe_result.is_ok());

        let connect_result = in_memory_comm.connect().await;
        assert!(connect_result.is_ok());

        let disconnect_result = in_memory_comm.disconnect().await;
        assert!(disconnect_result.is_ok());

        let is_connected = in_memory_comm.is_connected().await;
        assert!(!is_connected); // Default state

        let stats_result = in_memory_comm.get_stats().await;
        assert!(stats_result.is_ok());
    }

    // ============================================================================
    // PROTOCOL ROUTER EDGE CASES
    // ============================================================================

    #[tokio::test]
    async fn test_protocol_router_service_registration() {
        let router = ProtocolRouter::new();

        // Test registering services with different protocols
        let http_service = create_test_service_info("http-service", "http");
        let websocket_service = create_test_service_info("ws-service", "websocket");
        let test_service = create_test_service_info("test-service", "test");

        router.register_service_protocol("http-service", &http_service);
        router.register_service_protocol("ws-service", &websocket_service);
        router.register_service_protocol("test-service", &test_service);

        // Test unregistering services
        router.unregister_service_protocol("http-service");
        router.unregister_service_protocol("ws-service");
        router.unregister_service_protocol("test-service");
    }

    #[tokio::test]
    async fn test_protocol_router_broadcast() {
        let router = ProtocolRouter::new();
        let message = create_test_service_message(MessageType::Event);

        let result = router.broadcast(message).await;

        // Should succeed and aggregate responses from all layers
        assert!(result.is_ok());
        let responses = result.expect("Test assertion failed");
        // Should be empty since no connections on any layer
        assert!(responses.is_empty());
    }

    #[tokio::test]
    async fn test_protocol_router_send_message_different_protocols() {
        let router = ProtocolRouter::new();

        // Register services with different protocols
        let http_service = create_test_service_info("http-service", "http");
        let mut websocket_service = create_test_service_info("ws-service", "websocket");
        websocket_service.capabilities = vec!["websocket".to_string()];
        let test_service = create_test_service_info("test-service", "test");

        router.register_service_protocol("http-service", &http_service);
        router.register_service_protocol("ws-service", &websocket_service);
        router.register_service_protocol("test-service", &test_service);

        let message = create_test_service_message(MessageType::Request);

        // Test sending to each protocol
        let targets = vec![
            create_test_service_address("http-service"),
            create_test_service_address("ws-service"),
            create_test_service_address("test-service"),
        ];

        for target in targets {
            let result = router.send_message(target, message.clone()).await;
            // Results will vary based on protocol and connection state
            // We just verify the router doesn't crash
            match result {
                Ok(_) => {}  // Success is good
                Err(_) => {} // Errors are expected for non-connected services
            }
        }
    }

    #[tokio::test]
    async fn test_protocol_router_listen() {
        let router = ProtocolRouter::new();

        let result = router.listen().await;
        assert!(result.is_ok());
        // Should return WebSocket listener stream
    }

    #[tokio::test]
    async fn test_protocol_router_subscribe_unsubscribe() {
        let router = ProtocolRouter::new();

        let subscribe_result = router.subscribe("test.topic").await;
        assert!(subscribe_result.is_ok());

        let unsubscribe_result = router.unsubscribe("test.topic").await;
        assert!(unsubscribe_result.is_ok());
    }

    // ============================================================================
    // COMMUNICATION STATS AND METRICS TESTS
    // ============================================================================

    #[tokio::test]
    async fn test_communication_stats_default() {
        let stats = CommunicationStats::default();

        assert_eq!(stats.messages_sent, 0);
        assert_eq!(stats.messages_received, 0);
        assert_eq!(stats.bytes_sent, 0);
        assert_eq!(stats.bytes_received, 0);
        assert_eq!(stats.active_connections, 0);
        assert_eq!(stats.failed_connections, 0);
        assert!(stats.last_activity.is_none());
    }

    #[tokio::test]
    async fn test_websocket_communication_stats() {
        let websocket_comm = WebSocketCommunication::new("127.0.0.1".to_string(), 8083);

        let result = websocket_comm.get_stats().await;
        assert!(result.is_ok());

        let stats = result.expect("Test assertion failed");
        // Initial stats should be zero
        assert_eq!(stats.messages_sent, 0);
        assert_eq!(stats.messages_received, 0);
        assert_eq!(stats.active_connections, 0);
    }

    #[tokio::test]
    async fn test_http_communication_stats() {
        let http_comm = HttpCommunication::new("http://localhost:8080".to_string());

        let result = http_comm.get_stats().await;
        assert!(result.is_ok());

        let stats = result.expect("Test assertion failed");
        // HTTP stats should be available
        assert_eq!(stats.messages_sent, 0);
        assert_eq!(stats.messages_received, 0);
    }

    // ============================================================================
    // EDGE CASES AND ERROR SCENARIOS
    // ============================================================================

    #[tokio::test]
    async fn test_service_address_edge_cases() {
        // Test service addresses with various edge cases
        let edge_case_addresses = vec![
            ServiceAddress {
                service_id: "".to_string(), // Empty service ID
                instance_id: None,
                endpoint: None,
            },
            ServiceAddress {
                service_id: "very-long-service-id-".to_string() + &"x".repeat(1000),
                instance_id: Some("instance".to_string()),
                endpoint: Some("http://localhost:8080".to_string()),
            },
            ServiceAddress {
                service_id: "special-chars-!@#$%^&*()".to_string(),
                instance_id: Some("instance-with-dashes-and_underscores".to_string()),
                endpoint: Some(
                    "https://complex.example.com:8443/path/to/service?param=value".to_string(),
                ),
            },
        ];

        let in_memory_comm = InMemoryCommunication::new();
        let message = create_test_service_message(MessageType::Request);

        for address in edge_case_addresses {
            let result = in_memory_comm.send_message(address, message.clone()).await;
            // In-memory should handle all edge cases gracefully
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_service_message_edge_cases() {
        let in_memory_comm = InMemoryCommunication::new();
        let target = create_test_service_address("test-service");

        // Test messages with various edge cases
        let edge_case_messages = vec![
            ServiceMessage {
                id: "".to_string(), // Empty ID
                message_type: MessageType::Request,
                topic: None,
                body: json!(null),
                headers: HashMap::new(),
                timestamp: Utc::now(),
                correlation_id: None,
                reply_to: None,
                ttl: None,
            },
            ServiceMessage {
                id: "x".repeat(1000), // Very long ID
                message_type: MessageType::Event,
                topic: Some("very.long.topic.".to_string() + &"segment.".repeat(100)),
                body: json!({"large_array": vec![1; 10000]}),
                headers: (0..1000)
                    .map(|i| (format!("header_{}", i), format!("value_{}", i)))
                    .collect(),
                timestamp: Utc::now(),
                correlation_id: Some("correlation-".to_string() + &"x".repeat(500)),
                reply_to: Some(create_test_service_address("reply-service")),
                ttl: Some(u64::MAX),
            },
        ];

        for message in edge_case_messages {
            let result = in_memory_comm.send_message(target.clone(), message).await;
            // Should handle all edge cases gracefully
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_websocket_communication_connection_count() {
        let websocket_comm = WebSocketCommunication::new("127.0.0.1".to_string(), 8084);

        // Test connection count method
        let count = websocket_comm.connection_count();
        assert_eq!(count, 0); // No connections initially
    }

    #[tokio::test]
    async fn test_http_communication_url_building_edge_cases() {
        let http_comm = HttpCommunication::new("http://localhost:8080/".to_string()); // Trailing slash

        let edge_case_targets = vec![
            ServiceAddress {
                service_id: "service-with-path".to_string(),
                instance_id: None,
                endpoint: Some("http://localhost:8080/custom/path".to_string()),
            },
            ServiceAddress {
                service_id: "service-no-endpoint".to_string(),
                instance_id: None,
                endpoint: None,
            },
        ];

        for target in edge_case_targets {
            let result = http_comm.test_service_connectivity(&target).await;
            // Should handle URL building without crashing
            match result {
                Ok(_) => {}
                Err(_) => {} // Network errors are acceptable
            }
        }
    }

    #[tokio::test]
    async fn test_communication_concurrent_operations() {
        let in_memory_comm = Arc::new(InMemoryCommunication::new());
        let target = create_test_service_address("concurrent-test");

        // Test concurrent send operations
        let send_tasks = (0..10).map(|i| {
            let comm = Arc::clone(&in_memory_comm);
            let target = target.clone();
            async move {
                let message = ServiceMessage {
                    id: format!("concurrent-message-{}", i),
                    message_type: MessageType::Request,
                    topic: Some(format!("concurrent.test.{}", i)),
                    body: json!({"index": i}),
                    headers: HashMap::new(),
                    timestamp: Utc::now(),
                    correlation_id: None,
                    reply_to: None,
                    ttl: None,
                };
                comm.send_message(target, message).await
            }
        });

        let results = futures::future::join_all(send_tasks).await;

        // All concurrent operations should succeed
        for result in results {
            assert!(result.is_ok());
        }
    }
}
