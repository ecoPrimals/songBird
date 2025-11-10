# 🔧 Songbird Config Consolidation Action Plan
**Date**: November 10, 2025  
**Status**: 🎯 **READY FOR EXECUTION**  
**Goal**: 678 configs → ~120 configs (82% reduction)

---

## 📊 **DUPLICATE ANALYSIS** (High-Priority Consolidation)

### **Identified Duplicates** (Must Consolidate)

#### 1. **NetworkConfig** - 4+ Definitions 🔴 CRITICAL
```
DUPLICATES FOUND:
├─ songbird-canonical/src/config/environment.rs:107 [MIGRATE]
├─ songbird-config/src/config/hardcoded_elimination.rs:51 [REMOVE]
├─ songbird-config/src/config/mod.rs:237 [REMOVE]
└─ songbird-config/src/canonical/network.rs:125 (CanonicalNetworkConfig) [CANONICAL] ✅

ACTION: Keep CanonicalNetworkConfig, remove others, update all imports to:
  use songbird_config::canonical::network::CanonicalNetworkConfig;
```

#### 2. **SecurityConfig** - 4+ Definitions 🔴 CRITICAL
```
DUPLICATES FOUND:
├─ songbird-cli/src/cli/commands/firewall.rs:33 [MIGRATE or REMOVE]
├─ songbird-cli/src/cli/commands/quick.rs:36 [MIGRATE or REMOVE]
├─ songbird-config/src/config/hardcoded_elimination.rs:42 [REMOVE]
├─ songbird-config/src/config/mod.rs:301 [REMOVE]
└─ songbird-config/src/canonical/security.rs:148 (UniversalSecurityConfig) [CANONICAL] ✅

ACTION: Keep UniversalSecurityConfig, remove duplicates
  - CLI configs might be simplified wrappers → evaluate if needed
```

#### 3. **PerformanceConfig** - 5+ Definitions 🔴 CRITICAL
```
DUPLICATES FOUND:
├─ songbird-canonical/src/config/performance.rs:10 [MIGRATE]
├─ songbird-config/src/canonical/performance.rs:30 [CANONICAL] ✅
├─ songbird-config/src/config/hardcoded_elimination.rs:73 [REMOVE]
├─ songbird-config/src/lib.rs:155 [REMOVE]
└─ songbird-config/src/performance.rs:15 (PerformanceConfigCache) [EVALUATE]

ACTION: Keep songbird-config/src/canonical/performance.rs:30
  - Remove others except PerformanceConfigCache (may be domain-specific)
```

#### 4. **DiscoveryConfig** - 3+ Definitions 🟡
```
DUPLICATES FOUND:
├─ songbird-config/src/canonical/discovery.rs:29 [CANONICAL] ✅
├─ songbird-config/src/config/mod.rs:460 [REMOVE]
└─ Multiple specialized: ServiceDiscoveryConfig, CapabilityDiscoveryConfig, NetworkDiscoveryConfig

ACTION: Keep canonical DiscoveryConfig
  - Specialized configs may be domain-specific → evaluate individually
```

#### 5. **ServiceDiscoveryConfig** - 2+ Definitions 🟡
```
DUPLICATES FOUND:
├─ songbird-canonical/src/config/orchestration.rs:25 [MIGRATE]
└─ songbird-config/src/canonical/discovery.rs:98 [CANONICAL] ✅

ACTION: Consolidate to canonical/discovery.rs:98
```

#### 6. **LoadBalancingConfig** - 2+ Definitions 🟡
```
DUPLICATES FOUND:
├─ songbird-canonical/src/config/orchestration.rs:38
└─ songbird-config/src/canonical/network.rs:608

ACTION: Evaluate - may be different purposes (orchestration vs network)
  - If same: consolidate
  - If different: rename for clarity
```

#### 7. **CircuitBreakerConfig** - 2+ Definitions 🟡
```
DUPLICATES FOUND:
├─ songbird-config/src/canonical/network.rs:788
└─ songbird-config/src/canonical/resilience.rs:8 [CANONICAL] ✅

ACTION: Keep resilience version, remove network duplicate
```

#### 8. **RateLimitingConfig** - 2+ Definitions 🟡
```
DUPLICATES FOUND:
├─ songbird-config/src/canonical/network.rs:632
└─ songbird-config/src/canonical/resilience.rs:226 [CANONICAL] ✅

ACTION: Keep resilience version, remove network duplicate
```

#### 9. **LoggingConfig** - 3+ Definitions 🟡
```
DUPLICATES FOUND:
├─ songbird-cli/src/cli/commands/firewall.rs:50
├─ songbird-config/src/canonical/environment.rs:59
└─ songbird-config/src/canonical/observability.rs:54 [CANONICAL] ✅

ACTION: Keep observability version, remove others
```

#### 10. **HealthCheckConfig** - 2+ Definitions 🟡
```
DUPLICATES FOUND:
├─ songbird-config/src/canonical/primals.rs:242
└─ songbird-config/src/canonical/resilience.rs:406

ACTION: Evaluate purpose difference
  - Primal-specific vs general resilience
  - May need both with clear naming
```

---

## 📋 **CONSOLIDATION STRATEGY**

### **Phase 1: Critical Duplicates** (Week 1, 8-12 hours)

**Priority Order**:
1. NetworkConfig (4 → 1)
2. SecurityConfig (4 → 1)
3. PerformanceConfig (5 → 1-2)
4. DiscoveryConfig (3 → 1)

**Process per config**:
```bash
# Example: NetworkConfig consolidation

# Step 1: Identify canonical version
CANONICAL="songbird-config/src/canonical/network.rs:125 (CanonicalNetworkConfig)"

# Step 2: Find all usages of duplicates
grep -r "use.*NetworkConfig" crates/ --include="*.rs" | grep -v canonical

# Step 3: Update imports
sed -i 's/use.*config::NetworkConfig/use songbird_config::canonical::network::CanonicalNetworkConfig/g' <file>

# Step 4: Remove duplicate definitions
# Delete the struct definitions in:
# - songbird-canonical/src/config/environment.rs:107
# - songbird-config/src/config/hardcoded_elimination.rs:51
# - songbird-config/src/config/mod.rs:237

# Step 5: Validate
cargo check --workspace
cargo test --package <affected-packages>
```

### **Phase 2: Medium Priority** (Week 2, 8-10 hours)

5. ServiceDiscoveryConfig (2 → 1)
6. LoadBalancingConfig (2 → 1, or rename if different)
7. CircuitBreakerConfig (2 → 1)
8. RateLimitingConfig (2 → 1)
9. LoggingConfig (3 → 1)
10. HealthCheckConfig (2 → 1-2, evaluate)

### **Phase 3: Domain-Specific Review** (Week 3, 10-15 hours)

**Configs to Evaluate** (may be legitimately different):
- GamingNetworkConfig vs CanonicalNetworkConfig
- GamingPerformanceConfig vs PerformanceConfig
- EnvironmentSecurityConfig vs UniversalSecurityConfig
- ServiceConfig (appears in 2+ places)
- All *ProviderConfig variants

**Decision criteria**:
- Same fields? → **DUPLICATE** → Remove
- Different purpose? → **DOMAIN** → Keep with better naming
- Wrapper/subset? → **REMOVE** → Use canonical directly

---

## 🎯 **AUTOMATED CONSOLIDATION SCRIPT**

```bash
#!/bin/bash
# scripts/unification/consolidate_one_config.sh

CONFIG_NAME=$1
CANONICAL_PATH=$2

echo "🔧 Consolidating $CONFIG_NAME → $CANONICAL_PATH"
echo ""

# Find all definitions
echo "📍 Finding all definitions of $CONFIG_NAME..."
grep -rn "pub struct $CONFIG_NAME" crates/ --include="*.rs" | grep -v test

echo ""
echo "📊 Finding all usages..."
grep -r "use.*$CONFIG_NAME" crates/ --include="*.rs" | wc -l

echo ""
echo "⚠️  Manual steps required:"
echo "1. Review all definitions above"
echo "2. Choose canonical: $CANONICAL_PATH"
echo "3. Update imports in files using old definitions"
echo "4. Remove duplicate struct definitions"
echo "5. Run: cargo check --workspace"
echo "6. Run: cargo test --workspace"
echo ""
echo "📝 Example import update:"
echo "   OLD: use some_crate::$CONFIG_NAME;"
echo "   NEW: use $(echo $CANONICAL_PATH | cut -d: -f1 | sed 's|/|::|g' | sed 's|crates/||' | sed 's|src/||' | sed 's|.rs||')::$CONFIG_NAME;"
```

---

## 📈 **EXPECTED RESULTS**

### **Quantified Goals**

| Phase | Configs Before | Configs After | Reduction | Files Affected |
|-------|----------------|---------------|-----------|----------------|
| **Phase 1: Critical** | 678 | ~600 | -78 (-12%) | ~50 files |
| **Phase 2: Medium** | ~600 | ~450 | -150 (-25%) | ~40 files |
| **Phase 3: Domain Review** | ~450 | ~120 | -330 (-73%) | ~100 files |
| **TOTAL** | **678** | **~120** | **-558 (-82%)** | **~190 files** |

### **Time Investment**

- Phase 1: 8-12 hours (Critical duplicates)
- Phase 2: 8-10 hours (Medium priority)
- Phase 3: 10-15 hours (Domain review)
- **TOTAL: 26-37 hours** (~1 week focused work, or 3-4 weeks part-time)

---

## ✅ **VALIDATION CHECKLIST**

After each phase:

- [ ] `cargo check --workspace` passes
- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --workspace` passes (warnings OK)
- [ ] All imports updated (grep for old patterns)
- [ ] Duplicate definitions removed
- [ ] Documentation updated
- [ ] Run `./scripts/unification/track_progress.sh`

---

## 🎯 **QUICK START**

### **Start with NetworkConfig** (Highest Impact)

```bash
# 1. Identify all NetworkConfig definitions
grep -rn "pub struct.*NetworkConfig" crates/ --include="*.rs" | grep -v test

# 2. Choose canonical: CanonicalNetworkConfig in songbird-config/src/canonical/network.rs
CANONICAL="songbird_config::canonical::network::CanonicalNetworkConfig"

# 3. Find all usages
grep -r "use.*NetworkConfig" crates/ --include="*.rs" > network_config_usages.txt

# 4. Update imports (manually or with sed)
# For each file in network_config_usages.txt:
# - Replace import path with canonical
# - Test compilation

# 5. Remove duplicate definitions
# Delete the struct definitions in non-canonical files

# 6. Validate
cargo check --workspace
cargo test --package songbird-config
cargo test --package songbird-canonical

# 7. Track progress
./scripts/unification/track_progress.sh
```

---

## 📚 **REFERENCE PATTERNS**

### **Import Update Pattern**
```rust
// ❌ BEFORE (using duplicate)
use songbird_config::config::NetworkConfig;
use crate::config::NetworkConfig;

// ✅ AFTER (using canonical)
use songbird_config::canonical::network::CanonicalNetworkConfig;
// Optionally alias if name is long:
use songbird_config::canonical::network::CanonicalNetworkConfig as NetworkConfig;
```

### **Struct Removal Pattern**
```rust
// ❌ REMOVE THIS (duplicate definition)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub bind_address: SocketAddr,
    pub port: u16,
    // ... fields
}

// Replace with re-export if needed for backward compat:
pub use songbird_config::canonical::network::CanonicalNetworkConfig as NetworkConfig;
```

---

## 🚨 **COMMON PITFALLS**

### **1. Different Configs with Same Name**
**Problem**: Two configs called "NetworkConfig" but with different fields
**Solution**: 
- Rename one to be more specific (e.g., `GamingNetworkConfig`)
- Or consolidate if fields overlap significantly

### **2. Breaking External APIs**
**Problem**: Removing a public config breaks downstream users
**Solution**:
```rust
// Keep as type alias for backward compatibility
#[deprecated(since = "0.2.0", note = "Use CanonicalNetworkConfig")]
pub type NetworkConfig = CanonicalNetworkConfig;
```

### **3. Test Failures**
**Problem**: Tests break after consolidation
**Solution**:
- Update test imports
- Use test fixtures from canonical location
- Ensure test configs match canonical structure

---

## 📊 **PROGRESS TRACKING**

```bash
# Check config count weekly
grep -r "pub struct.*Config" crates --include="*.rs" | wc -l

# Track duplicates eliminated
# Week 1: 678 configs
# Week 2: ~600 configs (-12%)
# Week 3: ~450 configs (-34%)
# Week 4: ~120 configs (-82%) ✅ TARGET
```

---

## 🎉 **SUCCESS CRITERIA**

- [ ] **678 → ~120 configs** (82% reduction achieved)
- [ ] **Zero duplicate config definitions** (same name, similar fields)
- [ ] **All imports use canonical paths**
- [ ] **Workspace builds successfully** (`cargo check --workspace`)
- [ ] **All tests pass** (`cargo test --workspace`)
- [ ] **Documentation updated** (import paths, examples)
- [ ] **CI passes** (if configured)

---

**Status**: 🎯 **READY TO EXECUTE**  
**Priority**: **HIGH** - Highest impact unification opportunity  
**Timeline**: 3-4 weeks part-time, 1 week focused  
**Confidence**: **HIGH** - Clear duplicates identified

Start with NetworkConfig, validate the process, then systematically work through the others. Track progress weekly with `track_progress.sh`.

Good luck! 🚀

