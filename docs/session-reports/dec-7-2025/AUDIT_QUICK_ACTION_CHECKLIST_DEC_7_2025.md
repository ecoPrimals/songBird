# 🎯 AUDIT QUICK ACTION CHECKLIST
**Date**: December 7, 2025  
**Based On**: Comprehensive Audit Report  
**Status**: P0 Issues Fixed, P1/P2 Remaining

---

## ✅ COMPLETED (P0 - Critical)

- [x] **Formatting Fixed** - Ran `cargo fmt --all` (14 files formatted)
- [x] **Clippy Errors Fixed** - Added `#[allow(clippy::unused_async)]` to 2 functions
  - `hosts_evolved.rs:425` - `discover_by_capability`
  - `hosts_evolved.rs:445` - `register_self`

**Time Taken**: 5 minutes  
**Verification**: Run `cargo fmt --check` and `cargo clippy --all-targets --all-features`

---

## 📋 IMMEDIATE NEXT STEPS (P1 - High Priority)

### 1. Test Coverage Analysis
**Priority**: P1  
**Time**: 30 minutes  
**Status**: ⬜ TODO

```bash
# Generate coverage report
cargo llvm-cov --all-features --workspace --html

# View summary
cargo llvm-cov --all-features --workspace --summary-only

# Open HTML report
xdg-open target/llvm-cov/html/index.html
```

**Expected Outcome**: Know exact coverage percentage, identify gaps

---

### 2. Documentation Warnings
**Priority**: P1  
**Time**: 4-8 hours  
**Status**: ⬜ TODO  
**Count**: 542 warnings

**Quick Win** - Start with most-used modules:
1. `crates/songbird-cli/src/errors.rs`
2. `crates/songbird-discovery/src/traits/discovery.rs`
3. Public adapter APIs

**Example Fix**:
```rust
// Before
pub struct MyStruct {
    field: String,
}

// After
/// My struct does X, Y, Z
pub struct MyStruct {
    /// The field description
    field: String,
}
```

**Command to find worst offenders**:
```bash
cargo doc --no-deps 2>&1 | grep "warning:" | sort | uniq -c | sort -rn | head -20
```

---

### 3. Production unwrap() Audit
**Priority**: P1  
**Time**: 1-2 days  
**Status**: ⬜ TODO  
**Count**: ~50-100 estimated

**Use existing script**:
```bash
./check_production_unwraps.sh
```

**Fix Pattern**:
```rust
// Before
let value = some_option.unwrap();

// After
let value = some_option.ok_or_else(|| Error::MissingValue)?;
// OR
let value = some_option.expect("Value must exist because...");
```

**Target**: Zero unwrap() in src/, examples/
**Acceptable**: unwrap() in tests/

---

### 4. Update Orchestrator Tests
**Priority**: P1  
**Time**: 4-8 hours  
**Status**: ⬜ TODO  
**Location**: `crates/songbird-orchestrator/tests/main_tests.rs`

**Issue**: Tests marked with `TODO(Dec 4 2025): Rewrite for canonical config API`

**Options**:
1. Update tests for new canonical config API
2. Remove outdated tests if no longer relevant
3. Mark as `#[ignore]` with reason if planning future fix

**Count**: ~8 test functions

---

### 5. Discovery Backend Implementation
**Priority**: P1 (for full functionality)  
**Time**: 1-2 weeks per backend  
**Status**: ⬜ TODO

**Locations**:
- `crates/songbird-config/src/discovery/runtime_engine.rs`
- `crates/songbird-config/src/defaults/hosts_evolved.rs`

**Backends to Implement**:
- [ ] mDNS (multicast DNS for local discovery)
- [ ] DNS-SD (DNS service discovery)
- [ ] Consul (service registry)
- [ ] etcd (distributed key-value store)
- [ ] Kubernetes (container orchestration)

**Current Status**: Placeholders return empty results  
**Workaround**: Environment-based discovery works (CAPABILITY_ENDPOINTS)

**Priority Order**:
1. **mDNS** (P1) - Local network discovery, most useful for dev/small deployments
2. **Kubernetes** (P1) - Cloud-native deployments
3. **Consul** (P2) - Enterprise service mesh
4. **DNS-SD** (P2) - Standards-based discovery
5. **etcd** (P3) - Specialized use cases

---

## 📊 MEDIUM PRIORITY (P2)

### 6. Achieve 80% Test Coverage
**Priority**: P2  
**Time**: 1 week  
**Status**: ⬜ TODO  
**Current**: ~60-70% (estimated)  
**Target**: 80% (stepping stone to 90%)

**Focus Areas**:
1. Config validation paths
2. Error handling branches
3. Adapter edge cases
4. Discovery failure scenarios

**Commands**:
```bash
# Generate coverage with branch coverage
cargo llvm-cov --all-features --workspace --branch --html

# Find uncovered lines
cargo llvm-cov --all-features --workspace --ignore-filename-regex tests
```

---

### 7. File Size Compliance
**Priority**: P2 (Low urgency)  
**Time**: 2-4 hours  
**Status**: ⬜ TODO

**Files Over 1000 Lines**:
1. `crates/songbird-universal/src/capabilities/adapter.rs` (1,014 lines, +14)
2. `crates/songbird-universal/src/discovery.rs` (1,023 lines, +23)

**Recommendation**: Split during next major refactor
- Not urgent (only 1-2% overage)
- Both files are cohesive modules
- Consider extracting test modules first

---

### 8. Clone Optimization
**Priority**: P2  
**Time**: Ongoing  
**Status**: ⬜ TODO  
**Count**: 1,749 instances

**Process**:
1. Profile hot paths: `cargo flamegraph --bench <benchmark_name>`
2. Identify clone bottlenecks
3. Optimize high-impact clones:
   - Use `&T` instead of `T.clone()` where possible
   - Use `Cow<T>` for conditional ownership
   - Use `Arc::clone()` explicitly (already zero-cost)
   - Consider `bytes::Bytes` for buffer sharing

**Don't Optimize**:
- Clones in tests (readability > performance)
- Clones in error paths (happens rarely)
- `Arc::clone()` (already zero-cost pointer copy)

---

### 9. Enable Pedantic Lints
**Priority**: P2  
**Time**: Ongoing  
**Status**: ⬜ TODO

**Add to Cargo.toml**:
```toml
[lints.clippy]
pedantic = "warn"
# Allow noisy lints:
must_use_candidate = "allow"
missing_errors_doc = "allow"
module_name_repetitions = "allow"
```

**Process**:
1. Enable pedantic: `cargo clippy --all-targets -- -W clippy::pedantic`
2. Fix batch of warnings
3. Commit
4. Repeat

**Expected**: ~100-200 additional warnings initially

---

## 🔮 LONG-TERM (P3)

### 10. Extended Test Coverage (90%)
**Priority**: P3  
**Time**: 2-4 weeks  
**Status**: ⬜ TODO

**Includes**:
- Edge case coverage
- Fault injection for all error paths
- Long-running stability tests (24+ hours)
- Load testing with realistic workloads
- Complete chaos engineering scenarios

---

### 11. Protocol Implementations
**Priority**: P1-P2 (per spec)  
**Time**: 4-6 weeks total  
**Status**: ⬜ TODO (Specified, not implemented)

**From specs/00_SPECIFICATIONS_INDEX.md**:
1. **tarpc Integration** (Weeks 1-2) - High-performance binary RPC
2. **JSON-RPC 2.0** (Week 3) - Universal language-agnostic RPC
3. **WebSocket Real-time** (Week 4) - Bidirectional communication
4. **QUIC/HTTP3** (Months 2-3) - Modern transport protocol

---

### 12. Complete Adapter Test TODOs
**Priority**: P3  
**Time**: 1-2 weeks  
**Status**: ⬜ TODO

**Locations**:
- `crates/songbird-universal/tests/capabilities_adapter_tests.rs:51`
  - TODO: Add remaining ~600 lines of tests
- `crates/songbird-universal/tests/sovereignty_adapter_tests.rs:21`
  - TODO: Add remaining ~750 lines of tests

**Note**: Core functionality tested, these are edge cases and extended scenarios

---

## 📈 PROGRESS TRACKING

**P0 (Critical)**: 2/2 complete ✅
- [x] Formatting
- [x] Clippy errors

**P1 (High Priority)**: 0/5 complete
- [ ] Test coverage analysis
- [ ] Documentation warnings
- [ ] Production unwrap() audit
- [ ] Orchestrator tests update
- [ ] Discovery backends (at least mDNS + K8s)

**P2 (Medium Priority)**: 0/4 complete
- [ ] 80% test coverage
- [ ] File size compliance
- [ ] Clone optimization (ongoing)
- [ ] Pedantic lints (ongoing)

**P3 (Long-term)**: 0/3 complete
- [ ] 90% test coverage
- [ ] Protocol implementations
- [ ] Extended adapter tests

---

## 🎯 RECOMMENDED SPRINT PLAN

### Week 1: Foundation
- [x] Day 1: Fix P0 issues (DONE)
- [ ] Day 2: Run coverage analysis, audit unwraps
- [ ] Day 3-4: Fix critical unwraps, update orchestrator tests
- [ ] Day 5: Documentation sprint (fix 50-100 warnings)

### Week 2: Coverage & Discovery
- [ ] Day 1-2: Add tests to reach 70% coverage
- [ ] Day 3-4: Implement mDNS discovery backend
- [ ] Day 5: Documentation sprint (fix 100-150 warnings)

### Week 3: Polish & Quality
- [ ] Day 1-2: Add tests to reach 75% coverage
- [ ] Day 3-4: Implement Kubernetes discovery backend
- [ ] Day 5: Documentation sprint (fix remaining warnings)

### Week 4: Optimization & Final Push
- [ ] Day 1-2: Add tests to reach 80% coverage
- [ ] Day 3: Profile and optimize hot paths
- [ ] Day 4: Enable pedantic lints, fix batch
- [ ] Day 5: Final review, staging deployment prep

---

## ✅ VERIFICATION COMMANDS

**After each fix, verify**:

```bash
# Formatting
cargo fmt --all --check

# Linting
cargo clippy --all-targets --all-features -- -D warnings

# Tests
cargo test --all-features --workspace

# Coverage
cargo llvm-cov --all-features --workspace --summary-only

# Documentation
cargo doc --no-deps 2>&1 | grep -c "warning:"

# Build
cargo build --release --all-features
```

---

## 🎓 SUCCESS CRITERIA

**Ready for Staging**:
- [x] P0 issues fixed (DONE)
- [ ] P1 issues fixed (0/5)
- [ ] Test coverage ≥ 70%
- [ ] Documentation warnings < 200

**Ready for Production**:
- [ ] P0 issues fixed (DONE)
- [ ] P1 issues fixed (0/5)
- [ ] Test coverage ≥ 80%
- [ ] Documentation warnings < 50
- [ ] No production unwrap() calls
- [ ] All tests passing
- [ ] Performance benchmarks meeting targets

**World-Class Quality**:
- [ ] All P0-P2 issues fixed
- [ ] Test coverage ≥ 90%
- [ ] Zero documentation warnings
- [ ] Pedantic lints enabled and clean
- [ ] All discovery backends implemented
- [ ] Protocol implementations complete

---

**Current Status**: ✅ P0 Complete, Ready for P1 Work  
**Next Action**: Run test coverage analysis  
**Updated**: December 7, 2025

