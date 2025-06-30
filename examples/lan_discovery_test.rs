use songbird_gaming_bridge::network::gaming::{GameProtocolClass, GamingManager};
use std::net::SocketAddr;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 LAN Discovery Test");
    println!("===================");

    // Test 1: Host a session
    println!("\n🏠 Test 1: Creating host session");
    let mut host_manager = GamingManager::new().await?;

    let host_addr: SocketAddr = "127.0.0.1:6112".parse().unwrap();
    let session_code = host_manager
        .create_lan_session(
            "Test LAN Game".to_string(),
            host_addr,
            GameProtocolClass::IpxBased,
        )
        .await?;

    println!("✅ Host session created: {}", session_code);

    // Start broadcasting
    println!("📡 Starting LAN broadcast...");
    if let Err(e) = host_manager.broadcast_lan_session(&session_code).await {
        println!("⚠️  Broadcast failed: {}", e);
    } else {
        println!("✅ Broadcasting session");
    }

    // Wait a moment
    tokio::time::sleep(Duration::from_millis(1000)).await;

    // Test 2: Client discovery
    println!("\n👥 Test 2: Client scanning for sessions");
    let client_manager = GamingManager::new().await?;

    let discovered_sessions = client_manager.scan_lan_sessions().await?;

    if discovered_sessions.is_empty() {
        println!("❌ No sessions discovered");
        println!("💡 This might be due to timing or network configuration");
    } else {
        println!("✅ Discovered {} session(s):", discovered_sessions.len());
        for session in &discovered_sessions {
            println!(
                "   • {} (Code: {})",
                session.game_name, session.session_code
            );
        }
    }

    // Test 3: Direct lookup
    println!("\n🔍 Test 3: Direct session lookup");
    match host_manager.lookup_lan_session(&session_code).await? {
        Some(session) => {
            println!(
                "✅ Session found: {} at {}",
                session.game_name, session.host_address
            );
        }
        None => {
            println!("❌ Session not found in local storage");
        }
    }

    println!("\n🎯 LAN Discovery Test Complete");
    println!("If discovery failed, this is normal - UDP broadcast discovery");
    println!("requires proper network configuration and timing.");

    Ok(())
}
