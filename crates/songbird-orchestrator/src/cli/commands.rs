//! Command definitions for the Songbird CLI
//!
//! This module contains all the command and subcommand enums used by the CLI.

use clap::Subcommand;
// use songbird_types::unified_constants::*; // Module not available yet

/// Available CLI commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize a new Songbird configuration
    Init {
        /// Directory to initialize in
        #[arg(short, long)]
        directory: Option<String>,
        /// Skip interactive setup
        #[arg(long)]
        non_interactive: bool,
    },
    /// Quick start commands for common scenarios
    Quick {
        /// Quick command
        quick_command: String,
    },
    /// Service discovery and network scanning
    Discovery {
        #[command(subcommand)]
        discovery_command: Option<DiscoveryCommands>,
    },
    /// Basic federation and sharing
    Federation {
        /// Federation command
        federation_command: String,
    },
    /// `IoT` device management
    Iot {
        /// `IoT` command
        iot_command: String,
    },
    /// Plugin composition and management
    Compose {
        /// Compose command
        compose_command: String,
    },
    /// Node management
    Node {
        /// Node command
        node_command: String,
    },
    /// Service management
    Service {
        #[command(subcommand)]
        service_command: Option<ServiceCommands>,
    },
    /// Status and monitoring
    Status {
        /// Show overall system status
        #[command(subcommand)]
        status_command: Option<StatusCommands>,
    },
    /// Logs and diagnostics
    Logs {
        /// Logs command
        logs_command: String,
    },
    /// Scale and performance
    Scale {
        /// Scale command
        scale_command: String,
    },
    /// Security audit and management
    Security {
        /// Security command
        security_command: String,
    },
    /// Firewall and network protection
    Firewall {
        /// Firewall command
        firewall_command: String,
    },
    /// Internet connection and routing
    Internet {
        /// Internet command
        internet_command: String,
    },
    /// Join existing networks
    Join {
        /// Join command
        join_command: String,
    },
    /// Share resources and folders
    Share {
        /// Share command
        share_command: String,
    },
    /// Universal access and features
    Universal {
        /// Universal command
        universal_command: String,
    },
    /// Zero-touch deployment
    ZeroTouch {
        /// Zero-touch command
        zero_touch_command: String,
    },
    /// Orchestrator management
    Orchestrator {
        /// Orchestrator command
        orchestrator_command: String,
    },
    /// Version information
    Version {
        /// Show detailed version information
        #[arg(long)]
        detailed: bool,
    },
    /// Web dashboard
    Dashboard {
        /// Port to run dashboard on
        #[arg(short, long, default_value_t = 8080)]
        port: u16,
        /// Bind address
        #[arg(short, long, default_value = songbird_config::config::constants::network::DEFAULT_BIND_ADDRESS)]
        bind_address: String,
    },
}

/// Status subcommands
#[derive(Subcommand, Debug)]
pub enum StatusCommands {
    /// Show overall system status
    Overview,
    /// Show service statuses
    Services,
    /// Show network status
    Network,
    /// Show system health
    Health,
}

/// Service management subcommands
#[derive(Subcommand, Debug)]
pub enum ServiceCommands {
    /// List all services
    List,
    /// Show service details
    Show {
        service_name: String,
    },
    /// Start a service
    Start {
        service_name: String,
    },
    /// Stop a service
    Stop {
        service_name: String,
    },
    /// Restart a service
    Restart {
        service_name: String,
    },
}

/// Discovery subcommands
#[derive(Subcommand, Debug)]
pub enum DiscoveryCommands {
    /// Scan local network for services
    Scan {
        /// Network interface to scan
        #[arg(short, long)]
        interface: Option<String>,
        /// Timeout in seconds
        #[arg(short, long, default_value_t = 10)]
        timeout: u64,
        /// Port range to scan (e.g., 8000-9000)
        #[arg(short, long)]
        port_range: Option<String>,
    },
    /// Show discovered services
    List,
    /// Refresh service discovery
    Refresh,
    /// Test network connectivity
    Test {
        /// Target host or service
        target: String,
        /// Number of test attempts
        #[arg(short, long, default_value_t = 3)]
        count: u32,
    },
    /// Show network topology
    Topology,
    /// Advanced discovery options
    Advanced {
        /// Enable deep scanning
        #[arg(long)]
        deep_scan: bool,
        /// Include external services
        #[arg(long)]
        include_external: bool,
        /// Custom scan patterns
        #[arg(long)]
        pattern: Option<String>,
    },
}
