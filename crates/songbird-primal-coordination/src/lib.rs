// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::clone_on_ref_ptr,
    reason = "Arc::clone() is idiomatic for shared ownership in async contexts"
)]
#![allow(
    clippy::ignore_without_reason,
    reason = "Historical patterns in this crate; inherited workspace pedantic lints."
)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions"))]
//! # 🌳 Songbird Primal Coordination - Universal Signal and Coordinator
//!
//! **MISSION**: Songbird is the nervous system, not the organs
//!
//! This crate implements capability-based primal coordination with ZERO hardcoded primal names.
//! Each primal discovers itself and others through capabilities, not hardcoded connections.
//!
//! ## Core Principles
//!
//! 1. **Zero Primal Name Hardcoding**: Never reference `BearDog`, Toadstool, `NestGate`, Squirrel
//! 2. **Capability-Based Discovery**: Request "security", "compute", "storage", "ai"
//! 3. **Self-Knowledge Only**: Each primal knows itself, discovers others
//! 4. **N to 1 Coordination**: N primals connect through Songbird, not 2^N connections
//! 5. **Agnostic Abstraction**: Like gaming system evolution
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Songbird Coordinator                     │
//! │  (Universal Signal - Knows capabilities, not primals)       │
//! └───┬─────────┬─────────┬─────────┬─────────┬───────────────┘
//!     │         │         │         │         │
//!     ▼         ▼         ▼         ▼         ▼
//! ┌───────┐ ┌───────┐ ┌───────┐ ┌───────┐ ┌───────┐
//! │Primal │ │Primal │ │Primal │ │Primal │ │Primal │
//! │   A   │ │   B   │ │   C   │ │   D   │ │   E   │
//! │(Sec)  │ │(Comp) │ │(Stor) │ │(AI)   │ │(?)    │
//! └───────┘ └───────┘ └───────┘ └───────┘ └───────┘
//! ```
//!
//! ## Example Usage
//!
//! ```rust,ignore
//! use songbird_primal_coordination::*;
//!
//! async fn example() -> Result<()> {
//! // Create coordinator with zero knowledge
//! let mut coordinator = PrimalCoordinator::new();
//!
//! // Primals register themselves (not hardcoded)
//! // coordinator.register_primal(some_bridge).await?;
//!
//! // Request capability, get provider
//! let security_conn = coordinator.request_capability("security").await?;
//!
//! // Coordinate operation (e.g., genesis ceremony)
//! // let identity = coordinator.coordinate_genesis(node_id).await?;
//! # Ok(())
//! # }
//! ```
#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// Capability-based bridges and discovery helpers for connecting to primals.
pub mod bridge;

/// [`PrimalCoordinator`] and mesh coordination.
pub mod coordinator;

/// Error types and [`Result`] alias for this crate.
pub mod error;

/// Shared request/response and identity types for primal coordination.
pub mod types;

// Re-exports
pub use bridge::{PrimalBridge, PrimalConnection};
pub use coordinator::PrimalCoordinator;
pub use error::{PrimalCoordinationError, Result};
pub use types::*;

#[cfg(test)]
mod lib_smoke_tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use crate::{
        CapabilityType, DeploymentId, NodeId, PrimalCapabilities, PrimalCoordinationError,
        PrimalRequest, ServiceQuality,
    };
    use std::collections::HashMap;

    #[test]
    fn capability_type_display_matches_as_str() {
        assert_eq!(CapabilityType::Orchestration.to_string(), "orchestration");
        assert_eq!(format!("{}", CapabilityType::Networking), "networking");
    }

    #[test]
    fn primal_capabilities_supports_workload_and_default_quality() {
        let caps = PrimalCapabilities {
            services: vec!["batch".into()],
            resources: HashMap::new(),
            metadata: HashMap::new(),
            quality: ServiceQuality::default(),
        };
        let w = crate::Workload {
            id: "1".into(),
            service_type: "batch".into(),
            requirements: HashMap::new(),
            payload: serde_json::json!({}),
        };
        assert!(caps.supports_workload(&w));
    }

    #[test]
    fn node_id_and_deployment_id_display() {
        let n = NodeId("nid".into());
        let d = DeploymentId("did".into());
        assert_eq!(n.to_string(), "nid");
        assert_eq!(d.to_string(), "did");
    }

    #[test]
    fn primal_request_sign_lineage_roundtrip() {
        let req = PrimalRequest::SignLineage {
            keys: crate::GeneratedKeys {
                public_key: vec![1, 2],
                private_key_handle: "h".into(),
            },
            proof: crate::WitnessProof {
                data: vec![9],
            },
            node_id: NodeId("n".into()),
        };
        let v = serde_json::to_value(&req).unwrap();
        let back: PrimalRequest = serde_json::from_value(v).unwrap();
        assert!(matches!(back, PrimalRequest::SignLineage { .. }));
    }

    #[test]
    fn primal_coordination_error_display_variants() {
        let e = PrimalCoordinationError::UnexpectedResponse("bad".into());
        assert!(e.to_string().contains("Unexpected"));
        let e2 = PrimalCoordinationError::PrimalError("p".into());
        assert!(e2.to_string().contains("Primal error"));
        let e3 = PrimalCoordinationError::DiscoveryFailed("d".into());
        assert!(e3.to_string().contains("Discovery"));
    }
}
