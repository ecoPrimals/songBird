use songbird_gaming_bridge::network::gaming::{GameProtocolClass, GamingManager};
use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 CHAOS DIAGNOSTIC: Testing Real vs Mock Behavior");
    println!("==================================================");

    // Test 1: Empty network scan timing
    println!("\n📡 Test 1: Empty Network Scan");
    let mut gaming_manager = GamingManager::new().await?;

    let start_time = std::time::Instant::now();
    let sessions = gaming_manager.scan_for_games(None).await?;
    let scan_duration = start_time.elapsed();

    println!("⏱️  Scan took: {:?}", scan_duration);
    println!("🎮 Sessions found: {}", sessions.len());

    if scan_duration < Duration::from_millis(100) && !sessions.is_empty() {
        println!("❌ MOCK DETECTED: Instant fake sessions");
        for session in &sessions {
            println!(
                "   Mock session: {:?} ({}% confidence)",
                session.protocol_class,
                session.confidence * 100.0
            );
        }
    } else {
        println!("✅ REAL BEHAVIOR: Proper scan timing");
    }

    // Test 2: UDP socket binding test
    println!("\n🔌 Test 2: UDP Socket Binding");
    let host_addr: SocketAddr = "127.0.0.1:6112".parse().unwrap();
    let session_code = gaming_manager
        .create_lan_session(
            "Test Game".to_string(),
            host_addr,
            GameProtocolClass::IpxBased,
        )
        .await?;

    println!("🎮 Created session: {}", session_code);

    match gaming_manager.start_packet_bridge(&session_code).await {
        Ok(_) => println!("✅ Packet bridge started"),
        Err(e) => println!("❌ Packet bridge failed: {}", e),
    }

    // Try to bind to the same port
    match UdpSocket::bind("127.0.0.1:6112") {
        Ok(_socket) => {
            println!("❌ MOCK DETECTED: No real socket bound (we can bind to same port)");
        }
        Err(e) => {
            println!("✅ REAL BEHAVIOR: Port is actually bound ({})", e);
        }
    }

    // Test 3: Cross-instance session discovery
    println!("\n🌐 Test 3: Cross-Instance Session Discovery");
    let client_manager = GamingManager::new().await?;

    match client_manager.lookup_lan_session(&session_code).await {
        Ok(Some(session)) => {
            println!("❌ MOCK DETECTED: Found session across instances (no real networking)");
            println!(
                "   Session: {} at {}",
                session.game_name, session.host_address
            );
        }
        Ok(None) => {
            println!("✅ REAL BEHAVIOR: Sessions are instance-local (need real discovery)");
        }
        Err(e) => {
            println!("⚠️  Error: {}", e);
        }
    }

    // Test 4: Network interface detection
    println!("\n🌐 Test 4: Network Interface Detection");
    use pnet::datalink;

    let interfaces = datalink::interfaces();
    println!("🌐 Real network interfaces found: {}", interfaces.len());

    for interface in &interfaces[..std::cmp::min(3, interfaces.len())] {
        println!(
            "   • {} ({})",
            interface.name,
            if interface.is_up() { "UP" } else { "DOWN" }
        );
    }

    let start_time = std::time::Instant::now();
    let _sessions = gaming_manager
        .scan_for_games(Some("lo".to_string()))
        .await?;
    let scan_duration = start_time.elapsed();

    if scan_duration < Duration::from_millis(50) {
        println!("❌ MOCK DETECTED: Interface scan too fast");
    } else {
        println!("✅ REAL BEHAVIOR: Interface scan takes time");
    }

    println!("\n🎯 DIAGNOSTIC COMPLETE");
    println!("Summary: Check above for ❌ MOCK DETECTED vs ✅ REAL BEHAVIOR");

    Ok(())
}
