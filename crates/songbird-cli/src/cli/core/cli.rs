// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! CLI Application Core Core
//!
//! Main CLI application structure and execution logic.

#![allow(missing_docs, reason = "`run_cli` is internal wiring for the clap entrypoint")]

use crate::cli::commands::{self, Commands};
use songbird_types::{SongbirdError, SongbirdResult};

pub async fn run_cli() -> SongbirdResult<()> {
    use clap::Parser;

    // Parse command-line arguments using clap
    let cli = crate::Cli::parse();

    // Get the command to execute, default to version if none provided
    let command = cli.command.unwrap_or(Commands::Version {
        detailed: false,
    });

    match command {
        Commands::Tower {
            command,
        } => command.execute().await.map_err(|e| SongbirdError::Configuration {
            message: format!("Tower command failed: {e}"),
            field: None,
            suggestion: None,
        }),
        Commands::Version {
            detailed,
        } => commands::version::execute_version_command(detailed).await,
        Commands::Network {
            command,
        } => commands::network::handle_network_command(command).await,
        Commands::Federation {
            command,
        } => commands::federation::handle_federation_command(command).await,
        Commands::Config {
            command,
        } => commands::config::handle_config_command(command).await,
        Commands::Status {
            detailed,
            gaming: _,
        } => {
            commands::status::execute_status(detailed, None, crate::cli::types::OutputFormat::Auto)
                .await
        }
        Commands::Quick {
            name,
            auto_detect,
            family_safe,
        } => commands::quick::execute_quick_gaming(name, auto_detect, family_safe).await,
        Commands::Discover {
            timeout,
            protocol,
            continuous,
        } => commands::discovery::execute_discovery(timeout, protocol, continuous).await, // Additional commands can be added here as they are implemented
                                                                                          // The match is exhaustive for all currently defined Commands variants
    }
}

// NOTE: CliError moved to crate::errors for canonical definition
// Use: use crate::errors::CliError;

#[cfg(test)]
mod tests {
    use crate::{Cli, OutputFormat};
    use clap::Parser;

    #[test]
    fn test_cli_parsing() {
        // Test basic command parsing
        let cli = Cli::try_parse_from(["songbird", "version"]);

        assert!(cli.is_ok());

        let cli = Cli::try_parse_from(["songbird", "quick", "compute"]);

        assert!(cli.is_ok());
    }

    #[test]
    fn test_output_format_default() {
        assert_eq!(OutputFormat::default(), OutputFormat::Auto);
    }
}
