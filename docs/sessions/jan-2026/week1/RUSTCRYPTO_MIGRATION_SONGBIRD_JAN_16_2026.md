# 🦀 Songbird RustCrypto Migration - Jan 16, 2026

**Status**: 🔄 In Progress (Week 2 - BiomeOS Schedule)  
**Strategy**: Concentrated Gap Architecture  
**Goal**: Migrate internal crypto to RustCrypto, keep ring for TLS only

---

## 🎯 **Executive Summary**

### **The Strategy**

**BiomeOS Concentrated Gap Approach**:
- ✅ 4/5 primals → 100% RustCrypto (no TLS needed)
- ⚠️ Songbird → RustCrypto (internal) + ring (TLS only, temporary)

**Songbird's Unique Role**:
- External communication primal (federation, gaming platforms)
- Needs TLS for HTTP/WebSocket endpoints
- All other primals use Unix sockets (no TLS)

**Migration Path**:
1. ✅ **NOW**: Add RustCrypto dependencies
2. ⏳ **Week 2**: Migrate internal crypto (BTSP, BirdSong, auth)
3. ⏳ **Q2 2026**: Test rustls RustCrypto provider (beta)
4. ⏳ **Q3-Q4 2026**: Remove ring, achieve 100% pure Rust

---

## 📊 **Current Status**

### **✅ Completed**

1. **cmake Eliminated**
   - Switched from aws-lc-rs to ring
   - Zero external build dependencies
   - Cross-compilation works

2. **BiomeOS Debt Resolved**
   - Socket path issues fixed
   - 35 tests passing (100%)
   - Production-ready

3. **RustCrypto Dependencies Added**
   - aes-gcm 0.10 (encryption)
   - ed25519-dalek 2.1 (signatures)
   - x25519-dalek 2.0 (key exchange)
   - sha2 0.10 (hashing)
   - hmac 0.12 (authentication)
   - argon2 0.5 (key derivation)
   - chacha20poly1305 0.10 (alternative encryption)

### **⏳ In Progress**

4. **Internal Crypto Migration**
   - BTSP tunnels (pending)
   - BirdSong protocol (pending)
   - Auth operations (pending)
   - Testing (pending)

---

## 🎯 **Migration Scope**

### **What to Migrate (RustCrypto)**

#### **1. BTSP Tunnels** (Internal Communication)

**Current** (ring-based):
```rust
// BTSP currently uses ring for:
// - AES-GCM encryption
// - X25519 key exchange
// - Ed25519 authentication
```

**Target** (RustCrypto):
```rust
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::Aead;
use x25519_dalek::{EphemeralSecret, PublicKey};
use ed25519_dalek::{Signer, Verifier, Signature};
use sha2::Sha256;
use hmac::{Hmac, Mac};
```

**Impact**:
- Better performance (RustCrypto often faster)
- Audited by NCC Group
- Pure Rust (no vendored C)
- Active maintenance

---

#### **2. BirdSong Protocol** (Federation)

**Current** (ring-based):
```rust
// BirdSong currently uses ring for:
// - Ed25519 signatures
// - SHA-256 hashing
// - HMAC authentication
```

**Target** (RustCrypto):
```rust
use ed25519_dalek::{Keypair, Signer, Verifier};
use sha2::{Sha256, Digest};
use hmac::{Hmac, Mac};
type HmacSha256 = Hmac<Sha256>;
```

**Impact**:
- More ergonomic API
- Better Rust idioms
- Consistent with ecosystem

---

#### **3. Authentication & JWT**

**Current** (jwt-simple, already pure Rust!):
```rust
// jwt-simple is ALREADY pure Rust ✅
// No changes needed!
```

**Additional** (for password hashing):
```rust
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, SaltString},
    Argon2,
};
```

**Impact**:
- Industry-standard password hashing
- Memory-hard (resistant to brute force)
- Audited implementation

---

### **What to KEEP (ring, Temporary)**

#### **4. TLS / HTTPS** (External Communication)

**Current** (ring for TLS):
```rust
// Keep for TLS only:
rustls = { version = "0.23", features = ["ring"] }
reqwest = { version = "0.11", features = ["rustls-tls"] }
```

**Rationale**:
- rustls RustCrypto provider not yet released
- Must wait for Q3-Q4 2026
- ring is TEMPORARY stepping stone (unmaintained!)

**Future** (Q3-Q4 2026):
```rust
// When rustls RustCrypto provider is ready:
rustls = { version = "0.2x", features = ["rustcrypto"] }
```

---

## 📋 **Migration Tasks**

### **Phase 1: Dependencies** ✅ COMPLETE

- [x] Add aes-gcm dependency
- [x] Add ed25519-dalek dependency
- [x] Add x25519-dalek dependency
- [x] Add sha2 dependency
- [x] Add hmac dependency
- [x] Add argon2 dependency
- [x] Add chacha20poly1305 dependency
- [x] Update Cargo.toml comments

---

### **Phase 2: BTSP Tunnels** ⏳ NEXT

**Files to Update**:
- `crates/songbird-network-federation/src/btsp/`
- `crates/songbird-network-federation/src/btsp/tunnel.rs`
- `crates/songbird-network-federation/src/btsp/crypto.rs`

**Migration Steps**:
1. [ ] Replace ring AES-GCM with RustCrypto aes-gcm
2. [ ] Replace ring X25519 with RustCrypto x25519-dalek
3. [ ] Replace ring Ed25519 with RustCrypto ed25519-dalek
4. [ ] Update key generation
5. [ ] Update encryption/decryption
6. [ ] Update authentication
7. [ ] Add unit tests
8. [ ] Add integration tests
9. [ ] Performance benchmark (ensure no regression)

**Success Criteria**:
- [ ] All BTSP tests pass
- [ ] No performance regression
- [ ] No ring dependency for BTSP
- [ ] Code review complete

---

### **Phase 3: BirdSong Protocol** ⏳ PENDING

**Files to Update**:
- `crates/songbird-network-federation/src/birdsong/`
- `crates/songbird-network-federation/src/birdsong/protocol.rs`
- `crates/songbird-network-federation/src/birdsong/crypto.rs`

**Migration Steps**:
1. [ ] Replace ring Ed25519 with RustCrypto ed25519-dalek
2. [ ] Replace ring SHA-256 with RustCrypto sha2
3. [ ] Replace ring HMAC with RustCrypto hmac
4. [ ] Update signature generation/verification
5. [ ] Update hashing operations
6. [ ] Update HMAC operations
7. [ ] Add unit tests
8. [ ] Add E2E tests
9. [ ] Performance benchmark

**Success Criteria**:
- [ ] All BirdSong tests pass
- [ ] Federation still works
- [ ] No performance regression
- [ ] Code review complete

---

### **Phase 4: Auth Operations** ⏳ PENDING

**Files to Update**:
- `crates/songbird-orchestrator/src/access_control/`
- Password hashing (if applicable)
- API key generation

**Migration Steps**:
1. [ ] Add Argon2 password hashing (if needed)
2. [ ] Update any ring-based auth operations
3. [ ] Add unit tests
4. [ ] Security review

**Success Criteria**:
- [ ] All auth tests pass
- [ ] jwt-simple remains unchanged (already pure Rust)
- [ ] Argon2 for password hashing (if applicable)
- [ ] Security review complete

---

### **Phase 5: Testing & Verification** ⏳ PENDING

**Testing Strategy**:

1. **Unit Tests**
   - [ ] Test each crypto operation
   - [ ] Test key generation
   - [ ] Test encryption/decryption
   - [ ] Test signing/verification
   - [ ] Test hashing
   - [ ] Test HMAC

2. **Integration Tests**
   - [ ] BTSP tunnel establishment
   - [ ] BirdSong federation
   - [ ] End-to-end encryption
   - [ ] Cross-primal communication

3. **Performance Benchmarks**
   - [ ] Encryption/decryption throughput
   - [ ] Signing/verification speed
   - [ ] Hashing performance
   - [ ] Compare to ring baseline

4. **Security Validation**
   - [ ] Crypto primitives correct
   - [ ] No algorithm downgrade
   - [ ] Key sizes appropriate
   - [ ] Authentication working

**Success Criteria**:
- [ ] All tests pass (unit, integration, E2E)
- [ ] Performance within 5% of ring baseline
- [ ] Security review approved
- [ ] No regressions

---

### **Phase 6: Documentation** ⏳ PENDING

**Documentation Updates**:
1. [ ] Update crypto architecture docs
2. [ ] Document RustCrypto migration
3. [ ] Document TLS gap strategy
4. [ ] Update security documentation
5. [ ] Create migration guide for other primals
6. [ ] Share learnings in wateringHole/

**Success Criteria**:
- [ ] All docs up-to-date
- [ ] Clear evolution path documented
- [ ] Other primals can reference

---

## 🔒 **Security Considerations**

### **Why RustCrypto?**

**All RustCrypto Crates are Audited**:
- ✅ **AES-GCM**: NCC Group audit (2019, 2020)
- ✅ **ChaCha20-Poly1305**: NCC Group audit (2019)
- ✅ **Ed25519**: Multiple audits, widely used
- ✅ **SHA-2**: Audited, FIPS compliant
- ✅ **HMAC**: Audited, standard implementation
- ✅ **Argon2**: Audited, password hashing competition winner

**Higher Confidence**:
- Pure Rust implementations
- No C dependencies (memory safe!)
- Active maintenance
- Used by industry (1Password, Signal, etc.)

---

### **Why ring is Temporary**

**ring is Unmaintained** (BiomeOS Warning):
- Last significant update: 2021
- Creator (Brian Smith) stepped back
- No active development
- Must migrate anyway!

**ring as Stepping Stone**:
- ✅ Eliminates cmake (aws-lc-rs)
- ✅ Self-contained build
- ⚠️ But NOT long-term solution
- 🎯 Temporary for TLS only

**Final Evolution** (Q3-Q4 2026):
- rustls RustCrypto provider releases
- Migrate TLS to RustCrypto
- Remove ring completely
- 100% pure Rust achieved!

---

## 📊 **Success Metrics**

### **Per-Phase Verification**

**After Phase 1** (Dependencies):
```bash
cargo tree | grep -i "aes-gcm\|ed25519\|sha2"
# Should show RustCrypto crates!
```

**After Phase 2** (BTSP):
```bash
# No ring in BTSP code
grep -r "ring::" crates/songbird-network-federation/src/btsp/
# Should be EMPTY

# RustCrypto in BTSP
grep -r "aes_gcm\|ed25519_dalek" crates/songbird-network-federation/src/btsp/
# Should have matches!
```

**After Phase 3** (BirdSong):
```bash
# No ring in BirdSong code
grep -r "ring::" crates/songbird-network-federation/src/birdsong/
# Should be EMPTY

# RustCrypto in BirdSong
grep -r "ed25519_dalek\|sha2" crates/songbird-network-federation/src/birdsong/
# Should have matches!
```

**Final Verification**:
```bash
# ring ONLY in TLS code
cargo tree | grep ring
# Should ONLY show rustls → ring

# No ring in application code
grep -r "use ring::" crates/*/src/ --include="*.rs"
# Should be EMPTY (except TLS init)
```

---

## 📅 **Timeline**

### **Week 2 (Jan 24-30, 2026)** - BiomeOS Schedule

**Monday-Tuesday**: BTSP Migration
- Add RustCrypto dependencies ✅ DONE
- Migrate BTSP tunnels
- Unit tests
- Integration tests

**Wednesday**: BirdSong Migration
- Migrate BirdSong protocol
- Federation tests
- E2E tests

**Thursday**: Auth & Testing
- Auth operations (if needed)
- Performance benchmarks
- Security validation

**Friday**: Documentation & Handoff
- Update documentation
- Share results with BiomeOS
- Post to wateringHole/

---

### **Q2 2026 (Apr-Jun)**: TLS Provider Testing

**Monitor rustls Development**:
- [ ] Track rustls RustCrypto provider
- [ ] Test beta releases
- [ ] Report bugs
- [ ] Validate TLS 1.2 and 1.3

---

### **Q3-Q4 2026 (Jul-Dec)**: Final Evolution

**Remove ring Completely**:
- [ ] Migrate to rustls RustCrypto provider
- [ ] Remove ring dependency
- [ ] Verify 100% pure Rust
- [ ] Celebrate sovereignty! 🎉

---

## 💪 **Why This Matters**

### **Concentrated Gap Benefits**

**Security**:
- ✅ Only Songbird has HTTP client
- ✅ Other primals can't leak via HTTP
- ✅ Clear security boundary
- ✅ Single point of TLS evolution

**Simplicity**:
- ✅ 4/5 primals don't need TLS
- ✅ Clear architecture
- ✅ Easier to reason about
- ✅ Easier to audit

**Evolution**:
- ✅ Single migration point
- ✅ Other primals already done
- ✅ Clear timeline
- ✅ Measurable progress

---

### **Pure Rust Unlocks**

**Cross-Compilation**:
```bash
# ARM (no cmake!)
cargo build --release --target aarch64-unknown-linux-gnu

# RISC-V
cargo build --release --target riscv64gc-unknown-linux-gnu

# WebAssembly
cargo build --release --target wasm32-unknown-unknown
```

**One command, any target!**

---

## 🎊 **Conclusion**

### **Current State**

✅ **Dependencies Added**: RustCrypto crates in Cargo.toml  
✅ **Build Works**: Zero external dependencies  
✅ **Tests Pass**: BiomeOS integration (35 tests)  
⏳ **Migration Pending**: BTSP, BirdSong, auth

### **Next Steps**

1. **Immediate**: Migrate BTSP tunnels to RustCrypto
2. **This Week**: Complete BirdSong migration
3. **Q2 2026**: Test rustls RustCrypto provider
4. **Q3-Q4 2026**: Achieve 100% pure Rust

### **Philosophy Alignment**

✅ **Deep Debt Solution**: Not just eliminating cmake, evolving ALL crypto  
✅ **BiomeOS Strategy**: Concentrated gap, clear architecture  
✅ **TRUE PRIMAL**: Zero external dependencies, complete sovereignty  
✅ **User-Driven**: "Why can't we evolve it?" → We did!

---

**Status**: 📚 Phase 1 Complete, Phase 2 Ready  
**Timeline**: Week 2 for internal crypto, Q3-Q4 2026 for TLS  
**Impact**: Clear path to 100% pure Rust! 🦀✨

---

**Created**: January 16, 2026  
**Strategy**: BiomeOS Concentrated Gap Architecture  
**Next**: Migrate BTSP tunnels to RustCrypto AES-GCM

