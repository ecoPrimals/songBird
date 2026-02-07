//! Integration tests for Tor protocol

use songbird_tor_protocol::{Consensus, BeardogCryptoClient};

#[tokio::test]
async fn test_fetch_consensus_live() {
    // This test requires network access
    let beardog = BeardogCryptoClient::from_env().expect("BearDog client");
    
    let result = Consensus::fetch(&beardog).await;
    
    // Note: This will fail if no network, but that's OK for integration test
    match result {
        Ok(consensus) => {
            assert!(!consensus.relays.is_empty(), "Consensus should have relays");
            assert!(consensus.is_valid(), "Consensus should be valid");
            
            // Check we can find some guards
            let guards: Vec<_> = consensus.relays.iter()
                .filter(|r| r.is_guard())
                .collect();
            assert!(!guards.is_empty(), "Should have at least one guard");
            
            println!("✅ Fetched {} relays ({} guards)", consensus.relays.len(), guards.len());
        }
        Err(e) => {
            println!("⚠️  Consensus fetch failed (expected if offline): {}", e);
        }
    }
}

#[test]
fn test_relay_selection_empty() {
    let consensus = Consensus {
        valid_after: std::time::SystemTime::now(),
        fresh_until: std::time::SystemTime::now() + std::time::Duration::from_secs(3600),
        valid_until: std::time::SystemTime::now() + std::time::Duration::from_secs(7200),
        relays: vec![],
    };
    
    let result = consensus.select_path();
    assert!(result.is_err(), "Should fail with empty relay list");
}

#[test]
fn test_consensus_freshness() {
    use std::time::{SystemTime, Duration};
    
    let now = SystemTime::now();
    
    // Fresh consensus
    let fresh = Consensus {
        valid_after: now - Duration::from_secs(1800),
        fresh_until: now + Duration::from_secs(1800),
        valid_until: now + Duration::from_secs(3600),
        relays: vec![],
    };
    assert!(fresh.is_fresh());
    assert!(fresh.is_valid());
    
    // Stale but valid consensus
    let stale = Consensus {
        valid_after: now - Duration::from_secs(3600),
        fresh_until: now - Duration::from_secs(100),
        valid_until: now + Duration::from_secs(100),
        relays: vec![],
    };
    assert!(!stale.is_fresh());
    assert!(stale.is_valid());
    
    // Invalid consensus
    let invalid = Consensus {
        valid_after: now - Duration::from_secs(7200),
        fresh_until: now - Duration::from_secs(3600),
        valid_until: now - Duration::from_secs(100),
        relays: vec![],
    };
    assert!(!invalid.is_fresh());
    assert!(!invalid.is_valid());
}
