//! Test multiple relays to find one that works

use songbird_tor_protocol::{Consensus, BeardogCryptoClient};
use songbird_tor_protocol::connection::TorConnection;
use songbird_tor_protocol::protocol::{Cell, CellCommand};
use std::time::Duration;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    println!("=== MULTI-RELAY TEST ===\n");
    
    let beardog = BeardogCryptoClient::from_env().expect("BearDog client");
    
    println!("1. Fetching consensus...");
    let consensus = match tokio::time::timeout(
        Duration::from_secs(30),
        Consensus::fetch(&beardog)
    ).await {
        Ok(Ok(c)) => c,
        Ok(Err(e)) => { println!("❌ Failed: {}", e); return; }
        Err(_) => { println!("❌ Timeout"); return; }
    };
    
    // Get first 10 guards with different ports
    let guards: Vec<_> = consensus.relays.iter()
        .filter(|r| r.is_guard())
        .take(10)
        .cloned()
        .collect();
    
    println!("   Found {} guards to test\n", guards.len());
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    for (idx, mut guard) in guards.into_iter().enumerate() {
        println!("\n--- Testing relay {}: {} at {}:{} ---", 
                 idx + 1, guard.nickname, guard.address, guard.or_port);
        
        // Fetch ntor key if missing
        if guard.ntor_key.is_none() {
            print!("   Fetching ntor key... ");
            match Consensus::fetch_relay_ntor_key(&guard).await {
                Ok(key) => {
                    println!("✓");
                    guard.ntor_key = Some(key);
                }
                Err(e) => {
                    println!("✗ ({})", e);
                    fail_count += 1;
                    continue;
                }
            }
        } else {
            println!("   Has ntor key: ✓");
        }
        
        // Connect
        print!("   Connecting... ");
        let mut conn = TorConnection::new(guard.clone());
        match conn.connect().await {
            Ok(_) => println!("✓"),
            Err(e) => {
                println!("✗ ({})", e);
                fail_count += 1;
                continue;
            }
        }
        
        // Generate keypair and send CREATE2
        print!("   Sending CREATE2... ");
        let keypair = match beardog.x25519_generate_ephemeral() {
            Ok(k) => k,
            Err(e) => {
                println!("✗ keypair gen failed: {}", e);
                fail_count += 1;
                continue;
            }
        };
        
        let circ_id: u32 = 0x80000001;
        let mut payload = Vec::new();
        payload.extend_from_slice(&[0x00, 0x02]); // HTYPE = ntor
        payload.extend_from_slice(&[0x00, 0x54]); // HLEN = 84
        payload.extend_from_slice(&guard.fingerprint);
        payload.extend_from_slice(guard.ntor_key.as_ref().unwrap());
        payload.extend_from_slice(&keypair.public_key);
        
        let cell = Cell {
            circ_id,
            command: CellCommand::Create2,
            payload,
        };
        
        if let Err(e) = conn.send_cell(&cell).await {
            println!("✗ ({})", e);
            fail_count += 1;
            continue;
        }
        println!("✓");
        
        // Wait for response with shorter timeout
        print!("   Waiting for response (15s)... ");
        match tokio::time::timeout(
            Duration::from_secs(15),
            conn.recv_cell()
        ).await {
            Ok(Ok(response)) => {
                println!("✓");
                println!("   Response: circ_id={}, cmd={:?}", response.circ_id, response.command);
                success_count += 1;
            }
            Ok(Err(e)) => {
                println!("✗ ({})", e);
                fail_count += 1;
            }
            Err(_) => {
                println!("✗ (timeout)");
                fail_count += 1;
            }
        }
    }
    
    println!("\n=== RESULTS ===");
    println!("Success: {}", success_count);
    println!("Failed: {}", fail_count);
}
