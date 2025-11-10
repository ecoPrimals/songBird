# HealthCheckConfig Consolidation Plan
**Date**: November 10, 2025  
**Status**: Execution Ready  
**Impact**: 19 definitions → 2-3 definitions (85% reduction)

---

## 🎯 ANALYSIS

### **Current State: 19 Definitions**

#### **Canonical Versions** (KEEP - 2 definitions)

1. **`canonical/resilience.rs:406`** - For resilience/circuit breaking
   ```rust
   pub struct HealthCheckConfig {
       pub enabled: bool,
       pub interval_secs: u64,
       pub timeout_secs: u64,
       pub failure_threshold: u32,
       pub recovery_threshold: u32,  // Unique to resilience
       pub path: String,
   }
   ```
   **Purpose**: Backend health monitoring for circuit breakers
   **Keep**: YES ✅

2. **`canonical/primals.rs:242`** - For primal provider health checks
   ```rust
   pub struct HealthCheckConfig {
       pub enabled: bool,
       pub interval: Duration,
       pub endpoint_path: String,
       pub expected_status_codes: Vec<u16>,  // Unique to primals
       pub timeout: Duration,
       pub failure_threshold: u32,
   }
   ```
   **Purpose**: Primal provider HTTP health checks
   **Keep**: YES ✅

**Decision**: Keep both, but rename for clarity:
- `canonical/resilience.rs` → `ResilienceHealthCheckConfig`
- `canonical/primals.rs` → `PrimalHealthCheckConfig`

#### **Duplicates to REMOVE** (17 definitions)

| File | Line | Action |
|------|------|--------|
| `songbird-types/src/config/performance.rs` | 573 | REMOVE - use canonical/resilience |
| `songbird-types/src/config/discovery_corrupted.rs` | 245 | REMOVE - corrupted file? |
| `songbird-types/src/config/discovery.rs` | 247 | REMOVE - use canonical/resilience |
| `songbird-orchestrator/src/core/robustness/config.rs` | 189 | REMOVE - use canonical/resilience |
| `songbird-orchestrator/src/core/traits/health.rs` | 69 | REMOVE - use canonical |
| `songbird-observability/src/health/production_health.rs` | 60 | MIGRATE - may need observability-specific |
| `songbird-registry/src/types/health.rs` | 119 | REMOVE - use canonical/primals |
| `songbird-universal/src/types/config.rs` | 117 | REMOVE - use canonical |
| `songbird-discovery/src/traits/health.rs` | 26 | REMOVE - use canonical |
| `songbird-config/src/config/mod.rs` | 495 | REMOVE - deprecated location |
| `songbird-config/src/config/universal_primals_clean.rs` | 38 | REMOVE - use canonical/primals |
| `songbird-config/src/canonical/service.rs` | 36 | REMOVE - duplicate of resilience |
| `songbird-config/src/unified/robustness.rs` | 265 | REMOVE - merged into canonical/resilience |
| `songbird-primal-sdk/src/modern_api.rs` | 388 | REMOVE - use canonical/primals |
| `songbird-primal-sdk/src/config.rs` | 162 | REMOVE - use canonical/primals |

#### **Investigate**

| File | Line | Question |
|------|------|----------|
| `songbird-orchestrator/src/core/api/universal_service_registration/types.rs` | 213 | HealthCheckConfiguration (different name) - consolidate? |
| `songbird-config/src/unified/api.rs` | 225 | HealthCheckConfiguration - consolidate? |

---

## 🚀 EXECUTION PLAN

### **Phase 1: Rename Canonical Versions for Clarity** (30 min)

```bash
# Step 1: Rename in resilience.rs
# Change: pub struct HealthCheckConfig
# To:     pub struct ResilienceHealthCheckConfig

# Step 2: Rename in primals.rs  
# Change: pub struct HealthCheckConfig
# To:     pub struct PrimalHealthCheckConfig

# Step 3: Add re-export for backward compat (temporary)
# In canonical/mod.rs:
# pub use resilience::ResilienceHealthCheckConfig as HealthCheckConfig;
```

### **Phase 2: Update High-Impact Files** (1 hour)

#### Files using resilience health checks:
```rust
// UPDATE: songbird-orchestrator/src/core/robustness/config.rs:189
// OLD:
// pub struct HealthCheckConfig { ... }

// NEW:
pub use songbird_config::canonical::resilience::ResilienceHealthCheckConfig as HealthCheckConfig;
```

#### Files using primal health checks:
```rust
// UPDATE: songbird-registry/src/types/health.rs:119
// OLD:
// pub struct HealthCheckConfig { ... }

// NEW:
pub use songbird_config::canonical::primals::PrimalHealthCheckConfig as HealthCheckConfig;
```

### **Phase 3: Remove Duplicate Definitions** (1-2 hours)

```bash
# For each file in "Duplicates to REMOVE":
# 1. Replace struct definition with use statement
# 2. Update any field accesses if field names differ
# 3. Run: cargo check --package <affected-package>
# 4. Run: cargo test --package <affected-package>
# 5. Commit

# Example for songbird-types/src/config/performance.rs:573:
# BEFORE:
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub enabled: bool,
    pub interval_secs: u64,
    // ... fields
}

# AFTER:
pub use songbird_config::canonical::resilience::ResilienceHealthCheckConfig as HealthCheckConfig;
```

### **Phase 4: Validation** (30 min)

```bash
# Full workspace check
cargo check --workspace

# Run tests
cargo test --workspace

# Check for remaining duplicates
grep -rn "pub struct HealthCheckConfig" crates --include="*.rs" | grep -v "test" | wc -l
# Should be: 2 (only canonical versions)

# Update duplicate report
./scripts/unification/04_find_duplicates.sh
```

---

## 📊 EXPECTED RESULTS

**Before**: 19 HealthCheckConfig definitions  
**After**: 2 canonical definitions (renamed for clarity)  
**Reduction**: -89%

**Files Modified**: ~17 files  
**Imports Updated**: ~30-40 import statements  
**Time**: 2-3 hours total

---

## ⚠️ RISKS & MITIGATION

### **Risk 1: Field Name Mismatches**
Some definitions use `interval_secs: u64`, others use `interval: Duration`.

**Mitigation**:
- Check field usage in each file before removing
- Add conversion methods if needed
- Update call sites to match canonical field names

### **Risk 2: Breaking External APIs**
Some definitions may be public API.

**Mitigation**:
- Use type aliases for backward compatibility
- Add deprecation warnings with migration deadline
- Document in CHANGELOG.md

### **Risk 3: Test Failures**
Tests may rely on specific field names or types.

**Mitigation**:
- Run tests after each consolidation
- Update test fixtures to use canonical types
- Fix broken tests immediately

---

## 🎯 SUCCESS CRITERIA

- [ ] Only 2 canonical HealthCheckConfig definitions remain (renamed)
- [ ] All imports updated to use canonical versions
- [ ] `cargo check --workspace` passes
- [ ] `cargo test --workspace` passes
- [ ] Documentation updated
- [ ] CHANGELOG.md entry added

---

## 📝 NOTES

**Question**: Should `songbird-observability/src/health/production_health.rs:60` have its own ObservabilityHealthCheckConfig?

**Answer**: Review the fields. If identical to resilience version, consolidate. If observability-specific fields exist, keep as domain-specific config.

---

**Status**: Ready to execute  
**Next**: Start with Phase 1 (rename canonical versions)  
**Estimated Time**: 2-3 hours for complete consolidation

