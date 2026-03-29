// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Primal execution context, responses, dependencies, and discovery views.

use super::classification::PrimalType;
use super::health::HealthStatus;
use super::service::Endpoint;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Pass caller and device context into primal execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalContext {
    /// Authenticated user id when available.
    pub user_id: String,
    /// Stable device identifier.
    pub device_id: String,
    /// Deployment environment label.
    pub environment: String,
    /// Coarse security tier for policy checks.
    pub security_level: String,
    /// Free-form context for auditing.
    pub metadata: HashMap<String, String>,
}

/// Return structured output from a primal capability run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalResponse {
    /// Whether the capability succeeded.
    pub success: bool,
    /// Primary JSON payload.
    pub data: serde_json::Value,
    /// Secondary metadata for clients and logs.
    pub metadata: HashMap<String, String>,
    /// Wall-clock execution time in milliseconds.
    pub execution_time_ms: u64,
}

/// Declare another service or primal this primal relies on.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalDependency {
    /// Logical name of the dependency.
    pub service_name: String,
    /// Minimum compatible version range description.
    pub required_version: String,
    /// Whether startup may proceed if the dependency is absent.
    pub optional: bool,
    /// Capabilities required from the dependency.
    pub capabilities: Vec<String>,
}

/// Report outcome when two primals link at runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationResult {
    /// Whether integration completed successfully.
    pub success: bool,
    /// Capabilities now shared across the link.
    pub shared_capabilities: Vec<String>,
    /// Established channels (socket paths, topics, etc.).
    pub communication_channels: Vec<String>,
    /// Diagnostic metadata for operators.
    pub metadata: HashMap<String, String>,
}

/// Summarize a primal for discovery and federation views.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalInfo {
    /// Stable primal instance id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Domain classification.
    pub primal_type: PrimalType,
    /// Deployed version.
    pub version: String,
    /// Advertised capability names.
    pub capabilities: Vec<String>,
    /// Network endpoints for this primal.
    pub endpoints: Vec<Endpoint>,
    /// Latest health.
    pub health: HealthStatus,
    /// Arbitrary metadata for routing policy.
    pub metadata: HashMap<String, String>,
}
