// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! # Capability Registration - Neural API Integration
//!
//! This module handles automatic registration of Songbird's capabilities with
//! the Neural API, enabling TRUE PRIMAL loose coupling and capability-based discovery.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │ SONGBIRD STARTUP                                            │
//! │ 1. Initialize TLS stack                                     │
//! │ 2. Start JSON-RPC server                                    │
//! │ 3. ✨ Register capabilities with Neural API                │
//! │ 4. Accept requests                                          │
//! └─────────────────────┬───────────────────────────────────────┘
//!                       │
//!                       │ capability.register
//!                       │
//! ┌─────────────────────▼───────────────────────────────────────┐
//! │ NEURAL API - Capability Registry                            │
//! │ secure_http → [songbird-{family_id}]                        │
//! │   - http.get, http.post, http.put, http.delete            │
//! └─────────────────────┬───────────────────────────────────────┘
//!                       │
//! ┌─────────────────────▼───────────────────────────────────────┐
//! │ CONSUMER PRIMALS (AI coordination, etc.)                    │
//! │ neural_api.capability_call("secure_http", "http.post", {}) │
//! │ → Zero knowledge of Songbird required!                      │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Benefits
//!
//! - ✅ **Zero Configuration** - Primals discover Songbird automatically
//! - ✅ **Loose Coupling** - No hardcoded dependencies
//! - ✅ **Semantic APIs** - Operations like `http.post` just work
//! - ✅ **Isomorphic Evolution** - Songbird can evolve without breaking consumers
//! - ✅ **Fail-Safe** - Registration failure doesn't block Songbird startup

mod config;
mod lifecycle;
mod payload;
mod transport;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests;

pub use config::CapabilityRegistrationConfig;
pub use lifecycle::{
    check_neural_api_available, check_neural_api_available_at, register_capabilities,
    register_capabilities_with, unregister_capabilities, unregister_capabilities_with,
};
