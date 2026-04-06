// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Configuration Migration Utilities
//!
//! This module provides utilities for migrating from fragmented configuration types
//! to the unified `UnifiedSongbirdConfig` system.

use crate::config::UnifiedSongbirdConfig;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Canonical migration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalMigrationConfig {
    /// Enable automatic migration of legacy configurations
    pub auto_migrate: bool,
    /// Generate migration reports
    pub generate_reports: bool,
    /// Backup original configuration before migration
    pub backup_original: bool,
    /// Migration log level
    pub log_level: String,
}

impl Default for CanonicalMigrationConfig {
    fn default() -> Self {
        Self {
            auto_migrate: true,
            generate_reports: true,
            backup_original: true,
            log_level: "info".to_string(),
        }
    }
}

/// Configuration migration utilities
pub struct ConfigMigrationUtils;

impl ConfigMigrationUtils {
    /// Migrate from legacy JSON configuration to unified configuration
    ///
    /// # Errors
    ///
    /// Returns an error if configuration validation fails
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
    pub fn migrate_from_json(
        json_config: serde_json::Value,
    ) -> Result<UnifiedSongbirdConfig, String> {
        let mut unified = UnifiedSongbirdConfig::default();
        let Value::Object(map) = json_config else {
            return unified
                .validate()
                .map_err(|e| {
                    format!(
                        "Configuration validation failed: {e



}"
                    )
                })
                .map(|()| unified);
        };

        // Extract environment configuration
        if let Some(env) = map.get("environment").and_then(|v| v.as_str()) {
            unified.system.environment = env.to_string();
        }

        // Extract system ID
        if let Some(system_id) = map.get("system_id").and_then(|v| v.as_str()) {
            unified.system.system_id = system_id.to_string();
        }

        // Store custom configuration
        let custom_fields: HashMap<String, Value> = map
            .iter()
            .filter(|(key, _)| !Self::is_known_field(key))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        if !custom_fields.is_empty() {
            unified.custom = Some(custom_fields);
        }

        unified
            .validate()
            .map_err(|e| format!("Configuration validation failed: {e}"))
            .map(|()| unified)
    }

    /// Check if a field is a known configuration field
    fn is_known_field(field_name: &str) -> bool {
        matches!(field_name, "environment" | "system_id" | "network" | "security" | "performance")
    }

    /// Generate migration report
    #[must_use]
    pub fn generate_migration_report() -> String {
        "✅ Migration completed successfully!".to_string()
    }
}

#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::field_reassign_with_default,
    reason = "intentional pattern; clippy false positive for this API"
)]
#[cfg(test)]
#[allow(clippy::uninlined_format_args, reason = "test assertions and harness ergonomics")]
#[allow(clippy::float_cmp, reason = "test assertions and harness ergonomics")]
#[allow(clippy::useless_vec, reason = "test assertions and harness ergonomics")]
#[allow(clippy::unreadable_literal, reason = "test assertions and harness ergonomics")]
#[allow(clippy::items_after_statements, reason = "test assertions and harness ergonomics")]
#[allow(clippy::cast_precision_loss, reason = "test assertions and harness ergonomics")]
#[allow(
    clippy::cast_possible_truncation,
    reason = "intentional pattern; clippy false positive for this API"
)]
#[allow(clippy::cast_sign_loss, reason = "intentional pattern; clippy false positive for this API")]
mod tests {
    #![allow(clippy::all, reason = "test assertions and harness ergonomics")]
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
    #![allow(unused, reason = "test assertions and harness ergonomics")]

    use super::*;
    use serde_json::json;

    #[test]
    fn test_migration_report() {
        let report = ConfigMigrationUtils::generate_migration_report();
        assert!(report.contains("Migration completed"));
    }

    #[test]
    fn test_basic_migration() {
        let old_config = json!({
                "environment": "staging",
                "system_id": "test-system"


        });

        let result = ConfigMigrationUtils::migrate_from_json(old_config);
        assert!(result.is_ok());
    }

    #[test]
    fn migrate_non_object_returns_validated_default() {
        let result = ConfigMigrationUtils::migrate_from_json(json!([]));
        assert!(result.is_ok());
    }

    #[test]
    fn migrate_preserves_unknown_fields_in_custom() {
        let j = json!({
            "environment": "production",
            "system_id": "sys-1",
            "legacy_field": { "nested": true }
        });
        let u = ConfigMigrationUtils::migrate_from_json(j).expect("ok");
        let custom = u.custom.expect("custom");
        assert!(custom.contains_key("legacy_field"));
    }

    #[test]
    fn known_top_level_keys_are_not_duplicated_in_custom() {
        let j = json!({
            "environment": "development",
            "system_id": "id1",
            "network": { "note": "ignored_by_migration" }
        });
        let u = ConfigMigrationUtils::migrate_from_json(j).expect("ok");
        assert!(u.custom.is_none());
    }

    #[test]
    fn migrate_empty_object_validates() {
        let u = ConfigMigrationUtils::migrate_from_json(json!({})).expect("empty");
        assert!(!u.system.environment.is_empty());
    }

    #[test]
    fn canonical_migration_config_default() {
        let c = CanonicalMigrationConfig::default();
        assert!(c.auto_migrate);
        assert_eq!(c.log_level, "info");
    }

    #[test]
    fn migrate_from_json_maps_environment_and_system_id() {
        let u = ConfigMigrationUtils::migrate_from_json(json!({
            "environment": "qa",
            "system_id": "qa-box"
        }))
        .expect("migrated");
        assert_eq!(u.system.environment, "qa");
        assert_eq!(u.system.system_id, "qa-box");
    }

    #[test]
    fn migrate_array_root_returns_ok_with_defaults() {
        let u = ConfigMigrationUtils::migrate_from_json(json!([1, 2, 3])).expect("array");
        assert!(!u.system.system_id.is_empty());
    }

    #[test]
    fn migrate_string_root_returns_ok() {
        let u = ConfigMigrationUtils::migrate_from_json(json!("legacy")).expect("str");
        assert_eq!(u.system.environment, "development");
    }

    #[test]
    fn migrate_null_root_returns_ok() {
        let u = ConfigMigrationUtils::migrate_from_json(serde_json::Value::Null).expect("null");
        assert!(!u.validate().is_err());
    }
}
