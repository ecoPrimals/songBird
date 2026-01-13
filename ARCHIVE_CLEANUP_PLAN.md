# 🗂️ Archive & Cleanup Plan - January 12, 2026

**Purpose**: Clean up outdated code, docs, and TODOs while preserving fossil record  
**Status**: Ready for execution  
**Safe to Push**: ✅ Yes (after review)

---

## 📊 **ANALYSIS SUMMARY**

### Current State:
- ✅ Archive directory exists: `docs/archive/`
- ✅ Well-organized by version (v3.12 through v3.22)
- ✅ 76 TODOs found across 40 files
- ✅ Most TODOs are legitimate (future work)
- ⚠️ 2 outdated TODOs found
- ⚠️ 5 duplicate session summary docs at root

---

## 🗑️ **ITEMS TO ARCHIVE**

### 1. Duplicate Session Summaries (Root → Archive)

**Move to `docs/archive/v3.22/`**:
- `SESSION_SUMMARY_JAN_12_2026.md` (iteration 1)
- `FINAL_SESSION_SUMMARY_JAN_12_2026.md` (iteration 2)
- `SESSION_COMPLETE_JAN_12_2026.md` (iteration 3)
- `DAY1_SUMMARY_FINAL.md` (iteration 4)
- `DAY1_COMPLETE_JAN_12_2026.md` (iteration 5)

**Keep at Root** (canonical):
- `DAY1_FINAL_STATUS.md` ⭐ (authoritative)

**Reason**: These are earlier iterations of the same summary. Keep for fossil record but move to archive.

---

### 2. Outdated TODOs to Update

#### TODO 1: `crates/songbird-orchestrator/src/ipc/handlers.rs:513`
```rust
// TODO v3.19.3: Implement broadcaster.update_capabilities() method
```

**Status**: This is in the OLD handlers.rs file (1,229 lines) that we're refactoring  
**Action**: Will be removed when we complete handlers refactoring (in progress)  
**Priority**: ⏳ Handled by refactoring

#### TODO 2: `crates/songbird-orchestrator/src/connections/limited_btsp.rs:185`
```rust
// For now, we'll document this as a TODO for Phase 2
```

**Status**: Legitimate Phase 2 work  
**Action**: Keep as-is (properly scoped)  
**Priority**: ✅ No action needed

---

### 3. Outdated Documentation Files

**Already Archived** ✅:
- Old session docs (v3.12 - v3.21) in `docs/archive/`
- Version-specific evolution docs properly organized

**No Action Needed**: Archive structure is excellent!

---

## ✅ **RECOMMENDED ACTIONS**

### Phase 1: Archive Duplicate Summaries (Safe)

```bash
# Create v3.22 archive directory
mkdir -p docs/archive/v3.22/jan-12-2026-session

# Move duplicate session summaries
mv SESSION_SUMMARY_JAN_12_2026.md docs/archive/v3.22/jan-12-2026-session/
mv FINAL_SESSION_SUMMARY_JAN_12_2026.md docs/archive/v3.22/jan-12-2026-session/
mv SESSION_COMPLETE_JAN_12_2026.md docs/archive/v3.22/jan-12-2026-session/
mv DAY1_SUMMARY_FINAL.md docs/archive/v3.22/jan-12-2026-session/
mv DAY1_COMPLETE_JAN_12_2026.md docs/archive/v3.22/jan-12-2026-session/

# Keep at root:
# - DAY1_FINAL_STATUS.md (canonical)
# - COMPREHENSIVE_CODE_REVIEW_JAN_2026.md (active)
# - DEEP_DEBT_EVOLUTION_PLAN.md (active)
# - NEXT_SESSION.md (active)
# - All refactoring docs (active)
```

### Phase 2: Update Archive Index

Create `docs/archive/v3.22/README.md` documenting what's archived.

### Phase 3: Git Commit & Push

```bash
# Stage changes
git add docs/archive/v3.22/
git add SESSION_SUMMARY_JAN_12_2026.md  # (deleted from root)
git add FINAL_SESSION_SUMMARY_JAN_12_2026.md  # (deleted)
git add SESSION_COMPLETE_JAN_12_2026.md  # (deleted)
git add DAY1_SUMMARY_FINAL.md  # (deleted)
git add DAY1_COMPLETE_JAN_12_2026.md  # (deleted)

# Commit with clear message
git commit -m "Archive: Move duplicate Jan 12 session summaries to v3.22 archive

- Moved 5 duplicate session summary iterations to docs/archive/v3.22/
- Kept DAY1_FINAL_STATUS.md as canonical summary
- Preserves fossil record while cleaning root directory
- Part of documentation cleanup (see DOCS_CLEANUP_COMPLETE.md)"

# Push via SSH
git push origin main
```

---

## 📋 **DETAILED ANALYSIS**

### TODOs by Category:

**Legitimate Future Work** (74 TODOs):
- Phase 2 features (bluetooth, federation)
- E2E test expansion
- Performance optimizations
- Feature enhancements

**Outdated/In-Progress** (2 TODOs):
- `handlers.rs:513` - Being removed via refactoring ✅
- `limited_btsp.rs:185` - Properly scoped to Phase 2 ✅

**Action Required**: ✅ None (handled by ongoing work)

---

### Archive Directory Structure (Excellent!):

```
docs/archive/
├── v3.12/ (16 docs) - Anonymous discovery refactoring
├── v3.13/ (7 docs) - Deep debt evolution
├── v3.14/ (19 docs) - Testing evolution
├── v3.15/ (18 docs) - More evolution
├── v3.16/ (2 docs) - Tarpc evolution
├── v3.17/ (1 doc) - biomeOS handoff
├── v3.18/ (4 docs) - BTSP connection
├── v3.19/ (8 docs) - Modern Rust evolution
├── v3.20/ (6 docs) - Service registry
├── v3.21/ (11 docs) - Collaborative Intelligence
├── v3.22/ (3 docs) - Pure Rust evolution
└── [Proposed] v3.22/jan-12-2026-session/ (5 duplicate summaries)
```

**Quality**: ✅ Excellent organization, clear versioning

---

## 🚫 **DO NOT ARCHIVE**

### Active Documentation (Keep at Root):

**Current Status**:
- `DAY1_FINAL_STATUS.md` ⭐
- `STATUS.md`
- `00_START_HERE.md`
- `ROOT_DOCS_INDEX.md`

**Active Evolution**:
- `COMPREHENSIVE_CODE_REVIEW_JAN_2026.md`
- `DEEP_DEBT_EVOLUTION_PLAN.md`
- `EVOLUTION_PROGRESS_JAN_12_2026.md`
- `NEXT_SESSION.md`

**Active Refactoring**:
- `REFACTORING_1_COMPLETE_anonymous_discovery.md`
- `IPC_HANDLERS_REFACTORING_PLAN.md`
- `HANDLERS_REFACTORING_STATUS.md`
- `MOCK_TO_REAL_EVOLUTION_COMPLETE.md`

**Production**:
- `BIOMEOS_HANDOFF_V3_22_0.md`
- `PURE_RUST_V3_22_0_FINAL.md`
- `README.md`
- `CONTRIBUTING.md`
- `ROADMAP.md`
- `CHANGELOG.md`

**Reason**: All actively referenced or in use

---

## ✅ **FALSE POSITIVES**

### Not Actually Problems:

1. **Experimental Code** (songbird-bluetooth)
   - ✅ Properly marked as experimental
   - ✅ Has README explaining status
   - ✅ Isolated with `#[cfg(feature = "bluetooth")]`
   - **Action**: None needed

2. **Archive Code** (rendezvous/)
   - ✅ Intentionally kept for reference
   - ✅ Not compiled by default
   - ✅ Documented as archive
   - **Action**: None needed

3. **Demos & Examples**
   - ✅ Intentionally diverse (showing evolution)
   - ✅ Useful for showcasing capabilities
   - ✅ Not production code
   - **Action**: None needed

---

## 📊 **METRICS**

| Category | Count | Action |
|----------|-------|--------|
| **Duplicate Summaries** | 5 | Archive to v3.22/ |
| **Outdated TODOs** | 0 | None (2 are valid) |
| **Archive Docs** | 100+ | Well organized ✅ |
| **Active Docs** | 20+ | Keep at root ✅ |
| **False Positives** | 3 areas | Intentional ✅ |

---

## 🎯 **EXECUTION PLAN**

### Step 1: Create Archive Directory ✅
```bash
mkdir -p docs/archive/v3.22/jan-12-2026-session
```

### Step 2: Move Duplicate Summaries ✅
```bash
mv SESSION_SUMMARY_JAN_12_2026.md docs/archive/v3.22/jan-12-2026-session/
mv FINAL_SESSION_SUMMARY_JAN_12_2026.md docs/archive/v3.22/jan-12-2026-session/
mv SESSION_COMPLETE_JAN_12_2026.md docs/archive/v3.22/jan-12-2026-session/
mv DAY1_SUMMARY_FINAL.md docs/archive/v3.22/jan-12-2026-session/
mv DAY1_COMPLETE_JAN_12_2026.md docs/archive/v3.22/jan-12-2026-session/
```

### Step 3: Create Archive Index ✅
```bash
# Create README in archive directory
cat > docs/archive/v3.22/jan-12-2026-session/README.md << 'EOF'
# Jan 12, 2026 Session Summaries (Archived)

These are iteration summaries from the Day 1 evolution session.

**Canonical Summary**: See `/DAY1_FINAL_STATUS.md` at project root.

## Files:
1. SESSION_SUMMARY_JAN_12_2026.md - Initial summary
2. FINAL_SESSION_SUMMARY_JAN_12_2026.md - Comprehensive summary
3. SESSION_COMPLETE_JAN_12_2026.md - Verification summary
4. DAY1_SUMMARY_FINAL.md - Final summary
5. DAY1_COMPLETE_JAN_12_2026.md - Completion doc

All superseded by DAY1_FINAL_STATUS.md.
EOF
```

### Step 4: Git Commit & Push ✅
```bash
git add docs/archive/v3.22/
git add -u  # Stage deletions
git commit -m "Archive: Move duplicate Jan 12 session summaries to v3.22 archive"
git push origin main
```

---

## ✅ **VERIFICATION**

After execution:

```bash
# Verify archive created
ls -la docs/archive/v3.22/jan-12-2026-session/

# Verify root cleaned
ls -1 *.md | grep -E "SESSION|DAY1"
# Should only show: DAY1_FINAL_STATUS.md

# Verify git status
git status
```

---

## 🎉 **BENEFITS**

### After Cleanup:
- ✅ Root directory cleaner (5 fewer duplicate docs)
- ✅ Fossil record preserved in archive
- ✅ Clear canonical document (DAY1_FINAL_STATUS.md)
- ✅ Archive well-organized by version
- ✅ Git history preserved
- ✅ Easy to find both current and historical docs

---

## 📝 **NOTES**

### Why Keep Duplicates in Archive?
- **Fossil Record**: Shows evolution of thinking
- **Historical Context**: Useful for understanding decisions
- **Completeness**: Nothing lost, just organized

### Why Not Delete?
- **Git Philosophy**: Commits are cheap, deletions are forever
- **Future Reference**: Might need to reference specific iteration
- **Transparency**: Shows complete work history

---

**Status**: ✅ Ready to execute  
**Risk**: ✅ Low (just moving files)  
**Reversible**: ✅ Yes (git revert)  
**Safe to Push**: ✅ Yes

---

**Created**: January 12, 2026 - 9:15 PM  
**Next**: Execute cleanup and push via SSH

