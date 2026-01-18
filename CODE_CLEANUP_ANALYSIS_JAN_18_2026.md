# Code Cleanup Analysis - January 18, 2026

**Status**: Post Pure Songbird TLS Completion  
**Grade**: A++ (100% ecoBin)  
**Goal**: Identify archived code for cleanup (keeping docs as fossil record)

---

## 🎯 CLEANUP CANDIDATES

### 1. ✅ ALREADY CLEANED
- `crates/songbird-orchestrator/src/crypto/rustls_provider/` - REMOVED (Jan 18)
  - Complete directory deleted after pivot to Pure Songbird TLS
  - No longer needed (Pure TLS implementation in songbird-tls crate)

### 2. 🔧 DEPRECATED FILES TO REMOVE

#### A. Deprecated Code Files
```
crates/songbird-orchestrator/src/ipc/server.rs.deprecated
  • Size: 18KB
  • Last modified: Jan 11
  • Action: DELETE (code, not docs)
  • Reason: Superseded by current implementation
```

#### B. Deprecated Spec Files
```
specs/archive/deprecated-protocols/GRPC_GATEWAY_ADAPTER_SPECIFICATION.md.deprecated
  • Action: KEEP (docs = fossil record)
  • Reason: Historical documentation
```

### 3. ⚠️  DEPRECATED CODE PATTERNS (Active Files)

#### A. BEARDOG_URL Environment Variable
**Status**: Deprecated but still functional (backward compat)  
**Schedule**: Remove Q4 2026 (per DEPRECATION_SCHEDULE.md)  
**Files**:
- `crates/songbird-orchestrator/src/trust/escalation.rs` (lines 91-92)
- `crates/songbird-orchestrator/src/app/discovery_startup.rs` (line 15, 128)
- `crates/songbird-orchestrator/src/app/security_setup.rs` (lines 43, 69-74)
- `crates/songbird-orchestrator/src/access_control/auth.rs` (line 377)

**Action**: KEEP (removal scheduled for Q4 2026)  
**Notes**: Properly documented in DEPRECATION_SCHEDULE.md

#### B. Hardcoded Primal Type Aliases
**File**: `crates/songbird-orchestrator/src/core/biome/modules/types.rs`  
**Lines**: 363-733  
**Status**: Commented out with REMOVED/DEPRECATED markers  
**Action**: DELETE commented code (already marked as removed Jan 17, 2026)

**Example**:
```rust
// ✅ REMOVED (Jan 17, 2026): Hardcoded primal type aliases
// pub type NestGateConfig = AgnosticPrimalConfig;
// pub type ToadstoolConfig = AgnosticPrimalConfig;
// ...
```

**Action**: Remove commented-out code blocks (lines 363-733)

### 4. 🚫 FALSE POSITIVES (DO NOT REMOVE)

#### A. rustls References (Still Needed)
**Files**:
- `crates/songbird-network/src/tls.rs` - Current TLS helper (uses rustls)
- `crates/songbird-orchestrator/src/main.rs` - Crypto provider init (required)
- `crates/songbird-network-federation/src/tls.rs` - Federation TLS
- `crates/songbird-cli/src/cli/commands/tower.rs` - Tower command

**Reason**: These are **current working code** for existing rustls-based HTTPS server.  
**Status**: Will be replaced during **Integration Phase** (Week 5+) when songbird-tls is integrated.  
**Action**: KEEP until integration complete

#### B. SQL DELETE Statements
**Reason**: Normal database operations, not deprecated code  
**Action**: KEEP (required functionality)

#### C. HTTP DELETE Methods
**Reason**: REST API operations, not deprecated code  
**Action**: KEEP (required functionality)

#### D. DEFAULT_CLEANUP_INTERVAL
**Reason**: Active configuration constant for resource cleanup  
**Action**: KEEP (required functionality)

### 5. 📝 OUTDATED TODOs

**Found**: 8 matches total  
**Categories**:
1. Documentation TODOs (in archived docs) - KEEP as fossil record
2. Bluetooth pairing TODO - KEEP (legitimate future work)
3. System monitoring TODOs in specs - KEEP (design documentation)

**Action**: No cleanup needed (all legitimate or in docs)

---

## 🎯 RECOMMENDED CLEANUP ACTIONS

### Immediate (Safe to Delete)

1. **Delete deprecated code file**:
   ```bash
   rm crates/songbird-orchestrator/src/ipc/server.rs.deprecated
   ```

2. **Remove commented-out code** in `crates/songbird-orchestrator/src/core/biome/modules/types.rs`:
   - Lines 363-733 (hardcoded primal type aliases marked REMOVED)
   - These are large commented blocks already documented as removed Jan 17, 2026

### Future (Scheduled)

3. **Remove BEARDOG_URL deprecations** (Q4 2026):
   - Per DEPRECATION_SCHEDULE.md
   - Not immediate - properly scheduled removal

4. **Replace rustls references** (Week 5+ Integration):
   - After songbird-tls integration into songbird-orchestrator
   - Replace in:
     - `crates/songbird-network/src/tls.rs`
     - `crates/songbird-orchestrator/src/main.rs`
     - `crates/songbird-orchestrator/src/app/http_server.rs`
     - Federation and CLI components

### Keep (Docs = Fossil Record)

5. **Preserve all documentation**:
   - `specs/archive/deprecated-protocols/` - Historical specs
   - `docs/archive/older_docs` - Historical documentation
   - All session documentation in `docs/sessions/`

---

## 📊 CLEANUP SUMMARY

| Category | Count | Action |
|----------|-------|--------|
| Deprecated Code Files | 1 | DELETE |
| Commented Code Blocks | ~370 lines | REMOVE |
| Deprecated Functions (scheduled) | 5 | KEEP (Q4 2026) |
| False Positive (rustls) | 5 files | KEEP (integration pending) |
| Outdated TODOs | 0 | None found |
| Documentation (fossil) | All | KEEP |

---

## ✅ VERIFICATION CHECKLIST

Before cleanup:
- [x] Identify all deprecated code files
- [x] Verify rustls references are current (not deprecated)
- [x] Check DEPRECATION_SCHEDULE.md for scheduled removals
- [x] Ensure docs are preserved as fossil record
- [x] Confirm no active TODOs related to removed features

After cleanup:
- [ ] Run `cargo build` to ensure no breakage
- [ ] Run `cargo test` to ensure tests pass
- [ ] Verify git history preserves removed code
- [ ] Update DEPRECATION_SCHEDULE.md if needed
- [ ] Commit with clear message

---

## 🎯 EXECUTION PLAN

```bash
# 1. Remove deprecated code file
rm crates/songbird-orchestrator/src/ipc/server.rs.deprecated

# 2. Remove commented code blocks in types.rs
# (Manual edit to remove lines 363-733 marked as REMOVED)

# 3. Verify build
cargo build --workspace

# 4. Verify tests
cargo test --workspace

# 5. Commit
git add -A
git commit -m "refactor: Remove deprecated code files and commented-out types

Cleanup after Pure Songbird TLS completion:

Removed:
- crates/songbird-orchestrator/src/ipc/server.rs.deprecated (superseded)
- Commented-out hardcoded primal type aliases (marked REMOVED Jan 17)

Kept:
- All documentation (fossil record)
- Scheduled deprecations (BEARDOG_URL - Q4 2026)
- Active rustls references (integration pending)

Result: Cleaner codebase, no functionality changes"
```

---

## 🏆 EXPECTED RESULT

**Before Cleanup**:
- 1 deprecated code file (.deprecated extension)
- ~370 lines of commented-out code
- Clear codebase, but could be cleaner

**After Cleanup**:
- No deprecated code files
- No commented-out code blocks
- Pristine codebase
- All documentation preserved
- All scheduled deprecations remain (Q4 2026)
- All active code unchanged

**Impact**: Cleaner codebase, no functionality changes, proper separation of code vs docs.

---

**Status**: Ready for execution  
**Risk**: Low (only removing already-deprecated code)  
**Benefit**: Cleaner codebase aligned with "deep debt solutions"
