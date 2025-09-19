# Songbird Federation

**Fractal federation system for the Songbird ecosystem with zero-cost abstractions and Security Primal integration.**

## Overview

The `songbird-federation` crate provides a modern, hierarchical, self-sovereign federation system that enables Songbird instances to be deployed anywhere and coordinate in a fractal manner. Each level of the hierarchy maintains its own authority while participating in a larger mesh network.

## Architecture

### Fractal Hierarchy

```
Sovereign Network (Global)
├── Global Songbird (Continental)
│   ├── Regional Songbird (State/Province)
│   │   ├── Local Songbird (City/Tower)
│   │   │   └── Edge Songbird (Home/Friend)
```

### Key Features

- **Self-Sovereign**: Each node maintains its own authority
- **Hierarchical Reporting**: Nodes report to higher-level coordinators
- **Zero-Cost Abstractions**: Compile-time optimizations for performance
- **Security Primal Security**: Cryptographic identity and key management
- **Compute Primal Persistence**: Distributed state storage
- **Mesh Coordination**: Intelligent routing and load balancing

## Usage

```rust
use songbird_federation::{FractalFederationManager, ZeroCostFederationBuilder};

// Create a fractal federation manager
let manager = FractalFederationManager::new()
    .with_tier(FederationTier::Edge)
    .with_beardog_security()
    .with_toadstool_persistence()
    .build()
    .await?;

// Start the federation
manager.start().await?;
```

## Deployment Scenarios

### Edge Deployment
Single Songbird per tower/home for local coordination.

### Regional Coordination
Multiple Songbirds reporting to a regional coordinator for area-wide mesh networking.

### Global Mesh
Hierarchical network with sovereign governance for worldwide coordination.

### Friend Networks
Distributed home computing across personal networks.

## Integration

This crate integrates with:
- **Security Primal**: For genetic spawning and cryptographic security
- **Compute Primal**: For persistent state management and storage
- **Songbird Core**: For orchestration and service discovery

## License

Licensed under the same terms as the parent Songbird project. 