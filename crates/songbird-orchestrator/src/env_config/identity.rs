// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use songbird_types::primal_names;

use super::env;

/// Get this primal's name (self-knowledge)
#[must_use]
pub fn primal_name() -> String {
    env("PRIMAL_NAME").unwrap_or_else(|_| primal_names::SELF_NAME.to_string())
}

/// Get family/biome ID (self-knowledge)
///
/// Priority order (`BiomeOS` Neural API compatible):
/// 1. `SONGBIRD_ORCHESTRATOR_FAMILY_ID` (highest - Neural API standard)
/// 2. `SONGBIRD_ORCHESTRATOR_FAMILY` (alternative)
/// 3. `BIOMEOS_FAMILY_ID` (generic orchestrator)
/// 4. `SONGBIRD_FAMILY_ID` (legacy)
/// 5. `FAMILY_ID` (generic)
/// 6. Default: `"default"` (seed-derived family ID should be set via env)
#[must_use]
pub fn family_id() -> String {
    family_id_with(|k| songbird_process_env::var(k))
}

/// [`family_id`] with an injectable env reader (for unit tests and alternate backends).
#[must_use]
pub fn family_id_with<F>(env_reader: F) -> String
where
    F: Fn(&str) -> Result<String, std::env::VarError>,
{
    env_reader("SONGBIRD_ORCHESTRATOR_FAMILY_ID")
        .or_else(|_| env_reader("SONGBIRD_ORCHESTRATOR_FAMILY"))
        .or_else(|_| env_reader("BIOMEOS_FAMILY_ID"))
        .or_else(|_| env_reader("SONGBIRD_FAMILY_ID"))
        .or_else(|_| env_reader("FAMILY_ID"))
        .unwrap_or_else(|_| String::from("default"))
}

/// Get node ID (self-knowledge)
#[must_use]
pub fn node_id() -> String {
    env("SONGBIRD_NODE_ID").or_else(|_| env("NODE_ID")).unwrap_or_else(|_| String::from("default"))
}
