# 📍 HARDCODING ELIMINATION FRAMEWORK
## Modern, Capability-Based Configuration System

**Created**: December 8, 2025  
**Status**: Ready for Implementation  
**Total Hardcoded Values**: 3,717 (1,869 ports, 1,271 IPs, 577 other)

---

## 🎯 STRATEGY OVERVIEW

### **Philosophy**: Three-Tier Migration

1. **Test Fixtures** (1,200 instances) - Consolidate into shared modules
2. **Configuration Defaults** (1,800 instances) - Use canonical system
3. **Production Code** (717 instances) - Capability-based discovery

**Principle**: **Primal Self-Knowledge + Runtime Discovery**
- Each primal knows ONLY itself
- Discovers others via capability-based mechanisms
- Zero hardcoded peer information

---

## 📦 TIER 1: TEST FIXTURES (Consolidation)

### **Current Problem**:
Tests have hardcoded ports scattered across 561 test files:
```rust
// ❌ BAD: Hardcoded everywhere
let endpoint = "http://localhost:8080";  // Repeated 100+ times
```

### **Solution**: Centralized Test Fixtures

**Create**: `crates/songbird-test-utils/src/fixtures/ports.rs`

```rust
//! Consolidated Test Port Fixtures
//!
//! **Philosophy**: Tests can use hardcoded values, but they should be
//! centralized for easy maintenance and conflict prevention.

/// Test port allocation strategy:
/// - 8000-8099: Core services (orchestrator, discovery)
/// - 8100-8199: Primal services (toadstool, beardog, etc.)
/// - 8200-8299: Test utilities and mocks
/// - 8300-8399: Integration test scenarios

pub mod ports {
    /// Core Songbird orchestrator test port
    pub const ORCHESTRATOR: u16 = 8000;
    
    /// Discovery service test port
    pub const DISCOVERY: u16 = 8001;
    
    /// Federation coordinator test port
    pub const FEDERATION: u16 = 8002;
    
    /// Metrics collection test port
    pub const METRICS: u16 = 8003;
    
    // Primal service test ports (discovered via capabilities in production)
    pub mod primals {
        /// ToadStool compute service test port
        pub const TOADSTOOL: u16 = 8100;
        
        /// BearDog security service test port
        pub const BEARDOG: u16 = 8101;
        
        /// NestGate storage service test port
        pub const NESTGATE: u16 = 8102;
        
        /// Squirrel AI service test port
        pub const SQUIRREL: u16 = 8103;
    }
    
    // Mock service ports
    pub mod mocks {
        /// Mock HTTP server port
        pub const MOCK_HTTP: u16 = 8200;
        
        /// Mock gRPC server port
        pub const MOCK_GRPC: u16 = 8201;
    }
}

/// Test endpoint builders (DRY principle)
pub mod endpoints {
    use super::ports;
    
    /// Build orchestrator test endpoint
    pub fn orchestrator() -> String {
        format!("http://localhost:{}", ports::ORCHESTRATOR)
    }
    
    /// Build discovery test endpoint
    pub fn discovery() -> String {
        format!("http://localhost:{}", ports::DISCOVERY)
    }
    
    /// Build ToadStool test endpoint
    pub fn toadstool() -> String {
        format!("http://localhost:{}", ports::primals::TOADSTOOL)
    }
    
    /// Build BearDog test endpoint
    pub fn beardog() -> String {
        format!("http://localhost:{}", ports::primals::BEARDOG)
    }
    
    /// Build generic test endpoint
    pub fn generic(port: u16) -> String {
        format!("http://localhost:{}", port)
    }
    
    /// Build with custom host
    pub fn with_host(host: &str, port: u16) -> String {
        format!("http://{}:{}", host, port)
    }
}
```

### **Migration Example**:

```rust
// BEFORE: Scattered hardcoding
#[test]
fn test_connect_to_orchestrator() {
    let url = "http://localhost:8080";  // ❌ Magic number
    // ...
}

#[test]
fn test_discover_services() {
    let discovery_url = "http://localhost:8081";  // ❌ Different convention
    // ...
}

// AFTER: Centralized fixtures
use songbird_test_utils::fixtures::{ports, endpoints};

#[test]
fn test_connect_to_orchestrator() {
    let url = endpoints::orchestrator();  // ✅ Clear intent
    // ...
}

#[test]
fn test_discover_services() {
    let discovery_url = endpoints::discovery();  // ✅ Consistent
    // ...
}
```

**Benefits**:
- ✅ Single source of truth
- ✅ Easy to change ports globally
- ✅ Clear naming conventions
- ✅ Port conflict prevention
- ✅ Maintains test isolation

**Time Estimate**: 6-8 hours for full migration

---

## 📝 TIER 2: CONFIGURATION DEFAULTS (Canonical System)

### **Current Problem**:
Production code uses hardcoded defaults:
```rust
// ❌ BAD: Hardcoded in source
let port = 8080;  // Should come from config
```

### **Solution**: Canonical Defaults System

**Already Exists**: `crates/songbird-config/src/canonical/defaults.rs`

### **Usage Pattern**:

```rust
// BEFORE: Hardcoded
pub fn start_server() {
    let port = 8080;  // ❌ Hardcoded
    let addr = format!("0.0.0.0:{}", port);
    // ...
}

// AFTER: Canonical defaults
use songbird_config::canonical::defaults::ports;

pub fn start_server() {
    let port = ports::orchestrator_default();  // ✅ From config
    let addr = format!("0.0.0.0:{}", port);
    // ...
}

// EVEN BETTER: Environment-aware
use songbird_config::canonical::defaults;

pub fn start_server() {
    let port = defaults::get_port_range_start();  // Respects SONGBIRD_PORT_START
    let addr = defaults::get_canonical_bind_address();  // Respects env vars
    // ...
}
```

### **Environment Variable Hierarchy**:

```rust
// Order of precedence:
// 1. Explicit environment variable (SONGBIRD_ORCHESTRATOR_PORT)
// 2. Generic port range (SONGBIRD_PORT_START + offset)
// 3. Configuration file setting
// 4. Canonical default constant

pub fn get_orchestrator_port() -> u16 {
    std::env::var("SONGBIRD_ORCHESTRATOR_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| {
            let base = get_port_range_start();  // Respects SONGBIRD_PORT_START
            base  // Orchestrator gets base port
        })
}
```

**Migration Script**:

```bash
#!/bin/bash
# migrate_hardcoded_ports.sh

echo "Migrating hardcoded ports to canonical defaults..."

# Find all hardcoded 8080
rg "8080" crates/*/src -t rust | while read -r line; do
    file=$(echo "$line" | cut -d: -f1)
    echo "Found in: $file"
    # Add TODO comment for manual review
    sed -i 's/8080/8080 \/\/ TODO: Migrate to defaults::orchestrator_port()/g' "$file"
done

echo "Migration markers added. Review and replace manually."
```

**Time Estimate**: 8-10 hours for production code

---

## 🔍 TIER 3: RUNTIME DISCOVERY (Capability-Based)

### **Current Problem**:
Code hardcodes peer addresses:
```rust
// ❌ BAD: Assumes ToadStool location
let toadstool_url = "http://192.168.1.100:8080";
```

### **Solution**: Capability-Based Discovery

### **Primal Self-Knowledge Pattern**:

```rust
/// Each primal knows ONLY about itself
pub struct PrimalSelfKnowledge {
    /// My own identity (the ONLY thing I hardcode about myself)
    pub my_node_id: NodeId,
    pub my_capabilities: Vec<Capability>,
    pub my_advertise_address: SocketAddr,
    
    // NO peer addresses!
    // NO other primal assumptions!
}

impl PrimalSelfKnowledge {
    /// Discover providers by capability (NOT by name!)
    pub async fn discover_capability_providers(
        &self,
        capability: &str
    ) -> Vec<PrimalEndpoint> {
        // Uses:
        // - mDNS for local network
        // - DNS-SD for service discovery
        // - Consul/etcd if available
        // - Manual registration if needed
        
        self.discovery_engine
            .find_providers_by_capability(capability)
            .await
    }
}
```

### **Example Migration**:

```rust
// BEFORE: Hardcoded ToadStool
async fn submit_compute_job(job: ComputeJob) -> Result<JobId> {
    let toadstool_url = "http://192.168.1.100:8080";  // ❌ Hardcoded!
    let client = HttpClient::new(toadstool_url);
    client.submit_job(job).await
}

// AFTER: Capability-based discovery
async fn submit_compute_job(
    adapter: &UniversalAdapter,
    job: ComputeJob
) -> Result<JobId> {
    // Discover ANY compute provider (could be ToadStool, or something else!)
    let providers = adapter
        .discover_providers("compute")
        .await?;
    
    let provider = providers
        .into_iter()
        .find(|p| p.is_healthy())
        .ok_or(Error::NoHealthyProvider)?;
    
    let client = HttpClient::new(&provider.endpoint);
    client.submit_job(job).await
}
```

### **Discovery Mechanisms** (in priority order):

1. **mDNS** (local network, zero configuration)
```rust
// Automatically discovers services on LAN
let services = mdns_discover("_compute._tcp").await?;
```

2. **DNS-SD** (service discovery via DNS)
```rust
// Uses SRV records for service location
let services = dns_sd_lookup("compute.songbird.local").await?;
```

3. **Configuration** (explicit endpoints when discovery unavailable)
```rust
// Fallback: Read from config file
let endpoints = config.get_configured_endpoints("compute")?;
```

4. **Environment Variables** (12-factor app pattern)
```rust
// COMPUTE_ENDPOINT=http://toadstool.example.com:8080
let endpoint = env::var("COMPUTE_ENDPOINT").ok();
```

### **NO Primal Names in Routing**:

```rust
// ❌ BAD: Hardcoded primal names
if service_type == "toadstool" {
    connect_to_toadstool();
} else if service_type == "beardog" {
    connect_to_beardog();
}

// ✅ GOOD: Capability-based
let capability = request.required_capability();  // "compute", "security", etc.
let providers = discover_providers(capability).await?;
let provider = select_best_provider(providers)?;
route_to_provider(provider, request).await
```

**Time Estimate**: 10-12 hours for full implementation

---

## 🔧 IMPLEMENTATION PLAN

### **Phase 1: Test Fixtures** (Week 1)
- [ ] Create `fixtures/ports.rs`
- [ ] Create `fixtures/endpoints.rs`
- [ ] Migrate top 20 most-used test files
- [ ] Create migration script for remaining files
- [ ] Run migration script
- [ ] Verify tests still pass

### **Phase 2: Configuration Defaults** (Week 2)
- [ ] Audit production port usage
- [ ] Create canonical default functions
- [ ] Add environment variable support
- [ ] Migrate orchestrator code
- [ ] Migrate discovery code
- [ ] Migrate federation code

### **Phase 3: Runtime Discovery** (Week 2-3)
- [ ] Verify capability-based routing works
- [ ] Document discovery mechanisms
- [ ] Add fallback strategies
- [ ] Test with real primals
- [ ] Test with missing primals (error handling)

### **Phase 4: IP Address Migration** (Week 3)
- [ ] Create host detection utilities
- [ ] Migrate localhost references
- [ ] Migrate private IP references
- [ ] Add dual-stack IPv6 support

---

## 📊 SUCCESS METRICS

### **Before**:
- Hardcoded ports in src/: 269
- Hardcoded IPs in src/: ~400
- Primal name hardcoding: Some (needs audit)

### **After**:
- Hardcoded ports in src/: 0 ✅
- Hardcoded IPs in src/: 0 ✅
- Primal name hardcoding: 0 ✅
- Test fixtures centralized: ✅
- Environment-aware configuration: ✅
- Capability-based discovery: ✅

---

## 🎯 VALIDATION

### **Test Coverage**:
```bash
# Verify no hardcoded production ports
rg "8080|8081|8082" crates/*/src --type rust | grep -v test | grep -v "///"

# Should return nothing!
```

### **Capability Discovery Tests**:
```rust
#[tokio::test]
async fn test_no_primal_name_hardcoding() {
    // Should discover by capability, not name
    let adapter = UniversalAdapter::new();
    
    // This should work with ANY compute provider
    let providers = adapter.discover_providers("compute").await?;
    
    // Should NOT assume it's ToadStool
    assert!(!providers.is_empty(), "Should find at least one compute provider");
    // Provider could be ToadStool, or any other compute primal!
}
```

---

**Framework Ready**: December 8, 2025  
**Status**: Ready for implementation  
**Estimated Time**: 24-28 hours total

