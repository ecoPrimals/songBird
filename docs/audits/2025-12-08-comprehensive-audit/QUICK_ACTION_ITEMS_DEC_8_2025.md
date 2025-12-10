# ⚡ QUICK ACTION ITEMS - December 8, 2025

**Priority Order**: P0 → P1 → P2 → P3

---

## 🚨 P0: CRITICAL (FIX TODAY - 30 MINUTES)

### 1. Fix Syntax Errors (BLOCKS EVERYTHING)

**File 1**: `crates/songbird-discovery/tests/discovery_integration_comprehensive_tests.rs`
- Line 243: Extra closing brace
- Fix: Remove duplicate `}` or add missing opening brace
- Time: 5 minutes

**File 2**: `crates/songbird-network-federation/tests/node_info_tests.rs`
- Line 81: Unclosed vec! macro
- Fix: Add missing `]` in NodeInfo struct
- Time: 5 minutes

**File 3**: `crates/songbird-orchestrator/tests/compute_api_error_coverage_tests.rs`
- Line 129: Unclosed delimiter
- Fix: Balance braces in test functions
- Time: 5 minutes

**Verify**:
```bash
cargo build --lib
cargo test --lib
```

**Impact**: Unblocks all testing, linting, and coverage measurement

---

## ⚠️ P1: HIGH PRIORITY (THIS WEEK)

### 2. Measure Test Coverage (15 MINUTES)
```bash
cargo llvm-cov --html
open target/llvm-cov/html/index.html
```
**Goal**: Get baseline coverage number (target: 90%)

### 3. Split Oversized Test File (1 HOUR)

**File**: `crates/songbird-universal/tests/unified_adapter_core_tests.rs` (1,231 lines)

**Split into**:
- `unified_adapter_core_creation_tests.rs` (~300 lines)
- `unified_adapter_core_config_tests.rs` (~300 lines)
- `unified_adapter_core_capability_tests.rs` (~300 lines)
- `unified_adapter_core_lifecycle_tests.rs` (~300 lines)

### 4. Fix Deprecation Warnings (2-4 HOURS)

**Count**: 11 warnings

**Pattern**:
```rust
// BEFORE
use songbird_config::unified::SongbirdConfig;

// AFTER  
use songbird_config::canonical::SongbirdConfig;
```

**Files affected**:
- `crates/songbird-config/src/unified/mod.rs`
- Various files using deprecated config types

### 5. Audit Production Unwraps (8-12 HOURS)

**Count**: 321 files with unwrap/expect

**Process**:
1. Run `./check_production_unwraps.sh`
2. Categorize by risk:
   - Critical paths (must fix)
   - Error handling (should fix)
   - Tests (acceptable)
   - After checks (document)
3. Convert critical unwraps to proper error handling
4. Document intentional unwraps

---

## 🔄 P2: MEDIUM PRIORITY (THIS MONTH)

### 6. Complete TODO Items (30-40 HOURS)

**High Priority TODOs** (4 items):
1. DNS SRV lookup implementation (4-6 hours)
2. Network scanning discovery (8-12 hours)
3. Container orchestration discovery (8-12 hours)
4. Sovereignty test migration (6-8 hours)

**Medium Priority TODOs** (10 items):
- Canonical testing module (4-6 hours)
- Various documentation and optional features (8-12 hours total)

### 7. Hardcoding Migration (2-3 WEEKS)

**Count**: ~3,717 instances
- Ports: ~1,340
- IPs/Hosts: ~1,290
- Constants: ~1,087

**Framework exists**: `crates/songbird-config/src/defaults/`

**Approach**:
1. Week 1: Migrate ports (20-30 hours)
2. Week 2: Migrate hosts/IPs (20-30 hours)
3. Week 3: Migrate constants (20-30 hours)

**Pattern**:
```rust
// BEFORE
let port = 8080;
let host = "127.0.0.1";

// AFTER
use songbird_config::defaults::{ports, hosts};
let port = ports::orchestrator_port();
let host = hosts::localhost();
```

### 8. Expand Test Coverage (40-80 HOURS)

**Current**: Unknown (blocked by P0)  
**Target**: 90%  
**Gap**: TBD after measurement

**Strategy**:
1. Measure baseline (P1)
2. Identify uncovered areas
3. Write targeted tests
4. Focus on critical paths first

---

## 🎯 P3: LOW PRIORITY (NEXT QUARTER)

### 9. Optimize Clone Usage (40+ HOURS)

**Count**: 1,835 clone operations

**Approach**:
1. Profile first (identify hot paths)
2. Convert clones to borrows/references where possible
3. Use Cow<T> where appropriate
4. Benchmark improvements

**Note**: Only optimize if profiling shows actual impact

### 10. Full Pedantic Clippy (16-24 HOURS)

```bash
cargo clippy --all-targets --all-features -- -W clippy::all -W clippy::pedantic
```

**Expected issues**:
- Missing error docs
- Large error enums
- Module inception
- Unnecessary closures

### 11. Expand E2E Tests (40+ HOURS)

**Current**: 7 E2E test files  
**Add**:
- Full integration scenarios
- Multi-primal coordination
- Federation edge cases
- Real-world workflows

---

## 📊 PROGRESS TRACKING

### Completion Status

- [x] Comprehensive audit complete
- [x] Sovereignty/dignity verified (A+) 🏆
- [ ] P0: Fix syntax errors (30 min)
- [ ] P1: Measure coverage (15 min)
- [ ] P1: Split oversized file (1 hour)
- [ ] P1: Fix deprecations (2-4 hours)
- [ ] P1: Audit unwraps (8-12 hours)
- [ ] P2: Complete TODOs (30-40 hours)
- [ ] P2: Migrate hardcoding (80-120 hours)
- [ ] P2: Expand tests (40-80 hours)
- [ ] P3: Optimize clones (40+ hours)
- [ ] P3: Pedantic clippy (16-24 hours)
- [ ] P3: Expand E2E (40+ hours)

### Time Estimates

| Priority | Items | Time Range |
|----------|-------|------------|
| P0 | 1 | 30 minutes |
| P1 | 4 | 11-17 hours |
| P2 | 3 | 150-240 hours |
| P3 | 3 | 96+ hours |
| **Total** | **11** | **~257-358 hours** |

**Sprint 1 (Week 1)**: P0 + P1 = 11-17.5 hours  
**Sprint 2-4 (Weeks 2-4)**: P2 = 150-240 hours  
**Future Sprints**: P3 = 96+ hours (as time permits)

---

## 🎯 RECOMMENDED SCHEDULE

### Day 1 (Today)
- [x] Complete comprehensive audit
- [ ] Fix 3 syntax errors (30 min)
- [ ] Measure coverage (15 min)
- [ ] Document baseline metrics

### Week 1
- [ ] Split oversized test file (1 hour)
- [ ] Fix deprecation warnings (2-4 hours)
- [ ] Audit production unwraps (8-12 hours)

### Week 2-4
- [ ] Complete high-priority TODOs (30-40 hours)
- [ ] Begin hardcoding migration (80-120 hours)
- [ ] Expand test coverage (40-80 hours)

### Month 2-3 (As Time Permits)
- [ ] Optimize clone usage
- [ ] Pedantic clippy compliance
- [ ] Expand E2E tests

---

## 📋 VERIFICATION CHECKLIST

After each priority level:

### After P0
```bash
✓ cargo build --lib passes
✓ cargo test --lib passes (100%)
✓ All syntax errors fixed
```

### After P1
```bash
✓ Coverage measured (≥60% acceptable, ≥90% target)
✓ No files >1000 lines
✓ No deprecation warnings
✓ Production unwraps documented/fixed
```

### After P2
```bash
✓ All high-priority TODOs complete
✓ Hardcoding <500 instances
✓ Coverage ≥90%
✓ Build time <3 minutes
```

### After P3
```bash
✓ Clone count <500
✓ Pedantic clippy clean
✓ E2E coverage complete
✓ Performance benchmarks documented
```

---

## 🚀 QUICK COMMANDS

### Build & Test
```bash
# Full build
cargo build --lib

# Run tests
cargo test --lib

# Check format
cargo fmt --check

# Run clippy
cargo clippy --all-targets
```

### Coverage
```bash
# HTML report
cargo llvm-cov --html

# Text summary
cargo llvm-cov

# With tests
cargo llvm-cov --all-targets
```

### Quality Checks
```bash
# Deprecation warnings
cargo build 2>&1 | grep "warning: use of deprecated"

# File sizes
find crates -name "*.rs" -exec wc -l {} + | sort -rn | head -20

# Unwraps
./check_production_unwraps.sh

# Hardcoding
grep -r "8080\|8081\|127.0.0.1\|localhost" crates/*/src --count
```

---

## 📚 REFERENCE DOCUMENTS

- **Full Audit**: `COMPREHENSIVE_AUDIT_REPORT_DEC_8_2025.md` (detailed findings)
- **Executive Summary**: `AUDIT_EXECUTIVE_SUMMARY_DEC_8_2025.md` (one-page overview)
- **This Document**: Quick action items (you are here)

---

## 🎉 REMEMBER

1. **Fix P0 first** - Everything is blocked by syntax errors
2. **Measure before optimizing** - Get real coverage numbers
3. **Deploy early** - Don't wait for perfection
4. **Iterate based on data** - Let metrics guide priorities
5. **Maintain sovereignty** - Don't compromise ethics for speed

---

**Created**: December 8, 2025  
**Next Review**: After P0 completion  
**Status**: Ready for action ⚡

