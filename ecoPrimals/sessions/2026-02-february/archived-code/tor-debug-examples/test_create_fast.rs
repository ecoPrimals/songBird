//! Test CREATE_FAST - simpler circuit creation without ntor

use songbird_tor_protocol::{Consensus, BeardogCryptoClient};
use songbird_tor_protocol::connection::TorConnection;
use songbird_tor_protocol::protocol::{Cell, CellCommand};
use std::time::Duration;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    
    println!("=== TEST CREATE_FAST ===");
    println!("Testing simpler circuit creation (no ntor handshake)\n");
    
    // 1. Get BearDog client
    let beardog = BeardogCryptoClient::from_env().expect("BearDog client");
    
    // 2. Fetch consensus to find a relay
    println!("1. Fetching consensus...");
    let consensus = match tokio::time::timeout(
        Duration::from_secs(30),
        Consensus::fetch(&beardog)
    ).await {
        Ok(Ok(c)) => c,
        Ok(Err(e)) => { println!("❌ Failed: {}", e); return; }
        Err(_) => { println!("❌ Timeout"); return; }
    };
    
    // 3. Pick any relay
    let guard = consensus.relays.iter()
        .find(|r| r.is_guard())
        .expect("No guard found");
    
    println!("   Guard: {} at {}:{}", 
             guard.nickname, guard.address, guard.or_port);
    
    // 4. Connect
    println!("\n2. Connecting...");
    let mut conn = TorConnection::new(guard.clone());
    if let Err(e) = conn.connect().await {
        println!("❌ Failed: {}", e);
        return;
    }
    println!("   Connected!");
    
    // 5. Build CREATE_FAST cell
    // CREATE_FAST payload is just 20 random bytes (client's key material)
    println!("\n3. Sending CREATE_FAST cell...");
    
    let mut fast_payload = [0u8; 20];
    // Use some random-ish bytes
    for i in 0..20 {
        fast_payload[i] = (i as u8).wrapping_mul(7).wrapping_add(42);
    }
    
    let circ_id = 0x80000001u32; // MSB set for client-initiated
    
    let cell = Cell {
        circ_id,
        command: CellCommand::CreateFast,
        payload: fast_payload.to_vec(),
    };
    
    println!("   circ_id: {} (0x{:08x})", circ_id, circ_id);
    println!("   command: CreateFast (5)");
    println!("   payload (20 bytes): {:02x?}", &fast_payload);
    
    if let Err(e) = conn.send_cell(&cell).await {
        println!("❌ Send failed: {}", e);
        return;
    }
    println!("   Sent!");
    
    // 6. Wait for CREATED_FAST response
    println!("\n4. Waiting for CREATED_FAST response...");
    match conn.recv_cell().await {
        Ok(response) => {
            println!("   ✅ Received response!");
            println!("   circ_id: {} (0x{:08x})", response.circ_id, response.circ_id);
            println!("   command: {:?}", response.command);
            if response.payload.len() >= 20 {
                println!("   server key material: {:02x?}", &response.payload[0..20]);
            }
        }
        Err(e) => {
            println!("   ❌ Failed: {}", e);
        }
    }
}
