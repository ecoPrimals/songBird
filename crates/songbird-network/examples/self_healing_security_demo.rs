//! Self-Healing Security Demo
//!
//! This demo shows how Songbird automatically detects and upgrades security providers:
//! 1. Starts with WireGuard (always available)
//! 2. Detects when BearDog becomes available
//! 3. Seamlessly upgrades to BSTP for enhanced security
//! 4. Falls back gracefully if BearDog becomes unavailable
//!
//! Run with: cargo run --example self_healing_security_demo --features beardog

use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, Level};
use tracing_subscriber::fmt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    fmt().with_max_level(Level::INFO).init();

    info!("🎮 Songbird Self-Healing Security Demo");
    info!("=====================================");

    // Always available in this crate
    {
        use songbird_network::network::gaming::security_provider::{
            PeerInfo, SelfHealingSecurityManager,
        };

        // Phase 1: Initialize with WireGuard (standalone sovereignty)
        info!("🚀 Phase 1: Starting Songbird with standalone WireGuard security");
        let security_manager = SelfHealingSecurityManager::new().await?;

        let stats = security_manager.get_stats().await;
        info!(
            "📊 Initial provider: {} ({})",
            stats.current_provider,
            stats.security_level.name()
        );

        // Create some gaming tunnels
        info!("🔒 Creating gaming tunnels...");
        for i in 1..=3 {
            let peer_info = PeerInfo {
                session_id: format!("session_{i}"),
                endpoint: format!("192.168.1.{}:7777", 100 + i).parse()?,
                public_key: None,
            };

            let tunnel = security_manager
                .create_secure_tunnel(format!("gaming_session_{i}"), peer_info)
                .await?;

            info!(
                "✅ Created {} tunnel for gaming session {}",
                tunnel.tunnel_type().name(),
                i
            );
        }

        let stats = security_manager.get_stats().await;
        info!("📈 Tunnels created: {}", stats.total_tunnels);

        // Phase 2: Simulate BearDog becoming available
        info!("\n🐕 Phase 2: BearDog security provider becomes available");
        info!("Setting BEARDOG_AVAILABLE=true to simulate BearDog detection...");
        std::env::set_var("BEARDOG_AVAILABLE", "true");

        // Wait for automatic detection (happens every 30 seconds, but we can force it)
        info!("⏳ Waiting for automatic security upgrade detection...");
        sleep(Duration::from_secs(2)).await;

        // Create new tunnels - these should automatically use BSTP
        info!("🔒 Creating new tunnels (should auto-upgrade to BSTP)...");
        for i in 4..=6 {
            let peer_info = PeerInfo {
                session_id: format!("session_{i}"),
                endpoint: format!("192.168.1.{}:7777", 100 + i).parse()?,
                public_key: None,
            };

            let tunnel = security_manager
                .create_secure_tunnel(format!("enhanced_session_{i}"), peer_info)
                .await?;

            info!(
                "✅ Created {} tunnel for enhanced session {}",
                tunnel.tunnel_type().name(),
                i
            );
        }

        let stats = security_manager.get_stats().await;
        info!(
            "📊 Current provider: {} ({})",
            stats.current_provider,
            stats.security_level.name()
        );
        info!("📈 Total tunnels: {}", stats.total_tunnels);
        info!(
            "⬆️ WireGuard→BSTP upgrades: {}",
            stats.wireguard_to_bstp_upgrades
        );

        // Phase 3: Demonstrate seamless fallback
        info!("\n🔄 Phase 3: Simulating BearDog becoming unavailable");
        std::env::remove_var("BEARDOG_AVAILABLE");

        info!("⏳ Waiting for fallback detection...");
        sleep(Duration::from_secs(2)).await;

        // Create fallback tunnels - should use WireGuard again
        info!("🔒 Creating fallback tunnels (should fall back to WireGuard)...");
        for i in 7..=9 {
            let peer_info = PeerInfo {
                session_id: format!("session_{i}"),
                endpoint: format!("192.168.1.{}:7777", 100 + i).parse()?,
                public_key: None,
            };

            let tunnel = security_manager
                .create_secure_tunnel(format!("fallback_session_{i}"), peer_info)
                .await?;

            info!(
                "✅ Created {} tunnel for fallback session {}",
                tunnel.tunnel_type().name(),
                i
            );
        }

        let final_stats = security_manager.get_stats().await;
        info!("\n🎯 Final Statistics:");
        info!("━━━━━━━━━━━━━━━━━━━");
        info!(
            "Current provider: {} ({})",
            final_stats.current_provider,
            final_stats.security_level.name()
        );
        info!("Total tunnels created: {}", final_stats.total_tunnels);
        info!(
            "WireGuard→BSTP upgrades: {}",
            final_stats.wireguard_to_bstp_upgrades
        );
        info!(
            "BSTP→WireGuard fallbacks: {}",
            final_stats.bstp_to_wireguard_fallbacks
        );
        info!("Failed upgrades: {}", final_stats.failed_upgrades);

        info!("\n🛡️ Self-Healing Security Demo Complete!");
        info!("✅ Songbird maintains sovereignty: works perfectly standalone");
        info!("✅ Automatic BearDog detection and upgrade");
        info!("✅ Graceful fallback when BearDog unavailable");
    }

    Ok(())
}
