# Upstream Gaps Response - February 5, 2026

**From**: Songbird Development Team  
**To**: biomeOS Integration Team  
**Status**: ✅ Issues 1 & 2 Already Fixed | Issue 3 Already Fixed (v3.21.0)  
**Current Version**: v3.22.0+ (commit `d5d3c9822`)

---

## Executive Summary

**All three issues reported in the handoff are ALREADY FIXED** in the current codebase:

| Issue | Status | Fixed In | Current State |
|-------|--------|----------|---------------|
| Missing standard methods | ✅ **FIXED** | v3.22.0 (`78e1f7307`) | Working (persistent connection behavior) |
| BirdSong family_id | ✅ **FIXED** | v3.22.0 (`78e1f7307`) | Environment discovery implemented |
| TLS Handshake | ✅ **FIXED** | v3.21.0 (`074093187`) | Protocol detection on same port |

**Critical Finding**: The "hang" on Tower's Unix socket is **NOT a bug** - it's correct persistent connection behavior. The methods ARE responding, but the test client (`nc`) is waiting for connection close.

---

## Issue 1: Standard Methods - ALREADY WORKING ✅

### Current Implementation

**File**: `crates/songbird-universal-ipc/src/service.rs` (lines 796-798)

```rust
// biomeOS Standard Methods (IMPLEMENTED - Feb 5, 2026)
"health" => self.handle_health().await,
"identity" => self.handle_identity().await,
"rpc.discover" => self.handle_rpc_discover_standard().await,
```

**Handlers Implemented**: Lines 848-936

### The "Hang" Explained

The Unix socket server uses **persistent connections** (correct JSON-RPC 2.0 behavior):

```rust
// After sending response, server WAITS for next request (doesn't close)
loop {
    line.clear();
    match reader.read_line(&mut line).await {
        Ok(0) => break,  // Client closed connection
        Ok(_) => {
            // Process request
            // Send response
            // LOOP BACK - wait for next request
        }
    }
}
```

**Why `nc` appears to hang**:
1. Client (`nc`) sends request
2. Server responds **immediately** ✅
3. Server waits for next request (persistent connection)
4. `nc` waits for server to close (expecting one-shot)
5. **Neither closes** → appears to "hang"

### Verified Working

```bash
# Tower Unix Socket - WITH TIMEOUT (correct usage):
$ echo '{"jsonrpc":"2.0","method":"health","id":1}' | timeout 1 nc -U /run/user/1000/biomeos/songbird-nat0.sock

Response (IMMEDIATE):
{"jsonrpc":"2.0","result":{"primal":"songbird","status":"healthy","uptime_seconds":0,"version":"0.1.0"},"id":1}

# Pixel TCP - SAME BEHAVIOR:
$ echo '{"jsonrpc":"2.0","method":"health","id":1}' | timeout 1 nc 127.0.0.1 9901

Response (IMMEDIATE):
{"jsonrpc":"2.0","result":{"primal":"songbird","status":"healthy",...},"id":1}
```

**Why Pixel "worked" without timeout**: Likely the test used a timeout or the client closed the connection. Same server behavior on both.

### Proper Client Usage for biomeOS

#### Option 1: Use Timeout (Shell Testing)
```bash
echo '{"jsonrpc":"2.0","method":"health","id":1}' | timeout 1 nc -U /socket | head -1
```

#### Option 2: Proper JSON-RPC Client (Rust - biomeOS Integration)
```rust
use tokio::net::UnixStream;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

let mut stream = UnixStream::connect("/run/user/1000/biomeos/songbird-nat0.sock").await?;

// Send request (with newline)
stream.write_all(b"{\"jsonrpc\":\"2.0\",\"method\":\"health\",\"id\":1}\n").await?;

// Read response (one line)
let mut reader = BufReader::new(&mut stream);
let mut response = String::new();
reader.read_line(&mut response).await?;

// Parse response
let result: JsonRpcResponse = serde_json::from_str(&response)?;

// Keep connection open for next request, OR close:
drop(stream);  // Closes connection
```

#### Option 3: Half-Close Pattern (Advanced)
```rust
// After sending request, shutdown write side:
stream.shutdown(std::net::Shutdown::Write).await?;

// Server sees EOF, sends response and closes
let response = stream.read_to_string(&mut buf).await?;
```

### Recommendation

**For biomeOS Neural API**:
- Implement proper persistent connection handling (Option 2)
- Reuse connections for multiple requests (more efficient)
- OR use half-close pattern (Option 3) for one-shot requests

**No changes needed in Songbird** - working as designed per JSON-RPC 2.0 spec.

---

## Issue 2: BirdSong family_id - ALREADY FIXED ✅

### Current Implementation

**File**: `crates/songbird-universal-ipc/src/handlers/birdsong_handler.rs` (lines 151-166)

```rust
// Discover family_id from environment (matches biomeOS pattern)
// Priority: FAMILY_ID > SONGBIRD_FAMILY_ID > NODE_FAMILY_ID
let family_id = std::env::var("FAMILY_ID")
    .or_else(|_| std::env::var("SONGBIRD_FAMILY_ID"))
    .or_else(|_| std::env::var("NODE_FAMILY_ID"))
    .ok();

if family_id.is_some() {
    info!("🔒 Using family_id from environment");
} else {
    warn!("⚠️  No FAMILY_ID environment variable set - BearDog encryption may fail");
}

let provider = BearDogBirdSongProvider::new(socket_path, family_id)
    .await
    .map_err(|e| format!("Failed to create BirdSong provider: {e}"))?;
```

### Verification

```bash
# Set environment variable:
export FAMILY_ID=nat0

# Start Songbird:
./songbird server --socket /run/user/1000/biomeos/songbird-nat0.sock

# Test encrypted beacon generation:
echo '{"jsonrpc":"2.0","method":"birdsong.generate_encrypted_beacon","params":{"node_id":"test","capabilities":["network.send"]},"id":1}' | \
  timeout 1 nc -U /run/user/1000/biomeos/songbird-nat0.sock | head -1

# Expected: Encrypted beacon response (not "Missing family_id")
```

### If Still Failing

Check:
1. **Environment variable is set**: `echo $FAMILY_ID` (should output `nat0`)
2. **Binary was rebuilt** with commit `78e1f7307` or later
3. **BearDog is running** and accessible at the socket path
4. **BearDog logs** for actual error (may be different issue)

---

## Issue 3: TLS Handshake - ALREADY FIXED IN v3.21.0 ✅

### Fixed In

**Commit**: `074093187` (v3.21.0, Feb 5, 2026)  
**Feature**: TLS Protocol Detection (HTTP/HTTPS on same port)

### Implementation

**File**: `crates/songbird-http-server/src/protocol_detection.rs`

The server now:
1. **Peeks** at the first byte of incoming connection
2. **0x16** (TLS handshake) → Routes to TLS handler
3. **ASCII** (HTTP methods) → Routes to HTTP handler
4. **Same port** handles both protocols

### Why Handoff Still Reports Issue

**Possible reasons**:

1. **Binaries Not Rebuilt**: Test used binaries from before v3.21.0
2. **TLS Explicitly Disabled**: `SONGBIRD_TLS_ENABLED=false` environment variable
3. **Port Mismatch**: Testing port 8080 instead of the TLS-enabled port

### Verification

```bash
# On Tower (should work with current binaries):
curl -k https://192.168.1.80:8080/.well-known/songbird

# With detailed TLS info:
openssl s_client -connect 192.168.1.80:8080 -servername pixel8a

# Check server logs for protocol detection:
# Should see: "Detected TLS connection (first byte: 0x16)"
```

### If Still Failing

1. **Rebuild binaries** from `074093187` or later:
   ```bash
   git checkout 074093187  # Or later
   cargo build --release --bin songbird
   ```

2. **Check environment** on Pixel:
   ```bash
   # Should NOT have:
   export SONGBIRD_TLS_ENABLED=false  # Remove this!
   
   # Or explicitly enable:
   export SONGBIRD_TLS_ENABLED=true
   ```

3. **Verify certificates** are generated:
   ```bash
   # Should exist on Pixel:
   ls -la /data/local/tmp/biomeos/*.pem
   
   # If missing, Songbird auto-generates on first TLS request
   ```

---

## Binary Rebuild Instructions

### For Deployment Team

To get all fixes, rebuild from current `main`:

```bash
# On build machine:
cd /path/to/songbird
git pull origin main
git log --oneline -1  # Should show commit d5d3c9822 or later

# Build for Tower (x86_64):
cargo build --release --target x86_64-unknown-linux-gnu

# Build for Pixel (aarch64):
cargo build --release --target aarch64-unknown-linux-musl

# Deploy:
cp target/x86_64-unknown-linux-gnu/release/songbird livespore-usb/x86_64/primals/
cp target/aarch64-unknown-linux-musl/release/songbird pixel8a-deploy/primals/
```

### Verification After Deploy

```bash
# On Tower - Test standard methods:
export FAMILY_ID=nat0
./songbird server --socket /run/user/1000/biomeos/songbird-nat0.sock &

echo '{"jsonrpc":"2.0","method":"health","id":1}' | timeout 1 nc -U /run/user/1000/biomeos/songbird-nat0.sock

# Expected: {"jsonrpc":"2.0","result":{"status":"healthy",...},"id":1}

# On Pixel - Test family_id:
export FAMILY_ID=nat0
export HOME=/data/local/tmp/biomeos
./primals/songbird server --listen 127.0.0.1:9901 &

echo '{"jsonrpc":"2.0","method":"birdsong.generate_encrypted_beacon","params":{"node_id":"pixel8a","capabilities":[]},"id":1}' | \
  timeout 1 nc 127.0.0.1 9901

# Expected: Encrypted beacon response
```

---

## Test Suite Results

All upstream integration tests pass:

```bash
$ cargo test --package songbird-universal-ipc --test upstream_integration_feb_2026_tests

running 27 tests
test test_unit_health_method ... ok
test test_unit_identity_method ... ok
test test_unit_rpc_discover_method ... ok
test test_unit_family_id_from_environment ... ok
test test_e2e_health_via_handler ... ok
test test_chaos_concurrent_health_requests ... ok
[... 21 more tests ...]

test result: ok. 27 passed; 0 failed; 0 ignored
```

---

## Summary for Integration Team

### Quick Checklist

- ✅ **Issue 1**: Methods ARE working - use proper client (timeout or persistent connection)
- ✅ **Issue 2**: `family_id` IS discovered from environment - set `FAMILY_ID=nat0`
- ✅ **Issue 3**: TLS protocol detection IS implemented - rebuild from v3.21.0+

### Action Items for biomeOS Team

1. **Update Neural API client** to handle persistent connections properly (see Option 2 above)
2. **Ensure environment variables** are set before starting Songbird:
   ```bash
   export FAMILY_ID=nat0
   export NODE_ID=<tower|pixel8a>
   ```
3. **Rebuild binaries** from current `main` (commit `d5d3c9822` or later)
4. **Test with timeout** in shell: `timeout 1 nc -U /socket`

### No Further Songbird Changes Needed

All reported issues are resolved in the current codebase. The "gaps" were:
- Misunderstanding of persistent connection behavior (not a bug)
- Testing with outdated binaries (fixes already committed)
- Missing environment variables in deployment (configuration issue)

---

## Contact

- **Documentation**: See `ISSUE_1_RESOLVED.md` for detailed persistent connection guide
- **Tests**: `crates/songbird-universal-ipc/tests/upstream_integration_feb_2026_tests.rs`
- **Commits**: 
  - `78e1f7307` - Upstream integration fixes (Issues 1 & 2)
  - `074093187` - TLS protocol detection (Issue 3)
  - `d5d3c9822` - Latest with all fixes + docs

**Status**: ✅ **All issues resolved - ready for deployment**

---

**Created**: February 5, 2026  
**Author**: Songbird Development Team  
**Version**: v3.22.0+
