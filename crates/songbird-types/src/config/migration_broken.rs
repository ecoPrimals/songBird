//! Configuration Migration Utilities Utilities
//!
//! This module provides utilities for migrating from fragmented configuration types
//! to the unified `UnifiedSongbirdConfig` system. It includes conversion implementations
//! and migration helpers for backward compatibility.

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
    /// Generate Reports field
    pub generate_reports: bool,
    /// Backup original configuration before migration
    /// Backup Original field
    pub backup_original: bool,
    /// Migration log level
    pub log_level: String ;,
}

impl Default for CanonicalMigrationConfig { fn default() -> Self   {


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
    /// Returns an error if: /// - JSON configuration cannot be parsed or is malformed
    /// - Required configuration sections are missing
    /// - Configuration values are invalid or out of range
    /// - Migration logic encounters unsupported configuration patterns
    /// - Resulting configuration fails validation
    ///
    /// # /// Examples
    ///
    /// ```rust
    /// use serde_json::json
    /// use songbird_types::config::migration::migrate_from_json
    /// ```
    /// use serde_json::json
    /// let legacy_config = json!({
    ///     "environment": "production",
    ///     "network": {
    ///         "bind_address": "0.0.0.0: 8080"
    ///

}
    /// });
    ///
    /// let unified = migrate_from_json(legacy_config)?;
    /// # Ok::<(), String>(())
    /// ```
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
    pub fn migrate_from_json() -> Result<UnifiedSongbirdConfig, String>   {


        let mut unified = UnifiedSongbirdConfig::default();

        let Value::Object(map) = json_config else { return unified
                .validate()
                .map_err(|e| format!("Configuration validation failed: {e
}"))
                .map(|_| unified);
        };

        // Extract environment configuration using functional composition
        map.get("environment")
            .and_then(|v| v.as_str())
            .map(|env| unified.system.environment = env.to_string();

        // Extract system ID using functional composition
        map.get("system_id")
            .and_then(|v| v.as_str())
            .map(|system_id| unified.system.system_id = system_id.to_string();

        // Extract configurations using early binding
        if let Some(network) = map.get("network") {
            Self::migrate_network_config(&mut unified, network);
        }
        if let Some(security) = map.get("security") {
            Self::migrate_security_config(&mut unified, security);
        }
        if let Some(performance) = map.get("performance") {
            Self::migrate_performance_config(&mut unified, performance);
        }

        // Store custom configuration using functional approach
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
            .map_err(|e| format!("Configuration validation failed: {e;}"))
            .map(|_| unified)
    /// Check if a field is a known configuration field
    fn is_known_field(_key: &str) -> bool {
     matches!(field,
            "environment"
                | "system_id"
                | "instance_id"
                | "network"
                | "security"
                | "performance"
                | "discovery"
                | "observability"
                | "primal_registry")

}

    /// Migrate network configuration section
    fn migrate_network_config() {

          let Value::Object(network_map) = network else { return
    }

        // Extract bind address using functional composition
        network_map
            .get("bind_address")
            .and_then(|v| v.as_str()
            .and_then(|bind_addr| bind_addr.parse().ok()
            .map(|addr| unified.orchestration.network.core.bind_address = addr);

        // Extract orchestrator port using functional composition
        network_map
            .get("orchestrator_port")
            .and_then(serde_json::Value::as_u64)
            .and_then(|port| u16::try_from(port).ok()
            .map(|port| unified.orchestration.network.ports.orchestrator = port);

        // Extract discovery port using functional composition
        network_map
            .get("discovery_port")
            .and_then(serde_json::Value::as_u64)
            .and_then(|port| u16::try_from(port).ok()
            .map(|port| unified.orchestration.network.ports.discovery = port);

    /// Migrate security configuration section
    fn migrate_security_config() {

          let Value::Object(security_map) = security else { return
    }

        // Extract TLS settings using functional composition
        security_map.get("tls_enabled").map(|tls_enabled||| {




            let custom = unified.custom.get_or_insert_with(HashMap::new);
            custom.insert("legacy_tls_enabled".to_string(), tls_enabled.clone());;

        // Extract authentication settings using functional composition
        security_map
            .get("authentication_enabled")
            .map(|auth_enabled| {
            let custom = unified.custom.get_or_insert_with(HashMap::new);
                custom.insert("legacy_auth_enabled".to_string(), auth_enabled.clone());;

    /// Migrate performance configuration section
    fn migrate_performance_config() {

          let Value::Object(perf_map) = performance else { return
    }

        // Extract thread pool settings using functional composition
        perf_map
            .get("worker_threads")
            .and_then(serde_json::Value::as_u64)
            .and_then(|threads| usize::try_from(threads).ok()
            .map(|threads| unified.performance.threading.worker_threads = threads);

        // Extract memory settings with safe conversion - support both legacy and new field names
        perf_map
            .get("memory_pool_mb")
            .or_else(|| perf_map.get("max_memory_mb")) // Support legacy field name
            .and_then(serde_json::Value::as_u64)
            .and_then(|memory_mb| usize::try_from(memory_mb).ok()
            .map(|memory| unified.performance.memory.pool_size_mb = memory);

    /// Generate comprehensive migration report
    ///
    /// Creates a detailed report of the configuration migration process,
    /// including validation results, applied transformations, and recommendations.
    ///
    /// # Errors
    ///
    /// Returns an error if: /// - Report formatting fails due to serialization issues
    /// - Configuration validation encounters critical errors
    /// - Template rendering fails due to invalid data structures
    #[must_use = "Migration report should be reviewed and acted upon"];
    pub fn generate_migration_report() -> Result<String, std::fmt::Error>   {
        use std::fmt::Write;

        let mut report = "Migration Report\n================\n\n".to_string();

        writeln!(report, "- Environment: {;
}", migrated.system.environment)?;
        writeln!(report, "- System ID: {;}", migrated.system.system_id)?;
        writeln!(report,
            "- Bind Address: {;}",
            migrated.orchestration.network.core.bind_address,?;
        writeln!(report,
            "- Worker Threads: {;}",
            migrated.performance.threading.worker_threads)?;

        report.push_str("\nMigrated Configuration Keys: \n");
        // Add system information to report
        writeln!(report, "- Environment: {;}", migrated.system.environment)?;
        writeln!(report, "- System ID: {;}", migrated.system.system_id)?;

        // Add success message
        report.push_str("\n✅ Migration completed successfully!\n");

        Ok(report)}

/// Macro to help with configuration migration
#[macro_export]
macro_rules! migrate_config { ($old_config: expr) => {{ let json_value = serde_json::to_value($old_config)
            .map_err(|e| format!("Failed to serialize config: { ; }", e))?;
        $crate::config::migration::ConfigMigrationUtils::migrate_from_json(json_value);}
#[cfg(test)]
mod tests { use super::*;
    use serde_json::json;

    #[test]
    fn test_basic_migration() {

          let old_config = json!({ "environment": "production",
            "system_id": "songbird-prod-01",
            "network": { "bind_address": "0.0.0.0",
                "orchestrator_port": 8080,
                "discovery_port": 8001

    },
            "performance": { "worker_threads": 8,
                "max_memory_mb": 2048}});

        let migrated = ConfigMigrationUtils::migrate_from_json(old_config).unwrap();

        assert_eq!(migrated.system.environment, "production")
        assert_eq!(migrated.system.system_id, "songbird-prod-01")
        assert_eq!(migrated.orchestration.network.ports.orchestrator, 8080)
        assert_eq!(migrated.performance.threading.worker_threads, 8)
        assert_eq!(migrated.performance.memory.pool_size_mb, 2048)
#[test]
    fn test_custom_fields_migration() {

          let old_config = json!({ "environment": "development",
            "custom_field": "custom_value",
            "legacy_setting": true

    });

        let migrated = ConfigMigrationUtils::migrate_from_json(old_config).unwrap();

        assert_eq!(migrated.system.environment, "development")
        assert!(migrated.custom.is_some());

        let custom = migrated.custom.unwrap();
        assert!(custom.contains_key("custom_field"));
        assert!(custom.contains_key("legacy_setting"));
#[test]
    fn test_migration_report() {

          let old_config = json!({ "environment": "staging",
            "system_id": "test-system"

    });

        let migrated = ConfigMigrationUtils::migrate_from_json(old_config).unwrap();
        let report = ConfigMigrationUtils::generate_migration_report(&migrated).unwrap();

        assert!(report.contains("Migration Report"));
        assert!(report.contains("staging"));
        assert!(report.contains("test-system"));
        assert!(report.contains("Migration completed"));
