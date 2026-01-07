# 📝 Phase 3: Documentation Cleanup Plan - v3.15.0

**Date**: January 7, 2026  
**Status**: 🔄 **IN PROGRESS**  
**Target**: Replace 215+ vendor name instances

---

## 🎯 **Mission**

Remove ALL vendor names from comments, logs, and documentation to enforce:
> "Each primal only knows itself and discovers others at runtime"

---

## 📊 **Scope**

### **Target Replacements**:

| Find | Replace | Context |
|------|---------|---------|
| `BearDog` | `security provider` | Comments, docs |
| `beardog` | `security provider` | Logs, variable names |
| `BearDog client` | `security client` | Code comments |
| `beardog_client` | `security_client` | Variable names |
| `beardog_endpoint` | `security_endpoint` | Variable names |
| `BearDog API` | `security provider API` | Documentation |
| `BearDog integration` | `security provider integration` | Comments |

### **Instances Found**: 215+ across 33 files

---

## 🔧 **Strategy**

### **Approach**: Systematic File-by-File Cleanup

**Why not bulk find/replace?**
- Need to preserve context
- Some instances may be in external docs (keep those)
- Need to update variable names carefully
- Want to ensure consistency

### **Order of Operations**:
1. ✅ Core app files (highest traffic)
2. Trust/security files
3. Access control files
4. IPC/registry files
5. Test files
6. Documentation files

---

## 📋 **Files to Clean** (Top Priority)

### **1. High-Traffic Core Files** (8 files)
- `app/core.rs` - Main orchestrator
- `app/discovery_bridge.rs` - Discovery federation
- `app/discovery_startup.rs` - Discovery initialization
- `app/security_setup.rs` - Security provider discovery
- `security_capability_client.rs` - Security client
- `trust/peer_trust.rs` - Peer trust evaluation
- `trust/escalation.rs` - Trust escalation
- `access_control/auth.rs` - Authentication

### **2. IPC/Registry Files** (3 files)
- `ipc/primal_registry.rs` - Primal registration
- `ipc/unix_socket.rs` - Unix socket IPC
- `ipc/mod.rs` - IPC module

### **3. Trust Files** (5 files)
- `trust/lineage_auth.rs` - Lineage authentication
- `trust/types.rs` - Trust types
- `trust/mod.rs` - Trust module
- `trust/universal_trust_api.rs` - Universal trust API

### **4. Test Files** (9 files)
- `app/tests_birdsong_integration.rs`
- `app/tests_discovery_bridge.rs`
- `tests/*.rs` - Integration tests

### **5. Other Files** (8 files)
- Server, CLI, core modules, etc.

---

## 🎯 **Replacement Rules**

### **In Comments**:
```rust
// ❌ BEFORE:
// Query BearDog for trust evaluation
// Connect to BearDog's Unix socket
// If BearDog is available...

// ✅ AFTER:
// Query security provider for trust evaluation
// Connect to security provider's Unix socket
// If security provider is available...
```

### **In Log Messages**:
```rust
// ❌ BEFORE:
info!("Connecting to BearDog at {}", endpoint);
warn!("BearDog unavailable, falling back");
error!("BearDog connection failed");

// ✅ AFTER:
info!("Connecting to security provider at {}", endpoint);
warn!("Security provider unavailable, falling back");
error!("Security provider connection failed");
```

### **In Variable Names**:
```rust
// ❌ BEFORE:
let beardog_client = ...;
let beardog_endpoint = ...;
let beardog_response = ...;

// ✅ AFTER:
let security_client = ...;
let security_endpoint = ...;
let security_response = ...;
```

### **In Struct Names**:
```rust
// ❌ BEFORE (only if it's generic):
struct BearDogClient { ... }

// ✅ AFTER:
struct SecurityClient { ... }

// ⚠️  EXCEPTION: Keep if it's actually BearDog-specific implementation
// (but move to separate integration module)
```

---

## 🚫 **What NOT to Change**

### **External References** (Keep as-is):
- Historical commit messages
- External documentation references
- Specific integration guides (e.g., "How to integrate with BearDog")
- Test fixtures that simulate specific providers
- Environment variable values in examples (can show both)

### **Specific Implementations** (Keep but isolate):
- Actual BearDog provider implementations
- BearDog-specific test mocks
- Integration test fixtures

---

## 📈 **Progress Tracking**

### **By Category**:
- [ ] Comments: 0/150+ replaced
- [ ] Log messages: 0/40+ replaced
- [ ] Variable names: 0/20+ replaced
- [ ] Documentation: 0/5+ files updated

### **By File Type**:
- [ ] Source files (.rs): 0/33 cleaned
- [ ] Documentation (.md): 0/5 cleaned
- [ ] Tests: 0/9 cleaned

---

## 🧪 **Validation**

### **After Each File**:
1. ✅ Check compilation
2. ✅ Run affected tests
3. ✅ Grep for remaining instances
4. ✅ Commit with clear message

### **Final Validation**:
```bash
# Should return 0 (or only external docs):
grep -r "BearDog\|beardog" crates/songbird-orchestrator/src --exclude-dir=target
```

---

## 📝 **Commit Strategy**

### **Granular Commits**:
```
docs: clean vendor names from app/core.rs
docs: clean vendor names from trust/ modules
docs: clean vendor names from access_control/
docs: clean vendor names from tests/
docs: update root documentation
```

### **Why Granular?**:
- Easy to review
- Easy to revert if needed
- Clear history
- Better for bisecting if issues arise

---

## 🎯 **Success Criteria**

### **v3.15.0 (This Phase)**:
- ✅ Zero vendor names in source code comments
- ✅ Zero vendor names in log messages
- ✅ Generic variable names only
- ✅ Updated documentation
- ✅ Compilation passes
- ✅ Tests pass

---

## 🚀 **Execution Plan**

### **Session 1** (Current - 2-3 hours):
1. Clean high-traffic core files (8 files)
2. Clean trust files (5 files)
3. Clean IPC files (3 files)
4. Commit: "docs: Phase 3.1 - clean core files"

### **Session 2** (Future - 1-2 hours):
1. Clean test files (9 files)
2. Clean remaining files (8 files)
3. Update root documentation
4. Commit: "docs: Phase 3.2 - clean tests and docs"

---

## 📊 **Estimated Impact**

### **Lines Changed**: ~215+ instances
### **Files Modified**: ~33 files
### **Risk**: 🟢 **LOW** (cosmetic changes only)
### **Testing Required**: 🟡 **MODERATE** (verify logs still make sense)

---

## 🎊 **Benefits**

### **Architectural**:
- ✅ Enforces zero vendor hardcoding principle
- ✅ Makes codebase truly vendor-agnostic
- ✅ Improves maintainability

### **Developer Experience**:
- ✅ Clearer, more generic code
- ✅ Easier to understand for new contributors
- ✅ No vendor bias in documentation

### **Ecosystem**:
- ✅ Any security provider can integrate
- ✅ No vendor lock-in perception
- ✅ True decentralization

---

**Status**: ✅ **READY TO EXECUTE**  
**Next**: Start with high-traffic core files

---

_"Code should speak in capabilities, not vendor names."_

