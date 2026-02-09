//! Deliberate test with explicit delays and verbose logging

use songbird_tor_protocol::{Consensus, BeardogCryptoClient};
use songbird_tor_protocol::connection::TorConnection;
use songbird_tor_protocol::protocol::{Cell, CellCommand};
use std::time::Duration;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();
    
    println!("=== DELIBERATE CIRCUIT TEST ===\n");
    
    let beardog = BeardogCryptoClient::from_env().expect("BearDog client");
    
    // 1. Get consensus
    println!("1. Fetching consensus...");
    let consensus = Consensus::fetch(&beardog).await.expect("consensus");
    println!("   Got {} relays", consensus.relays.len());
    
    // 2. Pick a guard
    let mut guard = consensus.relays.iter()
        .find(|r| r.is_guard())
        .unwrap()
        .clone();
    println!("\n2. Selected guard: {} at {}:{}", guard.nickname, guard.address, guard.or_port);
    
    // 3. Fetch ntor key BEFORE connecting
    println!("\n3. Fetching ntor key...");
    if guard.ntor_key.is_none() {
        let ntor = Consensus::fetch_relay_ntor_key(&guard).await.expect("ntor key");
        println!("   Got ntor key: {:02x?}...", &ntor[..8]);
        guard.ntor_key = Some(ntor);
    }
    
    // 4. Wait a moment
    println!("\n4. Waiting 1 second before connecting...");
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    // 5. Connect
    println!("\n5. Connecting to relay...");
    let mut conn = TorConnection::new(guard.clone());
    conn.connect().await.expect("connect");
    println!("   Connected and link protocol complete!");
    
    // 6. Wait after connect
    println!("\n6. Waiting 2 seconds after connect...");
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // 7. Generate ephemeral key
    println!("\n7. Generating ephemeral X25519 keypair...");
    let keypair = beardog.x25519_generate_ephemeral().expect("keypair");
    println!("   Client pubkey: {:02x?}...", &keypair.public_key[..8]);
    
    // 8. Build CREATE2 cell
    println!("\n8. Building CREATE2 cell...");
    let circ_id: u32 = 0x80000001;
    
    let mut payload = Vec::with_capacity(88);
    // HTYPE = ntor (2)
    payload.extend_from_slice(&2u16.to_be_bytes());
    // HLEN = 84
    payload.extend_from_slice(&84u16.to_be_bytes());
    // NODEID = fingerprint (20 bytes)
    payload.extend_from_slice(&guard.fingerprint);
    // KEYID = ntor key (32 bytes)
    payload.extend_from_slice(guard.ntor_key.as_ref().unwrap());
    // CLIENT_PK = our ephemeral (32 bytes)
    payload.extend_from_slice(&keypair.public_key);
    
    println!("   Payload size: {} bytes", payload.len());
    println!("   HTYPE: 0x{:04x}", 2u16);
    println!("   HLEN: {}", 84u16);
    println!("   NODEID: {:02x?}", &guard.fingerprint);
    println!("   KEYID: {:02x?}", guard.ntor_key.as_ref().unwrap());
    println!("   CLIENT_PK: {:02x?}", &keypair.public_key);
    
    let cell = Cell {
        circ_id,
        command: CellCommand::Create2,
        payload,
    };
    
    // 9. Wait before sending
    println!("\n9. Waiting 1 second before sending CREATE2...");
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    // 10. Send CREATE2
    println!("\n10. Sending CREATE2 cell...");
    conn.send_cell(&cell).await.expect("send");
    println!("    Sent!");
    
    // 11. Wait for response
    println!("\n11. Waiting for CREATED2 response (30s)...");
    match tokio::time::timeout(Duration::from_secs(30), conn.recv_cell()).await {
        Ok(Ok(response)) => {
            println!("   ✅ SUCCESS! Got response:");
            println!("      circ_id: {} (0x{:08x})", response.circ_id, response.circ_id);
            println!("      command: {:?}", response.command);
            println!("      payload len: {}", response.payload.len());
        }
        Ok(Err(e)) => {
            println!("   ❌ Error: {}", e);
        }
        Err(_) => {
            println!("   ❌ Timeout - no response in 30 seconds");
        }
    }
}
