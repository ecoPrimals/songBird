//! Example: Using Genetic Lineage for Automatic Peer Trust
//!
//! This example demonstrates how to use genetic lineage to automatically
//! establish trust with peers from the same cryptographic family.
//!
//! Run with: cargo run --example genetic_lineage_usage

use anyhow::Result;
use songbird_discovery::DiscoveryPacket;
use songbird_orchestrator::{
    node_identity::NodeIdentity,
    registration::{create_registration_from_identity, RegistrationManager},
    trust::{LineageAuthenticator, LineageStatus, PeerAcceptanceDecision, UserRecommendation},
};
use songbird_types::lineage::LineageSignature;
use songbird_types::{LineageId, LineageProof};
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    println!("\n🧬 Genetic Lineage Integration Example\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // ========================================
    // STEP 1: Node Identity with Lineage
    // ========================================
    println!("📋 Step 1: Setting up node identity with genetic lineage\n");

    let mut identity = NodeIdentity::new_or_load(Some("example-node".to_string()))?;
    println!("   🆔 Node ID: {}", identity.node_id);

    // Simulate getting lineage from BearDog
    // In production, this would call BearDog API
    let (lineage_id, lineage_proof) = create_example_lineage("example-node").await?;

    if !identity.has_lineage() {
        identity.set_lineage(lineage_id.clone(), lineage_proof.clone())?;
        println!("   ✅ Genetic lineage set: {}", lineage_id);
    } else {
        println!("   ℹ️  Node already has genetic lineage");
    }

    // ========================================
    // STEP 2: Node Registration
    // ========================================
    println!("\n📋 Step 2: Creating node registration with lineage\n");

    let registration = create_registration_from_identity(
        &identity,
        "http://192.168.1.100:8080".to_string(),
        vec!["compute".to_string(), "storage".to_string()],
    )
    .await?;

    if registration.has_lineage() {
        println!("   ✅ Registration includes genetic lineage");
    } else {
        println!("   ⚠️  Registration without lineage");
    }

    // Use registration manager
    let mut reg_manager = RegistrationManager::new(60);
    reg_manager.register(registration);
    println!("   📝 Node registered with manager");

    // ========================================
    // STEP 3: Discovery with Lineage
    // ========================================
    println!("\n📋 Step 3: Broadcasting discovery packet with lineage\n");

    let discovery = DiscoveryPacket::new(
        identity.node_id.to_string(),
        vec!["compute".to_string(), "storage".to_string()],
        "http://192.168.1.100:8080",
    )
    .with_name("Example Node")
    .with_lineage(lineage_id.clone(), lineage_proof.clone());

    // Convert to mDNS TXT records
    let txt_records = discovery.to_txt_records();
    println!("   📡 mDNS TXT records created:");
    println!("      Keys: {:?}", txt_records.keys().collect::<Vec<_>>());
    println!("      Total size: ~{} bytes", txt_records.values().map(|v| v.len()).sum::<usize>());

    // ========================================
    // STEP 4: Lineage Authenticator
    // ========================================
    println!("\n📋 Step 4: Initializing lineage authenticator\n");

    let mut auth = LineageAuthenticator::new();
    auth.initialize("http://localhost:9000").await?;
    println!("   🐻 Authenticator initialized with BearDog mock");

    // ========================================
    // STEP 5: Evaluate Peers
    // ========================================
    println!("\n📋 Step 5: Evaluating discovered peers\n");

    // Scenario A: Same lineage (should auto-accept)
    println!("   Scenario A: Peer with same genetic lineage");
    let peer_same = create_peer_packet(
        "peer-same-lineage",
        Some(lineage_id.clone()),
        Some(lineage_proof.clone()),
    );
    evaluate_and_handle_peer(&mut auth, &peer_same).await?;

    // Scenario B: Different lineage (should prompt)
    println!("\n   Scenario B: Peer with different genetic lineage");
    let (diff_lineage, diff_proof) = create_example_lineage("different-tower").await?;
    let peer_diff =
        create_peer_packet("peer-different-lineage", Some(diff_lineage), Some(diff_proof));
    evaluate_and_handle_peer(&mut auth, &peer_diff).await?;

    // Scenario C: No lineage (should prompt)
    println!("\n   Scenario C: Peer without genetic lineage");
    let peer_none = create_peer_packet("peer-no-lineage", None, None);
    evaluate_and_handle_peer(&mut auth, &peer_none).await?;

    // ========================================
    // STEP 6: Registration Refresh
    // ========================================
    println!("\n📋 Step 6: Registration refresh with lineage\n");

    if reg_manager.needs_refresh() {
        reg_manager.refresh();
        println!("   🔄 Registration refreshed");
    } else {
        println!("   ℹ️  No refresh needed yet");
    }

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Example complete!\n");
    println!("Key Takeaways:");
    println!("  • Lineage enables automatic peer trust");
    println!("  • Same-lineage peers are auto-accepted");
    println!("  • Different/unknown lineages prompt user");
    println!("  • Fully backward compatible (optional)");
    println!("  • Ready for BearDog Phase 1.5 integration\n");

    Ok(())
}

/// Create an example lineage for demonstration
async fn create_example_lineage(node_id: &str) -> Result<(LineageId, LineageProof)> {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    let lineage_id = LineageId::new(format!("lineage:tower1:{}:example", timestamp));

    let signature = LineageSignature {
        signer_node_id: format!("signer-{}", node_id),
        signature: "example_signature_0x1234567890abcdef".to_string(),
        signed_data_hash: "hash_0xfedcba0987654321".to_string(),
        timestamp,
    };

    let proof = LineageProof::new(lineage_id.clone(), vec![signature], timestamp);

    Ok((lineage_id, proof))
}

/// Create a peer discovery packet
fn create_peer_packet(
    node_id: &str,
    lineage: Option<LineageId>,
    proof: Option<LineageProof>,
) -> DiscoveryPacket {
    let mut packet = DiscoveryPacket::new(
        node_id,
        vec!["compute".to_string()],
        format!("http://192.168.1.{}:8080", fastrand::u8(100..200)),
    )
    .with_name(format!("Peer {}", node_id));

    if let (Some(l), Some(p)) = (lineage, proof) {
        packet = packet.with_lineage(l, p);
    }

    packet
}

/// Evaluate peer and handle decision
async fn evaluate_and_handle_peer(
    auth: &mut LineageAuthenticator,
    peer: &DiscoveryPacket,
) -> Result<()> {
    let decision = auth
        .evaluate_peer(
            &peer.node_id,
            &peer.endpoint,
            &peer.capabilities,
            peer.genetic_lineage.as_ref(),
            peer.lineage_proof.as_ref(),
        )
        .await?;

    match decision {
        PeerAcceptanceDecision::AutoAccept {
            reason,
            lineage_id,
            confidence,
        } => {
            println!("      ✅ AUTO-ACCEPT");
            println!("         Reason: {}", reason);
            println!("         Lineage: {}", lineage_id);
            println!("         Confidence: {:.1}%", confidence * 100.0);
            println!("         → Establishing connection automatically...");
        }

        PeerAcceptanceDecision::PromptUser {
            peer_info,
            lineage_status,
            recommendation,
        } => {
            println!("      ⚠️  PROMPT USER");
            println!("         Peer: {}", peer_info.node_id);
            println!("         Endpoint: {}", peer_info.endpoint);
            println!("         Capabilities: {:?}", peer_info.capabilities);

            match lineage_status {
                LineageStatus::SameGenesis {
                    lineage_id,
                    genesis_timestamp,
                } => {
                    println!("         Status: Same genesis lineage");
                    println!("         Lineage: {}", lineage_id);
                    println!("         Genesis: {}", genesis_timestamp);
                }
                LineageStatus::DifferentGenesis {
                    their_lineage,
                    our_lineage,
                } => {
                    println!("         Status: Different genetic lineage");
                    println!("         Their lineage: {}", their_lineage);
                    println!("         Our lineage: {}", our_lineage);
                }
                LineageStatus::UnknownLineage => {
                    println!("         Status: No genetic lineage");
                }
                LineageStatus::InvalidProof {
                    error,
                } => {
                    println!("         Status: Invalid proof");
                    println!("         Error: {}", error);
                }
            }

            match recommendation {
                UserRecommendation::Accept => {
                    println!("         Recommendation: ✓ Accept (low risk)")
                }
                UserRecommendation::Neutral => {
                    println!("         Recommendation: ○ Neutral (user decides)")
                }
                UserRecommendation::Reject => {
                    println!("         Recommendation: ✗ Reject (higher risk)")
                }
            }

            println!("         → Prompting user for decision...");
        }

        PeerAcceptanceDecision::Reject {
            reason,
            severity,
        } => {
            println!("      ❌ REJECT");
            println!("         Reason: {}", reason);
            println!("         Severity: {:?}", severity);
            println!("         → Connection refused");
        }
    }

    Ok(())
}
