// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! # 🎮 Gaming CLI Commands
//!
//! **MODERN GAMING-FOCUSED COMMAND STRUCTURE** ✅
//!
//! Streamlined command structure focused on gaming functionality with
//! integration to the consolidated Songbird architecture.

#![allow(missing_docs, reason = "clap derives supply user-visible help text")]

use clap::Subcommand;

// Core gaming command modules
pub mod config;
pub mod federation;
pub mod network;
pub mod status;
pub mod tower; // NEW: Tower management
pub mod version;

// Legacy compatibility modules (simplified)
pub mod discovery;
pub mod quick;

/// **MODERN**: Gaming-focused command structure
#[derive(Debug, Clone, Subcommand)]
pub enum Commands {
    /// 🏰 Tower management (start orchestrator)
    #[command(about = "🏰 Start and manage Songbird towers")]
    Tower {
        #[command(subcommand)]
        command: tower::TowerCommand,
    },

    /// Mesh network probing and diagnostics
    #[command(about = "Probe mesh peer connectivity and latency")]
    Network {
        #[command(subcommand)]
        command: network::NetworkCommand,
    },

    /// 🤝 Gaming federation and matchmaking
    #[command(about = "🤝 Gaming federation and matchmaking")]
    Federation {
        #[command(subcommand)]
        command: federation::FederationCommand,
    },

    /// 🔧 Gaming configuration management
    #[command(about = "🔧 Gaming configuration and protocol management")]
    Config {
        #[command(subcommand)]
        command: config::ConfigCommand,
    },

    /// 📊 System and gaming status
    #[command(about = "📊 System and gaming status monitoring")]
    Status {
        /// Show detailed status information
        #[arg(long)]
        detailed: bool,

        /// Focus on gaming metrics
        #[arg(long)]
        gaming: bool,
    },

    /// 🚀 Quick gaming setup (simplified)
    #[command(about = "🚀 Quick gaming setup and discovery")]
    Quick {
        /// Gaming session name
        name: Option<String>,

        /// Auto-detect gaming protocols
        #[arg(long)]
        auto_detect: bool,

        /// Enable family-safe mode
        #[arg(long)]
        family_safe: bool,
    },

    /// 🔍 Gaming service discovery
    #[command(about = "🔍 Discover gaming services and sessions")]
    Discover {
        /// Discovery timeout in seconds
        #[arg(long, default_value = "10")]
        timeout: u64,

        /// Filter by protocol type
        #[arg(long)]
        protocol: Option<String>,

        /// Continuous discovery mode
        #[arg(long)]
        continuous: bool,
    },

    /// ℹ️ Version information
    #[command(about = "ℹ️ Display version and build information")]
    Version {
        /// Show detailed version information
        #[arg(long)]
        detailed: bool,
    },
}

/// Gaming-focused log levels
#[derive(Debug, Clone, PartialEq, Eq, clap::ValueEnum, Default)]
pub enum LogLevel {
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}
