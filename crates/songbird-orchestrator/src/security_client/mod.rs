// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Security capability client for cryptographic trust evaluation
//!
//! **MODERNIZED v3.12.3**: Now uses protocol-agnostic `SecurityAdapter`!
//!
//! This module provides a protocol-agnostic API for discovering and using security capabilities
//! without hardcoding specific primal names. Works with ANY primal that provides
//! security capabilities (identity, encryption, trust-evaluation) via ANY protocol.
//!
//! ## Modern Architecture (v3.12.3)
//!
//! - **Security Provider**: ANY primal offering security capabilities (discovered at runtime)
//! - **Protocol Detection**: Automatic (tarpc → JSON-RPC → HTTP)
//! - **Performance**: 10-50x faster with tarpc/JSON-RPC
//! - **Deployment**: Fractal (same code, any protocol)
//!
//! ## Usage (Protocol-Agnostic)
//!
//! ```rust,ignore
//! use songbird_orchestrator::security_capability_client::{SecurityCapabilityClient, TrustEvaluationRequest};
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Discover security provider at runtime (NO hardcoded endpoint!)
//! let endpoint = discover_capability("security").await?;
//! let client = SecurityCapabilityClient::from_endpoint(endpoint);
//!
//! // Get our identity
//! let identity = client.get_identity().await?;
//! println!("Our tag: {}", identity.encryption_tag);
//!
//! // Evaluate peer trust
//! let request = TrustEvaluationRequest {
//!     peer_id: "tower2".to_string(),
//!     peer_family: Some("a3f2".to_string()),
//!     peer_tags: vec!["crypto:family:a3f2".to_string()],
//!     connection_info: None,
//!     context: None,
//! };
//!
//! let decision = client.evaluate_trust(&request).await?;
//! match decision.decision.as_str() {
//!     "auto_accept" => println!("✅ Auto-accepting peer"),
//!     "prompt_user" => println!("⚠️ Prompting user for consent"),
//!     "reject" => println!("❌ Rejecting peer"),
//!     _ => println!("Unknown decision"),
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Module Structure
//!
//! - `client`: `SecurityCapabilityClient` implementation
//! - `types`: All request/response types

pub mod client;
pub mod types;

// Re-export main types
pub use client::SecurityCapabilityClient;
pub use types::{
    ConnectionInfo, CurrentLineageInfo, DiscoveryContext, IdentityResponse, TrustEvaluationRequest,
    TrustEvaluationResponse, VerificationResult,
};
