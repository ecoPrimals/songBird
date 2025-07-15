//! # Songbird CLI
//!
//! Command-line interface for the Songbird orchestrator platform, providing comprehensive
//! tools for service management, gaming optimization, and system administration.
//!
//! ## Features
//!
//! - **Service Management**: Start, stop, and manage services
//! - **Gaming Commands**: Gaming-specific network optimization and management
//! - **Federation Control**: Multi-region federation setup and management
//! - **Health Monitoring**: Real-time system health and performance monitoring
//! - **Configuration Management**: Dynamic configuration updates and validation
//! - **Biome Deployment**: BYOB (Bring Your Own Biome) deployment management
//! - **Security Operations**: Security policy management and audit controls
//! - **Network Diagnostics**: Network troubleshooting and performance analysis
//!
//! ## Architecture
//!
//! The CLI is organized into focused command modules:
//!
//! - `commands`: Core command implementations and handlers
//! - `config`: Configuration management and validation
//! - `discovery`: Service discovery and network scanning
//! - `gaming`: Gaming-specific commands and optimizations
//! - `federation`: Multi-region federation management
//! - `security`: Security and authentication commands
//!
//! ## Usage
//!
//! ### Basic Commands
//! ```bash
//! # Start orchestrator service
//! songbird start
//!
//! # Check system status
//! songbird status
//!
//! # Deploy a biome
//! songbird deploy --manifest biome.yaml
//! ```
//!
//! ### Gaming Commands
//! ```bash
//! # Optimize network for gaming
//! songbird gaming optimize
//!
//! # Scan for gaming services
//! songbird gaming scan
//!
//! # Set up gaming host
//! songbird gaming host --port 7777
//! ```
//!
//! ### Federation Commands
//! ```bash
//! # Initialize federation
//! songbird federation init
//!
//! # Join existing federation
//! songbird federation join --endpoint https://federation.example.com
//! ```
//!
//! ## Integration
//!
//! Use this crate to integrate CLI functionality into your applications:
//!
//! ```rust,no_run
//! use songbird_cli::cli::{CliConfig, SongbirdCli};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = CliConfig::default();
//!     let cli = SongbirdCli::new(config);
//!     
//!     // Execute CLI commands programmatically
//!     cli.execute_command("status").await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Error Handling
//!
//! All CLI operations return detailed error information with suggestions for
//! resolution. Common error scenarios include:
//!
//! - Configuration validation errors
//! - Network connectivity issues
//! - Permission and authentication failures
//! - Service unavailability
//!
//! ## Performance
//!
//! The CLI is optimized for responsive user experience:
//! - Command execution: <100ms for local operations
//! - Network operations: Timeout after 30s with progress indicators
//! - Large operations: Progress bars and incremental updates

pub mod cli;
pub use cli::*;
