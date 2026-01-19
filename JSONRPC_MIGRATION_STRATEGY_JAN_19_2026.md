# JSON-RPC Migration Strategy

**Date**: January 19, 2026  
**Goal**: Complete migration to Pure Rust JSON-RPC  
**Status**: Implementation paused for strategic decision

---

## Current Situation

### ✅ Completed
1. Created Pure Rust JSON-RPC types (~311 lines)
2. Created Pure Rust JSON-RPC handler (~335 lines)
3. Removed `jsonrpsee` dependency from Cargo.toml
4. Total Pure Rust code: ~646 lines

### ⏸️  Blocked
- IPC handlers extensively use `jsonrpsee::types` (88 references)
- Need compatibility layer or full rewrite

---

## Discovered Integration Depth

`jsonrpsee` is deeply integrated into IPC handlers:
- `jsonrpsee::types::Params` for parameter parsing
- `jsonrpsee::types::ErrorObject` for error handling
- 88 references across 6 files

---

## Solution Options

### Option A: Compatibility Shim (Quick - 1 hour)
Create types that match jsonrpsee's API:
```rust
pub mod compat {
    pub type Params<'a> = serde_json::Value;
    pub type ErrorObject<'static> = JsonRpcError;
}
```

**Pros**:
- Minimal code changes
- Quick to implement
- Can migrate gradually

**Cons**:
- Not truly removing jsonrpsee API
- Temporary solution

### Option B: Full Migration (Deep Debt - 4-6 hours)
Rewrite IPC handlers to use Pure Rust types directly:
```rust
// Before:
async fn handler(params: jsonrpsee::types::Params) 
    -> Result<Response, jsonrpsee::types::ErrorObject>

// After:
async fn handler(params: Option<serde_json::Value>) 
    -> Result<serde_json::Value, String>
```

**Pros**:
- True deep debt solution
- 100% Pure Rust
- No legacy API surface

**Cons**:
- Requires rewriting many handlers
- More testing needed
- Higher risk

### Option C: Unix Socket Only (Radical - 2 hours)
Remove JSON-RPC entirely from IPC, use only tarpc:
- IPC handlers only via tarpc (already Pure Rust)
- JSON-RPC only for external HTTP API
- Simplest, most Rust-native

**Pros**:
- Simplest solution
- Already have tarpc
- 100% Pure Rust
- Aligns with BearDog

**Cons**:
- Removes JSON-RPC from IPC (might be desired?)
- Need to verify nothing depends on it

---

## Recommendation

**Option C: Unix Socket Only (tarpc)** is most aligned with ecoPrimals philosophy:

1. **Primal-to-primal**: Use tarpc (type-safe, efficient)
2. **External API**: Use Pure Rust JSON-RPC (universal access)
3. **Simplest**: Remove intermediate abstraction

This matches BearDog's architecture:
- tarpc for inter-primal (PRIMARY)
- JSON-RPC for external (FALLBACK)
- HTTP for legacy (DEPRECATED)

---

## Next Steps

1. **Verify**: Are IPC handlers actually used for JSON-RPC?
2. **If yes**: Implement Option A (shim) then migrate to B
3. **If no**: Implement Option C (Unix socket only)

---

## Time Estimates

- **Option A** (shim): 1 hour
- **Option B** (full migration): 4-6 hours
- **Option C** (tarpc only): 2 hours

---

## Current Status

**Paused for strategic decision**:
- Need to verify actual usage of JSON-RPC in IPC
- Then choose best path forward
- All Pure Rust types ready to use

**Ready to resume** when strategy is chosen.

