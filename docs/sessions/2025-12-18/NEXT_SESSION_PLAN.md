# 📋 NEXT SESSION PLAN
**Prepared**: December 18, 2025  
**Current Progress**: Evolution momentum established  
**Priority**: Continue systematic improvements

---

## 🎯 PRIMARY OBJECTIVES

### 1. **Continue unwrap() Evolution** (7% → 25%)

**Target Files**:
- `resource_management/scheduler.rs` (18 unwraps) - PRIORITY
- `resource_management/admission.rs` (6 unwraps in tests)
- `task_lifecycle/checkpoint.rs` (2 unwraps)
- `rpc/tarpc_server.rs` (3 unwraps)

**Approach**:
```rust
// Pattern: Test code unwraps are acceptable
#[test]
fn test_something() {
    let result = do_thing().unwrap();  // OK in tests
}

// Pattern: Production code needs error handling
pub async fn do_thing() -> Result<T> {
    let value = thing.get()
        .ok_or_else(|| anyhow!("Missing thing"))?;  // Proper error handling
}
```

**Goal**: Fix 15-20 unwraps (25% total completion)

---

### 2. **Expand Documentation** (75% → 85%)

**Target Modules**:
- `resource_management::FairScheduler` - Core scheduling API
- `resource_management::AdmissionController` - Admission control
- `error_recovery` - Already has good structure, add examples
- `observability` - Metrics and monitoring APIs

**Pattern**:
```rust
/// Brief description.
///
/// Longer explanation of purpose and behavior.
///
/// # Arguments
/// * `param1` - What this parameter does
/// * `param2` - What this parameter does
///
/// # Returns
/// What is returned and when
///
/// # Errors
/// When and why errors occur
///
/// # Example
/// ```no_run
/// let thing = Thing::new();
/// thing.do_something()?;
/// ```
pub fn api_method(&self, param1: Type, param2: Type) -> Result<Return> { ... }
```

**Goal**: 20 APIs fully documented

---

### 3. **Test Coverage Analysis**

**Actions**:
```bash
# Run coverage on modified modules
cargo llvm-cov --package songbird-orchestrator --html

# Check specific modules
cargo llvm-cov --package songbird-orchestrator \
  --lib task_lifecycle --lib resource_management \
  --html
```

**Focus Areas**:
- Error paths in newly evolved unwrap() code
- Edge cases in admission control
- State transitions in task lifecycle
- Circuit breaker state machine

**Goal**: Identify 10-15 coverage gaps, add targeted tests

---

## 🔄 SECONDARY OBJECTIVES

### 4. **Profile unsafe Code** (Prepare for Migration)

**Setup**:
```bash
# Install benchmarking tools
cargo install cargo-criterion

# Run benchmarks
cd /home/eastgate/Development/ecoPrimals/songbird
cargo bench --package songbird-types

# Profile specific functions
cargo bench -- --profile-time=10 zero_copy
```

**Compare**:
- `SafeZeroCopyBuffer` (7 unsafe blocks)
- `ModernSafeBuffer` (0 unsafe blocks)

**Measure**:
- Throughput (ops/sec)
- Latency (μs per op)
- Memory usage
- CPU usage

**Goal**: Establish performance baseline, document <1% difference

---

### 5. **Design Capability Discovery** (Prototype)

**Create Proof-of-Concept**:
```rust
// File: crates/songbird-config/src/self_knowledge.rs

/// Node's self-knowledge (no external hardcoding)
pub struct SelfKnowledge {
    node_id: NodeId,
    capabilities: Vec<Capability>,
    bind_address: SocketAddr,  // What we bind to
    advertise_address: Option<SocketAddr>,  // What we advertise
}

/// Runtime discovery (no hardcoded addresses)
pub struct RuntimeDiscovery {
    discovery_methods: Vec<Box<dyn DiscoveryMethod>>,
}

impl RuntimeDiscovery {
    /// Discover providers for a capability
    pub async fn find_providers(&self, capability: &str) -> Result<Vec<Provider>> {
        // Try methods in order: mDNS, DNS-SD, service registry, etc.
        for method in &self.discovery_methods {
            if let Ok(providers) = method.discover(capability).await {
                if !providers.is_empty() {
                    return Ok(providers);
                }
            }
        }
        Err(anyhow!("No providers found for capability: {}", capability))
    }
}
```

**Test**:
- Dynamic primal addition/removal
- Graceful handling of missing capabilities
- Fallback discovery methods

**Goal**: Validate architecture with working prototype

---

## 📈 SUCCESS METRICS

### Session Goals:

**Minimum (Must Achieve)**:
- [ ] 15 unwraps evolved (→ 20/67 = 30%)
- [ ] 15 APIs documented (→ 80% coverage)
- [ ] 5 tests added (coverage gaps)

**Target (Should Achieve)**:
- [ ] 20 unwraps evolved (→ 25/67 = 37%)
- [ ] 20 APIs documented (→ 85% coverage)
- [ ] 10 tests added (error paths)
- [ ] unsafe profiling complete

**Stretch (Nice to Have)**:
- [ ] 25 unwraps evolved (→ 30/67 = 45%)
- [ ] 25 APIs documented (→ 90% coverage)
- [ ] 15 tests added (edge cases)
- [ ] Discovery prototype working

---

## 🔍 SPECIFIC TASKS

### Task List (Pick & Execute):

#### A. unwrap() Evolution Tasks:
- [ ] Fix scheduler.rs comparisons (use unwrap_or explicitly)
- [ ] Fix admission.rs test unwraps (document why acceptable)
- [ ] Fix checkpoint.rs timestamp handling
- [ ] Fix tarpc_server.rs connection handling

#### B. Documentation Tasks:
- [ ] Document FairScheduler struct
- [ ] Document FairScheduler::enqueue()
- [ ] Document FairScheduler::dequeue()
- [ ] Document AdmissionController::evaluate()
- [ ] Document AdmissionController::admit()
- [ ] Document AdmissionController::release()
- [ ] Document RetryPolicy API
- [ ] Document CircuitBreaker API
- [ ] Document MetricsCollector API

#### C. Testing Tasks:
- [ ] Add test: quota allocation edge case
- [ ] Add test: scheduler priority handling
- [ ] Add test: admission controller overload
- [ ] Add test: retry policy max attempts
- [ ] Add test: circuit breaker transitions
- [ ] Add test: task lifecycle state machine
- [ ] Add test: checkpoint compression failure
- [ ] Add test: storage corruption handling

#### D. Profiling Tasks:
- [ ] Benchmark SafeZeroCopyBuffer throughput
- [ ] Benchmark ModernSafeBuffer throughput
- [ ] Measure memory allocations (both)
- [ ] Profile in production-like workload
- [ ] Document performance characteristics

#### E. Discovery Tasks:
- [ ] Create SelfKnowledge struct
- [ ] Implement RuntimeDiscovery trait
- [ ] Add mDNS discovery method
- [ ] Add DNS-SD discovery method
- [ ] Test dynamic provider discovery
- [ ] Document discovery architecture

---

## 🎓 PATTERNS TO APPLY

### Pattern 1: Smart unwrap() Evolution

**In Production Code**:
```rust
// BEFORE: Panic on None
let value = map.get(&key).unwrap();

// AFTER: Graceful error with context
let value = map
    .get(&key)
    .ok_or_else(|| anyhow!("Missing key: {:?}", key))?;
```

**In Test Code** (acceptable):
```rust
// OK: Tests should fail fast with clear panic messages
#[test]
fn test_thing() {
    let result = thing.do_work().unwrap();
    assert_eq!(result, expected);
}

// EVEN BETTER: Explicit error message
#[test]
fn test_thing() {
    let result = thing.do_work().expect("Work should succeed");
    assert_eq!(result, expected);
}
```

### Pattern 2: Comprehensive Documentation

```rust
/// One-line summary of what this does.
///
/// Longer explanation covering:
/// - Purpose and behavior
/// - When to use vs alternatives
/// - Performance characteristics (if relevant)
///
/// # Arguments
/// * `param` - Clear explanation of parameter purpose and constraints
///
/// # Returns
/// What is returned and any special values/states
///
/// # Errors
/// All error conditions and their causes
///
/// # Panics
/// Document any panic conditions (there shouldn't be any!)
///
/// # Example
/// ```no_run
/// # use your_crate::Thing;
/// # fn example() -> anyhow::Result<()> {
/// let thing = Thing::new()?;
/// let result = thing.do_work()?;
/// # Ok(())
/// # }
/// ```
///
/// # See Also
/// - [`RelatedType`] for related functionality
/// - [`AlternativeMethod`] for alternative approach
pub fn api_method(&self, param: Type) -> Result<Return> { ... }
```

### Pattern 3: Test Coverage for Error Paths

```rust
#[tokio::test]
async fn test_error_handling_quota_exceeded() {
    let quota = ResourceQuota::new(UserId::from("test"));
    let mut excessive_request = HashMap::new();
    excessive_request.insert(
        ResourceType::Cpu,
        ResourceAmount::new(1000.0, ResourceUnit::Cores)  // Way over limit
    );
    
    // Should return error, not panic
    let result = quota.allocate(&excessive_request);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("exceed"));
}
```

---

## 🛠️ TOOLS & COMMANDS

### Essential Commands:

```bash
# Build & test
cargo build --workspace
cargo test --workspace --lib

# Coverage
cargo llvm-cov --workspace --html --ignore-run-fail
open target/llvm-cov/html/index.html

# Benchmarks
cargo bench --package songbird-types
cargo criterion

# Linting
cargo clippy --workspace --all-targets
cargo fmt --check

# Documentation
cargo doc --workspace --no-deps --open
```

### Quick Checks:

```bash
# Find remaining unwraps in production
grep -r "\.unwrap()" crates/songbird-orchestrator/src \
  --exclude-dir=tests | wc -l

# Check documentation coverage
cargo rustdoc -- -Z unstable-options --show-coverage

# Run specific test
cargo test --package songbird-orchestrator test_task_lifecycle

# Profile specific function
cargo bench --package songbird-types -- zero_copy
```

---

## 📝 SESSION CHECKLIST

### Before Starting:
- [ ] Review previous session notes
- [ ] Check current git status
- [ ] Run tests to ensure clean starting point
- [ ] Read EVOLUTION_STATUS_DEC_18_2025.md

### During Session:
- [ ] Work systematically (don't jump around)
- [ ] Run tests after each change
- [ ] Document as you go
- [ ] Update progress tracker

### After Session:
- [ ] Run full test suite
- [ ] Update EVOLUTION_STATUS document
- [ ] Create session summary
- [ ] Prepare next session plan
- [ ] Git commit progress (if ready)

---

## 🎯 EXPECTED OUTCOMES

### By End of Next Session:

**Code Quality**:
- 25-30% of production unwraps evolved
- 85-90% API documentation coverage
- 65-70% test coverage (from 63%)
- Clean builds, all tests passing

**Technical Debt**:
- unwrap count: 67 → ~50
- Undocumented APIs: ~25 → ~10
- Coverage gaps identified and prioritized

**Foundation Work**:
- unsafe profiling complete
- Discovery architecture validated
- Performance baselines established

**Documentation**:
- Session summary generated
- Patterns documented
- Evolution progress tracked

---

## 💡 TIPS FOR SUCCESS

### 1. Stay Systematic
- Pick one module, complete it fully
- Don't context-switch between different types of work
- Finish documentation while code is fresh

### 2. Test-Driven
- Run tests after every change
- Add tests for new error paths
- Verify no regressions

### 3. Document Decisions
- Why certain unwraps remain (if any)
- Performance trade-offs (safe vs unsafe)
- Architecture choices (discovery design)

### 4. Celebrate Progress
- Acknowledge completed milestones
- Track metrics improving
- Note perfect areas (mocks, files)

### 5. Maintain Momentum
- Quick wins (clippy fixes) boost morale
- Deep work (unwraps) shows progress
- Balance keeps energy high

---

## 🚀 READY TO CONTINUE

**Current State**: ✅ Production ready with ongoing evolution  
**Next Goal**: 25% unwrap evolution, 85% documentation  
**Timeline**: 2-3 hour session  
**Confidence**: 🟢 HIGH - Clear plan, proven patterns

**Let's continue evolving toward world-class Rust!** 🦀

---

**Plan Created**: December 18, 2025  
**Status**: Ready for next session  
**Priority**: P1 - High Impact Work


