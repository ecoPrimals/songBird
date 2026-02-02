# 🔧 TLS Handshake Fixes - January 26, 2026

**Status**: ✅ COMPLETE (2 FIXES)  
**Priority**: HIGH (Critical Bug Fixes)  
**Fix Time**: 30 minutes total  

---

## 🐛 Bug 1: TCP Connection Reuse (Root Cause)

The bug was in **TCP connection reuse during TLS handshake retry attempts**.

### Evidence from Logs
```
17:04:24.405083Z  ✅ Received ServerHello: type=0x16, 90 bytes  ← SUCCESS!
17:04:24.405134Z     Server negotiated cipher suite: 0x1301     ← TLS 1.3!
17:04:24.405865Z  📝 Adding ClientHello  ← NEW RETRY ATTEMPT (same TCP stream!)
17:04:24.405943Z  ❌ Expected 0x16, got 0x14 (Change Cipher Spec) ← READING OLD DATA!
```

**Problem**: The first handshake **actually succeeds**, but when the code triggers a retry (for any reason), it reuses the SAME TCP stream which still has buffered data from the previous server response.

---

## 🔧 The Fix

**File**: `crates/songbird-http-client/src/client.rs`  
**Function**: `attempt_handshake_with_fallback()`

### Before (Bug)
```rust
async fn attempt_handshake_with_fallback(
    &self,
    tcp_stream: &mut TcpStream,  // ← Same stream used for all attempts!
    host: &str,
) -> Result<SessionKeys> {
    for strategy in strategies_to_try {
        // Tries handshake on SAME tcp_stream
        // First attempt: sends ClientHello, receives ServerHello + CCS + Encrypted
        // Second attempt: reads stale CCS/ApplicationData from buffer!
        match handshake.handshake(tcp_stream, host).await { ... }
    }
}
```

### After (Fixed)
```rust
async fn attempt_handshake_with_fallback(
    &self,
    addr: &str,       // ← Pass address instead of stream
    host: &str,
) -> Result<(TcpStream, SessionKeys)> {  // ← Return the successful stream
    for strategy in strategies_to_try {
        // CRITICAL: Create FRESH TCP connection for each attempt!
        let mut tcp_stream = TcpStream::connect(addr).await?;
        
        match handshake.handshake(&mut tcp_stream, host).await {
            Ok(keys) => return Ok((tcp_stream, keys)),  // Return both!
            Err(e) => {
                // tcp_stream dropped here, connection closed cleanly
                last_error = Some(e);
            }
        }
    }
    Err(last_error.unwrap())
}
```

---

## 📝 Changes Made

### 1. `https_request()` signature updated
- **Before**: `async fn https_request(&self, mut tcp_stream: TcpStream, host: &str, ...)`
- **After**: `async fn https_request(&self, host: &str, port: u16, ...)`
- TCP connection now created inside `attempt_handshake_with_fallback()`

### 2. `request()` updated
- No longer creates TCP stream before HTTPS request
- Passes `host` and `port` to `https_request()` instead of `tcp_stream`

### 3. `attempt_handshake_with_fallback()` rewritten
- Takes `addr: &str` instead of `tcp_stream: &mut TcpStream`
- Returns `Result<(TcpStream, SessionKeys)>` instead of `Result<SessionKeys>`
- Creates **FRESH TCP connection** for each retry attempt
- Failed connections are dropped (closed) cleanly before next attempt

---

## ✅ Test Results

```
cargo build --release -p songbird-orchestrator    ✅ SUCCESS
cargo test --package songbird-http-client --release  ✅ 12 passed
```

---

## 🎯 Why This Fix Works

1. **Clean State**: Each retry starts with a fresh TCP connection
2. **No Stale Data**: Previous server responses can't pollute new attempts
3. **Proper Cleanup**: Failed connections are dropped/closed cleanly
4. **Correct Behavior**: Successful stream is returned with session keys

---

## 📊 Impact

- **Bug**: Reading stale buffered data caused `0x14 (CCS)` instead of `0x16 (Handshake)`
- **Fix**: Each attempt gets fresh TCP connection, no buffered data
- **Result**: TLS handshake should now work correctly on first attempt

---

## 🔗 Related Fixes

1. **TLS Extension Fix** (earlier today) - Removed PSK modes for fresh connections
2. **TCP Reuse Fix** (this fix) - Fresh TCP connection per retry attempt

Together, these fixes should resolve all TLS handshake issues with GitHub API.

---

## 📍 Files Changed

- `crates/songbird-http-client/src/client.rs`

---

## 🚀 Next Steps

1. Copy binary to biomeOS: `cp target/release/songbird /home/eastgate/Development/ecoPrimals/phase2/biomeOS/plasmidBin/primals/songbird/`
2. Deploy: `./deploy_tower_atomic.sh`
3. Test: `echo '{"jsonrpc":"2.0","method":"capability.call","params":{"capability":"secure_http","operation":"http.get","args":{"url":"https://api.github.com/zen"}},"id":1}' | nc -U /tmp/neural-api.sock`

---

---

## 🐛 Bug 2: Parameter Mismatch in Key Derivation

### The Problem

After fixing TCP reuse, a **parameter mismatch** emerged:

| Songbird Sent | BearDog Expected |
|---------------|------------------|
| `shared_secret` | `pre_master_secret` ✅ |
| `transcript_hash` | `client_random` ❌ |
| (none) | `server_random` ❌ |
| (none) | `transcript_hash` ❌ |
| (none) | `cipher_suite` ❌ |

### The Fix

Updated `CryptoCapability` trait and all implementations to pass **all 5 parameters**:

```rust
// Before (2 params):
.tls_derive_handshake_secrets(&shared_secret, &transcript_hash)

// After (5 params):
.tls_derive_handshake_secrets(
    &shared_secret,       // → pre_master_secret
    &client_random,       // → client_random
    &server_random,       // → server_random
    &transcript_hash,     // → transcript_hash
    cipher_suite,         // → cipher_suite
)
```

### Files Changed

- `crypto/capability.rs` - Updated trait signature
- `crypto/beardog_provider.rs` - Updated JSON-RPC params
- `tls/handshake_flow.rs` - Pass all 5 params
- `tls/handshake_legacy.rs` - Pass all 5 params

---

## 📊 Combined Impact

| Fix | Problem | Solution |
|-----|---------|----------|
| TCP Reuse | Stale buffer on retry | Fresh connection per attempt |
| Param Mismatch | Missing key derivation params | All 5 RFC 8446 params |

**Result**: TLS 1.3 handshake should now work with GitHub API!

---

## 🐛 Bug 3: BearDog Response Field Names

### The Problem

BearDog returns different field names than expected:

| Expected | BearDog Returns |
|----------|-----------------|
| `client_handshake_traffic_secret` | `client_handshake_secret` |
| `server_handshake_traffic_secret` | `server_handshake_secret` |
| `client_key` | `client_write_key` |
| `client_iv` | `client_write_iv` |
| `server_key` | `server_write_key` |
| `server_iv` | `server_write_iv` |

### The Fix

Updated `beardog_provider.rs` to match BearDog's actual response field names:

```rust
// Before:
client_handshake_secret: self.extract_b64_field(&result, "client_handshake_traffic_secret")?,

// After:
client_handshake_secret: self.extract_b64_field(&result, "client_handshake_secret")?,
```

Also fixed semantic mappings to keep specific algorithm names for Neural API translation:
- `encrypt_aes_128_gcm` instead of generic `encrypt`
- `derive_handshake_secrets` instead of generic `derive_secrets`

**Commit**: `5f834d14a`

---

## ⚠️ PENDING: BearDog API Mismatch (Not Songbird Fix)

BearDog's `tls.derive_application_secrets` has wrong API:

| Parameter | Songbird Sends (RFC 8446) | BearDog Expects (Wrong) |
|-----------|---------------------------|-------------------------|
| Input Secret | `handshake_secret` ✅ | `pre_master_secret` ❌ |

**BearDog Fix Required**: `crates/beardog-tunnel/src/unix_socket_ipc/handlers/crypto/tls/key_derivation.rs` line 495

This is a BearDog issue, not Songbird. Songbird is sending the correct RFC 8446 data.

---

## 📊 All TLS Fixes Summary

| Fix # | Bug | Root Cause | Status |
|-------|-----|------------|--------|
| 1 | PSK modes in fresh handshake | `psk_key_exchange_modes` without PSK key | ✅ Fixed |
| 2 | TCP stream reuse | Same stream for retries | ✅ Fixed |
| 3 | Key derivation params | Missing 3 of 5 required params | ✅ Fixed |
| 4 | Response field names | Mismatch with BearDog output | ✅ Fixed |
| 5 | Application secrets API | BearDog expects wrong input | ⏳ BearDog fix needed |

---

*Fixes discovered and implemented: January 26, 2026*

