# 🐦🐕 Songbird ↔ BearDog Cross-Primal Coordination

**Last Updated**: January 13, 2026  
**Status**: ✅ **ACTIVE - LIVESPORE EVOLUTION**

---

## 🎯 CURRENT INITIATIVE: LiveSpore Support

**Request From**: BearDog Team (via BiomeOS coordination)  
**Request Date**: January 13, 2026  
**Songbird Response**: ✅ APPROVED  
**Timeline**: 6 weeks (ending February 24, 2026)

---

## 📋 ACTIVE COORDINATION POINTS

### 1. BearDog Provides to Songbird

**✅ Ready Now**:
- `concurrent_helpers.rs` - Production test utilities
  - **Location**: `beardog/tests/support/concurrent_helpers.rs`
  - **Action**: Songbird copies to `songbird-test-utils/src/concurrent_helpers.rs`
  - **Timeline**: Week 1 (by January 20)

**⏳ In Development** (needed by Week 3):
- Key Derivation API
  ```rust
  POST /api/v1/lineage/derive-key
  {
    "genetic_lineage": "<hash>",
    "epoch": 12345,
    "purpose": "birdsong-encryption"
  }
  Response: {
    "key": "<32-byte-hex>",
    "epoch": 12345,
    "valid_until": 1735086400
  }
  ```
  - **Needed By**: January 27, 2026 (Week 3 start)
  - **Purpose**: BirdSong key rotation support

**✅ Already Exists**:
- Genesis ceremony integration (SoloKey + genetic lineage)
- Lineage verification API (`POST /api/v1/lineage/verify`)
- Encryption/decryption for BirdSong v2

### 2. Songbird Provides to BearDog

**⏳ In Development**:
- BirdSong v3.0 protocol specification
  - **Timeline**: Week 2 (by January 27)
  - **Content**: Multi-tag packet structure, routing metadata schema
  - **Location**: `specs/BIRDSONG_V3_SPECIFICATION.md`

- Tag Management API
  ```rust
  POST /api/v1/birdsong/tags/add
  GET /api/v1/birdsong/tags
  DELETE /api/v1/birdsong/tags/{tag}
  PATCH /api/v1/birdsong/tags/{tag}
  ```
  - **Timeline**: Week 2-3 (by February 3)
  - **Purpose**: Allow BearDog to manage Songbird callsign tags

**📅 Planned**:
- BirdSong v3.0 production release
  - **Timeline**: Week 6 (by February 24)
  - **Content**: Multi-tag support, key rotation, replay protection

### 3. Joint Deliverables

**Week 5 Joint Testing** (February 10-17):
- Multi-tag discovery scenarios (Songbird ↔ Songbird)
- Key rotation integration (Songbird ↔ BearDog)
- NAT traversal with genetic verification
- LiveSpore first-boot simulation
- Cross-primal integration tests

**Weekly Sync Meetings**:
- **Who**: Songbird + BearDog + BiomeOS teams
- **When**: Every Friday, 1 hour
- **Format**: Progress update, blocker discussion, next week planning
- **First Meeting**: January 17, 2026

---

## 📊 INTEGRATION STATUS

### Current Integrations (Production)

**✅ Genetic Lineage Integration** (since Songbird v3.0):
- BearDog generates genetic lineage
- Songbird broadcasts lineage in discovery packets
- Auto-trust for same-lineage peers
- **Status**: Production-stable

**✅ BirdSong v2.0 Encryption** (since Songbird v3.6):
- BearDog provides encryption keys (family seed)
- Songbird encrypts discovery packets (ChaCha20-Poly1305)
- Plaintext `family_id` + encrypted payload
- **Status**: Production-stable

**✅ Identity Attestations** (since Songbird v3.10):
- BearDog signs identity attestations
- Songbird includes in discovery packets
- Cryptographic proof of lineage
- **Status**: Production-stable

### Planned Integrations (LiveSpore Evolution)

**⏳ BirdSong v3.0 Multi-Tag** (Week 2-3):
- Multiple callsign tags per node
- Tag-specific routing metadata
- Institutional NAT support (MSU, university, etc.)
- **Timeline**: February 3, 2026

**⏳ Key Rotation** (Week 3-4):
- BearDog derives keys per epoch
- Songbird rotates keys automatically (30-day interval)
- Overlap period support (old + new keys valid)
- **Timeline**: February 10, 2026

**⏳ Genesis Ceremony Enhancement** (Week 4):
- Songbird adds callsign tag configuration
- BearDog provides genetic lineage generation
- Joint first-boot personalization
- **Timeline**: February 17, 2026

---

## 🔄 API CONTRACTS

### BearDog → Songbird

**Encryption Provider Trait** (current):
```rust
#[async_trait]
pub trait BirdSongCrypto: Send + Sync {
    fn family_id(&self) -> Option<String>;
    async fn encrypt_discovery(&self, plaintext: &[u8]) -> Result<Vec<u8>>;
    async fn decrypt_discovery(&self, ciphertext: &[u8]) -> Result<Vec<u8>>;
    fn is_available(&self) -> bool;
}
```

**Key Derivation API** (planned for Week 3):
```rust
// BearDog endpoint
POST /api/v1/lineage/derive-key
{
  "genetic_lineage": "<hash>",
  "epoch": 12345,
  "purpose": "birdsong-encryption"  // or "birdsong-signing", etc.
}
Response: {
  "key": "<64-char-hex>",  // 32 bytes
  "epoch": 12345,
  "valid_until": 1735086400,
  "algorithm": "hkdf-sha256"
}
```

### Songbird → BearDog

**Tag Management API** (planned for Week 2-3):
```rust
// Songbird IPC endpoint
POST /api/v1/birdsong/tags/add
{
  "tag": "MSU",
  "purpose": "Institutional",
  "priority": 100,
  "routing": {
    "primary_endpoint": "192.168.1.100:8080",
    "nat_config": {
      "traversal_method": "Institutional"
    }
  }
}

GET /api/v1/birdsong/tags
Response: [
  {
    "tag": "MSU",
    "purpose": "Institutional",
    "priority": 100,
    "active": true,
    "created_at": 1735000000
  }
]

DELETE /api/v1/birdsong/tags/{tag}
PATCH /api/v1/birdsong/tags/{tag}
```

---

## 🧪 TEST COORDINATION

### Shared Test Infrastructure

**BearDog's Concurrent Helpers** (Week 1):
- `ReadinessSignal` - Event-driven service startup
- `CompletionWaiter` - Async completion tracking
- `AsyncBarrier` - Coordination primitive
- `unique_unix_socket()` - Test isolation
- `RetryPolicy` - Network polling with backoff

**Songbird Adoption**:
- Copy to `songbird-test-utils/src/concurrent_helpers.rs`
- Replace 86 `sleep` calls in tests
- Achieve 5x test speedup (proven by BearDog)

### Cross-Primal Integration Tests

**Location**: `tests/cross_primal/beardog_integration.rs`

**Current Tests** (production):
- Genetic lineage verification
- BirdSong v2 encryption/decryption
- Identity attestation signing
- Auto-trust for same-lineage peers

**Planned Tests** (Week 5):
- Multi-tag discovery with BearDog
- Key rotation integration
- Genesis ceremony flow
- LiveSpore first-boot simulation

---

## 📈 QUALITY METRICS

### Current Integration Health

| Metric | Status | Notes |
|--------|--------|-------|
| BearDog API Availability | ✅ 100% | Production stable |
| Genetic Lineage Integration | ✅ Working | Auto-trust functional |
| BirdSong v2 Encryption | ✅ Working | ChaCha20-Poly1305 |
| Identity Attestations | ✅ Working | Cryptographic proof |
| Cross-Primal Tests | ✅ Passing | 15/15 tests green |

### LiveSpore Evolution Targets

| Metric | Target | Timeline |
|--------|--------|----------|
| BirdSong v3 Multi-Tag | Complete | Week 2-3 |
| Key Rotation Integration | Complete | Week 3-4 |
| Genesis Enhancement | Complete | Week 4 |
| Joint Testing | 100% Pass | Week 5 |
| Production Release | v3.23.0 | Week 6 |

---

## 🚧 KNOWN ISSUES & BLOCKERS

### Current Issues

**None** - All BearDog integrations working smoothly! ✅

### Future Considerations

**⚠️ Key Derivation API Timeline**:
- BearDog needs to implement by Week 3 (January 27)
- Songbird starts key rotation work in Week 3
- **Mitigation**: Weekly syncs to track progress

**⚠️ Genesis Ceremony Coordination**:
- BiomeOS, BearDog, and Songbird all involved
- Complex multi-primal flow
- **Mitigation**: Joint testing in Week 5

---

## 📅 TIMELINE

### Completed Milestones

- ✅ **v3.0** (Dec 2025): Genetic lineage integration
- ✅ **v3.6** (Dec 2025): BirdSong v2.0 encrypted discovery
- ✅ **v3.10** (Jan 2026): Identity attestations

### LiveSpore Evolution Milestones

- 📅 **Week 1** (Jan 13-20): Concurrent test evolution
- 📅 **Week 2-3** (Jan 20 - Feb 3): BirdSong v3.0 multi-tag
- 📅 **Week 3-4** (Jan 27 - Feb 10): Security hardening
- 📅 **Week 4-5** (Feb 3-17): BiomeOS integration
- 📅 **Week 5** (Feb 10-17): Joint testing
- 📅 **Week 6** (Feb 17-24): Production release

**Final Delivery**: February 24, 2026 (Songbird v3.23.0 with BirdSong v3.0)

---

## 🤝 COMMUNICATION CHANNELS

**Primary**:
- Weekly sync meetings (Fridays, 1 hour)
- Shared documentation in `wateringHole/birdsong/`

**Async**:
- Issues in respective repos
- Design discussions in `wateringHole/`
- Technical specs in `specs/`

**Emergency**:
- Blocker discussion in weekly syncs
- Direct team coordination as needed

---

## 📚 SHARED DOCUMENTATION

**In wateringHole** (`ecoPrimals/wateringHole/birdsong/`):
- `BIRDSONG_PROTOCOL.md` - Current v2.0 spec
- `BIRDSONG_V3_SPECIFICATION.md` - Planned (Week 2)

**In Songbird** (`songbird/specs/`):
- `BIRDSONG_PROTOCOL.md` - Canonical spec
- `GENETIC_LINEAGE_INTEGRATION.md` - BearDog integration guide

**In BearDog** (`beardog/specs/current/`):
- `LIVESPORE_FINAL_ARCHITECTURE.md` - LiveSpore design
- `HOT_PLUG_HSM_UPGRADE_SPECIFICATION.md` - HSM hierarchy

---

## 🎯 SUCCESS CRITERIA

### For LiveSpore Evolution

**BearDog Team Satisfied**:
- ✅ Multi-tag support working (MSU use case)
- ✅ Key rotation integrated
- ✅ Genesis ceremony enhanced
- ✅ Joint tests passing

**Songbird Team Satisfied**:
- ✅ Architecture integrity maintained
- ✅ Backward compatibility (v2 ↔ v3)
- ✅ 90%+ test coverage
- ✅ Production-grade security

**BiomeOS Team Satisfied**:
- ✅ LiveSpore first-boot working
- ✅ NUCLEUS discovery functional
- ✅ Self-replicating deployment ready

**All Teams**:
- ✅ Grade A+ (98/100) for Songbird
- ✅ Cross-primal coordination exemplary
- ✅ Shipped on time (February 24, 2026)

---

**Status**: 🎯 **ACTIVE - ON TRACK**  
**Next Sync**: Friday, January 17, 2026  
**Next Milestone**: Concurrent evolution complete (January 20, 2026)

🐦🐕🌱 **Cross-Primal Excellence in Action!**

