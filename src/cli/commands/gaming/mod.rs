//! Gaming CLI Commands - Main Module

pub mod discovery;
pub mod session;
pub mod setup;
pub mod utils;

use crate::cli::CliResult;
use crate::errors::{Result, SongbirdError};
use clap::{Parser, Subcommand};

// Re-export main functions
pub use discovery::*;
pub use session::*;
pub use setup::*;
pub use utils::*;

#[derive(Parser, Debug)]
pub struct GamingArgs {
    #[command(subcommand)]
    pub command: GamingCommand,
}

#[derive(Subcommand, Debug)]
pub enum GamingCommand {
    Scan {
        #[arg(long)]
        interface: Option<String>,
        #[arg(long)]
        duration: Option<u64>,
        #[arg(long)]
        continuous: bool,
        #[arg(long)]
        filter: Option<String>,
    },
    Host {
        #[arg(long)]
        auto: bool,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        encrypt: bool,
        #[arg(long)]
        private: bool,
    },
    Join {
        code: String,
    },
    Status,
    Browse,
    Diagnostics,
    Configure,
    OneTouch {
        name: String,
        #[arg(long)]
        family_safe: bool,
        #[arg(long)]
        parental_controls: bool,
        #[arg(long)]
        guests: bool,
    },
    ZeroTouch {
        #[arg(long)]
        endpoint: Option<String>,
        #[arg(long)]
        token: Option<String>,
    },
    FamilySafe {
        family_name: String,
    },
    QuickStart {
        #[arg(long)]
        auto_detect: bool,
        #[arg(long)]
        game: Option<String>,
        #[arg(long)]
        family_safe: bool,
        #[arg(long)]
        name: Option<String>,
    },
}

/// Handle gaming commands - main entry point
///
/// Dispatches gaming commands to their appropriate handlers.
/// Provides comprehensive gaming functionality including scanning,
/// hosting, joining sessions, and various setup modes.
///
/// # Arguments
/// * `args` - Gaming command arguments containing the specific command to execute
///
/// # Returns
/// Result indicating success or failure of the gaming command
pub async fn handle_gaming_command(args: GamingArgs) -> Result<()> {
    match args.command {
        GamingCommand::Scan {
            interface,
            duration,
            continuous,
            filter,
        } => scan_for_games(interface, duration, continuous, filter).await,
        GamingCommand::Host {
            auto,
            name,
            encrypt,
            private,
        } => host_gaming_session(auto, name, encrypt, private).await,
        GamingCommand::Join { code } => join_gaming_session(Some(code)).await,
        GamingCommand::Status => show_gaming_status().await,
        GamingCommand::Browse => execute_browse().await.map_err(|e| SongbirdError::Gaming {
            message: format!("Browse failed: {}", e),
            protocol: None,
        }),
        GamingCommand::Diagnostics => {
            execute_diagnostics()
                .await
                .map_err(|e| SongbirdError::Gaming {
                    message: format!("Diagnostics failed: {}", e),
                    protocol: None,
                })
        }
        GamingCommand::Configure => execute_configure()
            .await
            .map_err(|e| SongbirdError::Gaming {
                message: format!("Configure failed: {}", e),
                protocol: None,
            }),
        GamingCommand::OneTouch {
            name,
            family_safe,
            parental_controls,
            guests,
        } => execute_one_touch(name, family_safe, parental_controls, guests)
            .await
            .map_err(|e| SongbirdError::Gaming {
                message: format!("One-touch setup failed: {}", e),
                protocol: None,
            }),
        GamingCommand::ZeroTouch { endpoint, token } => execute_zero_touch(endpoint, token)
            .await
            .map_err(|e| SongbirdError::Gaming {
                message: format!("Zero-touch setup failed: {}", e),
                protocol: None,
            }),
        GamingCommand::FamilySafe { family_name } => execute_family_safe(family_name)
            .await
            .map_err(|e| SongbirdError::Gaming {
                message: format!("Family-safe setup failed: {}", e),
                protocol: None,
            }),
        GamingCommand::QuickStart {
            auto_detect,
            game,
            family_safe,
            name,
        } => execute_quick_start(auto_detect, game, family_safe, name)
            .await
            .map_err(|e| SongbirdError::Gaming {
                message: format!("Quick start failed: {}", e),
                protocol: None,
            }),
    }
}

/// Legacy gaming command executor for backward compatibility
///
/// Provides compatibility wrapper for older gaming command interfaces.
/// Converts gaming commands to the new Args format and executes them.
///
/// # Arguments
/// * `command` - Legacy gaming command to execute
///
/// # Returns
/// CLI result indicating success or error with appropriate CLI error types
pub async fn execute_gaming(command: GamingCommand) -> CliResult<()> {
    let args = GamingArgs { command };
    handle_gaming_command(args)
        .await
        .map_err(|e| crate::cli::CliError::ExecutionError(e.to_string()))
}
