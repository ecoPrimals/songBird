# Genetic Lineage Integration Guide

## Overview

Genetic Lineage enables automatic peer trust based on cryptographic ancestry. Nodes that share the same lineage (verified via BearDog) are automatically trusted, while nodes with different or unknown lineages require user consent.

## Core Concepts

### Lineage ID

A unique cryptographic identifier representing a node's genetic ancestry:

```rust
use songbird_types::LineageId;

// Create from string
let lineage_id = LineageId::new("lineage:tower1:1735660800:abc123");

// Extract tower ID (if format supports it)
if let Some(tower_id) = lineage_id.tower_id() {
    println!("Tower: {}", tower_id);
}
```

### Lineage Proof

Cryptographic proof containing the signature chain:

```rust
use songbird_types::{LineageId, LineageProof};
use songbird_types::lineage::LineageSignature;

let lineage_id = LineageId::new("my-lineage");
let signatures = vec![
    LineageSignature {
        signer_node_id: "node-1".to_string(),
        signature: "sig_hex_data".to_string(),
        signed_data_hash: "hash_hex".to_string(),
        timestamp: 1735660800,
    }
];

let proof = LineageProof::new(lineage_id, signatures, 1735660800);

// Check expiration
if proof.is_expired(3600) { // 1 hour TTL
    println!("Proof expired!");
}

// Check chain length
println!("Chain length: {}", proof.chain_length());
```

## Discovery Integration

### Broadcasting Lineage

Include lineage in discovery packets:

```rust
use songbird_discovery::DiscoveryPacket;

let packet = DiscoveryPacket::new(
    "my-node-id",
    vec!["compute".to_string(), "storage".to_string()],
    "http://192.168.1.100:8080"
)
.with_name("My Node")
.with_lineage(lineage_id, lineage_proof);

// Convert to mDNS TXT records
let txt_records = packet.to_txt_records();
```

### Parsing Lineage from Discovery

```rust
use songbird_discovery::DiscoveryPacket;
use std::collections::HashMap;

// Parse from mDNS TXT records
let txt_records: HashMap<String, String> = /* from mDNS */;
let packet = DiscoveryPacket::from_txt_records(&txt_records)?;

// Check if peer has lineage
if let Some(lineage_id) = &packet.genetic_lineage {
    println!("Peer lineage: {}", lineage_id);
    
    if let Some(proof) = &packet.lineage_proof {
        println!("Proof chain length: {}", proof.chain_length());
    }
}
```

### Backward Compatibility

Old nodes without lineage work seamlessly:

```rust
// Discovery packet without lineage
let packet = DiscoveryPacket::new(
    "old-node",
    vec!["compute".to_string()],
    "http://192.168.1.101:8080"
);

// genetic_lineage and lineage_proof will be None
assert!(packet.genetic_lineage.is_none());
assert!(packet.lineage_proof.is_none());
```

## Node Identity

### Setting Lineage on Node Identity

```rust
use songbird_orchestrator::node_identity::NodeIdentity;

// Create or load node identity
let mut identity = NodeIdentity::new_or_load(Some("my-node".to_string()))?;

// Check if identity has lineage
if !identity.has_lineage() {
    // Get lineage from BearDog (see BearDog integration section)
    let (lineage_id, proof) = get_lineage_from_beardog().await?;
    
    // Set lineage on identity
    identity.set_lineage(lineage_id, proof)?;
    
    println!("✅ Node identity updated with genetic lineage");
}

// Retrieve lineage
if let Some((lineage_id, proof)) = identity.get_lineage() {
    println!("Current lineage: {}", lineage_id);
}
```

## Node Registration

### Creating Registration with Lineage

```rust
use songbird_orchestrator::registration::NodeRegistration;

// Create registration with lineage
let registration = NodeRegistration::with_lineage(
    "node-123",
    "My Compute Node",
    vec!["compute".to_string(), "storage".to_string()],
    "http://192.168.1.100:8080",
    lineage_id,
    lineage_proof,
);

// Check if registration has lineage
assert!(registration.has_lineage());
```

### Creating Registration from Identity

Automatically includes lineage if identity has it:

```rust
use songbird_orchestrator::registration::create_registration_from_identity;

let registration = create_registration_from_identity(
    &identity,
    "http://192.168.1.100:8080".to_string(),
    vec!["compute".to_string()],
).await?;

if registration.has_lineage() {
    println!("✅ Registration includes genetic lineage");
} else {
    println!("ℹ️  Registration without lineage (BearDog not initialized)");
}
```

### Registration Manager

Manage registration lifecycle:

```rust
use songbird_orchestrator::registration::RegistrationManager;

let mut manager = RegistrationManager::new(60); // 60-second refresh interval

// Register node
manager.register(registration);

// Check if refresh needed
if manager.needs_refresh() {
    manager.refresh();
    println!("🔄 Registration refreshed");
}

// Update lineage after initial registration
let (new_lineage, new_proof) = get_updated_lineage().await?;
manager.update_lineage(new_lineage, new_proof)?;
```

## Peer Evaluation & Auto-Accept

### Setting Up Lineage Authenticator

```rust
use songbird_orchestrator::trust::LineageAuthenticator;

// Create authenticator
let mut auth = LineageAuthenticator::new();

// Initialize with BearDog endpoint
auth.initialize("http://localhost:9000").await?;

println!("🐻 Lineage authenticator initialized");
```

### Evaluating Peers

```rust
use songbird_orchestrator::trust::{PeerAcceptanceDecision, LineageStatus};

// Evaluate discovered peer
let decision = auth.evaluate_peer(
    &peer_packet.node_id,
    &peer_packet.endpoint,
    &peer_packet.capabilities,
    peer_packet.genetic_lineage.as_ref(),
    peer_packet.lineage_proof.as_ref(),
).await?;

// Handle decision
match decision {
    PeerAcceptanceDecision::AutoAccept { reason, lineage_id, confidence } => {
        println!("✅ Auto-accepting peer: {}", reason);
        println!("   Lineage: {}", lineage_id);
        println!("   Confidence: {:.1}%", confidence * 100.0);
        
        // Automatically establish connection
        connect_to_peer(&peer_packet).await?;
    }
    
    PeerAcceptanceDecision::PromptUser { peer_info, lineage_status, recommendation } => {
        match lineage_status {
            LineageStatus::SameGenesis { lineage_id, genesis_timestamp } => {
                println!("✓ Same genesis lineage: {}", lineage_id);
            }
            LineageStatus::DifferentGenesis { their_lineage, our_lineage } => {
                println!("⚠️  Different genetic lineage:");
                println!("   Theirs: {}", their_lineage);
                println!("   Ours:   {}", our_lineage);
            }
            LineageStatus::UnknownLineage => {
                println!("ℹ️  Peer has no genetic lineage");
            }
            LineageStatus::InvalidProof { error } => {
                println!("❌ Invalid lineage proof: {}", error);
            }
        }
        
        // Prompt user for decision
        let user_accepts = prompt_user_for_acceptance(&peer_info)?;
        if user_accepts {
            connect_to_peer(&peer_packet).await?;
        }
    }
    
    PeerAcceptanceDecision::Reject { reason, severity } => {
        println!("❌ Rejecting peer: {}", reason);
        println!("   Severity: {:?}", severity);
        
        // Log security event
        log_security_event("peer_rejected", &reason, &severity);
    }
}
```

## BearDog Integration

### Current Status: Mock Implementation

The current implementation uses a mock BearDog client for development:

```rust
// Mock client always returns valid=true
// Uses tower ID comparison for same_family() heuristic
```

### When BearDog Phase 1.5 is Ready

Replace the mock client with actual API calls:

```rust
// This will be handled internally by LineageAuthenticator
// No code changes needed in your application!

// The mock client will be replaced with:
// - POST /api/v1/lineage/verify
// - GET /api/v1/lineage/current  
// - POST /api/v1/lineage/same-family
```

### Expected BearDog API

```bash
# Verify lineage proof
curl -X POST http://localhost:9000/api/v1/lineage/verify \
  -H "Content-Type: application/json" \
  -d '{
    "proof": {
      "lineage_id": "lineage:tower1:1735660800:abc123",
      "signatures": [...],
      "genesis_timestamp": 1735660800
    }
  }'

# Response:
{
  "valid": true,
  "same_genesis": true,
  "messages": ["Cryptographic verification successful"]
}

# Get current lineage
curl http://localhost:9000/api/v1/lineage/current

# Response:
{
  "lineage_id": "lineage:tower1:1735660800:abc123",
  "proof": {...},
  "genesis_timestamp": 1735660800
}
```

## Complete Integration Example

```rust
use anyhow::Result;
use songbird_orchestrator::{
    node_identity::NodeIdentity,
    registration::{create_registration_from_identity, RegistrationManager},
    trust::LineageAuthenticator,
};
use songbird_discovery::DiscoveryPacket;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Create or load node identity
    let mut identity = NodeIdentity::new_or_load(Some("my-node".to_string()))?;
    println!("🆔 Node identity: {}", identity.node_id);
    
    // 2. Get lineage from BearDog (if available)
    if !identity.has_lineage() {
        if let Ok((lineage_id, proof)) = get_lineage_from_beardog().await {
            identity.set_lineage(lineage_id, proof)?;
            println!("🧬 Genetic lineage set");
        }
    }
    
    // 3. Create registration with lineage
    let registration = create_registration_from_identity(
        &identity,
        "http://192.168.1.100:8080".to_string(),
        vec!["compute".to_string(), "storage".to_string()],
    ).await?;
    
    let mut reg_manager = RegistrationManager::new(60);
    reg_manager.register(registration);
    println!("📝 Node registered");
    
    // 4. Initialize lineage authenticator
    let mut auth = LineageAuthenticator::new();
    auth.initialize("http://localhost:9000").await?;
    println!("🐻 Lineage authenticator ready");
    
    // 5. Discover and evaluate peers
    loop {
        // Discover peer (from mDNS, HTTP API, etc.)
        let peer_packet = discover_peer().await?;
        
        // Evaluate peer with lineage authentication
        let decision = auth.evaluate_peer(
            &peer_packet.node_id,
            &peer_packet.endpoint,
            &peer_packet.capabilities,
            peer_packet.genetic_lineage.as_ref(),
            peer_packet.lineage_proof.as_ref(),
        ).await?;
        
        // Handle decision
        handle_peer_decision(decision, &peer_packet).await?;
    }
}

async fn get_lineage_from_beardog() -> Result<(LineageId, LineageProof)> {
    // Implementation depends on BearDog Phase 1.5 API
    todo!("Integrate with BearDog API")
}

async fn discover_peer() -> Result<DiscoveryPacket> {
    // Your discovery implementation (mDNS, HTTP, etc.)
    todo!()
}

async fn handle_peer_decision(
    decision: PeerAcceptanceDecision,
    peer: &DiscoveryPacket
) -> Result<()> {
    // Your decision handling logic
    todo!()
}
```

## Security Best Practices

### 1. Always Verify Proofs

Never trust lineage without verification:

```rust
// ✅ Good: Verify through BearDog
let decision = auth.evaluate_peer(...).await?;

// ❌ Bad: Trust lineage_id without proof verification
if peer.genetic_lineage.is_some() {
    auto_accept(); // NO!
}
```

### 2. Handle Expiration

Check proof expiration before using:

```rust
if proof.is_expired(3600) { // 1 hour
    println!("⚠️  Proof expired, requesting fresh proof");
    request_fresh_proof(&peer).await?;
}
```

### 3. Log Security Events

Track all acceptance decisions:

```rust
match decision {
    PeerAcceptanceDecision::Reject { reason, severity } => {
        log::warn!("Rejected peer {}: {} (severity: {:?})",
                   peer_id, reason, severity);
    }
    _ => { /* log other decisions */ }
}
```

### 4. User Consent for Unknown Lineages

Always prompt users for unknown lineages:

```rust
PeerAcceptanceDecision::PromptUser { peer_info, .. } => {
    let consent = get_user_consent(&peer_info)?;
    if consent {
        // Remember user's decision for this lineage
        save_trusted_lineage(&peer_info.lineage)?;
    }
}
```

## Troubleshooting

### Peer Not Auto-Accepted

**Symptom**: Peers with same lineage require manual approval

**Check**:
1. Both nodes have `genetic_lineage` and `lineage_proof`
2. BearDog client initialized: `auth.initialize(endpoint).await?`
3. Proofs are not expired: `proof.is_expired(ttl)`
4. BearDog verification returns `valid: true`

```rust
// Debug peer evaluation
println!("Peer lineage: {:?}", peer.genetic_lineage);
println!("Peer proof valid: {}", peer.lineage_proof.as_ref().map(|p| !p.is_expired(3600)).unwrap_or(false));
```

### mDNS TXT Records Too Large

**Symptom**: Discovery fails with "TXT record too large"

**Solution**: Lineage proofs are automatically base64-encoded and should stay under 400 bytes. If still too large:

```rust
// Reduce metadata in proof
let proof = LineageProof::new(lineage_id, signatures, genesis_timestamp);
// proof.metadata is empty by default
```

### BearDog Connection Issues

**Symptom**: `BearDog client not initialized` error

**Solution**: Ensure BearDog is running and endpoint is correct:

```bash
# Check BearDog is running
curl http://localhost:9000/health

# Initialize with correct endpoint
auth.initialize("http://localhost:9000").await?;
```

## Migration Guide

### Adding Lineage to Existing Deployment

1. **Update Songbird** to version with lineage support
2. **Deploy BearDog Phase 1.5** (when available)
3. **Initialize lineage** on each node:
   ```rust
   identity.set_lineage(lineage_id, proof)?;
   ```
4. **Nodes without lineage continue working** (graceful degradation)

### Backward Compatibility

✅ Old discovery packets (no lineage) are accepted
✅ Old nodes can join new clusters
✅ New nodes can join old clusters
✅ Mixed deployments fully supported

No breaking changes!

## Performance Considerations

### Verification Caching

Lineage verifications are cached for 5 minutes:

```rust
// First verification: ~50-200ms (BearDog API call)
let decision1 = auth.evaluate_peer(...).await?;

// Subsequent verifications (same peer): ~1μs (cache hit)
let decision2 = auth.evaluate_peer(...).await?;
```

### mDNS Overhead

- Lineage ID: ~40 bytes
- Lineage Proof (base64): ~200-300 bytes
- Total: ~340 bytes (well under 400-byte limit)

### Memory Usage

- Per-peer cache: ~200 bytes
- 1000 peers: ~200KB

Negligible compared to other data structures.

## API Reference

Full API documentation available via rustdoc:

```bash
cargo doc --no-deps --open
```

Key modules:
- `songbird_types::lineage` - Core lineage types
- `songbird_discovery::DiscoveryPacket` - Discovery with lineage
- `songbird_orchestrator::node_identity` - Node identity
- `songbird_orchestrator::registration` - Node registration
- `songbird_orchestrator::trust::lineage_auth` - Peer evaluation

## Support & Questions

For questions or issues with genetic lineage integration:

1. Check this guide and API docs
2. Review test examples in `crates/songbird-orchestrator/tests/genetic_lineage_integration.rs`
3. Consult `GENETIC_LINEAGE_INTEGRATION_FINAL.md` for technical details
4. Contact biomeOS team for BearDog integration specifics

## Next Steps

- [ ] Integrate with BearDog Phase 1.5 when available
- [ ] Add metrics/observability for lineage operations
- [ ] Implement proof revocation checking
- [ ] Add admin CLI commands for lineage management

