# 🎯 Songbird Unification - Tactical Action Plan

**Date**: November 9, 2025  
**Purpose**: File-by-file tactical execution plan for unification  
**Companion to**: `CODEBASE_UNIFICATION_REPORT_NOV_2025.md`

---

## 🚀 Week 1: Audit & Quick Wins

### Day 1: Complete Inventory

**Task**: Create complete map of technical debt

**Scripts to Run**:
```bash
cd /home/eastgate/Development/ecoPrimals/songbird

# 1. Full config audit
./scripts/audit_configs.sh > reports/config_inventory_nov_9.txt

# 2. Legacy pattern audit
./scripts/detect_legacy.sh > reports/legacy_inventory_nov_9.txt

# 3. Deprecated items list
grep -r "#\[deprecated" --include="*.rs" crates/ -B 2 -A 5 > reports/deprecated_items_nov_9.txt

# 4. Error enum catalog
grep -r "pub enum.*Error" --include="*.rs" crates/ > reports/error_enums_nov_9.txt

# 5. Provider trait catalog
grep -r "pub trait.*Provider" --include="*.rs" crates/ > reports/provider_traits_nov_9.txt

# 6. Result type catalog
grep -r "pub type.*Result" --include="*.rs" crates/ > reports/result_types_nov_9.txt

# 7. Constants catalog
grep -r "pub const\|const [A-Z]" --include="*.rs" crates/ > reports/constants_nov_9.txt
```

**Expected Output**: 7 reports in `reports/` directory

---

### Day 2-3: Remove Deprecated Items (46 → 0)

**QUICK WIN**: All items already marked, just need removal

**Files to Modify** (from grep output):
```bash
# Get list of files with deprecated items
grep -r "#\[deprecated" --include="*.rs" crates/ -l | sort

# Expected files (partial list):
# - crates/songbird-config/src/lib.rs
# - crates/songbird-config/src/config/mod.rs
# - crates/songbird-discovery/src/lib.rs
# - crates/songbird-types/src/lib.rs
# - etc.
```

**Process for Each File**:
1. Find all usages of deprecated item: `rg "deprecated_item_name"`
2. Update usages to use new canonical version
3. Run tests: `cargo test --package <crate>`
4. Remove deprecated item
5. Run tests again to confirm
6. Commit with message: `refactor(crate): remove deprecated X, use canonical Y`

**Example**:
```rust
// crates/songbird-config/src/lib.rs

// STEP 1: Find usages
// rg "use songbird_config::config::" 

// STEP 2: Update all usages
// - use songbird_config::config::NetworkConfig;
// + use songbird_config::canonical::NetworkConfig;

// STEP 3: Remove deprecated module
// #[deprecated(since = "0.2.0", note = "Use canonical:: instead")]
// pub mod config;  // ← DELETE THIS

// STEP 4: Commit
// git commit -m "refactor(config): remove deprecated config module, all imports use canonical::"
```

**Validation**:
```bash
# Should return 0
grep -r "#\[deprecated" --include="*.rs" crates/ | wc -l
```

---

### Day 4-5: Consolidate Result Types (14 → 1)

**QUICK WIN**: Straightforward search and replace

**Target Pattern**:
```rust
// ❌ BEFORE: Custom result types
pub type ConfigResult<T> = Result<T, ConfigError>;
pub type DiscoveryResult<T> = Result<T, DiscoveryError>;
// ... etc

// ✅ AFTER: Unified result type
use songbird_types::SongbirdResult;
// All functions return SongbirdResult<T>
```

**Files to Modify**:
```bash
# Find all custom Result types
grep -r "pub type.*Result.*=.*Result<" --include="*.rs" crates/ -H
```

**Process**:
1. For each crate with custom Result type:
   - Add `use songbird_types::{SongbirdResult, SongbirdError};`
   - Replace `CustomResult<T>` with `SongbirdResult<T>`
   - Add `From<CustomError> for SongbirdError` if needed
   - Update all function signatures
   - Run tests
2. Delete custom Result type definition
3. Commit per crate

**Example Migration**:
```rust
// crates/songbird-discovery/src/lib.rs

// ❌ BEFORE
pub enum DiscoveryError { ... }
pub type DiscoveryResult<T> = Result<T, DiscoveryError>;

pub fn discover_service(name: &str) -> DiscoveryResult<ServiceInfo> {
    // ...
}

// ✅ AFTER
use songbird_types::{SongbirdResult, SongbirdError};

pub enum DiscoveryError { ... }  // Keep domain-specific error

impl From<DiscoveryError> for SongbirdError {
    fn from(e: DiscoveryError) -> Self {
        SongbirdError::Discovery {
            message: e.to_string(),
            backend: None,
            retry_strategy: None,
        }
    }
}

pub fn discover_service(name: &str) -> SongbirdResult<ServiceInfo> {
    // ...
}
```

**Validation**:
```bash
# Should return 1 (only SongbirdResult)
grep -r "pub type.*Result.*=.*Result<" --include="*.rs" crates/ | wc -l
```

---

## 🔥 Week 2: Discovery Crate Cleanup

### Target: Remove 98 Legacy Patterns from migration.rs

**File**: `crates/songbird-discovery/src/migration.rs` (654 lines)

**Status**: Scheduled for removal June 2026, but can deprecate now

**Action**:

1. **Add Deprecation Warning** (Day 1):
```rust
// Add to top of migration.rs
#![deprecated(
    since = "0.3.0",
    note = "Migration helper will be removed in June 2026. Use FederationAwareDiscovery directly. See docs/migration/federation_to_discovery.md"
)]
```

2. **Create Migration Guide** (Day 1-2):
```bash
# Create migration documentation
touch docs/migration/federation_to_discovery.md
```

Content:
```markdown
# Federation to Discovery Migration Guide

## Overview
The legacy `FederationMigrationHelper` is deprecated. Use `FederationAwareDiscovery` directly.

## Old Pattern (DEPRECATED)
```rust
use songbird_discovery::migration::FederationMigrationHelper;
let helper = FederationMigrationHelper::new(legacy_config);
let new_config = helper.migrate()?;
```

## New Pattern (RECOMMENDED)
```rust
use songbird_discovery::FederationAwareDiscovery;
let discovery = FederationAwareDiscovery::new(config).await?;
```

## Breaking Changes
- `LegacyFederationConfig` → Use `FederationDiscoveryConfig` directly
- `LegacySovereigntyLevel` → Use `SovereigntyLevel`
- `LegacyFederationMode` → No longer needed, auto-detected

## Timeline
- **Now - March 2026**: Deprecation warnings
- **March - June 2026**: Migration window with support
- **June 2026**: Removal of migration layer
```

3. **Test Coverage** (Day 3):
```bash
# Ensure no production code depends on migration.rs
rg "use.*migration::" crates/*/src --type rs
# Should only show test files and examples
```

4. **Mark All Migration Types** (Day 3-4):
```rust
// In migration.rs, mark all types:
#[deprecated(since = "0.3.0", note = "Use FederationAwareDiscovery directly")]
pub struct FederationMigrationHelper { ... }

#[deprecated(since = "0.3.0", note = "Use FederationDiscoveryConfig")]
pub struct LegacyFederationConfig { ... }

#[deprecated(since = "0.3.0", note = "Use SovereigntyLevel")]
pub enum LegacySovereigntyLevel { ... }

#[deprecated(since = "0.3.0", note = "Federation mode is auto-detected")]
pub enum LegacyFederationMode { ... }
```

5. **Update Examples** (Day 5):
```bash
# Find all examples using migration
find examples/ -name "*.rs" -exec grep -l "migration::" {} \;

# Update each to use modern pattern
```

**Validation**:
```bash
# No production code should use migration (except tests/examples)
rg "use.*migration::" crates/*/src --type rs | grep -v test | wc -l
# Should be 0
```

---

## 🏗️ Week 3: Configuration Module Consolidation

### Phase 3A: Migrate All Imports (config:: → canonical::)

**Target Files**: All files importing from `songbird_config::config::`

**Find Usages**:
```bash
# Find all imports of deprecated config module
rg "use songbird_config::config::" crates/ -l | sort > reports/config_imports.txt

# Count: likely 50-100 files
wc -l reports/config_imports.txt
```

**Process** (can parallelize across files):
```bash
# For each file in config_imports.txt:
for file in $(cat reports/config_imports.txt); do
    echo "Processing $file..."
    
    # Backup
    cp "$file" "$file.bak"
    
    # Replace imports
    sed -i 's/use songbird_config::config::/use songbird_config::canonical::/g' "$file"
    
    # Test
    cargo test --package $(echo $file | cut -d/ -f2) || {
        echo "FAILED: $file"
        mv "$file.bak" "$file"
        continue
    }
    
    # Clean up backup
    rm "$file.bak"
    
    echo "✅ $file"
done
```

**Validation**:
```bash
# Should return 0
rg "use songbird_config::config::" crates/ | wc -l
```

---

### Phase 3B: Identify Unique Code in unified/

**Target**: `crates/songbird-config/src/unified/` (12 files)

**Analysis Script**:
```bash
# For each file in unified/, check if equivalent exists in canonical/
cd crates/songbird-config/src

for unified_file in unified/*.rs; do
    base=$(basename $unified_file)
    canonical_file="canonical/$base"
    
    echo "=== Comparing $unified_file vs $canonical_file ==="
    
    if [ -f "$canonical_file" ]; then
        # Both exist - find unique functionality
        echo "Both exist - checking for unique code..."
        
        # Extract function signatures
        rg "pub fn" "$unified_file" | sort > /tmp/unified_fns.txt
        rg "pub fn" "$canonical_file" | sort > /tmp/canonical_fns.txt
        
        # Find functions only in unified
        comm -23 /tmp/unified_fns.txt /tmp/canonical_fns.txt > /tmp/unique_to_unified.txt
        
        if [ -s /tmp/unique_to_unified.txt ]; then
            echo "⚠️ UNIQUE FUNCTIONS IN UNIFIED:"
            cat /tmp/unique_to_unified.txt
        else
            echo "✅ No unique functionality - can delete"
        fi
    else
        echo "⚠️ Only in unified/ - need to move to canonical/"
    fi
    
    echo ""
done
```

**Expected Output**: Report showing which unified/ files to:
- **DELETE**: Pure duplicates of canonical/
- **MERGE**: Have unique functions to move to canonical/
- **MOVE**: Exist only in unified/, need to relocate to canonical/

---

### Phase 3C: Merge/Delete unified/ Module

**Process**:
1. For files with unique functionality:
   - Copy unique functions to corresponding canonical/ file
   - Update function docs to indicate source
   - Add tests
   - Verify all tests pass

2. For duplicate files:
   - Delete from unified/

3. Final cleanup:
   - Remove `pub mod unified;` from lib.rs
   - Delete empty unified/ directory
   - Update documentation

**Example Merge**:
```rust
// crates/songbird-config/src/canonical/performance.rs

// Add at end of file:
// === Merged from unified/performance.rs (Nov 2025) ===

/// Advanced performance tuning (formerly in unified module)
pub fn calculate_optimal_buffer_size(workload: WorkloadProfile) -> usize {
    // ... unique functionality moved from unified/
}
```

**Validation**:
```bash
# unified/ should not exist
[ ! -d crates/songbird-config/src/unified ] && echo "✅ unified/ removed"

# No imports of unified module
rg "use songbird_config::unified::" crates/ | wc -l
# Should be 0
```

---

## 🔧 Week 4: Error System Migration

### Target: 26 Error Enums → 1 (+2-3 with From)

**Strategy**:
1. **Keep**: Domain-specific errors that need special handling
   - `CliError` (already has From impl) ✅
   - `OrchestratorError` (if needed)
   - `SecurityError` (nested in SongbirdError) ✅

2. **Remove**: Duplicate error types that map directly to SongbirdError

**Phase 4A: Audit Error Enums**

```bash
# Create detailed error enum report
grep -r "pub enum.*Error" --include="*.rs" crates/ -B 5 -A 20 > reports/error_enums_detailed.txt

# Analyze each enum to determine:
# - Is it truly domain-specific?
# - Can it map to SongbirdError variants?
# - Does it have From impl already?
```

**Decision Matrix**:
| Crate | Error Type | Action | Reason |
|-------|-----------|--------|---------|
| songbird-types | `SongbirdError` | **KEEP** | Canonical |
| songbird-types | `SecurityError` | **KEEP** | Nested in SongbirdError |
| songbird-cli | `CliError` | **KEEP** | Has From impl ✅ |
| songbird-orchestrator | `OrchestratorError` | **EVALUATE** | May need From impl |
| songbird-discovery | `DiscoveryError` | **REMOVE** | Maps to SongbirdError::Discovery |
| songbird-registry | `RegistryError` | **REMOVE** | Maps to SongbirdError::Registry |
| songbird-universal | `UniversalError` | **REMOVE** | Maps to SongbirdError variants |
| ... | ... | ... | ... |

**Phase 4B: Migrate Each Error Type**

**Example: Remove DiscoveryError**

```rust
// crates/songbird-discovery/src/lib.rs

// ❌ BEFORE
#[derive(Debug, thiserror::Error)]
pub enum DiscoveryError {
    #[error("Service not found: {0}")]
    ServiceNotFound(String),
    
    #[error("Backend error: {0}")]
    BackendError(String),
    
    #[error("Timeout: {0}")]
    Timeout(String),
}

pub type DiscoveryResult<T> = Result<T, DiscoveryError>;

pub fn discover(name: &str) -> DiscoveryResult<Service> {
    Err(DiscoveryError::ServiceNotFound(name.to_string()))
}

// ✅ AFTER
use songbird_types::{SongbirdResult, SongbirdError};

pub fn discover(name: &str) -> SongbirdResult<Service> {
    Err(SongbirdError::Discovery {
        message: format!("Service not found: {}", name),
        backend: None,
        retry_strategy: Some("exponential_backoff".to_string()),
    })
}

// DiscoveryError removed entirely
```

**Validation Per Crate**:
```bash
# After migrating each crate, verify:
cargo test --package <crate>
cargo clippy --package <crate>

# Check error count
grep -r "pub enum.*Error" --include="*.rs" crates/<crate>/src | wc -l
# Should decrease
```

---

## 📊 Week 5-12: Systematic Cleanup

### Week 5: Remove Obvious Shims (150 instances)

**Target**: Shims with no dependencies

**Find Them**:
```bash
# Find files with "shim" in name or content
rg -i "shim" crates/ -l | sort > reports/shim_files.txt

# Analyze each for dependencies
for file in $(cat reports/shim_files.txt); do
    echo "=== $file ==="
    rg "use.*shim" crates/ --type rs | grep -v "$file"
done
```

**Process**:
1. Identify shims with no external dependencies
2. Replace shim calls with direct calls to canonical implementation
3. Delete shim file
4. Test

---

### Week 6-7: Provider Trait Consolidation

**Target**: 27 traits → 8-10 canonical traits

**Step 1: Map Trait Hierarchy**
```bash
# Extract all provider traits
grep -r "pub trait.*Provider" --include="*.rs" crates/ -A 10 > reports/provider_traits_full.txt

# Analyze overlaps and group by domain:
# - Service providers
# - Storage providers
# - Compute providers
# - Security providers
# - Observability providers
# - etc.
```

**Step 2: Design Canonical Hierarchy**
```rust
// crates/songbird-types/src/traits/canonical.rs

// Add new canonical traits:

/// Storage provider trait (consolidates 4-5 storage traits)
#[async_trait::async_trait]
pub trait CanonicalStorageProvider: Send + Sync {
    async fn store(&self, key: &str, value: Vec<u8>) -> SongbirdResult<()>;
    async fn retrieve(&self, key: &str) -> SongbirdResult<Vec<u8>>;
    async fn delete(&self, key: &str) -> SongbirdResult<()>;
    async fn list(&self, prefix: &str) -> SongbirdResult<Vec<String>>;
}

/// Compute provider trait (consolidates 3-4 compute traits)
#[async_trait::async_trait]
pub trait CanonicalComputeProvider: Send + Sync {
    async fn execute(&self, task: ComputeTask) -> SongbirdResult<ComputeResult>;
    async fn status(&self, task_id: &str) -> SongbirdResult<TaskStatus>;
    async fn cancel(&self, task_id: &str) -> SongbirdResult<()>;
}

// ... etc
```

**Step 3: Migrate Implementations**
- Update each provider implementation to use canonical trait
- Add backward compatibility if needed
- Remove old trait definition
- Test

---

### Week 8-10: Constants Consolidation

**Target**: 334 constants → ~50 organized constants

**Process**:
```bash
# Extract all constants with context
grep -r "pub const\|const [A-Z]" --include="*.rs" crates/ -B 2 -A 1 > reports/constants_with_context.txt

# Categorize by domain:
# - Network (ports, timeouts, URLs)
# - Security (key sizes, token expiry)
# - Performance (buffer sizes, pool sizes)
# - Discovery (intervals, retries)
# - etc.
```

**Target Structure**:
```rust
// crates/songbird-types/src/constants.rs

//! Canonical constants for Songbird
//!
//! All configurable values should use SafeEnv with these as defaults.

pub mod network {
    /// Default HTTP port
    pub const DEFAULT_HTTP_PORT: u16 = 8080;
    
    /// Default gRPC port
    pub const DEFAULT_GRPC_PORT: u16 = 9090;
    
    /// Default connection timeout (milliseconds)
    pub const DEFAULT_TIMEOUT_MS: u64 = 30_000;
    
    /// Maximum concurrent connections
    pub const MAX_CONNECTIONS: usize = 1_000;
}

pub mod discovery {
    /// Heartbeat interval (milliseconds)
    pub const HEARTBEAT_INTERVAL_MS: u64 = 5_000;
    
    /// Service discovery retry attempts
    pub const MAX_DISCOVERY_RETRIES: u32 = 3;
    
    /// Discovery timeout (milliseconds)
    pub const DISCOVERY_TIMEOUT_MS: u64 = 10_000;
}

pub mod security {
    /// Token expiry duration (hours)
    pub const TOKEN_EXPIRY_HOURS: u64 = 24;
    
    /// Maximum RSA key size (bits)
    pub const MAX_KEY_SIZE: usize = 4096;
    
    /// Minimum password length
    pub const MIN_PASSWORD_LENGTH: usize = 12;
}

pub mod performance {
    /// Default buffer pool size
    pub const DEFAULT_BUFFER_POOL_SIZE: usize = 1024;
    
    /// Maximum memory per request (MB)
    pub const MAX_MEMORY_MB: u64 = 256;
    
    /// Worker thread count (0 = auto-detect)
    pub const DEFAULT_WORKERS: usize = 0;
}

// ... etc
```

**Migration**:
1. Move each constant to appropriate module in constants.rs
2. Update imports: `use songbird_types::constants::network::DEFAULT_HTTP_PORT;`
3. Remove old constant definition
4. Test

---

### Week 11-12: Final Cleanup & Polish

**Tasks**:
- [ ] Delete empty legacy modules
- [ ] Update all documentation
- [ ] Update examples
- [ ] Run full test suite
- [ ] Generate metrics report
- [ ] Update CHANGELOG
- [ ] Create migration guide for external users

**Final Validation**:
```bash
# Run all metrics
./scripts/unification_metrics.sh

# Should show:
# - Configs: < 50
# - Legacy: 0
# - Errors: 1 (+2-3)
# - Traits: 8-10
# - Deprecated: 0
# - Constants: < 50
# - Results: 1
# - Files > 2000: 0
```

---

## 📁 File-Level Work Assignments

### Priority 1 Files (Week 1-2)

| File | Issue | Action | Effort |
|------|-------|--------|--------|
| `crates/songbird-config/src/lib.rs` | Deprecated module | Remove after migration | 2h |
| `crates/songbird-config/src/config/mod.rs` | Entire module deprecated | Delete | 1h |
| `crates/songbird-discovery/src/migration.rs` | 654 lines legacy | Deprecate | 3h |
| All files with `#[deprecated]` | 46 items | Remove | 8h |
| All files with custom Result types | 14 types | Convert to SongbirdResult | 6h |

**Total Week 1-2 Effort**: ~20 hours

---

### Priority 2 Files (Week 3-4)

| File | Issue | Action | Effort |
|------|-------|--------|--------|
| `crates/songbird-config/src/unified/*.rs` | 12 duplicate files | Merge or delete | 12h |
| All files importing `config::` | 50-100 files | Update imports | 10h |
| `crates/songbird-universal/src/adapters/ai.rs` | 29 legacy patterns | Modernize | 4h |
| Test files with legacy patterns | 120 instances | Update or delete | 8h |

**Total Week 3-4 Effort**: ~34 hours

---

### Priority 3 Files (Week 5-8)

| File | Issue | Action | Effort |
|------|-------|--------|--------|
| All error enum files | 23 files (after keeping 3) | Migrate to SongbirdError | 20h |
| All provider trait files | 17 files (after keeping 10) | Consolidate | 16h |
| Shim files | 150 instances | Remove | 12h |

**Total Week 5-8 Effort**: ~48 hours

---

### Priority 4 Files (Week 9-12)

| File | Issue | Action | Effort |
|------|-------|--------|--------|
| `crates/songbird-types/src/constants.rs` | Reorganize 334 constants | Consolidate | 16h |
| Documentation files | Update for new structure | Rewrite | 12h |
| Example files | Update patterns | Modernize | 8h |

**Total Week 9-12 Effort**: ~36 hours

---

## ⚡ Parallelization Opportunities

**Can Work in Parallel**:
1. **Remove deprecated** (Week 1) - independent per crate
2. **Result type migration** (Week 1) - independent per crate
3. **Import updates** (Week 3) - independent per file (use script)
4. **Error migrations** (Week 5-6) - independent per crate
5. **Trait consolidation** (Week 6-7) - can design traits in parallel with migrations

**Must Be Sequential**:
1. Config module consolidation (dependencies)
2. Final cleanup (after all migrations)

---

## 🎯 Success Tracking

**Daily Check-in**:
```bash
# Run metrics each day
./scripts/unification_metrics.sh > reports/metrics_$(date +%Y%m%d).txt

# Compare to yesterday
diff reports/metrics_$(date -d "yesterday" +%Y%m%d).txt reports/metrics_$(date +%Y%m%d).txt
```

**Weekly Report**:
```bash
# Generate weekly progress report
echo "=== Week X Progress ==="
echo "Start metrics:"
cat reports/metrics_week_x_start.txt
echo ""
echo "End metrics:"
./scripts/unification_metrics.sh
echo ""
echo "Files modified: $(git log --since="1 week ago" --name-only --pretty=format: | sort -u | wc -l)"
echo "Commits: $(git log --since="1 week ago" --oneline | wc -l)"
```

---

## 🔗 Tools & Scripts

### Create Metrics Script

```bash
#!/bin/bash
# scripts/unification_metrics.sh

echo "=== Songbird Unification Metrics ==="
echo "Date: $(date)"
echo ""

echo "1. Config Structs (target: < 50):"
grep -r "struct.*Config" --include="*.rs" crates/*/src 2>/dev/null | wc -l

echo "2. Legacy Patterns (target: 0):"
grep -ri "legacy\|shim\|wrapper" --include="*.rs" crates/*/src 2>/dev/null | wc -l

echo "3. Deprecated Items (target: 0):"
grep -r "#\[deprecated" --include="*.rs" crates/*/src 2>/dev/null | wc -l

echo "4. Error Enums (target: 1 +2-3):"
grep -r "pub enum.*Error" --include="*.rs" crates/ 2>/dev/null | wc -l

echo "5. Provider Traits (target: 8-10):"
grep -r "pub trait.*Provider" --include="*.rs" crates/*/src 2>/dev/null | grep -v test | wc -l

echo "6. Result Types (target: 1):"
grep -r "pub type.*Result" --include="*.rs" crates/ 2>/dev/null | wc -l

echo "7. Constants (target: < 50):"
grep -r "const " --include="*.rs" crates/ 2>/dev/null | grep -v "//" | wc -l

echo "8. Files > 2000 lines (target: 0):"
find crates/ -name "*.rs" -exec wc -l {} + 2>/dev/null | awk '$1 > 2000' | wc -l

echo ""
echo "=== End Metrics ==="
```

---

**Next Steps**: 
1. Review tactical plan
2. Create `scripts/unification_metrics.sh`
3. Run baseline metrics
4. Start Day 1 tasks

**Status**: 🟢 **READY FOR EXECUTION**

