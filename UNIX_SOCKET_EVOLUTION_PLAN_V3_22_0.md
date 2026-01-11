# 🎯 Unix Socket Evolution Plan - v3.22.0

**Date**: January 13, 2026  
**Priority**: HIGH - Blocking atomic deployment  
**Status**: 🚧 IN PROGRESS - Pure Rust Implementation Designed

---

## 📋 **Executive Summary**

biomeOS testing revealed `jsonrpsee` has Unix socket binding issues. BearDog v0.16.1 (production-tested ✅) uses pure `tokio::net::UnixListener` successfully. We're evolving Songbird to the same proven pattern.

**Problem**: `jsonrpsee::server::Server::build()` causes "invalid socket address" error  
**Solution**: Pure Rust `tokio::net::UnixListener` + manual JSON-RPC implementation  
**Inspiration**: BearDog v0.16.1's working Unix socket IPC implementation

---

## ✅ **What's Complete**

### **1. Pure Rust Server Implementation** ✅
- **File**: `crates/songbird-orchestrator/src/ipc/server_pure_rust.rs` (690 lines)
- **Features**:
  - Pure `tokio::net::UnixListener` (no jsonrpsee)
  - JSON-RPC 2.0 types (Request, Response, Error)
  - Atomic readiness flags (BearDog pattern)
  - 3-tier socket path fallback (biomeOS standard)
  - 11 API routes (3 P2P + 4 registry + 4 graph)
  - Graceful shutdown and cleanup
  - 6 unit tests (all passing)

### **2. Socket Configuration** ✅
- **Features**:
  - `SONGBIRD_SOCKET` override
  - `SONGBIRD_FAMILY_ID` and `SONGBIRD_NODE_ID`
  - XDG runtime directory support
  - Automatic parent directory creation
  - Zero unsafe code

---

## ⏳ **What Remains**

### **1. Handler Adaptation Layer** ⏳

**Current**: Handlers use `jsonrpsee::types::{ErrorObject, Params}`  
**Needed**: Adapt to `Result<serde_json::Value, JsonRpcError>`

**Approach**: Create adapter methods that:
1. Parse `Option<serde_json::Value>` params
2. Call existing handlers
3. Convert responses to `serde_json::Value`
4. Convert errors to `JsonRpcError`

**Estimate**: ~2-3 hours

### **2. Module Integration** ⏳

**Changes Needed**:
- Update `crates/songbird-orchestrator/src/ipc/mod.rs`:
  - Replace `pub mod server;` with `pub mod server_pure_rust;`
  - Re-export `UnixSocketServer` from pure Rust implementation
- Update `crates/songbird-orchestrator/Cargo.toml`:
  - Remove `jsonrpsee` dependency (optional - can keep for future)
- Update startup code:
  - Use new `UnixSocketServer::start()` API
  - Handle `Arc<UnixSocketServer>` pattern

**Estimate**: ~30 minutes

### **3. Comprehensive Testing** ⏳

**Test Scenarios**:
1. ✅ Unit tests (6/6 passing - socket path logic)
2. ⏳ E2E tests (adapt existing to new server)
3. ⏳ Integration with biomeOS launcher
4. ⏳ All 11 APIs working
5. ⏳ Concurrent connections
6. ⏳ Graceful shutdown

**Estimate**: ~2 hours

### **4. Documentation** ⏳

**Updates Needed**:
- README.md: v3.22.0 (Pure Rust Unix Socket)
- STATUS.md: Evolution summary
- CHANGELOG.md: Breaking changes (if any)
- New document: `UNIX_SOCKET_EVOLUTION_V3_22_0.md`

**Estimate**: ~1 hour

---

## 🎯 **Implementation Strategy**

### **Phase 1: Adapter Layer** (2-3 hours)

Create `crates/songbird-orchestrator/src/ipc/handlers_adapter.rs`:

```rust
use super::handlers::IpcHandlers;
use super::server_pure_rust::{JsonRpcError, JsonRpcResponse};

impl IpcHandlers {
    /// Adapter: discover_by_family (JSON-RPC → existing handler)
    pub async fn discover_by_family_adapter(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, JsonRpcError> {
        // 1. Parse params
        let request: DiscoverByFamilyRequest = match params {
            Some(p) => serde_json::from_value(p)
                .map_err(|e| JsonRpcError::invalid_params(format!("{}", e)))?,
            None => return Err(JsonRpcError::invalid_params("params required")),
        };

        // 2. Call existing handler (adapt jsonrpsee Params)
        let response = self.discover_by_family(/* convert request */).await
            .map_err(|e| JsonRpcError::internal_error(format!("{}", e)))?;

        // 3. Convert response to JSON
        serde_json::to_value(response)
            .map_err(|e| JsonRpcError::internal_error(format!("{}", e)))
    }
    
    // Repeat for all 11 APIs...
}
```

### **Phase 2: Wire Adapters to Server** (30 min)

Update `server_pure_rust.rs` to call adapter methods:

```rust
async fn handle_jsonrpc_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
    let result = match request.method.as_str() {
        "discover_by_family" => self.handlers.discover_by_family_adapter(request.params).await,
        "register_service" => self.handlers.register_service_adapter(request.params).await,
        // ... all 11 APIs
        _ => Err(JsonRpcError::method_not_found(&request.method)),
    };
    
    // ... build response
}
```

### **Phase 3: Integration** (30 min)

1. Update `ipc/mod.rs` to use pure Rust server
2. Update startup code to use new API
3. Test locally

### **Phase 4: Testing** (2 hours)

1. Run unit tests
2. Run E2E tests
3. Test with biomeOS launcher
4. Verify all 11 APIs work

### **Phase 5: Documentation** (1 hour)

1. Update version to v3.22.0
2. Write evolution document
3. Update README and STATUS

---

## 📊 **Estimated Timeline**

| Phase | Tasks | Time | Status |
|-------|-------|------|--------|
| **Design** | Pure Rust implementation | 2h | ✅ COMPLETE |
| **Phase 1** | Adapter layer | 2-3h | ⏳ PENDING |
| **Phase 2** | Wire adapters | 30m | ⏳ PENDING |
| **Phase 3** | Integration | 30m | ⏳ PENDING |
| **Phase 4** | Testing | 2h | ⏳ PENDING |
| **Phase 5** | Documentation | 1h | ⏳ PENDING |
| **Total** | | **6-7 hours** | 20% complete |

---

## 🎊 **Why This Evolution**

### **Problems with jsonrpsee**
- ❌ Complex Unix socket requirements
- ❌ "invalid socket address" errors
- ❌ Opaque error messages
- ❌ Heavy dependency for simple JSON-RPC

### **Benefits of Pure Rust**
- ✅ Direct control over socket binding
- ✅ Simple, transparent implementation
- ✅ Proven by BearDog v0.16.1 in production
- ✅ No external RPC library dependencies
- ✅ Better error messages
- ✅ Easier to debug

### **BearDog Validation**
BearDog v0.16.1 is running in production with:
- ✅ `/run/user/1000/beardog-nat0.sock` (XDG-compliant)
- ✅ Multiple successful test connections
- ✅ Zero socket binding issues
- ✅ Clean process management

**This proves the pure Rust approach works!**

---

## 🚀 **Next Steps**

### **Immediate**
1. Create adapter layer for all 11 APIs
2. Wire adapters to pure Rust server
3. Test locally with unit tests

### **Then**
4. Integration with biomeOS launcher
5. E2E testing
6. Documentation

### **Finally**
7. Version bump to v3.22.0
8. Handoff to biomeOS team

---

## 📝 **Files Created/Modified**

### **Created** ✅
- `crates/songbird-orchestrator/src/ipc/server_pure_rust.rs` (690 lines)

### **To Create** ⏳
- `crates/songbird-orchestrator/src/ipc/handlers_adapter.rs` (~500 lines)
- `UNIX_SOCKET_EVOLUTION_V3_22_0.md` (evolution document)

### **To Modify** ⏳
- `crates/songbird-orchestrator/src/ipc/mod.rs` (switch to pure Rust)
- `crates/songbird-orchestrator/src/main.rs` (use new server API)
- `crates/songbird-orchestrator/Cargo.toml` (optional: remove jsonrpsee)
- `README.md` (version bump)
- `STATUS.md` (evolution summary)

---

## 🎯 **Evolution Principles Met**

- ✅ **Deep debt solution**: Replacing problematic library, not patching
- ✅ **Modern idiomatic Rust**: Pure `tokio::net::UnixListener` + async/await
- ✅ **Zero hardcoding**: All env-driven configuration
- ✅ **Smart refactoring**: Modular design, reuses existing handlers
- ✅ **No production mocks**: Real Unix socket implementation
- ✅ **Proven pattern**: Based on BearDog v0.16.1 (production-tested)

---

## 💡 **Key Insights**

1. **BearDog's approach is proven** - They use pure `tokio::net::UnixListener` successfully
2. **jsonrpsee is overkill** - For Unix sockets, manual JSON-RPC is simpler
3. **Atomic readiness flags are brilliant** - Lock-free concurrent readiness checks
4. **biomeOS socket standard works** - BearDog proves the 3-tier fallback is sound

---

**Status**: 🚧 IN PROGRESS (20% complete - design done, implementation pending)  
**Confidence**: 💯 100% (BearDog proves it works)  
**Timeline**: ~6-7 hours to completion  
**Ready For**: Continued implementation

---

🎵 **Songbird v3.22.0: Pure Rust Unix Socket Evolution** 🎵  
**Different orders of the same architecture.** 🍄🐸

