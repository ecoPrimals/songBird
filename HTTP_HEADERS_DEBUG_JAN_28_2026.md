# HTTP Headers Debugging - January 28, 2026

**Date**: January 28, 2026 (Evening)  
**Issue**: HTTP headers not reaching server (Squirrel AI integration blocked)  
**Status**: 🔍 DEBUG LOGGING ADDED - Ready for testing

---

## Problem Summary

### Issue 1: `http.post` Loses Headers at IPC Layer

**Symptom**: When using `http.post`, headers are empty:
```
handle_post → HttpRequestParams { ..., headers: {}, ... }  // EMPTY!
```

**Expected**: Headers should be forwarded to actual HTTP request.

### Issue 2: `http.request` Receives Headers but Server Returns 401

**Symptom**: `http.request` shows correct headers in logs, but Anthropic returns 401:
```
handle_request → HttpRequestParams { ..., headers: {"x-api-key": "..."}, ... }  // CORRECT!
BUT server returns: 401 "x-api-key header is required"
```

**Root Cause**: Headers received by IPC handler but NOT included in actual HTTP request.

---

## Debug Logging Added

### Layer 1: IPC Handler (Entry Point)

**File**: `crates/songbird-orchestrator/src/ipc/handlers/http.rs`

**Logging Points**:

1. **`handle_post` entry** (Line 287):
   ```rust
   tracing::info!("🔍 handle_post → incoming params: {}", params);
   tracing::info!("🔍 handle_post → modified params: {}", req_params);
   ```
   **What to check**: Are headers present in the incoming params? Are they preserved after adding method="POST"?

2. **`handle_request` entry** (Line 183):
   ```rust
   tracing::info!("🔍 handle_request → params: {}", params);
   tracing::info!("🔍 handle_request → method: {}, url: {}, headers: {:?}", method_str, url, headers);
   ```
   **What to check**: Are headers successfully parsed from params into the HashMap?

### Layer 2: HTTP Client (Request Building)

**File**: `crates/songbird-http-client/src/request.rs`

**Logging Points**:

1. **Header merging** (Lines 70-71):
   ```rust
   tracing::info!("🔍 RequestBuilder → caller_headers: {:?}", caller_headers);
   tracing::info!("🔍 RequestBuilder → merged_headers: {:?}", headers);
   ```
   **What to check**: Are caller_headers being passed correctly? Are they preserved in merged_headers?

2. **Header writing** (Line 119):
   ```rust
   tracing::debug!("🔍 Writing header: {}: {}", key, value);
   ```
   **What to check**: Is each header actually being written to the request?

3. **Final HTTP request** (Lines 83-94):
   ```rust
   tracing::info!("🔍 Final HTTP request ({} lines):", lines.len());
   for (i, line) in lines.iter().enumerate() {
       tracing::info!("  {}: {}", i+1, line);
   }
   ```
   **What to check**: Are headers present in the final HTTP request bytes?

---

## Test Commands

### Test Issue 1: `http.post` (Headers Lost?)

```bash
# Start Songbird with verbose logging
RUST_LOG=info ./target/release/songbird server \
    --socket /tmp/songbird-debug.sock \
    --port 8080 &

# Wait for startup
sleep 3

# Send http.post with headers
echo '{"jsonrpc":"2.0","method":"http.post","params":{"url":"https://api.anthropic.com/v1/messages","headers":{"x-api-key":"test-key","content-type":"application/json"},"body":"e30="},"id":1}' | nc -U /tmp/songbird-debug.sock

# Check logs for:
# ✅ handle_post → incoming params: {headers: ...}
# ✅ handle_post → modified params: {headers: ..., method: POST}
# ✅ handle_request → headers: {...}
# ✅ RequestBuilder → caller_headers: {...}
# ✅ RequestBuilder → merged_headers: {...}
# ✅ Writing header: x-api-key: test-key
# ✅ Final HTTP request: ... x-api-key: test-key ...
```

### Test Issue 2: `http.request` (Headers Not Sent?)

```bash
# Send http.request with headers (explicit method)
echo '{"jsonrpc":"2.0","method":"http.request","params":{"method":"POST","url":"https://api.anthropic.com/v1/messages","headers":{"x-api-key":"test-key","content-type":"application/json"},"body":"e30="},"id":1}' | nc -U /tmp/songbird-debug.sock

# Check logs for same points as above
```

### Test with Real API (If You Have Key)

```bash
# Replace sk-xxx with actual Anthropic API key
API_KEY="sk-ant-xxx"

# Create proper request body
REQUEST_BODY='{"model":"claude-3-haiku-20240307","messages":[{"role":"user","content":"Hello"}],"max_tokens":10}'
BODY_BASE64=$(echo -n "$REQUEST_BODY" | base64)

# Send request
echo "{\"jsonrpc\":\"2.0\",\"method\":\"http.post\",\"params\":{\"url\":\"https://api.anthropic.com/v1/messages\",\"headers\":{\"x-api-key\":\"$API_KEY\",\"content-type\":\"application/json\",\"anthropic-version\":\"2023-06-01\"},\"body\":\"$BODY_BASE64\"},\"id\":1}" | nc -U /tmp/songbird-debug.sock
```

---

## Expected Log Flow (Success Case)

```
INFO  songbird_orchestrator::ipc::handlers::http: 🔍 handle_post → incoming params: {"url":"...","headers":{"x-api-key":"test-key"},"body":"..."}
INFO  songbird_orchestrator::ipc::handlers::http: 🔍 handle_post → modified params: {"url":"...","headers":{"x-api-key":"test-key"},"method":"POST","body":"..."}
INFO  songbird_orchestrator::ipc::handlers::http: 🔍 handle_request → params: {"url":"...","headers":{"x-api-key":"test-key"},"method":"POST","body":"..."}
INFO  songbird_orchestrator::ipc::handlers::http: 🔍 handle_request → method: POST, url: https://..., headers: {"x-api-key": "test-key"}
INFO  songbird_http_client: 🌐 HTTP POST https://...
INFO  songbird_http_client::request: 🔍 RequestBuilder → caller_headers: {"x-api-key": "test-key"}
INFO  songbird_http_client::request: 🔍 RequestBuilder → merged_headers: {"x-api-key": "test-key", "User-Agent": "...", ...}
DEBUG songbird_http_client::request: 🔍 Writing header: x-api-key: test-key
INFO  songbird_http_client::request: 🔍 Final HTTP request (10 lines):
INFO  songbird_http_client::request:   1: POST /v1/messages HTTP/1.1
INFO  songbird_http_client::request:   2: Host: api.anthropic.com
INFO  songbird_http_client::request:   3: User-Agent: ...
INFO  songbird_http_client::request:   4: content-type: application/json
INFO  songbird_http_client::request:   5: x-api-key: test-key
INFO  songbird_http_client::request:   6: Content-Length: ...
INFO  songbird_http_client::request:   7: 
INFO  songbird_http_client::request:   8: {...}
```

---

## Failure Scenarios to Look For

### Scenario A: Headers Lost at IPC Parsing

**Log Pattern**:
```
INFO: handle_post → incoming params: {"headers":{"x-api-key":"test-key"}}  ✅ Present
INFO: handle_request → headers: {}  ❌ EMPTY
```

**Root Cause**: Header parsing logic in `handle_request` is broken.

**Fix Location**: `crates/songbird-orchestrator/src/ipc/handlers/http.rs` lines 194-202

### Scenario B: Headers Lost at HTTP Client Call

**Log Pattern**:
```
INFO: handle_request → headers: {"x-api-key": "test-key"}  ✅ Present
INFO: RequestBuilder → caller_headers: {}  ❌ EMPTY
```

**Root Cause**: Headers not passed to `client.request()`.

**Fix Location**: `crates/songbird-orchestrator/src/ipc/handlers/http.rs` line 237

### Scenario C: Headers Lost During Merge

**Log Pattern**:
```
INFO: RequestBuilder → caller_headers: {"x-api-key": "test-key"}  ✅ Present
INFO: RequestBuilder → merged_headers: {"User-Agent": "..."}  ❌ Missing x-api-key
```

**Root Cause**: `headers_for_domain()` not merging caller headers correctly.

**Fix Location**: `crates/songbird-http-client/src/http_config.rs` line 318

### Scenario D: Headers Lost During Writing

**Log Pattern**:
```
INFO: RequestBuilder → merged_headers: {"x-api-key": "test-key"}  ✅ Present
INFO: Final HTTP request:
  1: POST /v1/messages HTTP/1.1
  2: Host: api.anthropic.com
  3: User-Agent: ...
  [NO x-api-key line]  ❌ Missing
```

**Root Cause**: `write_headers()` filtering out certain headers incorrectly.

**Fix Location**: `crates/songbird-http-client/src/request.rs` lines 112-124

---

## Quick Diagnosis Script

```bash
#!/bin/bash
# File: test_headers.sh

echo "🧪 Testing HTTP Header Propagation"
echo "=================================="

# Start Songbird
./target/release/songbird server --socket /tmp/songbird-test.sock --port 8080 2>&1 | tee /tmp/songbird-debug.log &
SONGBIRD_PID=$!
sleep 3

# Send test request
echo '{"jsonrpc":"2.0","method":"http.post","params":{"url":"https://httpbin.org/post","headers":{"X-Test-Header":"test-value"},"body":"e30="},"id":1}' | nc -U /tmp/songbird-test.sock > /tmp/response.json

# Wait for response
sleep 2

# Kill Songbird
kill $SONGBIRD_PID
wait $SONGBIRD_PID 2>/dev/null

# Analyze logs
echo ""
echo "📊 Log Analysis:"
echo "---------------"

if grep -q "handle_post → incoming params.*X-Test-Header" /tmp/songbird-debug.log; then
    echo "✅ Layer 1 (IPC Entry): Headers present in incoming params"
else
    echo "❌ Layer 1 (IPC Entry): Headers MISSING in incoming params"
fi

if grep -q "handle_request → headers.*X-Test-Header" /tmp/songbird-debug.log; then
    echo "✅ Layer 2 (IPC Handler): Headers parsed correctly"
else
    echo "❌ Layer 2 (IPC Handler): Headers lost during parsing"
fi

if grep -q "RequestBuilder → caller_headers.*X-Test-Header" /tmp/songbird-debug.log; then
    echo "✅ Layer 3 (HTTP Client): Headers passed to client"
else
    echo "❌ Layer 3 (HTTP Client): Headers lost before client"
fi

if grep -q "Writing header: X-Test-Header" /tmp/songbird-debug.log; then
    echo "✅ Layer 4 (Request Build): Headers written to request"
else
    echo "❌ Layer 4 (Request Build): Headers not written"
fi

if grep -q "X-Test-Header.*test-value" /tmp/songbird-debug.log; then
    echo "✅ Layer 5 (Final Request): Headers in final HTTP bytes"
else
    echo "❌ Layer 5 (Final Request): Headers missing from HTTP bytes"
fi

# Cleanup
rm -f /tmp/songbird-test.sock /tmp/response.json
```

---

## Files Modified

| File | Lines | Change |
|------|-------|--------|
| `crates/songbird-orchestrator/src/ipc/handlers/http.rs` | 183, 187, 276-280 | Added debug logging for params and headers |
| `crates/songbird-http-client/src/request.rs` | 70-71, 119, 83-94 | Added debug logging for header merging and writing |

---

## Next Steps

1. **Run test commands** with debug logging enabled
2. **Identify which layer** is losing the headers
3. **Apply targeted fix** based on diagnosis
4. **Verify** with real Anthropic API call
5. **Remove debug logging** (or keep behind feature flag)

---

## Impact

**Current**: 🔴 Squirrel AI integration blocked (all AI API calls fail with 401)  
**After Fix**: 🟢 Squirrel can make authenticated API calls to Anthropic, OpenAI, etc.

**Blocked Use Cases**:
- Squirrel AI providers (Anthropic, OpenAI, HuggingFace)
- Any API requiring authentication headers
- Custom header-based routing

---

**Generated**: January 28, 2026 (Evening)  
**Version**: v8.14.0  
**Commit**: Pending (debug logging)  
**Status**: 🔍 READY FOR TESTING

🔬 **Run tests and report which layer is losing headers!** 🔬

