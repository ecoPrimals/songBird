# Songbird Universal Orchestrator Examples

This directory contains examples demonstrating the capabilities of the Songbird Universal Orchestrator.

## 📁 Directory Structure

- **Current Examples** - Modern, production-ready code (100% Pure Rust)
- **`legacy/`** - Archived pre-ecoBin v2.0 examples (uses reqwest, not recommended)

**For new projects, use the current examples!** Legacy examples are kept for historical reference only.

## Demo: Universal Orchestration (`demo_orchestration.rs`)

This comprehensive demo showcases the key features implemented in the Songbird Universal Orchestrator:

### Features Demonstrated

1. **🔧 Service Registry & Dynamic Plugin Composition**
   - Creates service registry for managing orchestrated services
   - Registers dynamic plugins (BearDog encryption, Toadstool compute, NestGate storage)
   - Auto-composes a complete system from individual plugin capabilities
   - Eliminates the need for 256+ TOML configuration files through runtime composition

2. **🏗️ BYOB (Bring Your Own Biome) Deployment**
   - Registers team workspaces with resource quotas
   - Creates biome manifests with service dependencies
   - Deploys complete biomes with dependency resolution
   - Demonstrates service orchestration with health checks

3. **🔍 Auto-Discovery System**
   - Discovers services on the network automatically
   - Finds Primal endpoints (Toadstool, NestGate, BearDog, Squirrel)
   - Continuous background scanning for new services
   - Network topology mapping

### Key Innovations

- **Zero-Configuration Deployment**: No need for complex TOML configurations
- **Dynamic Plugin Composition**: Services composed at runtime like "lego blocks"
- **Universal Primal Coordination**: Seamless integration with all Primal services
- **Dependency Resolution**: Automatic service startup ordering with cycle detection
- **Health Monitoring**: Continuous health checks and service readiness validation

### Running the Demo

```bash
# Run the demonstration
cargo run --example demo_orchestration

# With detailed logging
RUST_LOG=info cargo run --example demo_orchestration
```

### Expected Output

The demo will show:
- Plugin registration and auto-composition
- Team workspace setup and biome deployment
- Service discovery and Primal endpoint detection
- Real-time orchestration status updates

### Architecture Highlights

The demo illustrates the core innovation of replacing traditional configuration-heavy orchestration with:

1. **Dynamic Service Discovery**: Services automatically found and registered
2. **Runtime Plugin Composition**: Capabilities combined on-demand
3. **Dependency-Aware Orchestration**: Services started in correct order
4. **Health-Driven Management**: Continuous monitoring and self-healing
5. **Primal Ecosystem Integration**: Seamless coordination with external services

This demonstrates how Songbird achieves the goal mentioned in the handoff notes: eliminating the complexity of managing 256+ configuration files through intelligent automation and dynamic composition.

### Related Documentation

- [Architecture Specifications](../specs/architecture.md)
- [BYOB System Design](../specs/byob-system.md)
- [Service Registry Design](../specs/service-registry.md)
- [Universal Orchestration](../specs/universal-orchestration.md) 