// Module imports
//! Songbird CLI Module
//!
//! Command-line interface for the Songbird Orchestrator
//! Makes distributed computing as simple as `songbird init`

pub mod commands;
pub mod config;
pub mod discovery;
pub mod templates;
pub mod ui;

use clap::{Parser, Subcommand};
use colored::Colorize;
use songbird_errors::SongbirdError;
use std::env;
use std::path::PathBuf;
use thiserror::Error;
use tracing::error;
// CLI module core
use self::commands::Commands;
use serde::{Deserialize, Serialize};

/// Enhanced CLI Error types with actionable suggestions
#[derive(Error, Debug)]
pub enum CliError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {message}")]
    Config {
        message: String,
        field: Option<String>,
        suggestion: Option<String>,
    },

    #[error("Network error: {message}")]
    Network {
        message: String,
        endpoint: Option<String>,
        suggestion: Option<String>,
    },

    #[error("Command error: {message}")]
    Command {
        message: String,
        command: Option<String>,
        suggestion: Option<String>,
    },

    #[error("Authentication error: {message}")]
    Auth {
        message: String,
        user: Option<String>,
        suggestion: Option<String>,
    },

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Gaming error: {message}")]
    Gaming {
        message: String,
        protocol: Option<String>,
        game: Option<String>,
        suggestion: Option<String>,
    },

    #[error("Discovery error: {message}")]
    Discovery {
        message: String,
        service: Option<String>,
        timeout: Option<u64>,
        suggestion: Option<String>,
    },

    #[error("Service error: {message}")]
    Service {
        message: String,
        service: Option<String>,
        status: Option<String>,
        suggestion: Option<String>,
    },

    #[error("Validation error: {message}")]
    Validation {
        message: String,
        field: Option<String>,
        expected: Option<String>,
        suggestion: Option<String>,
    },

    #[error("Songbird orchestrator error: {0}")]
    Orchestrator(#[from] SongbirdError),

    #[error("Execution error: {message}")]
    ExecutionError {
        message: String,
        command: Option<String>,
        exit_code: Option<i32>,
        suggestion: Option<String>,
    },

    #[error("User cancelled operation")]
    UserCancelled,

    #[error("Resource not found: {message}")]
    ResourceNotFound {
        message: String,
        resource: Option<String>,
        searched_paths: Option<Vec<String>>,
        suggestion: Option<String>,
    },

    #[error("Permission denied: {message}")]
    PermissionDenied {
        message: String,
        resource: Option<String>,
        suggestion: Option<String>,
    },

    #[error("Timeout error: {message}")]
    Timeout {
        message: String,
        operation: Option<String>,
        duration: Option<u64>,
        suggestion: Option<String>,
    },
}

impl CliError {
    /// Create a configuration error with suggestion
    pub fn config_error(message: &str, field: Option<&str>, suggestion: &str) -> Self {
        Self::Config {
            message: message.to_string(),
            field: field.map(|f| f.to_string()),
            suggestion: Some(suggestion.to_string()),
        }
    }

    /// Create a network error with endpoint and suggestion
    pub fn network_error(message: &str, endpoint: Option<&str>, suggestion: &str) -> Self {
        Self::Network {
            message: message.to_string(),
            endpoint: endpoint.map(|e| e.to_string()),
            suggestion: Some(suggestion.to_string()),
        }
    }

    /// Create a command error with suggestion
    pub fn command_error(message: &str, command: Option<&str>, suggestion: &str) -> Self {
        Self::Command {
            message: message.to_string(),
            command: command.map(|c| c.to_string()),
            suggestion: Some(suggestion.to_string()),
        }
    }

    /// Create an execution error with details
    pub fn execution_error(
        message: &str,
        command: Option<&str>,
        exit_code: Option<i32>,
        suggestion: &str,
    ) -> Self {
        Self::ExecutionError {
            message: message.to_string(),
            command: command.map(|c| c.to_string()),
            exit_code,
            suggestion: Some(suggestion.to_string()),
        }
    }

    /// Create a validation error with context
    pub fn validation_error(
        message: &str,
        field: Option<&str>,
        expected: Option<&str>,
        suggestion: &str,
    ) -> Self {
        Self::Validation {
            message: message.to_string(),
            field: field.map(|f| f.to_string()),
            expected: expected.map(|e| e.to_string()),
            suggestion: Some(suggestion.to_string()),
        }
    }

    /// Create a resource not found error
    pub fn resource_not_found(
        message: &str,
        resource: Option<&str>,
        searched_paths: Option<Vec<String>>,
        suggestion: &str,
    ) -> Self {
        Self::ResourceNotFound {
            message: message.to_string(),
            resource: resource.map(|r| r.to_string()),
            searched_paths,
            suggestion: Some(suggestion.to_string()),
        }
    }

    /// Create a timeout error
    pub fn timeout_error(
        message: &str,
        operation: Option<&str>,
        duration: Option<u64>,
        suggestion: &str,
    ) -> Self {
        Self::Timeout {
            message: message.to_string(),
            operation: operation.map(|o| o.to_string()),
            duration,
            suggestion: Some(suggestion.to_string()),
        }
    }

    /// Get the suggestion for recovery, if available
    pub fn get_suggestion(&self) -> Option<&str> {
        match self {
            Self::Config { suggestion, .. }
            | Self::Network { suggestion, .. }
            | Self::Command { suggestion, .. }
            | Self::Auth { suggestion, .. }
            | Self::Gaming { suggestion, .. }
            | Self::Discovery { suggestion, .. }
            | Self::Service { suggestion, .. }
            | Self::Validation { suggestion, .. }
            | Self::ExecutionError { suggestion, .. }
            | Self::ResourceNotFound { suggestion, .. }
            | Self::PermissionDenied { suggestion, .. }
            | Self::Timeout { suggestion, .. } => suggestion.as_deref(),
            Self::Orchestrator(err) => err.get_suggestion(),
            _ => None,
        }
    }

    /// Get the severity level of the error
    pub fn get_severity(&self) -> &str {
        match self {
            Self::Config { .. } | Self::Validation { .. } => "high",
            Self::Auth { .. } | Self::PermissionDenied { .. } => "high",
            Self::Network { .. } | Self::Service { .. } => "medium",
            Self::Timeout { .. } => "medium",
            Self::ResourceNotFound { .. } => "low",
            Self::UserCancelled => "low",
            Self::Orchestrator(err) => err.get_severity(),
            _ => "medium",
        }
    }

    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::Config { .. } | Self::Validation { .. } => false,
            Self::Auth { .. } | Self::PermissionDenied { .. } => false,
            Self::UserCancelled => false,
            Self::Orchestrator(err) => err.is_recoverable(),
            _ => true,
        }
    }

    /// Display error with enhanced formatting
    pub fn display_enhanced(&self) -> String {
        let severity_icon = match self.get_severity() {
            "high" => "🔴",
            "medium" => "🟡",
            "low" => "🟢",
            _ => "⚪",
        };

        let mut output = format!("{severity_icon} {self}");

        if let Some(suggestion) = self.get_suggestion() {
            output.push_str(&format!("\n💡 Suggestion: {suggestion}"));
        }

        if !self.is_recoverable() {
            output.push_str("\n⚠️  This error requires manual intervention to resolve.");
        }

        output
    }
}

/// CLI result type
pub type CliResult<T> = std::result::Result<T, CliError>;

/// Main CLI struct with enhanced help text
#[derive(Parser)]
#[command(
    name = "songbird",
    about = "🎼 Songbird Orchestrator - Distributed Computing Made Simple",
    long_about = "Songbird Orchestrator enables easy distributed computing across networks.\n\
                  Designed for students, researchers, and developers.\n\n\
                  Quick Start:\n\
                  • songbird quick                 - Auto-setup and join network\n\
                  • songbird init                  - Interactive setup wizard\n\
                  • songbird status                - Check system status\n\
                  • songbird --help                - Show all commands\n\n\
                  For more information, visit: https://github.com/ecoPrimals/songbird",
    version = env!("CARGO_PKG_VERSION"),
    author = "ecoPrimals <contact@ecoprimals.dev>",
    help_template = "\
{before-help}{name} {version}
{author-with-newline}{about-with-newline}
{usage-heading}
    {usage}

{all-args}{after-help}
"
)]
pub struct Cli {
    /// Enable verbose output for debugging
    #[arg(
        short,
        long,
        global = true,
        help = "Enable verbose output for debugging"
    )]
    pub verbose: bool,

    /// Suppress all output except errors
    #[arg(short, long, global = true, help = "Suppress all output except errors")]
    pub quiet: bool,

    /// Output format for commands that support it
    #[arg(
        long,
        global = true,
        value_enum,
        default_value = "auto",
        help = "Output format (auto, table, json, yaml, text)"
    )]
    pub output: OutputFormat,

    /// Configuration file path
    #[arg(
        short = 'c',
        long = "config",
        global = true,
        help = "Path to configuration file"
    )]
    pub config: Option<PathBuf>,

    /// Override default data directory
    #[arg(
        long = "data-dir",
        global = true,
        help = "Override default data directory"
    )]
    pub data_dir: Option<String>,

    /// Enable colored output (default: auto-detect)
    #[arg(
        long = "color",
        global = true,
        value_enum,
        default_value = "auto",
        help = "When to use colored output"
    )]
    pub color: ColorMode,

    /// Subcommands
    #[command(subcommand)]
    pub command: commands::Commands,
}

/// Color mode for output
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

/// Enhanced deployment types with better descriptions
#[derive(clap::ValueEnum, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum DeploymentType {
    #[value(
        name = "home-network",
        help = "Home network deployment for personal use"
    )]
    HomeNetwork,
    #[value(name = "research-cluster", help = "Research cluster for academic use")]
    ResearchCluster,
    #[value(
        name = "edge-deployment",
        help = "Edge deployment for distributed systems"
    )]
    EdgeDeployment,
    #[value(name = "development", help = "Development environment")]
    Development,
}

/// Configuration actions with enhanced descriptions
#[derive(Debug, Subcommand)]
pub enum ConfigAction {
    /// Show current configuration
    Show,
    /// Edit configuration interactively
    Edit,
    /// Validate configuration for errors
    Validate,
    /// Reset configuration to defaults
    Reset {
        /// Skip confirmation prompt
        #[arg(short = 'y', long, help = "Skip confirmation prompt")]
        yes: bool,
    },
    /// Export configuration to file
    Export {
        /// Output file path
        #[arg(short = 'o', long, help = "Output file path")]
        output: Option<String>,
        /// Export format
        #[arg(long, value_enum, default_value = "toml", help = "Export format")]
        format: ExportFormat,
    },
}

/// Export formats with descriptions
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum ExportFormat {
    #[value(help = "TOML format")]
    Toml,
    #[value(help = "JSON format")]
    Json,
    #[value(help = "YAML format")]
    Yaml,
}

/// Output formats with enhanced descriptions
#[derive(Debug, Clone, clap::ValueEnum, Serialize, Deserialize)]
pub enum OutputFormat {
    /// Automatically detect best format based on terminal
    Auto,
    /// Human-readable table format
    Table,
    /// JSON output for programmatic use
    Json,
    /// YAML output for configuration
    Yaml,
    /// Simple text format
    Text,
}

impl Cli {
    /// Execute the CLI command with enhanced error handling
    pub async fn execute(self) -> CliResult<()> {
        // Configure colored output
        colored::control::set_override(match self.color {
            ColorMode::Always => true,
            ColorMode::Never => false,
            ColorMode::Auto => atty::is(atty::Stream::Stdout),
        });

        // Set up logging level based on verbosity
        if !self.quiet {
            let level = if self.verbose { "debug" } else { "info" };
            std::env::set_var("RUST_LOG", format!("songbird={level}"));
        }

        // Execute the command with enhanced error handling
        match self.command {
            Commands::Version { detailed } => commands::version::show_version(detailed)
                .await
                .map_err(|e| {
                    CliError::command_error(
                        &e.to_string(),
                        Some("version"),
                        "Check if the application is properly installed",
                    )
                }),
            Commands::Quick { contribute, name } => {
                commands::quick::execute_quick(contribute, name)
                    .await
                    .map_err(|e| {
                        CliError::command_error(
                            &e.to_string(),
                            Some("quick"),
                            "Try 'songbird init' for step-by-step setup",
                        )
                    })
            }
            Commands::Share { resource, percent } => {
                commands::share::execute_share(resource, percent)
                    .await
                    .map_err(|e| {
                        CliError::command_error(
                            &e.to_string(),
                            Some("share"),
                            "Check system resources and network connectivity",
                        )
                    })
            }
            Commands::Init {
                deployment,
                quick,
                output_dir,
            } => commands::init::execute_init(deployment, quick, output_dir)
                .await
                .map_err(|e| {
                    CliError::command_error(
                        &e.to_string(),
                        Some("init"),
                        "Check write permissions in the target directory",
                    )
                }),
            Commands::Start {
                config,
                dashboard,
                port,
            } => commands::orchestrator::start_orchestrator(config.as_deref(), dashboard, port)
                .await
                .map_err(|e| {
                    CliError::command_error(
                        &e.to_string(),
                        Some("start"),
                        "Check configuration file and port availability",
                    )
                }),
            Commands::Stop { force } => commands::orchestrator::stop_orchestrator(force)
                .await
                .map_err(|e| {
                    CliError::command_error(
                        &e.to_string(),
                        Some("stop"),
                        "Try 'songbird stop --force' if normal shutdown fails",
                    )
                }),
            Commands::Status {
                detailed,
                watch,
                format,
            } => commands::status::show_status(detailed, watch, format)
                .await
                .map_err(|e| {
                    CliError::command_error(
                        &e.to_string(),
                        Some("status"),
                        "Check if the orchestrator is running",
                    )
                }),
            Commands::Logs {
                service,
                follow,
                lines,
                level,
            } => commands::logs::show_logs(service.as_deref(), follow, lines, level)
                .await
                .map_err(|e| {
                    CliError::command_error(
                        &e.to_string(),
                        Some("logs"),
                        "Check log file permissions and service name",
                    )
                }),
            Commands::Internet { command } => {
                crate::cli::commands::internet::execute_internet_command(&command)
                    .await
                    .map_err(|e| {
                        CliError::network_error(
                            &e.to_string(),
                            None,
                            "Check internet connectivity and tunnel configuration",
                        )
                    })
            }
            Commands::Firewall { command } => commands::firewall::execute_firewall(&command)
                .await
                .map_err(|e| {
                    CliError::command_error(
                        &e.to_string(),
                        Some("firewall"),
                        "Check firewall rules and system permissions",
                    )
                }),
            Commands::IoT { command } => commands::basic_iot::handle_basic_iot_command(command)
                .await
                .map_err(|e| {
                    CliError::command_error(
                        &e.to_string(),
                        Some("iot"),
                        "Check device connectivity and network configuration",
                    )
                }),
            Commands::Gaming { command } => {
                commands::gaming::handle_gaming_command(commands::gaming::GamingArgs { command })
                    .await
                    .map_err(|e| CliError::Gaming {
                        message: e.to_string(),
                        protocol: None,
                        game: None,
                        suggestion: Some(
                            "Check game installation and network settings".to_string(),
                        ),
                    })
            }
            Commands::Compose { command } => {
                commands::compose::handle_compose_command(commands::compose::ComposeArgs {
                    command,
                })
                .await
                .map_err(|e| {
                    CliError::command_error(
                        &e.to_string(),
                        Some("compose"),
                        "Check plugin dependencies and composition configuration",
                    )
                })
            }
            Commands::Federation { command } => {
                commands::basic_federation::handle_basic_federation_command(command)
                    .await
                    .map_err(|e| {
                        CliError::command_error(
                            &e.to_string(),
                            Some("federation"),
                            "Check federation endpoints and authentication",
                        )
                    })
            }
            Commands::Scale { args } => {
                commands::scale::handle_scale_command(args)
                    .await
                    .map_err(|e| {
                        CliError::command_error(
                            &e.to_string(),
                            Some("scale"),
                            "Check system resources and scaling configuration",
                        )
                    })
            }
            Commands::Join { network } => {
                commands::join::execute_join(network).await.map_err(|e| {
                    CliError::network_error(
                        &e.to_string(),
                        None,
                        "Check network availability and join credentials",
                    )
                })
            }
            Commands::ZeroTouch {
                dry_run,
                ref save_config,
                yes,
                ref output_file,
            } => {
                self.handle_zero_touch_command(
                    dry_run,
                    save_config.as_deref(),
                    yes,
                    output_file.as_deref(),
                )
                .await
            }
        }
    }

    /// Handle zero-touch deployment command with enhanced error handling
    async fn handle_zero_touch_command(
        &self,
        dry_run: bool,
        save_config: Option<&std::path::Path>,
        skip_confirmation: bool,
        output_summary: Option<&std::path::Path>,
    ) -> CliResult<()> {
        let command = crate::cli::commands::zero_touch::ZeroTouchCommand::new();
        command
            .execute(dry_run, save_config, skip_confirmation, output_summary)
            .await
            .map_err(|e| {
                CliError::command_error(
                    &e.to_string(),
                    Some("zero-touch"),
                    "Check system requirements and network connectivity",
                )
            })
    }
}

/// CLI configuration constants
pub mod constants {
    use std::time::Duration;

    /// Default configuration directory
    pub const DEFAULT_CONFIG_DIR: &str = ".songbird";

    /// Default configuration file name
    pub const DEFAULT_CONFIG_FILE: &str = "songbird.toml";

    /// Default data directory
    pub const DEFAULT_DATA_DIR: &str = ".songbird/data";

    /// Default log directory
    pub const DEFAULT_LOG_DIR: &str = ".songbird/logs";

    /// Default discovery timeout
    pub const DEFAULT_DISCOVERY_TIMEOUT: Duration = Duration::from_secs(5);

    /// Default connection timeout
    pub const DEFAULT_CONNECTION_TIMEOUT: Duration = Duration::from_secs(10);

    /// Default health check interval for CLI
    pub const DEFAULT_CLI_HEALTH_INTERVAL: Duration = Duration::from_secs(30);

    /// Maximum retry attempts for CLI operations
    pub const DEFAULT_MAX_RETRIES: u32 = 3;

    /// Default CLI operation timeout
    pub const DEFAULT_CLI_TIMEOUT: Duration = Duration::from_secs(30);
}

/// Enhanced error handling for CLI operations
pub mod error_handling {
    use super::*;

    /// Handle CLI error with enhanced display
    pub fn handle_cli_error(error: &CliError) -> ! {
        eprintln!("{}", error.display_enhanced());

        // Exit with appropriate code based on error type
        let exit_code = match error.get_severity() {
            "high" => 2,
            "medium" => 1,
            "low" => 0,
            _ => 1,
        };

        std::process::exit(exit_code);
    }

    /// Display suggestions for common errors
    pub fn display_common_solutions() {
        println!("\n{}", "Common Solutions:".bright_blue().bold());
        println!(
            "• Check configuration: {}",
            "songbird config show".bright_green()
        );
        println!(
            "• View system status: {}",
            "songbird status --detailed".bright_green()
        );
        println!("• Check logs: {}", "songbird logs --follow".bright_green());
        println!(
            "• Reset configuration: {}",
            "songbird config reset".bright_green()
        );
        println!("• Get help: {}", "songbird --help".bright_green());
    }
}

/// Execute start command with improved user experience
#[allow(dead_code)]
pub async fn execute_start_command(
    config: Option<&std::path::Path>,
    dashboard: bool,
    port: u16,
) -> CliResult<()> {
    use crate::cli::ui::*;

    // Validate configuration
    if let Some(config_path) = config {
        if !config_path.exists() {
            return Err(CliError::resource_not_found(
                "Configuration file not found",
                Some(&config_path.to_string_lossy()),
                None,
                "Create a configuration file with 'songbird init' or check the file path",
            ));
        }
    }

    // Validate port
    if port < 1024 {
        return Err(CliError::validation_error(
            "Port number too low",
            Some("port"),
            Some("1024-65535"),
            "Use a port number >= 1024 or run with elevated privileges",
        ));
    }

    print_info(&format!("Starting Songbird orchestrator on port {port}"));

    if dashboard {
        print_info(&format!(
            "Dashboard will be available at http://{}:{}",
            songbird_config::config::constants::network::DEFAULT_BIND_ADDRESS,
            songbird_config::config::constants::network::DEFAULT_DASHBOARD_PORT
        ));
    }

    // Placeholder for actual implementation
    Ok(())
}
