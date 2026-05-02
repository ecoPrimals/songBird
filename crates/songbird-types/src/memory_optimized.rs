// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Memory-Optimized Types
//!
//! **CANONICAL**: Zero-copy and memory-efficient types for high-performance scenarios

use crate::health::CanonicalHealthStatus;
use crate::primal::CanonicalPrimalType;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// Memory-optimized primal identifier with zero-copy string handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizedPrimalId<'a> {
    /// Primal type
    pub primal_type: CanonicalPrimalType,
    /// Instance identifier (zero-copy when possible)
    pub instance_id: Cow<'a, str>,
    /// Health status
    pub health_status: CanonicalHealthStatus,
}

impl<'a> OptimizedPrimalId<'a> {
    /// Create a new optimized primal ID with zero-copy string
    #[must_use]
    pub fn new(
        primal_type: CanonicalPrimalType,
        instance_id: impl Into<Cow<'a, str>>,
        health_status: CanonicalHealthStatus,
    ) -> Self {
        Self {
            primal_type,
            instance_id: instance_id.into(),
            health_status,
        }
    }

    /// Create from static string (zero allocation)
    #[must_use]
    pub const fn from_static(
        primal_type: CanonicalPrimalType,
        instance_id: &'static str,
        health_status: CanonicalHealthStatus,
    ) -> OptimizedPrimalId<'static> {
        OptimizedPrimalId {
            primal_type,
            instance_id: Cow::Borrowed(instance_id),
            health_status,
        }
    }

    /// Convert to owned version (allocates if needed)
    #[must_use]
    pub fn to_owned(&self) -> OptimizedPrimalId<'static> {
        OptimizedPrimalId {
            primal_type: self.primal_type.clone(),
            instance_id: Cow::Owned(self.instance_id.clone().into_owned()),
            health_status: self.health_status,
        }
    }

    /// Get the primal type
    #[must_use]
    pub const fn category(&self) -> &CanonicalPrimalType {
        &self.primal_type
    }

    /// Check if the primal is healthy
    #[must_use]
    pub const fn is_healthy(&self) -> bool {
        matches!(self.health_status, CanonicalHealthStatus::Healthy)
    }
}

/// Memory-optimized host representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizedHost {
    /// Localhost (zero allocation - uses canonical constant)
    Localhost,
    /// Custom host (allocated)
    Custom(String),
}

impl OptimizedHost {
    /// Parse from string with optimization for localhost
    #[allow(clippy::should_implement_trait, reason = "custom from_str name; not std::str::FromStr")]
    #[must_use]
    pub fn from_str(host: &str) -> Self {
        if crate::constants::is_loopback_host(host) {
            Self::Localhost
        } else {
            Self::Custom(host.to_string())
        }
    }

    /// Get as string reference
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Localhost => crate::constants::LOCALHOST,
            Self::Custom(host) => host,
        }
    }
}

/// Endpoint protocol enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum EndpointProtocol {
    /// HTTP protocol (typically port 80)
    #[default]
    Http,
    /// HTTPS protocol (typically port 443)
    Https,
    /// gRPC protocol
    Tarpc,
    /// Custom protocol
    Custom,
}

/// Memory-optimized endpoint representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizedEndpoint {
    /// Host
    pub host: OptimizedHost,
    /// Port
    pub port: u16,
    /// Protocol
    pub protocol: EndpointProtocol,
}

impl OptimizedEndpoint {
    /// Create a new optimized endpoint
    #[must_use]
    pub fn new(host: impl Into<String>, port: u16, _protocol: impl Into<String>) -> Self {
        Self {
            host: OptimizedHost::Custom(host.into()),
            port,
            protocol: EndpointProtocol::Custom,
        }
    }

    /// Create localhost endpoint (zero allocation)
    #[must_use]
    pub const fn localhost(port: u16) -> Self {
        Self {
            host: OptimizedHost::Localhost,
            port,
            protocol: EndpointProtocol::Http,
        }
    }
    /// Get URL string
    #[must_use]
    pub fn to_url(&self) -> String {
        let protocol = match self.protocol {
            EndpointProtocol::Http => "http",
            EndpointProtocol::Https => "https",
            EndpointProtocol::Tarpc => "tarpc",
            EndpointProtocol::Custom => "custom",
        };
        format!("{}://{}:{}", protocol, self.host.as_str(), self.port)
    }

    // Memory-optimized capabilities with bitflags
}

/// Compact capability flags for advertising what a primal or service supports.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OptimizedCapabilities {
    /// Security capability
    pub security: bool,
    /// Storage capability
    pub storage: bool,
    /// Compute capability
    pub compute: bool,
    /// Custom capabilities
    pub custom: Vec<String>,
}

impl OptimizedCapabilities {
    /// Create new empty capability set
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add security capability
    pub const fn with_security(&mut self) -> &mut Self {
        self.security = true;
        self
    }

    /// Add storage capability
    pub const fn with_storage(&mut self) -> &mut Self {
        self.storage = true;
        self
    }

    /// Add compute capability
    pub const fn with_compute(&mut self) -> &mut Self {
        self.compute = true;
        self
    }

    /// Add custom capability
    pub fn add_custom(&mut self, capability: impl Into<String>) -> &mut Self {
        self.custom.push(capability.into());
        self
    }

    /// Get capability count
    #[must_use]
    pub fn count(&self) -> usize {
        let base_count = [self.security, self.storage, self.compute].iter().filter(|&&x| x).count();
        base_count + self.custom.len()
    }

    /// Convert to string vector
    #[must_use]
    pub fn to_string_vec(&self) -> Vec<String> {
        let mut capabilities = Vec::new();

        if self.security {
            capabilities.push("security".to_string());
        }
        if self.storage {
            capabilities.push("storage".to_string());
        }
        if self.compute {
            capabilities.push("compute".to_string());
        }

        capabilities.extend(self.custom.clone());
        capabilities
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::unnecessary_wraps,
    clippy::field_reassign_with_default,
    clippy::uninlined_format_args,
    clippy::float_cmp,
    clippy::useless_vec,
    clippy::unreadable_literal,
    clippy::items_after_statements,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "test assertions and harness ergonomics"
)]
mod tests {
    #![allow(clippy::expect_used, reason = "test assertions")]
    #![allow(clippy::all, reason = "test assertions and harness ergonomics")]
    #![allow(unused, reason = "unused bindings/imports in this compilation unit")]

    use super::*;

    #[test]
    fn test_optimized_primal_id() {
        let id = OptimizedPrimalId::from_static(
            CanonicalPrimalType::Security,
            "security-001",
            CanonicalHealthStatus::Healthy,
        );

        assert!(id.is_healthy());
        assert_eq!(*id.category(), CanonicalPrimalType::Security);
    }

    #[test]
    fn test_optimized_primal_id_to_owned() {
        let id = OptimizedPrimalId::new(
            CanonicalPrimalType::Compute,
            "c-1",
            CanonicalHealthStatus::Degraded,
        );
        let owned = id.to_owned();
        assert_eq!(owned.instance_id.as_ref(), "c-1");
        assert!(!owned.is_healthy());
    }

    #[test]
    fn test_optimized_endpoint() {
        let endpoint = OptimizedEndpoint::localhost(8080);
        let url = endpoint.to_url();
        // Test the URL structure matches expected format
        assert!(url.starts_with("http://"));
        assert!(url.contains(":8080"));
    }

    #[test]
    fn test_optimized_endpoint_custom_protocol_url() {
        let mut ep = OptimizedEndpoint::new("example.com", 443, "https");
        ep.protocol = EndpointProtocol::Https;
        assert!(ep.to_url().starts_with("https://"));
        assert!(ep.to_url().contains("example.com:443"));
    }

    #[test]
    fn test_optimized_capabilities() {
        let mut caps = OptimizedCapabilities::new();
        caps.with_security().with_storage();

        caps.add_custom("custom-capability".to_string());

        assert_eq!(caps.count(), 3);
    }

    #[test]
    fn test_optimized_capabilities_to_string_vec_order() {
        let mut caps = OptimizedCapabilities::new();
        caps.with_compute();
        caps.add_custom("x");
        let v = caps.to_string_vec();
        assert!(v.contains(&"compute".to_string()));
        assert!(v.contains(&"x".to_string()));
    }

    #[test]
    fn test_optimized_capabilities_count_all_flags() {
        let mut caps = OptimizedCapabilities::new();
        caps.with_security().with_storage().with_compute();
        caps.add_custom("extra");
        assert_eq!(caps.count(), 4);
    }

    #[test]
    fn test_host_optimization() {
        let localhost = OptimizedHost::from_str("localhost");
        assert!(matches!(localhost, OptimizedHost::Localhost));
        assert_eq!(localhost.as_str(), "127.0.0.1");

        let custom = OptimizedHost::from_str("other.example");
        assert_eq!(custom.as_str(), "other.example");
    }
}
