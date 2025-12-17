//! Tests for communication configuration types

#[allow(clippy::unwrap_used, clippy::expect_used, clippy::unnecessary_wraps, clippy::field_reassign_with_default)]
#[cfg(test)]
mod tests {
    use super::super::communication::*;
    use std::time::Duration;

    #[test]
    fn test_canonical_communication_config_default() {
        let config = CanonicalCommunicationConfig::default();
        assert_eq!(config.http.timeout, Duration::from_secs(30));
        assert_eq!(config.http.max_connections_per_host, 10);
        assert!(config.http.http2_enabled);
    }

    #[test]
    fn test_http_client_config_default() {
        let config = HttpClientConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.max_connections_per_host, 10);
        assert!(config.http2_enabled);
        assert_eq!(config.user_agent, "songbird/1.0");
        assert!(config.default_headers.is_empty());
        assert!(config.compression_enabled);
        assert_eq!(config.keep_alive_timeout, Duration::from_secs(60));
        assert_eq!(config.max_redirects, 10);
    }

    #[test]
    fn test_http_client_config_clone() {
        let config = HttpClientConfig::default();
        let cloned = config.clone();
        assert_eq!(config.timeout, cloned.timeout);
        assert_eq!(config.max_connections_per_host, cloned.max_connections_per_host);
    }

    #[test]
    fn test_http_client_config_debug() {
        let config = HttpClientConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("HttpClientConfig"));
    }

    #[test]
    fn test_http_client_config_serialization() {
        let config = HttpClientConfig::default();
        let json = serde_json::to_string(&config).expect("Should serialize");
        assert!(json.contains("songbird"));
    }

    #[test]
    fn test_http_client_config_deserialization() {
        let config = HttpClientConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: HttpClientConfig =
            serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(deserialized.timeout, config.timeout);
    }

    #[test]
    fn test_http_client_config_custom_values() {
        let mut config = HttpClientConfig::default();
        config.timeout = Duration::from_secs(60);
        config.max_connections_per_host = 20;
        config.http2_enabled = false;

        assert_eq!(config.timeout, Duration::from_secs(60));
        assert_eq!(config.max_connections_per_host, 20);
        assert!(!config.http2_enabled);
    }

    #[test]
    fn test_grpc_config_default() {
        let config = GrpcConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert!(config.max_message_size > 0);
        assert_eq!(config.keep_alive_interval, Duration::from_secs(30));
        assert_eq!(config.keep_alive_timeout, Duration::from_secs(5));
        assert!(config.tls_enabled);
        assert_eq!(config.max_concurrent_streams, 100);
    }

    #[test]
    fn test_websocket_config_default() {
        let config = WebSocketConfig::default();
        assert_eq!(config.ping_interval, Duration::from_secs(30));
        assert_eq!(config.pong_timeout, Duration::from_secs(10));
        assert!(config.max_frame_size > 0);
        assert!(config.max_message_size > 0);
    }

    #[test]
    fn test_json_rpc_config_default() {
        let config = JsonRpcConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.max_batch_size, 100);
        assert!(config.batch_enabled);
        assert_eq!(config.version, "2.0");
    }

    #[test]
    fn test_performance_config_default() {
        let config = PerformanceConfig::default();
        assert!(config.enabled);
        // connection_pooling, request_batching, and caching are complex types - just verify creation works
    }

    #[test]
    fn test_circuit_breaker_config_default() {
        let config = CircuitBreakerConfig::default();
        assert_eq!(config.failure_threshold, 5);
        assert_eq!(config.timeout, Duration::from_secs(60));
        assert!(config.enabled);
    }

    #[test]
    fn test_canonical_communication_config_serialization() {
        let config = CanonicalCommunicationConfig::default();
        let json = serde_json::to_string(&config).expect("Should serialize");
        let deserialized: CanonicalCommunicationConfig =
            serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(deserialized.http.timeout, config.http.timeout);
    }

    #[test]
    fn test_http_client_config_with_headers() {
        let mut config = HttpClientConfig::default();
        config.default_headers.insert("Authorization".to_string(), "Bearer token".to_string());
        config.default_headers.insert("Content-Type".to_string(), "application/json".to_string());

        assert_eq!(config.default_headers.len(), 2);
        assert_eq!(config.default_headers.get("Authorization"), Some(&"Bearer token".to_string()));
    }

    #[test]
    fn test_grpc_config_clone() {
        let config = GrpcConfig::default();
        let cloned = config.clone();
        assert_eq!(config.timeout, cloned.timeout);
    }

    #[test]
    fn test_websocket_config_clone() {
        let config = WebSocketConfig::default();
        let cloned = config.clone();
        assert_eq!(config.ping_interval, cloned.ping_interval);
    }

    #[test]
    fn test_json_rpc_config_clone() {
        let config = JsonRpcConfig::default();
        let cloned = config.clone();
        assert_eq!(config.timeout, cloned.timeout);
    }

    #[test]
    fn test_performance_config_clone() {
        let config = PerformanceConfig::default();
        let cloned = config.clone();
        assert_eq!(config.enabled, cloned.enabled);
        // Complex fields verified by successful clone operation
    }

    #[test]
    fn test_circuit_breaker_config_clone() {
        let config = CircuitBreakerConfig::default();
        let cloned = config.clone();
        assert_eq!(config.failure_threshold, cloned.failure_threshold);
    }
}
