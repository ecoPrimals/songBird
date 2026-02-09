//! Circuit protocol - Build and manage Tor circuits
//!
//! **Status**: Phase 2B (In Progress)

mod create;
mod extend;
mod state;
/// Circuit manager module
pub mod manager;
mod onion;

pub use create::{NtorHandshake, HandshakeState, KeyMaterial};
pub use extend::CircuitExtender;
pub use state::{Circuit, CircuitHop, CircuitPurpose};
pub use manager::CircuitManager;
pub use onion::OnionCrypto;
