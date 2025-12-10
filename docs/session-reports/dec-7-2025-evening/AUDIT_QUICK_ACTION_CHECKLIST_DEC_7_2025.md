# ⚡ QUICK ACTION CHECKLIST - December 7, 2025 Evening Audit

**Status**: 🎯 Prioritized action items from comprehensive audit  
**Grade**: B+ (87/100) → Target: A (95/100)  
**Time to Target**: 120-150 hours (3-4 weeks)

---

## 🚨 PRIORITY 0: BLOCKERS (Fix Immediately)

### [ ] 1. Fix Compilation Error (1 hour)
```bash
File: crates/songbird-types/tests/config_canonical_environment_tests.rs:24
Error: no field `log_config` on type `CanonicalEnvironmentConfig`
Action: Fix struct definition or update test
```

### [ ] 2. Run Code Formatting (5 minutes)
```bash
cargo fmt --all
# Fixes: crates/songbird-config/src/cloud_agnostic.rs
```

---

## 🔥 PRIORITY 1: HIGH (Complete This Week)

### [ ] 3. Implement mDNS Discovery (30-40 hours)
**Files**: 
- `crates/songbird-config/src/discovery/mdns.rs` (4 TODOs)
- `crates/songbird-config/src/discovery/runtime_engine.rs` (line 245)

**Tasks**:
- [ ] Add `mdns` crate dependency
- [ ] Implement `advertise()` method
- [ ] Implement `query()` method  
- [ ] Implement `discover()` method
- [ ] Implement `shutdown()` method
- [ ] Add integration tests
- [ ] Test on local network

### [ ] 4. Fix Broken Tests (10-15 hours)
**File**: `crates/songbird-orchestrator/tests/main_tests.rs`

**Tasks**:
- [ ] Update `test_defaults()` for canonical config
- [ ] Update `test_environment_config()` for new API
- [ ] Update `test_performance_config()` structure
- [ ] Remove or fix `test_config_validation()`
- [ ] Update `test_orchestrator_initialization()`
- [ ] Fix `test_service_registration()`
- [ ] Update remaining 2 test functions
- [ ] Run full test suite: `cargo test --all`

### [ ] 5. Reduce Documentation Warnings (30-40 hours)
**Current**: 542 warnings → **Target**: <50 warnings

**Focus Files**:
- [ ] `crates/songbird-cli/src/errors.rs` - Add pub item docs
- [ ] `crates/songbird-discovery/src/traits/discovery.rs` - 24 warnings
- [ ] All public API functions (add `///` doc comments)
- [ ] All public structs (add `///` doc comments)
- [ ] All public enums (add `///` doc comments)

**Commands**:
```bash
cargo doc --no-deps 2>&1 | grep "warning:" | head -50
# Fix top 50, iterate
```

### [ ] 6. Audit Production Unwrap/Expect (30-40 hours)
**Found**: ~400 production instances → **Target**: <50 instances

**Process**:
1. [ ] Generate list:
   ```bash
   grep -r "\.unwrap()" crates/*/src --include="*.rs" > unwrap_audit.txt
   grep -r "\.expect(" crates/*/src --include="*.rs" >> unwrap_audit.txt
   ```

2. [ ] Categorize each instance:
   - High risk: External input, network calls, file I/O
   - Medium risk: Post-validation (document safety)
   - Low risk: Infallible operations (keep with comment)

3. [ ] Convert high-risk to proper error handling:
   ```rust
   // ❌ Before
   let value = map.get(key).unwrap();
   
   // ✅ After
   let value = map.get(key).ok_or(Error::KeyNotFound)?;
   ```

### [ ] 7. Split Large Files (4-6 hours)
**Files Over 1000 Lines**:

1. [ ] `crates/songbird-universal/src/discovery.rs` (1,023 lines)
   - Split into: `discovery/core.rs`, `discovery/backends.rs`, `discovery/types.rs`
   
2. [ ] `crates/songbird-universal/src/capabilities/adapter.rs` (1,014 lines)
   - Split into: `adapter/core.rs`, `adapter/implementations.rs`, `adapter/tests.rs`

---

## ⚡ PRIORITY 2: MEDIUM (Next Sprint)

### [ ] 8. Improve Test Coverage (40-60 hours)
**Current**: ~68% → **Target**: 90%

**Strategy**:
1. [ ] Run coverage analysis:
   ```bash
   cargo llvm-cov --all-features --workspace --html
   ```

2. [ ] Focus areas:
   - [ ] Error recovery paths
   - [ ] Edge cases in discovery
   - [ ] Concurrent operations
   - [ ] Fault injection scenarios

3. [ ] Add missing tests for:
   - [ ] `capabilities_adapter_tests.rs` (+600 lines)
   - [ ] `sovereignty_adapter_tests.rs` (+750 lines)

### [ ] 9. Implement Kubernetes Discovery (20-30 hours)
**File**: `crates/songbird-config/src/discovery/runtime_engine.rs:281`

**Tasks**:
- [ ] Add `kube` crate dependency
- [ ] Implement K8s service discovery
- [ ] Support in-cluster and kubeconfig modes
- [ ] Add namespace filtering
- [ ] Add label selector support
- [ ] Add integration tests with kind/minikube

### [ ] 10. Start tarpc Protocol Implementation (40-50 hours)
**Spec**: `specs/TARPC_JSON_RPC_PROTOCOL_SPEC.md`

**Week 1-2 Tasks**:
- [ ] Add `tarpc` crate dependency
- [ ] Create `crates/songbird-tarpc/` module
- [ ] Define service traits
- [ ] Implement client/server
- [ ] Add JSON-RPC gateway
- [ ] Performance benchmarks (target: 10-100x HTTP)

### [ ] 11. Expand E2E Testing (40-50 hours)
**Current**: 7 basic scenarios → **Target**: 20+ comprehensive scenarios

**New Tests**:
- [ ] Multi-node coordination (3+ nodes)
- [ ] Long-running stability (24h+ tests)
- [ ] Network partition scenarios
- [ ] Large-scale load tests (100+ concurrent)
- [ ] Cascading failure recovery
- [ ] Byzantine fault scenarios

### [ ] 12. Optimize Clone Usage (20-30 hours)
**Found**: 2,066 `.clone()` calls

**Strategy**:
1. [ ] Profile hot paths:
   ```bash
   cargo bench
   cargo flamegraph
   ```

2. [ ] Optimize expensive clones:
   - [ ] Use `&str` instead of `String.clone()`
   - [ ] Use `Cow<'a, T>` for conditional ownership
   - [ ] Use `Arc<T>` for shared ownership (expand usage)
   - [ ] Use `Bytes` crate for buffer sharing

---

## 🔮 PRIORITY 3: LOW (Future Work)

### [ ] 13. Implement Remaining Discovery Backends (40-50 hours)
- [ ] DNS-SD discovery (line 257)
- [ ] Consul discovery (line 269)
- [ ] etcd discovery (line 281)

### [ ] 14. Build Comprehensive Chaos Suite (60+ hours)
- [ ] Jepsen-style consistency testing
- [ ] Byzantine fault tolerance validation
- [ ] Resource exhaustion scenarios
- [ ] Cascading failure testing

### [ ] 15. Achieve Full Pedantic Compliance (120 hours)
- [ ] Fix all clippy pedantic warnings
- [ ] Enable stricter lints
- [ ] Document all private items
- [ ] Eliminate all bool-heavy structs

### [ ] 16. Complete Extended Test Coverage (30-40 hours)
- [ ] Add ~600 lines to capabilities tests
- [ ] Add ~750 lines to sovereignty tests
- [ ] Add edge case coverage

---

## 📊 PROGRESS TRACKING

### Completion Checklist

**Priority 0 (Blockers)**: 0/2 complete
- [ ] Compilation error
- [ ] Formatting

**Priority 1 (High)**: 0/5 complete
- [ ] mDNS discovery
- [ ] Broken tests
- [ ] Documentation warnings
- [ ] Unwrap audit
- [ ] Large file splits

**Priority 2 (Medium)**: 0/5 complete
- [ ] Test coverage
- [ ] K8s discovery
- [ ] tarpc protocol
- [ ] E2E testing
- [ ] Clone optimization

**Priority 3 (Low)**: 0/4 complete
- [ ] Remaining discovery backends
- [ ] Chaos suite
- [ ] Pedantic compliance
- [ ] Extended test coverage

---

## 🎯 MILESTONES

### Week 1 Target: B+ → A- (90/100)
- [x] Complete audit
- [ ] Fix P0 blockers (2 items)
- [ ] Complete 3/5 P1 items
- [ ] Grade: A- (90/100)

### Week 2 Target: A- → A (93/100)
- [ ] Complete remaining P1 items
- [ ] Start 2/5 P2 items
- [ ] Grade: A (93/100)

### Week 3 Target: A → A+ (95/100)
- [ ] Complete 4/5 P2 items
- [ ] 80%+ test coverage
- [ ] <100 doc warnings
- [ ] Grade: A+ (95/100)

### Month 1 Target: A+ (97/100)
- [ ] All P1 complete
- [ ] All P2 complete
- [ ] 2/4 P3 started
- [ ] Grade: A+ (97/100)

---

## 🛠️ QUICK COMMANDS

### Immediate Actions
```bash
# Fix formatting
cargo fmt --all

# Check compilation
cargo build --all --all-features

# Run tests
cargo test --all

# Check lints
cargo clippy --all-targets --all-features

# Generate docs
cargo doc --no-deps --open

# Check coverage
cargo llvm-cov --all-features --workspace --html
```

### Audit Commands
```bash
# Count TODOs
grep -r "TODO\|FIXME" crates/*/src --include="*.rs" | wc -l

# Count unwraps
grep -r "\.unwrap()" crates/*/src --include="*.rs" | wc -l

# Count clones
grep -r "\.clone()" crates --include="*.rs" | wc -l

# Check file sizes
find crates -name "*.rs" -exec wc -l {} + | awk '$1 > 1000 {print $0}'
```

---

## 📞 SUPPORT

**Full Audit Report**: `COMPREHENSIVE_AUDIT_REPORT_DEC_7_2025_EVENING.md`  
**Current Status**: `CURRENT_STATUS.md`  
**Specifications**: `specs/00_SPECIFICATIONS_INDEX.md`

---

**Created**: December 7, 2025, 8:00 PM  
**Next Review**: December 14, 2025 (1 week)  
**Target Completion**: January 7, 2026 (1 month)

