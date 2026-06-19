// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Progressive Trust Model Types
//!
//! Implements a progressive trust model with capability-based access control.
//!
//! ## Trust Levels
//!
//! - **Level 0 (None)**: No trust - reject connection
//! - **Level 1 (Limited)**: Same genetic family - `BirdSong` coordination only
//! - **Level 2 (Elevated)**: Human approved - full federation
//! - **Level 3 (Highest)**: Human entropy - sensitive operations
//!
//! ## Philosophy
//!
//! "Same family = can hear the song, NOT enter the nest"
//!
//! Genetic lineage establishes recognition but NOT full access.
//! Progressive elevation requires human oversight.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

/// Progressive trust levels for peer connections
///
/// Each level grants specific capabilities with clear boundaries.
///
/// **Phase 1 (v3.13.1)**: Accepts both integer and string formats from `security provider`!
/// - Deserialize: Accepts integer OR string (flexible!)
/// - Serialize: Always produces integer (compact, efficient)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum TrustLevel {
    /// No trust - different family or no lineage
    ///
    /// Allowed: Nothing
    /// Decision: Reject connection
    None = 0,

    /// Limited trust - same genetic family
    ///
    /// Allowed: `BirdSong` coordination, health checks, capability discovery
    /// Denied: Data access, commands, full federation
    /// Philosophy: "Can hear the song, cannot enter the nest"
    Limited = 1,

    /// Elevated trust - human approved
    ///
    /// Allowed: Full federation, resource sharing, data read
    /// Denied: Sensitive operations, key access, data write
    /// Requirement: Human approval via UI
    Elevated = 2,

    /// Highest trust - human entropy added
    ///
    /// Allowed: Everything including sensitive operations
    /// Denied: Nothing
    /// Requirement: Human entropy (`SoloKey`, Phone HSM)
    Highest = 3,
}

/// Custom serializer for `TrustLevel` (always serialize as integer)
impl Serialize for TrustLevel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

/// Custom deserializer for `TrustLevel` (Phase 1: Accept both int and string)
///
/// **`security provider` Compatibility**:
/// - Accepts integer: 0, 1, 2, 3
/// - Accepts string: "none", "limited", "elevated", "highest"
/// - Accepts aliases: "anonymous", "basic", "medium", "explicit"
impl<'de> Deserialize<'de> for TrustLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum TrustLevelHelper {
            Int(u8),
            String(String),
        }

        match TrustLevelHelper::deserialize(deserializer)? {
            // Integer format (security provider primary)
            TrustLevelHelper::Int(0) => Ok(Self::None),
            TrustLevelHelper::Int(1) => Ok(Self::Limited),
            TrustLevelHelper::Int(2) => Ok(Self::Elevated),
            TrustLevelHelper::Int(3) => Ok(Self::Highest),
            TrustLevelHelper::Int(n) => Err(serde::de::Error::custom(format!(
                "Invalid trust level integer: {n} (expected 0-3)",
            ))),

            // String format (aliases for compatibility)
            TrustLevelHelper::String(s) => match s.to_lowercase().as_str() {
                // None: Primary and aliases
                "none" | "anonymous" | "unknown" => Ok(Self::None),

                // Limited: Primary and aliases
                "limited" | "basic" => Ok(Self::Limited),

                // Elevated: Primary and aliases
                "elevated" | "medium" => Ok(Self::Elevated),

                // Highest: Primary and aliases
                "highest" | "explicit" | "full" => Ok(Self::Highest),

                _ => Err(serde::de::Error::custom(format!(
                    "Unknown trust level string: '{s}' (expected: none, limited, elevated, highest)"
                ))),
            },
        }
    }
}

impl TrustLevel {
    /// Get trust level from numeric value
    #[must_use]
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::Limited),
            2 => Some(Self::Elevated),
            3 => Some(Self::Highest),
            _ => None,
        }
    }

    /// Get numeric value
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    /// Get human-readable name
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Limited => "limited",
            Self::Elevated => "elevated",
            Self::Highest => "highest",
        }
    }

    /// Get `security provider` alias for compatibility
    #[must_use]
    pub const fn beardog_alias(self) -> &'static str {
        match self {
            Self::None => "anonymous",
            Self::Limited => "basic",
            Self::Elevated => "medium",
            Self::Highest => "explicit",
        }
    }

    /// Get description
    #[must_use]
    pub const fn description(self) -> &'static str {
        match self {
            Self::None => "No trust - reject connection",
            Self::Limited => "Limited trust - BirdSong coordination only (same family)",
            Self::Elevated => "Elevated trust - full federation (human approved)",
            Self::Highest => "Highest trust - all operations (human entropy)",
        }
    }

    /// Get default allowed capabilities for this level
    #[must_use]
    pub fn default_allowed_capabilities(self) -> Vec<String> {
        match self {
            Self::None => vec![],
            Self::Limited => vec![
                String::from("discovery"),
                String::from("coordination/*"),
                String::from("birdsong/*"),
                String::from("health"),
                String::from("capabilities"),
            ],
            Self::Elevated => vec![
                String::from("discovery"),
                String::from("coordination/*"),
                String::from("birdsong/*"),
                String::from("health"),
                String::from("capabilities"),
                String::from("federation/*"),
                String::from("data/read"),
            ],
            Self::Highest => vec![String::from("*")],
        }
    }

    /// Get default denied capabilities for this level
    #[must_use]
    pub fn default_denied_capabilities(self) -> Vec<String> {
        match self {
            Self::None => vec![String::from("*")],
            Self::Limited => vec![
                String::from("data/*"),
                String::from("commands/*"),
                String::from("federation/*"),
                String::from("keys/*"),
            ],
            Self::Elevated => vec![
                String::from("data/write"),
                String::from("commands/sensitive"),
                String::from("keys/*"),
            ],
            Self::Highest => vec![],
        }
    }
}

impl std::fmt::Display for TrustLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Trust evaluation result with capability restrictions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustEvaluation {
    /// Trust level determined
    pub level: TrustLevel,

    /// Confidence in this evaluation (0.0 - 1.0)
    pub confidence: f64,

    /// Human-readable reason
    pub reason: String,

    /// Machine-readable reason code
    pub reason_code: String,

    /// Capabilities allowed at this trust level
    pub allowed_capabilities: Vec<String>,

    /// Capabilities explicitly denied
    pub denied_capabilities: Vec<String>,

    /// Path to elevate trust level
    pub elevation_path: Option<ElevationPath>,

    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl TrustEvaluation {
    /// Create evaluation with default capabilities for level
    pub fn new(level: TrustLevel, reason: impl Into<String>) -> Self {
        let reason_str = reason.into();
        Self {
            level,
            confidence: 1.0,
            reason: reason_str.clone(),
            reason_code: Self::reason_to_code(&reason_str),
            allowed_capabilities: level.default_allowed_capabilities(),
            denied_capabilities: level.default_denied_capabilities(),
            elevation_path: Self::default_elevation_path(level),
            metadata: HashMap::new(),
        }
    }

    /// Check if an operation is allowed
    #[must_use]
    pub fn is_operation_allowed(&self, operation: &str) -> bool {
        is_operation_allowed(operation, &self.allowed_capabilities, &self.denied_capabilities)
    }

    fn reason_to_code(reason: &str) -> String {
        reason
            .to_lowercase()
            .replace(' ', "_")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect()
    }

    fn default_elevation_path(level: TrustLevel) -> Option<ElevationPath> {
        match level {
            TrustLevel::None | TrustLevel::Highest => None,
            TrustLevel::Limited => Some(ElevationPath {
                next_level: TrustLevel::Elevated,
                requirements: vec![String::from("human_approval")],
                method: String::from("user_consent_ui"),
            }),
            TrustLevel::Elevated => Some(ElevationPath {
                next_level: TrustLevel::Highest,
                requirements: vec![String::from("human_entropy")],
                method: String::from("solokey_or_phone_hsm"),
            }),
        }
    }
}

/// Path to elevate trust to next level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElevationPath {
    /// Next trust level achievable
    pub next_level: TrustLevel,

    /// Requirements to achieve next level
    pub requirements: Vec<String>,

    /// Method to achieve elevation
    pub method: String,
}

/// Evidence for trust elevation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElevationEvidence {
    /// Type of evidence (`human_approval`, `human_entropy`)
    pub evidence_type: String,

    /// Timestamp of evidence collection
    pub timestamp: String,

    /// Method used to collect evidence
    pub method: String,

    /// Optional entropy data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entropy: Option<Vec<u8>>,
}

/// Check if operation matches capability pattern
///
/// Supports wildcards: "data/*" matches "data/read", "data/write"
#[must_use]
pub fn is_operation_allowed(operation: &str, allowed: &[String], denied: &[String]) -> bool {
    // Check denied first (explicit deny overrides allow)
    if denied.iter().any(|pattern| matches_pattern(operation, pattern)) {
        return false;
    }

    // Check allowed
    allowed.iter().any(|pattern| matches_pattern(operation, pattern))
}

fn matches_pattern(operation: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true; // Wildcard matches everything
    }

    pattern
        .strip_suffix("/*")
        .map_or_else(|| operation == pattern, |prefix| operation.starts_with(prefix))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_level_ordering() {
        assert!(TrustLevel::None < TrustLevel::Limited);
        assert!(TrustLevel::Limited < TrustLevel::Elevated);
        assert!(TrustLevel::Elevated < TrustLevel::Highest);
    }

    #[test]
    fn test_trust_level_from_u8() {
        assert_eq!(TrustLevel::from_u8(0), Some(TrustLevel::None));
        assert_eq!(TrustLevel::from_u8(1), Some(TrustLevel::Limited));
        assert_eq!(TrustLevel::from_u8(2), Some(TrustLevel::Elevated));
        assert_eq!(TrustLevel::from_u8(3), Some(TrustLevel::Highest));
        assert_eq!(TrustLevel::from_u8(4), None);
    }

    #[test]
    fn test_operation_matching() {
        assert!(matches_pattern("data/read", "data/*"));
        assert!(matches_pattern("data/write", "data/*"));
        assert!(!matches_pattern("commands/exec", "data/*"));
        assert!(matches_pattern("anything", "*"));
        assert!(matches_pattern("health", "health"));
        assert!(!matches_pattern("health", "capabilities"));
    }

    #[test]
    fn test_limited_trust_allows_birdsong() {
        let eval = TrustEvaluation::new(TrustLevel::Limited, "same family");
        assert!(eval.is_operation_allowed("birdsong/sync"));
        assert!(eval.is_operation_allowed("coordination/state"));
        assert!(eval.is_operation_allowed("health"));
    }

    #[test]
    fn test_limited_trust_denies_data() {
        let eval = TrustEvaluation::new(TrustLevel::Limited, "same family");
        assert!(!eval.is_operation_allowed("data/read"));
        assert!(!eval.is_operation_allowed("data/write"));
        assert!(!eval.is_operation_allowed("commands/exec"));
    }

    #[test]
    fn test_elevated_trust_allows_federation() {
        let eval = TrustEvaluation::new(TrustLevel::Elevated, "human approved");
        assert!(eval.is_operation_allowed("federation/join"));
        assert!(eval.is_operation_allowed("data/read"));
        assert!(!eval.is_operation_allowed("data/write"));
    }

    #[test]
    fn test_highest_trust_allows_everything() {
        let eval = TrustEvaluation::new(TrustLevel::Highest, "human entropy");
        assert!(eval.is_operation_allowed("data/read"));
        assert!(eval.is_operation_allowed("data/write"));
        assert!(eval.is_operation_allowed("commands/sensitive"));
        assert!(eval.is_operation_allowed("keys/access"));
    }
}
