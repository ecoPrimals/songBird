# 🔍 Songbird Dependency Audit - Deep Debt Evolution

**Date**: January 31, 2026 (Night)  
**Purpose**: Eliminate inefficient dependencies and reduce binary size  
**Current Size**: x86_64: 27 MB, ARM64: 25 MB (larger than other primals)

═══════════════════════════════════════════════════════════════════
🎯 KEY FINDINGS: MAJOR OPTIMIZATION OPPORTUNITIES
═══════════════════════════════════════════════════════════════════

## **CRITICAL ISSUES FOUND**:

### 1. **DUPLICATE DNS RESOLVERS** ❌ (Bloat + Technical Debt)

**Problem**: TWO DNS resolver libraries for the SAME purpose!

```toml
# workspace.dependencies
hickory-resolver = "0.24"  # Modern (maintained)
trust-dns-resolver = "0.23"  # Legacy (UNMAINTAINED!)
```

**Impact**:
- ❌ Duplicate functionality (~500 KB each = 1 MB wasted)
- ❌ `trust-dns` is **unmaintained** (renamed to hickory)
- ❌ Security risk (no updates)
- ❌ Increases attack surface

**Current Usage**:
```bash
grep results:
- songbird-config: uses trust-dns-resolver
- songbird-universal: optional trust-dns-resolver
- Workspace: declares BOTH!
```

**Solution**: ✅ **Eliminate `trust-dns-resolver` completely**
- Replace all usage with `hickory-resolver`
- Est. savings: **~500 KB + removes unmaintained dep**

---

### 2. **REQWEST (Optional Unused?)** ⚠️ (Potential Bloat)

**Problem**: `reqwest` declared in workspace but marked REMOVED in many crates

```toml
# Workspace declares it:
reqwest = { version = "0.11", features = ["json"], default-features = false }

# But many crates say:
# REMOVED: reqwest (C dependency via native-tls → OpenSSL)
```

**Current Usage**:
- songbird-types: `optional = true` (reqwest feature)
- Most crates: **REMOVED** (commented out)

**Analysis**:
- ✅ Good: No native-tls/OpenSSL (Pure Rust maintained!)
- ⚠️ Question: Is reqwest actually used? If not, remove from workspace!

**Solution**: ✅ **Audit reqwest usage, potentially remove entirely**
- If used: Keep it (we use Pure Rust songbird-http-client anyway)
- If unused: Remove from workspace
- Est. savings: **~200-300 KB if removed**

---

### 3. **TOKIO "FULL" FEATURES** ⚠️ (Feature Bloat)

**Problem**: Using tokio with ALL features enabled

```toml
tokio = { version = "1.46", features = ["full"] }
```

**Impact**:
- ❌ Includes features we don't need
- ❌ "full" = all runtime features + sync + io + time + net + process + fs + signal + macros + parking_lot + test-util
- ❌ Larger binary size
- ❌ Longer compile times

**What We Actually Need**:
```toml
# Likely only need:
tokio = { version = "1.46", features = [
    "rt-multi-thread",  # Async runtime
    "net",              # Networking
    "io-util",          # IO utilities
    "macros",           # #[tokio::main]
    "sync",             # Channels, mutexes
    "time",             # Sleep, timeout
    "fs",               # File operations
] }
```

**Solution**: ✅ **Switch from "full" to explicit features**
- Audit actual tokio feature usage
- Remove unused features
- Est. savings: **~100-200 KB**

---

### 4. **DUPLICATE BASE64 LIBRARIES** ❌ (Cargo Tree Shows)

**Problem**: `base64 v0.21.7` used by multiple crates

```bash
Cargo tree output shows:
base64 v0.21.7
├── ron v0.8.1
│   └── config v0.14.1
```

**Analysis**:
- RON (Rusty Object Notation) brings in base64
- `config` crate uses RON
- Do we actually use RON format? Or just TOML/JSON?

**Solution**: ✅ **Investigate config crate usage**
- Check if RON support is actually used
- If not, disable RON feature in `config` crate
- Est. savings: **~50-100 KB**

---

### 5. **EXCESSIVE WORKSPACE DEPENDENCIES** ⚠️ (Organizational Bloat)

**Problem**: Workspace declares dependencies that may not all be used

**Questionable Entries**:
```toml
sys-info = "0.9"      # System info - needed?
num_cpus = "1.0"      # CPU count - tokio has this
wasi = "0.14"         # WASI target - are we targeting WASM?
criterion = { ... }   # Benchmark framework - should be dev-dep only!
tarpc = { ... }       # RPC framework - actually used?
```

**Solution**: ✅ **Audit workspace-level deps**
- Move `criterion` to `[workspace.dev-dependencies]`
- Check if `sys-info`, `num_cpus`, `wasi` actually used
- Remove unused workspace deps
- Est. savings: **~200-300 KB**

---

### 6. **CHRONO DEPENDENCY** ⚠️ (Potentially Replaceable)

**Problem**: `chrono` is a heavy time library

```toml
chrono = { version = "0.4", features = ["serde"] }
```

**Impact**:
- ❌ Large dependency (~300 KB)
- ❌ Complex API surface
- ❌ More than we need for timestamps

**Alternative**: 
- Use `std::time::SystemTime` for basic timestamps
- Use `time` crate (lighter alternative to chrono)

**Solution**: ✅ **Audit chrono usage**
- Check if we need full chrono functionality
- Consider replacing with `time` crate or std
- Est. savings: **~150-250 KB**

---

═══════════════════════════════════════════════════════════════════
📊 OPTIMIZATION ROADMAP
═══════════════════════════════════════════════════════════════════

## **Priority 1: ELIMINATE LEGACY DNS** (High Impact) 🔴

**Action**: Replace all `trust-dns-resolver` with `hickory-resolver`

**Files to Update**:
1. `Cargo.toml` (workspace.dependencies)
   - Remove: `trust-dns-resolver = "0.23"`
   - Keep only: `hickory-resolver = "0.24"`

2. `crates/songbird-config/Cargo.toml`
   - Replace: `trust-dns-resolver` → `hickory-resolver`

3. `crates/songbird-universal/Cargo.toml`
   - Replace optional dep: `trust-dns-resolver` → `hickory-resolver`

4. **Code Changes**:
   - `use trust_dns_resolver` → `use hickory_resolver`
   - API is nearly identical (hickory is fork of trust-dns)

**Estimated Savings**: **~500 KB + removes unmaintained dep**

**Estimated Effort**: **30 minutes** (simple find/replace + test)

---

## **Priority 2: TOKIO FEATURE REDUCTION** (Medium Impact) 🟡

**Action**: Switch from `features = ["full"]` to explicit list

**Investigation Steps**:
```bash
# Find all tokio feature usage
rg "tokio::" --type rust | grep -o "tokio::[a-z_]*" | sort | uniq

# Common needs:
# - tokio::spawn (rt-multi-thread)
# - tokio::net (net)
# - tokio::io (io-util)
# - tokio::time (time)
# - tokio::sync (sync)
# - tokio::fs (fs)
```

**Recommended Change**:
```toml
tokio = { version = "1.46", features = [
    "rt-multi-thread",
    "net",
    "io-util",
    "macros",
    "sync",
    "time",
    "fs",
    "signal",  # If we use Ctrl+C handling
] }
```

**Estimated Savings**: **~100-200 KB**

**Estimated Effort**: **1 hour** (audit usage + test)

---

## **Priority 3: AUDIT REQWEST USAGE** (Medium Impact) 🟡

**Action**: Determine if reqwest is actually used

**Investigation**:
```bash
# Check actual reqwest usage
rg "use reqwest" --type rust
rg "reqwest::" --type rust

# If found: Check if we can use songbird-http-client instead
# If not found: Remove from workspace entirely
```

**Decision Tree**:
- **If used**: Keep it (already using Pure Rust build)
- **If unused**: Remove from workspace.dependencies

**Estimated Savings**: **~200-300 KB** (if removed)

**Estimated Effort**: **30 minutes** (quick audit)

---

## **Priority 4: CONFIG CRATE RON FEATURE** (Low Impact) 🟢

**Action**: Disable RON support if unused

**Investigation**:
```bash
# Check if RON format is used
rg "\.ron" --type rust
rg "from_ron|to_ron" --type rust
```

**Change**:
```toml
config = { version = "0.14", default-features = false, features = ["toml", "json"] }
```

**Estimated Savings**: **~50-100 KB**

**Estimated Effort**: **15 minutes**

---

## **Priority 5: WORKSPACE DEPENDENCY CLEANUP** (Low Impact) 🟢

**Action**: Move dev-only deps and remove unused deps

**Changes**:
```toml
[workspace.dependencies]
# Remove if unused:
# sys-info = "0.9"  # Check actual usage
# num_cpus = "1.0"  # tokio provides this
# wasi = "0.14"     # Only if targeting WASM

[workspace.dev-dependencies]
# Move from dependencies to dev-dependencies:
criterion = { version = "0.5", features = ["html_reports"] }
```

**Estimated Savings**: **~200-300 KB**

**Estimated Effort**: **30 minutes**

---

## **Priority 6: CHRONO EVALUATION** (Low Impact) 🟢

**Action**: Evaluate if chrono can be replaced

**Investigation**:
```bash
# Check chrono usage complexity
rg "chrono::" --type rust | wc -l
rg "Utc::now|Local::now" --type rust
```

**Options**:
1. Keep chrono (if heavily used)
2. Replace with `time` crate (lighter)
3. Replace with `std::time` (zero-dep)

**Estimated Savings**: **~150-250 KB** (if replaced)

**Estimated Effort**: **2 hours** (if replacing)

---

═══════════════════════════════════════════════════════════════════
📈 PROJECTED IMPACT
═══════════════════════════════════════════════════════════════════

## **Total Potential Savings**:

| Optimization | Savings | Effort | Priority |
|--------------|---------|--------|----------|
| Eliminate trust-dns | ~500 KB | 30 min | 🔴 HIGH |
| Tokio features | ~150 KB | 1 hour | 🟡 MEDIUM |
| Audit reqwest | ~250 KB | 30 min | 🟡 MEDIUM |
| Config/RON | ~75 KB | 15 min | 🟢 LOW |
| Workspace cleanup | ~250 KB | 30 min | 🟢 LOW |
| Chrono eval | ~200 KB | 2 hours | 🟢 LOW |

**TOTAL SAVINGS**: **~1.4 MB (5-6% size reduction!)**

**TOTAL EFFORT**: **~5 hours**

**Current Binary Sizes**:
- x86_64: 27 MB → **~25.6 MB** (after optimization)
- ARM64: 25 MB → **~23.6 MB** (after optimization)

**After Stripping**:
- x86_64: ~11 MB → **~10 MB**
- ARM64: ~10 MB → **~9 MB**

---

═══════════════════════════════════════════════════════════════════
🎓 DEEP DEBT PHILOSOPHY
═══════════════════════════════════════════════════════════════════

## **What This Audit Reveals**:

### 1. **Dependency Debt is Real** ❌

**Problem**: Accumulated legacy dependencies over time
- `trust-dns` → `hickory` migration incomplete
- Workspace declares deps "just in case"
- Features enabled without audit ("full" tokio)

**Solution**: Systematic dependency hygiene
- Regular audits (quarterly)
- Explicit feature requirements
- Remove before adding

### 2. **Modern Idiomatic Rust = Lightweight** ✅

**Our Pure Rust Wins**:
- ✅ No OpenSSL/native-tls (removed reqwest native deps)
- ✅ Pure Rust TLS (songbird-tls)
- ✅ Pure Rust HTTP (songbird-http-client)

**Result**: Already eliminated C dependencies! 🎉

### 3. **Feature Bloat is Subtle** ⚠️

**Problem**: `features = ["full"]` is convenient but wasteful
- Includes code we never use
- Harder to reason about dependencies
- Larger binary, longer compile times

**Solution**: Explicit > Implicit
- Audit actual feature usage
- Declare only what's needed
- Document why each feature is required

---

═══════════════════════════════════════════════════════════════════
🚀 RECOMMENDED IMMEDIATE ACTIONS
═══════════════════════════════════════════════════════════════════

## **Start with Priority 1** (Quick Win!)

**Task**: Eliminate trust-dns-resolver (30 minutes)

**Steps**:
1. Remove `trust-dns-resolver = "0.23"` from workspace
2. Replace with `hickory-resolver` in crates
3. Update imports: `trust_dns_resolver` → `hickory_resolver`
4. Test + verify
5. Commit as "deep debt evolution"

**Impact**: 
- ✅ ~500 KB savings
- ✅ Removes unmaintained dependency
- ✅ Security improvement
- ✅ Easy win!

---

**Status**: ✅ **AUDIT COMPLETE**  
**Grade**: Current: B+ (good but can improve)  
**Target**: A++ (minimal, efficient dependencies)

🔍 **Dependency audit: Opportunities identified!** 🚀
