# 🔄 Hardcoding Elimination Guide

**STATUS**: 🟢 Active Migration - December 2024

## 🎯 Mission: Zero Hardcoded Knowledge

Songbird is evolving to eliminate ALL hardcoded:
1. **Primal Names**: BearDog, Toadstool, NestGate, Squirrel
2. **Vendor Names**: Kubernetes, Consul, Docker, etcd
3. **Port Numbers**: 8080, 8443, 9000, etc.
4. **Service Endpoints**: `http://localhost:8080/service`

## 🌱 Philosophy: Infant Discovery

Code starts with **ZERO** knowledge and discovers everything at runtime, like an infant learning about the world.

## 📊 Current Status

### ✅ Completed

1. **`songbird-primal-coordination` crate**: Capability-based coordination system
2. **`songbird-config/agnostic_primal_config.rs`**: Zero primal name configuration
3. **`songbird-discovery/primal_self_knowledge.rs`**: Self-discovery system
4. **`songbird-config/zero_touch/infant_config.rs`**: Zero-touch bootstrap

### 🚧 In Progress

1. Migrating genesis to use coordination crate
2. Migrating compute bridge to agnostic coordination
3. Removing hardcoded ports from test files
4. Evolving vendor-specific adapters to agnostic providers

### 📋 TODO

1. Remove hardcoded primal names from showcase scripts
2. Update documentation to use capability-based examples
3. Create runtime port allocation system
4. Implement full dynamic discovery for all capabilities

## 🔄 Migration Patterns

### Pattern 1: Primal Names → Capabilities

#### ❌ BEFORE (Hardcoded)
```rust
// Hardcoded primal name
let beardog = connect_to_beardog("https://localhost:8443");
let keys = beardog.generate_keys().await?;
```

#### ✅ AFTER (Agnostic)
```rust
use songbird_primal_coordination::{PrimalCoordinator, CapabilityType};

// Request capability, not specific primal
let coordinator = PrimalCoordinator::new(bridge);
let security_conn = coordinator.request_capability(CapabilityType::Security).await?;
let response = security_conn.send_request(PrimalRequest::GenerateKeys).await?;
```

### Pattern 2: Hardcoded Endpoints → Discovery

#### ❌ BEFORE (Hardcoded)
```rust
// Hardcoded endpoint
let beardog_endpoint = "https://localhost:8443";
let nestgate_endpoint = "http://localhost:8080/storage";
let toadstool_endpoint = "http://localhost:8082";
```

#### ✅ AFTER (Discovery)
```rust
use songbird_config::agnostic_primal_config::AgnosticPrimalConfig;

// Discover from environment
let config = AgnosticPrimalConfig::from_environment()?;
let security_endpoint = config.request_capability("security").await?;
let storage_endpoint = config.request_capability("storage").await?;
let compute_endpoint = config.request_capability("compute").await?;
```

**Environment Variables:**
```bash
# Set capability endpoints (discovered, not hardcoded)
export CAPABILITY_SECURITY_ENDPOINT="https://security-provider:8443"
export CAPABILITY_COMPUTE_ENDPOINT="http://compute-provider:8082"
export CAPABILITY_STORAGE_ENDPOINT="http://storage-provider:8080"
```

### Pattern 3: Hardcoded Ports → Dynamic Allocation

#### ❌ BEFORE (Hardcoded)
```rust
// Hardcoded ports
const ORCHESTRATOR_PORT: u16 = 8080;
const GAMING_PORT: u16 = 8081;
const FEDERATION_PORT: u16 = 8082;
```

#### ✅ AFTER (Dynamic)
```rust
use songbird_config::port_discovery::PortDiscovery;

// Discover available ports
let port_discovery = PortDiscovery::new();
let orchestrator_port = port_discovery.allocate_port("orchestrator").await?;
let gaming_port = port_discovery.allocate_port("gaming").await?;
let federation_port = port_discovery.allocate_port("federation").await?;
```

**Environment Variables:**
```bash
# Ports come from environment or are auto-allocated
export SERVICE_PORT=8080       # Or omit for auto-allocation
export GAMING_PORT=8081        # Or omit for auto-allocation
export FEDERATION_PORT=8082    # Or omit for auto-allocation
```

### Pattern 4: Vendor Names → Agnostic Providers

#### ❌ BEFORE (Hardcoded)
```rust
// Hardcoded vendor
use songbird_discovery::backends::KubernetesDiscovery;
use songbird_discovery::backends::ConsulDiscovery;

let discovery = if in_kubernetes() {
    KubernetesDiscovery::new()
} else {
    ConsulDiscovery::new()
};
```

#### ✅ AFTER (Agnostic)
```rust
use songbird_discovery::abstraction::DiscoveryProvider;

// Agnostic provider - discovers which system is available
let discovery = DiscoveryProvider::auto_detect().await?;
// Could be Kubernetes, Consul, Docker, or any other system
```

## 🔧 Migration Helpers

### Environment Variable Migration

Use the migration helper to convert legacy variables:

```rust
use songbird_config::agnostic_primal_config::PrimalConfigMigration;

// Automatically migrate legacy env vars
PrimalConfigMigration::migrate_legacy_env_vars();

// Maps:
// SONGBIRD_BEARDOG_ENDPOINT → CAPABILITY_SECURITY_ENDPOINT
// SONGBIRD_TOADSTOOL_ENDPOINT → CAPABILITY_COMPUTE_ENDPOINT
// SONGBIRD_NESTGATE_ENDPOINT → CAPABILITY_STORAGE_ENDPOINT
// SONGBIRD_SQUIRREL_ENDPOINT → CAPABILITY_AI_ENDPOINT
```

### Code Search & Replace

Find all hardcoded references:

```bash
# Find primal names
grep -r "BearDog\|Toadstool\|NestGate\|Squirrel" crates/ --exclude-dir=target

# Find hardcoded ports
grep -r ":\s*\d{4,5}" crates/ --exclude-dir=target | grep -v "//"

# Find vendor names
grep -r "kubernetes\|consul\|docker\|etcd" crates/ --exclude-dir=target -i
```

## 📝 Code Review Checklist

When reviewing code for hardcoding violations:

- [ ] No primal names in string literals
- [ ] No hardcoded port numbers
- [ ] No vendor-specific code paths
- [ ] All endpoints from environment or discovery
- [ ] Capability-based instead of name-based
- [ ] Tests use mock providers, not hardcoded names
- [ ] Documentation uses generic examples

## 🧪 Testing

### Unit Tests

Use mock providers:

```rust
#[cfg(test)]
mod tests {
    use songbird_primal_coordination::bridge::*;
    
    struct MockPrimalBridge;
    
    #[async_trait::async_trait]
    impl PrimalBridge for MockPrimalBridge {
        async fn connect(&self, capability: CapabilityType) -> Result<PrimalConnection> {
            // Mock implementation - no hardcoded names
            Ok(PrimalConnection::new(
                uuid::Uuid::new_v4().to_string(),
                format!("mock://{}", capability.as_str()),
                mock_capabilities(),
            ))
        }
        
        // ... other trait methods
    }
}
```

### Integration Tests

Use environment configuration:

```bash
# Set up test environment
export CAPABILITY_SECURITY_ENDPOINT="http://localhost:9001"
export CAPABILITY_COMPUTE_ENDPOINT="http://localhost:9002"
export CAPABILITY_STORAGE_ENDPOINT="http://localhost:9003"

# Run tests
cargo test --test integration_tests
```

## 🎯 Success Criteria

### Phase 1: Foundation (✅ COMPLETE)
- [x] `songbird-primal-coordination` crate created
- [x] `AgnosticPrimalConfig` implemented
- [x] `PrimalSelfKnowledge` for self-discovery
- [x] Migration helpers created

### Phase 2: Migration (🚧 IN PROGRESS)
- [ ] Genesis uses coordination crate
- [ ] Compute bridge uses agnostic coordination
- [ ] All config files use capability-based discovery
- [ ] Tests updated to use mock providers

### Phase 3: Cleanup (📋 TODO)
- [ ] Remove all primal name string literals
- [ ] Remove all hardcoded ports
- [ ] Remove all vendor-specific code paths
- [ ] Update all documentation

### Phase 4: Validation (📋 TODO)
- [ ] Code audit: zero hardcoded primal names
- [ ] Code audit: zero hardcoded ports
- [ ] Code audit: zero vendor-specific paths
- [ ] 100% capability-based discovery
- [ ] All tests pass with mock providers

## 📚 Reference

### Capability Types

Standard capabilities:
- `security`: Key management, signing, encryption
- `compute`: Task execution, ML inference
- `storage`: Data persistence, caching
- `ai`: Model serving, training
- `discovery`: Service registration, lookup
- `orchestration`: Coordination, scheduling
- `networking`: P2P, federation

### Environment Variables

#### Capability Endpoints
```bash
CAPABILITY_SECURITY_ENDPOINT=https://security-provider:8443
CAPABILITY_COMPUTE_ENDPOINT=http://compute-provider:8082
CAPABILITY_STORAGE_ENDPOINT=http://storage-provider:8080
CAPABILITY_AI_ENDPOINT=http://ai-provider:8083
```

#### Discovery Configuration
```bash
CAPABILITY_DISCOVERY_ENABLED=true
CAPABILITY_DISCOVERY_TIMEOUT=30
CAPABILITY_CACHE_TTL=300
ENABLE_DNS_SRV_DISCOVERY=true
ENABLE_MDNS_DISCOVERY=true
```

#### Service Mesh
```bash
SERVICE_MESH_ENABLED=true
SERVICE_MESH_PROTOCOL=tarpc
SERVICE_MESH_TLS=true
SERVICE_MESH_DISCOVERY_INTERVAL=60
```

## 🚀 Quick Start

### For New Code

```rust
use songbird_primal_coordination::{PrimalCoordinator, CapabilityType};
use songbird_config::agnostic_primal_config::AgnosticPrimalConfig;

// 1. Create configuration (zero hardcoded knowledge)
let config = AgnosticPrimalConfig::from_environment()?;

// 2. Create coordinator with discovery bridge
let bridge = DiscoveryBasedBridge::new(Arc::new(discovery_engine));
let coordinator = PrimalCoordinator::new(Arc::new(bridge));

// 3. Request capabilities as needed
let security_conn = coordinator.request_capability(CapabilityType::Security).await?;
let compute_conn = coordinator.request_capability(CapabilityType::Compute).await?;

// 4. Coordinate operations
let identity = coordinator.coordinate_genesis(new_node_id).await?;
let deployment_id = coordinator.deploy_compute(workload).await?;
```

### For Existing Code

1. Add migration helper to your main:
```rust
use songbird_config::agnostic_primal_config::PrimalConfigMigration;

fn main() {
    // Migrate legacy env vars
    PrimalConfigMigration::migrate_legacy_env_vars();
    
    // Rest of your code...
}
```

2. Gradually replace hardcoded references with capability-based discovery

3. Test with environment variables set

4. Remove hardcoded values

## 💡 Tips

1. **Start Small**: Migrate one module at a time
2. **Use Migration Helpers**: `PrimalConfigMigration::migrate_legacy_env_vars()`
3. **Test Incrementally**: Ensure tests pass after each change
4. **Document Changes**: Update docs to reflect new patterns
5. **Review Carefully**: Check for hidden hardcoding in tests and examples

## 🎓 Learning Resources

- `specs/PRIMAL_COORDINATION_ARCHITECTURE.md` - Overall architecture
- `crates/songbird-primal-coordination/` - Coordination implementation
- `crates/songbird-config/src/agnostic_primal_config.rs` - Agnostic config
- `crates/songbird-discovery/src/primal_self_knowledge.rs` - Self-discovery

---

**Last Updated**: December 24, 2025
**Status**: 🟢 Active Migration
**Contact**: ecoPrimals Team

