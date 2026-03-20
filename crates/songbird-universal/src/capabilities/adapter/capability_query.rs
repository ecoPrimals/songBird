// SPDX-License-Identifier: AGPL-3.0-only
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

        Err(CapabilityError::NetworkError("All capability endpoints failed".to_string()))
    }

    /// Infer basic capabilities from primal name and endpoint
    fn infer_basic_capabilities(endpoint: &str) -> Vec<Capability> {
        let primal_name = Self::extract_primal_name(endpoint);
        let primal_type = Self::infer_primal_type(&primal_name);

        match primal_type {
            PrimalType::Security => vec![
                Capability {
                    capability_type: "authentication".to_string(),
                    name: "auth_service".to_string(),
                    version: "1.0".to_string(),
                    parameters: HashMap::default(),
                    qos_metrics: QoSMetrics::default(),
                    available: true,
                },
                Capability {
                    capability_type: "encryption".to_string(),
                    name: "encryption_service".to_string(),
                    version: "1.0".to_string(),
                    parameters: HashMap::default(),
                    qos_metrics: QoSMetrics::default(),
                    available: true,
                },
            ],
            PrimalType::Compute => vec![Capability {
                capability_type: "compute".to_string(),
                name: "compute_service".to_string(),
                version: "1.0".to_string(),
                parameters: HashMap::default(),
                qos_metrics: QoSMetrics::default(),
                available: true,
            }],
            PrimalType::Storage => vec![Capability {
                capability_type: "storage".to_string(),
                name: "storage_service".to_string(),
                version: "1.0".to_string(),
                parameters: HashMap::default(),
                qos_metrics: QoSMetrics::default(),
                available: true,
            }],
            PrimalType::AI => vec![
                Capability {
                    capability_type: "ai".to_string(),
                    name: "ai_service".to_string(),
                    version: "1.0".to_string(),
                    parameters: HashMap::default(),
                    qos_metrics: QoSMetrics::default(),
                    available: true,
                },
                Capability {
                    capability_type: "ml".to_string(),
                    name: "ml_service".to_string(),
                    version: "1.0".to_string(),
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

    /// Infer primal type from name
    fn infer_primal_type(name: &str) -> PrimalType {
        let name_lower = name.to_lowercase();

        // Capability terms first, known provider names as secondary hints
        if name_lower.contains("security")
            || name_lower.contains("auth")
            || name_lower.contains("beardog")
        {
            PrimalType::Security
        } else if name_lower.contains("compute")
            || name_lower.contains("worker")
            || name_lower.contains("toadstool")
        {
            PrimalType::Compute
        } else if name_lower.contains("storage")
            || name_lower.contains("data")
            || name_lower.contains("nestgate")
        {
            PrimalType::Storage
        } else if name_lower.contains("ai")
            || name_lower.contains("squirrel")
            || name_lower.contains("ml")
        {
            PrimalType::AI
        } else {
            PrimalType::Generic
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_infer_primal_type() {
        assert_eq!(CapabilityQuery::infer_primal_type("beardog"), PrimalType::Security);
        assert_eq!(CapabilityQuery::infer_primal_type("toadstool"), PrimalType::Compute);
        assert_eq!(CapabilityQuery::infer_primal_type("nestgate"), PrimalType::Storage);
        assert_eq!(CapabilityQuery::infer_primal_type("squirrel"), PrimalType::AI);
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
        let caps = CapabilityQuery::infer_basic_capabilities("http://beardog:8443");
        assert_eq!(caps.len(), 2); // auth + encryption
        assert!(caps.iter().any(|c| c.capability_type == "authentication"));
        assert!(caps.iter().any(|c| c.capability_type == "encryption"));
    }
}
