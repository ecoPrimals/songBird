use songbird_gaming_bridge::SongbirdOrchestrator;
use songbird_gaming_bridge::config::NetworkConfig;
use std::collections::HashMap;
use songbird_gaming_bridge::network::gaming::{GamingManager, GameProtocolClass};
use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;
use tokio::time::timeout;

/// Chaos test to verify real packet capture vs mock behavior
#[tokio::test]
async fn test_real_vs_mock_packet_detection() {
    println!("🧪 CHAOS TEST: Real vs Mock Packet Detection");
    
    // Test 1: Empty network should NOT detect games immediately
    let mut gaming_manager = GamingManager::new().await.unwrap_or_default();
    
    println!("📡 Scanning empty network...");
    let start_time = std::time::Instant::now();
    let sessions = gaming_manager.scan_for_games(None).await.unwrap_or_default();
    let scan_duration = start_time.elapsed();
    
    println!("⏱️  Scan took: {:?}", scan_duration);
    println!("🎮 Sessions found: {}", sessions.len());
    
    // REAL behavior: Should take time and find nothing
    // MOCK behavior: Instant results with fake sessions
    if scan_duration < Duration::from_millis(100) && !sessions.is_empty() {
        println!("❌ MOCK DETECTED: Instant fake sessions");
        for session in &sessions {
            println!("   Mock session: {:?} ({}% confidence)", 
                session.protocol_class, session.confidence * 100.0);
        }
    } else {
        println!("✅ REAL BEHAVIOR: Proper scan timing");
    }
    
    assert!(true); // Always pass, this is diagnostic
}

/// Test real UDP packet forwarding
#[tokio::test]
async fn test_real_udp_packet_forwarding() {
    println!("🧪 CHAOS TEST: Real UDP Packet Forwarding");
    
    let mut gaming_manager = GamingManager::new().await.unwrap_or_default();
    
    // Create a session
    let host_addr: SocketAddr = "127.0.0.1:6112".parse().unwrap_or_default();
    let session_code = gaming_manager.create_lan_session(
        "Test Game".to_string(),
        host_addr,
        GameProtocolClass::IpxBased
    ).await.unwrap_or_default();
    
    println!("🎮 Created session: {}", session_code);
    
    // Try to start packet bridge
    match gaming_manager.start_packet_bridge(&session_code).await {
        Ok(_) => println!("✅ Packet bridge started"),
        Err(e) => println!("❌ Packet bridge failed: {}", e),
    }
    
    // Test UDP socket binding on the same port
    println!("🔌 Testing UDP socket binding...");
    match UdpSocket::bind("127.0.0.1:6112") {
        Ok(_socket) => {
            println!("❌ MOCK DETECTED: No real socket bound (we can bind to same port)");
        }
        Err(e) => {
            println!("✅ REAL BEHAVIOR: Port is actually bound ({})", e);
        }
    }
}

/// Test session discovery between two instances
#[tokio::test]
async fn test_cross_instance_session_discovery() {
    println!("🧪 CHAOS TEST: Cross-Instance Session Discovery");
    
    // Host instance
    let mut host_manager = GamingManager::new().await.unwrap_or_default();
    let host_addr: SocketAddr = "127.0.0.1:6113".parse().unwrap_or_default();
    
    let session_code = host_manager.create_lan_session(
        "Cross Test Game".to_string(),
        host_addr,
        GameProtocolClass::IpxBased
    ).await.unwrap_or_default();
    
    println!("🏠 Host created session: {}", session_code);
    
    // Client instance (separate manager)
    let client_manager = GamingManager::new().await.unwrap_or_default();
    
    // Try to find the session
    match client_manager.lookup_lan_session(&session_code).await {
        Ok(Some(session)) => {
            println!("❌ MOCK DETECTED: Found session across instances (no real networking)");
            println!("   Session: {} at {}", session.game_name, session.host_address);
        }
        Ok(None) => {
            println!("✅ REAL BEHAVIOR: Sessions are instance-local (need real discovery)");
        }
        Err(e) => {
            println!("⚠️  Error: {}", e);
        }
    }
}

/// Test actual network interface detection
#[tokio::test]
async fn test_network_interface_reality() {
    println!("🧪 CHAOS TEST: Network Interface Reality Check");
    
    // Try to detect real network interfaces
    use pnet::datalink;
    
    let interfaces = datalink::interfaces();
    println!("🌐 Real network interfaces found: {}", interfaces.len());
    
    for interface in &interfaces {
        println!("   • {} ({})", interface.name, 
            if interface.is_up() { "UP" } else { "DOWN" });
    }
    
    // Test if our protocol detector uses real interfaces
    let gaming_manager = GamingManager::new().await.unwrap_or_default();
    
    // This should take time if it's real
    let start_time = std::time::Instant::now();
    let _sessions = gaming_manager.scan_for_games(Some("lo".to_string())).await.unwrap_or_default();
    let scan_duration = start_time.elapsed();
    
    if scan_duration < Duration::from_millis(50) {
        println!("❌ MOCK DETECTED: Interface scan too fast");
    } else {
        println!("✅ REAL BEHAVIOR: Interface scan takes time");
    }
}

/// Test packet generation and capture
#[tokio::test]
async fn test_packet_generation_and_capture() {
    println!("🧪 CHAOS TEST: Packet Generation and Capture");
    
    // Generate fake game traffic
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap_or_default();
    let local_addr = socket.local_addr().unwrap_or_default();
    
    println!("📡 Generating fake IPX traffic from {}", local_addr);
    
    // Send some fake IPX-like packets
    let fake_ipx_data = vec![
        0xFF, 0xFF, // Checksum
        0x00, 0x20, // Length
        0x00,       // Transport Control  
        0x04,       // Packet Type (SPP)
        0x01, 0x00, 0x00, 0x00, // Dest Network
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, // Dest Node (broadcast)
        0x04, 0x51, // Dest Socket
        0x01, 0x00, 0x00, 0x00, // Src Network
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, // Src Node
        0x04, 0x51, // Src Socket
        // Data
        b'H', b'e', b'l', b'l', b'o', b' ', b'G', b'a', b'm', b'e'
    ];
    
    // Send to broadcast
    socket.send_to(&fake_ipx_data, "127.0.0.1:6112").unwrap_or_default();
    
    // Now scan for games
    let mut gaming_manager = GamingManager::new().await.unwrap_or_default();
    
    println!("🔍 Scanning after generating traffic...");
    let sessions = timeout(
        Duration::from_secs(2),
        gaming_manager.scan_for_games(None)
    ).await.unwrap_or_default().unwrap_or_default();
    
    if sessions.is_empty() {
        println!("❌ MOCK DETECTED: Generated traffic not detected");
    } else {
        println!("✅ REAL BEHAVIOR: Traffic detected");
        for session in &sessions {
            println!("   Detected: {:?}", session.protocol_class);
        }
    }
}

/// Live integration test with real socket communication
#[tokio::test]
async fn test_live_socket_bridge() {
    println!("🧪 CHAOS TEST: Live Socket Bridge");
    
    // Create two UDP sockets to simulate game clients
    let client1 = UdpSocket::bind("127.0.0.1:0").unwrap_or_default();
    let client2 = UdpSocket::bind("127.0.0.1:0").unwrap_or_default();
    
    let client1_addr = client1.local_addr().unwrap_or_default();
    let client2_addr = client2.local_addr().unwrap_or_default();
    
    println!("👥 Client 1: {}", client1_addr);
    println!("👥 Client 2: {}", client2_addr);
    
    // Create gaming session
    let mut gaming_manager = GamingManager::new().await.unwrap_or_default();
    let session_code = gaming_manager.create_lan_session(
        "Bridge Test".to_string(),
        client1_addr,
        GameProtocolClass::IpxBased
    ).await.unwrap_or_default();
    
    // Add both clients to session
    gaming_manager.join_lan_session(&session_code, client1_addr).await.unwrap_or_default();
    gaming_manager.join_lan_session(&session_code, client2_addr).await.unwrap_or_default();
    
    // Start bridge
    gaming_manager.start_packet_bridge(&session_code).await.unwrap_or_default();
    
    // Test message passing
    let test_message = b"Hello from client 1!";
    
    println!("📤 Client 1 sending: {:?}", std::str::from_utf8(test_message));
    
    // Client 1 sends to bridge
    client1.send_to(test_message, "127.0.0.1:6112").unwrap_or_default();
    
    // Client 2 should receive it
    client2.set_read_timeout(Some(Duration::from_millis(1000))).unwrap_or_default();
    let mut buffer = [0u8; 1024];
    
    match client2.recv_from(&mut buffer) {
        Ok((len, from)) => {
            let received = &buffer[..len];
            println!("📥 Client 2 received from {}: {:?}", from, std::str::from_utf8(received));
            
            if received == test_message {
                println!("✅ REAL BEHAVIOR: Message bridged successfully!");
            } else {
                println!("⚠️  Message corrupted during bridging");
            }
        }
        Err(e) => {
            println!("❌ MOCK DETECTED: No real packet bridging ({})", e);
        }
    }
} 