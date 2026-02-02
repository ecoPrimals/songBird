# 🗂️ Archive Review - February 2, 2026

**Purpose**: Identify archivable code, outdated TODOs, and cleanup opportunities  
**Strategy**: Keep docs as fossil record in `ecoPrimals/`, clean production code  
**Deployment**: Push via SSH after review

---

## 📊 **CURRENT STATE ANALYSIS**

### **TODO Comments**: 89 found across 51 files
- Most are legitimate (future enhancements, phase markers)
- Need to review for outdated items

### **Deprecated Code**: 32 `#[deprecated]` attributes found
- Primarily in `songbird-config` module
- Backward compatibility maintained through Q1 2026

### **Archived Specs**: 1 file
- `specs/archive/deprecated-protocols/GRPC_GATEWAY_ADAPTER_SPECIFICATION.md.deprecated`

### **Test Infrastructure**: 25 test files with ignored/skipped tests
- Need to review if these are intentional or outdated

---

## 🎯 **CLEANUP CATEGORIES**

### **1. Deprecated Config Module** (HIGH PRIORITY)

**Location**: `crates/songbird-config/src/config/`

**Status**: Marked deprecated since v0.2.0, migration path to `canonical::`

**Remaining Usage**: 8 files found
1. `crates/songbird-universal/src/communication.rs` ⚠️ **External usage**
2. `crates/songbird-config/src/lib.rs` (self-reference)
3. `crates/songbird-config/src/config/mod.rs` (internal)
4. `crates/songbird-config/src/config/constants.rs` (internal)
5. `crates/songbird-config/src/config/universal_primals.rs` (internal)
6. `crates/songbird-config/src/config/environment.rs` (internal)
7. `crates/songbird-config/src/primal_discovery.rs` (self-reference)
8. `crates/songbird-config/tests/config_basic_tests.rs` (test)

**Critical Finding**: Only **1 external usage** in production code (already commented out!)
- ✅ `songbird-universal/src/communication.rs` - Line 16 is commented out

**Action Items**:
- [ ] Remove commented import from `songbird-universal/src/communication.rs`
- [ ] Migrate test file `songbird-config/tests/config_basic_tests.rs`
- [ ] After Q1 2026, archive entire `config::` module to `ecoPrimals/code-archive/`

**Timeline**: Can remove after Q1 2026 (March 31, 2026)

---

### **2. Outdated TODOs** (MEDIUM PRIORITY)

**Files with Multiple TODOs**:
- `songbird-tls/src/cert/mod.rs` - 6 TODOs
- `songbird-universal/tests/integration_workflow_tests.rs` - 5 TODOs
- `rendezvous/src/api.rs` - 5 TODOs
- `songbird-bluetooth/src/gatt.rs` - 4 TODOs
- `songbird-genesis/src/physical_channels/*.rs` - 3 TODOs each

**Action Items**:
- [ ] Review each TODO for currency
- [ ] Convert valid TODOs to GitHub issues
- [ ] Remove completed or obsolete TODOs
- [ ] Document architectural decisions for "TODO: Future" items

---

### **3. Legacy Examples** (LOW PRIORITY)

**Candidates for Archive**:
- `examples/infant_discovery_demo.rs` - Has "OLD:" comments
- `examples/integration/ecosystem-primals/*.rs` - May be superseded
- `showcase/05-albatross-multiplex/` - Review if still relevant

**Action Items**:
- [ ] Test if examples still compile
- [ ] Verify if examples demonstrate current patterns
- [ ] Archive outdated examples to `ecoPrimals/code-archive/examples/`
- [ ] Keep examples that demonstrate production patterns

---

### **4. Test Cleanup** (MEDIUM PRIORITY)

**Findings**:
- 25 test files with `#[ignore]` or `#[should_panic]`
- Some may be integration tests requiring specific setup

**Action Items**:
- [ ] Review ignored tests for relevance
- [ ] Update tests to work with current architecture
- [ ] Archive obsolete tests to `ecoPrimals/code-archive/tests/`
- [ ] Document why certain tests are ignored

---

### **5. Deprecated Specs** (LOW PRIORITY)

**Found**:
- `specs/archive/deprecated-protocols/GRPC_GATEWAY_ADAPTER_SPECIFICATION.md.deprecated`

**Status**: Already in archive, properly documented

**Action Items**:
- [ ] Move to `ecoPrimals/specs-archive/` for fossil record
- [ ] Ensure INDEX.md references this spec
- [ ] Keep for historical context (no code removal needed)

---

## 🗂️ **PROPOSED ARCHIVE STRUCTURE**

```
ecoPrimals/
├── code-archive/
│   ├── songbird-config-legacy/     ← Deprecated config module (Q1 2026)
│   ├── examples/                   ← Outdated example code
│   ├── tests/                      ← Obsolete test cases
│   └── INDEX.md                    ← Archive catalog
│
├── specs-archive/
│   ├── grpc-gateway/               ← gRPC spec (already archived)
│   └── INDEX.md                    ← Spec catalog
│
└── sessions/                       ← Session docs (already organized)
    ├── feb-01-2026/
    ├── feb-01-2026-final/
    └── jan-31-2026/
```

---

## 📋 **DETAILED REVIEW FINDINGS**

### **A. songbird-config Module**

**Deprecated Since**: v0.2.0  
**Replacement**: `canonical::` module  
**Usage Count**: **8 files total, 0 active external!** ✅
- ✅ `songbird-universal/src/communication.rs` - Already commented out (line 16)
- 6 internal files in `songbird-config` itself
- 1 test file

**Migration Status**:
- ✅ No active external dependencies!
- `canonical::NetworkConfig` ✅ Available
- `canonical::EnvironmentConfig` ✅ Available
- `canonical::ServiceConfig` ✅ Available
- `canonical::SecurityConfig` ✅ Available

**Recommendation**: 
1. Audit all uses of `config::` imports
2. Migrate to `canonical::`
3. Archive old module after March 2026
4. Keep fossil record with migration guide

---

### **B. TODO Categories**

#### **Category 1: Feature Enhancements** (Keep)
Examples:
- Phase 2/3 placeholders
- Future protocol support
- Planned optimizations

**Action**: Convert to GitHub issues, keep TODOs with issue references

#### **Category 2: Implementation Notes** (Review)
Examples:
- "TODO: Better error handling"
- "TODO: Add validation"
- "TODO: Optimize this"

**Action**: Implement or document decision not to implement

#### **Category 3: Outdated** (Remove)
Examples:
- References to removed features
- Old architecture comments
- Completed migrations

**Action**: Remove and document in commit

---

### **C. Example Code Audit**

#### **Keep** (Production Patterns):
- `capability_discovery_example.rs` ✅
- `capability_based_configuration.rs` ✅
- `modern_configuration_complete.rs` ✅
- `genetic_lineage_usage.rs` ✅

#### **Review** (May Be Outdated):
- `infant_discovery_demo.rs` - Contains "OLD:" comments
- `integration/ecosystem-primals/*.rs` - Check relevance
- `showcase/05-albatross-multiplex/` - Verify currency

#### **Archive** (If Obsolete):
- Examples demonstrating deprecated patterns
- Examples that no longer compile
- Examples superseded by better demos

---

### **D. Test Infrastructure**

**Ignored Tests** (25 files):
- Integration tests requiring external services
- Performance tests requiring specific hardware
- Tests for future features

**Recommendation**:
1. Keep tests that will be enabled in deployment
2. Archive tests for removed features
3. Document why specific tests are ignored
4. Create GitHub issues for "future feature" tests

---

## 📋 **QUICK WINS IDENTIFIED**

### **1. Remove Commented Import** (30 seconds)
**File**: `crates/songbird-universal/src/communication.rs:16`
```rust
// Remove this line:
// use songbird_config::config::hardcoded_elimination::replace;
```
**Impact**: Clean up last external reference to deprecated module

### **2. Archive Deprecated Spec** (5 minutes)
**Source**: `specs/archive/deprecated-protocols/GRPC_GATEWAY_ADAPTER_SPECIFICATION.md.deprecated`  
**Destination**: `ecoPrimals/specs-archive/grpc-gateway/`  
**Impact**: Properly organize fossil record

### **3. Update Config Test** (15 minutes)
**File**: `crates/songbird-config/tests/config_basic_tests.rs`  
**Action**: Migrate test to use `canonical::` instead of `config::`  
**Impact**: Remove last external test dependency

---

## 🎯 **EXECUTION SUMMARY**

### **Key Finding**: Migration is 99% complete! ✅
- Zero active external dependencies on deprecated `config::` module
- Only 1 commented-out line to clean up
- All other uses are internal to the deprecated module itself

### **Immediate Actions Available**:
1. **Remove commented import** (30 sec) - Low risk
2. **Archive deprecated spec** (5 min) - Zero risk  
3. **Migrate test file** (15 min) - Low risk
4. **Create archive structure** (10 min) - Zero risk

**Total Time**: ~30 minutes for quick wins

---

## 🚀 **EXECUTION PLAN**

### **Phase 1: Immediate (This Session)** ✅
1. ✅ Create this review document
2. ✅ Audit deprecated config module usage (0 active external uses!)
3. [ ] Identify quick-win cleanups (obvious obsolete code)
4. [ ] Create archive structure in `ecoPrimals/code-archive/`
5. [ ] Create actionable cleanup plan

### **Phase 2: Quick Wins (Next 30 min)**
1. [ ] Review all 89 TODOs systematically
2. [ ] Create GitHub issues for valid future work
3. [ ] Remove obsolete TODOs
4. [ ] Document architectural decisions

### **Phase 3: Config Migration (Before March 2026)**
1. [ ] Complete migration to `canonical::`
2. [ ] Remove deprecated `config::` module
3. [ ] Archive old module with migration guide
4. [ ] Update all documentation

### **Phase 4: Example & Test Cleanup (Ongoing)**
1. [ ] Test all examples for compilation
2. [ ] Archive outdated examples
3. [ ] Review ignored tests
4. [ ] Document test infrastructure

---

## 📈 **METRICS**

| Metric | Count | Status |
|--------|-------|--------|
| **TODO Comments** | 89 | 🟡 Review needed |
| **Deprecated Attributes** | 32 | 🟡 Migration in progress |
| **Deprecated Files** | 1 | 🟢 Already archived |
| **Ignored Tests** | 25 | 🟡 Review needed |
| **Legacy Examples** | ~6 | 🟡 Review needed |

---

## ✅ **SUCCESS CRITERIA**

**Code Cleanliness**:
- [ ] Zero obsolete TODOs
- [ ] Deprecated code properly archived
- [ ] All examples compile and demonstrate current patterns
- [ ] Test infrastructure documented

**Documentation**:
- [ ] Archive catalog (INDEX.md) complete
- [ ] Migration guides preserved
- [ ] Fossil record maintained

**Production Readiness**:
- [ ] No confusing deprecated code in main tree
- [ ] Clear separation: production vs. archive
- [ ] Easy navigation for new developers

---

## 🎯 **RECOMMENDATIONS**

### **High Priority**:
1. **Config Module Migration**: Complete before Q1 2026 deadline
2. **TODO Audit**: Create GitHub issues, remove obsolete items
3. **Archive Structure**: Establish clear organization

### **Medium Priority**:
4. **Test Review**: Understand ignored tests, document or fix
5. **Example Validation**: Ensure examples are current

### **Low Priority**:
6. **Spec Archive**: Move deprecated specs to ecoPrimals
7. **Historical Preservation**: Document evolution decisions

---

## 📝 **NEXT STEPS**

**Immediate Actions**:
1. Count uses of deprecated `config::` module
2. Create archive directory structure
3. Review top 10 TODO files
4. Identify quick-win removals

**This Session Goals**:
- [ ] Complete deprecated config usage audit
- [ ] Create archive structure
- [ ] Archive 1-2 obsolete items as examples
- [ ] Document process for future cleanups

---

## 💬 **NOTES**

**Philosophy**: 
- Keep fossil record (ecoPrimals/)
- Clean production code
- Document decisions
- Maintain migration paths

**Timeline**:
- Q1 2026: Complete config migration
- Ongoing: TODO review and cleanup
- As needed: Archive obsolete code

---

*Status*: Initial review complete, ready for execution  
*Next*: Audit deprecated config module usage
