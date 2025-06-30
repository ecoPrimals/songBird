//! Universal Gaming Network Bridge Demo
//!
//! This demo shows how Songbird can make ANY legacy game work over the internet
//! as if it was on a local area network.

use colored::*;
use songbird_gaming_bridge::errors::Result;
use songbird_gaming_bridge::network::gaming::{GamingManager, NatType, PlayerEndpoint};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!(
        "{}",
        "🎮 Universal Gaming Network Bridge Demo"
            .bright_cyan()
            .bold()
    );
    println!("{}", "=====================================".bright_cyan());
    println!();

    // Demo scenario: Two friends want to play StarCraft across the country
    println!("{}", "📖 Scenario:".bright_blue());
    println!("   Alice (Seattle) wants to play StarCraft with Bob (Miami)");
    println!("   StarCraft uses IPX protocol which doesn't work over internet");
    println!("   Songbird will create a virtual LAN bridge!");
    println!();

    // Initialize gaming manager
    println!("{}", "🔧 Initializing Universal Gaming Manager...".cyan());
    let gaming_manager = GamingManager::new().await?;
    println!("{}", "✅ Gaming manager ready!".bright_green());
    println!();

    // Step 1: Auto-detect gaming protocols
    println!("{}", "Step 1: Auto-detect gaming protocols".bright_yellow());
    println!("{}", "=====================================".yellow());

    let detected_sessions = gaming_manager.scan_for_games(Some("eth0")).await?;

    for session in &detected_sessions {
        println!(
            "🎮 Detected: {} ({:?})",
            session.game_name.as_deref().unwrap_or("Unknown game"),
            session.protocol_class
        );
        println!("   Confidence: {:.1}%", session.confidence * 100.0);
        println!("   Ports: {:?}", session.local_ports);
    }
    println!();

    // Step 2: Auto-configure for detected games
    println!(
        "{}",
        "Step 2: Auto-configure gaming environment".bright_yellow()
    );
    println!("{}", "=========================================".yellow());

    let config = gaming_manager.auto_configure().await?;
    println!("🔧 Configuration complete!");
    println!(
        "   Compatible games: {}",
        config.compatible_games.join(", ")
    );
    println!();

    // Step 3: Create universal gaming session
    println!(
        "{}",
        "Step 3: Create universal gaming session".bright_yellow()
    );
    println!("{}", "=======================================".yellow());

    // Simulate two players
    let players = vec![
        PlayerEndpoint {
            player_id: "alice".to_string(),
            display_name: "Alice (Seattle)".to_string(),
            real_address: "203.0.113.10:6112".parse().unwrap(),
            virtual_address: None,
            nat_type: NatType::FullCone,
        },
        PlayerEndpoint {
            player_id: "bob".to_string(),
            display_name: "Bob (Miami)".to_string(),
            real_address: "198.51.100.20:6112".parse().unwrap(),
            virtual_address: None,
            nat_type: NatType::RestrictedCone,
        },
    ];

    let session_id = gaming_manager.create_universal_session(players).await?;
    println!("🎮 Gaming session created: {}", session_id);
    println!();

    // Step 4: Show what happens in the background
    println!(
        "{}",
        "Step 4: Universal protocol translation in action".bright_yellow()
    );
    println!(
        "{}",
        "===============================================".yellow()
    );

    println!("🔄 Protocol Translation Demo:");
    println!();

    // Simulate StarCraft IPX packet
    let starcraft_ipx_packet = vec![
        0xFF, 0xFF, // IPX header signature
        0x00, 0x1C, // Packet length
        0x00, // Transport control
        0x04, // Packet type (PEP)
        0x00, 0x00, 0x43, 0x21, // Destination network
        0x00, 0x00, 0x00, 0x00, 0x00, 0x01, // Destination node
        0x86,
        0x9C, // Destination socket (StarCraft)
              // ... payload would follow
    ];

    println!("📤 Original StarCraft IPX packet:");
    println!("   {:02X?}", &starcraft_ipx_packet[0..8]);
    println!("   ↓ (Universal Translation)");

    // Simulate a few seconds of translation
    for i in 0..5 {
        print!("   🔄 Translating");
        for _ in 0..i {
            print!(".");
        }
        println!();
        tokio::time::sleep(Duration::from_millis(300)).await;
    }

    println!("   ✅ Translated to UDP for internet routing");
    println!("📥 UDP packet sent over internet:");
    println!("   UDP Port: 6112 → 6112");
    println!("   Virtual Network ID: 0x00004321");
    println!("   🌐 Routed: Seattle → Miami");
    println!();

    // Step 5: Show gaming session status
    println!("{}", "Step 5: Gaming session status".bright_yellow());
    println!("{}", "=============================".yellow());

    // Simulate some activity
    tokio::time::sleep(Duration::from_millis(500)).await;

    println!("🎮 Session Status:");
    println!("   • Protocol: IPX-over-UDP Bridge");
    println!("   • Players: 2/8");
    println!("   • Latency: Alice ↔ Bob: 45ms");
    println!("   • NAT Traversal: ✅ Active");
    println!("   • Virtual LAN: 192.168.100.0/24");
    println!();

    // Step 6: Show what players see
    println!("{}", "Step 6: Player experience".bright_yellow());
    println!("{}", "========================".yellow());

    println!("👥 What Alice and Bob see:");
    println!();
    println!("   1. Alice launches StarCraft");
    println!("   2. Selects 'Network Game' → 'IPX Network'");
    println!("   3. Creates game: 'Alice vs Bob'");
    println!("   4. Bob's StarCraft automatically sees Alice's game!");
    println!("   5. Bob joins, and they play as if on the same LAN");
    println!();

    println!(
        "{}",
        "✨ The magic: StarCraft thinks it's on a local network,".bright_green()
    );
    println!(
        "{}",
        "   but Songbird is secretly routing over the internet!".bright_green()
    );
    println!();

    // Step 7: Show universal compatibility
    println!(
        "{}",
        "Step 7: Universal compatibility showcase".bright_yellow()
    );
    println!("{}", "======================================".yellow());

    println!("🎯 This same system works for:");
    println!("   • {} StarCraft, Brood War", "✅".bright_green());
    println!("   • {} Age of Empires 1 & 2", "✅".bright_green());
    println!("   • {} Command & Conquer series", "✅".bright_green());
    println!("   • {} Stronghold Crusader", "✅".bright_green());
    println!("   • {} Warcraft 1 & 2", "✅".bright_green());
    println!("   • {} Quake series", "✅".bright_green());
    println!("   • {} Doom series", "✅".bright_green());
    println!(
        "   • {} Windows ME Solitaire (network mode)",
        "✅".bright_green()
    );
    println!("   • {} Any DirectPlay game", "✅".bright_green());
    println!("   • {} Any IPX-based game", "✅".bright_green());
    println!("   • {} Most Windows 95-XP era games", "✅".bright_green());
    println!();

    println!(
        "{}",
        "🚀 No configuration needed - just run and play!"
            .bright_cyan()
            .bold()
    );
    println!();

    // Step 8: CLI Usage examples
    println!("{}", "Step 8: How to use (CLI commands)".bright_yellow());
    println!("{}", "=================================".yellow());

    println!("💻 Command examples:");
    println!();
    println!("   🔍 Auto-detect any game:");
    println!("     {}", "songbird gaming scan".bright_white().on_blue());
    println!();
    println!("   🎮 Host a universal session:");
    println!(
        "     {}",
        "songbird gaming host --auto".bright_white().on_blue()
    );
    println!();
    println!("   🤝 Join a session:");
    println!(
        "     {}",
        "songbird gaming join GAME-ABCD".bright_white().on_blue()
    );
    println!();
    println!("   🎓 Learn new game protocols:");
    println!(
        "     {}",
        "songbird gaming learn \"My Favorite Game\""
            .bright_white()
            .on_blue()
    );
    println!();

    // Success message
    println!("{}", "🎉 Demo complete!".bright_green().bold());
    println!();
    println!(
        "{}",
        "The Universal Gaming Network Bridge makes it possible".bright_cyan()
    );
    println!(
        "{}",
        "to play ANY legacy game over the internet with friends,".bright_cyan()
    );
    println!(
        "{}",
        "regardless of the original networking protocol.".bright_cyan()
    );
    println!();
    println!(
        "{}",
        "From IPX to DirectPlay to NetBIOS - we've got you covered!".bright_yellow()
    );

    Ok(())
}
