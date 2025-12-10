# 🎯 AUDIT ACTION PLAN - Priority-Ordered Tasks
**Date**: December 2, 2025  
**Based On**: Comprehensive Audit Report

---

## 🔴 **P0 - CRITICAL (Before Production)**

### **1. Production Unwrap Elimination** (1-2 weeks)
**Risk**: Production panics from unexpected errors  
**Count**: 1,385 instances across 152 files

```bash
# Audit command
rg "\.unwrap\(\)|\.expect\(" --type rust crates/*/src/ -g "!tests" -g "!benches"

# Track progress
git grep -c "unwrap()" crates/*/src/**/*.rs | grep -v test
```

**Action Steps**:
- [ ] Week 1: Audit all `src/` (non-test) unwraps
- [ ] Replace with proper `Result<T, E>` handling
- [ ] Add `#![deny(clippy::unwrap_used)]` to production crates
- [ ] Document justified unwraps with `#[allow(clippy::unwrap_used)]` and comments

**High Priority Files**:
```
❌ crates/songbird-config/src/capability_endpoints.rs: 33 instances
❌ crates/songbird-test-utils/src/coordination.rs: 5 instances  
❌ crates/songbird-cli/src/cli_comprehensive_tests.rs: 16 instances
```

---

### **2. Panic Statement Fixes** (1 week)
**Risk**: Production crashes from unexpected states  
**Count**: 152 panic!/unreachable!/unimplemented! across 47 files

```bash
# Find panics in production code
rg "panic!|unreachable!|unimplemented!" --type rust crates/*/src/ -g "!tests" -g "!benches"
```

**Action Steps**:
- [ ] Replace `unreachable!()` with error returns
- [ ] Implement all `unimplemented!()` stubs  
- [ ] Convert `panic!()` to proper error handling
- [ ] Add panic hooks for production debugging

---

### **3. Security Adapter Testing** (1 week)
**Risk**: Critical security component undertested  
**Current**: 14.16% coverage  
**Target**: 90% coverage

**Location**: `crates/songbird-universal/src/adapters/security.rs`

**Action Steps**:
- [ ] Day 1-2: Write authentication tests
- [ ] Day 3-4: Authorization and RBAC tests
- [ ] Day 5: Encryption/decryption tests
- [ ] Day 6-7: Integration and error path tests

**Test Scenarios Needed**:
```rust
// Authentication
- Valid credentials
- Invalid credentials
- Expired tokens
- Malformed tokens

// Authorization
- Permission checks
- Role validation
- Access control

// Encryption
- Data encryption/decryption
- Key management
- Algorithm selection
```

---

## 🟡 **P1 - HIGH PRIORITY (Next 4 Weeks)**

### **4. Zero-Hardcoding Migration** (2-3 weeks)
**Issue**: Violates stated universal adapter pattern  
**Count**: 1,445 IPs, 692 ports, 1,040 primal names

#### **Phase 1: Hardcoded IPs** (Week 1)
```bash
# Find hardcoded IPs
rg "127\.0\.0\.1|localhost|192\.168\.|10\.0\.|172\.16\.|0\.0\.0\.0" crates/*/src/ -g "!tests"
```

**Action Steps**:
- [ ] Move all IPs to environment variables
- [ ] Update config system with `SafeEnv`
- [ ] Create `.env.example` with all IP configs
- [ ] Document IP configuration in deployment guide

#### **Phase 2: Hardcoded Ports** (Week 2)
**Locations**:
- `crates/songbird-config/src/defaults/ports.rs`: 24 instances
- `crates/songbird-config/src/canonical/constants.rs`: 3 instances

**Action Steps**:
- [ ] Implement dynamic port discovery
- [ ] Service discovery should return port info
- [ ] Remove `DEFAULT_*_PORT` constants
- [ ] Port configuration via environment only

#### **Phase 3: Primal Name Removal** (Week 3)
**Critical**: This violates your universal adapter pattern!

**Worst Offenders**:
```
❌ crates/songbird-primal-sdk/src/* - Heavy primal-specific code
❌ crates/songbird-test-utils/src/mocks/* - Primal-specific mocks  
❌ crates/songbird-config/src/zero_hardcoding_migration.rs: 42 refs (ironic!)
```

**Action Steps**:
- [ ] Remove all primal-specific adapters
- [ ] Use capability-based discovery exclusively
- [ ] Refactor mocks to be capability-based
- [ ] Update all examples to use capabilities

**Example Transformation**:
```rust
// BEFORE (hardcoded) ❌
let beardog = BearDogAdapter::new();
beardog.authenticate(user).await?;

// AFTER (capability-based) ✅
let auth_providers = adapter.discover_capability_providers("authentication").await?;
let provider = auth_providers.first().ok_or(NoProviderError)?;
provider.authenticate(user).await?;
```

---

### **5. TODO/FIXME Cleanup** (4-6 weeks, ongoing)
**Count**: 3,478 instances across 321 files

#### **Week 1: Categorization**
```bash
# Generate TODO report
rg "TODO|FIXME|MOCK|HACK" --type rust > TODO_REPORT.txt

# Categorize by type
rg "TODO:" --type rust | wc -l  # ~2,500
rg "FIXME:" --type rust | wc -l  # ~500
rg "MOCK:" --type rust | wc -l   # ~400
rg "HACK:" --type rust | wc -l   # ~78
```

**Action Steps**:
- [ ] Create GitHub Issues for all P0/P1 TODOs
- [ ] Document all FIXMEs with context
- [ ] Plan mock replacement timeline
- [ ] Remove or justify all HACKs

#### **Week 2-4: Critical Path Items**
**Priority TODOs**:
```
⚠️ tests/e2e/fault_tolerance.rs: 29 TODOs (E2E tests incomplete)
⚠️ crates/songbird-universal/tests/*: 60+ TODOs (adapter tests)
⚠️ crates/songbird-test-utils/src/mocks/*: 100+ MOCKs
```

#### **Week 5-6: Systematic Cleanup**
- [ ] Complete all security-related TODOs
- [ ] Implement all networking TODOs
- [ ] Replace test mocks with real implementations
- [ ] Remove obsolete TODOs

---

### **6. Linting & Formatting Fixes** (30 minutes)
**Issue**: Minor syntax and formatting errors

```bash
# Fix attribute syntax
# File: crates/songbird-canonical/tests/types_enhanced_tests.rs:11
# Change: #![allow(clippy::unwrap_used)]
# To:     #[allow(clippy::unwrap_used)]

# Run formatter
cargo fmt --all

# Verify
cargo fmt --check
cargo clippy --workspace -- -D warnings
```

**Action Steps**:
- [ ] Fix inner attribute syntax error
- [ ] Run cargo fmt --all
- [ ] Enable pre-commit hooks
- [ ] Add CI formatting checks

---

## 🟢 **P2 - MEDIUM PRIORITY (Weeks 5-8)**

### **7. Test Coverage Expansion** (2-3 weeks)
**Current**: 60.04%  
**Target**: 90%  
**Gap**: 29.96%

#### **Week 1: Capability Adapter** (27.24% → 90%)
**File**: `crates/songbird-universal/src/capabilities/adapter.rs` (1,111 lines)

**Coverage Breakdown**:
```
Current: 234 / 859 lines covered (27.24%)
Need:    539 more lines covered
Focus:   Core methods and error paths
```

**Test Areas**:
- [ ] Service registration/deregistration
- [ ] Capability discovery
- [ ] Health monitoring
- [ ] Connection management
- [ ] Error handling paths
- [ ] Concurrent access scenarios

#### **Week 2: Security Adapter** (14.16% → 90%)
**File**: `crates/songbird-universal/src/adapters/security.rs`

See P0 item #3 above.

#### **Week 3: Config & Integration** (various → 90%)
**Files**:
- `crates/songbird-types/src/config/unified.rs` (12.36%)
- Integration test expansion
- E2E scenario coverage

---

### **8. Clone Optimization** (2-3 weeks)
**Count**: 1,735 .clone() calls  
**Impact**: Memory allocations and performance

#### **Week 1: Profiling**
```bash
# Find clone hotspots
cargo flamegraph --bin songbird -- [args]

# Count clones by crate
for crate in crates/*; do
  echo "$crate: $(rg "\.clone\(\)" "$crate" -c)"
done
```

**Action Steps**:
- [ ] Profile production workloads
- [ ] Identify hot path clones
- [ ] Measure baseline performance
- [ ] Document optimization targets

#### **Week 2-3: Optimization**
**Strategies**:
- Use `Arc<T>` for shared immutable data
- Use `Cow<'_, T>` for conditional ownership
- Borrow references instead of cloning
- Use `&str` instead of `String.clone()`

**Example**:
```rust
// BEFORE (cloning) ❌
fn process_config(config: Config) {
    for service in config.services.clone() {
        // use service
    }
}

// AFTER (borrowing) ✅
fn process_config(config: &Config) {
    for service in &config.services {
        // use service
    }
}

// OR with Arc for shared data
fn process_config(config: Arc<Config>) {
    for service in &config.services {
        // use service without cloning
    }
}
```

---

### **9. Documentation Expansion** (1-2 weeks)
**Issue**: Missing API documentation

```bash
# Check doc coverage
cargo doc --workspace --no-deps 2>&1 | grep warning | wc -l

# Add missing docs lint
cargo clippy -- -W clippy::missing_docs_in_private_items
```

**Action Steps**:
- [ ] Week 1: Add `#![warn(missing_docs)]` to all crates
- [ ] Document all public APIs
- [ ] Add module-level documentation
- [ ] Update code examples
- [ ] Week 2: Generate and review documentation
- [ ] Fix broken doc links
- [ ] Add usage examples

---

## 🔵 **P3 - LOWER PRIORITY (Weeks 9-12)**

### **10. Chaos Engineering Expansion** (2 weeks)
**Status**: Infrastructure ready, needs scenarios

**Test Scenarios**:
```rust
// Network failures
- Packet loss (10%, 50%, 100%)
- Latency injection (50ms, 500ms, 5s)
- Network partitions
- Connection drops

// Service failures
- Random instance kills
- Slow responses (timeout testing)
- Intermittent errors
- Resource exhaustion

// Resource constraints
- CPU throttling
- Memory pressure  
- Disk I/O limits
- Network bandwidth limits
```

---

### **11. Unsafe Code Review** (1 week)
**Count**: 169 unsafe blocks (mostly justified)

**Action Steps**:
- [ ] Document all unsafe blocks with safety invariants
- [ ] Memory safety audit of registry persistence (10 unsafe blocks)
- [ ] Add Miri testing for unsafe code
- [ ] Consider safe alternatives

---

### **12. Performance Benchmarking** (1-2 weeks)
**Status**: Infrastructure exists, needs baseline

```bash
# Run benchmarks
cargo bench

# Profile hot paths
cargo flamegraph --bench comprehensive_performance_benchmarks
```

**Metrics to Track**:
- Discovery latency (p50, p95, p99)
- Routing decision time
- Memory usage under load
- Concurrent request throughput

---

## 📅 **TIMELINE OVERVIEW**

### **Month 1** (Weeks 1-4)
- ✅ Week 1: P0 items (unwraps, panics, security)
- ✅ Week 2-3: Zero-hardcoding migration
- ✅ Week 4: TODO categorization and tracking

### **Month 2** (Weeks 5-8)
- ✅ Week 5-6: Test coverage expansion (60% → 80%)
- ✅ Week 7: Clone optimization
- ✅ Week 8: Documentation expansion

### **Month 3** (Weeks 9-12)
- ✅ Week 9-10: Test coverage completion (80% → 90%)
- ✅ Week 11: Chaos engineering
- ✅ Week 12: Performance benchmarking & optimization

---

## 📊 **PROGRESS TRACKING**

### **Use GitHub Issues/Projects**:
```
Labels:
- P0-critical
- P1-high
- P2-medium
- P3-low
- tech-debt
- unwrap-elimination
- hardcoding
- test-coverage
- performance
```

### **Weekly Review Template**:
```markdown
## Week X Progress

### Completed
- [ ] Task 1
- [ ] Task 2

### In Progress
- [ ] Task 3 (60%)

### Blockers
- Issue #123

### Metrics
- Coverage: X%
- Unwraps: X remaining
- TODOs: X remaining
```

---

## 🎯 **SUCCESS CRITERIA**

### **Production Ready Checklist**:
- [ ] Zero production unwraps (or all documented)
- [ ] Zero production panics (or all justified)
- [ ] Security adapter >85% coverage
- [ ] Overall coverage >85%
- [ ] Zero hardcoded IPs/ports in production code
- [ ] Capability-based discovery exclusively
- [ ] All P0 TODOs completed
- [ ] All tests passing
- [ ] Documentation complete
- [ ] Performance benchmarks passing

---

**Start Here**: P0 items, then work down the priority list systematically.

**Questions?**: Review the [Comprehensive Audit Report](./COMPREHENSIVE_AUDIT_REPORT_DEC_2_2025.md) for details.

**Status Tracking**: Update this document weekly with progress.

