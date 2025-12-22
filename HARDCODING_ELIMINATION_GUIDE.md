# 🚀 Hardcoding Elimination Guide

**Date**: December 22, 2025  
**Status**: In Progress  
**Goal**: Eliminate all 1,718 hardcoded localhost/port references

---

## 🎯 Philosophy

**Each primal knows only itself. All inter-primal communication is discovered at runtime.**

- ❌ No hardcoded primal names in code
- ❌ No hardcoded localhost addresses
- ❌ No hardcoded port numbers
- ✅ Runtime capability-based discovery
- ✅ Environment variable configuration
- ✅ Self-knowledge only

---

## 📊 Current State

### Hardcoding Inventory
```
Total hardcoded references: 1,718 across 324 files

Distribution:
- localhost: ~600 instances
- 127.0.0.1: ~400 instances  
- 0.0.0.0: ~300 instances
- Specific ports (8080, 3000, etc.): ~418 instances
```

### Critical Areas
1. **Network binding** - `crates/songbird-orchestrator/src/network/binding.rs`
2. **Service registry** - `crates/songbird-orchestrator/src/server/service_registry_api.rs`
3. **Federation** - `crates/songbird-network-federation/src/btsp/`
4. **Config constants** - `crates/songbird-config/src/config/constants.rs`
5. **Tests** - Extensive hardcoding (acceptable for local tests)

---

## 🔧 Migration Tools

### 1. RuntimeEndpointResolver (NEW)

Modern capability-based endpoint resolution:

```rust
use songbird_config::runtime_endpoint_resolver::RuntimeEndpointResolver;

// Create resolver
let resolver = RuntimeEndpointResolver::new();

// Register local services (self-knowledge)
resolver.register_local_service("my-capability", "http://0.0.0.0:8080").await?;

// Discover other services by capability
let compute_endpoint = resolver.resolve_capability("compute").await?;
let storage_endpoint = resolver.resolve_capability("storage").await?;
```

### 2. CapabilityDiscovery (Existing)

Discovery engine with multiple strategies:

```rust
use songbird_config::capability_discovery::CapabilityDiscovery;

let discovery = CapabilityDiscovery::new();

// Discover by capability, not by primal name
let endpoints = discovery.discover_by_capability("compute").await?;
```

### 3. Network Binding Strategy (Existing)

Intelligent, zero-config network binding:

```rust
use songbird_orchestrator::network::NetworkBindingStrategy;

// Auto-detect best binding (IPv4/IPv6/dual-stack)
let strategy = NetworkBindingStrategy::auto_detect().await?;
let addresses = strategy.bind_addresses(port)?;
```

---

## 🎨 Migration Patterns

### Pattern 1: Simple Hardcoded Endpoint

**❌ Before:**
```rust
let endpoint = "http://localhost:8080";
let client = create_client(endpoint)?;
```

**✅ After:**
```rust
let resolver = RuntimeEndpointResolver::new();
let endpoint = resolver.resolve_capability("orchestrator").await?;
let client = create_client(&endpoint)?;
```

### Pattern 2: Hardcoded Port Number

**❌ Before:**
```rust
let port = 8080;
let addr = format!("0.0.0.0:{}", port);
```

**✅ After:**
```rust
use songbird_types::SafeEnv;

let port = SafeEnv::parse("SONGBIRD_PORT", 8080);
let addr = format!("0.0.0.0:{}", port);
```

### Pattern 3: Primal-Specific Hardcoding

**❌ Before:**
```rust
let beardog_url = "http://localhost:9000";
let nestgate_url = "http://localhost:9001";
```

**✅ After:**
```rust
let resolver = RuntimeEndpointResolver::new();

// Discover by capability, not by primal name
let crypto_endpoint = resolver.resolve_capability("cryptography").await?;
let gateway_endpoint = resolver.resolve_capability("gateway").await?;
```

### Pattern 4: Test Fixtures

**❌ Before:**
```rust
#[tokio::test]
async fn test_service() {
    let endpoint = "http://localhost:8080";
    // test code...
}
```

**✅ After:**
```rust
#[tokio::test]
async fn test_service() {
    use songbird_test_utils::fixtures::ports::get_free_port;
    
    let port = get_free_port().await;
    let endpoint = format!("http://localhost:{}", port);
    // test code...
}
```

### Pattern 5: Configuration Defaults

**❌ Before:**
```rust
pub const DEFAULT_PORT: u16 = 8080;
pub const DEFAULT_HOST: &str = "127.0.0.1";
```

**✅ After:**
```rust
use songbird_types::SafeEnv;

pub fn default_port() -> u16 {
    SafeEnv::parse("SONGBIRD_PORT", 8080)
}

pub fn default_host() -> String {
    SafeEnv::get_or_default("SONGBIRD_HOST", "127.0.0.1")
}
```

---

## 🔄 Environment Variable Conventions

### Capability Endpoints
```bash
# General pattern: {CAPABILITY}_ENDPOINT
export COMPUTE_ENDPOINT="http://compute-service:8080"
export STORAGE_ENDPOINT="http://storage-service:8081"
export GATEWAY_ENDPOINT="http://gateway-service:8082"

# Alternative pattern: SONGBIRD_{CAPABILITY}_URL
export SONGBIRD_COMPUTE_URL="http://compute-service:8080"
```

### Port Configuration
```bash
# Service-specific ports
export SONGBIRD_ORCHESTRATOR_PORT=8080
export SONGBIRD_REGISTRY_PORT=8081

# Port range for dynamic allocation
export SONGBIRD_PORT_START=8000
export SONGBIRD_PORT_END=8100
```

### Network Binding
```bash
# Bind address (auto-detected if not set)
export SONGBIRD_BIND_ADDRESS="0.0.0.0"

# Force specific binding strategy
export SONGBIRD_NETWORK_STRATEGY="dual-stack"  # or "ipv4" or "ipv6"
```

---

## 📋 Migration Checklist

### Phase 1: Infrastructure (DONE)
- [x] Create `RuntimeEndpointResolver`
- [x] Document migration patterns
- [x] Establish environment variable conventions
- [x] Test basic capability resolution

### Phase 2: High-Traffic Paths (In Progress)
- [ ] Migrate orchestrator network binding
- [ ] Migrate service registry endpoints
- [ ] Migrate federation coordinator
- [ ] Migrate BTSP provider

### Phase 3: Configuration Layer
- [ ] Migrate `config/constants.rs` to use SafeEnv
- [ ] Update default value functions
- [ ] Remove hardcoded primal name constants

### Phase 4: Service Layer
- [ ] Migrate client connection logic
- [ ] Update service discovery
- [ ] Convert inter-primal communication

### Phase 5: Test Isolation
- [ ] Use dynamic port allocation in tests
- [ ] Update test fixtures
- [ ] Ensure tests don't conflict

### Phase 6: Validation
- [ ] Scan for remaining hardcoding (target: <100)
- [ ] Document remaining acceptable cases (test constants)
- [ ] Create automated detection

---

## 🎯 Success Criteria

### Must Have
- [ ] <100 hardcoded references in production code
- [ ] All inter-primal communication uses capability discovery
- [ ] Environment variable override for all endpoints
- [ ] Zero primal names in production code paths

### Should Have
- [ ] Automated hardcoding detection in CI
- [ ] Migration guide documentation
- [ ] Example configurations for all deployment scenarios

### Nice to Have
- [ ] Hot-reload capability for endpoint changes
- [ ] Health-based endpoint selection
- [ ] Automatic failover between discovered endpoints

---

## 🚫 Acceptable Hardcoding

Some hardcoding is acceptable in specific contexts:

### 1. Test Constants
```rust
// OK: Test fixtures with hardcoded ports
#[cfg(test)]
const TEST_PORT: u16 = 8080;
```

### 2. Default Fallbacks
```rust
// OK: Fallback for when all discovery fails
const DEVELOPMENT_FALLBACK: &str = "http://localhost:8080";
```

### 3. Protocol Standards
```rust
// OK: Standard protocol ports
const HTTP_DEFAULT_PORT: u16 = 80;
const HTTPS_DEFAULT_PORT: u16 = 443;
```

### 4. Loopback References
```rust
// OK: Explicit localhost for local-only services
const LOCALHOST: &str = "127.0.0.1";
```

---

## 📈 Progress Tracking

### Week 1 (Current)
- ✅ Infrastructure created
- ✅ Migration patterns documented
- ⏳ High-traffic path migration started

### Week 2 (Target)
- [ ] 50% reduction in hardcoding (1718 → 850)
- [ ] All service-to-service communication migrated
- [ ] Configuration layer updated

### Week 3 (Target)
- [ ] 80% reduction (1718 → 340)
- [ ] Test infrastructure updated
- [ ] Documentation complete

### Week 4 (Target)
- [ ] 95% reduction (1718 → <100)
- [ ] Automated detection in CI
- [ ] Production validation

---

## 🔍 Detection & Prevention

### Automated Detection

```bash
# Scan for hardcoded patterns
./scripts/detect-hardcoding.sh

# Example output:
# Found 245 hardcoded references:
# - 120 in production code (FAIL)
# - 125 in test code (OK)
```

### CI Integration

```yaml
# .github/workflows/hardcoding-check.yml
name: Hardcoding Detection
on: [push, pull_request]
jobs:
  detect:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Detect hardcoding
        run: ./scripts/detect-hardcoding.sh
      - name: Fail if threshold exceeded
        run: |
          count=$(grep -r "localhost" crates/ --include="*.rs" | grep -v "test" | wc -l)
          if [ $count -gt 100 ]; then
            echo "ERROR: $count hardcoded references found (max: 100)"
            exit 1
          fi
```

---

## 💡 Best Practices

### 1. Self-Knowledge Pattern
```rust
// Each service registers only its own capabilities
impl MyService {
    async fn start(&self) -> Result<()> {
        let resolver = RuntimeEndpointResolver::new();
        
        // Register self
        resolver.register_local_service(
            "my-capability",
            &self.listen_address
        ).await?;
        
        // Discover others by capability, not by name
        let needed_service = resolver.resolve_capability("needed-capability").await?;
        
        Ok(())
    }
}
```

### 2. Graceful Degradation
```rust
// Try discovery, fall back gracefully
async fn get_endpoint(resolver: &RuntimeEndpointResolver) -> String {
    resolver.resolve_capability("compute")
        .await
        .unwrap_or_else(|_| {
            warn!("Could not discover compute endpoint, using development fallback");
            "http://localhost:8080".to_string()
        })
}
```

### 3. Configuration Flexibility
```rust
// Support multiple configuration sources
async fn resolve_with_flexibility(capability: &str) -> Result<String> {
    // 1. Environment variable (highest priority)
    if let Ok(endpoint) = std::env::var(format!("{}_ENDPOINT", capability.to_uppercase())) {
        return Ok(endpoint);
    }
    
    // 2. Runtime discovery
    let resolver = RuntimeEndpointResolver::new();
    if let Ok(endpoint) = resolver.resolve_capability(capability).await {
        return Ok(endpoint);
    }
    
    // 3. Configuration file
    // ... load from config ...
    
    // 4. Development fallback
    warn!("Using development fallback for {}", capability);
    Ok(format!("http://localhost:8080"))
}
```

---

## 📚 References

- [Capability Discovery Documentation](./crates/songbird-config/src/capability_discovery.rs)
- [Runtime Endpoint Resolver](./crates/songbird-config/src/runtime_endpoint_resolver.rs)
- [Network Binding Strategy](./crates/songbird-orchestrator/src/network/binding.rs)
- [Zero Hardcoding Migration System](./crates/songbird-config/src/zero_hardcoding_migration.rs)

---

## 🤝 Contributing

When adding new code:
1. **Never** hardcode primal names
2. **Never** hardcode localhost/ports in production code
3. **Always** use capability-based discovery
4. **Always** support environment variable override
5. **Document** any acceptable hardcoding with inline comments

When reviewing code:
1. Check for hardcoded endpoints
2. Verify capability-based patterns are used
3. Ensure environment variable support
4. Test with different configurations

---

**Status**: 🟡 In Progress (Phase 2 of 6)  
**Target Completion**: 4 weeks  
**Next Review**: After Phase 2 completion

*Generated: December 22, 2025*

