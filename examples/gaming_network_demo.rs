use songbird_types::config::consolidated_canonical::CanonicalSongbirdConfig;
//! # Songbird Gaming Network Demo
//!
//! This example demonstrates the high-level Songbird Gaming Network API.
//! It shows how to:
//! - Initialize the gaming network
//! - Create gaming sessions for different games
//! - Monitor session statistics
//! - Handle multiple protocols
//!
//! Run with: `cargo run --example gaming_network_demo`

use songbird_network::{GamingNetwork, GamingNetworkConfig};
use tokio::time::{sleep, Duration};
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    println!("🎮 Songbird Gaming Network Demo");
    println!("================================");

    // Create gaming network with custom configuration
    let config = GamingNetworkConfig {
        enable_detection: true,
        enable_bridging: true,
        enable_nat_traversal: true,
        max_sessions: 50,
    };

    let gaming_network = GamingNetwork::with_config(config).await?;
    println!("✅ Gaming network initialized");

    // Start the gaming network
    gaming_network.start().await?;
    println!("🚀 Gaming network started");

    // Create gaming sessions for different classic games
    let games = vec![
        "StarCraft",
        "Age of Empires 2",
        "Quake",
        "Doom",
        "Counter-Strike 1.6",
        "Warcraft II",
        "Command & Conquer",
    ];

    println!("\n📋 Creating gaming sessions...");
    for game in &games {
        match gaming_network.create_session(game).await {
            Ok(session_info) => {
                println!(
                    "  ✅ {} session created: {} (Protocol: {:?})",
                    game, session_info.id, session_info.protocol
                );
            }
            Err(e) => {
                println!("  ❌ Failed to create {} session: {}", game, e);
            }
        }
        sleep(Duration::from_millis(100)).await;
    }

    // List all active sessions
    println!("\n📊 Active gaming sessions:");
    let sessions = gaming_network.list_sessions().await?;
    for session in &sessions {
        println!(
            "  🎯 {} - {} (State: {:?}, Players: {})",
            session.game_name, session.id, session.state, session.player_count
        );
    }

    // Get network statistics
    println!("\n📈 Network statistics:");
    let stats = gaming_network.get_stats().await?;
    println!("  • Running: {}", stats.running);
    println!("  • Total sessions: {}", stats.total_sessions);
    println!("  • Max sessions: {}", stats.max_sessions);
    println!("  • Protocol distribution:");
    for (protocol, count) in &stats.protocol_distribution {
        println!("    - {}: {} sessions", protocol, count);
    }

    // Simulate some activity
    println!("\n⏳ Simulating gaming network activity...");
    for i in 1..=5 {
        sleep(Duration::from_secs(1)).await;
        println!("  ⚡ Activity cycle {} - Network operating normally", i);
    }

    // Demonstrate session management
    if let Some(session) = sessions.first() {
        println!("\n🔍 Session details for {}:", session.game_name);
        if let Ok(Some(session_info)) = gaming_network.get_session(&session.id).await {
            println!("  • ID: {}", session_info.id);
            println!("  • Game: {}", session_info.game_name);
            println!("  • Protocol: {:?}", session_info.protocol);
            println!("  • State: {:?}", session_info.state);
            println!("  • Players: {}", session_info.player_count);
        }

        // Remove the session
        println!("  🗑️ Removing session...");
        gaming_network.remove_session(&session.id).await?;
        println!("  ✅ Session removed");
    }

    // Final statistics
    println!("\n📊 Final statistics:");
    let final_stats = gaming_network.get_stats().await?;
    println!("  • Total sessions: {}", final_stats.total_sessions);

    // Stop the gaming network
    println!("\n🛑 Stopping gaming network...");
    gaming_network.stop().await?;
    println!("✅ Gaming network stopped successfully");

    println!("\n🎉 Demo completed!");
    println!("\nThe Songbird Gaming Network provides:");
    println!("  ✅ High-level, easy-to-use API");
    println!("  ✅ Support for classic gaming protocols");
    println!("  ✅ Session management and monitoring");
    println!("  ✅ Protocol detection and translation");
    println!("  ✅ Production-ready architecture");

    Ok(())
} 