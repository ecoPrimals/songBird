// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Genesis Coordination Bridge
//!
//! Integrates genesis ceremony with the agnostic primal coordination system.
//! **ZERO HARDCODING**: No primal names, only capability requests.

use crate::error::{GenesisError, Result};

// NOTE: Conditional compilation for the coordination feature
// This allows genesis to work with or without the coordination crate
#[cfg(feature = "coordination")]
use std::sync::Arc;

#[cfg(feature = "coordination")]
use songbird_primal_coordination::{
    CapabilityType, PrimalCoordinator,
    types::{Identity, NodeId},
};

/// Genesis coordination using capability-based discovery
///
/// **EVOLUTION**: Replaces hardcoded `security provider` connections with capability-based security
#[cfg(feature = "coordination")]
pub struct GenesisCoordinationBridge {
    coordinator: Arc<PrimalCoordinator>,
}

#[cfg(feature = "coordination")]
impl GenesisCoordinationBridge {
    /// Create a new genesis coordination bridge
    #[must_use]
    pub fn new(coordinator: Arc<PrimalCoordinator>) -> Self {
        tracing::info!("🌱 Genesis: Using capability-based coordination (zero hardcoded primals)");
        Self {
            coordinator,
        }
    }

    /// Execute genesis ceremony using capability discovery
    ///
    /// # Errors
    ///
    /// Returns an error if coordination fails
    pub async fn execute_genesis(&self, node_id_str: String) -> Result<Identity> {
        tracing::info!("🌱 Genesis: Executing ceremony with capability-based coordination");

        // Convert string to NodeId
        let node_id = NodeId(node_id_str.clone());

        // Use coordinator to execute genesis
        let identity: Identity = self.coordinator.coordinate_genesis(node_id).await.map_err(
            |e: songbird_primal_coordination::PrimalCoordinationError| {
                GenesisError::CoordinationFailed(e.to_string())
            },
        )?;

        // Return the coordination Identity directly
        Ok(identity)
    }

    /// Request security capability for genesis operations
    ///
    /// # Errors
    ///
    /// Returns an error if no security capability is available
    pub async fn request_security_capability(&self) -> Result<()> {
        self.coordinator.request_capability(CapabilityType::Security).await.map_err(
            |e: songbird_primal_coordination::PrimalCoordinationError| {
                GenesisError::CoordinationFailed(format!("Security capability not available: {e}"))
            },
        )?;
        Ok(())
    }
}

/// Fallback genesis (no coordination)
///
/// Used when coordination feature is not enabled
#[cfg(not(feature = "coordination"))]
pub struct GenesisCoordinationBridge;

#[cfg(not(feature = "coordination"))]
impl GenesisCoordinationBridge {
    /// Create a new genesis coordination bridge (fallback)
    #[must_use]
    pub fn new_fallback() -> Self {
        tracing::warn!("🌱 Genesis: Coordination feature not enabled, using fallback mode");
        Self
    }

    /// Execute genesis ceremony (fallback mode)
    ///
    /// # Errors
    ///
    /// Returns an error indicating coordination is not available
    #[expect(
        clippy::unused_async,
        reason = "async signature for API compatibility with coordination-enabled builds"
    )]
    pub async fn execute_genesis(&self, _node_id_str: String) -> Result<()> {
        Err(GenesisError::CoordinationFailed("Coordination feature not enabled".to_string()))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;

    #[cfg(feature = "coordination")]
    #[tokio::test]
    async fn test_genesis_coordination_bridge_creation() {
        use songbird_primal_coordination::types::*;
        use songbird_primal_coordination::{
            DiscoveryBasedBridge, PrimalBridge, PrimalCoordinator, PrimalDiscovery,
            StaticPrimalDiscovery,
        };

        let discovery = Arc::new(PrimalDiscovery::Static(StaticPrimalDiscovery {
            endpoint: "http://mock:8080".into(),
            capabilities: PrimalCapabilities {
                services: vec!["security".to_string()],
                resources: std::collections::HashMap::new(),
                metadata: std::collections::HashMap::new(),
                quality: ServiceQuality::default(),
            },
        }));
        let bridge = Arc::new(PrimalBridge::DiscoveryBased(DiscoveryBasedBridge::new(discovery)));
        let coordinator = Arc::new(PrimalCoordinator::new(bridge));
        let genesis_bridge = GenesisCoordinationBridge::new(coordinator);

        // Should be able to request security capability
        let result = genesis_bridge.request_security_capability().await;
        assert!(result.is_ok());
    }

    #[cfg(not(feature = "coordination"))]
    #[tokio::test]
    async fn test_fallback_mode() {
        let bridge = GenesisCoordinationBridge::new_fallback();
        let err = bridge
            .execute_genesis("test-node-123".to_string())
            .await
            .expect_err("fallback must error");
        match err {
            GenesisError::CoordinationFailed(msg) => {
                assert!(msg.contains("not enabled"), "message should explain feature flag: {msg}");
            }
            other => panic!("expected CoordinationFailed, got {other:?}"),
        }
    }

    #[cfg(not(feature = "coordination"))]
    #[tokio::test]
    async fn fallback_execute_genesis_empty_node_id_still_errors() {
        let bridge = GenesisCoordinationBridge::new_fallback();
        let err = bridge
            .execute_genesis(String::new())
            .await
            .expect_err("coordination remains unavailable");
        assert!(
            matches!(err, GenesisError::CoordinationFailed(_)),
            "expected CoordinationFailed, got {err:?}"
        );
    }
}
