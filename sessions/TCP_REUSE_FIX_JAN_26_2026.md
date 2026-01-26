# 🔧 TCP Connection Reuse Fix - January 26, 2026

**Status**: ✅ COMPLETE  
**Priority**: HIGH (Critical Bug Fix)  
**Fix Time**: 15 minutes  

---

## 🐛 Root Cause

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

*Fix discovered and implemented: January 26, 2026*

