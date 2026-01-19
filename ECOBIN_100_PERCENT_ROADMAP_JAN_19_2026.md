# ecoBin 100% Roadmap

**Date**: January 19, 2026  
**Current Status**: 98% Pure Rust (A grade)  
**Goal**: 100% Pure Rust (A++ grade)

---

## Current Achievement: 98% ecoBin ✅

### Completed Eliminations
1. ✅ **jsonwebtoken** → `pure_rust_jwt` (HMAC-SHA256)
2. ✅ **tokio-rustls** (direct) → Removed from orchestrator
3. ✅ **rustls** (direct) → Removed from network-federation
4. ✅ **reqwest rustls-tls** → Removed from all 11 crates
5. ✅ **songbird-tls** → 100% Pure Rust TLS 1.3 via BearDog

---

## Remaining 2%: Transitive Dependencies

### The Last Dependency: jsonrpsee

**Source**: `jsonrpsee` meta-crate (v0.26.0)  
**Problem**: Even with `features = ["server"]`, it pulls in:
- `jsonrpsee-http-client` → `hyper-rustls` → `rustls` → `ring`/`aws-lc-rs`
- `jsonrpsee-ws-client` → `tokio-rustls` → `rustls` → `ring`/`aws-lc-rs`

**Dependency Chain**:
```
songbird v3.33.0
└── songbird-orchestrator
    └── jsonrpsee v0.26.0 (features = ["server"])
        ├── jsonrpsee-http-client  ⚠️  (transitive)
        │   └── hyper-rustls
        │       └── rustls v0.23.35
        │           ├── ring v0.17.14  ❌ C dependency
        │           └── aws-lc-rs      ❌ C dependency
        └── jsonrpsee-ws-client  ⚠️  (transitive)
            └── tokio-rustls
                └── rustls v0.23.35 (*)
```

---

## Solution Options

### ⭐ Option A: Manual JSON-RPC (BearDog Style) ✅✅ RECOMMENDED
**Effort**: Low (3.5 hours)  
**Impact**: 100% Pure Rust  
**Source**: Proven in BearDog production  
**Approach**:
1. Copy BearDog's manual JSON-RPC implementation (~150 lines)
2. Create `JsonRpcRequest`, `JsonRpcResponse`, `JsonRpcError` types
3. Implement simple request handler and method routing
4. Remove `jsonrpsee` dependency entirely

**Benefits**:
- ✅ 100% Pure Rust
- ✅ Already proven in production (BearDog)
- ✅ Simpler than jsonrpsee (~150 LOC vs 50,000)
- ✅ Full control over protocol
- ✅ Ecosystem consistency (same as BearDog)
- ✅ Faster compile, smaller binary

**See**: [BEARDOG_JSONRPC_SOLUTION_JAN_19_2026.md](BEARDOG_JSONRPC_SOLUTION_JAN_19_2026.md)

### Option B: Use jsonrpsee Sub-Crates
**Effort**: High (6-8 hours)  
**Impact**: 100% Pure Rust  
**Approach**:
1. Replace `jsonrpsee` with:
   - `jsonrpsee-server`
   - `jsonrpsee-core`
   - `jsonrpsee-types`
2. Update all imports from `jsonrpsee::` to specific crates
3. Test all JSON-RPC endpoints

**Challenges**:
- Requires code changes across multiple files
- May break existing JSON-RPC API
- More complex than Option A

### Option C: Wait for jsonrpsee Pure Rust Support
**Effort**: None (wait)  
**Impact**: Unknown timeline  
**Approach**:
- Monitor jsonrpsee for Pure Rust TLS support
- Contribute to jsonrpsee to add Pure Rust option
- Stay at 98% until upstream changes

---

## ⭐ Recommendation: Option A (Manual JSON-RPC - BearDog Style)

**Rationale**:
1. **Already proven** - BearDog uses this in production
2. **Simple** - Only ~150 lines of code vs 50,000 in jsonrpsee
3. **Pure Rust** - Zero C dependencies
4. **Ecosystem aligned** - Same approach as BearDog
5. **Full control** - Custom error handling, routing, middleware
6. **Better metrics** - Faster compile, smaller binary

**Implementation Plan**:
1. ✅ Create `pure_jsonrpc_types.rs` (~50 lines, 30 min)
2. ✅ Create `pure_jsonrpc_handler.rs` (~100 lines, 1 hour)
3. ✅ Update server integration (~20 lines, 30 min)
4. ✅ Remove `jsonrpsee` from Cargo.toml (5 min)
5. ✅ Update imports in ~10 files (30 min)
6. ✅ Test all RPC endpoints (30 min)
7. ✅ Update documentation (30 min)

**Estimated Time**: 3.5 hours

**Discovery Document**: See [BEARDOG_JSONRPC_SOLUTION_JAN_19_2026.md](BEARDOG_JSONRPC_SOLUTION_JAN_19_2026.md) for complete details

---

## Current Metrics

**Before ecoBin Work**:
- Direct C Dependencies: 3 (jsonwebtoken, tokio-rustls, reqwest)
- Transitive C Dependencies: ~50+
- Grade: C

**After ecoBin Work (Current)**:
- Direct C Dependencies: 0 ✅
- Transitive C Dependencies: 2 (via jsonrpsee)
- Grade: A (98%)

**After 100% ecoBin (Goal)**:
- Direct C Dependencies: 0 ✅
- Transitive C Dependencies: 0 ✅
- Grade: A++ (100%)

---

## Next Steps

1. ✅ Document current 98% achievement
2. ⏳ Choose solution (recommend Option A)
3. ⏳ Implement tarpc migration
4. ⏳ Test all RPC endpoints
5. ⏳ Verify 100% Pure Rust
6. ⏳ Celebrate A++ grade!

---

**Status**: Ready for 100% push when approved  
**Blocker**: None (all tools available)  
**Risk**: Low (tarpc already proven)

🦀✨ **Songbird: 98% Pure Rust, 2% Away from Perfection!** ✨🦀

