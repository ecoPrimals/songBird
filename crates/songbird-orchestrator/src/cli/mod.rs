// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    dead_code,
    reason = "CLI module is progressively wired — subcommand handlers and helpers become live as each command matures"
)]

//! # Songbird CLI Module
//!
//! This module provides the command-line interface for the Songbird Universal Orchestrator.
//! The CLI is organized into focused submodules for maintainability.

use anyhow::Result;
use clap::Parser;
use songbird_discovery::discovery::{config::SongbirdDiscoveryConfig, types::NodeType};
use std::sync::OnceLock;

// Import our organized modules
pub mod commands;
pub mod config;
pub mod utils;
pub mod handlers {
    pub mod discovery;
    pub mod init;
    pub mod service;
    pub mod status;
}

// Re-export important types for convenience
pub use commands::Commands;
pub use config::CliConfig;

/// Global discovery configuration - initialized once for performance
pub static DISCOVERY_CONFIG: OnceLock<SongbirdDiscoveryConfig> = OnceLock::new();

/// Main CLI structure
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Configuration file path
    #[arg(short, long)]
    pub config: Option<String>,

    /// Disable colored output
    #[arg(long)]
    pub no_color: bool,
}

/// Handle CLI command execution - main entry point
/// # Errors
///
/// Returns an error if the operation fails.
#[allow(clippy::too_many_lines, reason = "CLI command dispatch table")]
pub async fn handle_command(command: Commands, _config: &CliConfig) -> Result<()> {
    match command {
        Commands::Init {
            directory,
            non_interactive,
        } => handlers::init::handle_init_command(directory, non_interactive).await,
        Commands::Quick {
            quick_command,
        } => {
            tracing::info!(?quick_command, "quick command dispatched");
            Ok(())
        }
        Commands::Discovery {
            discovery_command,
        } => handlers::discovery::handle_discovery_command(discovery_command).await,
        Commands::Federation {
            federation_command,
        } => {
            tracing::info!(?federation_command, "federation command dispatched");
            Ok(())
        }
        Commands::Iot {
            iot_command,
        } => {
            tracing::info!(?iot_command, "IoT command dispatched");
            Ok(())
        }
        Commands::Compose {
            compose_command,
        } => {
            tracing::info!(?compose_command, "compose command dispatched");
            Ok(())
        }
        Commands::Node {
            node_command,
        } => {
            tracing::info!(?node_command, "node command dispatched");
            Ok(())
        }
        Commands::Service {
            service_command,
        } => handlers::service::handle_service_command(service_command).await,
        Commands::Status {
            status_command,
        } => handlers::status::handle_status_command(status_command).await,
        Commands::Logs {
            logs_command,
        } => {
            tracing::info!(?logs_command, "logs command dispatched");
            Ok(())
        }
        Commands::Scale {
            scale_command,
        } => {
            tracing::info!(?scale_command, "scale command dispatched");
            Ok(())
        }
        Commands::Security {
            security_command,
        } => {
            tracing::info!(?security_command, "security command dispatched");
            Ok(())
        }
        Commands::Firewall {
            firewall_command,
        } => {
            tracing::info!(?firewall_command, "firewall command dispatched");
            Ok(())
        }
        Commands::Internet {
            internet_command,
        } => {
            tracing::info!(?internet_command, "internet command dispatched");
            Ok(())
        }
        Commands::Join {
            join_command,
        } => {
            tracing::info!(?join_command, "join command dispatched");
            Ok(())
        }
        Commands::Share {
            share_command,
        } => {
            tracing::info!(?share_command, "share command dispatched");
            Ok(())
        }
        Commands::Universal {
            universal_command,
        } => {
            tracing::info!(?universal_command, "universal command dispatched");
            Ok(())
        }
        Commands::ZeroTouch {
            zero_touch_command,
        } => {
            tracing::info!(?zero_touch_command, "zero-touch command dispatched");
            Ok(())
        }
        Commands::Orchestrator {
            orchestrator_command,
        } => {
            tracing::info!(?orchestrator_command, "orchestrator command dispatched");
            Ok(())
        }
        Commands::Version {
            detailed,
        } => {
            tracing::info!(detailed, "version information requested");
            Ok(())
        }
        Commands::Dashboard {
            port,
            bind_address,
        } => {
            tracing::info!(%bind_address, port, "starting web dashboard");
            Ok(())
        }
    }
}

/// Get or create the discovery configuration (cached for performance)
pub fn get_discovery_config() -> &'static SongbirdDiscoveryConfig {
    DISCOVERY_CONFIG.get_or_init(|| SongbirdDiscoveryConfig {
        node_id: Some("orchestrator-cli".to_string()),
        node_type: NodeType::Orchestrator,
        institution: None,
        federation_enabled: false,
        health_check_interval_secs: 30,
        node_discovery_interval_secs: 60,
        trust_verification_enabled: false,
        max_federation_nodes: 100,
        network: songbird_discovery::discovery::config::NetworkConfig {
            multicast_address: songbird_types::constants::MDNS_MULTICAST_GROUP.to_string(),
            federation_port: songbird_types::defaults::ports::DEFAULT_TARPC_PORT,
            service_port: songbird_types::defaults::ports::DEFAULT_METRICS_PORT,
            bind_address: songbird_config::canonical::constants::get_bind_address(),
            announcement_interval_secs: 30,
            response_timeout_secs: 5,
            ping_timeout_secs: 3,
            max_packet_size: 1024,
            default_bandwidth_mbps: 100.0,
        },
        monitoring: songbird_discovery::discovery::config::MonitoringConfig {
            resource_update_interval_secs: 60,
            network_stats_window_secs: 300,
            storage_stats_window_secs: 300,
            process_scan_enabled: true,
            gpu_monitoring_enabled: false,
            detailed_cpu_monitoring: false,
        },
        trust: songbird_discovery::discovery::config::TrustConfig {
            institutional_base_score: 50,
            edu_domain_bonus: 20,
            gov_domain_bonus: 30,
            reputation_weight: 0.3,
            uptime_weight: 25,
            service_diversity_weight: 15,
            trust_thresholds: songbird_discovery::discovery::config::TrustThresholds {
                basic: 30,
                verified: 50,
                institutional: 70,
                consortium: 80,
            },
            interaction_penalties: songbird_discovery::discovery::config::InteractionPenalties {
                success_bonus: 0.01,
                slow_response_penalty: -0.005,
                failure_penalty: -0.02,
                timeout_penalty: -0.03,
                malicious_penalty: -0.1,
            },
        },
    })
}

/// Progress display utilities
pub fn show_progress(message: &str) {
    print!("⏳ {message} ");
    std::io::Write::flush(&mut std::io::stdout()).unwrap_or(());
}

/// Clear progress indicator
pub fn clear_progress() {
    print!("\r");
    std::io::Write::flush(&mut std::io::stdout()).unwrap_or(());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::SongbirdOrchestrator;
    use songbird_types::config::CanonicalSongbirdConfig;

    #[test]
    fn test_cli_config_creation() {
        let config = CliConfig::new();
        assert!(!config.is_verbose());
        assert!(config.is_colored_output_enabled());
        assert_eq!(config.get_config_path(), None);
    }

    #[test]
    fn test_cli_config_defaults() {
        let config = CliConfig::default();
        assert!(!config.is_verbose());
        assert!(config.is_colored_output_enabled());
    }

    #[test]
    fn test_cli_config_setters() {
        let mut config = CliConfig::new();

        config.set_verbose(true);
        assert!(config.is_verbose());

        config.set_colored_output(false);
        assert!(!config.is_colored_output_enabled());

        config.set_config_path("/test/config.toml".to_string());
        assert_eq!(config.get_config_path(), Some("/test/config.toml".to_string()));
    }

    #[tokio::test]
    async fn test_orchestrator_creation() {
        let config = CanonicalSongbirdConfig::default();
        let _orchestrator = SongbirdOrchestrator::new(config);
        // If this compiles and doesn't panic, initialization is successful
    }
}
