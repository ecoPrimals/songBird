// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! `CanonicalTransport` implementation for IGD (UPnP/NAT-PMP port mapping).

use songbird_types::{CanonicalTransport, SongbirdResult, TransportEndpoint, TransportHealth};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Transport adapter for IGD gateway operations.
pub struct IgdTransport {
    running: AtomicBool,
    active_mappings: AtomicU64,
}

impl IgdTransport {
    /// Create a new IGD transport adapter.
    #[must_use]
    pub fn new() -> Self {
        Self {
            running: AtomicBool::new(false),
            active_mappings: AtomicU64::new(0),
        }
    }

    /// Record a new port mapping.
    pub fn mapping_added(&self) {
        self.active_mappings.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a removed port mapping.
    pub fn mapping_removed(&self) {
        self.active_mappings.fetch_sub(1, Ordering::Relaxed);
    }
}

impl Default for IgdTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl CanonicalTransport for IgdTransport {
    fn transport_name(&self) -> &'static str {
        "IGD"
    }

    async fn is_ready(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }

    async fn start(&self) -> SongbirdResult<()> {
        self.running.store(true, Ordering::Release);
        Ok(())
    }

    async fn shutdown(&self) -> SongbirdResult<()> {
        self.running.store(false, Ordering::Release);
        Ok(())
    }

    async fn health(&self) -> TransportHealth {
        TransportHealth {
            ready: self.running.load(Ordering::Acquire),
            active_connections: self.active_mappings.load(Ordering::Relaxed),
            message: String::from("UPnP/NAT-PMP port mapping"),
        }
    }

    fn endpoints(&self) -> Vec<TransportEndpoint> {
        vec![]
    }
}
