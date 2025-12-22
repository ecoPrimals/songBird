# Security Analysis: Internet Deployment & Roaming Devices

## Current LAN Broadcast - What's Visible from Outside?

### What We're Broadcasting (Port 8888 UDP)

**Discovery Message Contents:**
```json
{
  "node_id": "496fe99e-0c8f-5a10-8d76-a0d52db5ee92",
  "node_name": "pop-os",
  "endpoints": [
    {
      "interface_type": "ethernet",
      "address": "192.168.1.134:8080",
      "protocols": ["https", "tarpc"],
      "preference": 100
    }
  ],
  "capabilities": ["orchestration", "federation"],
  "session_id": "rotating-session-id",
  "broadcast_timestamp": "2025-12-20T..."
}
```

### 🔴 Security Concerns for Internet Deployment

#### 1. **UDP Broadcast Visibility**
**Current (LAN):**
- Broadcasts to 255.255.255.255 (local network only)
- Stopped at router boundary
- Not visible on internet

**Problem (Internet):**
- Can't broadcast across internet
- Need different discovery mechanism
- NAT traversal required

#### 2. **Information Leakage**
**What's Currently Exposed:**
- ✅ Node ID (UUID - okay, not sensitive)
- ⚠️ Node name (hostname - could reveal info)
- 🔴 Internal IP addresses (private, but visible on LAN)
- 🔴 Capabilities (reveals what system can do)
- ✅ No secrets or keys (good)

**Risk Level:**
- **LAN:** Low (trusted network)
- **Contaminated LAN:** Medium (hostile observers)
- **Internet:** High (publicly visible)

#### 3. **HTTPS Endpoints**
**Current:**
- HTTPS on port 8080
- TLS encryption ✅
- Self-signed certs (okay for federation)

**Missing for Internet:**
- Certificate pinning
- Mutual TLS (mTLS)
- Certificate rotation
- Trust-on-first-use (TOFU) verification

## What We Need for Internet Deployment

### Phase 1: Secure Discovery (No Broadcast)

#### Option A: Rendezvous Server (RustDesk Model)
```rust
// Central discovery server (can be self-hosted)
struct RendezvousServer {
    // Nodes register with server
    // Server relays connection info
    // No direct peer-to-peer discovery needed
}
```

**Architecture:**
```
Mobile Device → Rendezvous Server ← Home Network
     |                |                    |
     +-- Encrypted ---+--- Encrypted ------+
         Connection       Connection
```

**Benefits:**
- Works through NAT
- No broadcast needed
- Can be self-hosted
- Privacy-preserving (encrypted channel)

#### Option B: DHT (Distributed Hash Table)
```rust
// Similar to BitTorrent/IPFS
struct DHTDiscovery {
    // Each node is part of global DHT
    // Discover by node_id
    // No central server
}
```

**Benefits:**
- Fully decentralized
- No single point of failure
- Censorship-resistant

**Drawbacks:**
- More complex
- Slower to find nodes
- DHT can leak some metadata

#### Option C: Onion Routing (Tor Hidden Services)
```rust
// Each node has .onion address
struct TorHiddenService {
    onion_address: "abcd1234....onion",
    // End-to-end encrypted
    // NAT traversal automatic
    // Anonymous
}
```

**Benefits:**
- Maximum privacy
- NAT traversal automatic
- Location hidden
- Censorship-resistant

**Drawbacks:**
- Requires Tor
- Higher latency
- Complexity

### Phase 2: NAT Traversal

#### Current Problem:
```
Internet
    |
  Router/NAT (blocks incoming)
    |
  Your Device (can't be reached)
```

#### Solutions:

**A. STUN/TURN (WebRTC Model)**
```rust
struct NATTraversal {
    // STUN: Discover public IP
    public_ip: discover_via_stun(),
    
    // ICE: Try multiple connection methods
    ice_candidates: vec![
        direct,           // Try direct first
        port_forwarded,   // Try UPnP
        relayed,          // Use TURN relay as fallback
    ],
    
    // TURN: Relay if nothing else works
    relay_server: "turn.example.com",
}
```

**B. Hole Punching**
```rust
// Both sides connect to relay
// Relay coordinates simultaneous connection
// Creates "hole" in both NATs
// Then direct peer-to-peer
```

**C. UPnP/NAT-PMP**
```rust
// Automatically configure router
// Open port forwarding
// Works on ~70% of home routers
```

### Phase 3: Mutual Authentication & Encryption

#### Beyond Basic HTTPS

**What We Have:**
- TLS encryption ✅
- Server certificates ✅

**What We Need:**

**A. Mutual TLS (mTLS)**
```rust
struct MutualTLS {
    // Both sides present certificates
    client_cert: Certificate,
    server_cert: Certificate,
    
    // Trust verified both ways
    trust_chain: verify_both_sides(),
}
```

**B. Trust-on-First-Use (TOFU)**
```rust
struct TOFUVerification {
    // First connection: record fingerprint
    first_seen: {
        node_id: "abc123",
        cert_fingerprint: "sha256:...",
        timestamp: "2025-12-20",
    },
    
    // Future connections: verify matches
    verify: |cert| {
        cert.fingerprint() == first_seen.fingerprint
    },
}
```

**C. Out-of-Band Verification**
```rust
// QR code, NFC, or shared secret
struct OutOfBandVerification {
    // Show QR code on one device
    qr_code: generate_qr(node_id + public_key),
    
    // Scan on other device
    verify: scan_and_compare(),
}
```

### Phase 4: Mobile/Roaming Architecture

#### RustDesk Model Applied to Songbird

**Current:**
```
Tower (Static IP) ←→ Tower (Static IP)
```

**Needed:**
```
Mobile Device ←→ Rendezvous ←→ Home Network
     (roaming)      (relay)         (static)
```

**Implementation:**

```rust
struct RoamingNode {
    // Stable identity (not tied to IP)
    node_id: Uuid,
    node_name: String,
    
    // Current connection method (changes as you roam)
    current_endpoint: DynamicEndpoint {
        // Could be WiFi, cellular, VPN, etc.
        connection_type: detect_connection(),
        
        // Public IP (via STUN)
        public_ip: discover_public_ip(),
        
        // Relay fallback
        relay: "relay.songbird.eco",
    },
    
    // Cryptographic identity (persistent)
    identity_key: Ed25519KeyPair,
    
    // Trust relationships (persistent)
    trusted_nodes: HashMap<NodeId, TrustLevel>,
}
```

**Connection Flow:**

1. **Device Roams** (WiFi → Cellular → New WiFi)
```rust
// IP changes but identity doesn't
on_network_change() {
    // Reconnect to rendezvous
    register_with_rendezvous(
        node_id,          // Same
        new_endpoint,     // New!
        identity_proof,   // Signed with same key
    );
    
    // Notify peers
    broadcast_to_trusted_nodes(
        "I'm at a new address",
        signed_message,
    );
}
```

2. **Peers Reconnect**
```rust
// Other nodes receive update
on_peer_address_change(peer_id, new_endpoint) {
    // Verify cryptographic identity
    if verify_signature(peer_id, new_endpoint) {
        // Update routing table
        update_peer_endpoint(peer_id, new_endpoint);
        
        // Reconnect
        establish_connection(new_endpoint);
    }
}
```

## What We're Missing for Internet Deployment

### Critical Gaps:

#### 1. **Discovery Mechanism**
- ❌ UDP broadcast doesn't work on internet
- ❌ No rendezvous server
- ❌ No DHT implementation
- ❌ No Tor integration

#### 2. **NAT Traversal**
- ❌ No STUN client
- ❌ No TURN relay
- ❌ No ICE negotiation
- ❌ No UPnP/NAT-PMP

#### 3. **Enhanced Authentication**
- ❌ No mTLS
- ❌ No TOFU verification
- ❌ No out-of-band verification
- ❌ No certificate pinning

#### 4. **Mobile Support**
- ❌ No roaming logic
- ❌ No connection migration
- ❌ No relay fallback
- ❌ No offline queueing

### What We Have (Security Foundation):

#### ✅ Good Foundation:
- Strong cryptographic identities (UUIDs)
- TLS encryption
- Session rotation
- Privacy-first architecture (no hardcoded IPs)
- Capability-based trust
- Multi-path transport (ready for failover)

## Roadmap to Internet Deployment

### Phase 1: Secure LAN (Current + Quick Wins)

**What to Add:**
1. **Certificate Pinning**
```rust
// Pin first certificate seen
// Reject if changes (TOFU)
```

2. **Encrypted Discovery Messages**
```rust
// Even on LAN, encrypt discovery
// Use shared secret or public key
```

3. **Network Boundary Detection**
```rust
// Detect if on untrusted network
// Increase security automatically
```

### Phase 2: Rendezvous Server (RustDesk Model)

**Implementation (~1-2 weeks):**

```rust
// crates/songbird-rendezvous/
struct RendezvousServer {
    // Nodes register here
    registrations: HashMap<NodeId, RegistrationInfo>,
    
    // Relay connection requests
    relay_request(from: NodeId, to: NodeId) {
        // Don't reveal IPs directly
        // Coordinate connection
        initiate_nat_traversal(from, to);
    },
}
```

**Benefits:**
- Can be self-hosted
- Works through NAT
- No code changes to federation
- Privacy-preserving

### Phase 3: NAT Traversal (~2-3 weeks)

**Add STUN/TURN:**
```rust
// Use existing crates
use stun::client::StunClient;
use turn::client::TurnClient;

// Discover public IP
let public_ip = stun_client.discover().await?;

// Setup relay as fallback
let relay = turn_client.allocate().await?;
```

### Phase 4: Mobile/Roaming (~1-2 weeks)

**Add Connection Migration:**
```rust
// Detect network changes
// Re-register with rendezvous
// Notify peers
// Maintain open connections
```

## RustDesk Architecture Comparison

### RustDesk Model:
```
Desktop ← (encrypted) → Relay Server ← (encrypted) → Mobile
    |                        |                          |
  Static                  Public                     Roaming
  (Home)                 (Cloud)                   (Anywhere)
```

### Songbird Equivalent:
```
Tower ← (encrypted) → Rendezvous ← (encrypted) → Mobile Songbird
   |                       |                           |
Home Lab              Self-Hosted                   Laptop/Phone
8080 HTTPS           Public/VPS                    Roaming
```

### Key Differences:

**RustDesk:**
- Remote desktop (screen sharing)
- Always relayed (simpler)
- Central relay server

**Songbird (Proposed):**
- Orchestration (task routing)
- Peer-to-peer preferred
- Relay as fallback
- Can be fully decentralized (DHT)
- Or self-hosted rendezvous

## Security Recommendations

### Immediate (Can Do Now):

1. **Environment-Based Security**
```rust
// Detect network type
match detect_network() {
    NetworkType::TrustedLAN => {
        // Current behavior
    },
    NetworkType::UntrustedLAN => {
        // Encrypt discovery
        // Require mTLS
    },
    NetworkType::Internet => {
        // Disable UDP broadcast
        // Use rendezvous only
        // Maximum encryption
    },
}
```

2. **Certificate Pinning**
```rust
// Store first-seen cert
// Warn on changes
```

3. **Firewall Rules**
```rust
// Only allow known node IDs
// Rate limit discovery
```

### Short-Term (1-2 months):

1. Implement rendezvous server
2. Add STUN client
3. Add certificate verification
4. Test on internet

### Long-Term (3-6 months):

1. Full NAT traversal (TURN)
2. Mobile client support
3. DHT discovery option
4. Tor integration option

## Answer to Your Questions

### "What's our broadcast look like from the outside?"

**From Internet:** Not visible (stopped at router)

**From Compromised LAN:**
- Node ID (UUID) - okay
- Hostname - reveals machine name
- Local IPs - private network topology
- Capabilities - what you can do
- No secrets/keys - good

**Risk:** Medium on untrusted LAN, Low on internet (not visible)

### "Can we use this for internet connections yet?"

**Short Answer:** Not yet, but foundation is solid.

**What Works:**
- ✅ HTTPS encryption
- ✅ Privacy-first architecture
- ✅ Strong identities

**What's Needed:**
- ❌ Rendezvous server (most critical)
- ❌ NAT traversal
- ❌ Enhanced authentication

**Timeline:** 1-2 months for basic internet support

### "How can Songbird work like RustDesk for roaming?"

**Architecture:**
```rust
// Mobile Songbird connects to rendezvous
// Maintains trust through cryptographic identity
// IP changes don't matter
// Stays connected to federation
```

**Key Insight:**
Your privacy-first, discovery-based architecture is PERFECT for roaming!
- No hardcoded IPs ✅
- Identity-based trust ✅
- Dynamic endpoints ✅

**Just need:**
- Rendezvous server
- Connection migration
- ~1-2 weeks of work

## Next Steps

Want me to:
1. Implement rendezvous server (RustDesk model)?
2. Add network type detection?
3. Create internet deployment guide?
4. Build mobile/roaming prototype?

Your privacy-first architecture is already 80% of what we need for internet deployment!

---

*ecoPrimals - Secure Federation for Untrusted Networks*

