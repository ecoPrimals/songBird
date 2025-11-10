# Config Consolidation Roadmap
**Created:** November 7, 2025  
**Scope:** songbird-config crate consolidation  
**Size:** 61 files, 19,450 lines of code  
**Estimated Effort:** 16-20 hours

---

## 🎯 **CONSOLIDATION OBJECTIVES**

### Primary Goals
1. **Establish `canonical/` as single source of truth** for all config types
2. **Eliminate duplicate config structs** across config/, unified/, and root level
3. **Clear migration paths** from old to new locations
4. **Zero breaking changes** to public APIs (use re-exports)

### Success Metrics
- All config types have ONE canonical definition
- Clear module hierarchy: `canonical/` → `config/` (deprecated) → root (legacy)
- Zero build errors or test failures
- Documentation showing migration paths

---

## 📊 **CURRENT STATE ANALYSIS**

### Module Structure (Before)
```
songbird-config/src/
├── canonical/           (9 files, ~2.5k lines) ✅ TARGET
│   ├── constants.rs
│   ├── environment.rs
│   ├── load_balancing.rs
│   ├── network.rs
│   ├── primals.rs
│   ├── resilience.rs
│   ├── security.rs
│   └── service.rs
│
├── config/              (15 files, ~7k lines) ⚠️ CONSOLIDATE
│   ├── agnostic_primals.rs (750 lines)
│   ├── constants.rs (706 lines)
│   ├── environment.rs (475 lines)
│   ├── hardcoded_elimination.rs (486 lines)
│   ├── network/ (919 lines + tests)
│   ├── network_endpoints.rs
│   ├── paths.rs (579 lines)
│   ├── providers.rs
│   ├── universal_primals.rs (631 lines)
│   ├── universal_primals_clean.rs
│   ├── validation.rs (disabled)
│   └── validation_clean.rs
│
├── unified/             (13 files, ~4k lines) ⚠️ DISABLED (errors)
│   ├── api.rs
│   ├── cli.rs
│   ├── core.rs
│   ├── discovery.rs
│   ├── federation.rs
│   ├── network.rs (771 lines)
│   ├── observability.rs
│   ├── performance.rs
│   ├── primals.rs
│   ├── robustness.rs
│   ├── security.rs (473 lines)
│   └── testing.rs
│
├── zero_touch/          (6 files, ~3k lines) ✅ KEEP (deployment-specific)
│   ├── config.rs
│   ├── deployment.rs
│   ├── environment.rs
│   ├── infant_config.rs
│   ├── network.rs
│   └── mod.rs
│
├── defaults/            (5 files, ~1k lines) ✅ KEEP (constants)
│   ├── endpoints.rs
│   ├── hosts.rs
│   ├── ports.rs
│   ├── timeouts.rs
│   └── mod.rs
│
└── Root Level           (15 files, ~4k lines) ⚠️ CONSOLIDATE
    ├── canonical_network.rs (→ move to canonical/)
    ├── capability_endpoints.rs (593 lines) ✅ KEEP
    ├── discoverable_endpoint.rs (524 lines) ✅ KEEP
    ├── environment.rs
    ├── environment_config_clean.rs
    ├── gaming.rs
    ├── hardcoded_elimination.rs
    ├── performance.rs
    ├── self_discovery.rs
    ├── zero_hardcoding_migration.rs (623 lines)
    └── zero_touch_config.rs (716 lines)
```

### Duplication Analysis

| Module | Locations | Total Lines | Status |
|--------|-----------|-------------|--------|
| **Network Config** | config/network/, canonical/network.rs, unified/network.rs, canonical_network.rs, zero_touch/network.rs | ~3,400 | ⚠️ 4 duplicates |
| **Environment Config** | config/environment.rs, canonical/environment.rs, zero_touch/environment.rs, environment.rs, environment_config_clean.rs | ~2,200 | ⚠️ 5 duplicates |
| **Primal Config** | config/agnostic_primals.rs, config/universal_primals.rs, canonical/primals.rs, unified/primals.rs | ~2,100 | ⚠️ 4 duplicates |
| **Constants** | config/constants.rs, canonical/constants.rs, defaults/* | ~1,700 | ⚠️ 2+ duplicates |
| **Security Config** | canonical/security.rs, unified/security.rs | ~700 | ⚠️ 2 duplicates |
| **Validation** | config/validation.rs (disabled), config/validation_clean.rs | ~400 | ⚠️ Fragmented |

**Total Duplication:** ~10,500 lines (54% of codebase)

---

## 🚀 **CONSOLIDATION STRATEGY**

### **PHASE 1: Foundation & Quick Wins** (2-3 hours)
**Goal:** Establish canonical structure and deprecate obvious duplicates

#### 1.1: Move Standalone Files into Canonical
- [ ] Move `canonical_network.rs` → `canonical/network_extended.rs` or merge with existing
- [ ] Review and integrate useful parts
- [ ] Update `canonical/mod.rs` exports

#### 1.2: Create Deprecation Notices
- [ ] Add deprecation notices to `config/` modules with migration paths
- [ ] Document which canonical module replaces each config module
- [ ] Add `#[deprecated]` attributes where appropriate

#### 1.3: Update lib.rs Re-exports
- [ ] Prefer `canonical::*` over `config::*` in public API
- [ ] Add re-exports for backward compatibility
- [ ] Document migration path in lib.rs

---

### **PHASE 2: Network Config Consolidation** (3-4 hours)
**Goal:** Single canonical network configuration

#### 2.1: Analyze Network Configs
- [ ] Compare `config/network/mod.rs` (919 lines)
- [ ] Compare `canonical/network.rs` (existing)
- [ ] Compare `canonical_network.rs` (standalone)
- [ ] Compare `unified/network.rs` (771 lines, disabled)
- [ ] Identify unique functionality in each

#### 2.2: Merge Network Configs
- [ ] Create comprehensive `canonical/network.rs`
- [ ] Include all unique features from each source
- [ ] Add tests for merged functionality
- [ ] Update type exports

#### 2.3: Deprecate Old Network Configs
- [ ] Mark `config/network/` as deprecated
- [ ] Add re-export from canonical
- [ ] Update all internal imports
- [ ] Run tests to verify

---

### **PHASE 3: Environment Config Consolidation** (2-3 hours)
**Goal:** Single canonical environment configuration

#### 3.1: Analyze Environment Configs
- [ ] Compare all 5 environment config locations
- [ ] Identify unique functionality
- [ ] Note which is most complete

#### 3.2: Merge Environment Configs
- [ ] Enhance `canonical/environment.rs`
- [ ] Migrate unique features from each source
- [ ] Keep `zero_touch/environment.rs` separate (deployment-specific)
- [ ] Add comprehensive tests

#### 3.3: Deprecate Old Environment Configs
- [ ] Mark `config/environment.rs` as deprecated
- [ ] Remove or repurpose root-level `environment.rs`
- [ ] Update imports throughout codebase

---

### **PHASE 4: Primal Config Consolidation** (3-4 hours)
**Goal:** Single canonical primal registry system

#### 4.1: Analyze Primal Configs
- [ ] Review `config/agnostic_primals.rs` (750 lines)
- [ ] Review `config/universal_primals.rs` (631 lines)
- [ ] Review `canonical/primals.rs` (current)
- [ ] Identify best patterns from each

#### 4.2: Merge Primal Configs
- [ ] Create comprehensive `canonical/primals.rs`
- [ ] Migrate capability-based patterns
- [ ] Keep simple, practical implementations
- [ ] Avoid over-engineering (per spec)

#### 4.3: Update SongbirdConfig
- [ ] Update `config/mod.rs` to use canonical primals
- [ ] Deprecate old primal modules
- [ ] Update all primal registry references

---

### **PHASE 5: Constants & Validation** (2-3 hours)
**Goal:** Unified constants and validation systems

#### 5.1: Constants Consolidation
- [ ] Merge `config/constants.rs` into `canonical/constants.rs`
- [ ] Keep `defaults/*` for actual default values
- [ ] Clear distinction: canonical = types, defaults = values

#### 5.2: Validation System
- [ ] Fix or remove `config/validation.rs` (currently disabled)
- [ ] Keep `validation_clean.rs` if it's better
- [ ] Consider moving to `canonical/validation.rs`
- [ ] Re-enable validation in config/mod.rs

#### 5.3: Security Config
- [ ] Merge `unified/security.rs` features into `canonical/security.rs`
- [ ] Update security types and patterns
- [ ] Ensure all security features available

---

### **PHASE 6: Cleanup & Documentation** (3-4 hours)
**Goal:** Remove duplicates, update docs, verify tests

#### 6.1: Remove Deprecated Files
- [ ] Delete fully migrated config/ files
- [ ] Remove unified/ directory (if all migrated)
- [ ] Clean up root-level redundant files
- [ ] Keep only: canonical/, defaults/, zero_touch/, lib.rs

#### 6.2: Update All Imports
- [ ] Search for `use crate::config::` → `use crate::canonical::`
- [ ] Update all test files
- [ ] Update examples and documentation
- [ ] Run `cargo fix` for automated updates

#### 6.3: Documentation
- [ ] Update lib.rs module documentation
- [ ] Create MIGRATION_GUIDE.md for users
- [ ] Add examples for common use cases
- [ ] Update README if needed

#### 6.4: Verification
- [ ] `cargo build --workspace` (zero errors)
- [ ] `cargo test --workspace` (all pass)
- [ ] `cargo clippy` (zero warnings)
- [ ] Manual smoke tests

---

## 📋 **EXPECTED OUTCOMES**

### Target Structure (After)
```
songbird-config/src/
├── canonical/           (12 files, ~12k lines) ⭐ SINGLE SOURCE OF TRUTH
│   ├── constants.rs     (merged)
│   ├── environment.rs   (merged)
│   ├── load_balancing.rs
│   ├── network.rs       (merged from 4 sources)
│   ├── observability.rs (from unified/)
│   ├── paths.rs         (from config/)
│   ├── primals.rs       (merged from 3 sources)
│   ├── resilience.rs
│   ├── security.rs      (merged)
│   ├── service.rs
│   ├── validation.rs    (fixed and moved)
│   └── mod.rs
│
├── defaults/            (5 files) ✅ KEEP
│   └── (actual default values, not types)
│
├── zero_touch/          (6 files) ✅ KEEP
│   └── (deployment-specific configs)
│
├── capability_endpoints.rs ✅ KEEP
├── discoverable_endpoint.rs ✅ KEEP
├── lib.rs               (updated exports)
└── [deprecated]/        (optional: archived old files)
```

### Metrics
- **Files:** 61 → ~30 (-51%)
- **Duplicate Lines:** 10,500 → 0 (-100%)
- **Canonical Lines:** 2,500 → 12,000 (+380%)
- **Module Depth:** 4 levels → 2 levels (-50%)

---

## 🎯 **MIGRATION GUIDE PREVIEW**

```rust
// ❌ OLD (deprecated)
use songbird_config::config::NetworkConfig;
use songbird_config::config::environment::EnvironmentConfig;

// ✅ NEW (canonical)
use songbird_config::canonical::{NetworkConfig, EnvironmentConfig};

// 🔄 BACKWARD COMPATIBLE (re-exports maintained)
use songbird_config::NetworkConfig; // Still works via lib.rs re-export
```

---

## ⚠️ **RISKS & MITIGATION**

### Risk 1: Breaking Changes
**Mitigation:** Maintain re-exports in lib.rs for all public types

### Risk 2: Test Failures
**Mitigation:** Run tests after each phase, rollback if needed

### Risk 3: Import Hell
**Mitigation:** Use `cargo fix --allow-dirty` to auto-update imports

### Risk 4: Merge Conflicts
**Mitigation:** Complete consolidation in dedicated session, avoid concurrent work

---

## 📈 **PROGRESS TRACKING**

| Phase | Tasks | Est. Hours | Status |
|-------|-------|------------|--------|
| Phase 1: Foundation | 3 | 2-3h | 🟡 In Progress |
| Phase 2: Network | 3 | 3-4h | ⚪ Pending |
| Phase 3: Environment | 3 | 2-3h | ⚪ Pending |
| Phase 4: Primals | 3 | 3-4h | ⚪ Pending |
| Phase 5: Constants | 3 | 2-3h | ⚪ Pending |
| Phase 6: Cleanup | 4 | 3-4h | ⚪ Pending |
| **TOTAL** | **19** | **16-20h** | **5% Complete** |

---

## 🎯 **NEXT ACTIONS**

### Immediate (This Session)
1. ✅ Create this roadmap
2. 🟡 Move `canonical_network.rs` into `canonical/`
3. 🟡 Add deprecation notices to key config/ modules
4. 🟡 Update lib.rs to prefer canonical exports

### Next Session
5. Network config consolidation (Phase 2)
6. Environment config consolidation (Phase 3)

---

**Status:** Roadmap complete, beginning Phase 1 execution...

