// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Circuit protocol - Build and manage Tor circuits
//!
//! **Status**: Phase 2B (In Progress)

mod create;
mod extend;
/// Circuit manager module
pub mod manager;
mod onion;
mod state;

pub use create::{HandshakeState, KeyMaterial, NtorHandshake};
pub use extend::CircuitExtender;
pub use manager::CircuitManager;
pub use onion::OnionCrypto;
pub use state::{Circuit, CircuitHop, CircuitPurpose};
