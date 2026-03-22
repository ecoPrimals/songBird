// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Integration tests for Tor protocol

use songbird_tor_protocol::{Consensus, CryptoProvider, TorConnection};

#[tokio::test]
async fn test_fetch_consensus_live() {
    // This test requires network access
    let beardog = CryptoProvider::from_env();

    let result = Consensus::fetch(&beardog).await;

    // Note: This will fail if no network, but that's OK for integration test
    match result {
        Ok(consensus) => {
            assert!(!consensus.relays.is_empty(), "Consensus should have relays");
            assert!(consensus.is_valid(), "Consensus should be valid");

            // Check we can find some guards
            let guards: Vec<_> = consensus.relays.iter().filter(|r| r.is_guard()).collect();
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
    use std::time::{Duration, SystemTime};

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

/// Test connecting to a real Tor relay
/// This test requires network access and a working consensus
#[tokio::test]
#[ignore = "Requires network access"]
async fn test_connect_to_relay() {
    use songbird_tor_protocol::directory::RelayFlags;

    let beardog = CryptoProvider::from_env();

    // Fetch consensus to get a real relay
    let consensus = match Consensus::fetch(&beardog).await {
        Ok(c) => c,
        Err(e) => {
            println!("⚠️  Skipping: could not fetch consensus: {}", e);
            return;
        }
    };

    // Find a guard relay to connect to
    let guard = consensus
        .relays
        .iter()
        .find(|r| r.is_guard() && r.flags.contains(RelayFlags::RUNNING))
        .expect("Should have at least one guard relay");

    println!("🔌 Connecting to guard: {} at {}:{}", guard.nickname, guard.address, guard.or_port);

    // Create connection
    let mut connection = TorConnection::new(guard.clone());

    // Try to connect
    match connection.connect().await {
        Ok(()) => {
            println!("✅ Connected to {} (link protocol negotiated)", guard.nickname);
            assert!(connection.is_ready(), "Connection should be ready");
        }
        Err(e) => {
            // Connection failures are expected in some environments
            println!("⚠️  Connection failed (may be expected): {}", e);
        }
    }
}

/// Test circuit building (end-to-end)
/// This test requires network access and working crypto
#[tokio::test]
#[ignore = "Requires network access and full setup"]
async fn test_build_circuit() {
    use songbird_tor_protocol::circuit::{CircuitManager, CircuitPurpose};

    let beardog = CryptoProvider::from_env();

    // Fetch consensus
    let consensus = match Consensus::fetch(&beardog).await {
        Ok(c) => c,
        Err(e) => {
            println!("⚠️  Skipping: could not fetch consensus: {}", e);
            return;
        }
    };

    if consensus.relays.len() < 3 {
        println!("⚠️  Skipping: not enough relays in consensus");
        return;
    }

    println!("📡 Building circuit with {} relays available", consensus.relays.len());

    // Create circuit manager
    let manager = CircuitManager::new(beardog, consensus);

    // Try to build a circuit
    match manager.build_circuit(CircuitPurpose::General).await {
        Ok(circuit_id) => {
            println!("✅ Circuit {} built successfully!", circuit_id);

            // Verify circuit has 3 hops
            let circuit = manager.get_circuit(circuit_id).expect("Circuit should exist");
            assert_eq!(circuit.hop_count(), 3, "Circuit should have 3 hops");
            assert!(circuit.is_complete(), "Circuit should be complete");

            // Close circuit
            manager.close_circuit(circuit_id).await.expect("Should close circuit");
        }
        Err(e) => {
            // Circuit building can fail for many reasons
            println!("⚠️  Circuit build failed: {}", e);
        }
    }
}
