# ✅ TimeoutConfig Consolidation - November 10, 2025

**Status**: 🔄 **IN PROGRESS**  
**Instances Found**: 9 TimeoutConfig + NetworkTimeouts  
**Strategy**: Consolidate to single canonical + re-exports

---

## 📊 Analysis

### Canonical Versions (3)

1. **`config/canonical/network/timeouts.rs`** - Network-specific timeouts
2. **`types/adapters/canonical.rs`** - General canonical
3. **`types/config/consolidated_canonical/network.rs`** - Network canonical

**Issue**: Three canonicals exist - need to choose one and consolidate

**Decision**: Use `config/canonical/network/timeouts.rs::NetworkTimeouts` for network-specific, and create a general `TimeoutConfig` in `canonical/resilience.rs` for operation timeouts

---

### Instances to Consolidate (6)

1. `orchestrator/core/robustness/config.rs::TimeoutConfig`
2. `orchestrator/core/robustness/config.rs::AdaptiveTimeoutConfig` (specialized)
3. `primal-sdk/config.rs::TimeoutConfig`
4. `config/config/hardcoded_elimination.rs::TimeoutConfig`
5. `types/config/adapters.rs::CanonicalTimeoutConfig`
6. `test-utils/config/mod.rs::TestTimeoutConfig` (test-specific, keep)

---

## 🎯 Consolidation Strategy

### Step 1: Canonical Definitions

**Network Timeouts** (Keep in `canonical/network/timeouts.rs`):
```rust
pub struct NetworkTimeouts {
    pub connection: Duration,
    pub request: Duration,
    pub idle: Duration,
    pub keepalive: Duration,
}
```

**General Timeouts** (Create in `canonical/resilience.rs`):
```rust
pub struct TimeoutConfig {
    pub default_timeout: Duration,
    pub connection_timeout: Duration,
    pub request_timeout: Duration,
    pub idle_timeout: Duration,
}
```

### Step 2: Re-export in Other Crates

Replace local definitions with re-exports

---

## ⚠️ Specialized Cases

### AdaptiveTimeoutConfig (orchestrator)

**Assessment**: ML-based, orchestrator-specific  
**Decision**: Keep as separate type, rename to `AdaptiveTimeoutConfig`

### TestTimeoutConfig (test-utils)

**Assessment**: Test-specific  
**Decision**: Keep as is

---

## 🎯 Estimated Impact

**Time**: 1-2 hours  
**Lines Removed**: ~100-150 lines  
**Consolidations**: 6-7  
**Grade Impact**: +0.3-0.5 points  
**Build Risk**: Low-Medium (timeout fields vary more than retry)

---

*TimeoutConfig Consolidation - November 10, 2025*  
*Priority 2.4: 🔄 IN PROGRESS*  
*Instances: 9 (3 canonical + 6 to consolidate)*

