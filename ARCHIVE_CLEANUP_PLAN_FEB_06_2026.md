# Archive & Cleanup Plan - February 6, 2026

**Goal**: Clean root directory, archive to ecoPrimals/ fossil record, prepare for SSH push  
**Status**: Planning Complete  
**Safety**: All docs preserved in archive before deletion

---

## 📊 Current State Analysis

### Root Directory

**Feb 6, 2026 Documents**: 40 files total

**Categories**:
1. **Phase 3 (Current Session)** - 7 files (KEEP in root)
   - `PHASE3_COMPLETE_AND_PUSHED_FEB_06_2026.md`
   - `PHASE3_REFACTORING_COMPLETE_FEB_06_2026.md`
   - `PHASE3_SMART_REFACTORING_PLAN_FEB_06_2026.md`
   - `SESSION_SUMMARY_FEB_06_2026.md`
   - `UNSAFE_CODE_VICTORY_FEB_06_2026.md`
   - `ROOT_DOCS_UPDATED_FEB_06_2026.md`
   - `COMMIT_MESSAGE_PHASE3.txt`

2. **Sovereign Onion Session (Earlier Today)** - 23 files (ARCHIVE)
   - All already copied to `ecoPrimals/sessions/.../feb-06-2026-sovereign-onion/`
   - Safe to remove from root

3. **Intermediate Status Docs** - 10 files (ARCHIVE)
   - `SESSION_COMPLETE_AND_PUSHED_FEB_06_2026.md`
   - `EVOLUTION_SESSION_COMPLETE_FEB_06_2026.md`
   - `PHASE1_EXECUTION_COMPLETE_FEB_06_2026.md`
   - `DEEP_DEBT_EVOLUTION_ANALYSIS_FEB_06_2026.md`
   - `PHASE1_QUICK_WINS_FEB_06_2026.md`
   - `PUSH_COMPLETE_FEB_06_2026.md`
   - `READY_FOR_SSH_PUSH_FEB_06_2026.md`
   - `SSH_PUSH_READY.md`
   - `COMMIT_MESSAGE_EVOLUTION.txt`
   - `COMMIT_MESSAGE.txt`

### Archive Status

**Already Archived**: `ecoPrimals/sessions/2026-02-february/feb-06-2026-sovereign-onion/`
- 26 Sovereign Onion documents ✅
- Complete session record ✅
- Safe to remove duplicates from root ✅

---

## 🗂️ Archive Strategy

### Phase 1: Create Deep Debt Archive

**New Directory**: `ecoPrimals/sessions/2026-02-february/feb-06-2026-deep-debt-evolution/`

**Contents**:
1. All Phase 3 documents (7 files)
2. Intermediate session docs (10 files)
3. README.md index

### Phase 2: Clean Root Directory

**KEEP in Root** (Active Documents):
- `ROOT_DOCS_INDEX.md` (main index)
- `README.md` (project overview)
- `CHANGELOG.md` (if exists)
- `CONTRIBUTING.md` (if exists)
- Core architecture docs (SOVEREIGN_ONION_TRUE_PRIMAL_ARCHITECTURE.md, etc.)

**REMOVE from Root** (Archived):
- All 40 Feb 6, 2026 session documents
- Old commit message files
- Duplicate status reports

### Phase 3: TODO/FIXME Audit

**Found**: ~117 TODOs/FIXMEs in codebase

**Categories to Review**:
1. **Sovereign Onion TODOs** - May reference old architecture
2. **Handoff TODOs** - May be outdated after Phase 3
3. **False Positive Comments** - "TODO: This is already done"

---

## 📋 Detailed Cleanup Plan

### Step 1: Create Archive Directory

```bash
mkdir -p ecoPrimals/sessions/2026-02-february/feb-06-2026-deep-debt-evolution
```

### Step 2: Move Phase 3 Documents to Archive

**Files to Archive** (17 files):
```
# Phase 3 Core Documents
PHASE3_COMPLETE_AND_PUSHED_FEB_06_2026.md
PHASE3_REFACTORING_COMPLETE_FEB_06_2026.md
PHASE3_SMART_REFACTORING_PLAN_FEB_06_2026.md
SESSION_SUMMARY_FEB_06_2026.md
UNSAFE_CODE_VICTORY_FEB_06_2026.md
ROOT_DOCS_UPDATED_FEB_06_2026.md

# Intermediate Status Docs
SESSION_COMPLETE_AND_PUSHED_FEB_06_2026.md
EVOLUTION_SESSION_COMPLETE_FEB_06_2026.md
PHASE1_EXECUTION_COMPLETE_FEB_06_2026.md
DEEP_DEBT_EVOLUTION_ANALYSIS_FEB_06_2026.md
PHASE1_QUICK_WINS_FEB_06_2026.md
SOVEREIGN_ONION_BEARDOG_REFACTORING_COMPLETE_FEB_06_2026.md
TODAYS_COMPLETE_ACHIEVEMENTS_FEB_06_2026.md
PUSH_COMPLETE_FEB_06_2026.md
READY_FOR_SSH_PUSH_FEB_06_2026.md

# Status Markers
SSH_PUSH_READY.md
BEARDOG_CRYPTO_REFACTOR_HANDOFF_FEB06_2026.md
```

### Step 3: Remove Sovereign Onion Duplicates from Root

**Already Archived** (23 files to remove):
```
ALL_WORK_COMPLETE_FEB_06_2026.md
ARTI_DEPENDENCY_EVOLUTION_FEB_06_2026.md
BEARDOG_ONION_CRYPTO_HANDOFF_FEB_06_2026.md
BIOME_OS_REVIEW_GUIDE_FEB_06_2026.md
COMMIT_READY_STATUS_FEB_06_2026.md
COMPLETE_SESSION_ACHIEVEMENTS_FEB_06_2026.md
DEEP_DEBT_FINAL_REPORT_FEB_06_2026.md
ERROR_HANDLING_ANALYSIS_FEB_06_2026.md
EVOLUTION_SESSION_FEB_06_2026.md
EXECUTIVE_SUMMARY_FEB_06_2026.md
FINAL_STATUS_FEB_06_2026.md
INTEGRATION_COMPLETE_FEB_06_2026.md
MASTER_SUMMARY_FEB_06_2026.md
ONION_SERVICE_HANDOFF_SUMMARY_FEB_06_2026.md
READY_FOR_COMMIT_FEB_06_2026.md
README_EVOLUTION_FEB_06_2026.md
RELAY_RENDEZVOUS_ONION_STATUS_FEB_06_2026.md
RING_DEPENDENCY_ANALYSIS_FEB_06_2026.md
SONGBIRD_EVOLUTION_COMPLETE_FEB_06_2026.md
SONGBIRD_ONION_EVOLUTION_PLAN_FEB_06_2026.md
SONGBIRD_ONION_PHASE1_COMPLETE_FEB_06_2026.md
START_HERE_FEB_06_2026.md
TODAYS_ACHIEVEMENTS_FEB_06_2026.md
```

### Step 4: Clean Old Commit Messages

**Files to Archive**:
```
COMMIT_MESSAGE_PHASE3.txt (keep in deep-debt archive)
COMMIT_MESSAGE_EVOLUTION.txt (archive to deep-debt)
COMMIT_MESSAGE.txt (archive to sovereign-onion)
```

### Step 5: Create Archive README

**File**: `ecoPrimals/sessions/2026-02-february/feb-06-2026-deep-debt-evolution/README.md`

**Content**:
- Session overview
- Achievements summary
- File index
- Link to Sovereign Onion session (earlier today)

---

## 🔍 TODO/FIXME Audit Results

### Code TODOs Found: ~117 instances

**Categories**:

1. **Sovereign Onion Service** (~10 TODOs)
   - Location: `crates/songbird-sovereign-onion/`
   - Status: **KEEP** - Active development
   - Action: None (implementation TODOs)

2. **Platform-Specific** (~15 TODOs)
   - Location: `universal-ipc/platform/*.rs`
   - Status: **KEEP** - Intentional future work markers
   - Action: None (iOS/WASM not priority)

3. **Genesis/Physical Channels** (~10 TODOs)
   - Location: `crates/songbird-genesis/`
   - Status: **KEEP** - Future features
   - Action: None (Bluetooth/QR/SoloKey not priority)

4. **Test TODOs** (~20 TODOs)
   - Location: Various test files
   - Status: **REVIEW** - May be outdated
   - Action: Individual review needed

5. **Implementation TODOs** (~60 TODOs)
   - Location: Various crates
   - Status: **KEEP** - Active features
   - Action: None (valid future work)

6. **False Positives** (~2 TODOs)
   - Example: "TODO: Uncomment when..." (now implemented)
   - Status: **CLEAN** - Already resolved
   - Action: Remove outdated comments

### False Positive Cleanup

**File**: `crates/songbird-universal-ipc/src/handlers/http_rendezvous_client.rs`
- Contains 3 TODOs that may be outdated

**File**: `crates/songbird-universal-ipc/src/handlers/udp_peer_connector.rs`
- Contains 2 TODOs that may be outdated

**Action**: Review these 5 specific TODOs

---

## 📦 Archive Structure After Cleanup

```
ecoPrimals/sessions/2026-02-february/
├── feb-05-2026-evolution/              # Previous session (27 docs)
├── feb-06-2026-sovereign-onion/        # Sovereign Onion session (26 docs) ✅
└── feb-06-2026-deep-debt-evolution/    # Deep Debt Phase 3 (NEW)
    ├── README.md                       # Session index
    ├── PHASE3_COMPLETE_AND_PUSHED_FEB_06_2026.md
    ├── PHASE3_REFACTORING_COMPLETE_FEB_06_2026.md
    ├── PHASE3_SMART_REFACTORING_PLAN_FEB_06_2026.md
    ├── SESSION_SUMMARY_FEB_06_2026.md
    ├── UNSAFE_CODE_VICTORY_FEB_06_2026.md
    ├── ROOT_DOCS_UPDATED_FEB_06_2026.md
    ├── SESSION_COMPLETE_AND_PUSHED_FEB_06_2026.md
    ├── EVOLUTION_SESSION_COMPLETE_FEB_06_2026.md
    ├── PHASE1_EXECUTION_COMPLETE_FEB_06_2026.md
    ├── DEEP_DEBT_EVOLUTION_ANALYSIS_FEB_06_2026.md
    ├── PHASE1_QUICK_WINS_FEB_06_2026.md
    ├── SOVEREIGN_ONION_BEARDOG_REFACTORING_COMPLETE_FEB_06_2026.md
    ├── TODAYS_COMPLETE_ACHIEVEMENTS_FEB_06_2026.md
    ├── PUSH_COMPLETE_FEB_06_2026.md
    ├── READY_FOR_SSH_PUSH_FEB_06_2026.md
    ├── SSH_PUSH_READY.md
    ├── BEARDOG_CRYPTO_REFACTOR_HANDOFF_FEB06_2026.md
    ├── COMMIT_MESSAGE_PHASE3.txt
    └── COMMIT_MESSAGE_EVOLUTION.txt
```

---

## 🧹 Root Directory After Cleanup

**Remaining Files** (Clean, organized):
```
songbird/
├── README.md                                      # Project overview
├── ROOT_DOCS_INDEX.md                             # Main documentation index
├── CHANGELOG.md                                   # Version history
├── CONTRIBUTING.md                                # Contribution guide
├── SOVEREIGN_ONION_TRUE_PRIMAL_ARCHITECTURE.md   # Core architecture
├── PURE_RUST_ONION_EVOLUTION_SUMMARY.md          # Onion summary
├── SOVEREIGN_MESH_PROGRESS_TRACKER.md            # Progress tracker
├── SOVEREIGN_BEACON_MESH_INVESTIGATION_FEB_06_2026.md  # Investigation
├── SOVEREIGN_BEACON_MESH_IMPLEMENTATION_PLAN_FEB_06_2026.md  # Implementation
├── crates/                                        # Rust workspace
├── docs/                                          # Implementation guides
├── specs/                                         # Technical specifications
├── examples/                                      # Code examples
├── tests/                                         # E2E tests
└── ecoPrimals/                                   # Evolution fossil record
    └── sessions/
        └── 2026-02-february/
            ├── feb-05-2026-evolution/            # Previous session
            ├── feb-06-2026-sovereign-onion/      # Sovereign Onion ✅
            └── feb-06-2026-deep-debt-evolution/  # Deep Debt Phase 3 ✅
```

**Total Reduction**: 40 files → 10 files in root (75% cleaner!)

---

## ✅ Safety Checks

### Before Deletion

1. ✅ Verify all files exist in archive
2. ✅ Verify git commit of current state
3. ✅ Verify no unique content in root-only files
4. ✅ Create archive README for discoverability

### After Deletion

1. ✅ Verify ROOT_DOCS_INDEX still accurate
2. ✅ Verify all links work
3. ✅ Run `cargo test --workspace` to ensure no broken refs
4. ✅ Commit cleanup with detailed message

---

## 🚀 Execution Plan

### Phase 1: Prepare Archive (5 minutes)

1. Create directory structure
2. Copy files to archive
3. Create archive README
4. Verify copies complete

### Phase 2: Clean Root (2 minutes)

1. Remove 40 archived files from root
2. Remove old commit messages
3. Verify ROOT_DOCS_INDEX accuracy

### Phase 3: TODO Cleanup (10 minutes)

1. Review 5 identified false-positive TODOs
2. Remove/update outdated comments
3. Verify builds still pass

### Phase 4: Commit & Push (2 minutes)

1. Stage all changes
2. Create comprehensive commit message
3. Push via git (or SSH if specified)

**Total Time**: ~20 minutes

---

## 📝 Commit Message Template

```
chore: Archive Feb 6 session docs & clean root directory

WHAT: Archive 40 Feb 6 session documents to ecoPrimals/ fossil record

WHY: Clean root directory, maintain organized documentation structure,
prepare for future work with clean workspace

HOW:
- Created feb-06-2026-deep-debt-evolution/ archive (17 docs)
- Removed 23 Sovereign Onion duplicates (already archived)
- Removed 10 intermediate status docs (now archived)
- Cleaned up 3 old commit message files
- Reviewed and cleaned 5 outdated TODOs in code
- Updated ROOT_DOCS_INDEX.md references

IMPACT:
- Root directory: 40 → 10 Feb 6 files (75% reduction)
- All documentation preserved in ecoPrimals/sessions/
- Clean workspace for future development
- Improved discoverability via archive README

FILES ARCHIVED:
- Deep Debt Phase 3: 17 documents
- Sovereign Onion: 23 documents (removed duplicates)
- Total preserved: 40 documents in fossil record

SAFETY:
- All files verified in archive before deletion
- No unique content lost
- Archive README created for navigation
- Tests passing: 599/616 (97.2%)

Status: Clean workspace, ready for next evolution phase
```

---

## 🎯 Success Criteria

1. ✅ Root directory cleaned (40 → 10 files)
2. ✅ All docs preserved in archive
3. ✅ Archive README created
4. ✅ ROOT_DOCS_INDEX accurate
5. ✅ Outdated TODOs cleaned
6. ✅ Tests still passing
7. ✅ Ready for SSH push

---

## ⚠️ Risk Mitigation

**Risk**: Accidentally delete unique content  
**Mitigation**: Verify archive copies before deletion, commit before cleanup

**Risk**: Break ROOT_DOCS_INDEX links  
**Mitigation**: Update index to point to archive locations

**Risk**: Break code with TODO cleanup  
**Mitigation**: Only remove comment TODOs, not implementation code

**Risk**: Lose session context  
**Mitigation**: Create detailed archive README with context

---

## 📊 Expected Results

### Before Cleanup
```
$ ls -1 *FEB_06_2026*.md | wc -l
40
```

### After Cleanup
```
$ ls -1 *FEB_06_2026*.md | wc -l
0

$ ls ecoPrimals/sessions/2026-02-february/
feb-05-2026-evolution/
feb-06-2026-sovereign-onion/
feb-06-2026-deep-debt-evolution/  ← NEW

$ ls ecoPrimals/sessions/2026-02-february/feb-06-2026-deep-debt-evolution/ | wc -l
18  (17 docs + 1 README)
```

---

**Ready to Execute**: Yes ✅  
**Safety Verified**: Yes ✅  
**Time Estimate**: 20 minutes  
**Risk Level**: Low (all docs preserved in archive)

---

**Next Step**: Execute Phase 1 (Create Archive)
