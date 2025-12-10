# 🧹 Primal-SDK Cleanup Plan
**Date**: December 2, 2025  
**Status**: Ready for Execution  
**Timeline**: 2-3 weeks  
**Priority**: P1 (High) - Required for Production

---

## 🎯 **OBJECTIVE**

**Eliminate primal-specific hardcoding** from `songbird-primal-sdk` crate while maintaining backward compatibility through capability-based discovery.

---

## 📊 **CURRENT STATE**

### **Primal-Specific Files Found** ⚠️
```
crates/songbird-primal-sdk/src/
├── beardog.rs          ❌ Hardcoded BearDog
├── squirrel.rs         ❌ Hardcoded Squirrel  
├── toadstool.rs        ❌ Hardcoded ToadStool
```

### **Good News**: Capability-Based Replacements Already Exist! ✅
```
crates/songbird-primal-sdk/src/
├── capability_security.rs        ✅ Replaces beardog
├── capability_ai.rs               ✅ Replaces squirrel
├── capability_compute.rs          ✅ Replaces toadstool
├── security_capability_client.rs  ✅ Security client
├── ai_capability.rs               ✅ AI client
├── compute_capability.rs          ✅ Compute client
```

### **Status**: Migration 70% Complete!
- ✅ New capability modules implemented
- ✅ Client abstractions created
- ⏸️ Old primal-specific modules still present
- ⏸️ Code still references old modules

---

## 📋 **CLEANUP TASKS**

### **Phase 1: Deprecation** (Week 1, Days 1-2)

#### **Task 1.1: Add Deprecation Warnings**
```rust
// In beardog.rs
#![deprecated(
    since = "0.5.0",
    note = "Use `capability_security` module instead. This module hardcodes BearDog and will be removed in v1.0.0"
)]

// In squirrel.rs
#![deprecated(
    since = "0.5.0",
    note = "Use `capability_ai` module instead. This module hardcodes Squirrel and will be removed in v1.0.0"
)]

// In toadstool.rs
#![deprecated(
    since = "0.5.0",
    note = "Use `capability_compute` module instead. This module hardcodes ToadStool and will be removed in v1.0.0"
)]
```

#### **Task 1.2: Update Cargo.toml**
```toml
# BEFORE:
keywords = ["songbird", "primal", "sdk", "integration", "orchestration", "universal", "beardog"]

# AFTER:
keywords = ["songbird", "capability", "sdk", "integration", "orchestration", "universal", "discovery"]
```

#### **Task 1.3: Update lib.rs Documentation**
```rust
// Enhance the deprecation notice in lib.rs
// ============================================================================
// ⚠️ DEPRECATED: Primal-Specific Modules (Remove in v1.0.0)
// ============================================================================
// 
// The following modules hardcode specific primal names and violate our
// universal adapter pattern. Use capability-based alternatives:
//
// OLD (Deprecated)          → NEW (Capability-Based)
// ----------------------------------------
// mod beardog;             → capability_security
// mod squirrel;            → capability_ai  
// mod toadstool;           → capability_compute
//
// Migration deadline: v1.0.0 (target: March 2026)
```

---

### **Phase 2: Find and Update References** (Week 1, Days 3-5)

#### **Task 2.1: Scan Codebase for Usage**
```bash
# Find all references to primal-specific modules
cd /home/eastgate/Development/ecoPrimals/songbird

# BearDog references
grep -r "use.*beardog" --include="*.rs" crates/ > beardog_refs.txt
grep -r "BearDog" --include="*.rs" crates/ >> beardog_refs.txt

# Squirrel references  
grep -r "use.*squirrel" --include="*.rs" crates/ > squirrel_refs.txt
grep -r "Squirrel" --include="*.rs" crates/ >> squirrel_refs.txt

# ToadStool references
grep -r "use.*toadstool" --include="*.rs" crates/ > toadstool_refs.txt
grep -r "ToadStool" --include="*.rs" crates/ >> toadstool_refs.txt
```

#### **Task 2.2: Create Migration Guide**
Create `PRIMAL_SDK_MIGRATION_GUIDE.md` with examples:

```rust
// ❌ OLD: Hardcoded primal name
use songbird_primal_sdk::beardog::BearDogClient;

let client = BearDogClient::new("http://beardog:8081").await?;
client.authenticate(credentials).await?;

// ✅ NEW: Capability-based discovery
use songbird_primal_sdk::capability_security::SecurityCapabilityClient;

let client = SecurityCapabilityClient::from_discovery().await?;
client.authenticate(credentials).await?;
```

#### **Task 2.3: Update Internal References**
Update all references within songbird-primal-sdk:
- `discovery/` modules using primal names
- `universal_adapter/` routing logic
- Test files
- Example code

---

### **Phase 3: Update Dependent Code** (Week 2)

#### **Task 3.1: Update Test Utilities**
Location: `crates/songbird-test-utils/src/mocks/`
- `mocks/beardog.rs` → Keep as test mock (acceptable)
- `mocks/squirrel.rs` → Keep as test mock (acceptable)
- `mocks/toadstool.rs` → Keep as test mock (acceptable)
- `mocks/nestgate.rs` → Keep as test mock (acceptable)

**Note**: Test mocks are fine! They simulate specific implementations for testing.

#### **Task 3.2: Update Examples**
Location: `examples/`
- Update to use capability-based discovery
- Show both old and new patterns during transition

#### **Task 3.3: Update Documentation**
- `README.md` - Show capability-based examples
- `specs/` - Update integration specifications
- Code comments - Remove primal-specific references

---

### **Phase 4: Remove Deprecated Modules** (Week 3)

#### **Task 4.1: Final Verification**
```bash
# Ensure no production code uses deprecated modules
cargo test --workspace
cargo clippy --workspace -- -W deprecated
```

#### **Task 4.2: Remove Files**
```bash
cd crates/songbird-primal-sdk/src/
git rm beardog.rs
git rm squirrel.rs  
git rm toadstool.rs
```

#### **Task 4.3: Update lib.rs**
Remove the deprecated module exports:
```rust
// Remove these lines:
// pub mod beardog;
// pub mod squirrel;
// pub mod toadstool;
```

#### **Task 4.4: Version Bump**
Update to v0.5.0 (deprecation) then v1.0.0 (removal)

---

## 📊 **MIGRATION TRACKING**

### **Files to Modify**

| File | Action | Priority | Estimate |
|------|--------|----------|----------|
| `Cargo.toml` | Update keywords | P1 | 5 min |
| `lib.rs` | Add deprecation docs | P1 | 30 min |
| `beardog.rs` | Deprecate | P1 | 10 min |
| `squirrel.rs` | Deprecate | P1 | 10 min |
| `toadstool.rs` | Deprecate | P1 | 10 min |
| `discovery/*` | Update refs | P1 | 4 hours |
| `examples/*` | Update examples | P2 | 2 hours |
| `specs/*` | Update docs | P2 | 2 hours |
| Remove old files | Delete | P1 | 30 min |

**Total Estimate**: 10-12 hours of focused work

---

## ✅ **SUCCESS CRITERIA**

### **Phase 1 Complete** ✅
- [ ] All primal-specific modules marked deprecated
- [ ] Cargo.toml updated (remove "beardog" keyword)
- [ ] Documentation shows capability-based pattern

### **Phase 2 Complete** ✅
- [ ] All references found and documented
- [ ] Migration guide created
- [ ] Internal references updated

### **Phase 3 Complete** ✅
- [ ] Examples use capability-based discovery
- [ ] Documentation updated
- [ ] Tests updated (mocks can stay)

### **Phase 4 Complete** ✅
- [ ] Deprecated files removed
- [ ] No compilation errors
- [ ] All tests passing
- [ ] Version bumped to v1.0.0

---

## 🎯 **BACKWARD COMPATIBILITY**

### **During Deprecation Period** (v0.5.0 - v0.9.0)
```rust
// Both work (with deprecation warnings on old way):
use songbird_primal_sdk::beardog::BearDogClient;  // ⚠️ Deprecated
use songbird_primal_sdk::capability_security::SecurityCapabilityClient;  // ✅ Recommended
```

### **After Removal** (v1.0.0+)
```rust
// Only capability-based works:
use songbird_primal_sdk::capability_security::SecurityCapabilityClient;  // ✅ Only way
```

---

## 📋 **TESTING PLAN**

### **Unit Tests**
- [ ] All capability modules have tests
- [ ] Discovery mechanisms tested
- [ ] Backward compatibility during deprecation

### **Integration Tests**
- [ ] Capability-based discovery works end-to-end
- [ ] Old code shows deprecation warnings
- [ ] New code compiles without warnings

### **E2E Tests**
- [ ] Full workflows use capability discovery
- [ ] No hardcoded primal names in logs/traces
- [ ] Dynamic provider selection works

---

## 🚀 **ROLLOUT PLAN**

### **v0.5.0: Deprecation** (This Week)
- Mark old modules deprecated
- Add capability-based examples
- Update documentation
- **Release notes**: "Primal-specific modules deprecated, use capability-based alternatives"

### **v0.6.0 - v0.9.0: Transition** (Months 1-2)
- Continue supporting both patterns
- Help users migrate
- Monitor usage

### **v1.0.0: Clean Slate** (Month 3)
- Remove deprecated modules
- 100% capability-based
- **Release notes**: "Clean, capability-based architecture achieved"

---

## 💡 **BENEFITS**

### **After Cleanup**:
1. ✅ **True Universal Pattern**: No hardcoded primal names
2. ✅ **Better Scalability**: New primals work automatically
3. ✅ **Sovereignty Compliant**: Users choose providers
4. ✅ **Cleaner Codebase**: Reduced maintenance burden
5. ✅ **Better Testing**: Capability-based mocking
6. ✅ **Production Ready**: No architectural violations

---

## 📞 **NEXT STEPS**

### **Immediate** (This Session)
- [ ] Create deprecation warnings
- [ ] Update Cargo.toml
- [ ] Scan for references

### **This Week**
- [ ] Complete Phase 1 (Deprecation)
- [ ] Start Phase 2 (Find References)
- [ ] Create migration guide

### **Next 2 Weeks**
- [ ] Complete Phase 2 & 3
- [ ] Update all code
- [ ] Test thoroughly

### **Week 3**
- [ ] Phase 4 (Removal)
- [ ] Release v1.0.0
- [ ] Update STATUS.md

---

**Timeline**: 2-3 weeks for complete cleanup  
**Effort**: 10-12 hours focused work  
**Risk**: Low (capability modules already exist)  
**Reward**: True universal architecture achieved! 🎉

