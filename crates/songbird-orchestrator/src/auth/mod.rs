// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Authentication and Authorization — Capability-Based Security Discovery
//!
//! This module handles JWT-based authentication via security provider delegation.
//!
//! ## Architecture
//!
//! Songbird delegates JWT generation and management to the security provider:
//! - **Security Provider**: Generates JWT secrets (Pure Rust ed25519-dalek)
//! - **Songbird**: Validates JWTs (ed25519-dalek verification)
//! - **Communication**: JSON-RPC over Unix socket (Pure Rust!)
//!
//! ## Discovery
//!
//! Songbird discovers security providers via capability-based discovery:
//! - Searches for "security" capability first
//! - Falls back to secure random if unavailable
//! - Maintains self-knowledge (only knows itself)

pub mod security_jwt_client; // Security provider JWT delegation (Pure Rust!)

pub mod capability_discovery; // Capability-based security discovery (TRUE PRIMAL!)

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests; // Integration tests for JWT delegation

pub use security_jwt_client::{
    fetch_jwt_secret_from_security_provider, generate_secure_random_jwt, provision_jwt_secret,
};

// Capability-based security discovery (preferred API)
pub use capability_discovery::{
    discover_security_socket, discover_security_socket_for_family, discover_security_socket_with,
    get_security_socket_for_jwt, get_security_socket_for_jwt_with,
};
