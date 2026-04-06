// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Hole-punch attempt state exposed to JSON-RPC callers.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Status of a punch attempt
#[derive(Debug, Clone)]
pub struct PunchAttempt {
    /// Target node ID
    pub target_node_id: Arc<str>,
    /// Current status
    pub status: PunchStatus,
    /// Number of attempts made
    pub attempts: u32,
    /// Max attempts before giving up
    pub max_attempts: u32,
    /// When the punch was started
    pub started: Instant,
    /// Connected address if successful
    pub connected_address: Option<SocketAddr>,
    /// Measured latency if successful
    pub latency: Option<Duration>,
}

/// Punch attempt status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PunchStatus {
    /// Punch in progress
    InProgress,
    /// Punch succeeded - direct connection established
    Succeeded,
    /// Punch failed - will use relay fallback
    Failed {
        reason: Arc<str>,
    },
}
