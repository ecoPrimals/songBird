// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Capability Query Module
//!
//! Handles querying primals for their capabilities and selecting best providers.
//! Includes HTTP client for capability queries and QoS-based selection.
//!
//! Part of the smart refactoring from monolithic adapter.rs

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

use super::super::error::CapabilityError;
use super::super::registry::CapabilityRegistry;
use super::super::types::{Capability, PrimalType, QoSMetrics};

/// Capability query component
#[derive(Debug, Clone)]
pub struct CapabilityQuery {
    /// Shared capability registry
    registry: Arc<RwLock<CapabilityRegistry>>,
}

impl CapabilityQuery {
    /// Create new capability query component
    pub const fn new(registry: Arc<RwLock<CapabilityRegistry>>) -> Self {
        Self {
            registry,
        }
    }

    /// Check if a primal provides a specific capability
    pub async fn check_primal_provides_capability(
        &self,
        primal_name: &str,
        capability_type: &str,
    ) -> bool {
        debug!("🔍 Checking if {} provides {}", primal_name, capability_type);

        let registry = self.registry.read().await;

        // Check if we have cached capabilities for this primal
        if let Some(capabilities) = registry.primal_capabilities.get(primal_name) {
            let provides = capabilities.iter().any(|cap| cap.capability_type == capability_type);

            if provides {
                debug!("✅ {} provides {}", primal_name, capability_type);
            } else {
                debug!("❌ {} does not provide {}", primal_name, capability_type);
            }

            return provides;
        }

        // No cached data - might need discovery first
        debug!("ℹ️  No capability data for {} - discovery needed", primal_name);
        false
    }

    /// Get the best primal for a capability based on `QoS` metrics
    ///
    /// Uses `QoS`-aware selection considering health, latency, load, and availability.
    /// Falls back to simple selection if `QoS` metrics are not available.
    pub async fn get_best_primal_for_capability(&self, capability_type: &str) -> Option<String> {
        debug!("🎯 Finding best primal for capability: {}", capability_type);

        let registry = self.registry.read().await;

        // Get all providers for this capability
        let providers =
            registry.capability_providers.get(capability_type).cloned().unwrap_or_default();

        if providers.is_empty() {
            debug!("❌ No providers found for capability: {}", capability_type);
            return None;
        }

        // ✨ Enhanced: QoS-aware selection
        // If QoS metrics are available, use intelligent selection
        // Otherwise fall back to first-available
        let best = if let Some(qos_selector) = registry.qos_selector.as_ref() {
            debug!("🎯 Using QoS-aware selection for {} providers", providers.len());
            qos_selector.select_best_provider(&providers).await
        } else {
            debug!("ℹ️  QoS selector not available, using first provider");
            providers.first().cloned()
        };

        if let Some(ref primal) = best {
            info!("✅ Selected {} for capability {}", primal, capability_type);
        }

        best
    }

    /// Query primal capabilities via HTTP
    pub async fn query_primal_capabilities(
        &self,
        endpoint: &str,
    ) -> Result<Vec<Capability>, CapabilityError> {
        debug!("📡 Querying capabilities from: {}", endpoint);

        // Try HTTP query
        match Self::http_query_capabilities(endpoint).await {
            Ok(capabilities) => {
                info!("✅ Got {} capabilities from {}", capabilities.len(), endpoint);
                Ok(capabilities)
            }
            Err(e) => {
                debug!("❌ HTTP query failed, trying inference: {}", e);
                // Fallback to inference
                Ok(Self::infer_basic_capabilities(endpoint))
            }
        }
    }

    /// HTTP query for capabilities
    async fn http_query_capabilities(endpoint: &str) -> Result<Vec<Capability>, CapabilityError> {
        // Create HTTP client
        let client = songbird_http_client::IpcHttpClient::new()
            .await
            .map_err(|e| CapabilityError::NetworkError(format!("HTTP client error: {e}")))?;

        // Try different capability endpoint patterns
        let capability_endpoints = [
            format!("{endpoint}/capabilities"),
            format!("{endpoint}/api/capabilities"),
            format!("{endpoint}/api/v1/capabilities"),
        ];

        for cap_endpoint in &capability_endpoints {
            match client.get(cap_endpoint).await {
                Ok(response) if response.is_success() => {
                    match response.json::<Vec<Capability>>().await {
                        Ok(capabilities) => return Ok(capabilities),
                        Err(e) => {
                            debug!("Failed to parse capabilities from {}: {}", cap_endpoint, e);
                        }
                    }
                }
                Ok(response) => {
                    debug!("Non-success status from {}: {}", cap_endpoint, response.status());
                }
                Err(e) => {
                    debug!("Failed to query {}: {}", cap_endpoint, e);
                }
            }
        }

        Err(CapabilityError::NetworkError(String::from("All capability endpoints failed")))
    }

    /// Infer basic capabilities from primal name and endpoint
    fn infer_basic_capabilities(endpoint: &str) -> Vec<Capability> {
        let primal_name = Self::extract_primal_name(endpoint);
        let primal_type = Self::infer_primal_type(&primal_name);

        match primal_type {
            PrimalType::Security => vec![
                Capability {
                    capability_type: String::from("authentication"),
                    name: String::from("auth_service"),
                    version: String::from("1.0"),
                    parameters: HashMap::default(),
                    qos_metrics: QoSMetrics::default(),
                    available: true,
                },
                Capability {
                    capability_type: String::from("encryption"),
                    name: String::from("encryption_service"),
                    version: String::from("1.0"),
                    parameters: HashMap::default(),
                    qos_metrics: QoSMetrics::default(),
                    available: true,
                },
            ],
            PrimalType::Compute => vec![Capability {
                capability_type: String::from("compute"),
                name: String::from("compute_service"),
                version: String::from("1.0"),
                parameters: HashMap::default(),
                qos_metrics: QoSMetrics::default(),
                available: true,
            }],
            PrimalType::Storage => vec![Capability {
                capability_type: String::from("storage"),
                name: String::from("storage_service"),
                version: String::from("1.0"),
                parameters: HashMap::default(),
                qos_metrics: QoSMetrics::default(),
                available: true,
            }],
            PrimalType::AI => vec![
                Capability {
                    capability_type: String::from("ai"),
                    name: String::from("ai_service"),
                    version: String::from("1.0"),
                    parameters: HashMap::default(),
                    qos_metrics: QoSMetrics::default(),
                    available: true,
                },
                Capability {
                    capability_type: String::from("ml"),
                    name: String::from("ml_service"),
                    version: String::from("1.0"),
                    parameters: HashMap::default(),
                    qos_metrics: QoSMetrics::default(),
                    available: true,
                },
            ],
            PrimalType::Generic | PrimalType::Discovery | PrimalType::Orchestration => vec![],
        }
    }

    /// Extract primal name from endpoint URL
    fn extract_primal_name(endpoint: &str) -> String {
        endpoint
            .trim_start_matches("http://")
            .trim_start_matches("https://")
            .split(':')
            .next()
            .unwrap_or("unknown")
            .split('.')
            .next()
            .unwrap_or("unknown")
            .to_string()
    }

    /// Infer primal type from name using only capability terms.
    ///
    /// Primal-agnostic: matches on domain terminology rather than specific
    /// primal names. Provider identities are discovered at runtime.
    fn infer_primal_type(name: &str) -> PrimalType {
        let name_lower = name.to_lowercase();

        if name_lower.contains("security")
            || name_lower.contains("auth")
            || name_lower.contains("crypto")
        {
            PrimalType::Security
        } else if name_lower.contains("compute")
            || name_lower.contains("worker")
            || name_lower.contains("exec")
        {
            PrimalType::Compute
        } else if name_lower.contains("storage")
            || name_lower.contains("data")
            || name_lower.contains("persist")
        {
            PrimalType::Storage
        } else if name_lower.contains("ai")
            || name_lower.contains("ml")
            || name_lower.contains("inference")
        {
            PrimalType::AI
        } else {
            PrimalType::Generic
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, reason = "test assertions")]

    use std::sync::Arc;

    use tokio::sync::RwLock;

    use super::*;

    #[tokio::test]
    async fn test_infer_primal_type() {
        assert_eq!(CapabilityQuery::infer_primal_type("security-provider"), PrimalType::Security);
        assert_eq!(CapabilityQuery::infer_primal_type("auth-service"), PrimalType::Security);
        assert_eq!(CapabilityQuery::infer_primal_type("compute-worker"), PrimalType::Compute);
        assert_eq!(CapabilityQuery::infer_primal_type("data-storage"), PrimalType::Storage);
        assert_eq!(CapabilityQuery::infer_primal_type("ml-inference"), PrimalType::AI);
        assert_eq!(CapabilityQuery::infer_primal_type("unknown-service"), PrimalType::Generic);
    }

    #[tokio::test]
    async fn test_extract_primal_name() {
        assert_eq!(CapabilityQuery::extract_primal_name("http://beardog:8443"), "beardog");
        assert_eq!(
            CapabilityQuery::extract_primal_name("https://toadstool.local:9000"),
            "toadstool"
        );
    }

    #[tokio::test]
    async fn test_infer_basic_capabilities() {
        let caps = CapabilityQuery::infer_basic_capabilities("http://security-provider:8443");
        assert_eq!(caps.len(), 2);
        assert!(caps.iter().any(|c| c.capability_type == "authentication"));
        assert!(caps.iter().any(|c| c.capability_type == "encryption"));

        let caps = CapabilityQuery::infer_basic_capabilities("http://compute-worker:9000");
        assert_eq!(caps.len(), 1);
        assert!(caps.iter().any(|c| c.capability_type == "compute"));
    }

    #[tokio::test]
    async fn test_infer_basic_capabilities_storage_and_ai() {
        let storage_caps =
            CapabilityQuery::infer_basic_capabilities("http://data-persist-service:8443");
        assert_eq!(storage_caps.len(), 1);
        assert_eq!(storage_caps[0].capability_type, "storage");

        let ai_caps = CapabilityQuery::infer_basic_capabilities("http://ml-inference:9000");
        assert_eq!(ai_caps.len(), 2);
        assert!(ai_caps.iter().any(|c| c.capability_type == "ai"));
        assert!(ai_caps.iter().any(|c| c.capability_type == "ml"));
    }

    #[tokio::test]
    async fn test_infer_basic_capabilities_generic_empty() {
        let caps = CapabilityQuery::infer_basic_capabilities("http://unknown-host:8080");
        assert!(caps.is_empty());
    }

    #[tokio::test]
    async fn test_extract_primal_name_localhost_only() {
        assert_eq!(CapabilityQuery::extract_primal_name("http://localhost"), "localhost");
    }

    #[tokio::test]
    async fn test_check_primal_provides_capability() {
        let mut registry = CapabilityRegistry::default();
        registry.primal_capabilities.insert(
            String::from("p1"),
            vec![Capability {
                capability_type: String::from("storage"),
                name: String::from("blob"),
                version: String::from("1"),
                parameters: Default::default(),
                qos_metrics: QoSMetrics::default(),
                available: true,
            }],
        );
        let registry = Arc::new(RwLock::new(registry));
        let q = CapabilityQuery::new(registry);

        assert!(q.check_primal_provides_capability("p1", "storage").await);
        assert!(!q.check_primal_provides_capability("p1", "compute").await);
        assert!(!q.check_primal_provides_capability("missing", "storage").await);
    }

    #[tokio::test]
    async fn test_get_best_primal_no_providers() {
        let registry = Arc::new(RwLock::new(CapabilityRegistry::default()));
        let q = CapabilityQuery::new(registry);
        assert!(q.get_best_primal_for_capability("anything").await.is_none());
    }

    #[tokio::test]
    async fn test_get_best_primal_first_without_qos_selector() {
        let mut reg = CapabilityRegistry::default();
        reg.capability_providers
            .insert(String::from("compute"), vec![String::from("alpha"), String::from("beta")]);
        reg.qos_selector = None;
        let registry = Arc::new(RwLock::new(reg));
        let q = CapabilityQuery::new(registry);
        assert_eq!(q.get_best_primal_for_capability("compute").await.as_deref(), Some("alpha"));
    }
}
