// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Classification enums for providers, primals, and services.

use serde::{Deserialize, Serialize};

/// Classify a provider implementation for routing and policy.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ProviderType {
    /// Exposes HTTP or RPC-style services.
    Service,
    /// Runs a Songbird primal workload.
    Primal,
    /// Participates in service discovery.
    Discovery,
    /// Advertises or resolves capabilities.
    Capability,
    /// Handles authn/authz and secrets.
    Security,
    /// Coordinates deployments and lifecycle.
    Orchestration,
    /// Emits metrics, logs, and traces.
    Observability,
    /// Extension point for custom provider kinds.
    Custom(String),
}

/// Identify which primal domain an instance belongs to.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PrimalType {
    /// Security and policy primal.
    Security,
    /// Storage primal.
    Storage,
    /// Compute primal.
    Compute,
    /// AI/ML primal.
    AI,
    /// Network primal.
    Network,
    /// Custom primal type label.
    Custom(String),
}

/// Describe the role of a discovered or registered service.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServiceType {
    /// Web or HTTP API tier.
    WebService,
    /// Database or datastore.
    Database,
    /// Message broker or queue.
    MessageQueue,
    /// In-memory or distributed cache.
    Cache,
    /// Object or file storage.
    FileStorage,
    /// Identity and authentication.
    Authentication,
    /// Custom service classification.
    Custom(String),
}
