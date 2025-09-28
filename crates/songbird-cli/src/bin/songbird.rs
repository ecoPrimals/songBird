// Module imports
//! Songbird CLI - The "Docker Moment" for Home Orchestration"
//!
//! Make distributed computing as simple as `songbird init`

use clap::Parser;
use songbird_cli::cli::types::Cli;
use tracing::{error, info};
#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("🎼 Songbird Orchestrator CLI starting");"
    // Parse command line arguments
    let cli = Cli::parse();
    // Execute the CLI command
    if let Err(e) = cli.execute().await {
        error!("❌ Command failed: {}", e);"
        std::process::exit(1);
    }
    info!("✅ Command completed successfully");"
}
