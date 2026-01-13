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
    pub fn from_static(
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
    /// Parse host from string
    ///
    /// # Errors
    /// This function never returns an error - it always succeeds
    #[allow(clippy::unnecessary_wraps)]
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(host: &str) -> Result<Self, String> {
        match host {
            "localhost" | "127.0.0.1" | "::1" => Ok(Self::Localhost),
            _ => Ok(Self::Custom(host.to_string())),
        }
    }

    /// Get as string reference
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Localhost => "127.0.0.1",
            Self::Custom(host) => host,
        }
    }
}

/// Endpoint protocol enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EndpointProtocol {
    /// HTTP protocol (typically port 80)
    Http,
    /// HTTPS protocol (typically port 443)
    Https,
    /// gRPC protocol
    Tarpc,
    /// Custom protocol
    Custom,
}

impl Default for EndpointProtocol {
    fn default() -> Self {
        Self::Http
    }
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
    pub fn with_security(&mut self) -> &mut Self {
        self.security = true;
        self
    }

    /// Add storage capability
    pub fn with_storage(&mut self) -> &mut Self {
        self.storage = true;
        self
    }

    /// Add compute capability
    pub fn with_compute(&mut self) -> &mut Self {
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
    fn test_optimized_endpoint() {
        let endpoint = OptimizedEndpoint::localhost(8080);
        let url = endpoint.to_url();
        // Test the URL structure matches expected format
        assert!(url.starts_with("http://"));
        assert!(url.contains(":8080"));
    }

    #[test]
    fn test_optimized_capabilities() {
        let mut caps = OptimizedCapabilities::new();
        caps.with_security().with_storage();

        caps.add_custom("custom-capability".to_string());

        assert_eq!(caps.count(), 3);
    }

    #[test]
    fn test_host_optimization() -> Result<(), Box<dyn std::error::Error>> {
        use crate::SongbirdError;
        let localhost = OptimizedHost::from_str("localhost").map_err(|e| {
            SongbirdError::configuration(format!("Test: localhost should parse: {e}"))
        })?;
        assert!(matches!(localhost, OptimizedHost::Localhost));
        assert_eq!(localhost.as_str(), "127.0.0.1");
        Ok(())
    }
}
