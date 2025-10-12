# 🔧 CONFIG CONSOLIDATION PLAN

**Date**: October 2, 2025  
**Priority**: CRITICAL  
**Current State**: 848 config structs  
**Target**: <200 canonical configs  
**Reduction**: ~650 configs (77% reduction)

---

## 📊 AUDIT RESULTS

### Discovery Configs: 48 STRUCTS FOUND ❌

**Canonical Location**: `songbird-types/src/config/consolidated_canonical/discovery.rs`

**Found Duplicates**:
1. `DiscoveryConfig` (19 instances!)
2. `ServiceDiscoveryConfig` (7 instances)
3. `NetworkDiscoveryConfig` (5 instances)
4. `CanonicalDiscoveryConfig` (3 instances)
5. `UnifiedDiscoveryConfig` (2 instances)
6. Plus 12 more specialized discovery configs

**Consolidation Strategy**:
```rust
// KEEP: songbird-types/src/config/consolidated_canonical/discovery.rs
pub struct CanonicalDiscoveryConfig {
    pub network: CanonicalNetworkDiscoveryConfig,
    pub service: CanonicalServiceDiscoveryConfig,
    pub capability: CapabilityDiscoveryConfig,
    pub federation: FederationDiscoveryConfig,
}

// REMOVE: All other 44 DiscoveryConfig definitions
// MIGRATE: Update imports to use canonical
```

---

## 🎯 CONSOLIDATION PHASES

### Phase 1: Discovery Configs (Week 1)
- **Target**: 48 → 4 configs
- **Effort**: 2 days
- **Actions**:
  1. Audit all 48 DiscoveryConfig definitions
  2. Identify unique fields from each
  3. Merge into canonical hierarchy
  4. Update all imports
  5. Remove deprecated definitions

### Phase 2: Network Configs (Week 1-2)
- **Estimate**: ~60 network-related configs
- **Target**: 60 → 6 configs
- **Actions**: Similar to Phase 1

### Phase 3: Security Configs (Week 2)
- **Estimate**: ~40 security configs
- **Target**: 40 → 5 configs

### Phase 4: Performance Configs (Week 2)
- **Estimate**: ~30 performance configs
- **Target**: 30 → 4 configs

### Phase 5: Federation Configs (Week 3)
- **Estimate**: ~25 federation configs
- **Target**: 25 → 3 configs

### Phase 6: API/Service Configs (Week 3)
- **Estimate**: ~50 API/service configs
- **Target**: 50 → 8 configs

### Phase 7: Gaming/Specialized Configs (Week 3-4)
- **Estimate**: ~40 specialized configs
- **Target**: 40 → 6 configs

### Phase 8: Cleanup & Validation (Week 4)
- Remove all deprecated config definitions
- Update all tests
- Validate workspace compiles
- Update documentation

---

## 📋 DISCOVERY CONFIG MIGRATION CHECKLIST

### Files to UPDATE (keep, consolidate into):
- ✅ `songbird-types/src/config/consolidated_canonical/discovery.rs` (canonical)
- ✅ `songbird-types/src/config/discovery.rs` (re-export canonical)

### Files to REMOVE (deprecated duplicates):
- [ ] `songbird-network-federation/src/network/mod.rs:283` - DiscoveryConfig
- [ ] `songbird-config/src/config/mod.rs:371` - DiscoveryConfig
- [ ] `songbird-federation/src/discovery/production_discovery.rs:34` - DiscoveryConfig
- [ ] `songbird-federation/src/types.rs:335` - DiscoveryConfig
- [ ] `songbird-core/src/traits/discovery.rs:219` - DiscoveryConfig
- [ ] `songbird-core/src/basic_iot/mod.rs:30` - DiscoveryConfig
- [ ] `songbird-discovery/src/traits/discovery.rs:243` - DiscoveryConfig
- [ ] `songbird-universal/src/discovery.rs:27` - DiscoveryConfig
- [ ] `songbird-universal/src/agnostic_service_discovery.rs:122` - DiscoveryConfig
- [ ] `songbird-universal/src/infant_discovery.rs:132` - DiscoveryConfig
- [ ] `songbird-universal/src/capabilities.rs:115` - DiscoveryConfig
- [ ] `songbird-network/src/network/discovery/types.rs:11` - DiscoveryConfig
- [ ] `songbird-network/src/network/gaming/production_lan/config.rs:23` - DiscoveryConfig
- [ ] `songbird-universal-primals/src/adaptive_discovery.rs:761` - DiscoveryConfig
- [ ] `songbird-universal-primals/src/discovery/universal_discovery.rs:54` - DiscoveryConfig
- [ ] `songbird-universal-primals/src/discovery/types.rs:92` - DiscoveryConfig

... (plus 32 more specialized configs)

---

## 🔧 MIGRATION SCRIPT TEMPLATE

```python
# scripts/consolidate_discovery_configs.py

import os
import re
from pathlib import Path

CANONICAL_PATH = "crates/songbird-types/src/config/consolidated_canonical/discovery.rs"

# Map of deprecated configs to canonical
MIGRATION_MAP = {
    "DiscoveryConfig": "CanonicalDiscoveryConfig",
    "ServiceDiscoveryConfig": "CanonicalServiceDiscoveryConfig",
    "NetworkDiscoveryConfig": "CanonicalNetworkDiscoveryConfig",
}

# Files to remove entirely (struct definition only)
REMOVE_DEFINITIONS = [
    ("crates/songbird-network-federation/src/network/mod.rs", 283),
    ("crates/songbird-config/src/config/mod.rs", 371),
    # ... add all others
]

def update_imports(file_path):
    """Update imports to use canonical config"""
    with open(file_path, 'r') as f:
        content = f.read()
    
    # Replace old imports
    content = re.sub(
        r'use crate::.*::DiscoveryConfig',
        'use songbird_types::config::consolidated_canonical::CanonicalDiscoveryConfig',
        content
    )
    
    # Save
    with open(file_path, 'w') as f:
        f.write(content)

def main():
    # Phase 1: Update all imports
    for rust_file in Path("crates").rglob("*.rs"):
        update_imports(rust_file)
    
    # Phase 2: Comment out deprecated definitions
    for file_path, line_num in REMOVE_DEFINITIONS:
        comment_out_struct(file_path, line_num)
    
    # Phase 3: Run cargo check
    os.system("cargo check --workspace")
    
    # Phase 4: Remove commented definitions
    # (only after validation)

if __name__ == "__main__":
    main()
```

---

## ✅ SUCCESS CRITERIA

### Quantitative:
- [ ] Total config structs: 848 → <200
- [ ] DiscoveryConfig variants: 48 → 4
- [ ] All workspace compiles with 0 errors
- [ ] <50 deprecation warnings (expected during transition)

### Qualitative:
- [ ] Single source of truth for each config domain
- [ ] Clear canonical locations documented
- [ ] All imports use canonical paths
- [ ] Tests pass
- [ ] Documentation updated

---

## 🚀 IMMEDIATE NEXT STEPS

1. **This Session**: Start Discovery Config consolidation
   - Merge first 5 simple DiscoveryConfig duplicates
   - Update imports in affected files
   - Validate compilation

2. **Tomorrow**: Continue Discovery Config work
   - Complete remaining 43 discovery configs
   - Run full test suite

3. **This Week**: Move to Network Config consolidation

---

## 📝 NOTES

- Keep consolidation atomic (one config type at a time)
- Always validate compilation after each merge
- Document any unique fields that need to be preserved
- Use feature flags if backward compat needed
- Create migration guide for external users

---

**Status**: READY TO EXECUTE  
**Owner**: Development Team  
**Timeline**: 3-4 weeks for full consolidation 