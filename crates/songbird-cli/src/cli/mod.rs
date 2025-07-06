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
use thiserror::Error;
// CLI module core
use self::commands::Commands;
use serde::{Deserialize, Serialize};
use songbird_errors::SongbirdError;
use std::path::PathBuf;
/// Songbird CLI Error types
#[derive(Error, Debug)]
pub enum CliError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Command error: {0}")]
    Command(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Gaming error: {message}")]
    Gaming {
        message: String,
        protocol: Option<String>,
    },

    #[error("Discovery error: {0}")]
    Discovery(String),

    #[error("Service error: {0}")]
    Service(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Songbird orchestrator error: {0}")]
    Orchestrator(#[from] SongbirdError),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("User cancelled operation")]
    UserCancelled,
}
/// CLI result type
pub type CliResult<T> = std::result::Result<T, CliError>;
/// Main CLI struct
#[derive(Parser)]
#[command(
    name = "songbird",
    about = "🎼 Songbird Orchestrator - Distributed Computing Made Simple",
    long_about = "Songbird Orchestrator enables easy distributed computing across networks.\nDesigned for students, researchers, and developers.",
    version = env!("CARGO_PKG_VERSION")
)]
pub struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,
    /// Suppress all output except errors
    #[arg(short, long, global = true)]
    pub quiet: bool,
    /// Output format for commands that support it
    #[arg(long, global = true, value_enum, default_value = "auto")]
    pub output: OutputFormat,
    /// Configuration file path
    #[arg(short = 'c', long = "config", global = true)]
    pub config: Option<PathBuf>,
    /// Override default data directory
    #[arg(long = "data-dir", global = true)]
    pub data_dir: Option<String>,
    /// Subcommands
    #[command(subcommand)]
    pub command: commands::Commands,
}

/// Deployment types
#[derive(clap::ValueEnum, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum DeploymentType {
    #[value(name = "home-network")]
    HomeNetwork,
    #[value(name = "research-cluster")]
    ResearchCluster,
    #[value(name = "edge-deployment")]
    EdgeDeployment,
    #[value(name = "development")]
    Development,
}
/// Configuration actions
#[derive(Debug, Subcommand)]
pub enum ConfigAction {
    /// Show current configuration
    Show,
    /// Edit configuration
    Edit,
    /// Validate configuration
    Validate,
    /// Reset to defaults
    Reset {
        /// Skip confirmation
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// Export configuration
    Export {
        /// Output file
        #[arg(short = 'o', long)]
        output: Option<String>,
        /// Export format
        #[arg(long, value_enum, default_value = "toml")]
        format: ExportFormat,
    },
}

/// Export formats
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum ExportFormat {
    Toml,
    Json,
    Yaml,
}
#[derive(Debug, Clone, clap::ValueEnum, Serialize, Deserialize)]
pub enum OutputFormat {
    /// Automatically detect best format
    Auto,
    /// Human-readable table format
    Table,
    /// JSON output
    Json,
    /// YAML output  
    Yaml,
    /// Simple text format
    Text,
}
impl Cli {
    /// Execute the CLI command
    pub async fn execute(self) -> CliResult<()> {
        // Set up logging level based on verbosity (no need to reinit subscriber)
        if !self.quiet {
            let level = if self.verbose { "debug" } else { "info" };
            std::env::set_var("RUST_LOG", format!("songbird={}", level));
        }
        // Execute the command
        match self.command {
            Commands::Version { detailed } => commands::version::show_version(detailed).await,
            Commands::Quick { contribute, name } => {
                commands::quick::execute_quick(contribute, name).await
            }
            Commands::Share { resource, percent } => {
                commands::share::execute_share(resource, percent).await
            }
            Commands::Init {
                deployment,
                quick,
                output_dir,
            } => commands::init::execute_init(deployment, quick, output_dir).await,
            Commands::Start {
                config,
                dashboard,
                port,
            } => {
                commands::orchestrator::start_orchestrator(config.as_deref(), dashboard, port).await
            }
            Commands::Stop { force } => commands::orchestrator::stop_orchestrator(force).await,
            Commands::Status {
                detailed,
                watch,
                format,
            } => commands::status::show_status(detailed, watch, format).await,
            Commands::Logs {
                service,
                follow,
                lines,
                level,
            } => commands::logs::show_logs(service.as_deref(), follow, lines, level).await,
            Commands::Internet { command } => {
                crate::cli::commands::internet::execute_internet_command(&command)
                    .await
                    .map_err(|e| crate::cli::CliError::Command(format!("{:?}", e)))
            }
            Commands::Firewall { command } => commands::firewall::execute_firewall(&command)
                .await
                .map_err(|e| CliError::Command(format!("Firewall command failed: {:?}", e))),
            Commands::IoT { command } => commands::basic_iot::handle_basic_iot_command(command)
                .await
                .map_err(CliError::Orchestrator),
            Commands::Gaming { command } => {
                commands::gaming::handle_gaming_command(commands::gaming::GamingArgs { command })
                    .await
                    .map_err(CliError::Orchestrator)
            }
            Commands::Compose { command } => {
                commands::compose::handle_compose_command(commands::compose::ComposeArgs {
                    command,
                })
                .await
                .map_err(CliError::Orchestrator)
            }
            Commands::Federation { command } => {
                commands::basic_federation::handle_basic_federation_command(command)
                    .await
                    .map_err(CliError::Orchestrator)
            }
            Commands::Scale { args } => commands::scale::handle_scale_command(args)
                .await
                .map_err(CliError::Orchestrator),
            Commands::Join { network } => commands::join::execute_join(network).await,
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

    /// Handle zero-touch deployment command
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
            .map_err(CliError::Orchestrator)
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
}
/// Execute start command with improved user experience
#[allow(dead_code)]
async fn execute_start(
    config_path: Option<PathBuf>,
    _enable_dashboard: bool,
    dashboard_port: u16,
) -> CliResult<()> {
    use songbird_config::config::OrchestratorConfig;
    use songbird_core::orchestrator::Orchestrator;
    println!("{}", ui::info("🚀 Starting Songbird Orchestrator..."));
    // Load configuration properly (no more hardcoding)
    let config = if let Some(path) = config_path {
        println!(
            "{}",
            ui::info(&format!(
                "📄 Loading configuration from: {}",
                path.display()
            ))
        );
        load_config_from_file(&path).await?
    } else {
        println!("{}", ui::info("⚙️  Using default configuration"));
        OrchestratorConfig::default()
    };

    // Fix the Orchestrator initialization by removing the .await
    let orchestrator = Orchestrator::new(config)?;

    // Create and start orchestrator
    orchestrator.start().await.map_err(CliError::Orchestrator)?;

    println!(
        "{}",
        ui::success("✅ Songbird Orchestrator started successfully!")
    );
    println!(
        "{}",
        ui::info(&format!(
            "📊 Dashboard available at: http://localhost:{}",
            dashboard_port
        ))
    );
    println!(
        "{}",
        ui::info("💡 Use 'songbird status' to check system status")
    );
    println!(
        "{}",
        ui::info("💡 Use 'songbird stop' to shut down gracefully")
    );
    // Keep running until interrupted
    tokio::signal::ctrl_c().await.map_err(CliError::Io)?;
    println!("{}", ui::info("⏹️  Stopping orchestrator..."));
    orchestrator.stop().await.map_err(CliError::Orchestrator)?;

    println!("{}", ui::success("✅ Stopped successfully"));
    Ok(())
}

/// Load configuration from file (no hardcoding)
#[allow(dead_code)]
async fn load_config_from_file(
    path: &PathBuf,
) -> CliResult<songbird_config::config::OrchestratorConfig> {
    if !path.exists() {
        return Err(CliError::Config(format!(
            "Configuration file not found: {}",
            path.display()
        )));
    }

    let contents = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| CliError::Config(format!("Failed to read config file: {}", e)))?;
    // Support multiple config formats based on extension
    let config = match path.extension().and_then(|ext| ext.to_str()) {
        Some("toml") => toml::from_str(&contents)
            .map_err(|e| CliError::Config(format!("Failed to parse TOML config: {}", e)))?,
        Some("yaml") | Some("yml") => serde_yaml::from_str(&contents)
            .map_err(|e| CliError::Config(format!("Failed to parse YAML config: {}", e)))?,
        Some("json") => serde_json::from_str(&contents)
            .map_err(|e| CliError::Config(format!("Failed to parse JSON config: {}", e)))?,
        _ => {
            return Err(CliError::Config(
                "Unsupported config file format. Use .toml, .yaml, .yml, or .json".to_string(),
            ));
        }
    };

    Ok(config)
}
/// Execute stop command
#[allow(dead_code)]
async fn execute_stop(force: bool) -> CliResult<()> {
    println!("{}", ui::info("⏹️  Stopping Songbird Orchestrator..."));
    if force {
        println!(
            "{}",
            ui::warn("⚠️  Force stopping - may not shut down gracefully")
        );
    }

    // Configurable stop timeout instead of hardcoded sleep
    let stop_timeout = std::env::var("SONGBIRD_STOP_TIMEOUT_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(2000); // Default 2 seconds
                          // Check if we should use simulation mode
    let simulation_mode = std::env::var("SONGBIRD_STOP_SIMULATION")
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(true); // Default to simulation since we don't have real orchestrator management yet
    if simulation_mode {
        println!("🎭 [SIMULATION MODE] Simulating orchestrator shutdown");

        // Simulate realistic shutdown process
        let steps = [
            (25, "📋 Saving current state..."),
            (50, "🔌 Closing connections..."),
            (75, "📊 Flushing metrics..."),
            (100, "✅ Shutdown complete!"),
        ];
        let step_duration = stop_timeout / steps.len() as u64;
        for (progress, message) in &steps {
            println!("   [{}%] {}", progress, message);
            tokio::time::sleep(tokio::time::Duration::from_millis(step_duration)).await;
        }
    } else {
        // Real orchestrator shutdown implementation
        match shutdown_real_orchestrator(force).await {
            Ok(()) => {
                println!("{}", ui::success("✅ Orchestrator stopped successfully"));
            }
            Err(e) => {
                println!(
                    "{}",
                    ui::warn(&format!("⚠️  Shutdown encountered issues: {:?}", e))
                );
                if !force {
                    println!("💡 Try using --force flag for forceful shutdown");
                    return Err(e);
                }
            }
        }
    }

    println!("{}", ui::success("✅ Orchestrator stopped"));
    Ok(())
}
/// Attempt to shutdown a real running orchestrator instance
async fn shutdown_real_orchestrator(force: bool) -> CliResult<()> {
    // Try to find running orchestrator process
    let orchestrator_pid = find_orchestrator_process().await?;
    if let Some(pid) = orchestrator_pid {
        println!(
            "{}",
            ui::info(&format!("📍 Found running orchestrator (PID: {})", pid))
        );
        if force {
            // Send SIGKILL (force terminate)
            terminate_process(pid, true).await?;
        } else {
            // Send SIGTERM (graceful shutdown)
            terminate_process(pid, false).await?;
            // Wait for graceful shutdown with timeout
            let shutdown_timeout = std::env::var("SONGBIRD_GRACEFUL_SHUTDOWN_TIMEOUT_MS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10000); // 10 seconds default

            if !wait_for_process_exit(pid, shutdown_timeout).await? {
                println!(
                    "{}",
                    ui::warn("⚠️  Graceful shutdown timed out, forcing termination")
                );
                terminate_process(pid, true).await?;
            }
        }
    } else {
        return Err(CliError::Command(
            "No running orchestrator found".to_string(),
        ));
    }

    Ok(())
}
/// Find running orchestrator process
async fn find_orchestrator_process() -> CliResult<Option<u32>> {
    #[cfg(unix)]
    {
        // Use pgrep to find songbird orchestrator process
        let output = std::process::Command::new("pgrep")
            .arg("-f")
            .arg("songbird")
            .output();
        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(pid_str) = stdout.lines().next() {
                    if let Ok(pid) = pid_str.trim().parse::<u32>() {
                        return Ok(Some(pid));
                    }
                }
            }
        }
    }

    #[cfg(windows)]
    {
        // Use tasklist to find songbird process
        let output = std::process::Command::new("tasklist")
            .arg("/FI")
            .arg("IMAGENAME eq songbird.exe")
            .arg("/FO")
            .arg("CSV")
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines().skip(1) {
                    // Skip header
                    let parts: Vec<&str> = line.split(',').collect();
                    if parts.len() >= 2 {
                        if let Ok(pid) = parts[1].trim_matches('"').parse::<u32>() {
                            return Ok(Some(pid));
                        }
                    }
                }
            }
        }
    }

    Ok(None)
}
/// Terminate process by PID
async fn terminate_process(pid: u32, force: bool) -> CliResult<()> {
    #[cfg(unix)]
    {
        let signal = if force { "KILL" } else { "TERM" };
        let output = std::process::Command::new("kill")
            .arg(format!("-{}", signal))
            .arg(pid.to_string())
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                println!(
                    "{}",
                    ui::info(&format!("📤 Sent {} signal to process {}", signal, pid))
                );
                return Ok(());
            }
        }
        Err(CliError::Command(format!(
            "Failed to send {} signal to process {}",
            signal, pid
        )))
    }

    #[cfg(windows)]
    {
        let flag = if force { "/F" } else { "/T" };
        let output = std::process::Command::new("taskkill")
            .arg(flag)
            .arg("/PID")
            .arg(pid.to_string())
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                println!(
                    "{}",
                    ui::info(&format!(
                        "📤 Terminated process {} ({})",
                        pid,
                        if force { "forced" } else { "graceful" }
                    ))
                );
                return Ok(());
            }
        }
        Err(CliError::Command(format!(
            "Failed to terminate process {}",
            pid
        )))
    }
}
/// Wait for process to exit
async fn wait_for_process_exit(pid: u32, timeout_ms: u64) -> CliResult<bool> {
    let start_time = std::time::Instant::now();
    let timeout = std::time::Duration::from_millis(timeout_ms);
    while start_time.elapsed() < timeout {
        if !is_process_running(pid).await? {
            return Ok(true); // Process exited
        }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
    Ok(false) // Timeout
}
/// Check if process is still running
async fn is_process_running(pid: u32) -> CliResult<bool> {
    #[cfg(unix)]
    {
        // Send signal 0 to check if process exists
        let output = std::process::Command::new("kill")
            .arg("-0")
            .arg(pid.to_string())
            .output();

        if let Ok(output) = output {
            return Ok(output.status.success());
        }
    }

    #[cfg(windows)]
    {
        let output = std::process::Command::new("tasklist")
            .arg("/FI")
            .arg(&format!("PID eq {}", pid))
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                return Ok(stdout.lines().count() > 1); // More than just header
            }
        }
    }

    Ok(false) // Assume not running if we can't check
}
