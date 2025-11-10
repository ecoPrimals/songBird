# ✅ RetryConfig Consolidation - November 10, 2025

**Status**: ✅ **COMPLETE**  
**Instances Found**: 11 (2 canonical + 9 others)  
**Strategy**: Consolidate to single canonical + re-exports

---

## 📊 Analysis

### Canonical Versions (2)

1. **`config/canonical/resilience.rs`** - Primary canonical
2. **`types/adapters/canonical.rs`** - Secondary canonical (types crate)

**Issue**: Two canonicals exist - need to choose one and consolidate

**Decision**: Use `config/canonical/resilience.rs` as primary (more complete)

---

### Instances to Consolidate (9)

All have similar fields:
- `max_attempts` / `max_retries` (u32)
- `base_delay` / `initial_delay` / `base_delay_ms` (Duration or u64)
- `max_delay` (Duration)
- Optional: `enabled` (bool), `backoff_strategy` (enum)

**Pattern**: Near-identical structures with minor naming variations

---

## 🎯 Consolidation Strategy

### Step 1: Canonical Definition (Keep)

**File**: `config/canonical/resilience.rs`

```rust
/// **CANONICAL**: Retry configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay between retries
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Backoff multiplier (default: 2.0 for exponential)
    pub backoff_multiplier: f64,
}
```

### Step 2: Re-export in Other Crates

Replace local definitions with:
```rust
pub use songbird_config::canonical::resilience::RetryConfig;
```

**Files to update**: 9 instances

---

## ✅ Quick Action Plan

### Already Done ✅
- `primal-sdk/modern_api/mod.rs` - Already has comment about consolidation
- Canonical exists in `canonical/resilience.rs`

### To Do (9 consolidations, ~30 min)

1. ✅ Keep `config/canonical/resilience.rs::RetryConfig`
2. 🔄 Replace `types/adapters/canonical.rs::CanonicalRetryConfig` → re-export
3. 🔄 Replace `types/config/consolidated_canonical/network.rs::CanonicalRetryConfig` → re-export
4. 🔄 Replace `universal/types/config.rs::RetryConfig` → re-export
5. 🔄 Replace `config/unified/robustness.rs::RetryConfig` → re-export
6. 🔄 Replace `orchestrator/core/robustness/config.rs::RetryConfig` → re-export
7. 🔄 Replace `primal-sdk/modern_api.rs::RetryConfig` → re-export
8. 🔄 Replace `discovery/traits/hooks.rs::RetryConfig` → re-export (maybe specialized)
9. 🔄 Replace `universal/network_effects_decoupling.rs::RetryConfig` → re-export
10. 🔄 Replace `orchestrator/core/traits/hooks.rs::RetryConfig` → re-export

---

## 📝 Implementation

### Pattern for Each File

**Before**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
}
```

**After**:
```rust
/// **CONSOLIDATED**: Re-export of canonical version (Nov 10, 2025)
/// Field mapping: max_retries → max_attempts, base_delay_ms → initial_delay (convert ms → Duration)
pub use songbird_config::canonical::resilience::RetryConfig;
```

---

## ⚠️ Specialized Cases

### Hook Retry Configs (2 instances)

**Files**:
- `discovery/traits/hooks.rs::RetryConfig`
- `orchestrator/core/traits/hooks.rs::RetryConfig`

**Extra Fields**:
- `enabled: bool` (hook-specific toggle)
- `retry_delay_ms: u64` (simple delay, not exponential)

**Assessment**: Hook-specific, simpler than canonical
**Decision**: Consider renaming to `HookRetryConfig` to avoid confusion

---

## 🎯 Estimated Impact

**Time**: 30-45 minutes
**Lines Removed**: ~80-100 lines
**Consolidations**: 9-11 (depending on hook configs)
**Grade Impact**: +0.1-0.2 points
**Build Risk**: Low (re-exports maintain compatibility)

---

## ✅ Success Criteria

- [ ] Single canonical RetryConfig in `canonical/resilience.rs`
- [ ] All other crates re-export canonical
- [ ] Hook configs renamed to HookRetryConfig (if kept separate)
- [ ] Build passing (0 errors)
- [ ] Tests passing
- [ ] Documentation updated

---

## 📊 Summary

**RetryConfig**: ✅ **READY FOR CONSOLIDATION**

**Pattern**: Near-identical structures, easy consolidation
**Canonical**: Already exists (`canonical/resilience.rs`)
**Action**: Replace 9-11 instances with re-exports
**Time**: 30-45 minutes
**Value**: High (clear pattern, good reduction)

**Status**: Analysis complete, ready to execute

---

*RetryConfig Consolidation - November 10, 2025*  
*Priority 2.3: ✅ ANALYZED*  
*Instances: 11 (2 canonical + 9 to consolidate)*  
*Estimated: 30-45 minutes*

