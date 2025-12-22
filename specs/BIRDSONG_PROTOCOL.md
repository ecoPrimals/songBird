# BirdSong Protocol - Lineage-Gated Broadcast

**Date**: December 21, 2025  
**Status**: Architectural Specification  
**Joint Effort**: BearDog (genetics/encryption) + Songbird (broadcast/routing)

---

## 🎵 Core Concept

### The Metaphor

> **BirdSong**: A broadcast that is obvious to family and noise otherwise.

**Like nature:**
- Birds sing in frequencies/patterns their species recognizes
- Others hear sound but not meaning
- Family knows the song, strangers hear noise

**In Songbird:**
- Broadcast encrypted messages
- Family (lineage) can decrypt
- Others cannot tell there's meaningful content
- Privacy through selective intelligibility

---

## 🔐 Responsibility Division

### BearDog Responsibilities

**Genetics & Lineage:**
- Generate lineage chains
- Sign parent-child relationships
- Maintain lineage graph
- Provide lineage proofs

**Encryption:**
- Generate lineage-derived keys
- Encrypt birdSong messages
- Decrypt for authorized descendants
- Key rotation and revocation

**Grant Authority:**
- "You can hear songs from your lineage"
- Cryptographic proof of family membership
- Selective key distribution

### Songbird Responsibilities

**Broadcast (BirdSong):**
- Send encrypted broadcasts (UDP)
- Routing and delivery
- Frequency and timing
- Message structure

**Use Lineage:**
- Request decryption keys from BearDog
- Apply keys to incoming birdSong
- "Hear" family vs "hear" noise
- React to decrypted messages

**Discovery:**
- Who's in my federation?
- What capabilities exist?
- How do I reach them?
- All via birdSong

---

## 🎶 BirdSong Protocol

### Message Structure

```rust
/// A birdSong broadcast message
struct BirdSong {
    // Version of the protocol
    version: u8,
    
    // Encrypted payload
    ciphertext: Vec<u8>,
    
    // Lineage hint (which family can decrypt)
    // NOT the full lineage, just a hint for key selection
    lineage_hint: LineageHint,
    
    // Timestamp (prevents replay)
    timestamp: u64,
    
    // Signature (authenticity, not decryption)
    signature: Vec<u8>,
}

/// Hint about which lineage can decrypt
enum LineageHint {
    // My direct descendants only
    DirectDescendants,
    
    // All descendants (any generation)
    AllDescendants,
    
    // Specific lineage root
    LineageRoot(String),  // root node ID
    
    // Broadcast to all (but only family can decrypt)
    Universal,
}
```

### Payload (Before Encryption)

```rust
/// What's actually being broadcast
struct BirdSongPayload {
    // Type of message
    message_type: BirdSongType,
    
    // Sender's stable node ID
    sender_id: String,
    
    // Message content
    content: BirdSongContent,
}

enum BirdSongType {
    // I'm here and alive
    Presence,
    
    // I have these capabilities
    CapabilityAnnouncement,
    
    // I have these endpoints
    TransportAnnouncement,
    
    // I need help (relay request)
    RelayRequest,
    
    // Federation event
    FederationEvent,
    
    // Custom message
    Custom(String),
}

enum BirdSongContent {
    Presence {
        node_name: String,
        capabilities: Vec<String>,
        endpoints: Vec<TransportEndpoint>,
        last_heartbeat: u64,
    },
    
    CapabilityAnnouncement {
        new_capabilities: Vec<String>,
        removed_capabilities: Vec<String>,
    },
    
    TransportAnnouncement {
        available_endpoints: Vec<TransportEndpoint>,
        preferred_endpoint: TransportEndpoint,
    },
    
    RelayRequest {
        requesting_node: String,
        target_node: String,
        reason: String,
    },
    
    FederationEvent {
        event_type: String,
        details: serde_json::Value,
    },
    
    Custom {
        data: Vec<u8>,
    },
}
```

---

## 🔑 Key Derivation & Distribution

### BearDog Key Generation

```rust
/// BearDog generates keys from lineage
impl BearDog {
    /// Generate broadcast key for a lineage root
    fn generate_lineage_broadcast_key(&self, root_id: &str) -> Result<BroadcastKey> {
        // Derive key from lineage root's identity
        let key_material = self.derive_key_material(root_id)?;
        
        // Create broadcast key
        let broadcast_key = BroadcastKey {
            key_id: format!("lineage:{}", root_id),
            key_data: key_material,
            valid_from: Utc::now(),
            valid_until: Utc::now() + Duration::days(30),
        };
        
        Ok(broadcast_key)
    }
    
    /// Distribute key to descendants
    fn distribute_key_to_descendants(
        &self,
        root_id: &str,
        key: &BroadcastKey,
    ) -> Result<()> {
        // Get all descendants from lineage graph
        let descendants = self.get_all_descendants(root_id)?;
        
        // For each descendant, encrypt key with their public key
        for descendant in descendants {
            let encrypted_key = self.encrypt_for_node(&key, &descendant.public_key)?;
            self.send_key_to_node(descendant.id, encrypted_key)?;
        }
        
        Ok(())
    }
}
```

### Songbird Key Usage

```rust
/// Songbird requests and uses keys
impl Songbird {
    /// Request broadcast key from BearDog
    async fn request_broadcast_key(&self, lineage_hint: &LineageHint) -> Result<BroadcastKey> {
        // Query BearDog for the key
        let key_request = KeyRequest {
            lineage_hint: lineage_hint.clone(),
            requester_proof: self.get_lineage_proof()?,
        };
        
        // BearDog verifies we're in the lineage, returns key
        let key = self.beardog_client.request_key(key_request).await?;
        
        Ok(key)
    }
    
    /// Decrypt birdSong if we have the key
    fn try_decrypt_birdsong(&self, song: &BirdSong) -> Option<BirdSongPayload> {
        // Try to find matching key
        let key = self.get_key_for_lineage(&song.lineage_hint)?;
        
        // Attempt decryption
        match self.decrypt_with_key(&song.ciphertext, &key) {
            Ok(payload) => Some(payload),
            Err(_) => None,  // Not for us, just noise
        }
    }
}
```

---

## 🌊 Broadcast Flow

### 1. Songbird Wants to Broadcast

```
Songbird Node A:
  1. Create payload (presence, capabilities, etc.)
  2. Request encryption from BearDog
     - "Encrypt this for my lineage"
  3. BearDog encrypts with lineage key
  4. Songbird broadcasts encrypted birdSong (UDP)
```

### 2. Other Nodes Receive

```
Songbird Node B (Family):
  1. Receive UDP broadcast
  2. See lineage hint
  3. Request decryption key from BearDog
     - "I'm in this lineage, give me the key"
  4. BearDog verifies lineage, provides key
  5. Decrypt and process message
  6. "Oh, Node A is announcing presence!"

Songbird Node C (Not Family):
  1. Receive UDP broadcast
  2. See lineage hint
  3. Request decryption key from BearDog
     - "I'm in this lineage, give me the key"
  4. BearDog verifies: "You're not in this lineage"
  5. No key provided
  6. Message remains encrypted (noise)
```

---

## 🎯 Integration with Existing Systems

### Discovery Protocol (UDP Broadcast)

**Before (Discovery v3.1):**
```rust
// Plaintext discovery message
struct DiscoveryMessage {
    node_id: String,         // Visible to all
    node_name: String,       // Visible to all
    endpoints: Vec<...>,     // Visible to all
    capabilities: Vec<...>,  // Visible to all
}
```

**After (BirdSong Discovery):**
```rust
// Encrypted birdSong
struct BirdSong {
    ciphertext: Vec<u8>,           // Encrypted payload
    lineage_hint: LineageHint,     // Which family
    signature: Vec<u8>,            // Authenticity
}

// Payload (encrypted, only family can see)
struct BirdSongPayload {
    node_id: String,         // Hidden from non-family
    node_name: String,       // Hidden from non-family
    endpoints: Vec<...>,     // Hidden from non-family
    capabilities: Vec<...>,  // Hidden from non-family
}
```

**Privacy Gain:**
- Non-family cannot see your node_id, name, capabilities, endpoints
- They can't even tell these are discovery messages
- Just see encrypted UDP traffic (noise)

### Lineage-Gated Relay

**Relay Request via BirdSong:**
```rust
// Node needs relay, broadcasts to family
let relay_request = BirdSongPayload {
    message_type: BirdSongType::RelayRequest,
    sender_id: self.node_id.clone(),
    content: BirdSongContent::RelayRequest {
        requesting_node: self.node_id.clone(),
        target_node: "node-i-want-to-reach".to_string(),
        reason: "NAT traversal failed".to_string(),
    },
};

// Encrypt for my lineage
let birdsong = beardog.encrypt_for_lineage(relay_request, LineageHint::AllDescendants)?;

// Broadcast (UDP)
songbird.broadcast(birdsong)?;

// Any ancestor in my lineage can hear and offer relay
// Others just hear encrypted noise
```

### Rendezvous Protocol

**BirdSong as Backup:**
- Primary: HTTPS to rendezvous server
- Backup: BirdSong broadcast on LAN
- If rendezvous down, family can still discover via birdSong

---

## 🔒 Security Properties

### What BirdSong Provides

**Privacy from Outsiders:**
- ✅ Non-family cannot decrypt content
- ✅ Cannot see node identities
- ✅ Cannot see capabilities
- ✅ Cannot see network topology
- ✅ Cannot distinguish message types

**Authenticity:**
- ✅ Signature proves sender identity
- ✅ Timestamp prevents replay
- ✅ Lineage proof gates decryption

**Selective Disclosure:**
- ✅ Broadcast to all, decrypt for family
- ✅ No need to know who's listening
- ✅ Family auto-discovers each other

### What BirdSong Does NOT Provide

**Traffic Analysis:**
- ❌ Timing still observable
- ❌ Frequency still observable
- ❌ Packet size still observable
- ➡️ Separate countermeasures needed (padding, dummy traffic)

**Anonymity from Family:**
- ❌ Family can see sender_id
- ❌ Family can see your capabilities
- ➡️ Intentional design for trust

**Forward Secrecy:**
- ❌ If lineage key compromised, past messages decryptable
- ➡️ Mitigate with key rotation (every 30 days)

---

## 🎭 Use Cases

### 1. Private Federation Discovery

**Scenario:** University federation, don't want outsiders to know topology

**Solution:**
```rust
// Each node broadcasts presence via birdSong
// Only family (same lineage root = same university) can decrypt
// Outside observers see encrypted traffic, learn nothing
```

### 2. Capability Announcement

**Scenario:** Node gains new capability (GPU available)

**Solution:**
```rust
// Broadcast capability update via birdSong
// Only family knows new GPU is available
// Outside observers don't know what capabilities exist
```

### 3. Relay Coordination

**Scenario:** Two family nodes behind NAT need relay

**Solution:**
```rust
// Node A broadcasts relay request via birdSong
// Ancestors in family hear request, offer relay
// Outside observers don't know relay is happening
```

### 4. Emergency Broadcast

**Scenario:** Federation event (node leaving, trust change)

**Solution:**
```rust
// Broadcast federation event via birdSong
// Only family knows about the event
// Outside observers don't know federation state changed
```

---

## 📊 Performance Considerations

### Key Caching

```rust
struct BirdSongKeyCache {
    // Cache decryption keys by lineage hint
    keys: HashMap<String, BroadcastKey>,
    
    // TTL for each key
    expiry: HashMap<String, DateTime<Utc>>,
}

impl BirdSongKeyCache {
    fn get_or_fetch(&mut self, hint: &LineageHint) -> Result<BroadcastKey> {
        // Check cache first
        if let Some(key) = self.get_cached_key(hint) {
            if !self.is_expired(hint) {
                return Ok(key);
            }
        }
        
        // Cache miss or expired, fetch from BearDog
        let key = self.fetch_from_beardog(hint)?;
        self.cache_key(hint, key.clone())?;
        
        Ok(key)
    }
}
```

**Benefit**: Don't query BearDog for every broadcast

### Batch Decryption

```rust
fn process_birdsong_batch(&self, songs: Vec<BirdSong>) -> Vec<BirdSongPayload> {
    // Group by lineage hint
    let grouped = self.group_by_lineage(songs);
    
    // Fetch keys once per lineage
    let mut results = Vec::new();
    for (hint, songs) in grouped {
        if let Ok(key) = self.get_key_for_lineage(&hint) {
            for song in songs {
                if let Ok(payload) = self.decrypt_with_key(&song.ciphertext, &key) {
                    results.push(payload);
                }
            }
        }
    }
    
    results
}
```

**Benefit**: Amortize key fetch cost

---

## 🔄 Migration Strategy

### Phase 1: Parallel Operation (Current)

**Both protocols active:**
- Old: Plaintext discovery (v3.1)
- New: BirdSong discovery

**Nodes support both:**
- Send: Both plaintext and birdSong
- Receive: Both plaintext and birdSong

### Phase 2: Gradual Adoption

**As BearDog deploys:**
- Nodes with BearDog: Prefer birdSong, support plaintext
- Nodes without BearDog: Use plaintext only
- Graceful interoperability

### Phase 3: BirdSong Only

**When all nodes have BearDog:**
- Deprecate plaintext discovery
- BirdSong becomes standard
- Privacy by default

**Timeline:** 3-6 months after BearDog integration complete

---

## 🧬 Lineage Scenarios

### Scenario 1: Single University

```
University Root
├── Department A (faculty)
│   ├── Professor 1
│   └── Professor 2
└── Department B (students)
    ├── Student 1
    └── Student 2
```

**BirdSong:**
- University broadcasts with `LineageHint::AllDescendants`
- All professors and students can decrypt
- Outside observers see noise

### Scenario 2: Multi-University Federation

```
Federation Root
├── University A
│   └── Departments...
└── University B
    └── Departments...
```

**BirdSong:**
- Federation-wide: `LineageHint::LineageRoot("federation-root")`
- All universities can decrypt
- University-specific: `LineageHint::LineageRoot("university-a")`
- Only University A can decrypt

### Scenario 3: Public + Private

```
Public Federation (no lineage required)
Private Lineages within
```

**BirdSong:**
- Public broadcasts: Traditional plaintext (interop)
- Private broadcasts: BirdSong for family only
- Dual mode for maximum flexibility

---

## 📚 Related Specifications

**BearDog Integration:**
- `BEARDOG_BTSP_HANDOFF.md` - Overall integration
- `specs/PRIMAL_RESPONSIBILITY_SEPARATION_SPEC.md` - Role separation

**Lineage System:**
- `specs/LINEAGE_GATED_RELAY_PROTOCOL.md` - Relay protocol
- This spec extends lineage to broadcasts

**Discovery:**
- Current: `specs/federation/DISCOVERY_PROTOCOL.md` (v3.1)
- Future: BirdSong replaces/augments discovery

**Privacy:**
- `docs/PRIVACY_FIRST_FEDERATION.md` - Privacy architecture

---

## 🚀 Implementation Plan

### Phase 1: BearDog Integration (BearDog team)

**Week 1-2:**
- Lineage key generation
- Key distribution to descendants
- Key revocation and rotation

**Week 3-4:**
- BirdSong encryption API
- BirdSong decryption API
- Key request protocol

### Phase 2: Songbird Integration (Songbird team)

**Week 1-2:**
- BirdSong message structures
- Encryption/decryption flow
- Key caching

**Week 3-4:**
- Replace plaintext discovery with birdSong
- Maintain backward compatibility
- Integration tests

### Phase 3: Deployment

**Week 1-2:**
- Deploy to test federation
- Monitor key distribution
- Verify privacy

**Week 3-4:**
- Production rollout
- Documentation
- Migration guide

**Total Timeline:** 8-12 weeks (parallel with Phase 3 NAT traversal)

---

## 🎉 Summary

### The Elegant Solution

> **BearDog handles**: Genetics, lineage, encryption  
> **Songbird uses**: That in its "birdSong"  
> **Result**: Broadcast that's obvious to family, noise to others

### Joint Responsibility

**BTSP = Joint Effort:**
- BearDog: The cryptography
- Songbird: The networking
- Together: Secure, private, sovereign federation

### Key Benefits

**Privacy:**
- Outsiders cannot decrypt broadcasts
- Cannot learn topology
- Cannot identify nodes
- Cannot map capabilities

**Usability:**
- Broadcast to all, decrypt for family
- No need to know who's listening
- Auto-discovery within lineage

**Sovereignty:**
- No central key server
- Lineage is the authority
- Revocation via lineage
- No vendor dependency

---

**Status**: Architectural specification complete  
**Dependencies**: BearDog lineage + key distribution  
**Timeline**: 8-12 weeks (parallel with Phase 3)  
**Impact**: Privacy-preserving discovery and coordination

*BirdSong: The sound of family, noise to others* 🎵🔒🧬✨

