//! Raw byte-level test - use existing TorConnection with enhanced debugging

use songbird_tor_protocol::{Consensus, BeardogCryptoClient};
use songbird_tor_protocol::connection::TorConnection;
use songbird_tor_protocol::directory::RelayInfo;
use songbird_tor_protocol::protocol::{Cell, CellCommand};
use std::time::Duration;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    
    println!("=== RAW TEST WITH NTOR KEY FETCH ===\n");
    
    // Get a relay
    let beardog = BeardogCryptoClient::from_env().expect("BearDog client");
    
    println!("1. Fetching relay from consensus...");
    let consensus = match tokio::time::timeout(
        Duration::from_secs(30),
        Consensus::fetch(&beardog)
    ).await {
        Ok(Ok(c)) => c,
        Ok(Err(e)) => { println!("❌ Failed: {}", e); return; }
        Err(_) => { println!("❌ Timeout"); return; }
    };
    
    // Get a guard - try relays on port 9001 specifically (to test non-443 ports)
    let guards_9001: Vec<_> = consensus.relays.iter()
        .filter(|r| r.is_guard() && r.or_port == 9001)
        .take(5)
        .cloned()
        .collect();
    
    let guards_other: Vec<_> = consensus.relays.iter()
        .filter(|r| r.is_guard() && r.or_port != 443)
        .take(5)
        .cloned()
        .collect();
    
    println!("   Found {} guards on port 9001", guards_9001.len());
    for g in &guards_9001 {
        println!("     - {} at {}:{}", g.nickname, g.address, g.or_port);
    }
    
    // Try a port 9001 guard if available, else any non-443
    let mut guard = guards_9001.first()
        .or_else(|| guards_other.first())
        .cloned()
        .unwrap_or_else(|| consensus.relays.iter().find(|r| r.is_guard()).unwrap().clone());
    
    println!("   Relay: {} at {}:{}", guard.nickname, guard.address, guard.or_port);
    println!("   Fingerprint: {:02x?}", &guard.fingerprint);
    println!("   Has ntor_key: {}", guard.ntor_key.is_some());
    
    // Fetch ntor key if missing
    if guard.ntor_key.is_none() {
        println!("\n   Fetching ntor key from descriptor...");
        match Consensus::fetch_relay_ntor_key(&guard).await {
            Ok(ntor_key) => {
                println!("   ✓ Got ntor key: {:02x?}", &ntor_key[..8]);
                guard.ntor_key = Some(ntor_key);
            }
            Err(e) => {
                println!("   ❌ Failed to fetch ntor key: {}", e);
                return;
            }
        }
    }
    
    // Connect
    println!("\n2. Connecting...");
    let mut conn = TorConnection::new(guard.clone());
    if let Err(e) = conn.connect().await {
        println!("❌ Connect failed: {}", e);
        return;
    }
    println!("   ✓ Connected and link protocol complete");
    
    // Send CREATE2
    println!("\n3. Sending CREATE2...");
    let keypair = beardog.x25519_generate_ephemeral().expect("keygen");
    
    let circ_id: u32 = 0x80000001;
    
    // Build CREATE2 payload manually for maximum clarity
    let mut payload = Vec::new();
    // HTYPE = ntor (0x0002)
    payload.extend_from_slice(&[0x00, 0x02]);
    // HLEN = 84 (0x0054)
    payload.extend_from_slice(&[0x00, 0x54]);
    // HDATA:
    //   node_id (20 bytes) = fingerprint
    payload.extend_from_slice(&guard.fingerprint);
    //   B (32 bytes) = relay's ntor key
    let ntor_key = guard.ntor_key.as_ref().expect("ntor_key");
    payload.extend_from_slice(ntor_key);
    //   X (32 bytes) = client ephemeral pubkey
    payload.extend_from_slice(&keypair.public_key);
    
    assert_eq!(payload.len(), 4 + 84, "CREATE2 payload should be 88 bytes");
    
    let cell = Cell {
        circ_id,
        command: CellCommand::Create2,
        payload,
    };
    
    println!("   circ_id: {} (0x{:08x})", circ_id, circ_id);
    println!("   payload len: {}", cell.payload.len());
    println!("   HTYPE: 0x{:02x}{:02x}", cell.payload[0], cell.payload[1]);
    println!("   HLEN: {} (0x{:02x}{:02x})", 
             u16::from_be_bytes([cell.payload[2], cell.payload[3]]),
             cell.payload[2], cell.payload[3]);
    println!("   node_id[0..8]: {:02x?}", &cell.payload[4..12]);
    println!("   B[0..8]: {:02x?}", &cell.payload[24..32]);
    println!("   X[0..8]: {:02x?}", &cell.payload[56..64]);
    
    if let Err(e) = conn.send_cell(&cell).await {
        println!("❌ Send failed: {}", e);
        return;
    }
    println!("   ✓ Sent");
    
    // Wait for response
    println!("\n4. Waiting for response...");
    match conn.recv_cell().await {
        Ok(response) => {
            println!("   ✅ Got response!");
            println!("   circ_id: {} (0x{:08x})", response.circ_id, response.circ_id);
            println!("   command: {:?}", response.command);
            println!("   payload len: {}", response.payload.len());
            if response.payload.len() >= 20 {
                println!("   payload[0..20]: {:02x?}", &response.payload[..20]);
            }
        }
        Err(e) => {
            println!("   ❌ Failed: {}", e);
        }
    }
}
