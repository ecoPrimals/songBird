//! CLI Types and Enums

use clap::ValueEnum;

/// Configuration actions for the config command
#[derive(Debug, Clone)]
pub enum ConfigAction {
    /// Show current configuration
    Show,
    /// Edit configuration interactively
    Edit,
    /// Validate current configuration
    Validate,
    /// Reset configuration to defaults
    Reset { yes: bool },
    /// Export configuration to file
    Export {
        output: Option<String>,
        format: ExportFormat,
    },
}

/// Export format for configuration
#[derive(Debug, Clone, ValueEnum)]
pub enum ExportFormat {
    /// TOML format
    Toml,
    /// JSON format
    Json,
    /// YAML format
    Yaml,
}

impl Default for ExportFormat {
    fn default() -> Self {
        Self::Toml
    }
}
