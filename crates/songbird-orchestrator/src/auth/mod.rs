//! Authentication and authorization for Songbird
//!
//! This module handles JWT-based authentication via BearDog delegation.
//!
//! ## Architecture
//!
//! Songbird delegates JWT generation and management to BearDog (security primal):
//! - **BearDog**: Generates JWT secrets (Pure Rust ed25519-dalek)
//! - **Songbird**: Validates JWTs (ed25519-dalek verification)
//! - **Communication**: JSON-RPC over Unix socket (Pure Rust!)
//!
//! ## Discovery
//!
//! Songbird discovers BearDog via capability-based discovery:
//! - Searches for "security" capability
//! - Falls back to secure random if unavailable
//! - Maintains self-knowledge (only knows itself)

pub mod beardog_jwt_client; // BearDog JWT delegation (Pure Rust!)
pub mod capability_discovery; // Capability-based BearDog discovery (TRUE PRIMAL!)

#[cfg(test)]
mod tests; // Integration tests for JWT delegation

pub use beardog_jwt_client::{
    fetch_jwt_secret_from_beardog, generate_secure_random_jwt, provision_jwt_secret,
};

pub use capability_discovery::{
    discover_beardog_socket, discover_beardog_socket_for_family, discover_beardog_socket_with,
    get_beardog_socket_for_jwt, get_beardog_socket_for_jwt_with,
};
