# 🌐 Protocol Evolution - Refined for Songbird Core

**Date**: February 8, 2026  
**Session**: Protocol Refinement  
**Focus**: Protocol-level only, Dark Forest compliance, no UI  
**Status**: 🎯 **REFINED RECOMMENDATIONS**

---

## 📋 Clarifications from Upstream

1. **Stay in Songbird** - No UI work, protocol layer only
2. **WireGuard is external** - Not for inter-primal comms, but can work as beacon site
3. **NFC must follow Dark Forest** - Zero metadata leakage, encrypted genesis only

---

## 🎯 Revised Protocol Priorities

### Priority 1: QUIC Protocol Layer ⭐⭐⭐

**Why**: Modern transport protocol, better than TCP for inter-primal comms

**Implementation**: `crates/songbird-quic/` (new crate)

**Capabilities**:
- 0-RTT connection establishment (faster than TLS 1.3)
- Multiplexed streams (no head-of-line blocking)
- Connection migration (survives IP changes)
- Built-in congestion control
- UDP-based (better NAT traversal than TCP)

**Dark Forest Integration**:
- All streams BearDog-encrypted
- No plaintext metadata in QUIC headers
- Connection IDs are ephemeral, non-correlatable
- SNI encrypted (no domain name leakage)

**Rust Crate**: `quinn` (pure Rust, production-ready)

**Protocol Stack**:
```rust
Application Data
    ↓
BearDog ChaCha20Poly1305 (application-level)
    ↓
QUIC Transport (quinn)
    ↓ (built-in encryption)
UDP
    ↓
IPv4/IPv6
```

**Implementation Effort**: 3-5 days

**Integration Points**:
- New transport option in multi-path strategy (Tier 2.5: between Onion and IPv4)
- Works with existing beacon system
- Falls back to TCP if QUIC unavailable
- Automatic protocol negotiation (ALPN)

**JSON-RPC Methods**:
```json
// Start QUIC listener
{"method": "quic.listen", "params": {"port": 4433, "bind_addr": "[::]"}}

// Connect via QUIC
{"method": "quic.connect", "params": {"peer_addr": "[2600::27]:4433"}}

// Get QUIC status
{"method": "quic.status"}
```

**Why This Matters**:
- Faster reconnection than TCP (0-RTT vs 3-way handshake)
- Better mobile performance (connection migration)
- Standard protocol (HTTP/3 uses QUIC)
- Better than WebSocket for real-time

**Status**: HIGH PRIORITY - Direct protocol improvement

---

### Priority 2: WireGuard as Beacon Site 💡

**Clarification**: WireGuard is NOT for inter-primal comms, it's an **external VPN that can be part of the beacon**

**Use Case**: Family member runs WireGuard VPN, advertises endpoint in beacon

**Architecture**:
```
Tower (running WireGuard server externally)
    ↓
Beacon includes: wg_endpoint: "1.2.3.4:51820"
    ↓ (BearDog-encrypted beacon)
DNS/Rendezvous/IPFS
    ↓
Family Member decrypts beacon
    ↓
Sees WireGuard endpoint as connection option
    ↓
Connects through WireGuard tunnel (external to Songbird)
    ↓
Once inside tunnel, uses normal Songbird protocols
```

**Implementation**: `crates/songbird-universal-ipc/src/handlers/beacon_handler.rs`

**New Beacon Fields**:
```rust
pub struct EncryptedBeacon {
    pub family_id: String,
    pub node_id: String,
    pub endpoints: Vec<Endpoint>,
    pub external_tunnels: Vec<ExternalTunnel>,  // NEW
    pub capabilities: Vec<String>,
}

pub struct ExternalTunnel {
    pub tunnel_type: TunnelType,  // WireGuard, OpenVPN, etc.
    pub endpoint: String,          // IP:port
    pub public_key: String,        // For WireGuard
    pub metadata: HashMap<String, String>,
}

pub enum TunnelType {
    WireGuard,
    OpenVPN,
    IPsec,
    // Future: ZeroTier, Tailscale, etc.
}
```

**Beacon Generation**:
```json
// Add external tunnel to beacon
{"method": "beacon.add_external_tunnel", "params": {
    "tunnel_type": "wireguard",
    "endpoint": "1.2.3.4:51820",
    "public_key": "base64_pubkey"
}}

// Beacon now includes external tunnel info (encrypted)
{"method": "birdsong.generate_encrypted_beacon"}
```

**Connection Flow**:
```
1. Peer decrypts beacon
2. Sees WireGuard endpoint available
3. Peer connects to WireGuard (external process/app)
4. Once inside VPN tunnel, peer can reach Tower's local address
5. Songbird uses normal protocols over the tunnel
```

**Implementation Effort**: 1-2 days (just beacon metadata, no protocol implementation)

**Why This Matters**:
- Leverages existing VPN infrastructure
- Adds another connection path to multi-path strategy
- No Songbird protocol work needed (WireGuard is external)
- Works with any external tunnel solution

**Status**: LOW EFFORT, HIGH VALUE - Simple beacon extension

---

### Priority 3: NFC Genesis with Dark Forest Protocol ⭐⭐⭐

**Critical Requirement**: ZERO metadata leakage during NFC exchange

**Implementation**: `crates/songbird-genesis/src/physical_channels/nfc.rs`

**Dark Forest Compliance**:

#### 1. No Plaintext Metadata

**Problem**: Standard NFC NDEF records leak metadata (device name, capabilities, etc.)

**Solution**: Custom binary protocol, all metadata encrypted

```rust
// NFC Exchange Protocol (Dark Forest compliant)
pub struct NfcGenesisProtocol {
    version: u8,                    // Protocol version
    encrypted_payload: Vec<u8>,     // BearDog-encrypted
    ephemeral_nonce: [u8; 24],      // One-time nonce
    // NO device metadata, NO plaintext identifiers
}

impl NfcGenesisProtocol {
    /// Exchange encrypted genesis credentials via NFC
    /// 
    /// Dark Forest guarantees:
    /// - No device identifiers transmitted
    /// - No capability metadata leaked
    /// - No timing correlation possible
    /// - Ephemeral keys only
    /// - Forward secrecy
    pub async fn exchange(&self, nfc_device: &mut NfcDevice) -> Result<GenesisCredentials> {
        // 1. Generate ephemeral X25519 keypair (BearDog)
        let ephemeral_keypair = self.beardog.generate_ephemeral_keypair().await?;
        
        // 2. Exchange public keys via NFC (no metadata)
        let peer_public = nfc_device.exchange_public_key(ephemeral_keypair.public).await?;
        
        // 3. Derive shared secret (BearDog)
        let shared_secret = self.beardog.x25519_diffie_hellman(
            ephemeral_keypair.secret,
            peer_public
        ).await?;
        
        // 4. Encrypt genesis credentials (BearDog)
        let encrypted_genesis = self.beardog.encrypt_genesis(
            &self.genesis_credentials,
            &shared_secret,
            &ephemeral_nonce
        ).await?;
        
        // 5. Exchange encrypted payload (no plaintext metadata)
        nfc_device.send(encrypted_genesis).await?;
        let peer_encrypted = nfc_device.receive().await?;
        
        // 6. Decrypt peer's genesis (BearDog)
        let peer_genesis = self.beardog.decrypt_genesis(
            peer_encrypted,
            &shared_secret,
            &peer_nonce
        ).await?;
        
        // 7. Verify family lineage (BearDog)
        self.beardog.verify_lineage(&peer_genesis.family_id).await?;
        
        // 8. Destroy ephemeral keys (forward secrecy)
        self.beardog.destroy_ephemeral(ephemeral_keypair.secret).await?;
        
        Ok(peer_genesis)
    }
}
```

#### 2. Timing Attack Prevention

**Problem**: NFC exchange timing could leak information

**Solution**: Constant-time operations, random delays

```rust
impl NfcGenesisProtocol {
    async fn exchange_with_timing_protection(&self) -> Result<GenesisCredentials> {
        // Add random delay (1-5 seconds) before starting
        let random_delay = self.beardog.random_delay_ms(1000, 5000).await?;
        tokio::time::sleep(Duration::from_millis(random_delay)).await;
        
        // Constant-time NFC operations
        let start = Instant::now();
        let result = self.exchange_internal().await;
        let elapsed = start.elapsed();
        
        // Pad to constant time (10 seconds)
        if elapsed < Duration::from_secs(10) {
            let padding = Duration::from_secs(10) - elapsed;
            tokio::time::sleep(padding).await;
        }
        
        result
    }
}
```

#### 3. No Correlation Between Devices

**Problem**: Multiple NFC exchanges could be correlated

**Solution**: Fresh ephemeral keys every exchange, no reuse

```rust
/// Each genesis ceremony gets fresh keys
pub struct GenesisSession {
    session_id: Uuid,               // Random, not stored
    ephemeral_x25519: X25519Keypair, // Generated per-session
    ephemeral_ed25519: Ed25519Keypair, // For signing only
    // NO long-term identifiers
    // NO device fingerprints
}

impl GenesisSession {
    /// Create session with zero correlation to previous sessions
    pub async fn new(beardog: &BeardogClient) -> Result<Self> {
        Ok(Self {
            session_id: Uuid::new_v4(), // Never transmitted
            ephemeral_x25519: beardog.generate_ephemeral_x25519().await?,
            ephemeral_ed25519: beardog.generate_ephemeral_ed25519().await?,
        })
    }
    
    /// Destroy all ephemeral material after exchange
    pub async fn destroy(&mut self, beardog: &BeardogClient) -> Result<()> {
        beardog.destroy_key(&self.ephemeral_x25519.secret).await?;
        beardog.destroy_key(&self.ephemeral_ed25519.secret).await?;
        
        // Overwrite memory (paranoid)
        self.session_id = Uuid::nil();
        
        Ok(())
    }
}
```

#### 4. Proximity Proof Without Identity

**Problem**: Need to prove physical proximity without revealing who

**Solution**: Challenge-response with ephemeral keys

```rust
pub struct ProximityProof {
    /// Random challenge (not stored)
    challenge: [u8; 32],
    
    /// Response signed by ephemeral key
    response_signature: Vec<u8>,
    
    /// Timestamp (rounded to minute for privacy)
    rounded_timestamp: u64,
    
    // NO device identifiers
    // NO MAC addresses
    // NO Bluetooth addresses
}

impl ProximityProof {
    /// Verify proximity without identity revelation
    pub async fn verify(&self, beardog: &BeardogClient) -> Result<()> {
        // Verify response matches challenge
        beardog.verify_signature(
            &self.challenge,
            &self.response_signature,
            // Public key from NFC exchange (ephemeral)
        ).await?;
        
        // Verify timestamp freshness (within 5 minutes)
        let now_rounded = Self::round_to_minute(Utc::now());
        if (now_rounded - self.rounded_timestamp).abs() > 300 {
            return Err(GenesisError::ProximityTimeout);
        }
        
        Ok(())
    }
    
    fn round_to_minute(timestamp: DateTime<Utc>) -> u64 {
        // Round to minute boundary (privacy protection)
        timestamp.timestamp() / 60 * 60
    }
}
```

#### 5. NFC Protocol Wire Format

**Dark Forest compliant binary format**:

```
[1 byte]   Protocol version (0x01)
[1 byte]   Message type (0x01 = genesis_request, 0x02 = genesis_response)
[2 bytes]  Payload length (big-endian)
[32 bytes] Ephemeral public key (X25519)
[24 bytes] Nonce
[N bytes]  Encrypted payload (BearDog ChaCha20Poly1305)
[64 bytes] Signature (ephemeral Ed25519)

Total overhead: 124 bytes + encrypted payload
No plaintext metadata
No device identifiers
```

#### 6. Platform Integration

**Android** (`src/platform/android.rs`):
```rust
#[cfg(target_os = "android")]
pub struct AndroidNfcTransport {
    // Use Android NFC API via JNI
    // No system identifiers exposed
}

impl AndroidNfcTransport {
    pub async fn exchange_dark_forest(&mut self, payload: &[u8]) -> Result<Vec<u8>> {
        // JNI call to Android NFC
        // Send/receive binary payload only
        // No NDEF, no metadata
        self.send_binary(payload).await?;
        let response = self.receive_binary().await?;
        Ok(response)
    }
}
```

**iOS** (`src/platform/ios.rs`):
```rust
#[cfg(target_os = "ios")]
pub struct IosNfcTransport {
    // Use Core NFC via FFI
    // No system identifiers exposed
}

impl IosNfcTransport {
    pub async fn exchange_dark_forest(&mut self, payload: &[u8]) -> Result<Vec<u8>> {
        // FFI call to Core NFC
        // Binary mode only (no NDEF)
        self.send_binary(payload).await?;
        let response = self.receive_binary().await?;
        Ok(response)
    }
}
```

**Linux/Desktop** (`src/platform/linux.rs`):
```rust
#[cfg(target_os = "linux")]
pub struct LibnfcTransport {
    // Use libnfc for desktop NFC readers
    // No system identifiers exposed
}
```

**Implementation Effort**: 5-7 days
- 2 days: Core protocol with Dark Forest guarantees
- 2 days: Android integration
- 2 days: iOS integration
- 1 day: Testing and validation

**Status**: HIGH PRIORITY - Critical for secure genesis ceremonies

---

## 🔒 Dark Forest Protocol Checklist

Every new protocol MUST satisfy:

### Zero Metadata Leakage ✅

- [ ] No plaintext device identifiers
- [ ] No MAC addresses transmitted
- [ ] No Bluetooth addresses transmitted
- [ ] No device names/models
- [ ] No OS version info
- [ ] No app version info
- [ ] No capability fingerprinting

### Ephemeral Credentials ✅

- [ ] Fresh keys per session
- [ ] No key reuse across sessions
- [ ] Automatic key destruction after use
- [ ] Forward secrecy guaranteed
- [ ] No long-term correlation possible

### Timing Protection ✅

- [ ] Constant-time operations
- [ ] Random delays added
- [ ] Timestamps rounded (minute boundary)
- [ ] No sub-second timing correlation

### BearDog-Only Crypto ✅

- [ ] All crypto via BearDog
- [ ] No embedded secrets
- [ ] No direct crypto operations
- [ ] Entropy from BearDog
- [ ] Key derivation via BearDog

### Proximity Proof ✅

- [ ] Physical proximity verified
- [ ] No identity revelation required
- [ ] Challenge-response protocol
- [ ] Replay attack prevention
- [ ] Distance bounding (optional)

---

## 🎯 Revised Implementation Roadmap

### Phase 1: Core Protocol Enhancement (1 week)

**Day 1-3: QUIC Integration**
- Integrate `quinn` crate
- BearDog encryption layer
- Protocol negotiation (ALPN)
- Fallback to TCP

**Day 4-5: WireGuard Beacon Extension**
- Extend beacon metadata
- Add `external_tunnels` field
- JSON-RPC methods
- Beacon generation/parsing

**Deliverable**: QUIC working, WireGuard-as-beacon integrated

### Phase 2: Dark Forest NFC Genesis (1 week)

**Day 1-2: Core Protocol**
- Dark Forest-compliant exchange protocol
- Zero metadata leakage guarantees
- Timing protection
- Ephemeral key management

**Day 3-4: Android Integration**
- JNI bindings to Android NFC
- Binary payload exchange
- Testing on physical devices

**Day 5-6: iOS Integration**
- FFI bindings to Core NFC
- Binary payload exchange
- Testing on physical devices

**Day 7: Testing & Validation**
- Dark Forest compliance verification
- Metadata leakage testing
- Timing attack testing
- Cross-platform validation

**Deliverable**: NFC genesis working on Android/iOS with Dark Forest guarantees

### Phase 3: Multi-Path Integration (3 days)

**Day 1: Protocol Priority Updates**
```
TIER 1:    IPv6 Direct
TIER 2:    Sovereign Onion
TIER 2.5:  QUIC (NEW)
TIER 3:    IPv4 Direct (IGD)
TIER 4:    WebSocket (firewall bypass)
TIER 5:    LAN Direct (mDNS)
TIER 6:    External Tunnel (WireGuard, etc.) (NEW)
TIER 7:    STUN Hole-Punch
TIER 8:    Family Relay
TIER 9:    Full Tor (anonymity)
```

**Day 2: Protocol Selection Logic**
- Try QUIC before falling back to TCP
- Check for external tunnel availability
- Automatic negotiation

**Day 3: Testing & Documentation**
- Multi-path integration tests
- Protocol fallback testing
- Documentation updates

**Deliverable**: Complete multi-path strategy with new protocols

---

## 📊 Effort Summary

| Component | Effort | Priority | Dark Forest |
|-----------|--------|----------|-------------|
| QUIC Protocol | 3 days | HIGH | ✅ Yes |
| WireGuard Beacon | 2 days | MEDIUM | ✅ Yes (encrypted beacon) |
| NFC Genesis | 7 days | HIGH | ✅ CRITICAL |
| Multi-Path Integration | 3 days | HIGH | N/A |
| **Total** | **15 days** | - | - |

---

## 🔐 Security Guarantees

### QUIC
- BearDog-encrypted application layer
- No SNI leakage (encrypted)
- Ephemeral connection IDs
- Perfect forward secrecy

### WireGuard Beacon
- Beacon fully encrypted (BearDog)
- No plaintext tunnel endpoints
- Only family can decrypt beacon
- External tunnel = external security model

### NFC Genesis
- Zero metadata leakage
- Ephemeral keys only
- No device correlation
- Timing attack resistant
- Proximity verified without identity
- BearDog-only crypto

---

## ✅ Success Criteria

### QUIC Integration
- [ ] Faster than TCP for reconnection (0-RTT working)
- [ ] Connection migration works (IP change survives)
- [ ] Falls back to TCP if unavailable
- [ ] BearDog encryption layer working
- [ ] Compatible with existing beacon system

### WireGuard Beacon
- [ ] External tunnel info in beacon (encrypted)
- [ ] Beacon generation includes tunnel metadata
- [ ] Beacon parsing extracts tunnel info
- [ ] Connection attempt via tunnel works
- [ ] No plaintext tunnel endpoints leaked

### NFC Genesis (Dark Forest Compliance)
- [ ] Zero metadata leakage verified
- [ ] No device fingerprinting possible
- [ ] Timing attacks prevented
- [ ] Ephemeral keys destroyed after use
- [ ] Works on Android and iOS
- [ ] Proximity verified without identity
- [ ] BearDog-only crypto (no embedded secrets)

---

## 🚀 Next Steps

1. **Approve refined roadmap** (15 days total)
2. **Start with QUIC** (3 days, immediate protocol improvement)
3. **Then NFC Genesis** (7 days, critical for secure onboarding)
4. **Finally WireGuard Beacon** (2 days, simple beacon extension)
5. **Integrate into multi-path** (3 days, complete system)

---

**Specification Version**: 1.0  
**Last Updated**: February 8, 2026  
**Status**: Ready for Implementation  
**Focus**: Protocol-level only, Dark Forest compliant, no UI

🦀 **Pure Rust** | 🔒 **Dark Forest Protocol** | 🧬 **Sovereign Architecture** | 🐕 **BearDog Crypto**
