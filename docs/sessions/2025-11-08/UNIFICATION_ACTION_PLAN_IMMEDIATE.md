# 🎯 Songbird Unification - Immediate Action Plan
**Date**: November 8, 2025  
**Priority**: Implementation-Ready Tasks  
**Time Estimate**: 2-3 weeks

---

## ⚡ PHASE 1: QUICK WINS (1-2 Days)

### Task 1.1: Update Constant Imports (92 locations)
**Effort**: 30 minutes | **Risk**: LOW | **Impact**: HIGH

#### Automated Migration:
```bash
cd /home/eastgate/Development/ecoPrimals/songbird

# Backup first
git stash
git checkout -b feature/constant-imports-migration

# Find all files using old pattern
grep -r "use songbird_config::config::constants::" crates/ --include="*.rs" -l > /tmp/constant_files.txt

# Automated replacement
find crates -name "*.rs" -type f -exec sed -i \
    's/use songbird_config::config::constants::/use songbird_config::canonical::constants::/g' {} \;

# Verify changes
git diff --stat

# Build to verify
cargo build --workspace

# If successful:
git add -A
git commit -m "refactor: migrate config::constants to canonical::constants

- Updated 92+ import statements
- All imports now use canonical::constants
- Maintains backward compatibility
- Prepares for config::constants removal in v0.3.0"
```

#### Verification:
```bash
# Should return 0:
grep -r "use songbird_config::config::constants::" crates/ --include="*.rs" | wc -l

# Should return 92+:
grep -r "use songbird_config::canonical::constants::" crates/ --include="*.rs" | wc -l
```

---

### Task 1.2: Fix unwrap_data Deprecations (3 locations)
**Effort**: 15 minutes | **Risk**: LOW | **Impact**: LOW

#### Location:
`crates/songbird-types/src/response.rs`

#### Changes Needed:
```rust
// Find instances:
grep -n "unwrap_data\|into_result" crates/songbird-types/src/response.rs

// Update pattern from:
let data = response.unwrap_data();

// To:
let data = response.into_result().map_err(|e| {
    SongbirdError::Runtime {
        message: format!("Response extraction failed: {:?}", e),
        component: Some("response_handler".to_string()),
        debug_info: None,
    }
})?;
```

#### Command:
```bash
# Edit the file
$EDITOR crates/songbird-types/src/response.rs

# Verify builds
cargo build -p songbird-types
cargo test -p songbird-types

# Commit
git add crates/songbird-types/src/response.rs
git commit -m "refactor: replace unwrap_data with modern into_result pattern

- Migrated 3 deprecated unwrap_data calls
- Uses proper error handling with SongbirdError
- Prepares for unwrap_data removal"
```

---

### Task 1.3: Verify and Remove Deprecated Primal Modules
**Effort**: 1 hour | **Risk**: MEDIUM | **Impact**: MEDIUM

#### Step 1: Check for Active Usage
```bash
cd /home/eastgate/Development/ecoPrimals/songbird

# Check beardog usage (excluding the module itself)
grep -r "use.*beardog\|BearDogPrimal\|beardog::" crates/ --include="*.rs" | \
    grep -v "crates/songbird-primal-sdk/src/beardog.rs" | \
    grep -v "^//" | \
    wc -l

# Check toadstool usage
grep -r "use.*toadstool\|ToadstoolPrimal\|toadstool::" crates/ --include="*.rs" | \
    grep -v "crates/songbird-primal-sdk/src/toadstool.rs" | \
    grep -v "^//" | \
    wc -l

# Check squirrel usage
grep -r "use.*squirrel\|SquirrelPrimal\|squirrel::" crates/ --include="*.rs" | \
    grep -v "crates/songbird-primal-sdk/src/squirrel.rs" | \
    grep -v "^//" | \
    wc -l
```

#### Step 2a: If Usage Count = 0 (Safe to Remove)
```bash
# Move to archive
mkdir -p crates/songbird-primal-sdk/src/_archived_deprecated_primals_2025
git mv crates/songbird-primal-sdk/src/beardog.rs crates/songbird-primal-sdk/src/_archived_deprecated_primals_2025/
git mv crates/songbird-primal-sdk/src/toadstool.rs crates/songbird-primal-sdk/src/_archived_deprecated_primals_2025/
git mv crates/songbird-primal-sdk/src/squirrel.rs crates/songbird-primal-sdk/src/_archived_deprecated_primals_2025/

# Update lib.rs to remove module declarations
$EDITOR crates/songbird-primal-sdk/src/lib.rs
# Comment out or remove:
# pub mod beardog;
# pub mod toadstool;
# pub mod squirrel;

# Verify builds
cargo build -p songbird-primal-sdk
cargo test -p songbird-primal-sdk

# Commit
git add -A
git commit -m "refactor: archive deprecated hardcoded primal modules

- Moved beardog, toadstool, squirrel to _archived_deprecated_primals_2025/
- These violated zero-hardcoding philosophy
- Users should migrate to capability-based clients:
  * BearDogPrimal → security_capability
  * ToadstoolPrimal → compute_capability
  * SquirrelPrimal → ai_capability
- Modules archived for reference, will be removed in v0.3.0"
```

#### Step 2b: If Usage Count > 0 (Add Stronger Deprecation)
```bash
# Add compile-time warnings to each file
# At the top of beardog.rs, toadstool.rs, squirrel.rs:

#![deprecated(
    since = "0.2.0",
    note = "Use capability-based clients instead. This module will be removed in v0.3.0 (Q2 2026)"
)]

// Then commit
git add crates/songbird-primal-sdk/src/{beardog,toadstool,squirrel}.rs
git commit -m "deprecate: add compile-time warnings for hardcoded primal modules

- Added #![deprecated] attributes
- Users will see compiler warnings
- Migration path documented in module docs
- Removal planned for v0.3.0 (Q2 2026)"
```

---

### Task 1.4: Remove Q2 2026 Archive (if date passed)
**Effort**: 5 minutes | **Risk**: LOW | **Impact**: LOW

```bash
# Remove the archive directory
rm -rf crates/songbird-config/src/_archived_q2_2026/

# Update mod.rs or lib.rs to remove reference
$EDITOR crates/songbird-config/src/lib.rs
# Remove: pub mod _archived_q2_2026;

# Verify builds
cargo build -p songbird-config

# Commit
git add -A
git commit -m "cleanup: remove Q2 2026 archived modules

- Archive period expired
- All users migrated to canonical modules
- Reduces codebase maintenance burden"
```

---

## 📊 PHASE 2: CONFIG CONSOLIDATION AUDIT (3-5 Days)

### Task 2.1: Audit unified/* vs canonical/*
**Effort**: 4-6 hours | **Risk**: LOW | **Impact**: HIGH

#### Step 1: Generate Structure Comparison
```bash
cd /home/eastgate/Development/ecoPrimals/songbird

# List all modules in canonical
find crates/songbird-config/src/canonical -name "*.rs" | sort > /tmp/canonical_modules.txt

# List all modules in unified
find crates/songbird-config/src/unified -name "*.rs" | sort > /tmp/unified_modules.txt

# Compare side by side
paste /tmp/canonical_modules.txt /tmp/unified_modules.txt | column -t
```

#### Step 2: Check for Duplicate Types
```bash
# Script to find duplicate struct names
cat > /tmp/find_duplicate_configs.sh << 'EOF'
#!/bin/bash

echo "=== Comparing Config Structs in unified/ vs canonical/ ==="

# Extract struct names from canonical
grep -h "pub struct.*Config" crates/songbird-config/src/canonical/*.rs | \
    sed 's/.*struct \([A-Za-z]*\).*/\1/' | sort > /tmp/canonical_structs.txt

# Extract struct names from unified
grep -h "pub struct.*Config" crates/songbird-config/src/unified/*.rs | \
    sed 's/.*struct \([A-Za-z]*\).*/\1/' | sort > /tmp/unified_structs.txt

# Find duplicates
echo "Duplicate struct names found in both:"
comm -12 /tmp/canonical_structs.txt /tmp/unified_structs.txt

echo ""
echo "Only in canonical/:"
comm -23 /tmp/canonical_structs.txt /tmp/unified_structs.txt

echo ""
echo "Only in unified/:"
comm -13 /tmp/canonical_structs.txt /tmp/unified_structs.txt
EOF

chmod +x /tmp/find_duplicate_configs.sh
/tmp/find_duplicate_configs.sh
```

#### Step 3: Create Audit Document
```bash
cat > UNIFIED_VS_CANONICAL_AUDIT.md << 'EOF'
# Config Module Audit: unified/ vs canonical/

## Purpose
Determine if unified/ modules duplicate canonical/ or serve distinct purposes.

## Findings

### Duplicate Modules (Can be Deprecated)
- [ ] unified/network.rs vs canonical/network.rs
  - Status: 
  - Action: 
  - Impact: 

- [ ] unified/discovery.rs vs canonical/discovery.rs
  - Status:
  - Action:
  - Impact:

### Distinct Modules (Keep Both)
- [ ] Module name:
  - Reason to keep:
  - Relationship to canonical:

## Recommendations
1. Deprecate: [list modules]
2. Merge: [list modules]
3. Keep distinct: [list modules]

## Migration Plan
[Document plan after audit complete]
EOF

# Open for manual review
$EDITOR UNIFIED_VS_CANONICAL_AUDIT.md
```

---

### Task 2.2: Create Unified/* Deprecation Plan
**Effort**: 2-3 hours | **Risk**: MEDIUM | **Impact**: HIGH

Based on audit results, create migration plan:

```bash
cat > UNIFIED_CONFIG_MIGRATION_PLAN.md << 'EOF'
# unified/* Config Migration Plan

## Summary
[Fill after audit]

## Modules to Deprecate
1. unified/network.rs
   - Replacement: canonical/network.rs
   - Active users: [count]
   - Migration effort: [estimate]

2. [Add more]

## Modules to Keep
1. [If any have distinct purpose]

## Migration Timeline
- Week 1: Add deprecation notices
- Week 2: Update first 50% of users
- Week 3: Update remaining users
- Week 4: Verify and remove

## Migration Script
[Add automated migration commands]
EOF

$EDITOR UNIFIED_CONFIG_MIGRATION_PLAN.md
```

---

## 🔧 PHASE 3: EXECUTE CONFIG MIGRATION (Week 3)

### Task 3.1: Add Deprecation Notices
**Effort**: 30 minutes | **Risk**: LOW

```bash
# Template for adding deprecation to unified modules
cat > /tmp/deprecation_template.txt << 'EOF'
//! ⚠️ **DEPRECATED - USE CANONICAL INSTEAD** (November 8, 2025)
//!
//! This module has been superseded by `canonical::MODULE_NAME`.
//!
//! ## Migration
//! ```rust
//! // OLD:
//! use songbird_config::unified::MODULE_NAME::SomeConfig;
//!
//! // NEW:
//! use songbird_config::canonical::MODULE_NAME::SomeConfig;
//! ```
//!
//! **Removal Target**: v0.3.0 (Q2 2026)

#![deprecated(
    since = "0.2.0",
    note = "Use canonical::MODULE_NAME instead"
)]
EOF

# Manually add to top of deprecated unified/* files
```

---

### Task 3.2: Update Imports Across Codebase
**Effort**: 1-2 hours | **Risk**: MEDIUM

```bash
# Example for network config migration
find crates -name "*.rs" -type f -exec sed -i \
    's/use songbird_config::unified::network::/use songbird_config::canonical::network::/g' {} \;

# Verify compilation
cargo build --workspace

# Check for any remaining uses
grep -r "use songbird_config::unified::" crates/ --include="*.rs" | wc -l
```

---

## 📝 PHASE 4: DOCUMENTATION (Week 4)

### Task 4.1: Update Architecture Documentation

```bash
# Update ARCHITECTURE_OVERVIEW.md with final state
$EDITOR ARCHITECTURE_OVERVIEW.md

# Add section:
## Configuration Architecture (November 2025)

All configuration now consolidated in **canonical/** modules:
- `canonical::network` - Network configuration
- `canonical::security` - Security settings
- `canonical::performance` - Performance tuning
- `canonical::discovery` - Service discovery
- `canonical::observability` - Monitoring config

Deprecated modules (remove in v0.3.0):
- `config::*` - Legacy configuration (use canonical::*)
- `unified::*` - Intermediate consolidation (use canonical::*)
```

---

### Task 4.2: Create Best Practices Guide

```bash
cat > SONGBIRD_BEST_PRACTICES.md << 'EOF'
# Songbird Development Best Practices

## Configuration Management

### ✅ DO:
- Import from `canonical::*` modules
- Use `SongbirdResult<T>` for all fallible operations
- Use `songbird_types::traits::canonical` for traits
- Keep files under 2000 lines

### ❌ DON'T:
- Import from `config::*` (deprecated)
- Import from `unified::*` (being phased out)
- Use `.unwrap()` or `.expect()` in production code
- Hardcode primal names (use capability-based discovery)

## Import Patterns

```rust
// ✅ GOOD: Modern imports
use songbird_config::canonical::{
    network::CanonicalNetworkConfig,
    security::CanonicalSecurityConfig,
};
use songbird_types::{
    SongbirdError,
    SongbirdResult,
    traits::canonical::{Provider, ServiceProvider},
};

// ❌ BAD: Deprecated imports
use songbird_config::config::constants::*;
use songbird_config::unified::network::*;
```

## Error Handling

```rust
// ✅ GOOD: Rich error context
pub fn validate_config(config: &Config) -> SongbirdResult<()> {
    if config.port == 0 {
        return Err(SongbirdError::Configuration {
            message: "Port cannot be zero".to_string(),
            field: Some("port".to_string()),
            suggestion: Some("Use a valid port number (1-65535)".to_string()),
        });
    }
    Ok(())
}

// ❌ BAD: Panic on error
pub fn validate_config(config: &Config) {
    assert!(config.port != 0, "Port cannot be zero");
}
```

## File Organization

- Keep files under 2000 lines
- Use submodules for logical grouping
- One primary struct per file
- Tests in same file or `tests/` subdirectory

## Naming Conventions

- Config structs: `CanonicalXxxConfig` (in canonical/*)
- Error variants: Descriptive with context fields
- Traits: `XxxProvider` pattern
- Results: Use `SongbirdResult<T>`
EOF
```

---

## ✅ COMPLETION CHECKLIST

### Phase 1: Quick Wins ✅
- [ ] Migrate 92 constant imports to canonical
- [ ] Fix 3 unwrap_data calls in response.rs
- [ ] Archive or add warnings to deprecated primals
- [ ] Remove Q2 2026 archive (if applicable)
- [ ] Run full workspace build and test

### Phase 2: Config Audit ✅
- [ ] Generate module structure comparison
- [ ] Identify duplicate Config structs
- [ ] Create UNIFIED_VS_CANONICAL_AUDIT.md
- [ ] Complete manual review
- [ ] Document findings and recommendations

### Phase 3: Config Migration ✅
- [ ] Add deprecation notices to unified/*
- [ ] Update imports across codebase
- [ ] Verify compilation success
- [ ] Run full test suite
- [ ] Commit changes with detailed message

### Phase 4: Documentation ✅
- [ ] Update ARCHITECTURE_OVERVIEW.md
- [ ] Create SONGBIRD_BEST_PRACTICES.md
- [ ] Update UNIFIED_TRAITS_QUICKREF.md (if needed)
- [ ] Update README.md with current state
- [ ] Create metrics dashboard

---

## 📊 SUCCESS METRICS

Track these metrics before and after:

```bash
# Config struct count
grep -r "pub struct.*Config" crates/songbird-config/src/ --include="*.rs" | wc -l

# Deprecated import count
grep -r "use songbird_config::config::" crates/ --include="*.rs" | wc -l

# TODO/FIXME count
grep -r "TODO\|FIXME" crates/ --include="*.rs" | wc -l

# Deprecated module count
find crates -name "*.rs" -exec grep -l "#\[deprecated\]" {} \; | wc -l

# Files over 2000 lines (should be 0)
find crates -name "*.rs" -exec wc -l {} \; | awk '$1 > 2000' | wc -l
```

---

## 🚀 GETTING STARTED

```bash
# 1. Create feature branch
cd /home/eastgate/Development/ecoPrimals/songbird
git checkout -b feature/final-unification-cleanup

# 2. Start with Phase 1, Task 1.1 (easiest first)
# Follow commands above

# 3. Commit frequently
git add -p  # Stage changes interactively
git commit -m "refactor: [specific change]"

# 4. Test after each task
cargo build --workspace
cargo test --workspace
cargo clippy --workspace

# 5. Push when phase complete
git push origin feature/final-unification-cleanup
```

---

## ⚠️ SAFETY GUIDELINES

### Before Making Changes:
1. ✅ Create feature branch
2. ✅ Verify current code builds
3. ✅ Run tests to establish baseline

### During Changes:
1. ✅ Change one thing at a time
2. ✅ Commit after each logical change
3. ✅ Test after each commit

### After Changes:
1. ✅ Full workspace build
2. ✅ Full test suite
3. ✅ Clippy checks
4. ✅ Documentation review

### If Something Breaks:
```bash
# Revert last commit
git reset --hard HEAD~1

# Or revert specific file
git checkout HEAD -- path/to/file.rs

# Or start over
git checkout main
git branch -D feature/final-unification-cleanup
```

---

**Action Plan Created**: November 8, 2025  
**Estimated Timeline**: 2-3 weeks  
**Risk Level**: LOW  
**Confidence**: HIGH

**Start with Phase 1, Task 1.1** - It's the quickest win!

