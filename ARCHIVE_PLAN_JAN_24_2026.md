# Documentation Archive Plan - January 24, 2026

## 📚 Overview

After the 19+ hour evolution session, we have 66 markdown documents in root.
This plan identifies what to keep vs archive for cleaner organization.

---

## ✅ KEEP IN ROOT (Core Documentation - 8 files)

### Essential Project Docs
1. `README.md` - Main project documentation
2. `STATUS.md` - Current status (v5.16.0)
3. `CHANGELOG.md` - Version history
4. `CONTRIBUTING.md` - Contribution guidelines
5. `ROADMAP.md` - Future plans
6. `QUICK_START.md` - Getting started guide
7. `DOCS_GUIDE.md` - Documentation index
8. `FUTURE_ENHANCEMENTS.md` - Enhancement ideas

---

## ✅ KEEP IN ROOT (Evolution Reports - 6 files)

### Phase Reports (January 24, 2026 Session)
1. `DEEP_DEBT_EVOLUTION_PLAN_JAN_24_2026.md` - 5-phase strategic plan
2. `PHASE_2_REFACTORING_COMPLETE_JAN_24_2026.md` - Module extraction complete
3. `PHASE_3_UNSAFE_AUDIT_COMPLETE_JAN_24_2026.md` - Safety validation
4. `PHASE_4_MODERN_RUST_AUDIT_COMPLETE_JAN_24_2026.md` - Idioms validation
5. `SESSION_SUMMARY_JAN_24_2026.md` - 19+ hour session summary
6. `HANDSHAKE_REFACTOR_PLAN_JAN_24_2026.md` - Refactoring strategy

**Why Keep**: These document the systematic evolution that transformed the architecture.

---

## 📦 ARCHIVE (Historical Debug/Session Docs - ~46 files)

### Category 1: TLS Debugging Sessions (Jan 22-23)
- All `*_FIX_JAN_2[0-3]_2026.md` files
- All `*_DEBUG_*_JAN_2[0-3]_2026.md` files
- All `SESSION[0-9]*_JAN_2[0-3]_2026.md` files

**Reason**: Specific debugging sessions, now superseded by Phase 2-4 reports

### Category 2: Integration/Handoff Docs
- `BIOMEOS_*.md` (multiple handoff documents)
- `BEARDOG_*.md` (coordination docs)
- `SONGBIRD_V5.*.md` (version-specific handoffs)

**Reason**: Version-specific, superseded by current STATUS.md

### Category 3: Protocol Evolution Docs
- `TLS_PROTOCOL_*.md` (evolution plans)
- `TLS_1.3_HARDENING_*.md`
- `RFC_8446_*.md` (implementation docs)

**Reason**: Implementation details, now complete and documented in modules

### Category 4: Specific Feature Docs
- `ADAPTIVE_TLS_*.md`
- `PROGRESSIVE_FALLBACK_*.md`
- `HTTP_MULTI_RECORD_*.md`

**Reason**: Feature-specific, now integrated and documented in code

### Category 5: Analysis/Investigation Docs
- `*_INVESTIGATION_*.md`
- `*_DIAGNOSTICS_*.md`
- `*_INSTRUMENTATION_*.md`
- `KEY_DERIVATION_*.md`
- `TRANSCRIPT_HASH_*.md`
- `DECRYPT_ERROR_*.md`

**Reason**: Point-in-time investigations, now resolved

---

## 📋 ARCHIVE STRUCTURE

```
archive/
├── historical-snapshots/
│   └── jan-2026-sessions/
│       ├── debugging/              (TLS debugging docs)
│       ├── integration/            (biomeOS handoffs)
│       ├── protocol-evolution/     (TLS protocol work)
│       ├── features/               (Specific features)
│       └── investigations/         (Troubleshooting)
```

---

## 🎯 BENEFITS

**Before**: 66 files in root (overwhelming)
**After**: 14 core files in root (clean, focused)

**Archive Benefits**:
- ✅ Fossil record preserved
- ✅ Historical context available
- ✅ Root directory clean
- ✅ Easy to find current docs
- ✅ Evolution story intact

---

## 🚀 EXECUTION PLAN

### Step 1: Create Archive Structure
```bash
mkdir -p archive/historical-snapshots/jan-2026-sessions/{debugging,integration,protocol-evolution,features,investigations}
```

### Step 2: Move Documents by Category
- Move debugging docs to `debugging/`
- Move handoff docs to `integration/`
- Move protocol docs to `protocol-evolution/`
- Move feature docs to `features/`
- Move investigation docs to `investigations/`

### Step 3: Verify & Commit
- Verify 14 files remain in root
- Commit with descriptive message
- Push to main

---

## ✅ RESULT

**Root Directory**: Clean, focused, current
**Archive**: Complete historical record
**Quality**: A++ organization

**"Clean root, preserved history!"** 📚✨
