# TLS decrypt_error Alert Investigation

**Date**: January 23, 2026 (9:15 PM)  
**Priority**: CRITICAL  
**Status**: Under Investigation  
**Version**: v5.12.1

---

## 🔍 Problem Statement

**Servers are sending fatal `decrypt_error` alerts instead of HTTP responses!**

**Alert Details**:
- ContentType: 0x15 (ALERT)
- Alert Level: 0x02 (fatal)
- Alert Description: 0x33 (51 decimal) = **decrypt_error**

**What This Means**: Server received our encrypted HTTP request but **cannot decrypt it**.

---

## ✅ What's Working

1. ✅ TLS 1.3 handshake completes successfully
2. ✅ ECDH key exchange works
3. ✅ Handshake traffic key derivation works
4. ✅ Application traffic key derivation works
5. ✅ Client Finished message is accepted by server
6. ✅ EOF handling works correctly (v5.12.1)

**Evidence**: Server accepts our Client Finished (encrypted with handshake keys), which means:
- Our handshake encryption works
- Server can derive the same handshake keys
- Handshake completes successfully

---

## ❌ What's NOT Working

**HTTP request encryption causes decrypt_error!**

**Sequence**:
1. ✅ Handshake completes
2. ✅ Client Finished sent (encrypted with handshake keys, seq=0)
3. ✅ Switch to application keys
4. ✅ HTTP request encrypted (with application keys, seq=0)
5. ❌ Server tries to decrypt HTTP request
6. ❌ Decryption fails → Server sends fatal `decrypt_error`
7. ❌ Connection closes

---

## 🎯 Root Cause Hypotheses

### Hypothesis #1: Wrong Transcript Hash for Application Keys ⭐ MOST LIKELY

**Issue**: Application keys might be derived with wrong transcript

**RFC 8446 Section 7.1**: Application traffic secrets are derived using:
```
Derive-Secret(Master-Secret, "c ap traffic", ClientHello...server Finished)
Derive-Secret(Master-Secret, "s ap traffic", ClientHello...server Finished)
```

**Critical**: Transcript should include server Finished but NOT client Finished!

**Check in Code** (`handshake.rs` line 500):
```rust
let secrets = self.beardog
    .tls_derive_application_secrets(
        &shared_secret, 
        &client_random, 
        &server_random, 
        &transcript_hash,  // ← This must be correct!
        self.cipher_suite
    )
    .await?;
```

**Questions**:
1. Does `transcript_hash` include server Finished? (Should: YES)
2. Does `transcript_hash` include client Finished? (Should: NO)
3. Is transcript computed correctly in BearDog?

---

### Hypothesis #2: Sequence Number Issue

**Issue**: Maybe sequence number management is wrong

**Current Behavior**:
- Client Finished: seq=0 (handshake keys)
- HTTP Request: seq=0 (application keys)

**This is CORRECT per RFC 8446!** Sequence numbers reset to 0 when switching traffic secrets.

**But Check**:
- Are we incrementing after each write? (YES - line 132 in `record.rs`)
- Are we using the right sequence number for nonce? (YES - XOR with IV)
- Is nonce calculation correct? (YES - RFC 8446 Section 5.3)

**Verdict**: Sequence numbers look correct ✅

---

### Hypothesis #3: Wrong Keys Being Used

**Issue**: Maybe we're using handshake keys instead of application keys?

**Check in Code** (`record.rs` line 76):
```rust
self.beardog.encrypt_aes_128_gcm(
    &self.keys.client_write_key,  // ← Application key, not handshake key
    &nonce,
    &plaintext_with_type,
    &aad,
).await
```

**Where are keys from?** (`handshake.rs` line 524):
```rust
Ok(SessionKeys {
    client_write_key: secrets.client_write_key,  // ← From application secrets
    server_write_key: secrets.server_write_key,
    client_write_iv: secrets.client_write_iv,
    server_write_iv: secrets.server_write_iv,
    cipher_suite: self.cipher_suite,
})
```

**Verdict**: We're using application keys correctly ✅

---

### Hypothesis #4: AAD Mismatch

**Issue**: Maybe AAD doesn't match TLS record header?

**Check in Code** (`record.rs` line 53):
```rust
let aad = [
    content_type::APPLICATION_DATA,  // 0x17
    0x03, 0x03,                      // TLS 1.2 (legacy)
    (encrypted_length >> 8) as u8,
    (encrypted_length & 0xFF) as u8,
];
```

**This matches TLS record header we send**: ✅
- ContentType: 0x17 (APPLICATION_DATA)
- Version: 0x0303 (TLS 1.2 legacy)
- Length: encrypted length (plaintext + 1 + 16)

**Verdict**: AAD looks correct ✅

---

### Hypothesis #5: BearDog Application Key Derivation Bug

**Issue**: Maybe BearDog is deriving wrong application keys?

**What We Know**:
- Handshake key derivation works (Client Finished accepted)
- Application key derivation might be different

**Check**:
1. Is BearDog using the correct HKDF labels?
   - "c ap traffic" for client application traffic secret
   - "s ap traffic" for server application traffic secret
2. Is transcript hash being used correctly?
3. Is the master secret correct?

**This requires BearDog team investigation!**

---

## 🧪 Diagnostic Steps

### Step 1: Add Comprehensive Logging

**Add to `client.rs` before sending HTTP request**:
```rust
info!("════════════════════════════════════════");
info!("📤 SENDING HTTP REQUEST (DIAGNOSTIC INFO)");
info!("════════════════════════════════════════");
info!("Cipher suite: 0x{:04x}", record_layer.keys.cipher_suite);
info!("Client write key length: {} bytes", record_layer.keys.client_write_key.len());
info!("Client write IV length: {} bytes", record_layer.keys.client_write_iv.len());
info!("Write sequence number: {}", record_layer.write_sequence_number);
info!("HTTP request size: {} bytes", http_request.len());
debug!("Client write key (hex): {}", hex::encode(&record_layer.keys.client_write_key));
debug!("Client write IV (hex): {}", hex::encode(&record_layer.keys.client_write_iv));
info!("════════════════════════════════════════");
```

### Step 2: Verify Transcript Hash

**Add to `handshake.rs` before deriving application keys** (line 485):
```rust
// Add detailed transcript logging
info!("════════════════════════════════════════");
info!("📊 TRANSCRIPT FOR APPLICATION KEY DERIVATION");
info!("════════════════════════════════════════");
info!("Transcript length: {} bytes", self.transcript.len());
info!("Transcript hash length: {} bytes", transcript_hash.len());
debug!("Full transcript (hex): {}", hex::encode(&self.transcript));
debug!("Transcript hash (hex): {}", hex::encode(&transcript_hash));
info!("Messages included:");
info!("  1. ClientHello (plaintext, no TLS header)");
info!("  2. ServerHello (plaintext, no TLS header)");
info!("  3. EncryptedExtensions (DECRYPTED, no TLS header)");
info!("  4. Certificate (DECRYPTED, no TLS header)");
info!("  5. CertificateVerify (DECRYPTED, no TLS header)");
info!("  6. Server Finished (DECRYPTED, no TLS header)");
info!("  ❌ NOT included: Client Finished (added later)");
info!("════════════════════════════════════════");
```

### Step 3: Compare Handshake vs Application Encryption

**Add logging to show the difference**:
```rust
// In send_client_finished (line 1340):
info!("🔐 HANDSHAKE ENCRYPTION (Client Finished):");
info!("   Using: HANDSHAKE traffic keys");
info!("   Sequence: 0 (first handshake message sent)");
info!("   Key source: client_handshake_traffic_secret");

// In record.rs write_application_data (line 68):
info!("🔐 APPLICATION ENCRYPTION (HTTP Request):");
info!("   Using: APPLICATION traffic keys");
info!("   Sequence: {}", self.write_sequence_number);
info!("   Key source: client_application_traffic_secret");
```

---

## 🔬 Test Plan

### Test 1: Minimal HTTP Request

**Try sending smallest possible HTTP request**:
```http
GET / HTTP/1.1\r\nHost: example.com\r\n\r\n
```

**Why**: If even a minimal request fails, it's definitely a key/encryption issue, not content-related.

### Test 2: Compare with OpenSSL

**Run OpenSSL s_client with keylog**:
```bash
SSLKEYLOGFILE=keys.log openssl s_client -connect example.com:443 -tls1_3
```

**Then compare**:
- Are our application keys the same as OpenSSL's?
- Are our nonces the same?
- Is our AAD the same?

### Test 3: Wireshark Capture

**Capture TLS traffic**:
```bash
tcpdump -i any -w songbird_tls.pcap host example.com
```

**Analyze**:
- Does our TLS record look correct?
- Is the encrypted length correct?
- Does the record structure match expectations?

---

## 🎯 Most Likely Root Cause

**Hypothesis #1: Transcript Hash Issue** ⭐

**Why This is Most Likely**:
1. Handshake encryption works (proves our encryption logic is sound)
2. Application encryption fails (suggests key derivation issue)
3. The ONLY difference is the transcript hash used for key derivation

**What to Check**:
1. Is BearDog computing transcript hash correctly?
2. Is transcript hash including the right messages?
3. Is HKDF-Expand-Label using correct labels for application keys?

**This requires coordination with BearDog team!**

---

## 📋 Action Items

### For Songbird Team (Immediate)

- [ ] **Add comprehensive diagnostic logging** (Steps 1-3 above)
- [ ] **Test with minimal HTTP request**
- [ ] **Verify transcript composition** (what's included/excluded)
- [ ] **Capture Wireshark trace** for analysis
- [ ] **Compare nonce/AAD with expectations**

### For BearDog Team (Critical)

- [ ] **Verify `tls_derive_application_secrets` implementation**
- [ ] **Check HKDF labels**: "c ap traffic", "s ap traffic"
- [ ] **Verify transcript hash is used correctly**
- [ ] **Compare with OpenSSL implementation**
- [ ] **Add debug logging for key derivation**

### For biomeOS Team

- [ ] **Run with `RUST_LOG=trace`** and capture full logs
- [ ] **Test multiple servers** (example.com, github.com, google.com)
- [ ] **Share Wireshark capture** if possible
- [ ] **Report exact error messages** from logs

---

## 🚨 Critical Questions

1. **Does BearDog's `tls_derive_application_secrets` use the transcript hash correctly?**
   - This is the MOST LIKELY issue

2. **Are we passing the correct transcript hash to BearDog?**
   - Should include: ClientHello, ServerHello, EncryptedExtensions, Certificate, CertificateVerify, server Finished
   - Should NOT include: client Finished

3. **Is the transcript being modified after we compute the hash?**
   - Check if anything adds to transcript between lines 485-500

---

## 💡 Key Insight

**The fact that Client Finished works but HTTP request doesn't strongly suggests**:
- ✅ Our encryption logic is correct
- ✅ Our AEAD implementation is correct
- ✅ Our nonce/AAD construction is correct
- ❌ Our APPLICATION key derivation has a bug

**The bug is most likely in**:
1. Transcript hash computation/usage for application keys
2. BearDog's implementation of `tls_derive_application_secrets`

---

## 📊 Next Steps

**Priority Order**:
1. **Add diagnostic logging** (30 min)
2. **Coordinate with BearDog** to verify key derivation (1 hour)
3. **Test with minimal request** (15 min)
4. **Compare with OpenSSL** (30 min)
5. **Fix root cause** (time depends on findings)

---

**Status**: Investigation in progress  
**ETA**: Unknown (depends on root cause)  
**Blocker**: Need BearDog team verification of application key derivation

**THE INVESTIGATION CONTINUES...** 🔍

