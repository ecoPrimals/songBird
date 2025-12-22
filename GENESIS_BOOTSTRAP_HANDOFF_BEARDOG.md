# 🔐 BearDog Genesis Bootstrap Handoff

**Date**: December 22, 2025  
**Status**: 🚧 Songbird Ready → BearDog Implementation Needed  
**Priority**: P0 - Core Security Feature

---

## 🎯 Executive Summary

**What**: Physical proximity genesis bootstrap for new nodes  
**Why**: "Never let a bird be alone in the dark forest"  
**Goal**: New nodes born with cryptographic identity witnessed by multiple primals

**Songbird Status**: ✅ **Ready**
- ✅ Genesis ceremony module complete (`songbird-genesis`)
- ✅ BirdSong payload integration complete
- ✅ Mock genesis test passing (showcase/14-physical-genesis/)
- ✅ Multi-primal coordination framework ready

**BearDog Status**: 🚧 **Implementation Needed**
- 🔜 Genesis lineage establishment
- 🔜 Witness signature verification
- 🔜 Physical proof validation

---

## 📋 What BearDog Needs to Do

### Phase 1: Genesis Lineage Establishment (Week 1-2)

**Goal**: Generate genetic lineage for new nodes during genesis

**New Module**: `beardog-lineage/src/genesis.rs`

```rust
/// Genesis lineage establishment for new nodes
pub trait GenesisLineageProvider {
    /// Create genetic lineage for a new node during genesis ceremony
    ///
    /// This is called by the genesis coordinator (Songbird) when a new
    /// node is being born. BearDog establishes the genetic lineage
    /// based on the witness device and existing primal witnesses.
    async fn establish_genesis_lineage(
        &self,
        request: GenesisLineageRequest,
    ) -> Result<GenesisLineageResponse>;

    /// Verify genesis lineage proof from another primal
    ///
    /// During multi-primal coordination, BearDog verifies lineage
    /// proofs from other primals (e.g., Songbird).
    async fn verify_genesis_lineage(
        &self,
        proof: GenesisLineageProof,
    ) -> Result<LineageVerificationResult>;
}

/// Request to establish genesis lineage for a new node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisLineageRequest {
    /// New node identifier
    pub new_node_id: String,

    /// Genesis ceremony ID (for correlation)
    pub ceremony_id: String,

    /// Physical witness device (SoloKey, etc.)
    pub witness_device_id: String,
    pub witness_pubkey: Vec<u8>,

    /// Physical channel used
    pub physical_channel: PhysicalChannelType,

    /// Other primal witnesses (for multi-primal verification)
    pub primal_witnesses: Vec<PrimalWitnessInfo>,

    /// Genesis timestamp
    pub birth_timestamp: DateTime<Utc>,
}

/// Response with genetic lineage for new node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisLineageResponse {
    /// Genetic lineage data (BearDog-specific)
    pub lineage_data: Vec<u8>,

    /// BearDog's signature over the lineage
    pub signature: Vec<u8>,

    /// Lineage root ID
    pub lineage_root: String,

    /// Trust level achieved
    pub trust_level: GenesisTrustLevel,

    /// When this lineage was established
    pub established_at: DateTime<Utc>,
}
```

**REST API Endpoints** (add to beardog-integration):

```
POST /genesis/lineage/establish
  - Body: GenesisLineageRequest
  - Response: GenesisLineageResponse
  - Creates genetic lineage for new node

POST /genesis/lineage/verify
  - Body: GenesisLineageProof
  - Response: LineageVerificationResult
  - Verifies genesis lineage from another primal
```

---

### Phase 2: Witness Signature Verification (Week 2-3)

**Goal**: Verify physical witness signatures (SoloKey, etc.)

**New Module**: `beardog-crypto/src/witness.rs`

```rust
/// Physical witness verification for genesis
pub trait WitnessVerifier {
    /// Verify signature from a physical witness device
    ///
    /// Validates FIDO2/WebAuthn signatures from hardware keys.
    async fn verify_witness_signature(
        &self,
        witness: GenesisWitnessInfo,
    ) -> Result<WitnessVerificationResult>;

    /// Verify batch of witness signatures (for efficiency)
    async fn verify_witness_signatures_batch(
        &self,
        witnesses: Vec<GenesisWitnessInfo>,
    ) -> Result<Vec<WitnessVerificationResult>>;
}

/// Genesis witness information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisWitnessInfo {
    /// Witness device ID (e.g., SoloKey serial)
    pub device_id: String,

    /// Public key from witness device
    pub public_key: Vec<u8>,

    /// Signature over the new node's identity
    pub signature: Vec<u8>,

    /// Physical channel type
    pub channel: PhysicalChannelType,

    /// Additional attestation data (FIDO2 specific)
    pub attestation: Option<Vec<u8>>,

    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Physical channel type (same as Songbird)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PhysicalChannelType {
    HardwareKey,      // FIDO2/WebAuthn (highest trust)
    QrCode,           // QR + out-of-band verification
    Bluetooth,        // BLE proximity
    Custom(String),   // Extensibility
}

/// Result of witness verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessVerificationResult {
    /// Was the signature valid?
    pub valid: bool,

    /// Trust level achieved
    pub trust_level: GenesisTrustLevel,

    /// Error message if invalid
    pub error: Option<String>,

    /// Attestation verified (for FIDO2)
    pub attestation_verified: bool,
}
```

**REST API Endpoints**:

```
POST /genesis/witness/verify
  - Body: GenesisWitnessInfo
  - Response: WitnessVerificationResult
  - Verifies physical witness signature

POST /genesis/witness/verify-batch
  - Body: Vec<GenesisWitnessInfo>
  - Response: Vec<WitnessVerificationResult>
  - Batch verification for efficiency
```

**Dependencies Needed**:
```toml
[dependencies]
webauthn-rs = "0.5"  # For FIDO2/WebAuthn verification
```

---

### Phase 3: Physical Proof Validation (Week 3-4)

**Goal**: Validate physical proximity proofs

**New Module**: `beardog-crypto/src/physical_proof.rs`

```rust
/// Physical proximity proof validation
pub trait PhysicalProofVerifier {
    /// Verify physical proximity proof
    ///
    /// Validates that physical channel proof is genuine and recent.
    async fn verify_physical_proof(
        &self,
        proof: PhysicalProximityProof,
    ) -> Result<ProofVerificationResult>;

    /// Get expected proof format for a channel type
    fn get_proof_requirements(
        &self,
        channel: PhysicalChannelType,
    ) -> ProofRequirements;
}

/// Physical proximity proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalProximityProof {
    /// Physical channel used
    pub channel: PhysicalChannelType,

    /// Proof data (channel-specific format)
    pub proof_data: Vec<u8>,

    /// Nonce (prevents replay)
    pub nonce: Vec<u8>,

    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Requirements for physical proof
#[derive(Debug, Clone)]
pub struct ProofRequirements {
    /// Minimum proof data length
    pub min_length: usize,

    /// Maximum age (seconds)
    pub max_age_seconds: u64,

    /// Required attestation fields
    pub required_attestation: bool,

    /// Trust level achievable
    pub max_trust_level: GenesisTrustLevel,
}

/// Result of proof verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofVerificationResult {
    /// Was the proof valid?
    pub valid: bool,

    /// Trust level achieved
    pub trust_level: GenesisTrustLevel,

    /// Proof age (seconds)
    pub age_seconds: u64,

    /// Error message if invalid
    pub error: Option<String>,
}
```

**REST API Endpoints**:

```
POST /genesis/proof/verify
  - Body: PhysicalProximityProof
  - Response: ProofVerificationResult
  - Verifies physical proximity proof

GET /genesis/proof/requirements?channel=HardwareKey
  - Response: ProofRequirements
  - Gets proof requirements for a channel type
```

---

### Phase 4: Genesis Tunnel Support (Week 4)

**Goal**: Special BTSP tunnels for genesis credential exchange

**Enhance**: `beardog-tunnel/src/genesis_tunnel.rs`

```rust
/// Genesis-specific tunnel operations
pub trait GenesisTunnelProvider {
    /// Create a genesis tunnel for credential exchange
    ///
    /// Genesis tunnels are short-lived, high-security tunnels
    /// used only during the genesis ceremony.
    async fn create_genesis_tunnel(
        &self,
        request: GenesisTunnelRequest,
    ) -> Result<GenesisTunnelSession>;

    /// Exchange genesis credentials over tunnel
    async fn exchange_genesis_credentials(
        &self,
        tunnel: &GenesisTunnelSession,
        credentials: GenesisCredentials,
    ) -> Result<ExchangeResult>;

    /// Close genesis tunnel
    async fn close_genesis_tunnel(
        &self,
        tunnel_id: &str,
    ) -> Result<()>;
}

/// Genesis tunnel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisTunnelRequest {
    /// Witness device as tunnel peer
    pub witness_device_id: String,

    /// New node as tunnel peer
    pub new_node_id: String,

    /// Physical channel (for trust level)
    pub physical_channel: PhysicalChannelType,

    /// Maximum tunnel lifetime (seconds)
    pub max_lifetime_seconds: u64,
}

/// Genesis tunnel session
#[derive(Debug, Clone)]
pub struct GenesisTunnelSession {
    /// Tunnel ID
    pub tunnel_id: String,

    /// Tunnel created at
    pub created_at: DateTime<Utc>,

    /// Tunnel expires at
    pub expires_at: DateTime<Utc>,

    /// Is tunnel still active?
    pub active: bool,
}

/// Genesis credentials for exchange
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisCredentials {
    /// New node public key
    pub node_pubkey: Vec<u8>,

    /// Witness signature
    pub witness_signature: Vec<u8>,

    /// Additional encrypted data
    pub encrypted_data: Vec<u8>,
}
```

**REST API Endpoints**:

```
POST /genesis/tunnel/create
  - Body: GenesisTunnelRequest
  - Response: GenesisTunnelSession
  - Creates genesis tunnel

POST /genesis/tunnel/{id}/exchange
  - Body: GenesisCredentials
  - Response: ExchangeResult
  - Exchanges credentials

DELETE /genesis/tunnel/{id}
  - Response: 204 No Content
  - Closes genesis tunnel
```

---

## 🧪 Testing Requirements

**BearDog Unit Tests** (create in each module):

```bash
# Test genesis lineage
cargo test -p beardog-lineage genesis

# Test witness verification
cargo test -p beardog-crypto witness

# Test physical proof validation
cargo test -p beardog-crypto physical_proof

# Test genesis tunnels
cargo test -p beardog-tunnel genesis
```

**Integration Tests with Songbird**:

```bash
# Songbird has created these tests - BearDog needs to pass them:
cd /path/to/songbird

# Test multi-primal coordination
./showcase/14-physical-genesis/04-multi-primal-coordination.sh

# Test genesis to discovery flow
./showcase/14-physical-genesis/05-genesis-to-discovery.sh

# Test genesis verification
./showcase/14-physical-genesis/06-genesis-verification.sh
```

---

## 📊 Integration Points with Songbird

### How Songbird Calls BearDog During Genesis

```rust
// 1. Songbird initiates genesis ceremony
let ceremony = GenesisCeremony::new(config);
ceremony.start().await?;

// 2. Songbird requests lineage from BearDog
let beardog_response = beardog_client
    .post("/genesis/lineage/establish")
    .json(&GenesisLineageRequest {
        new_node_id: "pixel-8a-123",
        ceremony_id: ceremony.id(),
        witness_device_id: "solokey-456",
        witness_pubkey: witness_key,
        physical_channel: PhysicalChannelType::HardwareKey,
        primal_witnesses: vec![songbird_witness],
        birth_timestamp: Utc::now(),
    })
    .send()
    .await?
    .json::<GenesisLineageResponse>()
    .await?;

// 3. Songbird includes BearDog's lineage in genesis certificate
genesis_cert.add_primal_lineage("beardog", beardog_response);

// 4. New node broadcasts genesis via BirdSong
let birdsong_payload = EncryptedBirdSong {
    version: 1,
    ciphertext: encrypted_discovery,
    lineage_hint: LineageHint::Universal,
    timestamp: Utc::now(),
    signature: node_signature,
    genesis_witness: Some(genesis_cert.into_proof()), // ← NEW!
};
```

### BirdSong Payload Enhancement

Songbird has already added `genesis_witness` to `EncryptedBirdSong`:

```rust:48:76:crates/songbird-network-federation/src/beardog/birdsong.rs
/// Encrypted birdSong message
///
/// This is what gets broadcast over UDP.
/// Only family (with lineage proof) can decrypt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedBirdSong {
    /// Protocol version (for future evolution)
    pub version: u8,

    /// Encrypted payload
    pub ciphertext: Vec<u8>,

    /// Hint about which lineage can decrypt
    /// NOT a full lineage proof, just a hint for key selection
    pub lineage_hint: LineageHint,

    /// Timestamp (prevents replay attacks)
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Signature (proves authenticity, NOT for decryption)
    pub signature: Vec<u8>,

    /// Optional genesis witness for new nodes
    /// Present when a new node is broadcasting its genesis certification
    /// during initial discovery ("bird in a dark forest")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genesis_witness: Option<GenesisWitnessProof>,
}
```

BearDog needs to decrypt and verify this `genesis_witness` field when present.

---

## 🛠️ Implementation Checklist

### Week 1-2: Genesis Lineage
- [ ] Create `beardog-lineage/src/genesis.rs`
- [ ] Implement `GenesisLineageProvider` trait
- [ ] Add REST endpoints: `/genesis/lineage/establish`, `/genesis/lineage/verify`
- [ ] Add unit tests
- [ ] Test with Songbird showcase scripts

### Week 2-3: Witness Verification
- [ ] Create `beardog-crypto/src/witness.rs`
- [ ] Implement `WitnessVerifier` trait
- [ ] Add `webauthn-rs` dependency for FIDO2 support
- [ ] Add REST endpoints: `/genesis/witness/verify`, `/genesis/witness/verify-batch`
- [ ] Add unit tests for SoloKey verification
- [ ] Test with real SoloKey hardware (if available)

### Week 3-4: Physical Proof Validation
- [ ] Create `beardog-crypto/src/physical_proof.rs`
- [ ] Implement `PhysicalProofVerifier` trait
- [ ] Add REST endpoints: `/genesis/proof/verify`, `/genesis/proof/requirements`
- [ ] Add unit tests
- [ ] Test with Songbird showcase scripts

### Week 4: Genesis Tunnels
- [ ] Create `beardog-tunnel/src/genesis_tunnel.rs`
- [ ] Implement `GenesisTunnelProvider` trait
- [ ] Add REST endpoints: `/genesis/tunnel/*`
- [ ] Add unit tests
- [ ] Test with Songbird showcase scripts

### Week 5: Integration Testing
- [ ] Run all Songbird showcase tests
- [ ] Fix any integration issues
- [ ] Performance testing (genesis ceremony latency)
- [ ] Security audit (physical proof validation)
- [ ] Documentation update

---

## 📈 Success Metrics

**Functional**:
- ✅ Genesis ceremony completes in < 5 seconds
- ✅ Multi-primal coordination with 2+ primals
- ✅ Physical witness verification (SoloKey)
- ✅ New node has genetic lineage from birth
- ✅ Genesis witness appears in BirdSong broadcasts

**Security**:
- ✅ FIDO2 attestation properly verified
- ✅ Replay attack protection (nonce + timestamp)
- ✅ Trust levels correctly computed
- ✅ Genesis tunnels auto-expire (< 60 seconds)

**Performance**:
- ✅ Witness verification: < 100ms
- ✅ Lineage establishment: < 1 second
- ✅ Batch verification: < 10ms per witness

---

## 🔄 Communication

**Questions?**
- Reach out via Songbird team
- Check `PHYSICAL_GENESIS_IMPLEMENTATION_PLAN.md` for architecture details
- Review `showcase/14-physical-genesis/` for test examples

**Status Updates**:
- Weekly progress updates appreciated
- Post blockers immediately
- Share integration test results

---

## 📚 Reference Documents

**Songbird Side** (already complete):
- `crates/songbird-genesis/` - Genesis ceremony module
- `crates/songbird-network-federation/src/beardog/genesis.rs` - Genesis types
- `showcase/14-physical-genesis/` - Test scripts
- `PHYSICAL_GENESIS_IMPLEMENTATION_PLAN.md` - Full plan
- `BEARDOG_GENESIS_HANDOFF_DEC_22_2025.md` - This document

**BearDog Side** (for you to create):
- `beardog-lineage/src/genesis.rs`
- `beardog-crypto/src/witness.rs`
- `beardog-crypto/src/physical_proof.rs`
- `beardog-tunnel/src/genesis_tunnel.rs`

---

## 🚀 Let's Make Genesis Happen!

**Goal**: "Never let a bird be alone in the dark forest"

**Timeline**: 4-5 weeks  
**Priority**: P0 (Core security feature)  
**Songbird Status**: ✅ Ready and waiting

When BearDog is ready, Songbird can test immediately using:
```bash
cd songbird
./showcase/14-physical-genesis/04-multi-primal-coordination.sh
```

---

**Last Updated**: December 22, 2025  
**Prepared By**: Songbird Team  
**For**: BearDog Team

🐻🔐✨ **Let's build the safest node genesis ever!** ✨🔐🐦

