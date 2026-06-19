// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Primal Types and Identifiers
//!
//! **CANONICAL**: Core primal types for the Songbird ecosystem

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// **CANONICAL**: Primal type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CanonicalPrimalType {
    /// Security primal
    Security,
    /// Storage primal
    Storage,
    /// Compute primal
    Compute,
    /// AI primal
    Ai,
    /// Orchestration primal
    Orchestration,
    /// Federation primal
    Federation,
    /// Discovery primal
    Discovery,
    /// Registry primal
    Registry,
    /// Observability primal
    Observability,
    /// Unknown or custom primal type
    Unknown(String),
}

impl Default for CanonicalPrimalType {
    fn default() -> Self {
        Self::Unknown(String::from("default"))
    }
}

impl fmt::Display for CanonicalPrimalType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_str = match self {
            Self::Security => "Security",
            Self::Storage => "Storage",
            Self::Compute => "Compute",
            Self::Ai => "AI",
            Self::Orchestration => "Orchestration",
            Self::Federation => "Federation",
            Self::Discovery => "Discovery",
            Self::Registry => "Registry",
            Self::Observability => "Observability",
            Self::Unknown(custom) => custom,
        };
        write!(f, "{type_str}")
    }
}

/// **CANONICAL**: Primal identifier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalPrimalId {
    /// Primal type
    pub primal_type: CanonicalPrimalType,
    /// Unique instance identifier
    pub instance_id: String,
    /// Version information
    pub version: String,
    /// Endpoints provided by this primal
    pub endpoints: HashMap<String, String>,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

impl Default for CanonicalPrimalId {
    fn default() -> Self {
        Self {
            primal_type: CanonicalPrimalType::default(),
            instance_id: String::from("default-instance"),
            version: String::from("0.1.0"),
            endpoints: HashMap::new(),
            metadata: HashMap::new(),
        }
    }
}

impl CanonicalPrimalId {
    /// Create a new primal ID
    #[must_use]
    pub fn new(
        primal_type: CanonicalPrimalType,
        instance_id: impl Into<String>,
        version: impl Into<String>,
    ) -> Self {
        Self {
            primal_type,
            instance_id: instance_id.into(),
            version: version.into(),
            endpoints: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add an endpoint
    pub fn with_endpoint(&mut self, name: impl Into<String>, url: impl Into<String>) -> &mut Self {
        self.endpoints.insert(name.into(), url.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Get the primal type
    #[must_use]
    pub const fn get_type(&self) -> &CanonicalPrimalType {
        &self.primal_type
    }

    /// Check if this is a security primal
    #[must_use]
    pub const fn is_security(&self) -> bool {
        matches!(self.primal_type, CanonicalPrimalType::Security)
    }

    /// Check if this is a storage primal
    #[must_use]
    pub const fn is_storage(&self) -> bool {
        matches!(self.primal_type, CanonicalPrimalType::Storage)
    }

    /// Check if this is a compute primal
    #[must_use]
    pub const fn is_compute(&self) -> bool {
        matches!(self.primal_type, CanonicalPrimalType::Compute)
    }
}

/// **CANONICAL**: Primal configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalPrimalConfig {
    /// Primal identifier
    pub id: CanonicalPrimalId,
    /// Enabled status
    pub enabled: bool,
    /// Configuration parameters
    pub config: HashMap<String, String>,
    /// Security level
    pub security_level: Option<String>,
}

impl Default for CanonicalPrimalConfig {
    fn default() -> Self {
        Self {
            id: CanonicalPrimalId::default(),
            enabled: true,
            config: HashMap::new(),
            security_level: None,
        }
    }
}

impl CanonicalPrimalConfig {
    /// Create a new primal configuration
    #[must_use]
    pub fn new(id: CanonicalPrimalId) -> Self {
        Self {
            id,
            enabled: true,
            config: HashMap::new(),
            security_level: None,
        }
    }

    /// Set security level
    pub fn with_security_level(&mut self, level: impl Into<String>) -> &mut Self {
        self.security_level = Some(level.into());
        self
    }

    /// Add configuration parameter
    pub fn with_config(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.config.insert(key.into(), value.into());
        self
    }
}

/// **CANONICAL**: Primal response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalPrimalResponse {
    /// Response status
    pub status: String,
    /// Response data
    pub data: Option<String>,
    /// Error message if any
    pub error_message: Option<String>,
    /// Request ID for tracing
    pub request_id: String,
    /// Response metadata
    pub metadata: Option<HashMap<String, String>>,
}

impl CanonicalPrimalResponse {
    /// Create a successful response
    #[must_use]
    pub fn success(request_id: impl Into<String>, data: impl Into<String>) -> Self {
        Self {
            status: String::from("success"),
            data: Some(data.into()),
            error_message: None,
            request_id: request_id.into(),
            metadata: None,
        }
    }

    /// Create an error response
    #[must_use]
    pub fn error(request_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            status: String::from("error"),
            data: None,
            error_message: Some(error.into()),
            request_id: request_id.into(),
            metadata: None,
        }
    }

    /// Create a service unavailable response
    #[must_use]
    pub fn service_unavailable(
        request_id: impl Into<String>,
        primal_id: impl Into<String>,
    ) -> Self {
        let mut metadata = HashMap::new();
        metadata.insert(String::from("primal_id"), primal_id.into());
        metadata.insert(String::from("error_type"), String::from("service_unavailable"));

        Self {
            status: String::from("service_unavailable"),
            data: None,
            error_message: Some(String::from("Service is currently unavailable")),
            request_id: request_id.into(),
            metadata: Some(metadata),
        }
    }

    /// Check if the response is successful
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.status == "success"
    }

    /// Check if the response is an error
    #[must_use]
    pub fn is_error(&self) -> bool {
        self.status == "error" || self.error_message.is_some()
    }
}

#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::field_reassign_with_default,
    reason = "intentional pattern; clippy false positive for this API"
)]
#[cfg(test)]
#[expect(clippy::uninlined_format_args, reason = "test assertions and harness ergonomics")]
#[expect(clippy::float_cmp, reason = "test assertions and harness ergonomics")]
#[expect(clippy::useless_vec, reason = "test assertions and harness ergonomics")]
#[expect(clippy::unreadable_literal, reason = "test assertions and harness ergonomics")]
#[expect(clippy::items_after_statements, reason = "test assertions and harness ergonomics")]
#[expect(clippy::cast_precision_loss, reason = "test assertions and harness ergonomics")]
#[allow(
    clippy::cast_possible_truncation,
    reason = "intentional pattern; clippy false positive for this API"
)]
#[expect(
    clippy::cast_sign_loss,
    reason = "intentional pattern; clippy false positive for this API"
)]
mod tests {
    #![allow(clippy::all, reason = "test assertions and harness ergonomics")]
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
    #![allow(unused, reason = "test assertions and harness ergonomics")]

    use super::*;

    #[test]
    fn test_primal_type_display() {
        assert_eq!(CanonicalPrimalType::Security.to_string(), "Security");
        assert_eq!(CanonicalPrimalType::Storage.to_string(), "Storage");
        assert_eq!(CanonicalPrimalType::Unknown(String::from("custom")).to_string(), "custom");
    }

    #[test]
    fn test_primal_id_creation() {
        let test_host =
            songbird_process_env::var("TEST_HOST").unwrap_or_else(|_| String::from("localhost"));
        let test_port = songbird_process_env::var("TEST_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);

        let mut id = CanonicalPrimalId::new(CanonicalPrimalType::Security, "security-001", "1.0.0");
        id.with_endpoint("health", format!("http://{test_host}:{test_port}/health"));
        id.with_metadata("region", "us-west-2");

        assert!(id.is_security());
        assert_eq!(id.instance_id, "security-001");
        assert_eq!(id.endpoints.len(), 1);
        assert_eq!(id.metadata.len(), 1);
    }

    #[test]
    fn test_primal_response() {
        let success = CanonicalPrimalResponse::success("data", "req-123");
        assert!(success.is_success());
        assert!(!success.is_error());

        let error = CanonicalPrimalResponse::error("Something went wrong", "req-124");
        assert!(!error.is_success());
        assert!(error.is_error());

        let unavailable = CanonicalPrimalResponse::service_unavailable(
            String::from("primal-001"),
            String::from("req-125"),
        );
        assert!(!unavailable.is_success());
        assert!(unavailable.is_error());
    }

    #[test]
    fn test_primal_config() {
        let id = CanonicalPrimalId::new(CanonicalPrimalType::Compute, "compute-001", "1.0.0");
        let mut config = CanonicalPrimalConfig::new(id);
        config.with_security_level("high");
        config.with_config("max_workers", "10");

        assert_eq!(config.security_level, Some(String::from("high")));
        assert_eq!(config.config.get("max_workers"), Some(&String::from("10")));
    }

    #[test]
    fn canonical_primal_type_unknown_variant_roundtrip() {
        let t = CanonicalPrimalType::Unknown("custom-x".into());
        let v = serde_json::to_string(&t).expect("ser");
        let back: CanonicalPrimalType = serde_json::from_str(&v).expect("de");
        assert_eq!(t, back);
    }

    #[test]
    fn canonical_primal_id_serde_roundtrip() {
        let mut id = CanonicalPrimalId::new(CanonicalPrimalType::Discovery, "d1", "2.0.0");
        id.with_endpoint("api", "http://127.0.0.1:1");
        let s = serde_json::to_string(&id).expect("ser");
        let back: CanonicalPrimalId = serde_json::from_str(&s).expect("de");
        assert_eq!(back.instance_id, "d1");
        assert_eq!(back.endpoints.get("api").map(String::as_str), Some("http://127.0.0.1:1"));
    }

    #[test]
    fn primal_id_get_type_and_flags() {
        let storage = CanonicalPrimalId::new(CanonicalPrimalType::Storage, "s", "1");
        assert!(storage.is_storage());
        assert!(!storage.is_security());

        let compute = CanonicalPrimalId::new(CanonicalPrimalType::Compute, "c", "1");
        assert!(compute.is_compute());
    }

    #[test]
    fn primal_response_service_unavailable_includes_metadata() {
        let r = CanonicalPrimalResponse::service_unavailable("req-9", "pid-1");
        assert_eq!(r.status, "service_unavailable");
        let meta = r.metadata.expect("meta");
        assert_eq!(meta.get("primal_id").map(String::as_str), Some("pid-1"));
    }

    #[test]
    fn primal_config_default_enabled() {
        let c = CanonicalPrimalConfig::default();
        assert!(c.enabled);
    }

    #[test]
    fn canonical_primal_type_default_is_unknown_default() {
        let d = CanonicalPrimalType::default();
        assert_eq!(d.to_string(), "default");
    }

    #[test]
    fn primal_response_error_is_error_and_not_success() {
        let r = CanonicalPrimalResponse::error("rid", "oops");
        assert!(r.is_error());
        assert!(!r.is_success());
    }

    #[test]
    fn primal_id_default_constructor() {
        let d = CanonicalPrimalId::default();
        assert_eq!(d.instance_id, "default-instance");
    }
}
