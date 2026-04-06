// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Hole Punch Coordinator
//!
//! Coordinates NAT traversal using signaling channel (Tor, WebSocket, etc.)
//!
//! ## Algorithm
//!
//! 1. Both peers register with rendezvous, sharing STUN-discovered addresses
//! 2. Initiator sends PunchRequest with nonce
//! 3. Responder sends PunchAck with coordinated start time
//! 4. Both start sending UDP packets at the same time
//! 5. First to receive reports PunchResult
//! 6. If failed, fall back to relay mode

mod config;
mod core;
mod punch;
mod relay;
mod stun;
mod types;
mod util;

#[cfg(test)]
mod tests;

pub use config::HolePunchConfig;
pub use core::HolePunchCoordinator;
pub use types::{CoordinatedPunchResult, PunchResult};
