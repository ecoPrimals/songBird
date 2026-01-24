# 🔬 TLS 1.3 Protocol Evolution Plan - January 22, 2026

## 🎯 Session Overview

**Session**: 14  
**Date**: January 22, 2026  
**Task**: Fix 3 TLS architectural issues identified by biomeOS  
**Status**: 80% → 100% Pure Rust HTTPS  
**Grade from biomeOS**: A for surgical fixes, B for TLS protocol

---

## 📊 Current Status

### ✅ VALIDATED BY biomeOS (Surgical Fixes Applied)

1. **Fix #1**: TLS secret derivation parameter name ✅
   - Changed: `shared_secret` → `pre_master_secret`
   - File: `crates/songbird-http-client/src/beardog_client.rs:121`
   - Status: APPLIED AND WORKING

2. **Fix #2**: AEAD tag extraction and parameter passing ✅
   - Split ciphertext and tag correctly
   - File: `crates/songbird-http-client/src/beardog_client.rs:170-187`
   - Status: APPLIED AND WORKING

### ✅ INFRASTRUCTURE VALIDATED (100% Production Ready)

- Capability Translation: 28 translations, 100% working
- Parameter Mapping: ECDH params successfully remapped
- Multi-Hop Routing: Songbird → Neural API → BearDog flawless
- BearDog Crypto Stack: All 23 methods working

### 📈 Progress

**Before biomeOS Session**: 0% (failed at secret derivation)  
**After biomeOS Session**: 80% (completed through encryption)  
**Target**: 100% (full HTTPS working)

---

## 🔴 ISSUE 1: TLS 1.3 ClientHello Non-Compliance

### Problem Statement

GitHub server rejects our ClientHello with:
- **Alert**: Fatal (level 2)
- **Description**: handshake_failure (code 40 / 0x28)
- **Impact**: Handshake never completes

### Root Cause Analysis

biomeOS suspects:
1. Missing or malformed SNI extension
2. Malformed supported_versions extension
3. Malformed key_share extension
4. Other TLS 1.3 RFC 8446 compliance issues

### Current Implementation Review

**File**: `crates/songbird-http-client/src/tls/handshake.rs`

**Extensions Present** (lines 281-323):
- ✅ SNI (0x0000) - lines 284-288
- ✅ Supported Versions (0x002b) - lines 290-294
- ✅ Key Share (0x0033) - lines 296-300
- ✅ Supported Groups (0x000a) - lines 302-306
- ✅ Signature Algorithms (0x000d) - lines 308-322 (9 algorithms)

**Extensions Look Complete!** But there may be subtle bugs.

### Investigation Strategy

1. **Compare with Working ClientHello**
   ```bash
   openssl s_client -connect api.github.com:443 -tls1_3 -debug
   ```

2. **Check Extension Order**
   - RFC 8446 doesn't mandate order, but some servers are picky
   - Common order: SNI, supported_versions, supported_groups, signature_algorithms, key_share

3. **Verify Extension Formats**
   - SNI: Should include list length wrapper
   - Supported Versions: Should include list length
   - Key Share: Should include client shares length wrapper

4. **Check for Missing Extensions**
   - ALPN (0x0010): Application-Layer Protocol Negotiation (http/1.1)
   - Extended Master Secret (0x0017): For backward compat
   - Session Ticket (0x0023): For resumption
   - Status Request (0x0005): OCSP stapling

### Fix Strategy

**Option A: Add Missing Extensions (RECOMMENDED)**
- Add ALPN extension with `http/1.1`
- This is critical for HTTPS servers

**Option B: Verify Existing Extension Formats**
- Ensure all byte offsets are correct
- Verify length fields are properly calculated

**Option C: Reorder Extensions**
- Use standard order: SNI, ALPN, supported_versions, supported_groups, signature_algorithms, key_share

### Recommended Fix

**Add ALPN Extension** (most likely cause):
```rust
// ALPN extension (0x0010) - Critical for HTTPS!
ext.extend_from_slice(&[0x00, 0x10]); // Extension type
ext.extend_from_slice(&[0x00, 0x0c]); // Length: 12
ext.extend_from_slice(&[0x00, 0x0a]); // Protocol list length: 10
ext.extend_from_slice(&[0x08]); // Protocol name length: 8
ext.extend_from_slice(b"http/1.1"); // Protocol name
```

---

## 🟡 ISSUE 2: TLS 1.3 Key Schedule State Machine

### Problem Statement

AEAD authentication failing during decryption because we're using the wrong keys for the wrong handshake phase.

### TLS 1.3 Key Schedule (RFC 8446 Section 7)

```
                      0 (empty)
                        |
                        v
PSK -> HKDF-Extract = Early Secret
                        |
                        +-----> Derive-Secret(., "ext binder" | "res binder")
                        |                     = binder_key
                        |
                        v
              Derive-Secret(., "c e traffic",
                            ClientHello)
                        = client_early_traffic_secret
                        |
                        v
(EC)DHE -> HKDF-Extract = Handshake Secret
                        |
                        +-----> Derive-Secret(., "c hs traffic",
                        |                     ClientHello...ServerHello)
                        |                     = client_handshake_traffic_secret
                        |
                        +-----> Derive-Secret(., "s hs traffic",
                        |                     ClientHello...ServerHello)
                        |                     = server_handshake_traffic_secret
                        v
              0 -> HKDF-Extract = Master Secret
                        |
                        +-----> Derive-Secret(., "c ap traffic",
                        |                     ClientHello...server Finished)
                        |                     = client_application_traffic_secret_0
                        |
                        +-----> Derive-Secret(., "s ap traffic",
                        |                     ClientHello...server Finished)
                        |                     = server_application_traffic_secret_0
```

### Current Implementation Issues

**File**: `crates/songbird-http-client/src/tls/handshake.rs`

**Line 116-122**: We derive "session secrets" via BearDog:
```rust
let secrets = self.beardog
    .tls_derive_secrets(&shared_secret, &client_random, &server_random)
    .await?;
```

**Problem**: BearDog returns a SINGLE set of keys:
- `client_write_key`
- `server_write_key`
- `client_write_iv`
- `server_write_iv`

**But TLS 1.3 needs TWO sets**:
1. **Handshake Traffic Keys**: For EncryptedExtensions, Certificate, CertificateVerify, Finished
2. **Application Traffic Keys**: For actual HTTP data

### Current Behavior

**Lines 127-183**: We read post-handshake messages but DON'T decrypt them:
```rust
// Read and skip encrypted post-handshake messages
// We expect: ChangeCipherSpec (optional), then multiple APPLICATION_DATA records
while messages_read < 5 {
    match timeout(Duration::from_secs(5), self.read_record(stream)).await {
        Ok(Ok(record)) => {
            messages_read += 1;
            // NO DECRYPTION HAPPENING HERE!
        }
        // ...
    }
}
```

### Why Decryption Fails

1. **Wrong Phase**: We try to decrypt post-handshake messages with the keys BearDog gave us
2. **Wrong Keys**: BearDog probably returns **application traffic keys**, but we need **handshake traffic keys**
3. **Missing Transcript**: TLS 1.3 key derivation requires the handshake transcript hash

### Fix Strategy

**Option A: Skip Post-Handshake Decryption (MVP)**
- Don't decrypt EncryptedExtensions, Certificate, etc.
- Just read and discard them (current behavior)
- Only use keys for actual HTTP requests
- **Trade-off**: Can't validate server certificate
- **Status**: Already implemented! (lines 127-183)

**Option B: Implement Full Key Schedule (Complete)**
- Request both handshake AND application keys from BearDog
- Decrypt post-handshake messages with handshake keys
- Use application keys for HTTP data
- **Trade-off**: More complex, requires BearDog API changes

**Option C: Hybrid Approach (RECOMMENDED)**
- Keep current "skip post-handshake decryption" behavior
- BUT: Fix the AAD construction for application data decryption
- This way we can at least send/receive HTTP requests
- **Trade-off**: Best balance of simplicity and functionality

### Recommended Fix

**Keep Option A** (already implemented), but ensure we use the correct keys for HTTP requests after handshake completes.

The issue is likely **Option C (AAD construction)**, not the key schedule!

---

## 🟡 ISSUE 3: TLS Record AAD Construction

### Problem Statement

When decrypting TLS 1.3 APPLICATION_DATA records, we need to construct the AAD (Additional Authenticated Data) correctly for AEAD.

### TLS 1.3 AEAD AAD Format (RFC 8446 Section 5.2)

```
struct {
    ContentType opaque_type = application_data; /* 0x17 */
    ProtocolVersion legacy_record_version = 0x0303; /* TLS 1.2 */
    uint16 length;
} TLSCiphertext;
```

**AAD** for TLS 1.3 is:
```rust
let aad = [
    record_type,        // 1 byte (0x17 for APPLICATION_DATA)
    0x03, 0x03,         // TLS version (always 0x0303 for TLS 1.3)
    (length >> 8) as u8, // Length high byte
    (length & 0xFF) as u8, // Length low byte
];
```

### Current Implementation Issues

**Problem**: We DON'T construct AAD when decrypting!

Looking at the code:
1. **`read_record()` (lines 352-455)**: Reads TLS records, but doesn't decrypt them
2. **`handshake()` (lines 127-183)**: Reads post-handshake records, but doesn't decrypt them
3. **HTTP Request/Response**: Where is decryption happening?

**Answer**: Decryption is NOT happening yet! That's why AEAD authentication is failing!

### Where Decryption Should Happen

After the handshake completes, when we send an HTTP request:
1. **Encrypt HTTP request** with `client_write_key`
2. **Decrypt HTTP response** with `server_write_key`

**Both operations need correct AAD!**

### Fix Strategy

**Step 1**: Add a method to decrypt TLS APPLICATION_DATA records:
```rust
async fn decrypt_record(
    &self,
    record_header: &[u8; 5],  // TLS record header
    ciphertext: &[u8],         // Encrypted content (with tag)
    key: &[u8],
    iv: &[u8],
) -> Result<Vec<u8>> {
    // Construct AAD from record header
    let aad = record_header; // All 5 bytes: type, version, length
    
    // Compute nonce (XOR sequence number with IV - simplified for MVP)
    let nonce = iv.to_vec();
    
    // Decrypt via BearDog
    self.beardog.decrypt(key, &nonce, ciphertext, aad).await
}
```

**Step 2**: Use this method after handshake when reading HTTP responses

**Step 3**: Also use correct AAD when encrypting HTTP requests

### Recommended Fix

Implement `decrypt_record()` and `encrypt_record()` methods that correctly construct AAD from TLS record headers.

---

## 🎯 Execution Plan

### Phase 1: Fix ClientHello (PRIORITY 1 - CRITICAL)

**Files to Modify**:
- `crates/songbird-http-client/src/tls/handshake.rs`

**Changes**:
1. Add ALPN extension to `build_extensions()`
2. Optionally reorder extensions for maximum compatibility
3. Add hex dump validation in logs

**Expected Result**: GitHub stops sending Fatal Alert 0x28

---

### Phase 2: Fix TLS Record Encryption/Decryption (PRIORITY 2)

**Files to Modify**:
- `crates/songbird-http-client/src/tls/handshake.rs` (add helper methods)
- `crates/songbird-http-client/src/client.rs` (use helpers for HTTP)

**Changes**:
1. Add `encrypt_record()` method with correct AAD
2. Add `decrypt_record()` method with correct AAD
3. Add nonce construction (sequence number XOR IV)
4. Use these methods when sending/receiving HTTP over TLS

**Expected Result**: AEAD authentication succeeds

---

### Phase 3: Integration Testing (PRIORITY 3)

**Tests**:
1. GitHub API (https://api.github.com/zen)
2. CloudFlare (https://cloudflare.com)
3. Google (https://www.google.com)

**Expected Result**: All HTTPS requests succeed

---

## 📝 Implementation Details

### Fix 1: Add ALPN Extension

**Location**: `crates/songbird-http-client/src/tls/handshake.rs:280-323`

**Add after SNI extension** (line 288):
```rust
// ALPN extension (0x0010) - Application-Layer Protocol Negotiation
ext.extend_from_slice(&[0x00, 0x10]); // Extension type
ext.extend_from_slice(&[0x00, 0x0c]); // Length: 12
ext.extend_from_slice(&[0x00, 0x0a]); // Protocol list length: 10
ext.extend_from_slice(&[0x08]); // Protocol name length: 8
ext.extend_from_slice(b"http/1.1"); // Protocol name
```

### Fix 2: Add Record Encryption/Decryption

**Location**: `crates/songbird-http-client/src/tls/handshake.rs` (new methods)

```rust
/// Encrypt application data for TLS 1.3
pub async fn encrypt_application_data(
    &self,
    plaintext: &[u8],
    keys: &SessionKeys,
    sequence_number: u64,
) -> Result<Vec<u8>> {
    // Construct TLS record header
    let record_type = 0x17; // APPLICATION_DATA
    let version = [0x03, 0x03]; // TLS 1.2 (compatibility)
    let length = (plaintext.len() + 16) as u16; // +16 for AEAD tag
    
    // AAD = entire record header
    let aad = [
        record_type,
        version[0],
        version[1],
        (length >> 8) as u8,
        (length & 0xFF) as u8,
    ];
    
    // Nonce = IV XOR sequence number (TLS 1.3 nonce construction)
    let mut nonce = keys.client_write_iv.clone();
    let seq_bytes = sequence_number.to_be_bytes();
    for (i, byte) in seq_bytes.iter().enumerate() {
        let nonce_idx = nonce.len() - 8 + i;
        if nonce_idx < nonce.len() {
            nonce[nonce_idx] ^= byte;
        }
    }
    
    // Encrypt
    let ciphertext = self.beardog.encrypt(
        &keys.client_write_key,
        &nonce,
        plaintext,
        &aad,
    ).await?;
    
    // Return full TLS record (header + ciphertext + tag)
    let mut record = Vec::new();
    record.extend_from_slice(&aad);
    record.extend_from_slice(&ciphertext);
    
    Ok(record)
}

/// Decrypt application data for TLS 1.3
pub async fn decrypt_application_data(
    &self,
    record_header: &[u8; 5],
    ciphertext: &[u8],
    keys: &SessionKeys,
    sequence_number: u64,
) -> Result<Vec<u8>> {
    // AAD = record header
    let aad = record_header;
    
    // Nonce = IV XOR sequence number
    let mut nonce = keys.server_write_iv.clone();
    let seq_bytes = sequence_number.to_be_bytes();
    for (i, byte) in seq_bytes.iter().enumerate() {
        let nonce_idx = nonce.len() - 8 + i;
        if nonce_idx < nonce.len() {
            nonce[nonce_idx] ^= byte;
        }
    }
    
    // Decrypt
    self.beardog.decrypt(
        &keys.server_write_key,
        &nonce,
        ciphertext,
        aad,
    ).await
}
```

---

## 🎉 Expected Outcomes

### After Phase 1 (ClientHello Fix)
- ✅ GitHub stops sending Fatal Alert 0x28
- ✅ ServerHello received successfully
- ✅ Handshake completes through post-handshake messages
- ⏳ HTTP requests still fail (need Phase 2)

### After Phase 2 (Record Encryption/Decryption)
- ✅ HTTP requests encrypted correctly
- ✅ HTTP responses decrypted correctly
- ✅ AEAD authentication succeeds
- ✅ Full HTTPS working end-to-end!

### After Phase 3 (Integration Testing)
- ✅ GitHub API works
- ✅ CloudFlare works
- ✅ Google works
- ✅ 100% Pure Rust HTTPS achieved! 🦀✨

---

## 📊 Success Metrics

**Before Session 14**:
- TLS Handshake: 80% complete
- HTTPS Requests: 0% working
- Infrastructure: 100% validated

**After Session 14 (Target)**:
- TLS Handshake: 100% complete ✅
- HTTPS Requests: 100% working ✅
- Infrastructure: 100% validated ✅

**Grade**: A → A+ (Full Pure Rust HTTPS)

---

## 🚀 Next Steps

1. ✅ Create this execution plan document
2. ⏳ Implement Fix 1 (Add ALPN extension)
3. ⏳ Test with GitHub API
4. ⏳ Implement Fix 2 (Record encryption/decryption)
5. ⏳ Test with GitHub API (end-to-end)
6. ⏳ Integration testing (CloudFlare, Google)
7. ⏳ Document completion
8. ⏳ Push to Git

---

**Status**: Ready for execution  
**Confidence**: HIGH (issues are well understood)  
**Complexity**: MEDIUM (surgical fixes, not architectural changes)  
**Timeline**: 1-2 hours for full implementation and testing

---

*Plan created: January 22, 2026*  
*Session 14: TLS Protocol Evolution*  
*Target: 100% Pure Rust HTTPS 🦀✨*

