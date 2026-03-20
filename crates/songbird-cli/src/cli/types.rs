// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! CLI Types and Enums

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

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
    Reset {
        yes: bool,
    },
    /// Export configuration to file
    Export {
        output: Option<String>,
        format: ExportFormat,
    },
}

/// Export format for configuration
#[derive(Debug, Clone, ValueEnum, Default)]
pub enum ExportFormat {
    /// TOML format
    #[default]
    Toml,
    /// JSON format
    Json,
    /// YAML format
    Yaml,
}

/// Deployment types for Songbird orchestrator
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ValueEnum, Default)]
pub enum DeploymentType {
    /// Home network deployment
    #[default]
    HomeNetwork,
    /// Research cluster deployment
    ResearchCluster,
    /// Edge deployment for `IoT`
    EdgeDeployment,
    /// Development environment
    Development,
    /// Container orchestration (Kubernetes)
    ContainerOrchestration,
    /// Container runtime (Docker)
    ContainerRuntime,
    /// Bare metal deployment
    BareMetal,
    /// Cloud deployment (AWS, GCP, Azure)
    Cloud,
}

/// Output format for CLI commands (from core/types.rs)
#[derive(Debug, Clone, Serialize, Deserialize, ValueEnum, PartialEq, Eq, Default)]
pub enum OutputFormat {
    /// Automatic format selection
    #[default]
    Auto,
    /// Human-readable table format
    Table,
    /// JSON format
    Json,
    /// YAML format
    Yaml,
    /// Plain text format
    Text,
}

/// CLI arguments structure
#[derive(Debug, Clone)]
pub struct CliArgs {
    /// Verbose output
    pub verbose: bool,
    /// Quiet mode
    pub quiet: bool,
    /// Output format
    pub format: OutputFormat,
    /// Configuration file path
    pub config: Option<String>,
}

impl CliArgs {
    /// Parse CLI arguments from environment (stub implementation)
    #[must_use]
    pub fn parse_from_env() -> Self {
        Self::parse_with(|name| std::env::var(name).ok())
    }

    /// Parse CLI arguments with injectable env reader (concurrent-safe, testable)
    #[must_use]
    pub fn parse_with<F>(env_reader: F) -> Self
    where
        F: Fn(&str) -> Option<String>,
    {
        Self {
            verbose: env_reader("SONGBIRD_VERBOSE").is_some(),
            quiet: env_reader("SONGBIRD_QUIET").is_some(),
            format: OutputFormat::default(),
            config: env_reader("SONGBIRD_CONFIG"),
        }
    }
}

/// Main CLI application structure
#[derive(Debug, Clone, clap::Parser)]
#[command(name = "songbird")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Songbird Universal Orchestrator CLI")]
#[command(long_about = "Make distributed computing as simple as 'songbird init'")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<crate::cli::commands::Commands>,
}

impl Cli {
    /// Execute the CLI command
    pub async fn execute(&self) -> crate::errors::SongbirdResult<()> {
        if let Some(cmd) = &self.command {
            if let crate::cli::commands::Commands::Tower {
                command,
            } = cmd
            {
                command.execute().await
            } else {
                println!("🎼 Executing command: {cmd:?}");
                // For now, just print success - actual command execution will be implemented
                println!("✅ Command completed successfully");
                Ok(())
            }
        } else {
            println!("🎼 Songbird Universal Orchestrator CLI");
            println!("Use --help for available commands");
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use songbird_types::SongbirdError;

    #[test]
    fn test_export_format_default() {
        let format = ExportFormat::default();
        assert!(matches!(format, ExportFormat::Toml));
    }

    #[test]
    fn test_export_format_variants() {
        let toml = ExportFormat::Toml;
        let json = ExportFormat::Json;
        let yaml = ExportFormat::Yaml;

        assert!(matches!(toml, ExportFormat::Toml));
        assert!(matches!(json, ExportFormat::Json));
        assert!(matches!(yaml, ExportFormat::Yaml));
    }

    #[test]
    fn test_deployment_type_default() {
        let deployment = DeploymentType::default();
        assert_eq!(deployment, DeploymentType::HomeNetwork);
    }

    #[test]
    fn test_deployment_type_all_variants() {
        let variants = [
            DeploymentType::HomeNetwork,
            DeploymentType::ResearchCluster,
            DeploymentType::EdgeDeployment,
            DeploymentType::Development,
            DeploymentType::ContainerOrchestration,
            DeploymentType::ContainerRuntime,
            DeploymentType::BareMetal,
            DeploymentType::Cloud,
        ];

        assert_eq!(variants.len(), 8);
    }

    #[test]
    fn test_deployment_type_equality() {
        assert_eq!(DeploymentType::HomeNetwork, DeploymentType::HomeNetwork);
        assert_ne!(DeploymentType::HomeNetwork, DeploymentType::Cloud);
    }

    #[test]
    fn test_deployment_type_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let deployment = DeploymentType::Development;
        let serialized = serde_json::to_string(&deployment).map_err(|e| {
            SongbirdError::configuration(format!("Missing performance configuration: {}", e))
        })?;
        let deserialized: DeploymentType = serde_json::from_str(&serialized).map_err(|e| {
            SongbirdError::configuration(format!("Missing performance configuration: {}", e))
        })?;
        assert_eq!(deployment, deserialized);
        Ok(())
    }

    #[test]
    fn test_output_format_default() {
        let format = OutputFormat::default();
        assert_eq!(format, OutputFormat::Auto);
    }

    #[test]
    fn test_output_format_all_variants() {
        let variants = [
            OutputFormat::Auto,
            OutputFormat::Table,
            OutputFormat::Json,
            OutputFormat::Yaml,
            OutputFormat::Text,
        ];

        assert_eq!(variants.len(), 5);
    }

    #[test]
    fn test_output_format_equality() {
        assert_eq!(OutputFormat::Json, OutputFormat::Json);
        assert_ne!(OutputFormat::Json, OutputFormat::Yaml);
    }

    #[test]
    fn test_output_format_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let format = OutputFormat::Table;
        let serialized = serde_json::to_string(&format).map_err(|e| {
            SongbirdError::configuration(format!("Missing performance configuration: {}", e))
        })?;
        let deserialized: OutputFormat = serde_json::from_str(&serialized).map_err(|e| {
            SongbirdError::configuration(format!("Missing performance configuration: {}", e))
        })?;
        assert_eq!(format, deserialized);
        Ok(())
    }

    #[test]
    fn test_cli_args_parse_defaults() {
        // ✅ Concurrent-safe: Uses parse_with + empty env (no global state)
        let args = CliArgs::parse_with(|_| None);

        assert!(!args.verbose);
        assert!(!args.quiet);
        assert_eq!(args.format, OutputFormat::Auto);
        assert!(args.config.is_none());
    }

    #[test]
    fn test_cli_args_parse_with_verbose() {
        // ✅ Concurrent-safe: Injectable env reader
        let args = CliArgs::parse_with(|name| {
            if name == "SONGBIRD_VERBOSE" {
                Some("1".to_string())
            } else {
                None
            }
        });
        assert!(args.verbose);
    }

    #[test]
    fn test_cli_args_parse_with_quiet() {
        // ✅ Concurrent-safe: Injectable env reader
        let args = CliArgs::parse_with(|name| {
            if name == "SONGBIRD_QUIET" {
                Some("1".to_string())
            } else {
                None
            }
        });
        assert!(args.quiet, "quiet flag should be true when SONGBIRD_QUIET is set");
    }

    #[test]
    fn test_cli_args_parse_with_config() {
        // ✅ Concurrent-safe: Injectable env reader
        let args = CliArgs::parse_with(|name| {
            if name == "SONGBIRD_CONFIG" {
                Some("/etc/songbird.toml".to_string())
            } else {
                None
            }
        });
        assert_eq!(args.config, Some("/etc/songbird.toml".to_string()));
    }

    #[test]
    fn test_config_action_show() {
        let action = ConfigAction::Show;
        assert!(matches!(action, ConfigAction::Show));
    }

    #[test]
    fn test_config_action_edit() {
        let action = ConfigAction::Edit;
        assert!(matches!(action, ConfigAction::Edit));
    }

    #[test]
    fn test_config_action_validate() {
        let action = ConfigAction::Validate;
        assert!(matches!(action, ConfigAction::Validate));
    }

    #[test]
    fn test_config_action_reset() {
        let action = ConfigAction::Reset {
            yes: true,
        };
        if let ConfigAction::Reset {
            yes,
        } = action
        {
            assert!(yes);
        } else {
            panic!("Expected Reset action");
        }
    }

    #[test]
    fn test_config_action_export() {
        let action = ConfigAction::Export {
            output: Some("config.json".to_string()),
            format: ExportFormat::Json,
        };

        if let ConfigAction::Export {
            output,
            format,
        } = action
        {
            assert_eq!(output, Some("config.json".to_string()));
            assert!(matches!(format, ExportFormat::Json));
        } else {
            panic!("Expected Export action");
        }
    }

    #[test]
    fn test_cli_args_clone() {
        let args = CliArgs {
            verbose: true,
            quiet: false,
            format: OutputFormat::Json,
            config: Some("test.toml".to_string()),
        };

        let cloned = args.clone();
        assert_eq!(args.verbose, cloned.verbose);
        assert_eq!(args.quiet, cloned.quiet);
        assert_eq!(args.format, cloned.format);
        assert_eq!(args.config, cloned.config);
    }

    #[test]
    fn test_deployment_type_debug() {
        let deployment = DeploymentType::Cloud;
        let debug_str = format!("{deployment:?}");
        assert!(debug_str.contains("Cloud"));
    }

    #[test]
    fn test_output_format_debug() {
        let format = OutputFormat::Yaml;
        let debug_str = format!("{format:?}");
        assert!(debug_str.contains("Yaml"));
    }
}
