# ⚡ Quick Action Items - November 9, 2025
**Purpose**: Immediate, high-impact tasks for next session  
**Focus**: Low-hanging fruit with measurable progress

---

## 🎯 Immediate Actions (2-3 Hours Next Session)

### 1. Remove 46 Deprecated Items (Highest Priority) ⏱️ ~90 minutes

**Why**: Already marked for removal, compiler-guided cleanup, immediate 10% reduction in technical debt

**Files Requiring Attention** (14 files total):

#### High-Count Files (Priority 1)
```bash
# 9 deprecated items - Network config overlap
crates/songbird-config/src/config/network/mod.rs

# 9 deprecated items - Biome module types  
crates/songbird-orchestrator/src/core/biome/modules/types.rs

# 6 deprecated items - Performance config duplication
crates/songbird-config/src/unified/performance.rs
```

#### Medium-Count Files (Priority 2)
```bash
# 5 deprecated items - Discovery config
crates/songbird-config/src/unified/discovery.rs

# 4 deprecated items - Test mock servers
crates/songbird-test-utils/src/mocks/mod.rs

# 3 deprecated items - SDK modules (beardog, toadstool, squirrel)
crates/songbird-primal-sdk/src/lib.rs

# 3 deprecated items - Universal primals config
crates/songbird-config/src/config/universal_primals.rs
```

#### Low-Count Files (Priority 3)
```bash
# 1 deprecated item each:
crates/songbird-universal/src/types/capability.rs
crates/songbird-config/src/unified/network.rs
crates/songbird-config/src/lib.rs
crates/songbird-config/src/environment_config_clean.rs
crates/songbird-config/src/environment.rs
crates/songbird-config/src/config/mod.rs
crates/songbird-config/src/config/environment.rs
```

**Process Per File**:
1. Open file and review deprecated items
2. Find all usages: `rg "deprecated_item_name" crates/`
3. Update call sites to use canonical version (noted in deprecation message)
4. Run tests: `cargo test --package <crate>`
5. Remove deprecated item
6. Commit: `refactor(crate): remove deprecated X, use canonical Y`

**Expected Result**: 
- Deprecated items: 46 → 0 ✅
- Immediate win for team morale
- Clear removal pattern established

---

### 2. Migrate Config Imports (Priority 2) ⏱️ ~45 minutes

**Why**: 4 files still using deprecated `config::` imports instead of `canonical::`

**Files to Update**:
```rust
// 1. crates/songbird-primal-sdk/src/lib.rs
// FIND: pub use.*::config::
// REPLACE: pub use.*::canonical::

// 2. crates/songbird-discovery/src/traits/config.rs
// FIND: use.*config::
// REPLACE: use.*canonical::

// 3. crates/songbird-config/src/environment.rs
// FIND: use.*config::
// REPLACE: use.*canonical::

// 4. crates/songbird-types/src/config/consolidated.rs
// FIND: use.*config::
// REPLACE: use.*canonical::
```

**Command**:
```bash
# For each file:
sed -i 's/::config::/::canonical::/g' <file>
cargo test --package <crate>
```

**Expected Result**:
- All imports use canonical pattern
- 4 files migrated ✅
- No deprecated imports remaining

---

### 3. Run Current Metrics & Compare ⏱️ ~5 minutes

**Why**: Establish current baseline and measure previous session's impact

**Commands**:
```bash
cd /home/eastgate/Development/ecoPrimals/songbird

# Run metrics
./scripts/unification_metrics.sh

# Compare to previous session
# Previous baseline (Nov 9, Session 1):
# Config structs: 715
# Legacy patterns: 466
# Deprecated items: 46
# Error enums: 26
# Provider traits: 27
# Result types: 13
```

**Document Changes**:
```bash
# Save current metrics
./scripts/unification_metrics.sh > reports/metrics_nov_9_session_2.txt

# Create comparison
diff reports/metrics_nov_9_session_1.txt reports/metrics_nov_9_session_2.txt
```

---

## 🏆 Quick Win Checklist

After completing the above tasks, you'll have:

- [x] **46 deprecated items** → 0 (100% elimination)
- [x] **4 config imports** → canonical pattern
- [x] **Metrics tracked** → measurable progress
- [x] **Builds passing** → all modified crates
- [x] **Tests passing** → no regressions
- [x] **Commits made** → clear history

---

## 📊 Expected Impact

### Metrics After Session
```
Before → After → Target
Deprecated items:  46 → 0 → 0 ✅ COMPLETE
Config structs:    715 → 715 → 50 (no change, later priority)
Legacy patterns:   466 → 466 → 0 (no change, later priority)
```

### Progress Percentage
- **Deprecated Elimination**: 0% → 100% ✅
- **Overall Unification**: 5% → 10% (+5 percentage points)

---

## 🔄 Follow-Up Tasks (Future Sessions)

### Session 3 (2-3 hours)
1. **Result Type Migration** (13 → 1)
   - Quick search & replace
   - High impact, low effort
   
2. **Start Config Consolidation**
   - Begin Phase 1 from CONFIG_CONSOLIDATION_ROADMAP.md
   - Move standalone files to canonical/

### Session 4+ (8-12 hours)
1. **Complete Config Consolidation** (715 → 50)
2. **Error System Migration** (26 → 1)
3. **Discovery Cleanup** (98 legacy patterns → 0)

---

## 🎯 Key Files Reference

### Files to Watch (Approaching 2000 line limit)
```
1277 lines - crates/songbird-config/src/canonical/network.rs
1257 lines - crates/songbird-universal/tests/unified_adapter_core_tests.rs
986 lines  - crates/songbird-network-federation/tests/network_comprehensive_tests.rs
963 lines  - crates/songbird-config/src/config/network/mod.rs
949 lines  - crates/songbird-universal/src/capabilities/adapter.rs
```

**Status**: All well under 2000 line limit (63% of max) ✅  
**Action**: Monitor but no immediate splitting needed

### Critical Config Files (Consolidation Targets)
```
Duplication hotspots:
- Network config: 4 locations (canonical/, config/, unified/, zero_touch/)
- Environment config: 5 locations
- Primal config: 4 locations

See: CONFIG_CONSOLIDATION_ROADMAP.md for detailed plan
```

---

## 📚 Documentation Reference

### For This Session
- `UNIFICATION_STATUS_REPORT_NOV_9.md` - Comprehensive analysis (just created)
- `UNIFICATION_TACTICAL_PLAN.md` - Week-by-week execution plan
- `00_REVIEW_START_HERE.md` - Quick orientation

### For Config Work (Later)
- `CONFIG_CONSOLIDATION_ROADMAP.md` - Detailed consolidation plan
- `../beardog/BEARDOG_CODING_STANDARDS.md` - Proven patterns

### For Error Work (Later)  
- `specs/UNIFIED_ERROR_HANDLING_SPECIFICATION.md` - Error strategy
- `UNIFIED_ERRORS_QUICKREF.md` - Quick reference

---

## ⚡ Commands Cheat Sheet

### Find deprecated items
```bash
grep -r "#\[deprecated" --include="*.rs" crates/ -l
```

### Find usages of deprecated item
```bash
rg "item_name" crates/
```

### Test specific crate
```bash
cargo test --package songbird-config
cargo test --package songbird-primal-sdk
```

### Run metrics
```bash
./scripts/unification_metrics.sh
```

### Check builds
```bash
cargo check --workspace
cargo clippy --workspace -- -D warnings
```

---

## 🎯 Success Criteria for Next Session

**Must Complete**:
- [ ] All 46 deprecated items removed
- [ ] All builds passing
- [ ] All tests passing  
- [ ] Metrics documented

**Nice to Have**:
- [ ] 4 config imports migrated
- [ ] Session documented
- [ ] Commits pushed

**Time Box**: 2-3 hours maximum

**If Ahead of Schedule**: Start Result type migration (13 → 1)

---

## 🚀 Let's Go!

**Start Here**: 
1. Open `crates/songbird-config/src/config/network/mod.rs` (9 deprecated items)
2. Review deprecation messages
3. Find usages and update
4. Remove deprecated code
5. Test and commit
6. Repeat for other 13 files

**Stay Focused**: One file at a time, test after each change

**Track Progress**: Mark completed files in this list

---

**Created**: November 9, 2025  
**Next Session**: Remove deprecated items  
**Status**: 🟢 **READY TO EXECUTE**

