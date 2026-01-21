# 🔍 TLS Handshake Gap Analysis - January 21, 2026

**Date**: January 21, 2026  
**Issue**: HTTPS timeouts after 15 seconds  
**Status**: ⚠️ **ROOT CAUSE IDENTIFIED**  
**Priority**: 🔴 **CRITICAL**

---

## Executive Summary

biomeOS deployment verification revealed:
- ✅ **HTTP Working**: example.com returns 400 (functional)
- ⏳ **HTTPS Timeout**: api.github.com hangs for 15 seconds
- ✅ **BearDog Ready**: All TLS crypto methods implemented
- ⚠️ **Songbird Incomplete**: TLS 1.3 handshake partially implemented

**Root Cause**: TLS 1.3 handshake implementation is incomplete. Only ClientHello/ServerHello are handled. Post-handshake encrypted messages (EncryptedExtensions, Certificate, Finished) are missing.

---

## 🔍 Technical Analysis

### TLS 1.3 Handshake Flow (RFC 8446)

```
Client                                           Server

ClientHello
+ key_share
+ signature_algorithms
+ supported_versions         -------->
                                                  ServerHello  ✅ WE READ THIS
                                                  + key_share
                                                  + supported_versions
                                        {EncryptedExtensions}  ❌ NOT HANDLED
                                        {CertificateRequest*}
                                               {Certificate}  ❌ NOT HANDLED
                                         {CertificateVerify}  ❌ NOT HANDLED
                                                   {Finished}  ❌ NOT HANDLED
                             <--------  [Application Data*]
{Finished}                   -------->  ❌ NOT SENT
[Application Data]           <------->  [Application Data]
```

**Legend**:
- `{}` = Encrypted with handshake keys
- `[]` = Encrypted with application keys
- `*` = Optional

### Current Implementation Gap

**File**: `crates/songbird-http-client/src/tls/handshake.rs`

**What We Do** ✅:
```rust
// Line 38-47: Send ClientHello
let client_hello = self.build_client_hello(...)?;
stream.write_all(&client_hello).await?;

// Line 49-51: Receive ServerHello
let server_hello = self.read_server_hello(stream).await?;

// Line 53-61: Derive keys
let (server_random, server_public) = self.parse_server_hello(&server_hello)?;
let shared_secret = self.beardog.ecdh_derive(&client_private, &server_public).await?;
let secrets = self.beardog.tls_derive_secrets(...).await?;

// Line 70-76: Return keys
Ok(SessionKeys { ... })
```

**What We DON'T Do** ❌:
1. Read EncryptedExtensions (encrypted with handshake traffic keys)
2. Read Certificate chain (encrypted)
3. Read CertificateVerify (encrypted)
4. Read server Finished (encrypted with handshake traffic keys)
5. Send client Finished (encrypted)

**Result**: Server waits for client Finished → Client waits for HTTP response → **DEADLOCK**

---

## 🐛 Why It Hangs

### Sequence of Events:

1. ✅ Client sends ClientHello
2. ✅ Server sends ServerHello
3. ✅ Client derives handshake keys
4. ❌ Server sends EncryptedExtensions (encrypted) — **WE IGNORE THIS**
5. ❌ Server sends Certificate (encrypted) — **WE IGNORE THIS**
6. ❌ Server sends CertificateVerify (encrypted) — **WE IGNORE THIS**
7. ❌ Server sends Finished (encrypted) — **WE IGNORE THIS**
8. ❌ Client should send Finished (encrypted) — **WE NEVER SEND THIS**
9. ❌ Client tries to send HTTP request (encrypted with wrong keys)
10. ⏳ Server ignores invalid data, waits for client Finished
11. ⏳ Client waits for HTTP response
12. ⏳ **15-second timeout**

---

## 🏗️ Required Fixes

### 1. Derive Handshake Traffic Keys ⚠️

**Problem**: We derive application traffic secrets but never derive handshake traffic secrets.

**Current Code**:
```rust
let secrets = self.beardog.tls_derive_secrets(
    &shared_secret, 
    &client_random, 
    &server_random
).await?;
```

**Issue**: This likely returns **application** traffic keys, but we need **handshake** traffic keys first!

**Fix**: BearDog needs to return BOTH:
- Handshake traffic keys (for reading post-handshake messages)
- Application traffic keys (for HTTP data)

---

### 2. Read & Decrypt Post-Handshake Messages ❌

**Required**:
```rust
// After ServerHello, switch to encrypted mode
let handshake_keys = derive_handshake_keys(&shared_secret, ...);

// Read EncryptedExtensions
let encrypted_extensions = read_encrypted_record(stream, &handshake_keys).await?;

// Read Certificate
let certificate = read_encrypted_record(stream, &handshake_keys).await?;

// Read CertificateVerify
let cert_verify = read_encrypted_record(stream, &handshake_keys).await?;

// Read server Finished
let server_finished = read_encrypted_record(stream, &handshake_keys).await?;

// Verify server Finished
verify_finished(&server_finished, &handshake_transcript)?;
```

**Note**: All these messages are encrypted with handshake traffic keys!

---

### 3. Send Client Finished ❌

**Required**:
```rust
// Compute client Finished message
let client_finished_payload = compute_finished(&handshake_transcript, &client_handshake_key)?;

// Encrypt and send
send_encrypted_record(stream, &client_finished_payload, &handshake_keys).await?;
```

---

### 4. Transcript Hash Maintenance ❌

**Problem**: TLS 1.3 Finished message requires a transcript hash of all handshake messages.

**Required**:
```rust
let mut transcript = Vec::new();
transcript.extend_from_slice(&client_hello);
transcript.extend_from_slice(&server_hello);
transcript.extend_from_slice(&encrypted_extensions);
transcript.extend_from_slice(&certificate);
transcript.extend_from_slice(&cert_verify);

let transcript_hash = blake3_hash(&transcript);
```

**Then**: Use transcript hash to compute Finished verify data.

---

### 5. Add Timeouts ⏳

**Problem**: No timeout on socket reads.

**Current Code**:
```rust
stream.read_exact(&mut header).await?;
```

**Fix**:
```rust
tokio::time::timeout(
    Duration::from_secs(10),
    stream.read_exact(&mut header)
).await??;
```

---

## 📋 Implementation Plan

### Phase 1: Immediate (1-2 hours) - Make HTTPS Work

**Goal**: Complete basic TLS 1.3 handshake (no certificate validation)

**Tasks**:
1. ✅ Add handshake traffic key derivation to BearDog
2. ✅ Read all post-handshake encrypted messages
3. ✅ Send client Finished message
4. ✅ Add timeout protection
5. ✅ Add handshake logging

**Outcome**: HTTPS should work (but no cert validation)

---

### Phase 2: Security (2-4 hours) - Add Certificate Validation

**Goal**: Properly validate server certificates

**Tasks**:
1. Parse Certificate messages
2. Extract certificate chain
3. Delegate verification to BearDog
4. Validate hostname matches certificate
5. Check certificate expiration

**Outcome**: Production-grade TLS security

---

### Phase 3: Testing (2-3 hours) - Comprehensive Test Suite

**Goal**: Ensure reliability

**Tasks**:
1. Unit tests for each handshake step
2. E2E tests with real servers (github.com, google.com, etc.)
3. Error case testing (invalid certs, wrong keys, etc.)
4. Performance benchmarking
5. Concurrent request testing

**Outcome**: High confidence in HTTPS stack

---

## 🎯 Minimum Viable Fix (TODAY)

To unblock biomeOS immediately:

### Option A: Complete TLS 1.3 Handshake (2-3 hours)
**Pros**: Proper solution, unblocks HTTPS  
**Cons**: Requires careful implementation

### Option B: Use rustls (30 minutes)
**Pros**: Battle-tested TLS library, immediate fix  
**Cons**: Adds dependency, but rustls is Pure Rust

### Recommendation: **Option A**

Completing our custom TLS handshake is:
1. Educational - we learn TLS 1.3 deeply
2. Aligned with Tower Atomic vision
3. Not that complex (just missing 5 steps)
4. Gives us full control

---

## 🔑 Key Insights

### Why HTTP Works But HTTPS Doesn't

**HTTP**:
```
Client → Server: HTTP request (plaintext)
Server → Client: HTTP response (plaintext)
✅ Simple, works!
```

**HTTPS (Current)**:
```
Client → Server: ClientHello
Server → Client: ServerHello
Client: Derives keys, tries to send HTTP
Server: Still waiting for Finished message
❌ Deadlock!
```

**HTTPS (Should Be)**:
```
Client → Server: ClientHello
Server → Client: ServerHello + encrypted messages + Finished
Client → Server: Finished
Client → Server: HTTP request (encrypted)
Server → Client: HTTP response (encrypted)
✅ Works!
```

---

## 📊 Complexity Assessment

### TLS 1.3 Handshake Completion

| Component | Complexity | Time | Status |
|-----------|------------|------|--------|
| Handshake key derivation | LOW | 30min | ❌ TODO |
| Read encrypted records | MEDIUM | 1h | ❌ TODO |
| Decrypt with handshake keys | LOW | 30min | ❌ TODO |
| Parse Certificate | MEDIUM | 1h | ⏳ DEFER |
| Verify certificate | HIGH | 2h | ⏳ DEFER |
| Send Finished | LOW | 30min | ❌ TODO |
| Transcript hash | MEDIUM | 1h | ❌ TODO |
| Add timeouts | LOW | 15min | ❌ TODO |

**Total for MVP**: ~4 hours (defer certificate validation)

---

## 🚀 Recommendation

### For biomeOS (Immediate)

**Option 1**: Use HTTP endpoints where possible
- Many package repos support HTTP (faster anyway)
- No security issue for verified packages (checksums)

**Option 2**: Wait 1 day for TLS completion
- We'll complete the handshake today
- Should be ready for testing tomorrow

### For Songbird Team (Us)

**Priority 1** (Today): Complete TLS 1.3 handshake
1. Add handshake traffic keys to BearDog RPC
2. Implement post-handshake message reading
3. Send client Finished
4. Add comprehensive logging
5. Test with real servers

**Priority 2** (Tomorrow): Certificate validation
1. Parse certificate chain
2. Validate signatures
3. Check hostname
4. Check expiration

**Priority 3** (This Week): Testing & hardening
1. Unit tests (20+)
2. E2E tests (5+ real servers)
3. Error cases
4. Performance benchmarks

---

## 📚 References

- **TLS 1.3 RFC**: https://www.rfc-editor.org/rfc/rfc8446
- **Handshake Protocol**: Section 4 of RFC 8446
- **Key Schedule**: Section 7.1 of RFC 8446
- **Finished Message**: Section 4.4.4 of RFC 8446

---

## ✅ Success Criteria

### Phase 1 Complete When:
- ✅ `echo '{"method":"GET","url":"https://api.github.com/zen"}' | nc -U /tmp/songbird.sock` returns JSON
- ✅ No 15-second timeout
- ✅ HTTP response status code received
- ⚠️ Certificate validation can be skipped (accept all certs)

### Phase 2 Complete When:
- ✅ Invalid certificates rejected
- ✅ Expired certificates rejected
- ✅ Hostname mismatch rejected
- ✅ Self-signed certificates rejected (unless explicitly trusted)

### Phase 3 Complete When:
- ✅ 20+ unit tests passing
- ✅ 5+ real HTTPS servers tested (github.com, google.com, cloudflare.com, etc.)
- ✅ < 100ms TLS handshake latency
- ✅ Concurrent requests work correctly

---

**Status**: ⚠️ **READY TO FIX**  
**Estimated Time**: 4-6 hours for MVP HTTPS  
**Blocker**: None (all info available)  
**Priority**: 🔴 **CRITICAL** (blocks biomeOS HTTPS usage)

---

**🔥 LET'S COMPLETE THIS TLS HANDSHAKE! 🔥**

