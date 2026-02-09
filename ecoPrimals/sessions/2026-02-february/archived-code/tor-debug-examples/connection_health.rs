//! Test connection health after CREATE2

use songbird_tor_protocol::{Consensus, BeardogCryptoClient};
use songbird_tor_protocol::connection::TorConnection;
use songbird_tor_protocol::protocol::{Cell, CellCommand};
use std::time::Duration;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    println!("=== CONNECTION HEALTH TEST ===\n");
    
    let beardog = BeardogCryptoClient::from_env().expect("BearDog client");
    
    println!("1. Getting guard and ntor key...");
    let consensus = Consensus::fetch(&beardog).await.expect("consensus");
    let mut guard = consensus.relays.iter()
        .find(|r| r.is_guard())
        .unwrap()
        .clone();
    
    if guard.ntor_key.is_none() {
        let ntor = Consensus::fetch_relay_ntor_key(&guard).await.expect("ntor");
        guard.ntor_key = Some(ntor);
    }
    println!("   Guard: {} ({:02x?}...)", guard.nickname, &guard.fingerprint[..4]);
    
    println!("\n2. Connecting...");
    let mut conn = TorConnection::new(guard.clone());
    conn.connect().await.expect("connect");
    println!("   Connected!");
    
    // Build and send CREATE2
    println!("\n3. Sending CREATE2...");
    let keypair = beardog.x25519_generate_ephemeral().expect("keypair");
    let mut payload = Vec::with_capacity(88);
    payload.extend_from_slice(&2u16.to_be_bytes());
    payload.extend_from_slice(&84u16.to_be_bytes());
    payload.extend_from_slice(&guard.fingerprint);
    payload.extend_from_slice(guard.ntor_key.as_ref().unwrap());
    payload.extend_from_slice(&keypair.public_key);
    
    let cell = Cell {
        circ_id: 0x80000001,
        command: CellCommand::Create2,
        payload,
    };
    conn.send_cell(&cell).await.expect("send CREATE2");
    println!("   Sent!");
    
    // Wait 5 seconds
    println!("\n4. Waiting 5 seconds...");
    tokio::time::sleep(Duration::from_secs(5)).await;
    
    // Check if we can still send
    println!("\n5. Sending PADDING cell to check connection...");
    let padding = Cell {
        circ_id: 0,
        command: CellCommand::Padding,
        payload: vec![0u8; 507],
    };
    match conn.send_cell(&padding).await {
        Ok(_) => println!("   Still connected (can send)"),
        Err(e) => println!("   Connection broken: {}", e),
    }
    
    // Try receiving with short timeout
    println!("\n6. Checking for any response (2s)...");
    match tokio::time::timeout(Duration::from_secs(2), conn.recv_cell()).await {
        Ok(Ok(response)) => {
            println!("   Got response: {:?}, circ_id={}", response.command, response.circ_id);
        }
        Ok(Err(e)) => {
            println!("   Receive error: {}", e);
        }
        Err(_) => {
            println!("   No response in 2 seconds");
        }
    }
    
    // Send another CREATE2 with different circuit ID
    println!("\n7. Sending second CREATE2 (circ_id 0x80000002)...");
    let keypair2 = beardog.x25519_generate_ephemeral().expect("keypair");
    let mut payload2 = Vec::with_capacity(88);
    payload2.extend_from_slice(&2u16.to_be_bytes());
    payload2.extend_from_slice(&84u16.to_be_bytes());
    payload2.extend_from_slice(&guard.fingerprint);
    payload2.extend_from_slice(guard.ntor_key.as_ref().unwrap());
    payload2.extend_from_slice(&keypair2.public_key);
    
    let cell2 = Cell {
        circ_id: 0x80000002,
        command: CellCommand::Create2,
        payload: payload2,
    };
    match conn.send_cell(&cell2).await {
        Ok(_) => println!("   Sent second CREATE2"),
        Err(e) => println!("   Failed to send: {}", e),
    }
    
    // Final check
    println!("\n8. Final check for any response (5s)...");
    match tokio::time::timeout(Duration::from_secs(5), conn.recv_cell()).await {
        Ok(Ok(response)) => {
            println!("   ✅ Got response: {:?}, circ_id={}", response.command, response.circ_id);
        }
        Ok(Err(e)) => {
            println!("   ❌ Error: {}", e);
        }
        Err(_) => {
            println!("   ❌ No response");
        }
    }
}
