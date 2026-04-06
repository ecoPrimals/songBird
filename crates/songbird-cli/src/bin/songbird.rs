// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

// Module imports
//! Songbird CLI - The "Docker Moment" for Home Orchestration"
//!
//! Make distributed computing as simple as `songbird init`

use clap::Parser;
use songbird_cli::cli::types::Cli;
use tracing::{error, info};

#[tokio::main]
async fn main() {
    // ✅ FIX: Parse arguments FIRST (before any initialization)
    // This allows --help and --version to exit immediately
    let cli = Cli::parse();

    // Only initialize tracing AFTER arg parsing
    // This prevents hangs when user just wants help/version
    tracing_subscriber::fmt::init();

    info!("🎼 Songbird Orchestrator CLI starting");

    // Execute the CLI command
    if let Err(e) = cli.execute().await {
        error!("❌ Command failed: {}", e);
        std::process::exit(1);
    }

    info!("✅ Command completed successfully");
}
