//! Quick connection and circuit building test

use songbird_tor_protocol::{Consensus, BeardogCryptoClient};
use songbird_tor_protocol::circuit::{CircuitManager, CircuitPurpose};
use std::time::Duration;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    
    println!("1. Creating BearDog client...");
    let beardog = BeardogCryptoClient::from_env().expect("BearDog client");
    
    println!("2. Fetching consensus...");
    let consensus = match tokio::time::timeout(
        Duration::from_secs(30),
        Consensus::fetch(&beardog)
    ).await {
        Ok(Ok(c)) => c,
        Ok(Err(e)) => {
            println!("❌ Fetch failed: {}", e);
            return;
        }
        Err(_) => {
            println!("❌ Fetch timed out");
            return;
        }
    };
    
    println!("   Relays: {}", consensus.relays.len());
    
    // Check if any relay has ntor_key
    let with_ntor = consensus.relays.iter().filter(|r| r.ntor_key.is_some()).count();
    println!("   With ntor_key: {}", with_ntor);
    
    println!("3. Selecting path...");
    let path = match consensus.select_path() {
        Ok(p) => p,
        Err(e) => {
            println!("❌ Path selection failed: {}", e);
            return;
        }
    };
    
    println!("   Guard: {} (ntor: {:?})", path.guard.nickname, path.guard.ntor_key.is_some());
    println!("   Middle: {} (ntor: {:?})", path.middle.nickname, path.middle.ntor_key.is_some());
    println!("   Exit: {} (ntor: {:?})", path.exit.nickname, path.exit.ntor_key.is_some());
    
    println!("4. Creating circuit manager...");
    let manager = CircuitManager::new(beardog, consensus);
    
    println!("5. Building circuit...");
    match tokio::time::timeout(
        Duration::from_secs(60),
        manager.build_circuit(CircuitPurpose::General)
    ).await {
        Ok(Ok(circuit_id)) => {
            println!("✅ Circuit built! ID: {}", circuit_id);
        }
        Ok(Err(e)) => {
            println!("❌ Circuit build failed: {}", e);
        }
        Err(_) => {
            println!("❌ Circuit build timed out");
        }
    }
}
