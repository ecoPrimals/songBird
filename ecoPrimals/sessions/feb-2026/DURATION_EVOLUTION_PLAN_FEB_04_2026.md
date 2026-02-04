# Duration Constant Evolution Plan
**Date**: February 4, 2026  
**Status**: Analysis Complete, Awaiting Execution  
**Priority**: HIGH (658 instances identified)  
**Estimated Effort**: 6-8 hours (phased approach)

---

## 🎯 Objective

Evolve hardcoded `Duration` constants to environment-aware configuration, following the **"No Hardcoding"** deep debt principle while maintaining backward compatibility.

---

## 📊 Current State

**Total Duration Instances**: ~658 (production + tests)

**Categories Identified**:

### 1. **Configuration Values** (HIGH PRIORITY) - ~150 instances
Values that should be environment-configurable for deployment flexibility.

**Examples**:
```rust
// Health check intervals
health_check_interval: Duration::from_secs(30)
discovery_timeout: Duration::from_secs(10)

// Connection timeouts
acquire_timeout: Duration::from_secs(5)
max_idle_time: Duration::from_secs(60)

// Circuit breaker settings
timeout: Duration::from_secs(60)
recovery_timeout: Duration::from_secs(30)

// Cleanup intervals
cleanup_interval: Duration::from_secs(30)
session_ttl_cleanup: Duration::from_secs(300)
```

### 2. **Test Values** (LOW PRIORITY, KEEP HARDCODED) - ~400 instances
Test sleep durations, assertion timeouts, test fixtures.

**Examples**:
```rust
// Test coordination sleeps
tokio::time::sleep(Duration::from_millis(100)).await;

// Test assertions
assert!(duration < Duration::from_secs(5));

// Test fixtures
let client = StunClient::with_timeout(Duration::from_secs(10));
```

**Status**: ✅ **KEEP AS IS** - Test hardcoding is appropriate

### 3. **Protocol Timing** (MEDIUM PRIORITY) - ~50 instances
Short coordination sleeps, protocol timeouts.

**Examples**:
```rust
// Async coordination
tokio::time::sleep(Duration::from_millis(10)).await;

// Protocol timeouts
read_timeout: Duration::from_millis(100)

// Retry backoff
Duration::from_millis(50)
```

**Assessment**: Some should be configurable, others OK as constants

### 4. **Already Evolved** (✅ DONE) - ~58 instances
Some Duration values already support environment configuration.

**Examples**:
```rust
// StunClient timeout (configurable)
StunClient::with_timeout(Duration::from_secs(timeout_secs))

// JsonRpcClient timeout (configurable)
client.with_timeout(Duration::from_secs(30))

// ConnectionPool (has default values, can be overridden)
ConnectionPoolConfig {
    max_idle_time: Duration::from_secs(60),
    // ...
}
```

---

## 🎓 Evolution Philosophy

### Principles

1. **Environment-Aware Defaults**
   - Default values from environment variables
   - Fallback to sensible constants
   - Runtime configurable

2. **Backward Compatible**
   - Existing behavior preserved
   - Same default values
   - Zero breaking changes

3. **Smart Categorization**
   - Configuration → Environment-aware
   - Tests → Keep hardcoded (appropriate)
   - Protocol → Case-by-case analysis

4. **Centralized Management**
   - New `TimeoutConfig` system (extend existing)
   - Similar to `env_config.rs` for ports
   - Single source of truth

---

## 🏗️ Proposed Architecture

### New Module: `crates/songbird-config/src/timeouts.rs`

```rust
//! Timeout and Duration Configuration
//!
//! Environment-aware timeout defaults following zero-hardcoding philosophy.

use std::time::Duration;
use songbird_types::SafeEnv;

/// Health check and monitoring timeouts
pub mod health {
    use super::*;
    
    /// Health check interval
    /// ENV: SONGBIRD_HEALTH_CHECK_INTERVAL_SECS (default: 30)
    pub fn check_interval() -> Duration {
        Duration::from_secs(
            SafeEnv::parse("SONGBIRD_HEALTH_CHECK_INTERVAL_SECS", 30)
        )
    }
    
    /// Health check timeout
    /// ENV: SONGBIRD_HEALTH_CHECK_TIMEOUT_SECS (default: 5)
    pub fn check_timeout() -> Duration {
        Duration::from_secs(
            SafeEnv::parse("SONGBIRD_HEALTH_CHECK_TIMEOUT_SECS", 5)
        )
    }
}

/// Connection pool timeouts
pub mod pool {
    use super::*;
    
    /// Maximum idle time for pooled connections
    /// ENV: SONGBIRD_POOL_MAX_IDLE_SECS (default: 60)
    pub fn max_idle() -> Duration {
        Duration::from_secs(
            SafeEnv::parse("SONGBIRD_POOL_MAX_IDLE_SECS", 60)
        )
    }
    
    /// Connection acquisition timeout
    /// ENV: SONGBIRD_POOL_ACQUIRE_TIMEOUT_SECS (default: 5)
    pub fn acquire_timeout() -> Duration {
        Duration::from_secs(
            SafeEnv::parse("SONGBIRD_POOL_ACQUIRE_TIMEOUT_SECS", 5)
        )
    }
    
    /// Pool cleanup interval
    /// ENV: SONGBIRD_POOL_CLEANUP_INTERVAL_SECS (default: 30)
    pub fn cleanup_interval() -> Duration {
        Duration::from_secs(
            SafeEnv::parse("SONGBIRD_POOL_CLEANUP_INTERVAL_SECS", 30)
        )
    }
}

/// Circuit breaker timeouts
pub mod circuit_breaker {
    use super::*;
    
    /// Circuit breaker timeout
    /// ENV: SONGBIRD_CB_TIMEOUT_SECS (default: 60)
    pub fn timeout() -> Duration {
        Duration::from_secs(
            SafeEnv::parse("SONGBIRD_CB_TIMEOUT_SECS", 60)
        )
    }
    
    /// Circuit breaker recovery timeout
    /// ENV: SONGBIRD_CB_RECOVERY_SECS (default: 30)
    pub fn recovery() -> Duration {
        Duration::from_secs(
            SafeEnv::parse("SONGBIRD_CB_RECOVERY_SECS", 30)
        )
    }
}

/// Discovery timeouts
pub mod discovery {
    use super::*;
    
    /// Discovery timeout
    /// ENV: SONGBIRD_DISCOVERY_TIMEOUT_SECS (default: 10)
    pub fn timeout() -> Duration {
        Duration::from_secs(
            SafeEnv::parse("SONGBIRD_DISCOVERY_TIMEOUT_SECS", 10)
        )
    }
}

/// Cleanup and maintenance intervals
pub mod cleanup {
    use super::*;
    
    /// Session TTL cleanup interval
    /// ENV: SONGBIRD_SESSION_CLEANUP_SECS (default: 300)
    pub fn session_ttl() -> Duration {
        Duration::from_secs(
            SafeEnv::parse("SONGBIRD_SESSION_CLEANUP_SECS", 300)
        )
    }
    
    /// Service registry cleanup interval
    /// ENV: SONGBIRD_REGISTRY_CLEANUP_SECS (default: 60)
    pub fn registry() -> Duration {
        Duration::from_secs(
            SafeEnv::parse("SONGBIRD_REGISTRY_CLEANUP_SECS", 60)
        )
    }
    
    /// Trust relationship cleanup interval
    /// ENV: SONGBIRD_TRUST_CLEANUP_SECS (default: 300)
    pub fn trust() -> Duration {
        Duration::from_secs(
            SafeEnv::parse("SONGBIRD_TRUST_CLEANUP_SECS", 300)
        )
    }
}
```

### Usage Example

**Before** (hardcoded):
```rust
let checker = HealthChecker::new(Duration::from_secs(30));
```

**After** (environment-aware):
```rust
use songbird_config::timeouts;

let checker = HealthChecker::new(timeouts::health::check_interval());
```

---

## 📋 Phased Evolution Strategy

### Phase 1: Foundation (1-2 hours)
**Goal**: Create timeout configuration module

**Tasks**:
1. Create `crates/songbird-config/src/timeouts.rs`
2. Implement timeout categories (health, pool, circuit_breaker, discovery, cleanup)
3. Add environment variable parsing with SafeEnv
4. Write comprehensive tests
5. Document environment variables

**Deliverables**:
- timeouts.rs module (~300 lines)
- 20+ unit tests
- Environment variable documentation

---

### Phase 2: High-Priority Migration (2-3 hours)
**Goal**: Migrate configuration-related Durations

**Target Files** (~30 files, ~80 instances):
1. Connection pool configuration
2. Health check intervals
3. Circuit breaker timeouts
4. Discovery timeouts
5. Cleanup intervals

**Pattern**:
```rust
// Old
Duration::from_secs(30)

// New
timeouts::health::check_interval()
```

**Validation**:
- All tests still pass
- Same default behavior
- Environment variables work

---

### Phase 3: Medium-Priority Migration (2-3 hours)
**Goal**: Migrate protocol timing where beneficial

**Target Files** (~20 files, ~40 instances):
1. RPC timeouts
2. Retry backoff intervals
3. Connection attempt timeouts

**Assessment Criteria**:
- Is it user-facing? → Migrate
- Is it internal coordination? → Consider keeping
- Would configuration help debugging? → Migrate

---

### Phase 4: Documentation & Testing (1 hour)
**Goal**: Comprehensive documentation and validation

**Tasks**:
1. Update deployment documentation
2. Create environment variable reference
3. Add integration tests for timeout configuration
4. Performance validation (ensure no regression)
5. Update session completion summary

---

## 🎯 Priority Targets

### Immediate (Phase 2)

**High-Value Targets** (first 10 to migrate):

1. **Health Check Interval** (`songbird-universal/src/lib.rs`)
   ```rust
   // Line 119
   health_check_interval: Duration::from_secs(30)
   ↓
   health_check_interval: timeouts::health::check_interval()
   ```

2. **Discovery Timeout** (`songbird-universal/src/lib.rs`)
   ```rust
   // Line 118
   discovery_timeout: Duration::from_secs(10)
   ↓
   discovery_timeout: timeouts::discovery::timeout()
   ```

3. **Connection Pool Max Idle** (`songbird-http-client/src/connection_pool.rs`)
   ```rust
   // Line 109
   max_idle_time: Duration::from_secs(60)
   ↓
   max_idle_time: timeouts::pool::max_idle()
   ```

4. **Pool Acquire Timeout** (`songbird-http-client/src/connection_pool.rs`)
   ```rust
   // Line 110
   acquire_timeout: Duration::from_secs(5)
   ↓
   acquire_timeout: timeouts::pool::acquire_timeout()
   ```

5. **Pool Cleanup Interval** (`songbird-http-client/src/connection_pool.rs`)
   ```rust
   // Line 111
   cleanup_interval: Duration::from_secs(30)
   ↓
   cleanup_interval: timeouts::pool::cleanup_interval()
   ```

6. **Circuit Breaker Timeout** (`songbird-orchestrator/src/resilience/circuit_breaker.rs`)
   ```rust
   // Line 137
   timeout: Duration::from_secs(60)
   ↓
   timeout: timeouts::circuit_breaker::timeout()
   ```

7. **Circuit Breaker Recovery** (`songbird-orchestrator/src/core/substrate/clients.rs`)
   ```rust
   // Line 34
   CircuitBreaker::new(5, Duration::from_secs(30))
   ↓
   CircuitBreaker::new(5, timeouts::circuit_breaker::recovery())
   ```

8. **Trust Cleanup Interval** (`songbird-orchestrator/src/app/core.rs`)
   ```rust
   // Line 489
   Duration::from_secs(300)
   ↓
   timeouts::cleanup::trust()
   ```

9. **Session TTL Cleanup** (find location)
   ```rust
   Duration::from_secs(300)
   ↓
   timeouts::cleanup::session_ttl()
   ```

10. **Registry Cleanup** (find location)
    ```rust
    Duration::from_secs(60)
    ↓
    timeouts::cleanup::registry()
    ```

---

## 🧪 Testing Strategy

### Unit Tests
- Each timeout function returns correct default
- Environment variables override defaults
- Invalid values fall back to defaults

### Integration Tests
- End-to-end with environment variables set
- Verify actual behavior changes
- Performance baseline (no regression)

### Backward Compatibility
- All existing tests pass unchanged
- Same default behavior
- Zero breaking changes

---

## 📊 Success Criteria

### Quantitative
- ✅ 80+ configuration Durations migrated
- ✅ All tests passing (196/201 maintained)
- ✅ Zero breaking changes
- ✅ Environment variables documented

### Qualitative
- ✅ Deployment flexibility improved
- ✅ No hardcoding in configuration
- ✅ Clear categorization (config vs test)
- ✅ Modern idiomatic Rust patterns

---

## 🔮 Out of Scope (Deliberately)

### Test Durations (KEEP AS IS)
- Test sleep coordination: `Duration::from_millis(100)`
- Test assertion timeouts: `Duration::from_secs(5)`
- Test fixtures: OK to hardcode

**Rationale**: Test hardcoding is appropriate and aids clarity

### Protocol Constants (KEEP AS IS)
- Very short sleeps: `Duration::from_millis(10)` for async coordination
- Protocol-defined timeouts (e.g., STUN, TLS)

**Rationale**: Protocol timing often shouldn't be configurable

---

## 💡 Alternative: Incremental Evolution

If full migration is too large, consider **incremental approach**:

1. **Week 1**: Create timeouts.rs module + health checks (20 instances)
2. **Week 2**: Connection pool configuration (15 instances)
3. **Week 3**: Circuit breakers + discovery (20 instances)
4. **Week 4**: Cleanup intervals + remaining (25 instances)

Each week delivers value independently, reducing risk.

---

## 🎓 Deep Debt Alignment

### Principles Applied

✅ **No Hardcoding**:
- Configuration from environment
- Sensible defaults
- Runtime flexibility

✅ **Modern Idiomatic Rust**:
- Centralized timeout module
- Type-safe configuration
- SafeEnv pattern

✅ **Backward Compatible**:
- Same defaults
- No breaking changes
- Graceful evolution

✅ **Smart Refactoring**:
- Keep test hardcoding (appropriate)
- Migrate configuration (beneficial)
- Case-by-case protocol timing

---

## 📝 Implementation Checklist

### Phase 1: Foundation
- [ ] Create `timeouts.rs` module
- [ ] Implement health timeout functions
- [ ] Implement pool timeout functions
- [ ] Implement circuit_breaker timeout functions
- [ ] Implement discovery timeout functions
- [ ] Implement cleanup timeout functions
- [ ] Write 20+ unit tests
- [ ] Document environment variables

### Phase 2: High-Priority Migration
- [ ] Migrate health check intervals (2 instances)
- [ ] Migrate connection pool timeouts (5 instances)
- [ ] Migrate circuit breaker timeouts (3 instances)
- [ ] Migrate discovery timeouts (2 instances)
- [ ] Migrate cleanup intervals (3 instances)
- [ ] Run full test suite
- [ ] Validate backward compatibility

### Phase 3: Medium-Priority Migration
- [ ] Assess protocol timing instances (~40)
- [ ] Migrate RPC timeouts
- [ ] Migrate retry intervals
- [ ] Migrate connection timeouts
- [ ] Run full test suite

### Phase 4: Documentation
- [ ] Update deployment docs
- [ ] Create environment variable reference
- [ ] Add integration tests
- [ ] Performance validation
- [ ] Create completion summary

---

## 🚀 Recommendation

**Approach**: **Phased Evolution** (4 phases over 6-8 hours)

**Rationale**:
- High value (80+ configuration instances)
- Manageable scope per phase
- Independent value delivery
- Low risk (backward compatible)

**Alternative**: **Incremental Weekly Evolution** (4 weeks, 20-25 instances/week)

**Rationale**:
- Lower weekly time commitment
- Each week delivers value
- Easier to validate
- More sustainable pace

---

## 📋 Next Steps

**Option A - Full Execution** (recommended if 6-8 hours available):
1. Execute Phase 1 (foundation)
2. Execute Phase 2 (high-priority)
3. Execute Phase 3 (medium-priority)
4. Execute Phase 4 (documentation)

**Option B - Incremental** (recommended if time-constrained):
1. Week 1: Phase 1 + health checks
2. Week 2: Connection pool
3. Week 3: Circuit breakers + discovery
4. Week 4: Cleanup + documentation

**Option C - Defer**:
- Document as future work
- Focus on other high-priority items
- Revisit when time permits

---

*Waiting for user directive to proceed with Option A, B, or C*

---

**Status**: ⏸️ **AWAITING USER DECISION**  
**Analysis**: ✅ COMPLETE  
**Implementation**: ⏳ READY TO START
