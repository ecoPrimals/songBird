# 🔍 BearDog Crypto API Alignment Analysis

**Date**: January 18, 2026  
**Status**: 🔄 API Contract Analysis  
**Purpose**: Align Songbird client with BearDog's actual API

---

## 🎯 Problem Statement

**Current Status**: 1/5 tests passing (Blake3 hash ✅)

**Issue**: Our Songbird client was designed with assumptions about BearDog's API that don't match the actual implementation.

**Root Cause**: We designed the client based on our spec, but BearDog implemented a slightly different (and better!) API.

---

## 📊 API Contract Analysis

### ✅ **Working: Blake3 Hash**

**BearDog Returns**:
```json
{
  "hash": "base64_encoded_hash",
  "algorithm": "BLAKE3"
}
```

**Songbird Expects**: ✅ MATCHES!

---

### ❌ **Broken: Ed25519 Sign**

**BearDog Returns**:
```json
{
  "signature": "base64_encoded_signature",
  "algorithm": "Ed25519",
  "key_id": "key_identifier"
}
```

**Songbird Expects**: Likely matches, need to check error details

---

### ❌ **Broken: X25519 Generate Ephemeral**

**BearDog Returns**:
```json
{
  "public_key": "base64_encoded_public_key",
  "secret_key": "base64_encoded_secret_key",
  "algorithm": "X25519"
}
```

**Songbird Expects**:
```rust
(Vec<u8>, String)  // (public_key, secret_key_id)
```

**MISMATCH**: BearDog returns the actual `secret_key` bytes, not an ID!

**Why This is Better**: Stateless! BearDog doesn't need to store ephemeral keys. The caller keeps the secret key.

---

### ❌ **Broken: X25519 Derive Secret**

**BearDog Expects**:
```json
{
  "our_secret": "base64_encoded_secret_key",
  "their_public": "base64_encoded_public_key"
}
```

**Songbird Sends**: Likely expects `secret_key_id` instead of `our_secret`

**MISMATCH**: We need to send the actual secret key bytes, not an ID!

---

### ❌ **Broken: ChaCha20-Poly1305 Encrypt**

**BearDog Returns**:
```json
{
  "ciphertext": "base64",
  "nonce": "base64",
  "tag": "base64",
  "algorithm": "ChaCha20-Poly1305"
}
```

**Songbird Expects**: Need to verify structure

---

### ❌ **Broken: HMAC-SHA256**

**BearDog Returns**:
```json
{
  "mac": "base64",
  "algorithm": "HMAC-SHA256"
}
```

**Songbird Expects**: Likely expects `hmac` instead of `mac`

**MISMATCH**: Field name difference

---

## 🎯 Fixes Required

### 1. **X25519 Generate Ephemeral**

**Change Return Type**:
```rust
// OLD:
pub async fn x25519_generate_ephemeral(
    socket_path: &str,
    purpose: &str,
) -> Result<(Vec<u8>, String)>  // (public_key, secret_key_id)

// NEW:
pub async fn x25519_generate_ephemeral(
    socket_path: &str,
    purpose: &str,
) -> Result<(Vec<u8>, Vec<u8>)>  // (public_key, secret_key)
```

**Why**: BearDog returns actual keys, not IDs (stateless design!)

---

### 2. **X25519 Derive Secret**

**Change Parameters**:
```rust
// OLD:
pub async fn x25519_derive_secret(
    socket_path: &str,
    secret_key_id: &str,  // ID!
    their_public_key: &[u8],
) -> Result<Vec<u8>>

// NEW:
pub async fn x25519_derive_secret(
    socket_path: &str,
    our_secret_key: &[u8],  // Actual bytes!
    their_public_key: &[u8],
) -> Result<Vec<u8>>
```

**Why**: BearDog doesn't store keys, expects actual bytes

---

### 3. **HMAC-SHA256**

**Change Response Field**:
```rust
// OLD response parsing:
let hmac = response.result.hmac;  // Wrong field!

// NEW response parsing:
let mac = response.result.mac;  // Correct field!
```

**Why**: BearDog uses `mac` field name, not `hmac`

---

### 4. **ChaCha20-Poly1305**

**Verify Return Structure**: Need to check if `tag` is separate or included in `ciphertext`

---

## 💡 Key Insight: BearDog's Design is BETTER!

**Our Original Design**: Key IDs (stateful, BearDog stores keys)
- ❌ Requires BearDog to manage ephemeral key lifecycle
- ❌ Requires secure key storage in BearDog
- ❌ Requires key cleanup/expiration
- ❌ More complex state management

**BearDog's Actual Design**: Direct key bytes (stateless!)
- ✅ BearDog is stateless for ephemeral keys
- ✅ Caller manages their own keys
- ✅ No key storage/cleanup needed
- ✅ Simpler, more secure
- ✅ Better performance

**Result**: BearDog's API is more elegant! We should align to it.

---

## 🎯 Action Plan

1. ✅ **Blake3**: Already working!
2. 🔄 **X25519 Generate**: Change return type to `(Vec<u8>, Vec<u8>)`
3. 🔄 **X25519 Derive**: Change param from `secret_key_id: &str` to `our_secret_key: &[u8]`
4. 🔄 **HMAC**: Change response field from `hmac` to `mac`
5. 🔄 **Ed25519**: Verify and fix if needed
6. 🔄 **ChaCha20**: Verify structure and fix if needed

---

## 🏆 Expected Outcome

**After Alignment**: 5/5 tests passing!

**Quality**: Client matches BearDog's actual (superior!) API design

**Next Step**: Evolve to capability-based discovery (not hardcoded BearDog)

---

**Status**: Ready to execute API alignment fixes!

🦀🐻🐕✨ Align | Test | Evolve to Capability-Based! ✨🐕🐻🦀

