# Core.rs Refactoring Plan - Phase 5C

**Date**: February 5, 2026  
**Target**: `core.rs` (1,064 lines)  
**Status**: 🚧 **READY TO EXECUTE**

---

## Analysis Summary

**Current file**: `crates/songbird-orchestrator/src/app/core.rs` (1,064 lines)

### Structure Breakdown

| Section | Lines | Percentage | Issue |
|---------|-------|------------|-------|
| Struct definition | ~30 | 3% | ✅ OK |
| `new()` constructor | ~125 | 12% | ✅ OK |
| **`start()` method** | **~630** | **59%** | ❌ **HUGE!** |
| `stop()` method | ~18 | 2% | ✅ OK |
| Query methods | ~50 | 5% | ✅ OK |
| Command routing | ~80 | 8% | ✅ OK |
| Helper methods | ~120 | 11% | ✅ OK |

**Problem**: The `start()` method alone is 630 lines (59% of the file)!

---

## Refactoring Strategy

### Approach: Extract `start()` into Sub-Methods

Instead of splitting into multiple files (which would break cohesion), extract logical subsections of the `start()` method into private helper methods.

**Pattern**: 
```rust
// Before:
pub async fn start(&mut self) -> Result<()> {
    // 630 lines of inline startup logic
}

// After:
pub async fn start(&mut self) -> Result<()> {
    self.provision_security().await?;
    self.start_http_server_internal().await?;
    self.start_ipc_services().await?;
    self.start_discovery().await?;
    self.start_federation().await?;
    self.start_monitoring().await?;
    Ok(())
}

async fn provision_security(&mut self) -> Result<()> { ... }
async fn start_http_server_internal(&mut self) -> Result<()> { ... }
// etc.
```

---

## Logical Subsections of `start()`

### 1. Security Provisioning (~60 lines)
- JWT secret provisioning from BearDog
- Security identity query
- USB seed integration

**Extract to**: `async fn provision_security(&mut self) -> Result<()>`

### 2. HTTP Server Startup (~40 lines)
- Bind address configuration
- HTTP server initialization
- Port detection

**Extract to**: `async fn start_http_server_internal(&mut self) -> Result<()>`

### 3. IPC Services (~60 lines)
- IPC server startup
- Universal IPC Broker
- tarpc server

**Extract to**: `async fn start_ipc_services(&mut self) -> Result<()>`

### 4. Discovery & Networking (~180 lines)
- Discovery listener finalization
- Beacon broadcast
- Anonymous discovery
- Peer connectivity

**Extract to**: `async fn start_discovery(&mut self) -> Result<()>`

### 5. Federation Setup (~120 lines)
- Node identity re-registration
- Peer discovery
- Federation coordinator
- Cross-primal connectivity

**Extract to**: `async fn start_federation(&mut self) -> Result<()>`

### 6. Monitoring & Cleanup (~80 lines)
- Service registry cleanup
- Health monitoring
- Observability setup

**Extract to**: `async fn start_monitoring(&mut self) -> Result<()>`

### 7. Final Status (~50 lines)
- Status reporting
- Capability announcement
- Startup complete message

**Extract to**: `fn log_startup_complete(&self)`

---

## Expected Result

### Before
```rust
core.rs (1,064 lines)
├── struct SongbirdOrchestrator (30 lines)
├── impl SongbirdOrchestrator
│   ├── new() (125 lines)
│   ├── start() (630 lines) ❌ HUGE!
│   ├── stop() (18 lines)
│   └── ... other methods (261 lines)
```

### After
```rust
core.rs (~550 lines)
├── struct SongbirdOrchestrator (30 lines)
├── impl SongbirdOrchestrator
│   ├── new() (125 lines)
│   ├── start() (~80 lines) ✅ Orchestration only
│   │   ├── provision_security() (~60 lines)
│   │   ├── start_http_server_internal() (~40 lines)
│   │   ├── start_ipc_services() (~60 lines)
│   │   ├── start_discovery() (~180 lines)
│   │   ├── start_federation() (~120 lines)
│   │   ├── start_monitoring() (~80 lines)
│   │   └── log_startup_complete() (~50 lines)
│   ├── stop() (18 lines)
│   └── ... other methods (261 lines)
```

**Reduction**: 1,064 → ~1,150 lines total (slight increase due to method signatures/docs)  
**Largest Method**: 630 → 180 lines (71% reduction!)  
**Maintainability**: +++

---

## Benefits

✅ **Readability**: `start()` method is now high-level orchestration  
✅ **Maintainability**: Each subsystem in its own method  
✅ **Testability**: Can test subsystems independently  
✅ **Deep Debt**: +0.1% (method extraction, improved structure)  
✅ **Zero Risk**: All private methods, same behavior  

---

## Implementation Steps

1. ✅ Create refactoring plan (this document)
2. ⏳ Extract `provision_security()` method
3. ⏳ Extract `start_http_server_internal()` method
4. ⏳ Extract `start_ipc_services()` method
5. ⏳ Extract `start_discovery()` method
6. ⏳ Extract `start_federation()` method
7. ⏳ Extract `start_monitoring()` method
8. ⏳ Extract `log_startup_complete()` method
9. ⏳ Update `start()` to orchestrate extracted methods
10. ⏳ Verify build passes
11. ⏳ Run tests
12. ⏳ Commit and push

---

## Alternative: File Splitting (NOT RECOMMENDED)

We could split into:
- `core/types.rs` - Struct definition
- `core/lifecycle.rs` - start(), stop()
- `core/queries.rs` - Query methods
- `core/commands.rs` - Command routing

**Why NOT recommended**:
- Breaks cohesion (struct and methods separated)
- Harder to navigate (multi-file struct impl)
- More complex module structure
- Same line count (just redistributed)

**Better approach**: Keep in one file, extract huge method into smaller methods

---

## Risk Assessment

**Very Low Risk** - Internal refactoring only

- All methods are private (not pub)
- No API changes
- Same behavior, better structure
- Easy to test and verify

---

**Status**: 🚧 **Ready to proceed with method extraction**  
**Next**: Extract security provisioning logic
