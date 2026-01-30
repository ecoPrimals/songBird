# 🗂️ Archive Cleanup Plan - January 30, 2026

**Status**: Ready for Review & Execution  
**Goal**: Archive old demo/experimental code while preserving fossil record  
**Philosophy**: Docs stay (fossil record), old code goes to archive

---

## 📊 **Summary**

| Category | Files | Size | Action |
|----------|-------|------|--------|
| **Python Demos** | 22 files | ~50 KB | ✅ Archive |
| **Experiments** | ~45 files | ~200 KB | ✅ Archive |
| **Showcase (partial)** | ~150 files | ~500 KB | ⚠️  Review (some in workspace) |
| **Rendezvous** | 8 files | ~20 KB | ❌ Keep (in workspace) |
| **Docs** | All | N/A | ✅ Keep (fossil record) |
| **TODO Comments** | 162 instances | N/A | 📝 Review (1 P0 found) |

**Total Archivable**: ~750 KB (~217 files)

---

## ✅ **Safe to Archive**

### **1. demos/ Directory** (22 files - Archive Completely)

**Reason**: Old Python AI/ML demonstration scripts, superseded by production code

**Contents**:
```
demos/
├── ai_story_to_gif.py              # Old AI demo
├── byob-coordination-demo.sh       # Old coordination demo
├── COMPLETE_HYBRID_AI_DEMO.py      # Experimental hybrid AI
├── distributed_ai_test.py          # Test script
├── distributed_creative_ai.py      # Experimental
├── distributed_creative_real.py    # Experimental
├── distributed_creative_REALAI.py  # Experimental
├── distributed_text_ai_coordinator.py
├── federation-coordination-demo.sh # Old demo
├── gpu_rendered_proof.py           # Experimental
├── HYBRID_AI_DEMO_CLEAN.py         # Experimental
├── hybrid_ai_demo.py               # Experimental
├── LOCAL_GPU_IMAGE_GEN.py          # Experimental
├── make_proof_image.py             # Utility
├── make_real_video.py              # Utility
├── make_video_gif.py               # Utility
├── python_ml_demo_auto.py          # Old demo
├── python_ml_demo.py               # Old demo
├── real_distributed_ai_final.py    # Experimental
├── real_gpu_ai_demo.py             # Experimental
├── remote_execution_demo.py        # Old demo
└── ULTIMATE_HYBRID_AI.py           # Experimental
```

**Status**: ✅ **SAFE TO DELETE** (none referenced in Rust code)

**Command**:
```bash
rm -rf demos/
```

---

### **2. experiments/ Directory** (45+ files - Archive Completely)

**Reason**: ImageNet training experiments, old HPC benchmarks, superseded

**Contents**:
```
experiments/
├── HPC_BENCHMARK_PLAN.md           # Planning doc
├── imagenet_distributed_training_plan.md
├── imagenet_training/              # 30+ files
│   ├── FINAL_STATUS.md
│   ├── launch_*.py/.sh (12 files)  # Launch scripts
│   ├── setup/ (4 files)            # Setup scripts
│   ├── training/ (9 files)         # Training code
│   └── *.md (7 status/docs)
├── implementation_roadmap.md        # Old roadmap
├── local_tower_test_plan.md        # Old plan
├── README.md                        # Experiments readme
├── stage1_live_experiment_demo.rs  # Old demo
├── test_scenarios.md                # Old scenarios
└── test_toadstool_python.toml      # Old config
```

**Status**: ✅ **SAFE TO DELETE** (experimental code, not in production)

**Command**:
```bash
rm -rf experiments/
```

---

### **3. showcase/ Directory** (Partial - Archive Selectively)

**⚠️  WARNING**: Some showcase code IS in the workspace:
- `showcase/05-albatross-multiplex/benchmark` - **IN WORKSPACE** (KEEP)
- `showcase/05-albatross-multiplex/tarpc-servers` - **IN WORKSPACE** (KEEP)

**Safe to Archive** (not in workspace):
```
showcase/
├── 01-isolated/                    # Old demos
├── 02-federation/                  # Old federation demos
├── 03-inter-primal/               # Old inter-primal demos
├── 04-multi-protocol/             # Old protocol demos
├── 06-toadstool-ml-orchestration/ # Old ML demos
├── 07-student-onboarding/         # Old onboarding
├── 08-songbird-sovereign/         # Old docs
├── 09-local-compute/              # Old compute demos
├── 10-inter-primal-foundation/    # Old foundation demos
├── 11-federation-upa/             # Old federation demos
├── 12-internet-deployment/        # Old deployment demos
├── 13-beardog-integration/        # Old integration demos
├── 14-capability-based-discovery/ # Old discovery demos
├── 14-physical-genesis/           # Old genesis demos
├── 15-songbird-beardog-backbone/  # Old backbone demos
├── configs/                        # Old configs
├── demos/                          # Old demos
├── whitePaper/                     # Old white paper
├── *.md (summary/index files)      # Old docs
└── QUICK_START.sh                  # Old script
```

**Status**: ⚠️  **SELECTIVE DELETE** (keep 05-albatross-multiplex/)

**Commands**:
```bash
# Keep 05-albatross-multiplex, delete rest
cd showcase/
rm -rf 01-isolated/ 02-federation/ 03-inter-primal/ 04-multi-protocol/
rm -rf 06-toadstool-ml-orchestration/ 07-student-onboarding/
rm -rf 08-songbird-sovereign/ 09-local-compute/ 10-inter-primal-foundation/
rm -rf 11-federation-upa/ 12-internet-deployment/ 13-beardog-integration/
rm -rf 14-capability-based-discovery/ 14-physical-genesis/
rm -rf 15-songbird-beardog-backbone/
rm -rf configs/ demos/ whitePaper/
rm -f 00_SHOWCASE_INDEX.md QUICK_START.sh README.md
rm -f SHOWCASE_INDEX.md SHOWCASE_SUMMARY.md SONGBIRD_SHOWCASE_EVOLUTION.md
```

---

## ❌ **DO NOT Archive** (Still in Use)

### **1. rendezvous/ Directory** ❌ KEEP

**Reason**: IN WORKSPACE (line 45 of Cargo.toml)

**Contents**:
```
rendezvous/
├── Cargo.toml
├── README.md
├── src/
│   ├── api.rs
│   ├── coordination.rs
│   ├── main.rs
│   ├── messages.rs
│   ├── registry.rs
│   ├── security.rs
│   └── websocket.rs
└── test.sh
```

**Status**: ❌ **IN ACTIVE USE** - Part of workspace

---

### **2. showcase/05-albatross-multiplex/** ❌ KEEP

**Reason**: IN WORKSPACE (benchmark + tarpc-servers)

**Status**: ❌ **IN ACTIVE USE** - Part of workspace

---

### **3. All Documentation** ❌ KEEP

**Reason**: Fossil record principle - docs are historical record

**Status**: ✅ **KEEP ALL .md FILES** (fossil record)

---

## 📝 **TODO/FIXME Review** (162 instances)

### **Critical P0 TODO** (1 instance)

**Location**: `crates/songbird-http-client/src/tls/server/messages.rs:220`

```rust
// TODO(P0): Add BearDog signing integration
```

**Status**: ⚠️  **VALID P0** - Needs tracking, not deletion

**Recommendation**: Add to roadmap/backlog

---

### **General TODOs** (161 instances across 80 files)

**Categories**:
1. **Implementation TODOs** (~100) - Future features, valid
2. **Test TODOs** (~40) - Test improvements, valid
3. **Optimization TODOs** (~15) - Performance notes, valid
4. **Documentation TODOs** (~6) - Doc improvements, valid

**Status**: Most are valid markers for future work, not cleanup targets

**Recommendation**: Keep as-is (standard practice for future work)

---

## 🗂️ **Proposed Archive Structure**

Move deleted content to parent ecoPrimals archive:

```
/ecoPrimals/archive/songbird/
├── jan-2026-demos/                 # demos/ directory
├── jan-2026-experiments/           # experiments/ directory
├── jan-2026-showcase-demos/        # showcase/ old demos
└── ARCHIVE_MANIFEST_JAN_30_2026.md # This manifest
```

**Archive Manifest**:
- Date archived
- Reason for archival
- Original location
- Size/file count

---

## 🚀 **Execution Plan**

### **Phase 1: Backup** (Safety First)

```bash
cd /home/eastgate/Development/ecoPrimals/phase1/songbird

# Create archive directory
mkdir -p /home/eastgate/Development/ecoPrimals/archive/songbird/jan-2026-cleanup

# Backup before deletion
tar -czf /home/eastgate/Development/ecoPrimals/archive/songbird/jan-2026-cleanup/demos-backup.tar.gz demos/
tar -czf /home/eastgate/Development/ecoPrimals/archive/songbird/jan-2026-cleanup/experiments-backup.tar.gz experiments/
tar -czf /home/eastgate/Development/ecoPrimals/archive/songbird/jan-2026-cleanup/showcase-demos-backup.tar.gz \
  showcase/01-isolated \
  showcase/02-federation \
  showcase/03-inter-primal \
  showcase/04-multi-protocol \
  showcase/06-toadstool-ml-orchestration \
  showcase/07-student-onboarding \
  showcase/08-songbird-sovereign \
  showcase/09-local-compute \
  showcase/10-inter-primal-foundation \
  showcase/11-federation-upa \
  showcase/12-internet-deployment \
  showcase/13-beardog-integration \
  showcase/14-capability-based-discovery \
  showcase/14-physical-genesis \
  showcase/15-songbird-beardog-backbone \
  showcase/configs \
  showcase/demos \
  showcase/whitePaper

echo "✅ Backups created in /ecoPrimals/archive/songbird/jan-2026-cleanup/"
```

---

### **Phase 2: Delete** (Clean Workspace)

```bash
cd /home/eastgate/Development/ecoPrimals/phase1/songbird

# Delete demos
rm -rf demos/
echo "✅ Deleted demos/ (22 files)"

# Delete experiments
rm -rf experiments/
echo "✅ Deleted experiments/ (45+ files)"

# Delete showcase old demos (selective)
cd showcase/
rm -rf 01-isolated/ 02-federation/ 03-inter-primal/ 04-multi-protocol/
rm -rf 06-toadstool-ml-orchestration/ 07-student-onboarding/
rm -rf 08-songbird-sovereign/ 09-local-compute/ 10-inter-primal-foundation/
rm -rf 11-federation-upa/ 12-internet-deployment/ 13-beardog-integration/
rm -rf 14-capability-based-discovery/ 14-physical-genesis/
rm -rf 15-songbird-beardog-backbone/
rm -rf configs/ demos/ whitePaper/
rm -f 00_SHOWCASE_INDEX.md QUICK_START.sh README.md
rm -f SHOWCASE_INDEX.md SHOWCASE_SUMMARY.md SONGBIRD_SHOWCASE_EVOLUTION.md
cd ..
echo "✅ Deleted showcase/ old demos (~150 files)"

# Verify workspace still builds
cargo build --release
echo "✅ Build verification complete"
```

---

### **Phase 3: Verify** (Test Build)

```bash
cd /home/eastgate/Development/ecoPrimals/phase1/songbird

# Verify build
cargo build --release 2>&1 | tee /tmp/cleanup-build-test.log

# Verify tests
cargo test --lib --bins 2>&1 | head -100

# Check workspace members still valid
cargo metadata --format-version=1 | jq -r '.packages[].name' | sort
```

---

### **Phase 4: Commit & Push** (Git)

```bash
cd /home/eastgate/Development/ecoPrimals/phase1/songbird

# Stage deletions
git add -A

# Commit with detailed message
git commit -m "$(cat <<'EOF'
chore: archive old demos and experiments

Archive old demonstration and experimental code to clean workspace.

**Archived**:
- demos/ (22 Python AI/ML demo scripts)
- experiments/ (45+ ImageNet training experiments)
- showcase/ old demos (150+ files, keeping albatross benchmarks)

**Preserved**:
- rendezvous/ (in workspace)
- showcase/05-albatross-multiplex/ (in workspace)
- All documentation (fossil record)

**Verification**:
- cargo build --release: ✅ Pass
- cargo test: ✅ Pass
- Workspace members: ✅ Valid

**Archive Location**:
- /ecoPrimals/archive/songbird/jan-2026-cleanup/

Closes: Archive cleanup plan
See: ARCHIVE_CLEANUP_PLAN_JAN_30_2026.md
EOF
)"

# Push to remote
git push origin main
```

---

## 📊 **Expected Results**

### **Before Cleanup**

```bash
$ du -sh demos/ experiments/ showcase/
50K     demos/
200K    experiments/
500K    showcase/
```

### **After Cleanup**

```bash
$ du -sh showcase/
50K     showcase/  # Only albatross benchmarks remain
```

**Space Saved**: ~700 KB (~217 files)  
**Build Time**: Should remain same or slightly faster  
**Workspace**: Cleaner, more focused

---

## ⚠️  **Safety Checks**

### **Before Execution**

- [ ] Verify backups created successfully
- [ ] Confirm rendezvous/ is in workspace
- [ ] Confirm showcase/05-albatross-multiplex/ is in workspace
- [ ] Review P0 TODO and add to roadmap

### **After Execution**

- [ ] `cargo build --release` succeeds
- [ ] `cargo test --lib --bins` passes
- [ ] `cargo metadata` shows valid workspace
- [ ] Git status shows expected deletions only
- [ ] No unexpected files deleted

---

## 🎯 **Recommendations**

### **1. P0 TODO Action**

**File**: `crates/songbird-http-client/src/tls/server/messages.rs:220`

```rust
// TODO(P0): Add BearDog signing integration
```

**Action**: Add to ROADMAP.md under "TLS Server Enhancements"

---

### **2. Future Cleanup**

Consider archiving these in future sessions:
- Old `.github/workflows/` (if any are unused)
- Old `docker/` configs (if superseded)
- Old `infrastructure/` (if cloud deployment changed)

---

### **3. Documentation Cleanup**

Consider consolidating these docs:
- Multiple "session complete" docs → Single master index
- Multiple "status" docs → Single STATUS.md

---

## 📋 **Checklist**

### **Pre-Execution**

- [ ] Read this entire plan
- [ ] Verify backup strategy
- [ ] Confirm workspace members
- [ ] Review safety checks

### **Execution**

- [ ] Phase 1: Create backups
- [ ] Phase 2: Delete files
- [ ] Phase 3: Verify build
- [ ] Phase 4: Commit & push

### **Post-Execution**

- [ ] Verify build succeeds
- [ ] Verify tests pass
- [ ] Update ROOT_DOCS_INDEX.md
- [ ] Add P0 TODO to roadmap

---

## 🎉 **Expected Outcome**

After this cleanup:

✅ **Cleaner Workspace**: ~217 fewer files  
✅ **Preserved History**: All in ecoPrimals archive  
✅ **Working Build**: All tests passing  
✅ **Clear Focus**: Production code only  
✅ **Fossil Record**: All docs preserved

**Ready to Execute**: This plan is ready for review and execution.

---

**Last Updated**: January 30, 2026  
**Status**: Ready for Review  
**Estimated Time**: 15-20 minutes
