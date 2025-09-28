# 🎯 Trait Migration Guide

## Canonical Trait Locations

After consolidation, all traits have canonical locations:

### ServiceDiscovery
- **Canonical Location**: `songbird-discovery::traits::ServiceDiscovery`
- **Re-exported in**: `songbird-core::traits::discovery`
- **Migration**: Replace all local ServiceDiscovery definitions with imports

```rust
// OLD (duplicate definitions)
use songbird_core::traits::discovery::ServiceDiscovery;

// NEW (canonical)
use songbird_discovery::traits::ServiceDiscovery;
```

### PrimalProvider  
- **Canonical Location**: `songbird-universal-primals::traits::PrimalProvider`
- **Re-exported in**: `songbird-universal::traits`
- **Migration**: Use the canonical trait for all primal implementations

```rust
// OLD (if using local definitions)
use crate::traits::PrimalProvider;

// NEW (canonical)
use songbird_universal_primals::traits::PrimalProvider;
```

### HealthCheck
- **Canonical Location**: `songbird-observability::health::HealthCheck`
- **Re-exported in**: `songbird-core::traits`
- **Migration**: Use observability crate for all health functionality

```rust
// OLD (if using local definitions)  
use songbird_core::traits::health::HealthCheck;

// NEW (canonical)
use songbird_observability::health::HealthCheck;
```

## Benefits

- **Single Source of Truth**: One definition per trait
- **Consistent APIs**: All implementations use the same interface
- **Reduced Compilation**: Eliminates duplicate trait definitions
- **Clear Dependencies**: Explicit trait ownership by crate purpose
