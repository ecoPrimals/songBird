//! Debug CREATE2 - test circuit creation with detailed logging

use songbird_tor_protocol::{Consensus, BeardogCryptoClient};
use songbird_tor_protocol::connection::TorConnection;
use songbird_tor_protocol::protocol::{Cell, CellCommand, CELL_LEN};
use std::time::Duration;
use tracing::{debug, info};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();
    
    println!("=== DEBUG CREATE2 TEST ===");
    
    // 1. Get BearDog client
    let beardog = BeardogCryptoClient::from_env().expect("BearDog client");
    
    // 2. Fetch consensus
    println!("\n1. Fetching consensus...");
    let consensus = match tokio::time::timeout(
        Duration::from_secs(30),
        Consensus::fetch(&beardog)
    ).await {
        Ok(Ok(c)) => c,
        Ok(Err(e)) => { println!("❌ Failed: {}", e); return; }
        Err(_) => { println!("❌ Timeout"); return; }
    };
    println!("   Found {} relays", consensus.relays.len());
    
    // 3. Pick a specific guard (try a different one than lisdex)
    let guard = consensus.relays.iter()
        .find(|r| r.is_guard() && r.nickname != "lisdex")
        .expect("No guard found");
    
    println!("\n2. Selected guard: {} at {}:{}", 
             guard.nickname, guard.address, guard.or_port);
    println!("   Fingerprint (hex): {}", 
             guard.fingerprint.iter().map(|b| format!("{:02x}", b)).collect::<String>());
    
    // 4. Fetch ntor key
    println!("\n3. Fetching ntor key...");
    let mut guard = guard.clone();
    match Consensus::fetch_relay_ntor_key(&guard).await {
        Ok(ntor_key) => {
            guard.ntor_key = Some(ntor_key);
            println!("   ntor key (hex): {}", 
                     ntor_key.iter().map(|b| format!("{:02x}", b)).collect::<String>());
        }
        Err(e) => {
            println!("❌ Failed to fetch ntor key: {}", e);
            return;
        }
    }
    
    // 5. Connect to relay
    println!("\n4. Connecting to relay...");
    let mut conn = TorConnection::new(guard.clone());
    if let Err(e) = conn.connect().await {
        println!("❌ Connection failed: {}", e);
        return;
    }
    println!("   Connected!");
    
    // 6. Generate ephemeral keypair
    println!("\n5. Generating ephemeral X25519 keypair...");
    let ephemeral = beardog.x25519_generate_ephemeral()
        .expect("Failed to generate keypair");
    println!("   Client public key (hex): {}", 
             ephemeral.public_key.iter().map(|b| format!("{:02x}", b)).collect::<String>());
    
    // 7. Build CREATE2 payload
    println!("\n6. Building CREATE2 cell...");
    let ntor_key = guard.ntor_key.unwrap();
    let node_id = &guard.fingerprint;
    
    // ntor HDATA = ID (20) || B (32) || X (32) = 84 bytes
    let mut hdata = Vec::with_capacity(84);
    hdata.extend_from_slice(node_id);
    hdata.extend_from_slice(&ntor_key);
    hdata.extend_from_slice(&ephemeral.public_key);
    
    // CREATE2 payload = HTYPE (2) || HLEN (2) || HDATA
    let mut payload = Vec::new();
    payload.extend_from_slice(&0x0002u16.to_be_bytes()); // HTYPE = ntor
    payload.extend_from_slice(&(hdata.len() as u16).to_be_bytes()); // HLEN
    payload.extend_from_slice(&hdata);
    
    println!("   HTYPE: 0x0002 (ntor)");
    println!("   HLEN: {} bytes", hdata.len());
    println!("   HDATA[0..20] (node_id): {:02x?}", &hdata[0..20]);
    println!("   HDATA[20..52] (ntor_key): {:02x?}", &hdata[20..52]);
    println!("   HDATA[52..84] (client_pk): {:02x?}", &hdata[52..84]);
    
    // Build cell with MSB set for client-initiated circuit
    let circ_id = 0x80000001u32;
    let cell = Cell {
        circ_id,
        command: CellCommand::Create2,
        payload,
    };
    
    let encoded = cell.encode();
    println!("\n7. Encoded cell ({} bytes):", encoded.len());
    println!("   bytes[0..4] (circ_id): {:02x?}", &encoded[0..4]);
    println!("   byte[4] (command): {:02x}", encoded[4]);
    println!("   bytes[5..9] (HTYPE|HLEN): {:02x?}", &encoded[5..9]);
    
    // 8. Send CREATE2
    println!("\n8. Sending CREATE2...");
    if let Err(e) = conn.send_cell(&cell).await {
        println!("❌ Failed to send: {}", e);
        return;
    }
    println!("   Sent!");
    
    // 9. Wait for response
    println!("\n9. Waiting for CREATED2 (30s timeout)...");
    match conn.recv_cell().await {
        Ok(response) => {
            println!("   ✅ Received response!");
            println!("   circ_id: {} (0x{:08x})", response.circ_id, response.circ_id);
            println!("   command: {:?}", response.command);
            println!("   payload[0..20]: {:02x?}", &response.payload[0..20.min(response.payload.len())]);
        }
        Err(e) => {
            println!("   ❌ Failed: {}", e);
        }
    }
}
