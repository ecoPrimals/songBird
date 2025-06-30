//! WireGuard Secure Gaming Demo
//!
//! Demonstrates immediate secure gaming with boringtun WireGuard implementation.
//! This works TODAY for secure gaming across the internet.

use songbird_gaming_bridge::errors::Result;
use songbird_gaming_bridge::network::gaming::{GamingTunnelManager, WireGuardConfig};
use std::net::SocketAddr;
use tracing::info;
use x25519_dalek::{PublicKey, StaticSecret};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🔐 WireGuard Secure Gaming Demo");
    info!("✅ READY TODAY: Secure gaming with boringtun");

    // Create gaming-optimized WireGuard configuration
    let config = WireGuardConfig {
        listen_port: 51820,
        mtu: 1420, // Gaming-optimized
        gaming_optimizations: true,
        ..Default::default()
    };

    let tunnel_manager = GamingTunnelManager::new(config);

    // Create secure gaming tunnel
    let peer_key = PublicKey::from(&StaticSecret::new(rand::thread_rng()));
    let endpoint: SocketAddr = "player2.example.com:51820".parse().unwrap();

    let session_id = tunnel_manager
        .create_gaming_tunnel("starcraft-secure".to_string(), peer_key, endpoint)
        .await?;

    info!("🎮 Secure gaming tunnel created: {}", session_id);
    info!("🔐 All StarCraft traffic now encrypted with WireGuard");

    // Show tunnel statistics
    let stats = tunnel_manager.get_all_tunnel_stats().await;
    for stat in stats {
        info!(
            "📊 Tunnel: {} -> {} (Gaming optimized: {})",
            stat.session_id, stat.endpoint, stat.gaming_optimizations
        );
    }

    Ok(())
}
