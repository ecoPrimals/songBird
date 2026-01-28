# Squirrel AI HTTP Headers Investigation - Handoff

**Date**: January 28, 2026 (Evening → Late Evening)  
**Priority**: 🔴 **HIGH** → 🟢 **RESOLVED**  
**Status**: ✅ **FIX COMPLETE** - Root cause found and fixed  
**Commits**: `79bbd3371` (debug), `a6d702dcd` (fix)

---

## ✅ **FIX COMPLETE!** (Jan 28, 2026 - Late Evening)

### Root Cause Found by biomeOS Team

**Location**: `songbird-universal-ipc/src/handlers/http_handler.rs` (NOT `songbird-orchestrator`)

**The Bug**:
1. `http.post` match arm (lines 410-424): Did not extract headers from params
2. `handle_post` method (lines 359-382): Created empty HashMap for headers

**The Fix** (Commit `a6d702dcd`):
1. Updated `handle_post` signature to accept `caller_headers` parameter
2. Updated `http.post` match arm to extract headers from params
3. Updated `service.rs` to extract and pass headers
4. Build verified: ✅ Clean release (56.47s)
5. Tests verified: ✅ 40 passing

**Impact**: 🟢 Squirrel AI integration **UNBLOCKED**

📖 **See [HTTP_HEADERS_FIX_COMPLETE_JAN_28_2026.md](HTTP_HEADERS_FIX_COMPLETE_JAN_28_2026.md) for complete details**

---

## Executive Summary (Historical)

**Problem**: HTTP headers not reaching servers when Squirrel calls Songbird's HTTP methods.

**Impact**: All Squirrel AI API calls (Anthropic, OpenAI, HuggingFace) fail with 401 Unauthorized.

**Solution**: Added comprehensive debug logging across 5 layers to pinpoint where headers are lost.

**Next Step**: Run test commands to identify the exact layer losing headers, then apply targeted fix.

---

## Issues Identified

### Issue 1: `http.post` Loses Headers at IPC Layer

**Symptom**:
```json
// Request sent by Squirrel
{"jsonrpc":"2.0","method":"http.post","params":{"url":"...","headers":{"x-api-key":"sk-xxx"},"body":"..."}}

// Received by handle_post
{"headers": {}}  // ❌ EMPTY!
```

**Hypothesis**: Headers being lost during JSON-RPC parameter parsing or `http.post` → `http.request` forwarding.

### Issue 2: `http.request` Receives Headers but Server Returns 401

**Symptom**:
```
INFO: handle_request → headers: {"x-api-key": "sk-xxx"}  ✅ Present in IPC handler
ERROR: Server returned 401: "x-api-key header is required"  ❌ Not in HTTP request
```

**Hypothesis**: Headers received by IPC handler but not included in actual HTTP request bytes sent over the wire.

---

## Debug Logging Added

### 5-Layer Tracing Architecture

```
┌────────────────────────────────────────────────────────────┐
│ Layer 1: IPC Entry (http.post convenience method)         │
│ File: crates/songbird-orchestrator/src/ipc/handlers/http.rs│
│ Logs: Incoming params, modified params with method=POST   │
└─────────────────────┬──────────────────────────────────────┘
                      │
┌─────────────────────▼──────────────────────────────────────┐
│ Layer 2: IPC Handler (handle_request)                     │
│ File: crates/songbird-orchestrator/src/ipc/handlers/http.rs│
│ Logs: Parsed method, url, headers HashMap                 │
└─────────────────────┬──────────────────────────────────────┘
                      │
┌─────────────────────▼──────────────────────────────────────┐
│ Layer 3: HTTP Client (request builder initialization)     │
│ File: crates/songbird-http-client/src/request.rs          │
│ Logs: caller_headers received, merged_headers after config│
└─────────────────────┬──────────────────────────────────────┘
                      │
┌─────────────────────▼──────────────────────────────────────┐
│ Layer 4: Header Writing (write_headers loop)              │
│ File: crates/songbird-http-client/src/request.rs          │
│ Logs: Each individual header being written                │
└─────────────────────┬──────────────────────────────────────┘
                      │
┌─────────────────────▼──────────────────────────────────────┐
│ Layer 5: Final HTTP Request (raw bytes)                   │
│ File: crates/songbird-http-client/src/request.rs          │
│ Logs: Complete HTTP request (first 20 lines)              │
└────────────────────────────────────────────────────────────┘
```

---

## Quick Test

### Start Songbird with Debug Logging

```bash
cd /home/eastgate/Development/ecoPrimals/phase1/songbird

# Start with verbose logging
RUST_LOG=info ./target/release/songbird server \
    --socket /tmp/songbird-debug.sock \
    --port 8080 \
    2>&1 | tee /tmp/songbird-headers.log &

# Wait for startup
sleep 3
```

### Test Issue 1: `http.post` (Most Likely Culprit)

```bash
# Send http.post with headers
echo '{"jsonrpc":"2.0","method":"http.post","params":{"url":"https://httpbin.org/post","headers":{"X-Test":"value","Content-Type":"application/json"},"body":"e30="},"id":1}' \
| nc -U /tmp/songbird-debug.sock
```

### Test Issue 2: `http.request`

```bash
# Send http.request with headers
echo '{"jsonrpc":"2.0","method":"http.request","params":{"method":"POST","url":"https://httpbin.org/post","headers":{"X-Test":"value","Content-Type":"application/json"},"body":"e30="},"id":1}' \
| nc -U /tmp/songbird-debug.sock
```

### Check Logs for Header Propagation

```bash
# Should see 5 layers of logging
grep "🔍" /tmp/songbird-headers.log

# Expected output (success):
# INFO: 🔍 handle_post → incoming params: {..."headers":{"X-Test":"value"}...}
# INFO: 🔍 handle_post → modified params: {..."headers":{"X-Test":"value"}..."method":"POST"}
# INFO: 🔍 handle_request → headers: {"X-Test": "value"}
# INFO: 🔍 RequestBuilder → caller_headers: {"X-Test": "value"}
# INFO: 🔍 RequestBuilder → merged_headers: {"X-Test": "value", "User-Agent": "..."}
# DEBUG: 🔍 Writing header: X-Test: value
# INFO: 🔍 Final HTTP request (10 lines):
# INFO:   5: X-Test: value
```

---

## Diagnosis Decision Tree

```
Start: Headers sent by Squirrel
  ↓
Layer 1: handle_post incoming params
  ├─ ✅ Headers present → Continue to Layer 2
  └─ ❌ Headers missing → **FIX: JSON-RPC parsing**
     Location: crates/songbird-orchestrator/src/ipc/pure_rust_server/server.rs
     Issue: Params not forwarded correctly from JSON-RPC to handler
  
Layer 2: handle_request parsed headers
  ├─ ✅ Headers present → Continue to Layer 3
  └─ ❌ Headers missing → **FIX: IPC handler parsing**
     Location: crates/songbird-orchestrator/src/ipc/handlers/http.rs:194-202
     Issue: Header extraction from params broken
  
Layer 3: RequestBuilder caller_headers
  ├─ ✅ Headers present → Continue to Layer 4
  └─ ❌ Headers missing → **FIX: HTTP client call**
     Location: crates/songbird-orchestrator/src/ipc/handlers/http.rs:237
     Issue: Headers not passed to client.request()
  
Layer 4: Header writing (debug logs)
  ├─ ✅ Headers logged → Continue to Layer 5
  └─ ❌ Headers not logged → **FIX: Header merge**
     Location: crates/songbird-http-client/src/http_config.rs:318
     Issue: headers_for_domain() not merging caller headers
  
Layer 5: Final HTTP request bytes
  ├─ ✅ Headers in request → **MYSTERY: Server rejecting valid headers?**
  │   → Check TLS encryption, chunking, or server-side logging
  └─ ❌ Headers missing → **FIX: Header writing**
     Location: crates/songbird-http-client/src/request.rs:112-124
     Issue: write_headers() filtering out headers incorrectly
```

---

## Files You May Need to Fix

Based on layer that fails:

| Layer | File | Lines | What to Fix |
|-------|------|-------|-------------|
| 1 | `crates/songbird-orchestrator/src/ipc/pure_rust_server/server.rs` | 399-401 | JSON-RPC params forwarding |
| 2 | `crates/songbird-orchestrator/src/ipc/handlers/http.rs` | 194-202 | Header parsing from params |
| 3 | `crates/songbird-orchestrator/src/ipc/handlers/http.rs` | 237 | Passing headers to HTTP client |
| 4 | `crates/songbird-http-client/src/http_config.rs` | 293-321 | Header merging logic |
| 5 | `crates/songbird-http-client/src/request.rs` | 112-124 | Header writing loop |

---

## After Diagnosis

### If Fix is Straightforward

1. Apply fix to identified layer
2. Rebuild: `cargo build --release`
3. Re-run test commands
4. Verify with real Anthropic API call
5. Remove debug logging (or keep behind `SONGBIRD_DEBUG_HEADERS` env var)

### If Fix is Complex

1. Report which layer is failing
2. Share relevant log snippet
3. Pair with Songbird team for targeted fix

---

## Testing with Real Anthropic API

Once headers are flowing correctly:

```bash
# Replace sk-xxx with real Anthropic API key
API_KEY="sk-ant-REDACTED"

# Create Claude request
REQUEST='{"model":"claude-3-haiku-20240307","messages":[{"role":"user","content":"Hello"}],"max_tokens":10}'
BODY_B64=$(echo -n "$REQUEST" | base64 -w 0)

# Send via Songbird
echo "{\"jsonrpc\":\"2.0\",\"method\":\"http.post\",\"params\":{\"url\":\"https://api.anthropic.com/v1/messages\",\"headers\":{\"x-api-key\":\"$API_KEY\",\"content-type\":\"application/json\",\"anthropic-version\":\"2023-06-01\"},\"body\":\"$BODY_B64\"},\"id\":1}" \
| nc -U /tmp/songbird-debug.sock

# Expected success:
# {"jsonrpc":"2.0","result":{"status":200,"headers":{...},"body":"<base64>"},"id":1}

# If still 401:
# - Check API key is valid
# - Check header names (case-sensitive!)
# - Check Anthropic API version
```

---

## Removal of Debug Logging (Future)

After fix is confirmed, we can:

**Option 1**: Remove debug logging entirely  
**Option 2**: Keep behind feature flag:
```rust
#[cfg(feature = "debug-headers")]
tracing::info!("🔍 ...");
```

**Option 3**: Keep behind env var:
```rust
if std::env::var("SONGBIRD_DEBUG_HEADERS").is_ok() {
    tracing::info!("🔍 ...");
}
```

Recommend **Option 3** for production debugging.

---

## Estimated Time to Resolution

| Scenario | Time |
|----------|------|
| **Best Case**: Headers just need base64 decoding | 1 hour |
| **Likely Case**: Logic bug in one layer | 2-4 hours |
| **Worst Case**: Architectural issue (e.g. TLS encryption stripping headers) | 1-2 days |

---

## Contact

**For pairing**: Songbird team available for real-time debugging  
**For async**: Run tests, share logs from `/tmp/songbird-headers.log`  
**For escalation**: This blocks all Squirrel AI providers (HIGH priority)

---

## Documentation

- Full test guide: `HTTP_HEADERS_DEBUG_JAN_28_2026.md` (355 lines)
- Test script: Included in debug doc
- Log analysis: Automated script in debug doc

---

## Summary

✅ **Debug logging complete** (5 layers, 40+ new log points)  
✅ **Build verified** (release mode, 1m 24s)  
✅ **Test commands ready** (httpbin.org + Anthropic examples)  
✅ **Diagnosis guide** (decision tree + fix locations)  
🔴 **Next: Run tests and report which layer fails**

---

**Generated**: January 28, 2026 (Evening)  
**Commit**: `79bbd3371`  
**Status**: 🔍 READY FOR TESTING  
**Blocks**: Squirrel AI integration (HIGH priority)

🚀 **Run the quick test commands above and report results!** 🚀

