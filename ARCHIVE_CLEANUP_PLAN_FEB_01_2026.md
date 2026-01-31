# 🧹 Archive Cleanup Plan - February 1, 2026

**Purpose**: Move superseded/interim documents to fossil record archive, clean outdated TODOs

---

## 📋 **CLEANUP STRATEGY**

### **1. Superseded Documents → ecoPrimals Archive**

Move interim session documents that are superseded by comprehensive summaries:

#### **Superseded by `LEGENDARY_SESSION_COMPLETE_FEB_01_2026.md`**:
- `CONTINUE_DIRECTIVE_COMPLETE_FEB_01_2026.md` (subsumed)
- `EPIC_SESSION_FINAL_HANDOFF_FEB_01_2026.md` (subsumed)
- `FINAL_LEGENDARY_SESSION_SUMMARY_JAN_31_2026.md` (superseded)
- `EPIC_SESSION_SUMMARY_JAN_31_2026.md` (superseded)

#### **Interim Phase Documents** (move to ecoPrimals):
- `PHASE1_COMPLETE_SUMMARY_JAN_31_2026.md` (interim)
- `PHASE2_READY_HANDOFF_JAN_31_2026.md` (interim)
- `HANDOFF_ARCHIVE_COMPLETE_FEB_01_2026.md` (interim)

#### **Archive Documents** (already about archiving, move to ecoPrimals):
- `ARCHIVE_CLEANUP_FINAL_FEB_01_2026.md` (meta-archive doc)

---

### **2. Keep in Root** (Essential Reference)

✅ **KEEP** - Active, authoritative documents:
- `LEGENDARY_SESSION_COMPLETE_FEB_01_2026.md` ⭐ Primary summary
- `ALL_DEEP_DEBT_DIRECTIVES_COMPLETE_FEB_01_2026.md` - Deep debt results
- `MDNS_INTEGRATION_COMPLETE_FEB_01_2026.md` - mDNS technical guide  
- `MDNS_ALREADY_COMPLETE_FEB_01_2026.md` - mDNS analysis
- `PHASE2_HARDCODING_ANALYSIS_FEB_01_2026.md` - Key insight
- `DEEP_DEBT_STATUS_COMPREHENSIVE_FEB_01_2026.md` - Status
- `ISOMORPHIC_IPC_PHASE3_COMPLETE_FEB_01_2026.md` - IPC Phase 3
- `ISOMORPHIC_IPC_VALIDATION_COMPLETE_FEB_01_2026.md` - IPC validation
- `ISOMORPHIC_IPC_EVOLUTION_JAN_31_2026.md` - IPC architecture
- `HANDOFF_ISOMORPHIC_IPC_BIOMEOS_FEB_01_2026.md` - biomeOS handoff
- `DEPENDENCY_AUDIT_DEEP_DEBT_JAN_31_2026.md` - Audit basis
- `DEPENDENCY_CLEANUP_LTO_EXECUTION_JAN_31_2026.md` - Optimizations
- `PRIORITY_5_6_FINAL_ANALYSIS_JAN_31_2026.md` - Priority 5-6
- `PRIORITY_3_4_CONFIG_OPTIMIZATION_JAN_31_2026.md` - Priority 3-4
- `ARM64_GENOMEBIN_V3_DEEP_DEBT_ANALYSIS_JAN_31_2026.md` - ARM64 analysis
- `ARM64_LOCAL_BUILD_SESSION_JAN_31_2026.md` - Build session
- `SONGBIRD_DEEP_DEBT_EVOLUTION_NUCLEUS_JAN_31_2026.md` - Deep debt evolution
- `GENOMEBIN_WEEK3_COMPLETE_JAN_31_2026.md` - Week 3
- `GENOMEBIN_WEEK2_DEPLOYMENT_COMPLETE_JAN_31_2026.md` - Week 2
- `GENOMEBIN_WEEK1_VICTORY_JAN_31_2026.md` - Week 1
- `GENOMEBIN_WRAPPER_TESTING_JAN_31_2026.md` - Testing

---

### **3. Outdated TODOs to Clean**

#### **Code TODOs - Now Complete (Remove)**:

1. **`crates/songbird-config/src/capability_discovery.rs:356`**
   ```rust
   #[allow(clippy::unused_async)] // TODO: Will use .await when implementing mDNS discovery
   ```
   **Action**: Remove comment - mDNS is now implemented!
   
2. **`crates/songbird-discovery/src/lineage_discovery.rs:97,111`**
   ```rust
   // TODO: Actual mDNS broadcast implementation
   // TODO: Actual mDNS discovery implementation
   ```
   **Action**: Remove or update - mDNS is implemented in songbird-config

3. **`crates/songbird-orchestrator/src/universal_adapter.rs:252`**
   ```rust
   // TODO: Implement actual mDNS discovery
   ```
   **Action**: Update to reference songbird-config implementation

4. **`examples/infant_discovery_demo.rs:161`**
   ```rust
   // TODO: Implement when mdns crate is available
   ```
   **Action**: Remove - mdns-sd is available and integrated

---

### **4. False Positive TODOs** (Keep - Future Enhancements)

These are legitimate future enhancements, NOT outdated:

✅ **KEEP**:
- `docs/genetic-lineage-integration.md:398` - "Integrate with BearDog API" (future)
- `crates/songbird-http-client/src/tls/handshake_v2/mod.rs:40` - Module integration (future)
- `docs/ENHANCEMENT_ROADMAP.md` - ZeroKnowledgeBootstrap TODOs (future features)

---

## 🎯 **EXECUTION PLAN**

### **Step 1: Create Archive Structure**
```bash
mkdir -p ecoPrimals/sessions/feb-01-2026
mkdir -p ecoPrimals/sessions/jan-31-2026
```

### **Step 2: Move Superseded Documents**
```bash
# Superseded by comprehensive summary
mv CONTINUE_DIRECTIVE_COMPLETE_FEB_01_2026.md ecoPrimals/sessions/feb-01-2026/
mv EPIC_SESSION_FINAL_HANDOFF_FEB_01_2026.md ecoPrimals/sessions/feb-01-2026/
mv FINAL_LEGENDARY_SESSION_SUMMARY_JAN_31_2026.md ecoPrimals/sessions/jan-31-2026/
mv EPIC_SESSION_SUMMARY_JAN_31_2026.md ecoPrimals/sessions/jan-31-2026/

# Interim phase documents
mv PHASE1_COMPLETE_SUMMARY_JAN_31_2026.md ecoPrimals/sessions/jan-31-2026/
mv PHASE2_READY_HANDOFF_JAN_31_2026.md ecoPrimals/sessions/jan-31-2026/
mv HANDOFF_ARCHIVE_COMPLETE_FEB_01_2026.md ecoPrimals/sessions/feb-01-2026/

# Meta-archive document
mv ARCHIVE_CLEANUP_FINAL_FEB_01_2026.md ecoPrimals/sessions/feb-01-2026/
```

### **Step 3: Clean Outdated TODOs**

**File**: `crates/songbird-config/src/capability_discovery.rs`
- Line 356: Remove `// TODO: Will use .await when implementing mDNS discovery`

**File**: `crates/songbird-discovery/src/lineage_discovery.rs`
- Line 97: Update to reference `songbird-config::discovery::MdnsDiscovery`
- Line 111: Same

**File**: `crates/songbird-orchestrator/src/universal_adapter.rs`
- Line 252: Update to reference implementation

**File**: `examples/infant_discovery_demo.rs`
- Line 161: Remove outdated TODO

### **Step 4: Create Archive Index**

Create `ecoPrimals/INDEX.md` documenting what's archived and why.

---

## 📊 **IMPACT SUMMARY**

### **Documents to Archive**: 8 files
- 4 superseded summaries
- 3 interim phase documents
- 1 meta-archive document

### **TODOs to Clean**: 4 locations
- 1 outdated comment (mDNS in capability_discovery.rs)
- 3 outdated TODOs (lineage_discovery, universal_adapter, infant_demo)

### **Keep in Root**: 21 essential documents
All authoritative, technical, or active reference documents

---

## ✅ **BENEFITS**

1. **Cleaner Root** - Only essential, authoritative docs visible
2. **Preserved History** - All work kept as fossil record in ecoPrimals/
3. **Updated Code** - Outdated TODOs cleaned
4. **Better Navigation** - Clear what's current vs historical

---

**Ready to execute!** 🚀
