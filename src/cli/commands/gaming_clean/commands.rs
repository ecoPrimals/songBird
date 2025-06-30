/// Gaming CLI Command Definitions
/// 
/// This module contains only the command structure definitions,
/// keeping them separate from the implementation logic.

use clap::{Parser, Subcommand};
use crate::network::gaming::GameProtocolClass;

#[derive(Parser, Debug)]
pub struct GamingArgs {
    #[command(subcommand)]
    pub command: GamingCommand,
}

#[derive(Subcommand, Debug)]
pub enum GamingCommand {
    /// Scan for gaming traffic
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
    /// Host a gaming session
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
    /// Join a gaming session
    Join { 
        code: String 
    },
    /// Show gaming status
    Status,
    /// Browse available sessions
    Browse,
    /// Run gaming diagnostics
    Diagnostics,
    /// Configure gaming settings
    Configure,
    /// One-touch setup for families
    OneTouch {
        name: String,
        #[arg(long)]
        family_safe: bool,
        #[arg(long)]
        parental_controls: bool,
        #[arg(long)]
        guests: bool,
    },
    /// Zero-touch setup with BearDog security
    ZeroTouch {
        #[arg(long)]
        endpoint: Option<String>,
        #[arg(long)]
        token: Option<String>,
    },
    /// Family-safe gaming setup
    FamilySafe { 
        family_name: String 
    },
    /// Quick one-click gaming setup
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

/// Discovery message structure for network gaming
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiscoveryMessage {
    pub session_code: String,
    pub host_address: std::net::SocketAddr,
    pub game_name: String,
    pub protocol_class: GameProtocolClass,
    pub max_players: u8,
    pub current_players: u8,
} 