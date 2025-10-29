//! Version Command Implementation
//!
//! Shows current version and build information

#![allow(unexpected_cfgs)]

// Module imports
// Version Command
//
// Shows version information about Songbird Orchestrator

use crate::cli::ui;
use crate::errors::CliResult;
use colored::Colorize;
/// Execute the version command
pub async fn execute_version_command(detailed: bool) -> CliResult<()> {
    if detailed {
        show_detailed_version().await
    } else {
        show_simple_version().await
    }
}
/// Show simple version information
pub async fn show_simple_version() -> CliResult<()> {
    println!("🎼 Songbird Orchestrator v{}", env!("CARGO_PKG_VERSION"));
    Ok(())
}

/// Show detailed version information
pub async fn show_detailed_version() -> CliResult<()> {
    println!("🎼 Songbird Orchestrator");
    println!("========================");
    println!();
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!("Build: {} ({})", env!("CARGO_PKG_VERSION"), build_info());
    println!("Authors: {}", env!("CARGO_PKG_AUTHORS"));
    println!("Homepage: {}", env!("CARGO_PKG_HOMEPAGE"));
    println!("Repository: {}", env!("CARGO_PKG_REPOSITORY"));
    println!("Description:");
    println!("  {}", env!("CARGO_PKG_DESCRIPTION"));
    println!("Features enabled:");

    #[cfg(feature = "built-in-observability")]
    println!("  ✅ Built-in observability");
    #[cfg(feature = "prometheus-export")]
    println!("  ✅ Prometheus metrics export");
    #[cfg(feature = "jaeger-tracing")]
    println!("  ✅ Jaeger distributed tracing");
    #[cfg(feature = "production-security")]
    println!("  ✅ Production security features");
    #[cfg(feature = "circuit-breakers")]
    println!("  ✅ Circuit breakers");
    #[cfg(not(any(
        feature = "built-in-observability",
        feature = "prometheus-export",
        feature = "jaeger-tracing",
        feature = "production-security",
        feature = "circuit-breakers"
    )))]
    println!("  ℹ️  Using default feature set");
    println!("System information:");
    println!("  OS: {}", std::env::consts::OS);
    println!("  Architecture: {}", std::env::consts::ARCH);
    println!("  Rust version: {}", build_rust_version());

    Ok(())
}

/// Get build information
fn build_info() -> String {
    format!(
        "{} {}",
        option_env!("VERGEN_GIT_SHA").unwrap_or("unknown"),
        option_env!("VERGEN_BUILD_DATE").unwrap_or("unknown")
    )
}

/// Get Rust version used for build
fn build_rust_version() -> String {
    option_env!("VERGEN_RUSTC_SEMVER").unwrap_or(env!("CARGO_PKG_RUST_VERSION")).to_string()
}

/// Show version information
pub async fn show_version(detailed: bool) -> CliResult<()> {
    let version = env!("CARGO_PKG_VERSION");
    let name = env!("CARGO_PKG_NAME");
    println!("{} v{}", ui::title(name), version.bright_cyan());

    if detailed {
        println!();
        println!("Build Information:");
        println!("  Version: {version}");
        println!("  Target: {}", std::env::consts::ARCH);
        println!("  OS: {}", std::env::consts::OS);

        #[cfg(debug_assertions)]
        println!("  Build Type: Debug");
        #[cfg(not(debug_assertions))]
        println!("  Build Type: Release");

        println!("Features:");
        println!("  - Zero-touch deployment ✓");
        println!("  - Hyper HTTP client ✓");
        println!("  - Service orchestration ✓");
        println!("  - Load balancing ✓");
        println!("  - Built-in observability ✓");
    }

    Ok(())
}
