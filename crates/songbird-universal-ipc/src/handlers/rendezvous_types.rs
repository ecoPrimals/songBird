// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Shared rendezvous types (extracted to avoid circular deps between `rendezvous_handler` and `http_rendezvous_client`).

use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct RendezvousRegisterParams {
    /// Rendezvous server URL
    pub server: Arc<str>,
    /// Our node ID
    pub node_id: Arc<str>,
    /// Our family ID (for family-scoped discovery)
    pub family_id: Arc<str>,
    /// Public address (from STUN)
    pub public_address: Arc<str>,
}

#[derive(Debug, Serialize, Clone)]
pub struct RendezvousRegisterResult {
    /// Registration ID
    pub registration_id: String,
    /// Expiry time (ISO 8601)
    pub expires_at: String,
    /// Rendezvous token for peers
    pub rendezvous_token: String,
}

#[derive(Debug, Deserialize)]
pub struct RendezvousLookupParams {
    /// Rendezvous server URL
    pub server: Arc<str>,
    /// Target node ID or family ID
    pub target: Arc<str>,
}

#[derive(Debug, Serialize, Clone)]
pub struct RendezvousLookupResult {
    /// Found peers
    pub peers: Vec<RendezvousPeer>,
}

#[derive(Debug, Serialize, Clone)]
pub struct RendezvousPeer {
    pub node_id: Arc<str>,
    pub family_id: Arc<str>,
    pub public_address: Arc<str>,
    pub rendezvous_token: Arc<str>,
}
