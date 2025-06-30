// Module imports
//! CLI Commands Module
//!
//! Contains all the CLI command implementations

use clap::Subcommand;
pub mod config;
pub mod discovery;
pub mod init;
pub mod logs;
pub mod node;
pub mod orchestrator;
pub mod service;
pub mod status;
pub mod version;
// New simple commands for students
pub mod basic_federation;
pub mod basic_iot; // Universal IoT device connectivity
pub mod compose; // New dynamic composition command
pub mod firewall;
pub mod gaming;
pub mod internet;
pub mod join;
pub mod quick;
pub mod scale; // Adaptive scaling command
pub mod share;
pub mod universal;
pub mod zero_touch; // Friend federation for data sharing
pub mod security_audit;

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Display version information
    Version {
        /// Show detailed version information
        #[arg(long)]
        detailed: bool,
    },
    /// Quick start - automatically discover and join network (STUDENT FRIENDLY)
    #[command(about = "🚀 Quick start - automatically discover and join the network")]
    Quick {
        /// What you want to contribute (compute, storage, or data)
        #[arg(value_enum, default_value = "compute")]
        contribute: quick::ContributeType,

        /// Optional: Your name/identifier
        name: Option<String>,
    },
    /// Join an existing Songbird network automatically (STUDENT FRIENDLY)  
    #[command(about = "🤝 Join an existing Songbird network (auto-discovery)")]
    Join {
        /// Optional: Specific network name to join
        network: Option<String>,
    },
    /// Share resources with the network (STUDENT FRIENDLY)
    #[command(about = "📤 Share your resources with the network")]
    Share {
        /// What to share (compute, storage, data, or all)
        #[arg(value_enum, default_value = "all")]
        resource: share::ResourceType,
        /// Amount to share (percentage of total resources)
        #[arg(long, default_value = "50")]
        percent: u8,
    },
    /// Zero-touch deployment - automatic setup and deployment (ULTRA CONVENIENT)
    #[command(about = "🪄 Zero-touch deployment - completely automatic setup")]
    ZeroTouch {
        /// Dry run - show what would be deployed without actually deploying
        #[arg(long)]
        dry_run: bool,
        /// Save generated configuration to file
        #[arg(long)]
        save_config: Option<std::path::PathBuf>,
        /// Skip confirmation prompts
        #[arg(short, long)]
        yes: bool,
        /// Output deployment summary to file
        #[arg(long = "output-file")]
        output_file: Option<std::path::PathBuf>,
    },
    /// Interactive setup wizard (for advanced users)
    #[command(about = "⚙️  Interactive setup wizard")]
    Init {
        /// Deployment type
        #[arg(short, long, value_enum, default_value = "home-network")]
        deployment: crate::cli::DeploymentType,
        /// Skip prompts and use defaults
        #[arg(long)]
        quick: bool,
        /// Output directory for configuration files
        #[arg(short = 'o', long = "output-dir", default_value = ".")]
        output_dir: std::path::PathBuf,
    },
    /// Start orchestrator
    #[command(about = "▶️  Start the orchestrator")]
    Start {
        /// Configuration file path
        #[arg(short, long)]
        config: Option<std::path::PathBuf>,
        /// Enable dashboard
        #[arg(long)]
        dashboard: bool,
        /// Dashboard port
        #[arg(long, default_value = "8080")]
        port: u16,
    },
    /// Stop orchestrator
    #[command(about = "⏹️  Stop the orchestrator")]
    Stop {
        /// Force stop without graceful shutdown
        #[arg(long)]
        force: bool,
    },
    /// Show system status
    #[command(about = "📊 Show system status")]
    Status {
        /// Show detailed information
        #[arg(long)]
        detailed: bool,
        /// Refresh interval in seconds for live updates
        #[arg(long)]
        watch: Option<u64>,
        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: crate::cli::OutputFormat,
    },
    /// View and follow logs
    #[command(about = "📋 View and follow logs")]
    Logs {
        /// Service to show logs for
        service: Option<String>,
        /// Follow log output
        follow: bool,
        /// Number of lines to show
        #[arg(short, long, default_value = "100")]
        lines: usize,
        /// Log level filter
        #[arg(long, value_enum, default_value = "info")]
        level: LogLevel,
    },
    /// Internet connection wizard for secure remote connectivity
    #[command(about = "🌐 Setup secure internet connections between Songbird nodes")]
    Internet {
        #[command(subcommand)]
        command: InternetCommands,
    },
    /// Firewall configuration wizard for system-agnostic security
    #[command(about = "🛡️ Configure system-agnostic firewall protection for Songbird")]
    Firewall {
        #[command(subcommand)]
        command: FirewallCommands,
    },
    /// Universal IoT device connectivity (scanners, printers, cameras, etc.)
    #[command(about = "🔌 Universal IoT device connectivity - connect ANY device")]
    IoT {
        #[command(subcommand)]
        command: BasicIoTCommands,
    },
    /// Universal legacy gaming network bridge
    #[command(
        about = "🎮 Universal gaming network bridge - play ANY legacy game over the internet"
    )]
    Gaming {
        #[command(subcommand)]
        command: gaming::GamingCommand,
    },
    /// Dynamic plugin composition - Lego-block service integration
    #[command(about = "🧩 Dynamic plugin composition - services work together like Lego blocks")]
    Compose {
        #[command(subcommand)]
        command: compose::ComposeCommand,
    },
    /// Friend federation - backup data with friends
    #[command(about = "🤝 Friend federation - backup data with trusted friends")]
    Federation {
        #[command(subcommand)]
        command: BasicFederationCommands,
    },
    /// Adaptive scaling - from chickadee to albatross
    #[command(about = "🐦 Adaptive scaling - scale from chickadee to albatross based on load")]
    Scale {
        #[command(flatten)]
        args: scale::ScaleArgs,
    },
}
/// Log levels for filtering
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}
/// Internet connection subcommands
#[derive(Debug, Clone, clap::Subcommand)]
pub enum InternetCommands {
    /// Start the Internet Connection Wizard
    #[command(about = "🧙‍♂️ Start the interactive Internet Connection Wizard")]
    Wizard {
        /// Environment preset (family-network, student-mobile, remote-office)
        environment: Option<String>,
        /// Tunnel technology (wireguard, tailscale, openvpn, zerotier)
        tunnel: Option<String>,
        /// Family network name
        network_name: Option<String>,
        /// Disable automatic configuration discovery
        #[arg(long)]
        no_discovery: bool,
    },
    /// Show internet connection status
    #[command(about = "📊 Show internet connection status")]
    Status,
    /// Connect to a family network
    #[command(about = "🔗 Connect to a family network")]
    Connect {
        /// Family network name to connect to
        network: String,
    },
    /// Disconnect from internet tunnels
    #[command(about = "🔌 Disconnect from internet tunnels")]
    Disconnect,
    /// Manage internet connection configuration
    #[command(about = "⚙️ Manage internet connection configuration")]
    Config {
        #[command(subcommand)]
        action: InternetConfigAction,
    },
}
/// Internet configuration actions
#[derive(Debug, Clone, clap::Subcommand)]
pub enum InternetConfigAction {
    /// Show current configuration
    Show,

    /// Validate configuration file
    Validate {
        /// Configuration file to validate
        config: Option<std::path::PathBuf>,
    },
    /// Discover Songbird port configuration
    Ports,
}
/// Firewall configuration subcommands
#[derive(Debug, Clone, clap::Subcommand)]
pub enum FirewallCommands {
    /// Interactive firewall configuration wizard
    #[command(about = "🧙 Interactive firewall configuration wizard")]
    Wizard {
        /// Configuration file path
        config: Option<std::path::PathBuf>,
        /// Environment type (home-hpc, development, production)
        environment: Option<String>,
        /// Firewall backend (ufw, iptables, windows, pfctl, freebsd, manual, auto)
        backend: Option<String>,
        /// Security level (low, medium, high, maximum)
        security_level: Option<String>,
        /// Skip security validation
        #[arg(long)]
        no_validation: bool,
    },
    /// Show firewall status
    #[command(about = "📊 Show current firewall status")]
    Status,
    /// Enable Songbird firewall protection
    #[command(about = "✅ Enable firewall protection")]
    Enable,
    /// Disable Songbird firewall protection
    #[command(about = "❌ Disable firewall protection")]
    Disable,
    /// Reset firewall configuration to defaults
    #[command(about = "🔄 Reset to secure defaults")]
    Reset,
    /// Test firewall configuration
    #[command(about = "🧪 Test firewall configuration")]
    Test,
    /// Manage firewall configuration
    #[command(about = "⚙️ Manage firewall configuration")]
    Config {
        #[command(subcommand)]
        action: FirewallAction,
    },
}
/// Firewall configuration actions
#[derive(Debug, Clone, clap::Subcommand)]
pub enum FirewallAction {
    /// Show current configuration
    #[command(about = "📋 Show current configuration")]
    Show,
    /// Edit configuration file
    #[command(about = "✏️ Edit configuration file")]
    Edit,
    /// Validate configuration
    #[command(about = "🔍 Validate configuration")]
    Validate,
    /// Export configuration
    #[command(about = "📤 Export configuration")]
    Export {
        /// Export file path
        path: std::path::PathBuf,
    },
    /// Import configuration
    #[command(about = "📥 Import configuration")]
    Import {
        /// Import file path
        path: std::path::PathBuf,
    },
}
// IoT device management removed - out of scope for gaming orchestration

/// Basic IoT device connectivity - universal connector functions
#[derive(Debug, Clone, clap::Subcommand)]
pub enum BasicIoTCommands {
    /// Discover devices on network (scanners, printers, cameras, etc.)
    #[command(about = "🔍 Discover any device on your network")]
    Discover {
        /// Device type to look for (scanner, printer, camera, any)
        #[arg(long, default_value = "any")]
        device_type: String,
        /// Show detailed device information
        #[arg(long)]
        detailed: bool,
    },
    /// Connect to a simple IoT device
    #[command(about = "🔗 Connect to any IoT device")]
    Connect {
        /// Device address (IP, URL, etc.)
        address: String,
        /// Device type (scanner, printer, camera, etc.)
        device_type: String,
        /// Friendly name for the device
        name: String,
    },
    /// List connected devices
    #[command(about = "📋 List connected IoT devices")]
    List {
        /// Filter by device type
        #[arg(long)]
        device_type: Option<String>,
    },
    /// Send simple command to device
    #[command(about = "⚡ Send command to IoT device")]
    Command {
        /// Device name or address
        device: String,
        /// Simple action (scan, print, status, etc.)
        action: String,
    },
}

/// Basic friend federation - simple data sharing between friends
#[derive(Debug, Clone, clap::Subcommand)]
pub enum BasicFederationCommands {
    /// Connect to a friend's SongBird for data sharing
    #[command(about = "🤝 Connect to a friend for data sharing")]
    Connect {
        /// Friend's SongBird address (IP or domain)
        address: String,
        /// Friendly name for this connection
        #[arg(long)]
        name: String,
        /// Trust level (friend, family, acquaintance)
        #[arg(long, default_value = "friend")]
        trust: String,
    },
    /// Share a folder with connected friends
    #[command(about = "📤 Share a folder with friends")]
    Share {
        /// Path to folder to share
        folder: std::path::PathBuf,
        /// Friend names to share with (comma-separated)
        friends: String,
        /// Permission level (read, backup, sync)
        #[arg(long, default_value = "read")]
        permission: String,
    },
    /// List connected friends
    #[command(about = "👥 List connected friends")]
    List,
    /// Backup data to friends' SongBirds
    #[command(about = "💾 Backup data to friends")]
    Backup {
        /// Path to backup
        path: std::path::PathBuf,
        /// Friends to backup to
        friends: String,
        /// Encrypt the backup
        #[arg(long)]
        encrypt: bool,
    },
    /// Check federation status
    #[command(about = "📊 Check federation status")]
    Status,
}

/// Workflow management actions
#[derive(Debug, Clone, clap::Subcommand)]
pub enum WorkflowAction {
    /// List all workflows
    #[command(about = "📋 List all workflows")]
    List,
    /// Create workflow from file
    #[command(about = "🔧 Create workflow from file")]
    Create {
        /// Workflow definition file (YAML)
        file: std::path::PathBuf,
    },
    /// Execute a workflow
    #[command(about = "⚡ Execute a workflow")]
    Execute {
        /// Workflow name
        name: String,
    },
    /// Enable a workflow
    #[command(about = "✅ Enable a workflow")]
    Enable {
        /// Workflow name
        name: String,
    },
    /// Disable a workflow
    #[command(about = "❌ Disable a workflow")]
    Disable {
        /// Workflow name
        name: String,
    },
}
