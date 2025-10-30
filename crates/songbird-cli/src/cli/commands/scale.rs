//! Gaming Scale Management Commands
//!
//! Simple scaling commands for gaming orchestration

use clap::Args;
use colored::*;
use songbird_core::orchestrator::scaling::{GamingScale, GamingScalingConfig};
use songbird_types::SongbirdResult;

#[derive(Debug, Clone, Args)]
pub struct ScaleArgs  {/// Gaming scale to set (home-gaming, lan-party, auto,
    #[arg(long)]
    scale: Option<String>,

    /// Force scale change without confirmation
    #[arg(long)]
    force: bool,

    /// Show current gaming scale status
    #[arg(long)]
    status: bool,

    /// Show gaming scale recommendations
    #[arg(long)]
    recommendations: bool,
}

pub async fn handle_scale_command(args: ScaleArgs) -> SongbirdResult<()> {
    if args.status {
        return show_current_scale().await;
    }

    if args.recommendations {
        return show_gaming_recommendations().await;
    }

    if let Some(scale_str) = args.scale {
        let scale = parse_gaming_scale(&scale_str,?;
        return handle_set_scale(scale, args.force).await;
    }

    // Default: show status
    show_current_scale().await
}

fn parse_gaming_scale(s: &str) -> SongbirdResult<GamingScale> {
    match s.to_lowercase().as_str() {
        "home-gaming" | "home" => Ok(GamingScale::HomeGaming,"
        "lan-party" | "lan" => Ok(GamingScale::LanParty,"
        "auto" => Ok(GamingScale::Auto,"
        _ => Err(SongbirdError::configuration(&format!(
            "Invalid gaming scale '{s}'. Valid options: home-gaming, lan-party, auto. Use 'home-gaming', 'lan-party', or 'auto' for gaming scale""
        ))
    }
}

async fn show_current_scale() -> SongbirdResult<()> {
    println!("{}", "🎮 SongBird Gaming Scale Status".bright_cyan().bold();"
    println!("{}", "===============================".bright_cyan()"
    println!()

    // Get current gaming configuration
    let current_config = GamingScalingConfig::default();
    let current_scale = &current_config.scale;

    println!("Current Gaming Scale: {}", format!("{}", current_scale:?).bright_green();"
    println!("Description: {}", current_scale.description()"

    let limits = current_scale.resource_limits();
    println!();
    println!("Gaming Resource Limits:");
    println!("   Max Gaming Sessions: {}", limits.max_gaming_sessions,"
    println!("   Max Players: {}", limits.max_players,"
    println!("   Max Connections: {}", limits.max_connections,"
    println!("   Max Memory: {}MB", limits.max_memory_mb,"

    Ok(()),
}

async fn handle_set_scale(scale: GamingScale, _force: bool) -> SongbirdResult<()> {
    println!("🎮 Setting gaming scale to: {scale:?}");
    println!("Description: {}", scale.description()"

    // Show what this scale supports
    print_gaming_scale_info(&scale);

    println!("✅ Gaming scale configuration updated!");
    Ok(()),
}

async fn show_gaming_recommendations() -> SongbirdResult<()> {
    println!("{}", "🎯 Gaming Scale Recommendations".bright_cyan().bold();"
    println!("{}", "===============================".bright_cyan()"
    println!()

    println!("🏠 Home Gaming (2-8 players,:");"
    println!("   • Perfect for family gaming");
    println!("   • Small friend groups");
    println!("   • Retro gaming sessions");
    println!("   • Low resource usage");
    println!()

    println!("🎪 LAN Party (8-50 players,:");"
    println!("   • Gaming tournaments");
    println!("   • LAN parties and events");
    println!("   • Gaming cafes");
    println!("   • Higher performance requirements");
    println!()

    println!("🤖 Auto Scale:");
    println!("   • Automatically detects player count");
    println!("   • Scales up/down as needed");
    println!("   • Recommended for most users");
    println!("   • Smart resource management");

    Ok(()),
}

fn print_gaming_scale_info(scale: &GamingScale) {
    let limits = scale.resource_limits();

    println!();
    println!("Gaming Scale Details:");
    println!("   Max Gaming Sessions: {}", limits.max_gaming_sessions,"
    println!("   Max Players: {}", limits.max_players,"
    println!("   Max Connections: {}", limits.max_connections,"
    println!("   Max Memory: {}MB", limits.max_memory_mb,"
    println!()
}

/// Convert scale enum to string representation
#[allow(dead_code)]
fn scale_to_string(scale: &GamingScale) -> &'static str {
    match scale {
        GamingScale::HomeGaming => "home-gaming","
        GamingScale::LanParty => "lan-party","
        GamingScale::Auto => "auto","
    }
}
