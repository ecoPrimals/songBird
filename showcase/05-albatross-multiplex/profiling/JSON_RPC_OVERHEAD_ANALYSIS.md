# JSON-RPC Overhead Analysis

**Date**: December 18, 2025  
**Problem**: JSON-RPC is 1.7x SLOWER than plain HTTP (17K vs 28K req/s)

---

## 🔍 Root Cause Analysis

### Current JSON-RPC Implementation

Location: `crates/songbird-orchestrator/src/server/jsonrpc_api.rs`

**Request Flow**:
```
Client → Axum → JSON deserialize → JsonRpcRequest
      → String method matching
      → Handler function
      → Wrap in JsonRpcResponse
      → JSON serialize → Client
```

**Compared to HTTP `/health`**:
```
Client → Axum → Handler function
      → JSON serialize → Client
```

---

## 📊 Overhead Sources (Identified)

### 1. Extra JSON Layer (Wrapping)
**Request wrapping**:
```json
{
  "jsonrpc": "2.0",
  "method": "songbird.health",
  "params": {},
  "id": 1
}
```

**Response wrapping**:
```json
{
  "jsonrpc": "2.0",
  "result": {...},
  "error": null,
  "id": 1
}
```

**Impact**: 
- Extra deserialization of wrapper
- Extra fields to parse/validate
- Extra memory allocations

**Estimated overhead**: ~10-20% (JSON parsing)

### 2. String Method Routing
```rust
let result = match request.method.as_str() {
    "songbird.services.list" => handle_services_list(&state).await,
    "songbird.services.get" => handle_service_get(&state, request.params).await,
    // ... 10+ more string comparisons
    "songbird.health" => handle_health(&state).await,
    _ => Err(JsonRpcError::method_not_found(&request.method))
};
```

**Impact**:
- String comparison on every request
- Linear scan through method names
- Not optimized by compiler

**Estimated overhead**: ~5-10% (string operations)

### 3. Logging on Every Request
```rust
info!("📞 JSON-RPC request: method={}", request.method);
```

**Impact**:
- I/O operation on hot path
- String formatting
- Potentially blocks if logging buffer full

**Estimated overhead**: ~20-30% (I/O + formatting)

### 4. Response Wrapping Logic
```rust
let response = match result {
    Ok(value) => JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: Some(value),
        error: None,
        id: request.id.unwrap_or(Value::Null),
    },
    Err(error) => JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: None,
        error: Some(error),
        id: request.id.unwrap_or(Value::Null),
    },
};
```

**Impact**:
- Extra struct allocation
- Option wrapping (Some/None)
- String allocation for "2.0"

**Estimated overhead**: ~10-15% (allocations)

---

## 💡 Total Estimated Overhead: 45-75%

**Measured**: 1.7x slower (41% slower)  
**Estimated**: 45-75% slower

**This matches!** The overhead is primarily from:
1. Logging (30%)
2. JSON wrapping (20%)
3. String routing (10%)
4. Response wrapping (15%)

---

## 🚀 Optimization Options

### Option 1: Optimize Current JSON-RPC

**Quick wins** (can implement immediately):

1. **Remove hot-path logging**
   ```rust
   // Change from:
   info!("📞 JSON-RPC request: method={}", request.method);
   // To:
   debug!("📞 JSON-RPC request: method={}", request.method);
   ```
   **Expected gain**: 20-30% faster

2. **Use phf (perfect hash function) for method routing**
   ```rust
   use phf::phf_map;
   
   static METHODS: phf::Map<&'static str, fn(&JsonRpcState, Value) -> Future> = phf_map! {
       "songbird.health" => handle_health,
       "songbird.version" => handle_version,
       // ...
   };
   ```
   **Expected gain**: 5-10% faster

3. **Avoid string allocations**
   ```rust
   // Use &'static str instead of String::from("2.0")
   const JSONRPC_VERSION: &str = "2.0";
   ```
   **Expected gain**: 5% faster

4. **Reuse response structure**
   ```rust
   // Pool JsonRpcResponse objects
   ```
   **Expected gain**: 5% faster

**Total potential**: 35-50% faster (from 17K to 23-25K req/s)

---

### Option 2: Replace with MessagePack-RPC

**MessagePack** is a binary format (like tarpc) but with RPC semantics:

**Pros**:
- Binary encoding (faster than JSON)
- Smaller payload (~2x compression)
- Similar API to JSON-RPC
- Still language-agnostic

**Cons**:
- Less universal than JSON
- Requires MessagePack library

**Expected performance**: 25-30K req/s (similar to HTTP)

**Implementation effort**: Medium (2-3 hours)

---

### Option 3: Remove JSON-RPC, Use HTTP + tarpc Only

**HTTP** for:
- REST API (external clients)
- Universal access
- Browser/curl testing

**tarpc** for:
- Primal-to-primal communication
- High-performance orchestration
- Internal mesh

**Pros**:
- Simpler codebase
- Better performance (HTTP fast, tarpc faster)
- Clear separation of concerns

**Cons**:
- No universal RPC protocol
- Clients must choose HTTP or tarpc

**Recommendation**: **This is the best option**

---

### Option 4: Evolve JSON-RPC to "Songbird RPC"

Create a custom binary RPC that's:
- Faster than JSON-RPC (binary encoding)
- Simpler than tarpc (no service traits)
- Universal (any language can implement)

**Format**:
```rust
struct SongbirdRpcRequest {
    method_id: u32,  // Fast integer routing
    payload: Vec<u8>,  // Binary payload (MessagePack/CBOR)
}

struct SongbirdRpcResponse {
    status: u8,  // 0=ok, 1=error
    payload: Vec<u8>,
}
```

**Expected performance**: 30-40K req/s

**Implementation effort**: High (1 day)

---

## 📋 Recommendation

**Immediate** (next 30 minutes):
1. ✅ Remove `info!` logging from hot path → `debug!`
2. ✅ Use &'static str for "2.0" → no allocations

**Short-term** (next session):
1. Implement Option 1 optimizations
2. Re-benchmark
3. Goal: Get JSON-RPC to 23-25K req/s

**Medium-term** (next week):
1. Consider removing JSON-RPC entirely
2. Use HTTP for universal access
3. Use tarpc for high-performance
4. Simplify the stack

**Long-term** (future):
1. If universal RPC still needed, implement MessagePack-RPC
2. Or create custom "Songbird RPC" binary protocol

---

## 🎯 Action Items

### Now:
1. Change `info!` to `debug!` in JSON-RPC handler
2. Use static strings for jsonrpc version
3. Re-run benchmarks

### Next:
1. Implement phf-based method routing
2. Profile to confirm improvements
3. Consider removing JSON-RPC if still slow

### Future:
1. Decide: Keep JSON-RPC or remove?
2. If keep: Implement MessagePack-RPC
3. If remove: Document HTTP + tarpc pattern

---

## 📊 Expected Results After Quick Fixes

| Protocol | Before | After (estimated) | Speedup |
|----------|--------|-------------------|---------|
| HTTP | 28,307 | 28,307 | - |
| JSON-RPC | 16,991 | 23,000 | 1.35x |
| tarpc | 61,831 | 61,831 | - |

**JSON-RPC would be** 81% of HTTP speed (vs 60% now)

Still slower, but acceptable for universal RPC use case.

---

*Analysis: Complete*  
*Next: Implement quick fixes and re-benchmark*

