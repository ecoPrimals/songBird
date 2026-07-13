// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Primal introspection and self-description
//!
//! Extracted from `service.rs` for smart refactoring. These methods provide
//! self-knowledge — the primal describing its own capabilities, methods,
//! and identity. They follow the TRUE PRIMAL principle: Songbird only
//! knows about itself, never about other primals.
//!
//! ## Methods
//!
//! - `primal.info` - Primal metadata
//! - `primal.capabilities` - Detailed capability descriptions
//! - `rpc.methods` - Available JSON-RPC methods
//! - `rpc.discover` - biomeOS standard method listing
//! - `health` - Health status
//! - `identity` - Primal identity
//!
//! ## Submodules
//!
//! - `capability_tokens` — flat token list for `capabilities.list`
//! - `health_payloads` — liveness / readiness / check payloads
//! - `primal` — `primal.info` and `primal.capabilities`
//! - `rpc` — method listings and `discover_capabilities`
//! - `identity_payloads` — family identity and `identity` response

pub(crate) mod capability_tokens;
mod health_payloads;
mod identity_payloads;
mod primal;
mod rpc;

pub use capability_tokens::{
    CAPABILITY_METHOD_MAP, CONSUMED_CAPABILITIES, SONGBIRD_CAPABILITY_STRINGS, capabilities_list,
    capabilities_list_with_runtime, capabilities_methods,
};
pub use health_payloads::{SubsystemStatus, health, health_check, health_liveness, health_readiness};
pub use identity_payloads::{canonical_family_id, identity, identity_get};
pub use primal::{
    btsp_capabilities, primal_announce, primal_announce_with_socket, primal_capabilities,
    primal_info,
};
pub use rpc::{discover_capabilities, normalize_method, rpc_discover_standard, rpc_methods};

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests;
