# 🔄 Trait Import Migration Guide

**Date**: October 2, 2025  
**Status**: In Progress (Examples Complete)  
**Effort**: 2-3 hours (30-50 files)  
**Priority**: High (enables full unification)

---

## 🎯 GOAL

Migrate all trait imports from fragmented local definitions to canonical source:

```rust
// ❌ OLD (Fragmented)
use songbird_discovery::traits::ServiceDiscovery;
use crate::traits::ServiceInfo;
use songbird_core::traits::HealthCheck;

// ✅ NEW (Canonical)
use songbird_types::traits::canonical::{ServiceProvider, ServiceInfo, Provider};
```

---

## 📊 MIGRATION STATUS

**Files Requiring Updates**: ~45
- `songbird-discovery`: 18 files
- `songbird-core`: 8 files
- `songbird-registry`: 4 files
- `songbird-network`: 6 files
- `songbird-cli`: 3 files
- `songbird-universal-primals`: 15 files
- Other crates: 6 files

**Completed**: 0 files  
**Remaining**: ~45 files

---

## 🔧 MIGRATION PATTERNS

### Pattern 1: ServiceDiscovery Trait

**OLD**:
```rust
use crate::traits::{ServiceDiscovery, ServiceEvent, ServiceQuery};
use crate::traits::service::ServiceInfo;
```

**NEW**:
```rust
use songbird_types::traits::canonical::{
    DiscoveryProvider as ServiceDiscovery,
    ServiceInfo,
    ServiceEvent,
    DiscoveryQuery as ServiceQuery,
};
```

### Pattern 2: Health Check Traits

**OLD**:
```rust
use crate::traits::{HealthCheck, HealthMonitor, HealthStatus};
```

**NEW**:
```rust
use songbird_types::traits::canonical::{Provider, HealthStatus};
// HealthCheck is now part of Provider trait
```

### Pattern 3: Service Info Types

**OLD**:
```rust
use songbird_discovery::traits::service::ServiceInfo;
use songbird_discovery::traits::ServiceStatus;
```

**NEW**:
```rust
use songbird_types::traits::canonical::{ServiceInfo, HealthStatus as ServiceStatus};
```

### Pattern 4: Communication Layer

**OLD**:
```rust
use songbird_discovery::traits::communication::{CommunicationLayer, ServiceAddress};
```

**NEW**:
```rust
// Communication traits remain in songbird-network
use songbird_network::communication::types::{CommunicationLayer, ServiceAddress};
```

### Pattern 5: Load Balancer Traits

**OLD**:
```rust
use crate::traits::load_balancer::{LoadBalancer, ServiceStats};
```

**NEW**:
```rust
// Load balancer remains domain-specific in songbird-core
use songbird_core::load_balancer::{LoadBalancer, ServiceStats};
```

---

## 📋 FILE-BY-FILE CHECKLIST

### Songbird-Discovery (18 files - High Priority)

- [ ] `src/abstraction/adapters/consul_adapter.rs`
  - Lines: 19
  - Pattern: ServiceDiscovery, ServiceEvent, ServiceInfo, ServiceQuery

- [ ] `src/abstraction/adapters/kubernetes_adapter.rs`
  - Lines: 19
  - Pattern: ServiceDiscovery, ServiceEvent, ServiceInfo, ServiceQuery

- [ ] `src/abstraction/adapters/static_adapter.rs`
  - Lines: 16-17
  - Pattern: ServiceHealthStatus, ServiceEvent, ServiceInfo, ServiceQuery

- [ ] `src/abstraction/providers.rs`
  - Lines: 13-14
  - Pattern: ServiceHealthStatus, ServiceEvent, ServiceInfo, ServiceQuery

- [ ] `src/abstraction/delegation.rs`
  - Lines: 14-15
  - Pattern: ServiceHealthStatus, ServiceEvent, ServiceInfo, ServiceQuery

- [ ] `src/discovery/songbird_discovery.rs`
  - Lines: 14-15
  - Pattern: ServiceEvent, ServiceDiscovery, ServiceQuery, ServiceHealthStatus, ServiceInfo

- [ ] `src/discovery/enhanced_discovery.rs`
  - Lines: 14-15
  - Pattern: ServiceDiscovery, ServiceHealthStatus, ServiceInfo

- [ ] `src/discovery/backends/service_discovery.rs`
  - Lines: 15-17
  - Pattern: ServiceHealthStatus, ServiceInfo, ServiceStatus, ServiceDiscovery, ServiceEvent, ServiceQuery

- [ ] `src/discovery/backends/static_discovery.rs`
  - Lines: 8-9
  - Pattern: ServiceDiscovery, ServiceEvent, ServiceHealthStatus, ServiceQuery, ServiceInfo

- [ ] `src/discovery/backends/kubernetes.rs`
  - Lines: 7-8, 139
  - Pattern: ServiceDiscovery, ServiceEvent, ServiceHealthStatus, ServiceQuery, ServiceInfo, ServiceStatus

- [ ] `src/discovery/backends/consul.rs`
  - Lines: 8-9, 81
  - Pattern: ServiceDiscovery, ServiceEvent, ServiceHealthStatus, ServiceQuery, ServiceInfo, ServiceStatus

- [ ] `src/discovery/backends/container_orchestration.rs`
  - Lines: 12-14
  - Pattern: ServiceHealthStatus, ServiceInfo, ServiceStatus, ServiceDiscovery, ServiceEvent, ServiceQuery

- [ ] `src/discovery/service_registry.rs`
  - Lines: 8
  - Pattern: ServiceEvent, ServiceQuery

- [ ] `src/discovery/node_registry.rs`
  - Lines: 10
  - Pattern: ServiceEvent

- [ ] `src/discovery/types/mod.rs`
  - Lines: 5-6
  - Pattern: ServiceHealthStatus, ServiceInfo

- [ ] `src/discovery/factory.rs`
  - Lines: 87
  - Pattern: ServiceStatus

- [ ] `src/federation_aware_discovery.rs`
  - Lines: 7
  - Pattern: ServiceDiscovery, ServiceInfo

- [ ] `src/lib.rs`
  - Lines: 114, 122
  - Pattern: Re-exports

### Songbird-Core (8 files - High Priority)

- [ ] `src/traits/discovery.rs`
  - Lines: 10
  - Pattern: Import from songbird_discovery

- [ ] `src/traits/hooks.rs`
  - Lines: 8
  - Pattern: ServiceInfo, ServiceRequest, ServiceResponse

- [ ] `src/traits/load_balancer.rs`
  - Lines: 7
  - Pattern: ServiceInfo, ServiceRequest

- [ ] `src/load_balancer/manager.rs`
  - Lines: 6-7
  - Pattern: LoadBalancer traits

- [ ] `src/load_balancer/strategies.rs`
  - Lines: 1
  - Pattern: LoadBalancer traits

- [ ] `src/primal_integration.rs`
  - Lines: 14
  - Pattern: ServiceEndpoint

- [ ] `src/orchestrator/request_router.rs`
  - Lines: 6-8, 130
  - Pattern: LoadBalancer, CommunicationLayer, ServiceRequest/Response

- [ ] `src/load_balancer/types.rs`
  - Lines: 78
  - Pattern: LoadBalancerStats

### Songbird-Registry (4 files - Medium Priority)

- [ ] `src/zero_cost_service_registry.rs`
  - Lines: 7
  - Pattern: ServiceInfo, HealthStatus

- [ ] `src/lib.rs`
  - Lines: 116
  - Pattern: PluginRegistry

- [ ] `src/service/mod.rs`
  - Lines: 14
  - Pattern: ServiceInfo, UniversalService

- [ ] `src/plugin/mod.rs`
  - Lines: 12
  - Pattern: Plugin traits

### Songbird-Network (6 files - Medium Priority)

- [ ] `src/communication/websocket/connection.rs`
  - Lines: 11
  - Pattern: ServiceAddress

- [ ] `src/communication/memory.rs`
  - Lines: 9
  - Pattern: Communication traits

- [ ] `src/communication/websocket/server.rs`
  - Lines: 21
  - Pattern: Communication traits

### Songbird-CLI (3 files - Low Priority)

- [ ] `src/cli/commands/compose.rs`
  - Lines: 14, 427
  - Pattern: PluginCapability

### Songbird-Universal-Primals (15 files - Medium Priority)

All files in `src/discovery/ecosystem/` and `src/` that use:
- `use crate::traits::PrimalCapability`
- `use crate::traits::PrimalProvider`
- `use crate::traits::PrimalContext`

Should migrate to:
- `use songbird_types::traits::canonical::{PrimalProvider, Capability as PrimalCapability, PrimalContext}`

---

## 🚀 MIGRATION PROCEDURE

### For Each File:

1. **Identify current imports**
   ```bash
   grep "use.*traits::" filename.rs
   ```

2. **Update imports to canonical**
   - Replace local trait imports
   - Add songbird_types::traits::canonical
   - Use type aliases where names differ

3. **Update trait bounds**
   - Old: `T: ServiceDiscovery`
   - New: `T: DiscoveryProvider`

4. **Verify compilation**
   ```bash
   cargo check --package <crate-name>
   ```

5. **Run tests**
   ```bash
   cargo test --package <crate-name>
   ```

---

## 📈 SUCCESS METRICS

**Before**:
- 3+ duplicate trait definitions (core, discovery, types)
- 45+ files with inconsistent imports
- Unclear trait hierarchy

**After**:
- 1 canonical trait source (songbird-types)
- All imports point to canonical
- Clear trait hierarchy
- Backward compatibility maintained

---

## ⚠️ MIGRATION NOTES

### What Changes:
- Import paths only
- Type aliases where needed
- Trait bound names (ServiceDiscovery → DiscoveryProvider)

### What Stays Same:
- Trait method signatures
- Implementation logic
- API surface
- Behavior

### Backward Compatibility:
- Old trait modules become re-exports
- Deprecation warnings guide users
- No breaking changes for consumers

---

## 🎯 PRIORITY ORDER

1. **High**: songbird-discovery (18 files) - Core discovery functionality
2. **High**: songbird-core (8 files) - Core orchestration
3. **Medium**: songbird-registry (4 files) - Registry integration
4. **Medium**: songbird-network (6 files) - Network layer
5. **Medium**: songbird-universal-primals (15 files) - Primal integration
6. **Low**: songbird-cli (3 files) - CLI commands

---

## 💡 QUICK WINS

**Start with these 5 files for immediate impact:**

1. `songbird-discovery/src/lib.rs` - Public API re-exports
2. `songbird-discovery/src/traits/mod.rs` - Convert to re-exports
3. `songbird-core/src/traits/mod.rs` - Convert to re-exports  
4. `songbird-discovery/src/discovery/songbird_discovery.rs` - Core discovery
5. `songbird-core/src/orchestrator/request_router.rs` - Core routing

These 5 files will enable the rest of the migration and show clear progress.

---

**Status**: Ready for systematic execution  
**Next**: Update 5 quick-win files, then proceed systematically  
**Estimated Completion**: 2-3 focused hours 