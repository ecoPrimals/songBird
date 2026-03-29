// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Provider configuration and metadata.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

/// Pass startup settings and feature toggles into a provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Opaque JSON settings keyed by provider-specific names.
    pub settings: HashMap<String, serde_json::Value>,
    /// Feature flags enabled for this provider instance.
    pub enabled_features: Vec<String>,
    /// Deployment environment label (for example `production`).
    pub environment: String,
}

/// Describe a provider for dashboards and support tooling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetadata {
    /// Short human-readable summary.
    pub description: String,
    /// Searchable labels for discovery UIs.
    pub tags: Vec<String>,
    /// Link to external documentation.
    pub documentation_url: Option<String>,
    /// Escalation contact for operators.
    pub support_contact: Option<String>,
    /// Creation time of this metadata record.
    pub created_at: SystemTime,
    /// Last update time of this metadata record.
    pub updated_at: SystemTime,
}
