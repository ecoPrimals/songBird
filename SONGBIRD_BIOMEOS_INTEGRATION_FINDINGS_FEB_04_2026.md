# Songbird - biomeOS Integration Findings
**Date**: February 4, 2026  
**Status**: INVESTIGATION COMPLETE - Ready for Implementation

---

## Executive Summary

Upstream biomeOS validation found **7 missing methods** and **1 config bug** blocking peer discovery and beacon exchange. Investigation reveals Songbird has robust infrastructure but lacks the specific biomeOS-standard methods.

### Critical Findings

| Issue | Status | Severity | Notes |
|-------|--------|----------|-------|
| BearDog socket hardcoded | ⚠️ PARTIAL | HIGH | XDG discovery works, but fallback hardcoded |
| Missing `health` | ✅ EXISTS | LOW | Exists as `primal.health`, needs bare `health` alias |
| Missing `identity` | ❌ MISSING | HIGH | Required for all primals |
| Missing `rpc.discover` | ❌ MISSING | HIGH | Required for capability discovery |
| Missing `network.beacon_exchange` | ❌ MISSING | CRITICAL | Blocks beacon meetings |
| Missing `network.broadcast` | ❌ MISSING | CRITICAL | Blocks Dark Forest |
| Missing `network.listen` | ❌ MISSING | CRITICAL | Blocks Dark Forest |
| Missing `encrypt_discovery` | ❌ MISSING | HIGH | Wrapper for BearDog |
| Missing `decrypt_discovery` | ❌ MISSING | HIGH | Wrapper for BearDog |

---

## What EXISTS (Good News!)

### 1. ✅ BearDog Socket Discovery (Mostly Correct)

**Location**: `crates/songbird-http-client/src/crypto/socket_discovery.rs`

```rust
pub fn discover_beardog_socket() -> String {
    discover_socket("BEARDOG_SOCKET", "beardog", "/tmp/beardog.sock")
}
```

**Discovery Chain**:
1. `$BEARDOG_SOCKET` env var ✅
2. `$XDG_RUNTIME_DIR/biomeos/beardog-{family}.sock` ✅  
3. `/tmp/beardog.sock` (legacy fallback) ✅

**Issue Found**: `handle_http_request` (line 417) has hardcoded fallback:
```rust
let neural_api_socket = std::env::var("NEURAL_API_SOCKET")
    .unwrap_or_else(|_| "/tmp/neural-api-nat0.sock".to_string());
```
Should use `discover_neural_api_socket()` instead!

### 2. ✅ Health Check (Exists but Wrong Name)

**Location**: `crates/songbird-orchestrator/src/ipc/unix/handlers.rs:173`

```rust
pub async fn handle_health(registry: Arc<RwLock<PrimalRegistry>>) -> Result<Value, JsonRpcError> {
    Ok(serde_json::json!({
        "status": "healthy",
        "registered_primals": primal_count,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}
```

**Routing**: `primal.health` (line 362 in `server.rs`)

**Issue**: biomeOS expects bare `health`, not `primal.health`.

**Solution**: Add alias routing for backward compatibility:
- `health` → `handle_health_standard()` (new, matches biomeOS spec)
- `primal.health` → `handle_health()` (keep for backward compat)

### 3. ✅ JSON-RPC 2.0 Infrastructure

**Locations**:
- HTTP: `crates/songbird-orchestrator/src/server/jsonrpc_api.rs`
- Unix Socket: `crates/songbird-orchestrator/src/ipc/unix/server.rs`

**Features**:
- Full JSON-RPC 2.0 spec compliance ✅
- Request/response validation ✅
- Standard error codes (-32700 to -32603) ✅
- Method routing infrastructure ✅
- Concurrent connection handling ✅

**Ready for new method additions!**

### 4. ✅ Capability System

**Location**: `crates/songbird-orchestrator/src/ipc/unix/handlers.rs:318`

```rust
pub async fn handle_discover_capabilities() -> Result<Value, JsonRpcError> {
    let capabilities = vec![
        "http.post",
        "http.get",
        "http.request",
        "discovery.announce",
        "discovery.query",
        "security.verify",
    ];
    
    Ok(serde_json::json!({
        "capabilities": capabilities,
        "metadata": {
            "primal_name": "songbird",
            "version": env!("CARGO_PKG_VERSION"),
            "family_id": family_id
        }
    }))
}
```

**Issue**: Routed as `discover_capabilities` (line 373), biomeOS expects `rpc.discover`.

---

## What's MISSING

### 1. ❌ Standard Method: `identity`

**Required By**: All primals (biomeOS standard)

**Expected Response**:
```json
{
  "primal": "songbird",
  "version": "0.1.0",
  "family_id": "nat0",
  "capabilities": [
    "network.broadcast",
    "network.listen",
    "network.beacon_exchange",
    "encrypt_discovery",
    "decrypt_discovery"
  ]
}
```

**Implementation Plan**: Create `handle_identity()` in `handlers.rs`.

---

### 2. ❌ Standard Method: `rpc.discover`

**Required By**: All primals (biomeOS standard)

**Expected Response**:
```json
{
  "methods": [
    {
      "name": "health",
      "params": [],
      "description": "Health check"
    },
    {
      "name": "network.beacon_exchange",
      "params": ["endpoint", "beacon_id", "beacon_seed_encrypted"],
      "description": "Exchange beacon seeds with peer"
    }
  ]
}
```

**Implementation Plan**: 
- Refactor existing `discover_capabilities` → `rpc.discover`
- Add method introspection
- Keep `discover_capabilities` as alias for backward compat

---

### 3. ❌ Network Method: `network.beacon_exchange`

**Priority**: CRITICAL (blocks beacon meetings)

**Purpose**: Perform beacon seed exchange during a "meeting"

**Expected Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "network.beacon_exchange",
  "params": {
    "endpoint": "192.168.1.100:8080",
    "beacon_id": "our_beacon_id_here",
    "beacon_seed_encrypted": "encrypted_seed_for_peer"
  },
  "id": 1
}
```

**Expected Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "success": true,
    "peer_beacon_id": "peer_beacon_id_here",
    "peer_seed_encrypted": "encrypted_seed_from_peer",
    "peer_family_hint": "8ff3b864a4bc589a"
  },
  "id": 1
}
```

**Implementation Plan**:
1. Create `handle_beacon_exchange()` in `handlers.rs`
2. Use existing `ConnectionManager` for peer connectivity
3. Integrate with Dark Forest protocol

**Dependencies**: 
- Dark Forest protocol (already implemented in `songbird-universal-ipc`)
- BearDog for encryption (via capability.call or direct socket)

---

### 4. ❌ Network Methods: `network.broadcast` & `network.listen`

**Priority**: CRITICAL (blocks Dark Forest)

**`network.broadcast`** - Broadcast encrypted beacon to network

**Expected Request**:
```json
{
  "method": "network.broadcast",
  "params": {
    "payload_encrypted": "encrypted_beacon_broadcast",
    "ttl": 60,
    "channel": "dark_forest"
  }
}
```

**`network.listen`** - Listen for encrypted beacon broadcasts

**Expected Request**:
```json
{
  "method": "network.listen",
  "params": {
    "channel": "dark_forest",
    "timeout_seconds": 30
  }
}
```

**Implementation Plan**:
1. Create `handle_network_broadcast()` and `handle_network_listen()` in new `handlers/network.rs`
2. Integrate with existing Dark Forest implementation in `songbird-universal-ipc`
3. Use existing discovery infrastructure

---

### 5. ❌ Encryption Wrappers: `encrypt_discovery` & `decrypt_discovery`

**Priority**: HIGH (required for beacon exchange)

**Purpose**: Wrapper methods that delegate to BearDog for beacon encryption/decryption

**`encrypt_discovery`** - Encrypt payload for discovery broadcast

**Expected Request**:
```json
{
  "method": "encrypt_discovery",
  "params": {
    "payload": {"type": "beacon_announce", "beacon_id": "xxx"},
    "use_beacon_seed": true
  }
}
```

**`decrypt_discovery`** - Decrypt discovery broadcast

**Expected Request**:
```json
{
  "method": "decrypt_discovery",
  "params": {
    "encrypted_b64": "encrypted_data_here",
    "known_beacon_seeds": ["seed1_hex", "seed2_hex"]
  }
}
```

**Implementation Plan**:
1. Create `handle_encrypt_discovery()` and `handle_decrypt_discovery()` in new `handlers/crypto.rs`
2. Delegate to BearDog via:
   - Option A: Direct socket call using `discover_beardog_socket()`
   - Option B: Via Neural API capability.call (better for decoupling)
3. Call BearDog's `beacon.encrypt` and `beacon.try_decrypt` methods

**Dependencies**: BearDog implementing `beacon.*` methods (see separate handoff doc)

---

## Implementation Priority

### Phase 1: Fix Config Bug (15 min)
1. ✅ Fix hardcoded `/tmp/neural-api-nat0.sock` in `handlers.rs:417`
   - Replace with `discover_neural_api_socket()`

### Phase 2: Standard Methods (30 min)
1. ✅ Add `identity` handler
2. ✅ Rename `discover_capabilities` → `rpc.discover`
3. ✅ Add bare `health` alias (keep `primal.health` for backward compat)

### Phase 3: Encryption Wrappers (1 hour)
1. ✅ Create `handlers/crypto.rs`
2. ✅ Implement `encrypt_discovery` (delegate to BearDog)
3. ✅ Implement `decrypt_discovery` (delegate to BearDog)
4. ✅ Add routing in `server.rs`

### Phase 4: Network Methods (2-3 hours)
1. ✅ Create `handlers/network.rs`
2. ✅ Implement `network.beacon_exchange`
3. ✅ Implement `network.broadcast`
4. ✅ Implement `network.listen`
5. ✅ Add routing in `server.rs`
6. ✅ Integration tests

---

## Testing Strategy

### Unit Tests
- Each handler in isolation
- Mock BearDog responses for crypto wrappers
- Mock peer responses for beacon_exchange

### Integration Tests
- End-to-end beacon exchange between two Songbird instances
- Broadcast → listen flow
- Encrypt → decrypt roundtrip

### E2E Tests (with real BearDog)
- Full beacon meeting flow
- Dark Forest discovery
- Verify lineage integration

---

## Dependencies

### External
- **BearDog**: Must implement `beacon.encrypt` and `beacon.try_decrypt` methods
  - Status: Separate handoff doc issued (BEARDOG_BEACON_EVOLUTION_FEB04_2026.md)
  - Blocking: `encrypt_discovery`, `decrypt_discovery`, `network.beacon_exchange`

### Internal
- **Dark Forest Protocol**: Already implemented ✅
  - Location: `crates/songbird-universal-ipc/src/handlers/birdsong_handler.rs`
  - Status: Production-ready
  
- **Discovery Infrastructure**: Already implemented ✅
  - Location: `crates/songbird-discovery/`
  - Status: Production-ready
  
- **ConnectionManager**: Already implemented ✅
  - Location: `crates/songbird-orchestrator/src/app/connection_manager.rs`
  - Status: Production-ready

---

## Files to Modify

| File | Changes | Complexity |
|------|---------|------------|
| `crates/songbird-orchestrator/src/ipc/unix/handlers.rs` | Fix hardcode, add `identity`, rename method | LOW |
| `crates/songbird-orchestrator/src/ipc/unix/server.rs` | Add method routing | LOW |
| `crates/songbird-orchestrator/src/ipc/handlers/crypto.rs` | NEW - encryption wrappers | MEDIUM |
| `crates/songbird-orchestrator/src/ipc/handlers/network.rs` | NEW - network methods | HIGH |
| `crates/songbird-orchestrator/src/ipc/handlers/mod.rs` | Export new modules | LOW |

---

## Architectural Notes

### Why Wrapper Methods?

The upstream request asks for `encrypt_discovery` and `decrypt_discovery` in Songbird, even though BearDog does the actual crypto. This is **correct architecture**:

1. **Separation of Concerns**: BearDog = crypto provider, Songbird = network orchestrator
2. **Capability Translation**: biomeOS routes `encrypt_discovery` → Songbird → BearDog `beacon.encrypt`
3. **Protocol Encapsulation**: Songbird knows beacon exchange protocol, BearDog just does crypto
4. **Future-Proofing**: Can swap crypto provider without changing network layer

### Socket Discovery Architecture (Already Correct!)

Songbird already implements the correct XDG discovery chain:
```
$BEARDOG_SOCKET → XDG runtime dir → /tmp (legacy)
```

The **only issue** is one hardcoded fallback in HTTP delegation.

---

## Success Criteria

### Must Have (Blocking)
- ✅ `identity` method responds correctly
- ✅ `rpc.discover` lists all methods
- ✅ `health` (bare) alias works
- ✅ `encrypt_discovery` / `decrypt_discovery` delegate to BearDog
- ✅ `network.beacon_exchange` successfully exchanges beacons
- ✅ `network.broadcast` / `network.listen` work end-to-end
- ✅ No hardcoded socket paths

### Nice to Have (Polish)
- ✅ Integration tests for all new methods
- ✅ Comprehensive error handling
- ✅ Metrics/observability for new methods
- ✅ Documentation updates

---

## Next Steps

1. **Proceed with implementation** following priority order
2. **Coordinate with BearDog evolution** (parallel track)
3. **Test incrementally** (unit → integration → E2E)
4. **Update documentation** (method reference, biomeOS integration guide)
5. **Push to origin** once tests pass

---

**Investigation Complete**: All gaps identified, implementation plan ready.

**Ready to Proceed**: Architecture validated, dependencies clear, implementation straightforward.

---

**Investigator**: Claude (Songbird Deep Debt Evolution)  
**Reviewed By**: [Pending]  
**Status**: ✅ READY FOR IMPLEMENTATION
