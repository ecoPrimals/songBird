// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

use super::node::NodeRegistration;

/// Federation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationStats {
    /// Total number of nodes (including inactive)
    pub total_nodes: usize,

    /// Number of active nodes
    pub active_nodes: usize,

    /// Total CPU cores across active nodes
    pub total_cpu_cores: usize,

    /// Total memory in GB across active nodes
    pub total_memory_gb: usize,

    /// Total storage in GB across active nodes
    pub total_storage_gb: usize,

    /// Federation uptime in seconds since creation
    pub uptime_seconds: Option<u64>,
}

/// Federation status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationStatus {
    /// Federation unique ID
    pub federation_id: String,

    /// Number of active nodes
    pub active_nodes: usize,

    /// All registered nodes
    pub nodes: Vec<NodeRegistration>,

    /// Total resources
    pub total_cpu_cores: usize,
    pub total_memory_gb: usize,
    pub total_storage_gb: usize,

    /// Federation uptime in seconds
    pub uptime_seconds: i64,
}
