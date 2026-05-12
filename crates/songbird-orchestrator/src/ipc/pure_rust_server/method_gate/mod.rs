// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Pre-dispatch capability gate for JSON-RPC methods (JH-0).
//!
//! Every incoming RPC call passes through [`MethodGate::check`] *before*
//! reaching the dispatch table. The gate classifies methods into
//! [`MethodAccessLevel::Public`] (health probes, identity, capability
//! advertisement — always allowed) and [`MethodAccessLevel::Protected`]
//! (require a valid capability token once enforcement is activated).
//!
//! Two enforcement modes:
//! - **Permissive** (default): protected methods are logged but allowed,
//!   preserving backward compatibility during ecosystem rollout.
//! - **Enforced**: protected methods without a valid token are rejected
//!   with `PERMISSION_DENIED` (-32001).
//!
//! Implements the ecosystem standard defined in
//! `primalSpring/wateringHole/METHOD_GATE_STANDARD.md`.

pub(crate) use super::protocol;

mod caller;
mod classification;
mod gate;
mod token;

// Re-export public API (flat surface, no breaking changes)
pub use caller::{CallerContext, ConnectionOrigin, PeerCredentials};
pub use classification::{
    MethodAccessLevel, classify_method, extract_bearer_token, scope_permits_method,
};
pub use gate::{
    EnforcementMode, MethodGate, dispatch_auth_method, handle_auth_check, handle_auth_mode,
    handle_auth_peer_info, is_gate_handled_method,
};
pub use token::{BearDogVerifier, NoopVerifier, TokenClaims, TokenVerifier, TokenVerifyError};

/// Server-defined error codes (JSON-RPC 2.0 range: -32000 to -32099).
pub mod error_codes {
    /// Caller identity could not be established.
    pub const UNAUTHORIZED: i32 = -32_000;
    /// Caller identity established but lacks scope for the method.
    pub const PERMISSION_DENIED: i32 = -32_001;
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests;
