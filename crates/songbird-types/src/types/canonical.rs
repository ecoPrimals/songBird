// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Core Types
//!
//! **CANONICAL**: Fundamental types for the Songbird ecosystem

use crate::errors::SongbirdError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// **CANONICAL**: Endpoint information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CanonicalEndpoint {
    /// Host address
    pub host: String,
    /// Port number
    pub port: u16,
    /// Protocol (http, https, grpc, etc.)
    pub protocol: String,
    /// Optional path
    pub path: Option<String>,
}

impl CanonicalEndpoint {
    /// Create a new endpoint
    #[must_use]
    pub fn new(host: impl Into<String>, port: u16, protocol: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            port,
            protocol: protocol.into(),
            path: None,
        }
    }

    /// Add a path to the endpoint
    #[must_use]
    pub fn with_path(&mut self, path: impl Into<String>) -> &mut Self {
        self.path = Some(path.into());
        self
    }

    /// Get the full URL
    #[must_use]
    pub fn url(&self) -> String {
        let base = format!("{}://{}:{}", self.protocol, self.host, self.port);
        match &self.path {
            Some(path) => format!("{base}{path}"),
            None => base,
        }
    }

    /// Check if the endpoint is available (basic connectivity check)
    #[must_use]
    pub const fn is_available(&self) -> bool {
        // In a real implementation, this would check connectivity
        // For now, just validate that required fields are present
        !self.host.is_empty() && self.port > 0 && !self.protocol.is_empty()
    }
}

impl fmt::Display for CanonicalEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.url())
    }
}

/// **CANONICAL**: Address information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CanonicalAddress {
    /// Street address
    pub street: Option<String>,
    /// City
    pub city: Option<String>,
    /// State/Province
    pub state: Option<String>,
    /// Country
    pub country: Option<String>,
    /// Postal/ZIP code
    pub postal_code: Option<String>,
    /// Address type (e.g., "home", "work", "datacenter")
    pub addr_type: Option<String>,
}

impl CanonicalAddress {
    /// Create a new address
    #[must_use]
    pub fn new(host: impl Into<String>, port: u16, protocol: impl Into<String>) -> Self {
        Self {
            street: Some(format!("{}:{}", host.into(), port)),
            city: None,
            state: None,
            country: None,
            postal_code: None,
            addr_type: Some(protocol.into()),
        }
    }

    /// Set the address type
    #[must_use]
    pub fn with_type(&mut self, addr_type: impl Into<String>) -> &mut Self {
        self.addr_type = Some(addr_type.into());
        self
    }

    /// Set the city
    #[must_use]
    pub fn with_city(&mut self, city: impl Into<String>) -> &mut Self {
        self.city = Some(city.into());
        self
    }

    /// Set the country
    #[must_use]
    pub fn with_country(&mut self, country: impl Into<String>) -> &mut Self {
        self.country = Some(country.into());
        self
    }
}

/// **CANONICAL**: Request wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalRequest {
    /// Request ID for tracing
    pub request_id: String,
    /// Request type/operation
    pub operation: String,
    /// Request payload
    pub payload: serde_json::Value,
    /// Request metadata
    pub metadata: HashMap<String, String>,
    /// Request timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl CanonicalRequest {
    /// Create a new request
    #[must_use]
    pub fn new(operation: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            request_id: uuid::Uuid::new_v4().to_string(),
            operation: operation.into(),
            payload,
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Add metadata to the request
    pub fn with_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// **CANONICAL**: Response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalResponse {
    /// Request ID this response corresponds to
    pub request_id: String,
    /// Response status
    pub status: String,
    /// Response data
    pub data: Option<serde_json::Value>,
    /// Error message if any
    pub error_message: Option<String>,
    /// Response metadata
    pub metadata: HashMap<String, String>,
    /// Response timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl CanonicalResponse {
    /// Create a successful response
    #[must_use]
    pub fn success(request_id: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            request_id: request_id.into(),
            status: "success".to_string(),
            data: Some(data),
            error_message: None,
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Create an error response
    #[must_use]
    pub fn error(request_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            request_id: request_id.into(),
            status: "error".to_string(),
            data: None,
            error_message: Some(error.into()),
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Check if the response indicates success
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.status == "success" && self.error_message.is_none()
    }
}

/// **CANONICAL**: Node type for distributed systems
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CanonicalNodeType {
    /// Tower node (high-performance compute node)
    Tower,
    /// Edge node (lightweight, distributed)
    #[default]
    Edge,
    /// Gateway node (entry point)
    Gateway,
    /// Storage node (data persistence)
    Storage,
    /// Coordinator node (orchestration)
    Coordinator,
}

impl fmt::Display for CanonicalNodeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_str = match self {
            Self::Tower => "Tower",
            Self::Edge => "Edge",
            Self::Gateway => "Gateway",
            Self::Storage => "Storage",
            Self::Coordinator => "Coordinator",
        };
        write!(f, "{type_str}")
    }
}

impl std::str::FromStr for CanonicalNodeType {
    type Err = SongbirdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "tower" => Ok(Self::Tower),
            "edge" => Ok(Self::Edge),
            "gateway" => Ok(Self::Gateway),
            "storage" => Ok(Self::Storage),
            "coordinator" => Ok(Self::Coordinator),
            _ => Err(SongbirdError::Validation {
                message: "Invalid node type".to_string(),
                field: Some("node_type".to_string()),
                suggestion: Some(
                    "Expected Tower, Edge, Gateway, Storage, or Coordinator".to_string(),
                ),
            }),
        }
    }
}

#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::field_reassign_with_default
)]
#[cfg(test)]
#[allow(clippy::uninlined_format_args)]
#[allow(clippy::float_cmp)]
#[allow(clippy::useless_vec)]
#[allow(clippy::unreadable_literal)]
#[allow(clippy::items_after_statements)]
#[allow(clippy::cast_precision_loss)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
mod tests {
    #![allow(clippy::all)]
    #![allow(unused)]

    use super::*;

    #[test]
    fn test_canonical_endpoint() {
        let mut endpoint = CanonicalEndpoint::new("localhost", 8080, "http");
        let _ = endpoint.with_path("/api/v1");

        assert_eq!(endpoint.host, "localhost");
        assert_eq!(endpoint.port, 8080);
        assert_eq!(endpoint.protocol, "http");
        assert_eq!(endpoint.path, Some("/api/v1".to_string()));
        // Test URL construction works correctly
        let expected_url = format!(
            "{}://{}:{}{}",
            endpoint.protocol,
            endpoint.host,
            endpoint.port,
            endpoint.path.as_ref().unwrap_or(&String::new())
        );
        assert_eq!(endpoint.url(), expected_url);
        assert!(endpoint.is_available());
    }

    #[test]
    fn test_canonical_address() {
        let mut address = CanonicalAddress::new("localhost", 8080, "http");
        let _ = address.with_city("San Francisco");
        let _ = address.with_country("USA");
        let _ = address.with_type("datacenter");

        assert_eq!(address.city, Some("San Francisco".to_string()));
        assert_eq!(address.country, Some("USA".to_string()));
        assert_eq!(address.addr_type, Some("datacenter".to_string()));
    }

    #[test]
    fn test_canonical_request() {
        let request = CanonicalRequest::new(
            "health_check".to_string(),
            serde_json::json!({"status": "check"}),
        );

        assert_eq!(request.operation, "health_check");
        assert!(!request.request_id.is_empty());
    }

    #[test]
    fn test_canonical_response() {
        let request_id = uuid::Uuid::new_v4();
        let success_response = CanonicalResponse::success(
            request_id.to_string(),
            serde_json::json!({"status": "healthy"}),
        );

        assert!(success_response.is_success());
        assert_eq!(success_response.status, "success");

        let error_response = CanonicalResponse::error(request_id.to_string(), "Test error");
        assert!(!error_response.is_success());
        assert_eq!(error_response.error_message, Some("Test error".to_string()));
    }

    #[test]
    fn test_canonical_node_type() -> Result<(), Box<dyn std::error::Error>> {
        let node_type = CanonicalNodeType::Tower;
        assert_eq!(node_type.to_string(), "Tower");

        let parsed: Result<CanonicalNodeType, _> = "tower".parse();
        assert!(parsed.is_ok());
        assert_eq!(
            parsed.map_err(|e| SongbirdError::configuration(format!(
                "Test: 'tower' should parse to CanonicalNodeType: {e}"
            )))?,
            CanonicalNodeType::Tower
        );

        let invalid: Result<CanonicalNodeType, _> = "invalid".parse();
        assert!(invalid.is_err());
        Ok(())
    }
}
