// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Public API for `UniBin` integration
//!
//! This module exposes the main entry points and types needed for
//! the unified `songbird` binary to route to orchestrator functionality.
//!
//! # Architecture
//!
//! The `bin_interface` is organized by CLI command:
//! - `server`: Server mode with IPC integration
//! - `doctor`: Health diagnostics and system checks
//! - `config`: Configuration management
//!
//! This refactoring (Phase 5B, Feb 2026) split the original 1,171-line monolith
//! into focused modules for better maintainability and testing.

use clap::{Args, Subcommand};

// Re-export main entry point function
pub use crate::app::start_orchestrator;

// Re-export command implementations
pub use self::config::run_config;
pub use self::doctor::run_doctor;
pub use self::server::run_server;

// Module declarations
mod config;
mod doctor;
mod server;

/// Server mode arguments
#[derive(Args, Debug, Clone)]
pub struct ServerArgs {
    /// HTTP server port (external discovery gateway)
    ///
    /// Songbird operates in dual-mode:
    /// • External TCP port (for LAN discovery beacons) ← this flag
    /// • Internal Unix socket (for inter-primal IPC) ← see --socket
    ///
    /// This port is used for:
    /// - Broadcasting discovery beacons to peers
    /// - Initial peer handshake
    /// - Federation negotiation
    /// - External API access
    ///
    /// Required when discovery is enabled (default).
    /// Environment-aware: Respects `SONGBIRD_HTTP_PORT`, `SONGBIRD_PORT`, or PORT.
    #[arg(long, short, default_value_t = crate::env_config::http_port())]
    pub port: u16,

    /// Federation port (alias for --port, clearer intent)
    ///
    /// Use this flag when explicitly configuring for LAN discovery/federation.
    /// If both --port and --federation-port are specified, --federation-port takes precedence.
    #[arg(long)]
    pub federation_port: Option<u16>,

    /// Run as daemon (background process)
    #[arg(long, short)]
    pub daemon: bool,

    /// Configuration file path
    #[arg(long, short)]
    pub config: Option<String>,

    /// Enable verbose logging
    #[arg(long, short)]
    pub verbose: bool,

    /// Unix socket path for IPC (JSON-RPC 2.0)
    ///
    /// Enables external primals to access HTTP/HTTPS capabilities via Unix socket.
    /// This is the INTERNAL interface for inter-primal communication.
    ///
    /// Songbird operates in dual-mode:
    /// • External TCP port (for LAN discovery) ← see --port
    /// • Internal Unix socket (for inter-primal IPC) ← this flag
    ///
    /// XDG-compliant path example: /run/user/1000/biomeos/songbird-nat0.sock
    /// Legacy fallback: /tmp/songbird-nat0.sock
    #[arg(long)]
    pub socket: Option<String>,

    /// Security provider socket path for crypto operations (defaults via capability discovery)
    ///
    /// If not specified, uses capability-based discovery:
    /// 1. `$SECURITY_PROVIDER_SOCKET` env var
    /// 2. `$XDG_RUNTIME_DIR/biomeos/security.sock` (capability symlink)
    /// 3. `$BEARDOG_SOCKET` (legacy)
    #[arg(long = "security-socket", alias = "beardog-socket")]
    pub security_socket: Option<String>,

    /// TCP listen address for IPC (alternative to Unix socket)
    ///
    /// Use this on platforms where Unix sockets are restricted (Android, Windows).
    /// When specified, Songbird uses TCP instead of Unix socket for inter-primal IPC.
    ///
    /// Example: --listen 127.0.0.1:9901
    ///
    /// Security: Only binds to localhost by default for same-device IPC.
    #[arg(long)]
    pub listen: Option<String>,

    /// Enable Dark Forest mode (encrypted BirdSong beacons, no plaintext fallback)
    ///
    /// When enabled, all discovery beacons are encrypted via security provider.
    /// Plaintext beacon fallback is disabled for maximum privacy.
    /// Equivalent to setting `SONGBIRD_DARK_FOREST=true`.
    #[arg(long, env = "SONGBIRD_DARK_FOREST")]
    pub dark_forest: bool,

    /// PID file directory (for Android/container substrates)
    ///
    /// Override the default PID file location for environments with
    /// restricted filesystem access (Android SELinux, containers).
    /// Equivalent to setting `SONGBIRD_PID_DIR`.
    #[arg(long, env = "SONGBIRD_PID_DIR")]
    pub pid_dir: Option<String>,
}

/// Doctor mode arguments
#[derive(Args, Debug, Clone)]
pub struct DoctorArgs {
    /// Run comprehensive checks (includes primal connectivity)
    #[arg(long, short)]
    pub comprehensive: bool,

    /// Output format (text, json, toml)
    #[arg(long, default_value = "text")]
    pub format: String,
}

/// Configuration management commands
#[derive(Subcommand, Debug, Clone)]
pub enum ConfigCommands {
    /// Show current configuration
    Show {
        /// Show sensitive values (API keys, etc.)
        #[arg(long)]
        show_secrets: bool,

        /// Output format (text, json, toml)
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Validate configuration
    Validate,

    /// Generate default configuration template
    Init {
        /// Output path for generated config
        #[arg(long, default_value = "songbird.toml")]
        output: String,

        /// Overwrite existing file
        #[arg(long)]
        force: bool,
    },
}
