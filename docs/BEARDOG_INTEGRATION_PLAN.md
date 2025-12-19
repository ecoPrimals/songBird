# BearDog Integration Plan

**Target Date:** End of December 2025  
**Status:** Design Phase (BearDog going live)  
**Current Mode:** Standalone JWT authentication  
**Future Mode:** BearDog genetic identity

---

## Overview

Songbird currently operates in **standalone mode** with JWT-based authentication. When BearDog comes online (end of December 2025), we'll integrate genetic identity, hardware binding, and the full entropy hierarchy.

This document outlines the integration strategy, extension points, and migration path.

---

## Current Architecture (Standalone)

### Authentication Flow

```
Student Laptop
    ↓ JWT Token
Songbird Orchestrator
    ↓ Token Validation
Access Control System
    ↓ Capability Check
Task Execution
```

### Token Structure

```rust
pub struct AccessToken {
    pub token_type: TokenType::JWT,
    pub subject: String,          // "student-123"
    pub role: Role::Student,
    pub issued_at: i64,
    pub expires_at: i64,
}
```

**Limitations:**
- No hardware binding
- Password-based (can be stolen)
- No genetic identity
- Limited remote admin security

---

## Future Architecture (BearDog-Enhanced)

### Authentication Flow

```
Student Laptop + SoloKey (optional)
    ↓ BearDog Capability Token
Songbird Orchestrator
    ↓ BearDog Genetic Verification
Access Control System
    ↓ Hierarchical Capability Check
Task Execution (with cryptographic provenance)
```

### Enhanced Token Structure

```rust
pub struct AccessToken {
    pub token_type: TokenType::BearDog,
    pub genetic_signature: GeneticIdentity,  // Who (immutable)
    pub hardware_binding: Option<HardwareKey>, // Where (SoloKey, device TPM)
    pub capabilities: Vec<Capability>,       // What (fine-grained)
    pub entropy_layer: EntropyLayer,         // Trust level
    pub issued_at: i64,
    pub expires_at: i64,
}
```

**Capabilities:**
- Hardware-bound identity (SoloKey/TPM)
- Genetic identity (immutable, transferable)
- Entropy hierarchy (graduated trust)
- Cryptographic provenance (RhizoCrypt)

---

## Entropy Hierarchy

BearDog enforces a graduated trust model based on verification strength.

### Layers

1. **Public Entropy** (Anyone)
   - No auth required
   - View public info only
   - Example: System status page

2. **Device Entropy** (Authenticated)
   - Password/PIN verification
   - Maps to: Student, TA roles
   - Example: Submit tasks, view own results

3. **Genetic Entropy** (Verified Identity)
   - Genetic signature verification
   - Maps to: Professor role
   - Example: Manage course, view all student tasks

4. **Hardware Entropy** (Hardware-Bound)
   - SoloKey/TPM required
   - Maps to: Admin role (local)
   - Example: Manage quotas, configure system

5. **Root Entropy** (Physical Presence)
   - Hardware key + physical presence
   - Maps to: RemoteAdmin role
   - Example: View infrastructure IPs, restart services

### Mapping to Songbird Roles

```rust
impl EntropyLayer {
    pub fn required_for_role(role: &Role) -> Self {
        match role {
            Role::Anonymous => EntropyLayer::Public,
            Role::Student { .. } => EntropyLayer::Device,
            Role::TA { .. } => EntropyLayer::Device,
            Role::Professor { .. } => EntropyLayer::Genetic,
            Role::Admin { .. } => EntropyLayer::Hardware,
            Role::RemoteAdmin { .. } => EntropyLayer::Root,
        }
    }
}
```

---

## Integration Points

### 1. Token Validation

**Current (Standalone):**
```rust
pub async fn validate(&self, token: &AccessToken) -> Result<Identity> {
    // Check JWT expiry
    if token.is_expired() {
        return Err(anyhow!("Token expired"));
    }
    
    Ok(Identity {
        id: token.subject.clone(),
        role: token.role.clone(),
    })
}
```

**Future (BearDog-Enhanced):**
```rust
pub async fn validate(&self, token: &AccessToken) -> Result<Identity> {
    match token.token_type {
        TokenType::JWT => self.validate_jwt(token).await,
        TokenType::BearDog => self.validate_beardog(token).await,
    }
}

async fn validate_beardog(&self, token: &AccessToken) -> Result<Identity> {
    // 1. Verify genetic signature
    let genetic_id = self.beardog_client
        .verify_genetic_signature(&token.genetic_signature)
        .await?;
    
    // 2. Verify hardware binding (if required)
    if let Some(hardware_key) = &token.hardware_binding {
        self.verify_hardware_key(hardware_key).await?;
    }
    
    // 3. Check entropy layer matches role requirements
    let required_entropy = EntropyLayer::required_for_role(&token.role);
    if token.entropy_layer < required_entropy {
        return Err(anyhow!("Insufficient entropy for role"));
    }
    
    // 4. Return verified identity
    Ok(Identity {
        id: genetic_id,
        role: token.role.clone(),
        entropy_layer: token.entropy_layer,
    })
}
```

### 2. Capability Token Issuance

**BearDog Issues Tokens:**
```rust
pub struct BearDogClient {
    endpoint: String,
}

impl BearDogClient {
    pub async fn issue_capability_token(
        &self,
        genetic_id: &GeneticIdentity,
        role: &Role,
        duration: std::time::Duration,
    ) -> Result<AccessToken> {
        // Request BearDog to issue capability token
        let response = self.client
            .post(&format!("{}/capabilities/issue", self.endpoint))
            .json(&CapabilityRequest {
                genetic_id: genetic_id.clone(),
                role: role.clone(),
                duration,
            })
            .send()
            .await?;
        
        Ok(response.json().await?)
    }
}
```

**Songbird validates but doesn't create tokens** (BearDog is source of truth)

### 3. Hardware Key Verification

**SoloKey Integration:**
```rust
pub async fn verify_hardware_key(&self, key: &HardwareKey) -> Result<()> {
    // For admin/remote admin, require SoloKey
    match key {
        HardwareKey::SoloKey { challenge, response } => {
            // Verify FIDO2 challenge-response
            self.fido2_verifier.verify(challenge, response).await?;
        }
        HardwareKey::DeviceTPM { attestation } => {
            // Verify device TPM attestation
            self.tpm_verifier.verify(attestation).await?;
        }
    }
    
    Ok(())
}
```

**Physical Presence Detection:**
```rust
pub async fn verify_physical_presence(&self, key: &HardwareKey) -> Result<()> {
    // For infrastructure access, require physical presence
    // SoloKey must be inserted AND have recent touch
    
    let presence = self.solokey_client
        .check_presence_timeout(std::time::Duration::from_secs(30))
        .await?;
    
    if !presence {
        return Err(anyhow!("Physical presence required"));
    }
    
    Ok(())
}
```

### 4. Genetic Identity

**Immutable, Transferable Identity:**
```rust
pub struct GeneticIdentity {
    pub signature: Vec<u8>,    // Cryptographic signature (immutable)
    pub genesis_block: String, // RhizoCrypt anchor
    pub issued_at: i64,
}

impl GeneticIdentity {
    pub fn verify(&self, beardog_endpoint: &str) -> Result<bool> {
        // Verify against BearDog's RhizoCrypt anchor
        // This signature can't be forged, only transferred with consent
        Ok(true)
    }
}
```

**Key Properties:**
- **Immutable**: Can't be changed or forged
- **Transferable**: Can be delegated with consent
- **Cryptographically anchored**: RhizoCrypt DAG prevents tampering

---

## Migration Strategy

### Phase 1: Dual Mode Support (January 2025)

**Support both JWT and BearDog:**
```rust
pub enum AuthMode {
    Standalone,              // JWT only
    BearDogEnhanced {        // BearDog + JWT fallback
        genetic_verification_endpoint: String,
        hardware_binding_required: bool,
    },
}
```

**Deployment:**
- Campus: Standalone (JWT)
- Home lab: BearDog-enhanced
- Students choose mode

### Phase 2: BearDog Preferred (Q2 2025)

**BearDog default, JWT for onboarding:**
```rust
pub async fn authenticate(&self, request: &AuthRequest) -> Result<AccessToken> {
    // Try BearDog first
    if let Ok(token) = self.try_beardog_auth(request).await {
        return Ok(token);
    }
    
    // Fall back to JWT for new users
    self.jwt_auth(request).await
}
```

**New students:**
1. Start with JWT (email/password)
2. Upgrade to BearDog genetic ID (optional, recommended)
3. Add SoloKey for hardware binding (optional, for admins)

### Phase 3: BearDog Native (Q3 2025)

**Full genetic identity by default:**
- New users get genetic ID at signup
- Hardware binding required for elevated access
- JWT deprecated (only for emergency recovery)

---

## Security Enhancements

### Graduated Disclosure + Entropy

**Current:**
```
Student → Educational Info
TA → Operational Info
Professor → Administrative Info
Admin → Infrastructure Info (no hardware check)
```

**With BearDog:**
```
Device Entropy (password) → Educational Info
Device Entropy (password) → Operational Info
Genetic Entropy (genetic ID) → Administrative Info
Hardware Entropy (SoloKey) → Infrastructure Info (no IPs yet)
Root Entropy (SoloKey + physical presence) → Full Infrastructure Info
```

### Remote Admin Access

**Current Problem:**
- Admin can see infrastructure IPs from anywhere
- Compromised password = full access
- No hardware binding

**BearDog Solution:**
```rust
pub async fn check_infrastructure_access(
    &self,
    token: &AccessToken,
) -> Result<()> {
    // 1. Verify entropy layer
    if token.entropy_layer < EntropyLayer::Root {
        return Err(anyhow!("Infrastructure requires Root entropy"));
    }
    
    // 2. Verify hardware key
    if token.hardware_binding.is_none() {
        return Err(anyhow!("Infrastructure requires hardware key"));
    }
    
    // 3. Verify physical presence (for IP disclosure)
    if self.requires_ip_disclosure() {
        self.verify_physical_presence(&token.hardware_binding.unwrap()).await?;
    }
    
    Ok(())
}
```

**Remote admin can:**
- View node health (no IPs)
- Restart services (no IPs)
- View logs (no IPs)

**Remote admin CANNOT (without physical presence):**
- View internal IPs
- Change network config
- Access raw infrastructure

---

## Implementation Roadmap

### Now (December 2025)

- [x] Standalone JWT authentication working
- [x] Graduated information disclosure
- [x] Capability-based access control
- [x] Extension points for BearDog

### Q1 2025 (January-March)

- [ ] BearDog client library integration
- [ ] Dual mode support (JWT + BearDog)
- [ ] Genetic identity verification endpoint
- [ ] Campus deployment with JWT

### Q2 2025 (April-June)

- [ ] Hardware key integration (SoloKey)
- [ ] Physical presence detection
- [ ] Root entropy for infrastructure access
- [ ] BearDog preferred mode

### Q3 2025 (July-September)

- [ ] Full genetic identity by default
- [ ] JWT deprecated (recovery only)
- [ ] Multi-campus federation with BearDog
- [ ] Academic paper publication

---

## Testing Strategy

### Unit Tests

```rust
#[tokio::test]
async fn test_entropy_layer_enforcement() {
    let ac = AccessControl::new(AuthMode::BearDogEnhanced {
        genetic_verification_endpoint: "http://localhost:9000".into(),
        hardware_binding_required: true,
    });
    
    // Device entropy cannot access infrastructure
    let device_token = AccessToken {
        token_type: TokenType::BearDog,
        entropy_layer: EntropyLayer::Device,
        role: Role::Admin { admin_id: "admin".into() },
        ..Default::default()
    };
    
    assert!(
        !ac.check_access(&device_token, &Capability::ViewInfrastructureInfo).await.unwrap()
    );
    
    // Root entropy CAN access infrastructure
    let root_token = AccessToken {
        token_type: TokenType::BearDog,
        entropy_layer: EntropyLayer::Root,
        role: Role::Admin { admin_id: "admin".into() },
        hardware_binding: Some(HardwareKey::SoloKey { ... }),
        ..Default::default()
    };
    
    assert!(
        ac.check_access(&root_token, &Capability::ViewInfrastructureInfo).await.unwrap()
    );
}
```

### Integration Tests

```bash
# Test JWT mode (now)
cargo test --test orchestrator_integration_tests

# Test BearDog mode (Q1)
BEARDOG_ENDPOINT=http://localhost:9000 cargo test --test beardog_integration_tests

# Test dual mode (Q1)
cargo test --test dual_mode_tests
```

### Hardware Tests

```bash
# Test SoloKey integration (Q2)
cargo test --test solokey_tests -- --test-threads=1 --ignored

# Test physical presence (Q2)
cargo test --test physical_presence_tests -- --test-threads=1 --ignored
```

---

## Success Criteria

### Technical

- [ ] JWT mode works (now)
- [ ] BearDog mode works (Q1)
- [ ] Dual mode works (Q1)
- [ ] Hardware key works (Q2)
- [ ] Physical presence works (Q2)
- [ ] All tests passing

### Security

- [ ] No IP leakage without Root entropy
- [ ] Hardware binding enforced for admin
- [ ] Physical presence required for infrastructure
- [ ] Genetic ID properly verified
- [ ] Audit trail complete

### User Experience

- [ ] Students: seamless (no hardware required)
- [ ] TAs: seamless (no hardware required)
- [ ] Professors: optional genetic ID (no hardware required)
- [ ] Admins: hardware key required (SoloKey provided)
- [ ] Remote admins: limited access until physical presence

---

## BearDog API Contract

### Genetic Identity Verification

```http
POST /identity/verify
Content-Type: application/json

{
  "genetic_signature": "base64-encoded-signature",
  "genesis_block": "rhizocrypt-anchor-hash"
}

Response 200:
{
  "valid": true,
  "identity_id": "genetic-id-abc123",
  "issued_at": 1703001234,
  "trust_score": 0.98
}
```

### Capability Token Issuance

```http
POST /capabilities/issue
Content-Type: application/json

{
  "genetic_id": "genetic-id-abc123",
  "role": "student",
  "course_id": "CSE-847",
  "duration_sec": 86400
}

Response 200:
{
  "token": "beardog-capability-token-...",
  "capabilities": ["view-educational-info", "submit-task", "view-own-tasks"],
  "entropy_layer": "device",
  "expires_at": 1703087634
}
```

### Hardware Key Verification

```http
POST /hardware/verify
Content-Type: application/json

{
  "hardware_key_type": "solokey",
  "challenge": "base64-challenge",
  "response": "base64-fido2-response"
}

Response 200:
{
  "valid": true,
  "hardware_id": "solokey-abc123",
  "physical_presence": true,
  "trust_score": 1.0
}
```

---

## Summary

**Now:** Standalone JWT mode, ready for campus deployment.

**Q1 2025:** BearDog integration begins, dual mode support.

**Q2 2025:** Hardware key integration, full entropy hierarchy.

**Q3 2025:** BearDog native, genetic identity by default.

**Extension points are ready. Integration will be seamless.** 🐕🎵✨

