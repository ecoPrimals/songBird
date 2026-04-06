// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Unit tests for canonical types

#[cfg(test)]
mod tests {
    #![allow(clippy::uninlined_format_args, reason = "test assertions and harness ergonomics")]
    #![allow(clippy::float_cmp, reason = "test assertions and harness ergonomics")]
    #![allow(clippy::useless_vec, reason = "test assertions and harness ergonomics")]
    #![allow(clippy::unreadable_literal, reason = "test assertions and harness ergonomics")]

    use crate::types::*;

    #[test]
    fn test_service_id_creation() {
        let id = ServiceId::new("test-service");
        assert_eq!(id.as_str(), "test-service");
    }

    #[test]
    fn test_service_id_from_string() {
        let id = ServiceId::from("my-service".to_string());
        assert_eq!(id.as_str(), "my-service");
    }

    #[test]
    fn test_service_id_from_str() {
        let id = ServiceId::from("str-service");
        assert_eq!(id.as_str(), "str-service");
    }

    #[test]
    fn test_service_id_clone() {
        let id1 = ServiceId::new("test");
        let id2 = id1.clone();
        assert_eq!(id1.as_str(), id2.as_str());
    }

    #[test]
    fn test_service_id_equality() {
        let id1 = ServiceId::new("same");
        let id2 = ServiceId::new("same");
        let id3 = ServiceId::new("different");

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_endpoint_creation() {
        let endpoint = Endpoint::new("http", "localhost", 8080);
        assert_eq!(endpoint.protocol, "http");
        assert_eq!(endpoint.host, "localhost");
        assert_eq!(endpoint.port, 8080);
        assert!(endpoint.path.is_none());
    }

    #[test]
    fn test_endpoint_with_path() {
        let endpoint = Endpoint::new("http", "localhost", 8080).with_path("/api/v1");
        assert_eq!(endpoint.path, Some("/api/v1".to_string()));
    }

    #[test]
    fn test_endpoint_to_url_without_path() {
        let endpoint = Endpoint::new("http", "example.com", 8080);
        assert_eq!(endpoint.to_url(), "http://example.com:8080");
    }

    #[test]
    fn test_endpoint_to_url_with_path() {
        let endpoint = Endpoint::new("https", "api.example.com", 443).with_path("api/v1");
        assert_eq!(endpoint.to_url(), "https://api.example.com:443/api/v1");
    }

    #[test]
    fn test_endpoint_clone() {
        let endpoint1 = Endpoint::new("tcp", "localhost", 9000);
        let endpoint2 = endpoint1.clone();
        assert_eq!(endpoint1.protocol, endpoint2.protocol);
        assert_eq!(endpoint1.host, endpoint2.host);
        assert_eq!(endpoint1.port, endpoint2.port);
    }

    #[test]
    fn test_endpoint_equality() {
        let ep1 = Endpoint::new("http", "localhost", 8080);
        let ep2 = Endpoint::new("http", "localhost", 8080);
        let ep3 = Endpoint::new("https", "localhost", 8080);

        assert_eq!(ep1, ep2);
        assert_ne!(ep1, ep3);
    }
}
