# HealthCheckConfig Analysis
**Date**: November 10, 2025  
**Total Instances**: 16-18 definitions  
**Variants**: 15 different implementations

---

## 📊 FINDINGS SUMMARY

### Classification Result: **MIXED - True Duplicates + Specialized Variants**

**Analysis reveals**:
- ✅ **8-10 TRUE duplicates** → Can be consolidated to 1 canonical
- ⚠️ **2-3 SPECIALIZED variants** → Should remain separate (complementary)
- 🔍 **3-5 HYBRID cases** → Need field analysis

---

## 🎯 VARIANTS IDENTIFIED

### **Group A: Basic Health Check** (TRUE DUPLICATES - MERGE)

**Canonical Target**: `crates/songbird-canonical/src/config/resilience.rs`

#### Variant 2 (CANONICAL - USE THIS):
```rust
// crates/songbird-config/src/canonical/resilience.rs:406
pub struct HealthCheckConfig {
    pub enabled: bool,
    pub interval_secs: u64,
    pub timeout_secs: u64,
    pub path: String,
    pub failure_threshold: u32,
    pub recovery_threshold: u32,
}
```

**Why this one?**:
- Most comprehensive field set
- Already in canonical location
- Has both failure and recovery thresholds
- Clean, simple types (u64 for durations)

#### Duplicates to MIGRATE to Canonical (8-10 instances):

1. **`songbird-config/src/config/mod.rs:497`** (90% overlap)
   ```rust
   pub struct HealthCheckConfig {
       pub enabled: bool,
       pub endpoint: String,        // Same as 'path'
       pub interval_seconds: u64,   // Same as 'interval_secs'
       pub timeout_seconds: u64,    // Same as 'timeout_secs'
       pub retries: u32,            // Can add to canonical
   }
   ```
   **Action**: Replace with canonical, add retries field if needed

2. **`songbird-primal-sdk/src/config.rs:162`** (85% overlap)
   ```rust
   pub struct HealthCheckConfig {
       pub endpoint: String,
       pub interval_seconds: u64,
       pub timeout_seconds: u64,
       pub retry_count: u32,
       pub enabled: bool,
   }
   ```
   **Action**: Replace with canonical

3. **`songbird-primal-sdk/src/modern_api.rs:388`** (80% overlap)
   ```rust
   pub struct HealthCheckConfig {
       pub endpoint: String,
       pub interval_secs: u64,
       pub timeout_secs: u64,
   }
   ```
   **Action**: Replace with canonical

4. **`songbird-primal-sdk/src/universal_registry/config.rs:28`** (80% overlap)
   ```rust
   pub struct HealthCheckConfig {
       pub path: String,
       pub interval_seconds: u64,
       pub timeout_seconds: u64,
       pub failure_threshold: u32,
       pub success_threshold: u32,  // Same as recovery_threshold
   }
   ```
   **Action**: Replace with canonical

5. **`songbird-canonical/src/config/service.rs:36`** (70% overlap - SIMPLER)
   ```rust
   pub struct HealthCheckConfig {
       pub endpoint: String,
       pub interval: u64,
       pub timeout: u64,
   }
   ```
   **Action**: Replace with canonical or deprecate

6. **`songbird-types/src/config/discovery.rs:247`** (85% overlap)
   ```rust
   pub struct HealthCheckConfig {
       pub enabled: bool,
       pub endpoint: String,
       pub interval_seconds: u64,
       pub timeout_seconds: u64,
   }
   ```
   **Action**: Replace with canonical

7. **`songbird-types/src/config/performance.rs:573`** (85% overlap)
   ```rust
   pub struct HealthCheckConfig {
       pub enabled: bool,
       pub interval: Duration,  // Different type but same concept
       // ... (similar fields)
   }
   ```
   **Action**: Replace with canonical

8. **`songbird-discovery/src/traits/health.rs:26`** (85% overlap)
   ```rust
   pub struct HealthCheckConfig {
       pub interval: Duration,
       pub timeout: Duration,
       pub retries: u32,
       pub endpoint: Option<String>,
       pub enabled: bool,
   }
   ```
   **Action**: Replace with canonical

9. **`songbird-universal/src/types/config.rs:117`** (80% overlap)
   ```rust
   pub struct HealthCheckConfig {
       pub interval: Duration,
       pub timeout: Duration,
       // ... (similar fields)
   }
   ```
   **Action**: Replace with canonical

10. **`songbird-config/src/unified/robustness.rs:265`** (90% overlap)
    ```rust
    pub struct HealthCheckConfig {
        pub enabled: bool,
        pub interval: Duration,
        pub timeout: Duration,
        pub path: String,
        pub failure_threshold: u32,
        pub recovery_threshold: u32,
    }
    ```
    **Action**: Almost identical! Definitely replace with canonical

---

### **Group B: Specialized Production Health** (COMPLEMENTARY - KEEP)

#### Variant 7 (KEEP SEPARATE):
```rust
// crates/songbird-observability/src/health/production_health.rs:60
pub struct HealthCheckConfig {
    pub check_interval: Duration,
    pub request_timeout: Duration,
    pub degraded_threshold: u32,
    pub unhealthy_threshold: u32,
    pub max_concurrent_checks: usize,
    pub health_endpoints: Vec<String>,  // MULTIPLE endpoints!
}
```

**Why keep separate?**:
- Production monitoring specific
- Multiple endpoints (not single endpoint)
- Degraded vs unhealthy states
- Concurrent check management
- Different domain (observability vs config)

**Rename to**: `ProductionHealthCheckConfig` or `HealthMonitoringConfig`

#### Variant 8 (KEEP SEPARATE):
```rust
// crates/songbird-orchestrator/src/core/robustness/config.rs:189
pub struct HealthCheckConfig {
    pub check_interval: Duration,
    pub check_timeout: Duration,
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub enable_deep_checks: bool,
    pub deep_check_config: DeepHealthCheckConfig,  // NESTED config!
}
```

**Why keep separate?**:
- Deep health checks feature (nested config)
- Orchestrator-specific functionality
- More complex health checking logic

**Rename to**: `OrchestrationHealthCheckConfig` or keep with clear module path

---

### **Group C: Trait/Interface Definitions** (KEEP)

#### Registry Health Check:
```rust
// crates/songbird-registry/src/types/health.rs:119
pub struct HealthCheckConfig {
    pub check_type: HealthCheckType,  // ENUM for different check types
    pub interval: Duration,
    pub timeout: Duration,
    // ...
}
```

**Why keep separate?**:
- Has `check_type` enum (HTTP, TCP, custom, etc.)
- Registry-specific health semantics
- Part of registry types, not general config

**Action**: Keep but ensure it uses canonical for basic fields

---

## ✅ CONSOLIDATION STRATEGY

### Phase 1: Enhance Canonical (if needed)
```rust
// crates/songbird-canonical/src/config/resilience.rs
// ENHANCE to include best fields from all variants

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HealthCheckConfig {
    /// Enable health checks
    pub enabled: bool,
    
    /// Health check interval in seconds
    pub interval_secs: u64,
    
    /// Health check timeout in seconds
    pub timeout_secs: u64,
    
    /// Health check endpoint path
    pub path: String,
    
    /// Number of consecutive failures before marking unhealthy
    pub failure_threshold: u32,
    
    /// Number of consecutive successes before marking healthy
    pub recovery_threshold: u32,
    
    /// Optional: Maximum retry attempts (0 = no retries)
    #[serde(default)]
    pub max_retries: u32,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_secs: 30,
            timeout_secs: 5,
            path: "/health".to_string(),
            failure_threshold: 3,
            recovery_threshold: 2,
            max_retries: 0,
        }
    }
}
```

### Phase 2: Migration Plan (10 files)

**Priority 1** (Identical configs - safe merge):
1. `songbird-config/src/unified/robustness.rs:265` → Use canonical
2. `songbird-config/src/config/mod.rs:497` → Use canonical  
3. `songbird-primal-sdk/src/config.rs:162` → Use canonical

**Priority 2** (Minor field differences):
4. `songbird-primal-sdk/src/modern_api.rs:388` → Use canonical
5. `songbird-primal-sdk/src/universal_registry/config.rs:28` → Use canonical
6. `songbird-types/src/config/discovery.rs:247` → Use canonical
7. `songbird-types/src/config/performance.rs:573` → Use canonical

**Priority 3** (Need careful migration):
8. `songbird-discovery/src/traits/health.rs:26` → Use canonical (trait context)
9. `songbird-universal/src/types/config.rs:117` → Use canonical
10. `songbird-canonical/src/config/service.rs:36` → Deprecate or remove

### Phase 3: Rename Specialized Variants

**Rename to avoid confusion**:
1. `observability/.../production_health.rs` → `ProductionHealthMonitoringConfig`
2. `orchestrator/.../robustness/config.rs` → Keep as `HealthCheckConfig` (module-scoped)
   - Or rename to `OrchestrationHealthCheckConfig`

### Phase 4: Update All Imports

**Pattern**:
```rust
// ❌ OLD:
use crate::config::HealthCheckConfig;

// ✅ NEW:
use songbird_canonical::config::HealthCheckConfig;
```

---

## 📋 ESTIMATED EFFORT

### Analysis: ✅ COMPLETE (1 hour)
### Consolidation: 3-4 hours
- Enhance canonical if needed: 30 min
- Migrate 10 files: 2-3 hours (15-20 min each)
- Update imports: 30 min
- Testing: 30-60 min

### Expected Outcome:
- 16-18 → 3-4 definitions (12-15 eliminated)
- Reduction: 75-80%
- Grade impact: +0.3-0.5 points

---

## 🚨 SAFETY NOTES

### Breaking Changes:
- Public API changes in `songbird-primal-sdk`
- Type changes in `songbird-types`

### Mitigation:
- Add `#[deprecated]` for 1-2 releases
- Re-export from old locations
- Update all internal uses first

### Testing Required:
- `cargo test --package songbird-config`
- `cargo test --package songbird-canonical`
- `cargo test --package songbird-discovery`
- `cargo test --package songbird-primal-sdk`
- `cargo test --workspace` (full suite)

---

## ✅ DECISION: PROCEED WITH CONSOLIDATION

**Rationale**:
- Clear majority (10/16) are true duplicates
- Canonical location already exists
- Breaking changes manageable
- High impact (75% reduction)
- Proven pattern from Week 1

**Next Step**: Implement Phase 1 (enhance canonical if needed)

---

*Analysis complete: November 10, 2025*  
*Ready for consolidation: ✅ YES*

