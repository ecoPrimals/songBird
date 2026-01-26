# 🐻 BearDog SHA-384 Handoff - January 26, 2026

**Status**: ⏳ Waiting for BearDog Evolution  
**Impact**: 95% → 100% TLS Validation  
**Upstream Doc**: Received from biomeOS

---

## 🎯 Summary

Songbird has achieved **95% TLS 1.3 validation** (20/21 endpoints).
The remaining 5% requires **SHA-384 support** from BearDog for cipher suite 0x1302.

---

## 📊 The Problem

**Cipher Suite 0x1302** (TLS_AES_256_GCM_SHA384) requires SHA-384 for:
- Transcript hashing
- HKDF key derivation
- HMAC operations

**Current**: Both BearDog and Songbird are hardcoded to SHA-256.

---

## 🔧 BearDog Tasks Required

### P0: Add `crypto.hash_for_cipher` Method

```json
// Request
{
  "method": "crypto.hash_for_cipher",
  "params": {
    "data": "base64-encoded-data",
    "cipher_suite": 4866  // 0x1302
  }
}

// Response (48-byte SHA-384 hash)
{
  "result": {
    "hash": "base64-encoded-48-byte-sha384-hash"
  }
}
```

### P0: Update `tls.derive_handshake_secrets`

Select HKDF based on cipher_suite:
- 0x1301, 0x1303 → HKDF-SHA256
- 0x1302 → HKDF-SHA384

### P0: Update `tls.derive_application_secrets`

Same pattern - cipher-aware HKDF selection.

---

## 🎵 Songbird Tasks (After BearDog)

### P1: Update Transcript Hashing

```rust
// Change from:
let hash = Sha256::digest(&transcript);

// To:
let hash = crypto.hash_for_cipher(&transcript, cipher_suite).await?;
```

### P1: Pass cipher_suite to Derivation Calls

Already passing cipher_suite in params, just need BearDog to use it.

---

## ✅ Songbird Ready State

| Component | Status |
|-----------|--------|
| cipher_suite passed in params | ✅ Ready |
| transcript data available | ✅ Ready |
| capability.call integration | ✅ Ready |
| Error handling | ✅ Ready |

---

## 📁 Files to Modify (After BearDog)

**Songbird**:
- `crates/songbird-http-client/src/tls/handshake_refactored/transcript.rs`
- `crates/songbird-http-client/src/crypto/capability.rs`
- `crates/songbird-http-client/src/crypto/beardog_provider.rs`

---

## 🎯 Success Criteria

| Test | Current | Target |
|------|---------|--------|
| TLS validation | 95% | 100% |
| 0x1301 (AES-128-GCM) | ✅ | ✅ |
| 0x1302 (AES-256-GCM) | ❌ | ✅ |
| 0x1303 (ChaCha20) | ✅ | ✅ |

---

## 📞 Coordination

- **BearDog Path**: `/home/eastgate/Development/ecoPrimals/phase1/beardog`
- **biomeOS Graph**: `tower_atomic_bootstrap.toml`
- **Standards**: `/home/eastgate/Development/ecoPrimals/wateringHole/`

---

**Created**: January 26, 2026  
**Waiting On**: BearDog SHA-384 evolution  
**Songbird Status**: ✅ Ready to integrate when BearDog complete

