# 🎯 Week 3 Migration Targets

**Generated**: October 12, 2025  
**Total Scope**: 64 vendor files + 56 primal files + 137 numeric files  
**Strategy**: High-impact first, then systematic cleanup  

---

## 📋 **VENDOR HARDCODING** (64 files)

### **🔥 Critical Priority** (Production Code - Target First)

```
1. crates/songbird-discovery/src/abstraction/adapters/kubernetes_adapter.rs
2. crates/songbird-discovery/src/abstraction/adapters/consul_adapter.rs
3. crates/songbird-config/src/config/constants.rs
4. crates/songbird-primal-sdk/src/adaptive_discovery.rs
5. crates/songbird-universal/src/service_discovery.rs
6. crates/songbird-discovery/src/discovery/backends/service_discovery.rs
7. crates/songbird-discovery/src/discovery/backends/container_orchestration.rs
```

**Impact**: These files directly implement vendor-specific logic. Replacing with capability-based discovery has maximum effect.

### **⚡ High Priority** (Integration & CLI Code)

```
8-22. crates/songbird-cli/src/cli/commands/*.rs (15 files)
    - migrate.rs, init.rs, service.rs, compose.rs, etc.
    - These expose vendor assumptions to users
    
23-30. crates/songbird-discovery/src/abstraction/*.rs (8 files)
    - Modern factory patterns
    - Adapter registries
```

**Impact**: User-facing code - clean CLI means clean usage patterns.

### **📝 Medium Priority** (Discovery & Config)

```
31-45. crates/songbird-discovery/src/*.rs (15 files)
    - federation_aware_discovery.rs
    - agnostic_service_mesh.rs  
    - universal_primal_adapter.rs
    
46-55. crates/songbird-config/src/*.rs (10 files)
    - zero_hardcoding_migration.rs (enhance)
    - agnostic_primal_migration.rs (enhance)
```

**Impact**: Infrastructure code - already partially agnostic, just needs completion.

### **✅ Low Priority** (Tests & Disabled Files)

```
56-64. **/tests/*.rs.disabled, **/benches/*.rs.disabled (9 files)
    - Leave for last
    - May just delete if obsolete
```

---

## 🦁 **PRIMAL HARDCODING** (56 files)

### **🔥 Critical Priority** (Cross-Primal References)

```
1. crates/songbird-primal-sdk/src/beardog.rs
2. crates/songbird-primal-sdk/src/toadstool.rs  
3. crates/songbird-primal-sdk/src/squirrel.rs
4. crates/songbird-config/src/config/universal_primals.rs
5. crates/songbird-config/src/config/agnostic_primals.rs (review)
```

**Impact**: MAXIMUM - these are the exact files that hardcode primal relationships. Eliminating these achieves the core goal.

**Action**: Replace entire client implementations with capability-based discovery.

### **⚡ High Priority** (SDK & Capability Modules)

```
6. crates/songbird-primal-sdk/src/capability_ai.rs
7. crates/songbird-primal-sdk/src/capability_storage.rs  
8. crates/songbird-primal-sdk/src/capability_compute.rs
9. crates/songbird-primal-sdk/src/capability_orchestrator.rs
10. crates/songbird-primal-sdk/src/global_adapter.rs
```

**Impact**: These define capability interfaces - should be primal-agnostic by definition.

### **📝 Medium Priority** (Config & Discovery)

```
11-25. crates/songbird-config/src/config/*.rs (15 files)
    - hardcoded_elimination.rs
    - vendor_agnostic_migration.rs
    - zero_touch/*.rs
    
26-40. crates/songbird-discovery/src/*.rs (15 files)
    - Federation aware components
    - Universal adapter implementations
```

### **✅ Low Priority** (Examples & CLI)

```
41-56. crates/songbird-cli/src/cli/commands/*.rs (16 files)
    - User-facing commands that reference primals
    - Update after SDK is clean
```

---

## 🔢 **NUMERIC HARDCODING** (137 files, 516 instances)

### **Strategy**: Extract to configuration, enable discovery

### **Tier 1: Network Configuration** (220 instances, 35 files)

**Default Ports**:
```
8080  → SERVICE_PORT or discovery (120 instances)
3000  → DEV_PORT or discovery (40 instances)
5432  → DATABASE_PORT or discovery (20 instances)
6379  → CACHE_PORT or discovery (15 instances)
9000  → STORAGE_PORT or discovery (10 instances)
Others → Capability-specific env vars (15 instances)
```

**Key Files**:
```
crates/songbird-config/src/config/network.rs
crates/songbird-config/src/config/constants.rs
crates/songbird-config/src/canonical_network.rs
crates/songbird-types/src/config/network.rs
```

**Action**: Create `DiscoverablePort` enum, implement discovery fallbacks.

### **Tier 2: Bind Addresses** (180 instances, 60 files)

**Hardcoded IPs**:
```
localhost  → SERVICE_HOST or discovery (90 instances)
127.0.0.1  → BIND_ADDRESS or discovery (50 instances)  
0.0.0.0    → LISTEN_ADDRESS or discovery (40 instances)
```

**Key Files**:
```
crates/*/src/**/*.rs (widespread across all crates)
```

**Action**: Extract to `DiscoverableAddress` configuration.

### **Tier 3: Endpoint URLs** (116 instances, 42 files)

**URL Patterns**:
```
http://localhost:* → SERVICE_ENDPOINT or discovery (80 instances)
https://127.0.0.1:* → SECURE_ENDPOINT or discovery (20 instances)
*/health, */status → HEALTH_PATH or convention (16 instances)
```

**Key Files**:
```
crates/*/tests/*.rs (60 instances - mostly test code)
crates/songbird-config/src/config/*.rs (30 instances)  
crates/songbird-cli/src/cli/commands/*.rs (26 instances)
```

**Action**: Extract to environment, provide discovery paths, keep fallbacks for dev/test only.

---

## 🎯 **EXECUTION ORDER**

### **Day 1: Foundation** (6-8 hours)

```
Morning (3-4 hours):
  ✅ Review existing agnostic infrastructure
  ✅ Enhance ZeroHardcodingMigrator for our specific patterns
  ✅ Create configuration templates
  
Afternoon (3-4 hours):
  🎯 Vendor Critical Priority (files 1-7)
  🎯 Test vendor agnosticism
```

### **Day 2: Primals** (6-8 hours)

```
Morning (3-4 hours):
  🦁 Primal Critical Priority (files 1-5)
  🦁 Update primal SDK
  🦁 Test primal independence
  
Afternoon (3-4 hours):
  🦁 Primal High Priority (files 6-10)
  🦁 Network effect testing
```

### **Day 3: Numeric & Polish** (4-6 hours)

```
Morning (2-3 hours):
  🔢 Extract numeric hardcoding
  🔢 Implement discoverable config
  
Afternoon (2-3 hours):
  🍼 Enable infant discovery  
  ✅ Full integration testing
  📝 Update documentation
```

---

## 📊 **SUCCESS METRICS**

```
Zero Vendor Names:     0/64 files cleaned → 64/64 ✅
Zero Primal Names:     0/56 files cleaned → 56/56 ✅
Zero Numeric Hardcode: 0/516 instances → 516/516 ✅ (allow <10 dev fallbacks)

Infant Discovery:      Not enabled → Fully operational ✅
Universal Adapter:     Partial → Complete network effects ✅
All Tests:             Passing → Passing ✅

Grade:                 B+ (90/100) → A- (92/100) ✅
```

---

**Ready to execute?** Starting with Vendor Critical Priority files...
