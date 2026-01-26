# 🐻 BearDog SHA-384 Handoff - January 26, 2026

**Status**: ✅ **BEARDOG READY - SONGBIRD IMPLEMENTATION NEEDED**  
**Impact**: 95% → 100% TLS Validation  
**Upstream Doc**: Received from biomeOS

---

## 🎉 BEARDOG SHA-384 EVOLUTION COMPLETE!

**Tested Jan 26, 2026** - BearDog's new method returns 48-byte SHA-384 hashes:

```json
// Request
{
  "method": "crypto.hash_for_cipher",
  "params": {
    "data": "dGVzdA==",
    "cipher_suite": 4866
  }
}

// Response ✅ WORKING!
{
  "result": {
    "algorithm": "SHA-384",
    "cipher_suite": 4866,
    "hash": "doQSMg97CqWBL85CjcRwazyuUOAqZMqhangiSb/o78S37xzLEmJV0ZYEff7fF6Cp",
    "hash_length": 48
  }
}
```

---

## 🎵 SONGBIRD TASKS (READY TO IMPLEMENT NOW)

### P0: Update Transcript Hashing (PRIMARY FIX)

**File**: `crates/songbird-http-client/src/tls/handshake_refactored/transcript.rs`

```rust
// CURRENT (line 276-278) - HARDCODED SHA-256
pub(super) fn compute_transcript_hash(&self) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(&self.transcript);
    let hash = hasher.finalize().to_vec();
    // Returns 32 bytes ALWAYS
}

// REQUIRED - Cipher-aware hashing via BearDog
pub(super) async fn compute_transcript_hash(&self, cipher_suite: u16) -> Result<Vec<u8>> {
    // Use BearDog's crypto.hash_for_cipher for cipher-aware hashing
    let hash = self.crypto.hash_for_cipher(&self.transcript, cipher_suite).await?;
    // Returns 32 bytes for 0x1301/0x1303, 48 bytes for 0x1302
    Ok(hash)
}
```

### P0: Add `hash_for_cipher` to CryptoCapability Trait

**File**: `crates/songbird-http-client/src/crypto/capability.rs`

```rust
#[async_trait]
pub trait CryptoCapability: Send + Sync {
    // ... existing methods ...
    
    /// Cipher-suite aware hashing (BearDog selects SHA-256 or SHA-384)
    async fn hash_for_cipher(&self, data: &[u8], cipher_suite: u16) -> Result<Vec<u8>>;
}
```

### P0: Implement in BearDogProvider

**File**: `crates/songbird-http-client/src/crypto/beardog_provider.rs`

```rust
async fn hash_for_cipher(&self, data: &[u8], cipher_suite: u16) -> Result<Vec<u8>> {
    let params = json!({
        "data": BASE64_STANDARD.encode(data),
        "cipher_suite": cipher_suite
    });
    
    let result = self.call("crypto.hash_for_cipher", Some(params)).await?;
    self.extract_b64_field(&result, "hash")
}
```

---

## 📁 Files to Modify

| File | Change |
|------|--------|
| `crypto/capability.rs` | Add `hash_for_cipher` to trait |
| `crypto/beardog_provider.rs` | Implement `hash_for_cipher` |
| `tls/handshake_refactored/transcript.rs` | Use cipher-aware hashing |
| `tls/handshake_refactored/handshake_flow.rs` | Pass `cipher_suite` to transcript hash |

---

## 🧪 Test After Implementation

```bash
# Sites that use cipher 0x1302 (TLS_AES_256_GCM_SHA384)
# These currently fail with "transcript_hash must be 48 bytes"

# NCBI
echo '{"jsonrpc":"2.0","method":"http.request","params":{"method":"GET","url":"https://www.ncbi.nlm.nih.gov","headers":{}},"id":1}' | nc -U /tmp/songbird-nat0.sock

# Azure
echo '{"jsonrpc":"2.0","method":"http.request","params":{"method":"GET","url":"https://azure.microsoft.com","headers":{}},"id":1}' | nc -U /tmp/songbird-nat0.sock
```

---

## 🎯 Success Criteria

| Test | Current | Target |
|------|---------|--------|
| TLS validation | 85% | 100% |
| 0x1301 (AES-128-GCM-SHA256) | ✅ | ✅ |
| 0x1302 (AES-256-GCM-SHA384) | ❌ 48-byte hash needed | ✅ |
| 0x1303 (ChaCha20-Poly1305-SHA256) | ✅ | ✅ |

---

## ⏱️ Estimated Effort

| Task | Time |
|------|------|
| Add trait method | 5 min |
| Implement provider | 10 min |
| Update transcript module | 30 min |
| Update handshake flow | 20 min |
| Testing | 30 min |
| **Total** | **~2 hours** |

---

## 📞 BearDog API Reference

**Validated Capabilities** (Jan 26, 2026):

| Method | Status |
|--------|--------|
| `crypto.hash_for_cipher` | ✅ Returns 48 bytes for 0x1302 |
| `tls.derive_handshake_secrets` | ✅ Uses cipher-aware HKDF |
| `tls.derive_application_secrets` | ✅ Uses cipher-aware HKDF |

---

**Created**: January 26, 2026  
**Updated**: January 26, 2026 - BearDog evolution confirmed!  
**BearDog Status**: ✅ COMPLETE  
**Songbird Status**: 🔧 IMPLEMENTATION NEEDED
