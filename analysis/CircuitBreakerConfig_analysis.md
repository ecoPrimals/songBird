# CircuitBreakerConfig Analysis
**Date**: November 10, 2025  
**Total Instances**: 13 definitions  
**Variants**: 13 different implementations

---

## 📊 FINDINGS SUMMARY

### Classification Result: **MIXED - Some True Duplicates + Specialized Variants**

**Analysis reveals**:
- ✅ **8-9 TRUE duplicates** → Can be consolidated to 1 canonical
- ⚠️ **2-3 SPECIALIZED variants** → Should remain separate (service-specific)
- 🔍 **1-2 HYBRID cases** → Need careful review

**Consolidation Potential**: ~60-70% (8-9 out of 13)

---

## 🎯 CANONICAL VERSION

### **Selected Canonical**: `songbird-config/src/canonical/resilience.rs`

```rust
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures before opening the circuit
    pub failure_threshold: u32,
    /// Time to wait before attempting to close the circuit
    pub timeout: Duration,
    /// Number of successful requests needed to close the circuit
    pub success_threshold: u32,
    /// Maximum number of requests allowed in half-open state
    pub half_open_max_requests: u32,
    /// Whether the circuit breaker is enabled
    pub enabled: bool,
}
```

**Why this one?**:
- Most comprehensive field set
- Already in canonical location
- Has half-open state support
- Includes enable flag
- Well-documented

---

## 📋 VARIANTS ANALYSIS

### **Group A: TRUE DUPLICATES** (8-9 instances - CONSOLIDATE)

#### 1. **canonical/network.rs** (95% match) ✅ CONSOLIDATE
```rust
pub struct CircuitBreakerConfig {
    pub enabled: bool,
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub timeout_secs: u64,  // vs Duration
}
```
**Difference**: Uses `u64` seconds instead of `Duration`, missing `half_open_max_requests`  
**Action**: Replace with canonical

#### 2. **canonical/adapters.rs** (95% match) ✅ CONSOLIDATE
```rust
pub struct CircuitBreakerConfig {
    pub enabled: bool,
    pub failure_threshold: u32,
    pub timeout_seconds: u64,
    pub success_threshold: u32,
}
```
**Difference**: Uses `u64` seconds, missing `half_open_max_requests`  
**Action**: Replace with canonical

#### 3. **unified/api.rs** (75% match) ✅ CONSOLIDATE
```rust
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub timeout: Duration,
    pub recovery_timeout: Duration,  // Extra field
    pub enabled: bool,
}
```
**Difference**: Has `recovery_timeout`, missing `success_threshold` and `half_open`  
**Action**: Replace with canonical (recovery_timeout ≈ timeout in half-open)

#### 4. **primal-sdk/modern_api.rs** (70% match) ✅ CONSOLIDATE
```rust
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub timeout_duration_secs: u64,
    pub retry_delay_secs: u64,
}
```
**Difference**: Extra `retry_delay_secs`, uses u64  
**Action**: Replace with canonical

#### 5. **primal-sdk/modern_api/mod.rs** (40% match) ⚠️ INCOMPLETE
```rust
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub recovery_pub half_open_max_calls: u32,  // SYNTAX ERROR!
}
```
**Difference**: Corrupted definition!  
**Action**: Definitely replace with canonical

#### 6. **primal-sdk/universal_registry/config.rs** (60% match) ✅ CONSOLIDATE
```rust
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub timeout_duration: Duration,
    pub half_open_max_calls: u32,
}
```
**Difference**: Missing `success_threshold` and `enabled`  
**Action**: Replace with canonical

#### 7. **universal/circuit_breaker.rs** (80% match) ✅ CONSOLIDATE
```rust
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub timeout: Duration,
    pub success_threshold: u32,
}
```
**Difference**: Missing `half_open_max_requests` and `enabled`  
**Action**: Replace with canonical

#### 8. **universal/types/config.rs** (70% match) ✅ CONSOLIDATE
```rust
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub failure_window: Duration,  // Different concept
    pub recovery_timeout: Duration,
    pub success_threshold: u32,
}
```
**Difference**: Has `failure_window` (different from timeout), missing `enabled`  
**Action**: Replace with canonical (or consider adding failure_window to canonical)

---

### **Group B: SPECIALIZED VARIANTS** (2-3 instances - KEEP SEPARATE)

#### 9. **unified/robustness.rs** (COMPLEX) 🟡 REVIEW
```rust
pub struct CircuitBreakerConfig {
    pub enabled: bool,
    pub failure_threshold: u32,
    pub timeout: Duration,
    pub recovery_timeout: Duration,
    pub half_open_max_calls: u32,
    pub success_threshold: u32,
    pub min_throughput_threshold: u32,  // EXTRA
}
```
**Why special?**: Has `min_throughput_threshold` for advanced monitoring  
**Action**: Consider consolidating OR keep if throughput monitoring is critical

#### 10. **orchestrator/.../universal_service_registration/types.rs** (SPECIALIZED) 🔴 KEEP
```rust
pub struct CircuitBreakerConfig {
    pub failure_threshold_percentage: f64,  // PERCENTAGE not count!
    pub minimum_request_threshold: u32,
    pub request_volume_threshold: u32,
    pub sleep_window_seconds: u64,
}
```
**Why special?**: Uses **percentage-based** failure detection (different algorithm!)  
**Action**: KEEP SEPARATE - Different implementation strategy

#### 11. **orchestrator/core/robustness/config.rs** (SERVICE-SPECIFIC) 🟡 REVIEW
```rust
pub struct CircuitBreakerConfig {
    pub service_name: String,  // SERVICE-SPECIFIC
    pub failure_threshold: u32,
    pub timeout: Duration,
    // Has corrupted Default implementation
}
```
**Why special?**: Includes `service_name` (service-specific instance)  
**Action**: Could consolidate base config, keep service_name wrapper

#### 12. **types/config/communication.rs** (TYPES CRATE) 📝 KEEP WITH NOTES
```rust
pub struct CircuitBreakerConfig {
    pub failure_threshold: usize,  // usize not u32!
    pub success_threshold: usize,
    pub timeout: Duration,
    pub half_open_timeout: Duration,
    pub enabled: bool,
    pub reset_timeout_multiplier: f64,  // EXTRA
}
```
**Why special?**: Types crate (no config dependency), uses `usize`, has multiplier  
**Action**: Keep with documentation notes (like HealthCheckConfig)

---

## ✅ CONSOLIDATION STRATEGY

### Phase 1: Enhance Canonical (if needed)
Current canonical is good! Minimal enhancement needed.

Possible additions:
- `failure_window: Option<Duration>` (from universal/types)
- `recovery_timeout: Option<Duration>` (from unified variants)

### Phase 2: Consolidate TRUE Duplicates (8-9 instances)

**Priority 1** (Identical patterns):
1. canonical/network.rs → Use canonical/resilience
2. canonical/adapters.rs → Use canonical/resilience
3. universal/circuit_breaker.rs → Use canonical/resilience

**Priority 2** (Minor differences):
4. unified/api.rs → Use canonical/resilience
5. primal-sdk/modern_api.rs → Use canonical/resilience
6. primal-sdk/modern_api/mod.rs → Use canonical/resilience (FIX CORRUPTION!)
7. primal-sdk/universal_registry/config.rs → Use canonical/resilience
8. universal/types/config.rs → Use canonical/resilience

**Priority 3** (Review needed):
9. unified/robustness.rs → Consider consolidating (has min_throughput_threshold)

### Phase 3: Keep Specialized Variants

**Keep as-is** (with documentation):
- orchestrator/.../universal_service_registration/types.rs (percentage-based)
- types/config/communication.rs (types crate, uses usize)

**Review** (may consolidate):
- orchestrator/core/robustness/config.rs (service-specific, needs cleanup)

---

## 📊 ESTIMATED EFFORT

### Analysis: ✅ COMPLETE (30 minutes)
### Consolidation: 2-3 hours
- Phase 1: Enhance canonical (15-30 min)
- Phase 2: Consolidate 8-9 files (1.5-2 hours, ~10-15 min each)
- Phase 3: Document specialized variants (30-45 min)

### Expected Outcome:
- 13 → 4-5 definitions (60-70% reduction)
- Reduction: 8-9 instances consolidated
- Grade impact: +0.5-0.7 points

---

## 🚨 CRITICAL FINDINGS

### **CORRUPTION DETECTED**: primal-sdk/modern_api/mod.rs
```rust
pub recovery_pub half_open_max_calls: u32,  // SYNTAX ERROR!
```
**Impact**: This won't compile!  
**Action**: Fix immediately during consolidation

### **SPECIALIZED ALGORITHM**: orchestrator universal_service_registration
Uses **percentage-based** failure detection instead of count-based.  
**Action**: Keep separate, this is intentionally different

---

## ✅ DECISION: PROCEED WITH CONSOLIDATION

**Rationale**:
- Clear majority (8-9/13) are true duplicates
- Canonical location exists and is comprehensive
- Corrupted definition needs fixing anyway
- High impact (60-70% reduction)
- Proven pattern from HealthCheckConfig

**Next Step**: Enhance canonical if needed, then consolidate 8-9 instances

---

*Analysis complete: November 10, 2025*  
*Ready for consolidation: ✅ YES*  
*Estimated time: 2-3 hours*

