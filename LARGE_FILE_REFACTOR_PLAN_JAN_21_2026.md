# Large File Smart Refactoring Plan - Songbird v4.8.0
**Date**: January 21, 2026  
**Goal**: Domain-driven refactoring (not arbitrary splits)  
**Status**: 🔄 EXECUTING

---

## Philosophy: Smart vs. Dumb Refactoring

### ❌ Dumb Refactoring (What We DON'T Want)
- Arbitrary line count splits
- Breaking related functions apart
- Creating `_part1.rs`, `_part2.rs` files
- Losing domain cohesion

### ✅ Smart Refactoring (What We DO Want)
- **Domain-driven**: Split by business logic
- **Cohesive**: Related functions stay together
- **Clear interfaces**: Each module has a purpose
- **Testable**: Can test each concern independently
- **Maintainable**: Easier to find and modify code

---

## Target Files (10 files, 8,325 total lines)

| # | File | Lines | Status |
|---|------|-------|--------|
| 1 | `server/federation_api.rs` | 971 | ⏰ Planned |
| 2 | `ipc/unix_socket.rs` | 949 | ⏰ Planned |
| 3 | `app/core.rs` | 915 | ⏰ Planned |
| 4 | `security_capability_client.rs` | 898 | ⏰ Planned |
| 5 | `crypto/beardog_crypto_client.rs` | 891 | ⏰ Planned |
| 6 | `graph/coordination.rs` | 859 | ⏰ Planned |
| 7 | `ipc/server_pure_rust.rs` | 856 | ⏰ Planned |
| 8 | `core/biome/modules/types.rs` | 850 | ⏰ Planned |
| 9 | `core/ai_orchestration_engine.rs` | 833 | ⏰ Planned |
| 10 | `core/mod.rs` | 782 | ⏰ Planned |

---

## File-by-File Smart Refactoring Strategy

### 1. server/federation_api.rs (971 lines) → 4 modules

**Current Structure** (monolithic):
- Node management (join, status, nodes, heartbeat)
- Service federation (register, list, get, stats)
- Capability registration (NEW system)
- Trust-aware endpoints (graduated disclosure)
- Helper functions (mixed concerns)

**Refactored Structure** (domain-driven):

```
server/
├── federation/
│   ├── mod.rs              (Main router builder, ~100 lines)
│   ├── node_management.rs  (Node ops, ~250 lines)
│   ├── service_endpoints.rs (Service registry, ~200 lines)
│   ├── capability_endpoints.rs (Capability providers, ~300 lines)
│   └── types.rs            (Shared types, ~100 lines)
```

**Benefits**:
- Clear separation of concerns
- Easier to test each domain independently
- Can evolve capability system without touching node management
- Reduces cognitive load (each file has ONE purpose)

---

### 2. ipc/unix_socket.rs (949 lines) → 3 modules

**Current Structure** (monolithic):
- JSON-RPC request handlers
- HTTP delegation handlers
- Unix socket server logic
- Protocol parsing
- Response formatting

**Refactored Structure** (protocol-driven):

```
ipc/
├── unix_socket/
│   ├── mod.rs            (Server setup, ~200 lines)
│   ├── jsonrpc_handler.rs (JSON-RPC 2.0 protocol, ~350 lines)
│   ├── http_handler.rs   (HTTP delegation, ~250 lines)
│   └── protocol.rs       (Parsing/formatting, ~150 lines)
```

**Benefits**:
- Protocol handlers isolated
- Can swap HTTP implementation without touching JSON-RPC
- Easier to add new protocols (WebSocket, gRPC, etc.)
- Clear request/response flow

---

### 3. app/core.rs (915 lines) → 4 modules

**Current Structure** (monolithic):
- App initialization
- Service lifecycle
- Health monitoring
- Shutdown handling
- Configuration loading

**Refactored Structure** (lifecycle-driven):

```
app/
├── core/
│   ├── mod.rs          (Main orchestrator, ~200 lines)
│   ├── initialization.rs (Startup logic, ~250 lines)
│   ├── lifecycle.rs    (Start/stop/health, ~300 lines)
│   ├── shutdown.rs     (Graceful shutdown, ~150 lines)
│   └── config.rs       (Config loading, ~100 lines)
```

**Benefits**:
- Lifecycle phases clearly separated
- Easier to test initialization without running full app
- Shutdown logic isolated for reliability
- Configuration changes don't affect lifecycle

---

### 4. security_capability_client.rs (898 lines) → 3 modules

**Current Structure** (monolithic):
- Lineage verification
- Trust evaluation
- HTTP client operations
- Response parsing
- Error handling

**Refactored Structure** (concern-driven):

```
security/
├── capability_client/
│   ├── mod.rs          (Client setup, ~150 lines)
│   ├── lineage.rs      (Lineage verification, ~350 lines)
│   ├── trust.rs        (Trust evaluation, ~300 lines)
│   └── http.rs         (HTTP operations, ~100 lines)
```

**Benefits**:
- Lineage logic isolated (critical for security)
- Trust evaluation testable independently
- HTTP transport can be swapped
- Clear security boundaries

---

### 5. crypto/beardog_crypto_client.rs (891 lines) → 3 modules

**Current Structure** (monolithic):
- Crypto operations (sign, verify, encrypt, decrypt)
- JSON-RPC communication
- Response handling
- Error mapping

**Refactored Structure** (operation-driven):

```
crypto/
├── beardog_client/
│   ├── mod.rs          (Client setup, ~150 lines)
│   ├── operations.rs   (Crypto ops, ~400 lines)
│   ├── jsonrpc.rs      (RPC protocol, ~250 lines)
│   └── errors.rs       (Error handling, ~100 lines)
```

**Benefits**:
- Crypto operations grouped logically
- RPC protocol isolated (can swap to HTTP, etc.)
- Error handling centralized
- Easier to add new crypto operations

---

### 6. graph/coordination.rs (859 lines) → 4 modules

**Current Structure** (monolithic):
- Graph state management
- Capability coordination
- Task distribution
- Status tracking
- Validation logic

**Refactored Structure** (layer-driven):

```
graph/
├── coordination/
│   ├── mod.rs          (Coordinator core, ~200 lines)
│   ├── state.rs        (State management, ~250 lines)
│   ├── distribution.rs (Task distribution, ~250 lines)
│   ├── tracking.rs     (Status tracking, ~150 lines)
│   └── validation.rs   (Validation logic, ~100 lines)
```

**Benefits**:
- State management isolated (critical for consistency)
- Task distribution testable independently
- Validation rules centralized
- Clear data flow

---

### 7. ipc/server_pure_rust.rs (856 lines) → 3 modules

**Current Structure** (monolithic):
- Pure Rust HTTP server
- Request routing
- Handler implementations
- Response formatting

**Refactored Structure** (layer-driven):

```
ipc/
├── pure_rust_server/
│   ├── mod.rs          (Server setup, ~200 lines)
│   ├── handlers.rs     (Request handlers, ~400 lines)
│   ├── routing.rs      (Route configuration, ~150 lines)
│   └── responses.rs    (Response formatting, ~100 lines)
```

**Benefits**:
- Handler logic isolated
- Routing configuration clear
- Response formatting reusable
- Easier to add new endpoints

---

### 8. core/biome/modules/types.rs (850 lines) → 4 modules

**Current Structure** (monolithic):
- BiomeOS integration types
- Request/Response types
- Capability definitions
- Module configurations

**Refactored Structure** (type-driven):

```
core/biome/modules/
├── types/
│   ├── mod.rs          (Re-exports, ~50 lines)
│   ├── requests.rs     (Request types, ~250 lines)
│   ├── responses.rs    (Response types, ~250 lines)
│   ├── capabilities.rs (Capability types, ~200 lines)
│   └── config.rs       (Configuration types, ~100 lines)
```

**Benefits**:
- Request/response types clearly separated
- Capability definitions isolated
- Configuration types centralized
- Easier to maintain type consistency

---

### 9. core/ai_orchestration_engine.rs (833 lines) → 4 modules

**Current Structure** (monolithic):
- AI task orchestration
- Model selection
- Execution management
- Result aggregation

**Refactored Structure** (stage-driven):

```
core/ai/
├── orchestration_engine/
│   ├── mod.rs          (Engine core, ~200 lines)
│   ├── task_planning.rs (Task planning, ~250 lines)
│   ├── execution.rs    (Execution management, ~250 lines)
│   └── aggregation.rs  (Result aggregation, ~150 lines)
```

**Benefits**:
- Planning phase isolated
- Execution logic testable independently
- Aggregation strategies centralized
- Clear AI pipeline flow

---

### 10. core/mod.rs (782 lines) → Re-organize (not split)

**Current Structure** (re-export heavy):
- Module declarations
- Re-exports
- Type aliases
- Constants

**Refactored Strategy** (clean up, not split):
- Review all re-exports
- Remove unused exports
- Group related exports
- Document export strategy

**Benefits**:
- Clearer module structure
- Easier to find types
- Reduced cognitive load
- Better IDE autocomplete

---

## Execution Strategy

### Phase 1: High-Impact Files (First 3)
1. ✅ `federation_api.rs` - Most benefit from domain split
2. ✅ `unix_socket.rs` - Clear protocol boundaries
3. ✅ `core.rs` - Lifecycle clarity critical

### Phase 2: Security & Crypto (Next 2)
4. ✅ `security_capability_client.rs` - Security isolation important
5. ✅ `beardog_crypto_client.rs` - Crypto operations critical

### Phase 3: Coordination & Services (Next 2)
6. ✅ `coordination.rs` - State management complexity
7. ✅ `server_pure_rust.rs` - Handler organization

### Phase 4: Types & AI (Final 3)
8. ✅ `types.rs` - Type organization
9. ✅ `ai_orchestration_engine.rs` - AI pipeline clarity
10. ✅ `mod.rs` - Clean up re-exports

---

## Testing Strategy

### Per-File Refactoring:
1. ✅ Read entire file
2. ✅ Identify domain boundaries
3. ✅ Create module directory
4. ✅ Extract domain-specific code
5. ✅ Update imports across codebase
6. ✅ Run `cargo build`
7. ✅ Run `cargo test`
8. ✅ Verify no functionality changed
9. ✅ Commit with descriptive message

### Validation:
- All tests pass
- No new warnings
- Same performance
- Clearer code structure

---

## Success Metrics

### Quantitative:
- ✅ No file > 500 lines (except re-export modules)
- ✅ Average file size: 150-300 lines
- ✅ Zero functionality regressions
- ✅ All tests pass

### Qualitative:
- ✅ Easier to find code
- ✅ Clearer responsibilities
- ✅ Better testability
- ✅ Improved maintainability

---

## Example: federation_api.rs Refactoring

### Before (971 lines, monolithic):
```rust
// federation_api.rs
pub fn federation_routes(...) -> Router { ... }
pub fn federation_routes_with_capabilities(...) -> Router { ... }
pub fn federation_routes_with_trust(...) -> Router { ... }
async fn federation_join(...) { ... }
async fn federation_status(...) { ... }
async fn register_capability_provider(...) { ... }
async fn capability_provider_heartbeat(...) { ... }
async fn list_services(...) { ... }
async fn register_service(...) { ... }
// ... 40+ more functions mixed together
```

### After (4 files, ~250 lines each, domain-driven):

```rust
// federation/mod.rs
pub mod node_management;
pub mod service_endpoints;
pub mod capability_endpoints;
pub mod types;

pub use types::FederationAppState;

pub fn federation_routes(...) -> Router {
    Router::new()
        .merge(node_management::routes())
        .merge(service_endpoints::routes())
        .merge(capability_endpoints::routes())
}
```

```rust
// federation/node_management.rs
//! Federation node management operations

pub fn routes() -> Router {
    Router::new()
        .route("/join", post(join))
        .route("/status", get(status))
        .route("/nodes", get(list_nodes))
        .route("/heartbeat", post(heartbeat))
}

async fn join(...) { ... }
async fn status(...) { ... }
async fn list_nodes(...) { ... }
async fn heartbeat(...) { ... }
```

```rust
// federation/service_endpoints.rs
//! Service registry operations

pub fn routes() -> Router {
    Router::new()
        .route("/services", get(list))
        .route("/services", post(register))
        .route("/services/:id", get(get))
}

async fn list(...) { ... }
async fn register(...) { ... }
async fn get(...) { ... }
```

```rust
// federation/capability_endpoints.rs
//! Capability provider registration

pub fn routes() -> Router {
    Router::new()
        .route("/register", post(register))
        .route("/capability/heartbeat", post(heartbeat))
        .route("/providers", get(list))
}

async fn register(...) { ... }
async fn heartbeat(...) { ... }
async fn list(...) { ... }
```

**Result**: Same functionality, 4x clearer structure!

---

## Timeline Estimate

**Per File**: 30-60 minutes (depending on complexity)  
**Total**: 5-10 hours for all 10 files

**Breakdown**:
- Analysis: 10 minutes
- Code extraction: 20-30 minutes
- Import updates: 10-15 minutes
- Testing: 10-15 minutes

---

## Current Status

**Completed**: 0/10 files  
**In Progress**: Planning  
**Next**: Execute on `federation_api.rs`

---

## Notes

- This is TRUE technical debt reduction
- Not just "making files smaller"
- Domain-driven architecture
- Maintainability > arbitrary metrics
- Quality > speed

---

**Status**: Ready for execution  
**Recommendation**: Start with Phase 1 (high-impact files)  
**Expected Outcome**: More maintainable, testable, understandable codebase

