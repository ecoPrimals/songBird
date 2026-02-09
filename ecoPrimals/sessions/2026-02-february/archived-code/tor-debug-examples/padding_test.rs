//! Test sending PADDING cells to verify relay communication

use songbird_tor_protocol::{Consensus, BeardogCryptoClient};
use songbird_tor_protocol::connection::TorConnection;
use songbird_tor_protocol::protocol::{Cell, CellCommand};
use std::time::Duration;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    
    println!("=== PADDING CELL TEST ===\n");
    
    let beardog = BeardogCryptoClient::from_env().expect("BearDog client");
    
    println!("1. Fetching consensus...");
    let consensus = Consensus::fetch(&beardog).await.expect("consensus");
    
    let guard = consensus.relays.iter().find(|r| r.is_guard()).unwrap().clone();
    println!("   Relay: {}", guard.nickname);
    
    println!("\n2. Connecting...");
    let mut conn = TorConnection::new(guard.clone());
    conn.connect().await.expect("connect");
    println!("   Connected!");
    
    // Send multiple PADDING cells to see if relay is receiving and processing
    println!("\n3. Sending PADDING cells...");
    for i in 0..3 {
        let padding_cell = Cell {
            circ_id: 0,  // Link-level
            command: CellCommand::Padding,
            payload: vec![0u8; 507],
        };
        
        conn.send_cell(&padding_cell).await.expect("send padding");
        println!("   Sent PADDING cell {}", i + 1);
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    // Now try CREATE2 - need to refetch ntor key since we used guard earlier
    println!("\n4. Fetching ntor key and sending CREATE2...");
    let mut guard_for_create = guard.clone();
    if guard_for_create.ntor_key.is_none() {
        let ntor = Consensus::fetch_relay_ntor_key(&guard_for_create).await.expect("ntor key");
        guard_for_create.ntor_key = Some(ntor);
    }
    
    let keypair = beardog.x25519_generate_ephemeral().expect("keypair");
    
    let mut payload = Vec::new();
    payload.extend_from_slice(&[0x00, 0x02]); // HTYPE = ntor
    payload.extend_from_slice(&[0x00, 0x54]); // HLEN = 84
    payload.extend_from_slice(&guard_for_create.fingerprint);
    payload.extend_from_slice(guard_for_create.ntor_key.as_ref().unwrap());
    payload.extend_from_slice(&keypair.public_key);
    
    let create2 = Cell {
        circ_id: 0x80000001,
        command: CellCommand::Create2,
        payload,
    };
    
    conn.send_cell(&create2).await.expect("send create2");
    println!("   Sent CREATE2 cell");
    
    // Wait for response
    println!("\n5. Waiting for CREATED2 response (15s)...");
    match tokio::time::timeout(Duration::from_secs(15), conn.recv_cell()).await {
        Ok(Ok(cell)) => {
            println!("   ✅ Got response: {:?}, circ_id={}", cell.command, cell.circ_id);
        }
        Ok(Err(e)) => {
            println!("   ❌ Error: {}", e);
        }
        Err(_) => {
            println!("   ❌ Timeout - no response");
        }
    }
}
