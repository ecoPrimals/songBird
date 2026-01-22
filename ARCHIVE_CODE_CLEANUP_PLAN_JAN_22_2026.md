# Archive Code Cleanup Plan - January 22, 2026

**Date**: January 22, 2026  
**Purpose**: Review and clean archive code while preserving documentation  
**Policy**: Keep docs as fossil record, remove outdated code

---

## 🎯 Cleanup Strategy

**Principles**:
1. ✅ **Documentation = Fossil Record** - Keep all .md files
2. ✅ **Code = Can Be Cleaned** - Remove outdated .rs, .sh, benchmarks
3. ✅ **Scripts = Review** - Keep useful, remove obsolete
4. ✅ **Tests = Modernize** - Update to use SongbirdHttpClient where appropriate
5. ✅ **Benchmarks = Archive** - Move outdated benchmarks to archive/

---

## 📊 Findings

### 1. Orphaned Files

**README_CLEAN.md**:
- Status: Leftover from documentation cleanup
- Action: **DELETE** ❌
- Reason: Temporary file, no longer needed

**Empty Log Files**:
- File: `showcase/06-toadstool-ml-orchestration/logs/eastgate.log`
- Status: Empty log file
- Action: **DELETE** ❌
- Reason: Empty, no content

### 2. Outdated Benchmarks

**benches/tarpc_performance_benchmarks.rs**:
- Issue: Uses `reqwest` (eliminated dependency)
- Lines: 355 lines
- Last Updated: November 11, 2025
- Action: **MOVE TO ARCHIVE** 📦
- Destination: `archive/benchmarks/tarpc_performance_benchmarks.rs`
- Reason: Uses eliminated dependency, outdated

**Analysis of Other Benchmarks**:
- 11 benchmark files in `benches/` directory
- Total size: ~170KB of benchmark code
- Many may reference old patterns (reqwest, ring, openssl)
- Recommendation: Review all benchmarks for outdated dependencies

### 3. Scripts with Outdated References

**Scripts mentioning reqwest/ring/openssl** (10 files):
```
scripts/activate_comprehensive_testing.sh
scripts/btsp-health-monitor.sh
scripts/deploy-ecosystem.sh
scripts/deploy_foundation.sh
scripts/eliminate_all_hardcoding.sh
scripts/implement_production_readiness.sh
scripts/migrate_config_domain.sh
scripts/migrate_primal_references.sh
scripts/pedantic_perfection_analyzer.sh
scripts/performance_benchmarking_pipeline.sh
```

**Action**: **REVIEW INDIVIDUALLY** 🔍
- Check if mentions are in documentation comments (keep)
- Check if used for actual dependency management (update)
- Check if scripts are still used (archive if obsolete)

### 4. Test Files with reqwest

**tests/e2e/test_environment.rs**:
- Uses: `reqwest::Client` for health checks
- Lines: 7 usages
- Status: **KEEP FOR NOW** ⏸️
- Reason: E2E tests intentionally use external HTTP client to test server
- Note: This is acceptable - tests should use external clients

**Recommendation**: Document this as intentional external testing

### 5. TODO/FIXME Comments

**Count**: 90 total
- Checked for outdated reqwest/ring/openssl TODOs: **0 found** ✅
- Status: **NO ACTION NEEDED** ✅
- Reason: No outdated TODOs found, all are legitimate future work

---

## 🎯 Recommended Actions

### Phase 1: Immediate Cleanup (Safe)

**1. Remove Orphaned Files**:
```bash
rm README_CLEAN.md
rm showcase/06-toadstool-ml-orchestration/logs/eastgate.log
```

**Impact**: Low risk, obvious cleanup

### Phase 2: Benchmark Review and Archive

**2. Move Outdated Benchmark to Archive**:
```bash
mkdir -p archive/benchmarks
git mv benches/tarpc_performance_benchmarks.rs archive/benchmarks/
```

**3. Review All Benchmarks**:
```bash
# Check each benchmark for reqwest/ring/openssl
for f in benches/*.rs; do
    echo "=== $f ==="
    grep -n "reqwest\|use ring\|openssl" "$f" || echo "Clean"
done
```

**Action**: Archive any benchmarks using eliminated dependencies

### Phase 3: Script Review

**4. Review Scripts Individually**:
- Check if mentions are in comments (documentation)
- Check if actually used for dependency management
- Archive obsolete scripts
- Update scripts that reference old patterns

**Detailed Review Needed**:
- `scripts/eliminate_all_hardcoding.sh` - May be obsolete (hardcoding eliminated)
- `scripts/migrate_primal_references.sh` - May be obsolete (migration complete)
- `scripts/migrate_config_domain.sh` - May be obsolete (migration complete)

### Phase 4: Documentation

**5. Document Intentional External Dependencies in Tests**:
- Add comment to `tests/e2e/test_environment.rs` explaining reqwest use
- Update test documentation to clarify external vs internal testing

---

## 📦 Archive Organization

### Proposed Structure

```
archive/
├── benchmarks/              (NEW - archived benchmarks)
│   ├── tarpc_performance_benchmarks.rs
│   └── [other outdated benchmarks]
├── scripts/                 (NEW - archived scripts)
│   ├── eliminate_all_hardcoding.sh (if obsolete)
│   ├── migrate_primal_references.sh (if obsolete)
│   └── [other obsolete scripts]
├── demo-prototypes/         (EXISTING - demo binaries)
├── historical-snapshots/    (EXISTING - session docs)
└── jan-2026-sessions/       (EXISTING - session docs)
```

---

## ✅ Cleanup Checklist

### Immediate (Phase 1)
- [ ] Delete `README_CLEAN.md`
- [ ] Delete empty log file

### Benchmarks (Phase 2)
- [ ] Create `archive/benchmarks/` directory
- [ ] Move `tarpc_performance_benchmarks.rs` to archive
- [ ] Review remaining 10 benchmarks for outdated deps
- [ ] Move any additional outdated benchmarks

### Scripts (Phase 3)
- [ ] Review `scripts/eliminate_all_hardcoding.sh`
- [ ] Review `scripts/migrate_primal_references.sh`
- [ ] Review `scripts/migrate_config_domain.sh`
- [ ] Review remaining 7 scripts
- [ ] Archive obsolete scripts
- [ ] Update scripts with outdated references

### Documentation (Phase 4)
- [ ] Add comment to `test_environment.rs` explaining reqwest
- [ ] Update test documentation
- [ ] Create archive cleanup summary document

### Final (Phase 5)
- [ ] Run `cargo test` to verify no breakage
- [ ] Run `cargo build --release` to verify compilation
- [ ] Git commit with cleanup summary
- [ ] Git push

---

## 🔍 Detailed Benchmark Review

### Files to Check

1. `benches/comprehensive_performance_benchmarks.rs` (25K)
2. `benches/comprehensive_unification_benchmarks.rs` (1.1K)
3. `benches/critical_path_benchmarks.rs` (6.4K)
4. `benches/fractal_federation_performance.rs` (14K)
5. `benches/hot_path_benchmarks.rs` (7.7K)
6. `benches/phase3_performance_benchmarks.rs` (13K)
7. `benches/tarpc_performance_benchmarks.rs` (11K) - **CONFIRMED OUTDATED**
8. `benches/ultra_pedantic_benchmarks.rs` (7.5K)
9. `benches/ultra_pedantic_performance.rs` (7.7K)
10. `benches/unified_types_benchmarks.rs` (3.1K)
11. `benches/zero_cost_abstractions_benchmark.rs` (11K)

**Subdirectory Benchmarks** (benches/performance_benchmarks/):
12. `concurrency_performance.rs` (27K)
13. `memory_patterns.rs` (17K)
14. `real_world_scenarios.rs` (18K)
15. `simple_performance.rs` (5.9K)
16. `zero_cost_optimizations.rs` (15K)
17. `zero_cost_service_benchmark.rs` (7.5K)

**Total**: 17 benchmark files, ~170KB

---

## 🔍 Script Review Guide

### Questions to Ask

For each script:
1. **Is it still used?** Check git history, last modified date
2. **Does it reference eliminated dependencies?** Check for reqwest/ring/openssl
3. **Is its purpose complete?** (e.g., migration scripts after migration done)
4. **Can it be updated?** Or should it be archived?

### Review Template

```markdown
Script: [name]
Purpose: [what it does]
Last Modified: [date]
References: [reqwest/ring/openssl?]
Status: [active/obsolete/update]
Action: [keep/archive/update/delete]
Reason: [explanation]
```

---

## 📊 Impact Assessment

### Low Risk Cleanup

- Delete `README_CLEAN.md`: **Safe** ✅
- Delete empty log: **Safe** ✅
- Move benchmark to archive: **Safe** ✅ (benchmarks don't affect runtime)

### Medium Risk Cleanup

- Archive obsolete scripts: **Review Required** ⚠️
  - Need to verify not used in CI/CD
  - Need to verify not used by team
  - Document what each script did

### High Risk Cleanup

- None identified ✅

---

## 🎯 Expected Results

### After Cleanup

**Removed**:
- 2 orphaned files
- 1+ outdated benchmark(s)
- 0-3 obsolete scripts

**Organized**:
- Benchmarks properly archived
- Scripts reviewed and documented
- Clear archive structure

**Preserved**:
- All documentation (.md files)
- All active code
- All legitimate TODOs

**Quality**:
- Cleaner repository
- No outdated dependencies referenced
- Better organization

---

## 📝 Documentation Policy

### What to Keep (Fossil Record)

✅ **All .md files** - Historical context  
✅ **Session summaries** - Evolution tracking  
✅ **Architecture docs** - Design decisions  
✅ **Status reports** - Progress tracking  
✅ **Cleanup docs** - Maintenance history

### What Can Be Cleaned

❌ **Outdated benchmarks** - Move to archive  
❌ **Obsolete scripts** - Move to archive  
❌ **Temporary files** - Delete  
❌ **Empty files** - Delete  
❌ **Duplicate files** - Delete

### What to Review

🔍 **Scripts** - Active vs obsolete  
🔍 **Tests** - External vs internal  
🔍 **Benchmarks** - Dependencies used  
🔍 **TODOs** - Outdated vs legitimate

---

## 🚀 Next Steps

### 1. Execute Phase 1 (Immediate)
```bash
cd /home/eastgate/Development/ecoPrimals/phase1/songbird
rm README_CLEAN.md
rm showcase/06-toadstool-ml-orchestration/logs/eastgate.log
git add -A
git commit -m "chore: Remove orphaned files"
```

### 2. Execute Phase 2 (Benchmarks)
```bash
# Create archive directory
mkdir -p archive/benchmarks

# Move outdated benchmark
git mv benches/tarpc_performance_benchmarks.rs archive/benchmarks/

# Review other benchmarks
for f in benches/*.rs benches/performance_benchmarks/*.rs; do
    echo "=== $f ==="
    grep -c "reqwest\|use ring\|openssl" "$f" 2>/dev/null || echo "0"
done

# Move any additional outdated benchmarks found
git add -A
git commit -m "chore: Archive outdated benchmarks"
```

### 3. Execute Phase 3 (Scripts)
```bash
# Review each script individually
# Document findings
# Archive or update as appropriate
```

### 4. Document and Push
```bash
# Create final summary
# Commit all changes
git push origin main
```

---

## 📊 Summary

**Found**:
- 2 orphaned files (README_CLEAN.md, empty log)
- 1+ outdated benchmark(s) with reqwest
- 10 scripts to review for outdated references
- 90 TODO/FIXME comments (all legitimate)
- 1 test file with reqwest (intentional for e2e testing)

**Action Plan**:
- Phase 1: Remove 2 orphaned files
- Phase 2: Archive 1+ benchmarks
- Phase 3: Review 10 scripts
- Phase 4: Document test patterns
- Phase 5: Commit and push

**Policy**:
- ✅ Keep all documentation as fossil record
- ✅ Clean outdated code
- ✅ Organize archives properly
- ✅ Preserve legitimate TODOs

**Status**: Ready for execution  
**Risk**: Low (safe cleanup)  
**Impact**: Better organization, no outdated deps referenced

---

**Ready to proceed with cleanup!** 🧹

