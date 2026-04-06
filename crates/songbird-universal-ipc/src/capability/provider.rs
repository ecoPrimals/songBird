// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Provider types for capability-based discovery

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

/// A discovered provider (primal offering capabilities)
///
/// Providers are discovered at runtime based on the capabilities they offer.
/// Applications never hardcode primal names - they discover providers by capability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    /// Provider ID (unique identifier, NOT necessarily a primal name)
    pub id: String,

    /// Capabilities this provider offers
    pub capabilities: Vec<String>,

    /// Virtual endpoint (platform-agnostic)
    ///
    /// Format: `/primal/{id}` or `/provider/{id}`
    pub virtual_endpoint: String,

    /// Metadata about this provider
    pub metadata: ProviderMetadata,

    /// When this provider was discovered
    #[serde(skip, default = "SystemTime::now")]
    pub discovered_at: SystemTime,
}

/// Provider metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetadata {
    /// Provider version (if available)
    pub version: Option<String>,

    /// Provider description
    pub description: Option<String>,

    /// Protocols supported (http, tarpc, json-rpc, grpc, etc.)
    pub protocols: Vec<String>,

    /// Additional custom metadata
    pub custom: HashMap<String, String>,

    /// Health status (if known)
    pub health: HealthStatus,

    /// Discovery method used
    pub discovery_method: String,
}

/// Provider health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthStatus {
    /// Health unknown (not yet checked)
    Unknown,
    /// Provider is healthy
    Healthy,
    /// Provider is degraded (functioning but with issues)
    Degraded,
    /// Provider is unhealthy (not functioning)
    Unhealthy,
}

impl Provider {
    /// Create a new provider
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        capabilities: Vec<String>,
        virtual_endpoint: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            capabilities,
            virtual_endpoint: virtual_endpoint.into(),
            metadata: ProviderMetadata::default(),
            discovered_at: SystemTime::now(),
        }
    }

    /// Check if provider offers a specific capability
    #[must_use]
    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities.iter().any(|c| c == capability)
    }

    /// Check if provider offers all required capabilities
    #[must_use]
    pub fn has_capabilities(&self, required: &[String]) -> bool {
        required.iter().all(|req| self.has_capability(req))
    }

    /// Update health status
    pub const fn update_health(&mut self, status: HealthStatus) {
        self.metadata.health = status;
    }

    /// Check if provider is usable (healthy or degraded)
    #[must_use]
    pub const fn is_usable(&self) -> bool {
        matches!(
            self.metadata.health,
            HealthStatus::Healthy | HealthStatus::Degraded | HealthStatus::Unknown
        )
    }
}

impl Default for ProviderMetadata {
    fn default() -> Self {
        Self {
            version: None,
            description: None,
            protocols: vec!["json-rpc".to_string()], // Default protocol
            custom: HashMap::new(),
            health: HealthStatus::Unknown,
            discovery_method: "unknown".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let provider = Provider::new(
            "test-provider".to_string(),
            vec!["crypto".to_string(), "signing".to_string()],
            "/primal/test-provider".to_string(),
        );

        assert_eq!(provider.id, "test-provider");
        assert_eq!(provider.capabilities.len(), 2);
        assert_eq!(provider.virtual_endpoint, "/primal/test-provider");
    }

    #[test]
    fn test_has_capability() {
        let provider = Provider::new(
            "test".to_string(),
            vec!["crypto".to_string(), "signing".to_string()],
            "/primal/test".to_string(),
        );

        assert!(provider.has_capability("crypto"));
        assert!(provider.has_capability("signing"));
        assert!(!provider.has_capability("storage"));
    }

    #[test]
    fn test_has_capabilities() {
        let provider = Provider::new(
            "test".to_string(),
            vec!["crypto".to_string(), "signing".to_string()],
            "/primal/test".to_string(),
        );

        assert!(provider.has_capabilities(&["crypto".to_string(), "signing".to_string()]));
        assert!(!provider.has_capabilities(&["crypto".to_string(), "storage".to_string()]));
    }

    #[test]
    fn test_health_status() {
        let mut provider = Provider::new(
            "test".to_string(),
            vec!["crypto".to_string()],
            "/primal/test".to_string(),
        );

        assert_eq!(provider.metadata.health, HealthStatus::Unknown);
        assert!(provider.is_usable());

        provider.update_health(HealthStatus::Healthy);
        assert!(provider.is_usable());

        provider.update_health(HealthStatus::Unhealthy);
        assert!(!provider.is_usable());
    }
}
