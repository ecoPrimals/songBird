// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Capability contracts and parameter specifications.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Advertise a callable capability with typed parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    /// Stable capability name.
    pub name: String,
    /// Semantic version of the capability contract.
    pub version: String,
    /// What callers should expect when invoking this capability.
    pub description: String,
    /// Parameter schemas keyed by parameter name.
    pub parameters: HashMap<String, ParameterSpec>,
}

/// Describe one parameter accepted by a capability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSpec {
    /// Logical type name (JSON schema id or shorthand).
    pub parameter_type: String,
    /// Whether callers must supply this parameter.
    pub required: bool,
    /// Human-readable parameter description.
    pub description: String,
    /// Default used when the parameter is omitted.
    pub default_value: Option<serde_json::Value>,
}

/// Richer capability description for catalogs and codegen.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityMetadata {
    /// Capability name.
    pub name: String,
    /// Long-form description for tooling.
    pub description: String,
    /// Contract version.
    pub version: String,
    /// Parameter specs keyed by name.
    pub parameters: HashMap<String, ParameterSpec>,
    /// Example JSON payloads for documentation.
    pub examples: Vec<serde_json::Value>,
}
