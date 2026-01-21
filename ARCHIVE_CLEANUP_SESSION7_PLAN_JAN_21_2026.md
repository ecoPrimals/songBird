# Archive Code Cleanup Plan - Session 7.5
**Date**: January 21, 2026  
**Status**: Review Phase  
**Principle**: Keep docs as fossil record, clean deprecated code

---

## 📊 Cleanup Candidates Identified

### 1. Deprecated Code Files with reqwest (10 files - NOT COMPILED)

**Status**: These files have reqwest references but DO NOT compile or affect production

```
❌ crates/songbird-orchestrator/src/core/primal_integration.rs
   - Severely corrupted (syntax errors throughout)
   - Only referenced by deprecated biomeos/integration.rs
   - NOT imported in lib.rs module tree
   - Size: ~342 lines
   - Status: SAFE TO DELETE

❌ crates/songbird-orchestrator/src/core/biome/modules/orchestrator.rs
   - Deprecated biomeOS integration (superseded by IPC)
   - NOT used in production
   - Size: Unknown
   - Status: CANDIDATE FOR DELETION

❌ crates/songbird-orchestrator/src/core/biome/modules/lifecycle.rs
   - Deprecated biomeOS service lifecycle
   - Part of unused biome module
   - Status: CANDIDATE FOR DELETION

❌ crates/songbird-orchestrator/src/core/biomeos/client.rs
   - Legacy biomeOS client (superseded by IPC)
   - Status: CANDIDATE FOR DELETION

❌ crates/songbird-orchestrator/src/core/biomeos/universal_adapter.rs
❌ crates/songbird-orchestrator/src/core/biomeos/universal_adapter_complete.rs
   - Duplicate/experimental adapters
   - NOT used (we have production universal_adapter.rs)
   - Status: CANDIDATE FOR DELETION

❌ crates/songbird-orchestrator/src/core/api/ai_workload_classification/mod.rs
   - Unused AI module (experimental)
   - Status: CANDIDATE FOR DELETION

❌ crates/songbird-orchestrator/src/core/substrate/os_substrate.rs
❌ crates/songbird-orchestrator/src/core/substrate/clients.rs
❌ crates/songbird-orchestrator/src/core/substrate/connection_pool.rs
   - Experimental substrate code
   - NOT used in production
   - Status: CANDIDATE FOR DELETION
```

**Total Lines**: ~1,500-2,000 lines of unused code  
**Impact**: Zero (not compiled, not imported)  
**Risk**: None (production unaffected)

---

### 2. Outdated TODO Comments (31 instances across 18 files)

**Categories**:
- ✅ **COMPLETED**: TODOs for work already done (reqwest, BearDog integration, hardcoding)
- 🔄 **LEGITIMATE**: Real future work (caching, optimization)
- ❓ **UNCLEAR**: Need review

**Files with TODOs** (Sample - need detailed review):
```
crates/songbird-orchestrator/src/universal_adapter.rs:          3 TODOs
crates/songbird-orchestrator/src/http_gateway/universal_proxy.rs: 3 TODOs
crates/songbird-orchestrator/src/ipc/handlers/p2p_discovery.rs:   5 TODOs
crates/songbird-orchestrator/src/main.rs:                         5 TODOs
crates/songbird-orchestrator/src/access_control/auth.rs:          1 TODO
(13 more files...)
```

**Action**: Review each TODO to determine if completed/outdated

---

### 3. Archive Documentation Bloat (130+ files)

**Problem**: Many duplicate "FINAL" documents from iterative sessions

**Cleanup Targets**:

```
📁 archive/jan-2026-sessions/multiple-finals/ (17 files!)
   - 40_PERCENT_MILESTONE_JAN_19_2026.md
   - 80_PERCENT_MILESTONE_JAN_19_2026.md
   - COMPLETE_SESSION_SUMMARY_JAN_19_2026.md
   - COMPREHENSIVE_SESSION_SUMMARY_JAN_19_2026.md
   - CONTINUATION_SESSION_COMPLETE_JAN_19_2026.md
   - EXTENDED_SESSION_FINAL_JAN_19_2026.md
   - FINAL_CODEBASE_STATUS_JAN_19_2026.md
   - FINAL_SESSION_COMPLETE_JAN_19_2026.md
   - FINAL_SESSION_REPORT_JAN_19_2026.md
   - FINAL_SESSION_SUMMARY_JAN_19_2026.md
   - FINAL_STATUS_80_PERCENT_JAN_19_2026.md
   - MARATHON_SESSION_FINAL_JAN_19_2026.md
   - SESSION_COMPLETE_JAN_19_2026.md
   - SESSION_COMPLETE_ULTIMATE_SUMMARY_JAN_19_2026.md
   - ULTIMATE_MARATHON_SESSION_JAN_19_2026.md
   - ULTIMATE_SESSION_SUMMARY_JAN_19_2026.md
   - ULTRA_MARATHON_COMPLETE_JAN_19_2026.md

📁 archive/jan-2026-final-session/ (62 files!)
   - Many incremental status updates
   - Duplicative progress reports
   - Can consolidate to 5-10 key docs

📁 archive/jan-2026-sessions/intermediate-status/ (13 files)
   - Multiple status snapshots
   - Can consolidate to key milestones
```

**Strategy**: 
- Keep 1 comprehensive final summary per major session
- Keep key milestones (40%, 80%, 100%)
- Archive the rest into a single "incremental-progress" folder with README
- **KEEP ALL DOCS** as fossil record (just reorganize)

---

## 🎯 Proposed Cleanup Actions

### Phase 1: Delete Deprecated Code (SAFE - Zero Impact)

**Files to DELETE**:
1. `core/primal_integration.rs` (corrupted, unused)
2. `core/biome/` entire directory (deprecated)
3. `core/biomeos/` entire directory (superseded by IPC)
4. `core/substrate/` entire directory (experimental)
5. `core/api/ai_workload_classification/` directory (unused)

**Verification**:
```bash
# Ensure none are imported
grep -r "primal_integration\|biome::\|biomeos::\|substrate::\|ai_workload" crates/songbird-orchestrator/src/lib.rs
# Expected: No matches (confirms not imported)
```

**Lines Removed**: ~1,500-2,000  
**Build Impact**: None (already not compiled)

---

### Phase 2: Clean Outdated TODOs (CAREFUL - Requires Review)

**Process**:
1. Review each of 31 TODOs
2. Remove if completed (e.g., "TODO: Migrate to BearDog" - DONE!)
3. Update if still valid (e.g., "TODO: Add caching" → "FUTURE: Add caching v5.1+")
4. Document decisions

**Example Completed TODOs** (likely candidates):
- Any mentioning reqwest migration
- Any mentioning BearDog integration
- Any mentioning hardcoding removal
- Any mentioning Universal Adapter evolution

---

### Phase 3: Archive Documentation Consolidation (SAFE - Keep as Fossil Record)

**Actions**:
```
1. Create: archive/jan-2026-sessions/incremental-progress/
   - Move 50+ incremental status docs here
   - Add README explaining they're historical snapshots

2. Keep in main archive:
   - Final comprehensive summaries (1 per session)
   - Key milestones (40%, 80%, 100%)
   - Critical decisions and strategy docs

3. Result:
   - Easier navigation
   - Fossil record preserved
   - Clear progression visible
```

---

## 📋 Execution Checklist

### Step 1: Verify Deprecated Code is Unused
```bash
# Check imports
grep -r "mod primal_integration\|mod biome\|mod biomeos\|mod substrate" crates/songbird-orchestrator/src/

# Check references
rg "use.*primal_integration\|use.*biome::\|use.*biomeos::\|use.*substrate::" crates/songbird-orchestrator/src/

# Expected: Only comments or dead code paths
```

### Step 2: Delete Deprecated Code
```bash
rm crates/songbird-orchestrator/src/core/primal_integration.rs
rm -rf crates/songbird-orchestrator/src/core/biome/
rm -rf crates/songbird-orchestrator/src/core/biomeos/
rm -rf crates/songbird-orchestrator/src/core/substrate/
rm -rf crates/songbird-orchestrator/src/core/api/ai_workload_classification/
```

### Step 3: Verify Build Still Works
```bash
cargo build -p songbird-orchestrator --lib
# Expected: Clean build (no errors)
```

### Step 4: Review and Clean TODOs
```bash
# Generate list of all TODOs with context
rg "TODO|FIXME|XXX|HACK" crates/songbird-orchestrator/src/ -A 2 -B 1 > TODO_REVIEW.txt

# Manual review required - update as needed
```

### Step 5: Reorganize Archive Docs
```bash
# Create incremental progress folder
mkdir -p archive/jan-2026-sessions/incremental-progress/

# Move incremental docs (keep key summaries)
# Manual curation required
```

### Step 6: Commit and Push
```bash
git add -A
git commit -m "🧹 Archive Cleanup - Session 7.5

Removed deprecated code:
- core/primal_integration.rs (corrupted)
- core/biome/ (superseded by IPC)
- core/biomeos/ (superseded by IPC)
- core/substrate/ (experimental)
- core/api/ai_workload_classification/ (unused)

Cleaned outdated TODOs

Reorganized archive docs (fossil record preserved)

Lines removed: ~1,500-2,000
Build: Clean ✅
Impact: Zero (deprecated code not compiled)"

git push origin main
```

---

## 🎯 Expected Outcomes

**Code Cleanup**:
- ✅ 5 directories/10 files deleted (~1,500-2,000 lines)
- ✅ Zero impact on production (not compiled)
- ✅ Cleaner codebase navigation

**TODO Cleanup**:
- ✅ Remove ~10-15 completed TODOs
- ✅ Update ~5-10 to clarify future work
- ✅ Keep ~5-10 legitimate TODOs

**Archive Organization**:
- ✅ Easier to find key docs
- ✅ Fossil record preserved
- ✅ Clear progression visible

**Build Verification**:
- ✅ Clean build
- ✅ All tests pass
- ✅ No regressions

---

## ⚠️ Safety Checks

**Before Deletion**:
1. ✅ Verify files not imported in lib.rs
2. ✅ Search for any production references
3. ✅ Backup via git (can always revert)

**After Deletion**:
1. ✅ cargo build --lib (must succeed)
2. ✅ cargo test --lib (must pass)
3. ✅ No new compiler errors

---

## 📚 Fossil Record Preservation

**Principle**: "We keep docs as fossil record"

**What We Keep**:
- ✅ ALL documentation (just reorganized)
- ✅ All commit history (in git)
- ✅ Key milestones and decisions
- ✅ Evolution progression

**What We Remove**:
- ❌ Deprecated CODE (not docs)
- ❌ Completed TODOs (work is done)
- Nothing else!

---

**Status**: Ready for execution  
**Risk**: Minimal (deprecated code, well-verified)  
**Benefit**: Cleaner codebase, easier navigation  
**Time**: 30-45 minutes

