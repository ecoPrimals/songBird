// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![cfg_attr(
    test,
    expect(clippy::float_cmp, reason = "test: exact float comparison is intentional")
)]
//! Peer Trust Evaluation
//!
//! Evaluates whether to trust discovered peers by consulting the security provider (security provider).
//! This is part of the USB seed integration - security provider makes the trust decision based on
//! genetic lineage derived from the USB family seed.

pub(crate) mod evaluation;
pub mod types;

pub use evaluation::evaluate_peer_trust;
pub use types::{DiscoveredPeer, PeerTrustDecision};

#[cfg(test)]
#[path = "peer_trust_tests.rs"]
mod tests;
