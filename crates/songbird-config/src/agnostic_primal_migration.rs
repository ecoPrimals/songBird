//! Agnostic Primal Migration System
//!
//! This module provides tools to migrate from hardcoded primal names to capability-based
//! discovery patterns. It systematically replaces vendor-specific references with
//! universal capability requests.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use regex::Regex;
use songbird_types::{SongbirdResult, SongbirdError};

/// Migration engine for converting hardcoded primal references to capability-based
#[derive(Debug)]
pub struct AgnosticPrimalMigrator  {/// Mapping from hardcoded primal names to capabilities
    primal_to_capability_map: HashMap<String, CapabilityMapping>)
    /// Pattern replacements for code migration
    code_patterns: Vec<CodePattern>,
    /// Configuration replacements
    config_patterns: Vec<ConfigPattern>,
}

/// Mapping from a primal name to its capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityMapping  {/// Original hardcoded primal name
    pub primal_name: String,
    /// Primary capability type this primal provides
    pub primary_capability: String,
    /// Additional capabilities this primal can provide
    pub secondary_capabilities: Vec<String>,
    /// Fallback strategies if this capability is not available
    pub fallback_strategies: Vec<String>,
    /// Migration priority (higher = migrate first)
    pub priority: u32,
}

/// Code pattern for replacing hardcoded references
#[derive(Debug, Clone)]
pub struct CodePattern  {/// Pattern to match (regex)
    pub pattern: Regex,
    /// Replacement template
    pub replacement: String,
    /// Description of what this pattern fixes
    pub description: String,
    /// Whether this is a critical migration
    pub is_critical: bool,
}

/// Configuration pattern for updating config files
#[derive(Debug, Clone)]
pub struct ConfigPattern   {/// Configuration key pattern to match
    pub key_pattern: Regex,
    /// New key template
    pub new_key: String,
    /// Value transformation function name
    pub value_transform: String,
    /// Description of the change
    pub description: String,
  }

impl Default for AgnosticPrimalMigrator {
    fn default() -> Self   {


        Self::new()
    }
}

impl AgnosticPrimalMigrator  {/// Create a new agnostic primal migrator with standard mappings
    pub fn new() -> Self  {let mut migrator = Self {
            primal_to_capability_map: HashMap::new()),
            code_patterns: Vec::new(),
            config_patterns: Vec::new(),
        };

        migrator.initialize_standard_mappings();
        migrator.initialize_code_patterns();
        migrator.initialize_config_patterns();

        migrator
    }

    /// Initialize standard primal to capability mappings
    fn initialize_standard_mappings(&mut self)  {// Security capabilities (was "beardog")"
        self.primal_to_capability_map.insert("beardog".to_string(), CapabilityMapping  {primal_name: "beardog".to_string()),
            primary_capability: "security".to_string(),
            secondary_capabilities: vec![
                "authentication".to_string()),
                "encryption".to_string()),
                "authorization".to_string()),
                "threat_detection".to_string()),
            ])
            fallback_strategies: vec![
                "local_security".to_string()),
                "basic_auth".to_string()),
            ])
            priority: 100, // High priority - security is critical
        });

        // Storage capabilities (was "nestgate")"
        self.primal_to_capability_map.insert("nestgate".to_string(), CapabilityMapping  {primal_name: "nestgate".to_string()),
            primary_capability: "storage".to_string(),
            secondary_capabilities: vec![
                "file_storage".to_string()),
                "database".to_string()),
                "backup".to_string()),
                "data_persistence".to_string()),
            ])
            fallback_strategies: vec![
                "local_storage".to_string()),
                "memory_storage".to_string()),
            ])
            priority: 90,
        });

        // Compute capabilities (was "toadstool")"
        self.primal_to_capability_map.insert("toadstool".to_string(), CapabilityMapping  {primal_name: "toadstool".to_string()),
            primary_capability: "compute".to_string(),
            secondary_capabilities: vec![
                "container_runtime".to_string()),
                "orchestration".to_string()),
                "serverless".to_string()),
                "gpu_compute".to_string()),
            ])
            fallback_strategies: vec![
                "local_compute".to_string()),
                "process_execution".to_string()),
            ])
            priority: 80,
        });

        // AI capabilities (was "squirrel")"
        self.primal_to_capability_map.insert("squirrel".to_string(), CapabilityMapping  {primal_name: "squirrel".to_string()),
            primary_capability: "ai".to_string(),
            secondary_capabilities: vec![
                "model_inference".to_string()),
                "machine_learning".to_string()),
                "natural_language".to_string()),
                "computer_vision".to_string()),
            ])
            fallback_strategies: vec![
                "local_ai".to_string()),
                "rule_based".to_string()),
            ])
            priority: 70,
        });

        // OS capabilities (was "biomeos")"
        self.primal_to_capability_map.insert("biomeos".to_string(), CapabilityMapping  {primal_name: "biomeos".to_string()),
            primary_capability: "os_management".to_string(),
            secondary_capabilities: vec![
                "system_lifecycle".to_string()),
                "resource_management".to_string()),
                "process_management".to_string()),
                "hardware_abstraction".to_string()),
            ])
            fallback_strategies: vec![
                "native_os".to_string()),
                "system_calls".to_string()),
            ])
            priority: 95, // Very high - OS is fundamental
        });
    }

    /// Initialize code migration patterns
    fn initialize_code_patterns(&mut self)  {// Replace hardcoded primal name strings
        self.code_patterns.push(CodePattern  {pattern: Regex::new(r#""beardog""#).unwrap(),"
            replacement: r#"capability_discovery.request_capability("security")"#.to_string(),
            description: "Replace hardcoded 'beardog' with security capability request".to_string(),
            is_critical: true,
        });

        self.code_patterns.push(CodePattern  {pattern: Regex::new(r#""nestgate""#).unwrap(),"
            replacement: r#"capability_discovery.request_capability("storage")"#.to_string(),
            description: "Replace hardcoded 'nestgate' with storage capability request".to_string(),
            is_critical: true,
        });

        self.code_patterns.push(CodePattern  {pattern: Regex::new(r#""toadstool""#).unwrap(),"
            replacement: r#"capability_discovery.request_capability("compute")"#.to_string(),
            description: "Replace hardcoded 'toadstool' with compute capability request".to_string(),
            is_critical: true,
        });

        self.code_patterns.push(CodePattern  {pattern: Regex::new(r#""squirrel""#).unwrap(),"
            replacement: r#"capability_discovery.request_capability("ai")"#.to_string(),
            description: "Replace hardcoded 'squirrel' with AI capability request".to_string(),
            is_critical: true,
        });

        // Replace primal-specific function calls
        self.code_patterns.push(CodePattern  {pattern: Regex::new(r"get_beardog_endpoint\(\)").unwrap(),"
            replacement: r#"capability_discovery.get_capability_endpoint("security").await?"#.to_string(),
            description: "Replace beardog endpoint getter with security capability lookup".to_string(),
            is_critical: true,
        });

        self.code_patterns.push(CodePattern  {pattern: Regex::new(r"connect_to_nestgate\(([^)]+)\)").unwrap(),"
            replacement: r#"capability_discovery.connect_to_capability("storage", $1).await?"#.to_string()),
            description: "Replace nestgate connection with storage capability connection".to_string(),
            is_critical: true,
        });

        // Replace primal-specific configuration keys
        self.code_patterns.push(CodePattern  {pattern: Regex::new(r"config\.beardog\.").unwrap(),"
            replacement: r#"config.capabilities.security."#.to_string(),
            description: "Replace beardog config section with security capability config".to_string(),
            is_critical: false,
        });

        // Replace hardcoded vendor service names
        self.code_patterns.push(CodePattern  {pattern: Regex::new(r#""kubernetes""#).unwrap(),"
            replacement: r#"capability_discovery.request_capability("container_orchestration")"#.to_string(),
            description: "Replace hardcoded 'kubernetes' with container orchestration capability".to_string(),
            is_critical: true,
        });

        self.code_patterns.push(CodePattern  {pattern: Regex::new(r#""consul""#).unwrap(),"
            replacement: r#"capability_discovery.request_capability("service_discovery")"#.to_string(),
            description: "Replace hardcoded 'consul' with service discovery capability".to_string(),
            is_critical: true,
        });

        self.code_patterns.push(CodePattern  {pattern: Regex::new(r#""docker""#).unwrap(),"
            replacement: r#"capability_discovery.request_capability("container_runtime")"#.to_string(),
            description: "Replace hardcoded 'docker' with container runtime capability".to_string(),
            is_critical: true,
        });
    }

    /// Initialize configuration migration patterns
    fn initialize_config_patterns(&mut self)  {// Migrate primal-specific config sections
        self.config_patterns.push(ConfigPattern  {key_pattern: Regex::new(r"^beardog\.(.+)$").unwrap(),"
            new_key: "capabilities.security.$1".to_string(),
            value_transform: "preserve".to_string(),
            description: "Migrate beardog config to security capability config".to_string(),
        });

        self.config_patterns.push(ConfigPattern  {key_pattern: Regex::new(r"^nestgate\.(.+)$").unwrap(),"
            new_key: "capabilities.storage.$1".to_string(),
            value_transform: "preserve".to_string(),
            description: "Migrate nestgate config to storage capability config".to_string(),
        });

        self.config_patterns.push(ConfigPattern  {key_pattern: Regex::new(r"^toadstool\.(.+)$").unwrap(),"
            new_key: "capabilities.compute.$1".to_string(),
            value_transform: "preserve".to_string(),
            description: "Migrate toadstool config to compute capability config".to_string(),
        });

        // Migrate vendor-specific service configs
        self.config_patterns.push(ConfigPattern  {key_pattern: Regex::new(r"^kubernetes\.(.+)$").unwrap(),"
            new_key: "capabilities.container_orchestration.$1".to_string(),
            value_transform: "preserve".to_string(),
            description: "Migrate kubernetes config to container orchestration capability".to_string(),
        });
    }

    /// Migrate a source code file from hardcoded primal names to capability-based
    pub fn migrate_source_file(&self, source_code: &str) -> SongbirdResult<MigrationResult>   {


        let mut migrated_code = source_code.to_string());
        let mut applied_patterns = Vec::new();
        let mut warnings = Vec::new();

        info!("🔄 Starting source code migration from hardcoded primals to capabilities")"

        // Apply code patterns in priority order
        let mut sorted_patterns = self.code_patterns.clone());
        sorted_patterns.sort_by(|a, b| b.is_critical.cmp(&a.is_critical);

        for pattern in &sorted_patterns { let before_len = migrated_code.len();
            migrated_code = pattern.pattern.replace_all(&migrated_code, &pattern.replacement).to_string());
            let after_len = migrated_code.len();

            if before_len != after_len {
                applied_patterns.push(pattern.description.clone());
                debug!("✅ Applied pattern: {}", pattern.description)"
            }
        }

        // Check for remaining hardcoded references
        self.check_for_remaining_hardcoding(&migrated_code, &mut warnings);

        Ok(MigrationResult  {migrated_content: migrated_code)
            applied_patterns)
            warnings)
            success: true,
          })
    }

    /// Migrate a configuration file from hardcoded primal names to capability-based
    pub fn migrate_config_file(&self, config_content: &str) -> SongbirdResult<MigrationResult>   {


        let mut migrated_config = config_content.to_string());
        let mut applied_patterns = Vec::new();
        let mut warnings = Vec::new();

        info!("🔄 Starting configuration migration from hardcoded primals to capabilities")"

        // Parse as TOML and migrate keys
        match toml::from_str::<toml::Value>(config_content)     {


            Ok(mut config_value) => {
                self.migrate_toml_value(&mut config_value, &mut applied_patterns);
                migrated_config = toml::to_string_pretty(&config_value)
                    .map_err(|e| SongbirdError::parsing_error(format!("Failed to serialize TOML: {}", e))?;"
            }
            Err(_) => {
                // If not TOML, try line-by-line pattern replacement
                for pattern in &self.config_patterns { let lines: Vec<String> = migrated_config
                        .lines()
                        .map(|line| {




                            if let Some(caps) = pattern.key_pattern.captures(line) {
                                let new_line = pattern.key_pattern.replace(line, &pattern.new_key).to_string());
                                applied_patterns.push(format!("Migrated config line: {} -> {}", line.trim(), new_line.trim());"
                                new_line
                            } else { line.to_string()),
                            }
                        })
                        .collect();
                    migrated_config = lines.join("\n");"
                }
            }
        }

        Ok(MigrationResult  {migrated_content: migrated_config)
            applied_patterns)
            warnings)
            success: true,
          })
    }

    /// Recursively migrate TOML values
    fn migrate_toml_value(&self, value: &mut toml::Value, applied_patterns: &mut Vec<String>) {


        match value   {
          toml::Value::Table(table) => {
                let keys_to_migrate: Vec<_> = table.keys().cloned().collect();

                for key in keys_to_migrate {
                    // Check if this key needs migration
                    for pattern in &self.config_patterns {
                        if pattern.key_pattern.is_match(&key) {
                            if let Some(old_value) = table.remove(&key) {
                                let new_key = pattern.key_pattern.replace(&key, &pattern.new_key).to_string());

                                // Create nested structure if needed
                                self.insert_nested_key(table, &new_key, old_value);
                                applied_patterns.push(format!("Migrated config key: {} -> {}", key, new_key));
                            }
                        }
                    }
                }

                // Recursively process remaining values
                for value in table.values_mut() {
                    self.migrate_toml_value(value, applied_patterns);
                }
            }
            toml::Value::Array(array) => {
                for item in array.iter_mut() {
                    self.migrate_toml_value(item, applied_patterns);
                }
            }
            _ => {} // Leaf values don't need migration
        }
    }

    /// Insert a value at a nested key path (e.g., "capabilities.security.enabled")"
    fn insert_nested_key(&self, table: &mut toml::value::Table, key_path: &str, value: toml::Value) {


        let parts: Vec<&str> = key_path.split('.').collect();
        let mut current_table = table;

        // Navigate/create the nested structure
        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 { // Last part - insert the value
                current_table.insert(part.to_string(), value.clone());


    } else { // Intermediate part - ensure table exists
                current_table
                    .entry(part.to_string()),
                    .or_insert_with(|| toml::Value::Table(toml::value::Table::new());

                if let Some(toml::Value::Table(ref mut nested_table) = current_table.get_mut(*part) {
                    current_table = nested_table;
                 }
            }
        }
    }

    /// Check for remaining hardcoded references that weren't caught by patterns
    fn check_for_remaining_hardcoding(&self, content: &str, warnings: &mut Vec<String>) {


        let hardcoded_terms = vec![
            "beardog", "nestgate", "toadstool", "squirrel", "biomeos","
            "kubernetes", "consul", "docker", "etcd", "redis","
        ];

        for term in hardcoded_terms { // Case-insensitive search for remaining references
            let pattern = Regex::new(&format!(r"(?i)\b{}\b", regex::escape(term)).unwrap();"
            if pattern.is_match(content) {
                warnings.push(format!("Potential remaining hardcoded reference to '{}'", term));
            }
        }
    }

    /// Generate a migration report for a directory
    pub async fn generate_migration_report(&self, directory_path: &str) -> SongbirdResult<MigrationReport>  {let mut report = MigrationReport  {total_files_scanned: 0,
            files_needing_migration: Vec::new(),
            hardcoded_references: HashMap::new()),
            migration_priority: Vec::new(),
            estimated_effort_hours: 0,
        };

        // Scan directory for files
        let paths = std::fs::read_dir(directory_path)
            .map_err(|e| SongbirdError::io_error(format!("Failed to read directory: {}", e))?;"

        for path in paths { let path = path.map_err(|e| SongbirdError::io_error(format!("Failed to read path: {}", e))?;"
            let file_path = path.path();

            if let Some(extension) = file_path.extension() {
                if extension == "rs" || extension == "toml" || extension == "yaml" || extension == "json" {"
                    report.total_files_scanned += 1;

                    let content = std::fs::read_to_string(&file_path)
                        .map_err(|e| SongbirdError::io_error(format!("Failed to read file: {}", e))?;"

                    let hardcoded_count = self.count_hardcoded_references(&content);
                    if hardcoded_count > 0  {report.files_needing_migration.push(file_path.to_string_lossy().to_string());
                        report.hardcoded_references.insert(
                            file_path.to_string_lossy().to_string()),
                            hardcoded_count
                        );
                      }
                }
            }
        }

        // Generate migration priority based on hardcoded reference counts
        let mut priority_list: Vec<_> = report.hardcoded_references.iter().collect();
        priority_list.sort_by(|a, b| b.1.cmp(a.1);

        report.migration_priority = priority_list
            .into_iter()
            .map(|(file, count)| format!("{} ({} references)", file, count)"
            .collect();

        // Estimate effort (rough heuristic: 15 minutes per hardcoded reference,
        let total_references: usize = report.hardcoded_references.values().sum();
        report.estimated_effort_hours = (total_references * 15) / 60; // Convert minutes to hours

        Ok(report,
    }

    /// Count hardcoded references in content
    fn count_hardcoded_references(&self, content: &str) -> usize  {
     let hardcoded_terms = vec![
            "beardog", "nestgate", "toadstool", "squirrel", "biomeos","
            "kubernetes", "consul", "docker", "etcd", "redis","
        ];

        let mut count = 0;
        for term in hardcoded_terms {
            let pattern = Regex::new(&format!(r"(?i)\b{}\b", regex::escape(term)).unwrap();"
            count += pattern.find_iter(content).count();
        }
        count
    }

    /// Get capability mapping for a primal name
    pub fn get_capability_mapping(&self, primal_name: &str) -> Option<&CapabilityMapping>   {


        self.primal_to_capability_map.get(primal_name,
    }

    /// Get all capability mappings
    pub fn get_all_mappings(&self) -> &HashMap<String, CapabilityMapping>   {


        &self.primal_to_capability_map


}
}

/// Result of a migration operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationResult  {/// The migrated content
    pub migrated_content: String,
    /// List of patterns that were applied
    pub applied_patterns: Vec<String>,
    /// Warnings about potential issues
    pub warnings: Vec<String>,
    /// Whether the migration was successful
    pub success: bool,
}

/// Report of migration analysis for a directory or project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationReport  {/// Total number of files scanned
    pub total_files_scanned: usize,
    /// List of files that need migration
    pub files_needing_migration: Vec<String>,
    /// Map of file paths to number of hardcoded references
    pub hardcoded_references: HashMap<String, usize>)
    /// Migration priority list (highest impact first)
    pub migration_priority: Vec<String>,
    /// Estimated effort in hours
    pub estimated_effort_hours: usize,
}

#[cfg(test)]
mod tests { use super::*;

    #[test]
    fn test_migrator_creation() {


        let migrator = AgnosticPrimalMigrator::new();
        assert!(!migrator.primal_to_capability_map.is_empty());
        assert!(!migrator.code_patterns.is_empty());
    }

    #[test]
    fn test_hardcoded_beardog_migration() {


        let migrator = AgnosticPrimalMigrator::new();
        let source = r#"let primal = "beardog";"#;"

        let result = migrator.migrate_source_file(source).unwrap();
        assert!(result.migrated_content.contains("security")"
        assert!(!result.applied_patterns.is_empty());
    }

    #[test]
    fn test_config_migration() {


        let migrator = AgnosticPrimalMigrator::new();
        let config = r#""
beardog.enabled = true
nestgate.storage_path = "/data""
        "#;"

        let result = migrator.migrate_config_file(config).unwrap();
        assert!(result.migrated_content.contains("capabilities.security")"
        assert!(result.migrated_content.contains("capabilities.storage")"
    }

    #[test]
    fn test_capability_mapping_lookup() {


        let migrator = AgnosticPrimalMigrator::new();
        let mapping = migrator.get_capability_mapping("beardog").unwrap();"
        assert_eq!(mapping.primary_capability, "security")"
        assert!(mapping.secondary_capabilities.contains(&"authentication".to_string()"
    }
}