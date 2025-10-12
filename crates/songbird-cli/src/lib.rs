//! # 🎮 Songbird Gaming CLI
//!
//! **MODERN GAMING-FOCUSED COMMAND-LINE INTERFACE** ✅
//!
//! Command-line interface for the Songbird gaming orchestrator platform, providing
//! streamlined tools for gaming session management, network optimization, and gaming federation.
//!
//! ## 🎯 **Gaming-First Features**
//!
//! - **🎮 Gaming Sessions**: Create, join, and manage gaming sessions
//! - **🌐 Network Gaming**: Gaming-specific network optimization and protocol support
//! - **🤝 Gaming Federation**: Multi-region gaming federation and matchmaking
//! - **📊 Gaming Metrics**: Real-time gaming performance and latency monitoring
//! - **🔧 Gaming Config**: Dynamic gaming configuration and protocol management
//! - **🛡️ Gaming Security**: Gaming-specific security and anti-cheat integration
//! - **🔍 Gaming Discovery**: Automatic gaming service discovery and scanning
//!
//! ## 🏗️ **Modern Architecture**
//!
//! The CLI is organized into focused gaming command modules:
//!
//! - `gaming`: Core gaming session and protocol commands
//! - `network`: Gaming network optimization and diagnostics
//! - `federation`: Gaming federation and matchmaking
//! - `security`: Gaming security and authentication
//! - `config`: Gaming configuration management
//!
//! ## 🚀 **Usage Examples**
//!
//! ### Gaming Session Commands
//! ```bash
//! # Quick start a gaming session
//! songbird gaming host --name "My Game" --protocol udp"
//!
//! # Join an existing gaming session
//! songbird gaming join GAME-CODE-123
//!
//! # Scan for nearby gaming sessions
//! songbird gaming scan --protocol all
//! ```
//!
//! ### Network Gaming Commands
//! ```bash
//! # Optimize network for gaming
//! songbird network optimize --game-mode
//!
//! # Test gaming network latency
//! songbird network test --gaming-protocols
//!
//! # Configure gaming port forwarding
//! songbird network ports --gaming --auto-configure
//! ```
//!
//! ### Gaming Federation Commands
//! ```bash
//! # Initialize gaming federation
//! songbird federation init --gaming
//!
//! # Join gaming federation
//! songbird federation join --gaming-endpoint https://gaming.example.com
//!
//! # Create gaming lobby
//! songbird federation lobby --create --name "Epic Battle""
//! ```
//!
//! ## 🔧 **Integration**
//!
//! Integrates seamlessly with the consolidated Songbird architecture:
//! - Uses `songbird-network-federation` for gaming protocols
//! - Uses `songbird-security-errors` for unified error handling
//! - Uses `songbird-orchestrator` for session coordination

#![warn(unsafe_code)]
#![warn(clippy::all)]
#![warn(rust_2018_idioms)]

// Core CLI modules
pub mod cli;
pub mod errors;

// Re-export main CLI types
pub use cli::{Cli, CliArgs, OutputFormat};
pub use errors::{CliError, CliResult};
