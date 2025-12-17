# 🚀 MIGRATION GUIDE: Hardcoded Endpoints → Capability Discovery

**Date**: December 14, 2025  
**Purpose**: Guide for migrating from deprecated hardcoded primal endpoints to capability-based discovery  
**Status**: Infrastructure exists, migration in progress

---

## 🎯 WHY MIGRATE?

### Sovereignty Principles
- ✅ Each primal knows only itself
- ✅ Runtime discovery of other primals
- ✅ No hardcoded dependencies
- ✅ Graceful degradation when services unavailable

### Technical Benefits
- ✅ Works in any environment (dev, staging, prod, k8s, bare metal)
- ✅ No code changes when adding new primals
- ✅ Automatic load balancing and failover
- ✅ Service discovery via multiple methods (DNS, mDNS, registry)

---

## ❌ DEPRECATED PATTERN (Old Way)

```rust
use songbird_config::config::constants::network::*;

// ❌ DON'T: Hardcoded primal endpoints
#[allow(deprecated)]
let beardog_url = DEFAULT_BEARDOG_ENDPOINT;  // "http://localhost:8004"
#[allow(deprecated)]
let toadstool_url = DEFAULT_TOADSTOOL_ENDPOINT;  // "http://localhost:8001"
#[allow(deprecated)]
let nestgate_url = DEFAULT_NESTGATE_ENDPOINT;  // "http://localhost:8003"
#[allow(deprecated)]
let squirrel_url = DEFAULT_SQUIRREL_ENDPOINT;  // "http://localhost:8002"

// Problems:
// - Hardcoded localhost (won't work in production)
// - Hardcoded ports (conflicts likely)
// - Assumes specific primals exist
// - No failover or load balancing
// - Violates sovereignty principles
```

---

## ✅ NEW PATTERN (Capability Discovery)

### Method 1: Capability-Based Discovery (Recommended)

```rust
use songbird_config::capability_endpoints::{
    CapabilityEndpointResolver, CapabilityType
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create resolver (caches results, TTL: 5 minutes)
    let resolver = CapabilityEndpointResolver::new();
    
    // ✅ DO: Discover by capability, not by primal name
    let security_endpoint = resolver
        .get_endpoint(CapabilityType::Security)
        .await?;
    
    let compute_endpoint = resolver
        .get_endpoint(CapabilityType::Compute)
        .await?;
    
    let storage_endpoint = resolver
        .get_endpoint(CapabilityType::Storage)
        .await?;
    
    let ai_endpoint = resolver
        .get_endpoint(CapabilityType::Ai)
        .await?;
    
    println!("Security: {}", security_endpoint);
    println!("Compute: {}", compute_endpoint);
    println!("Storage: {}", storage_endpoint);
    println!("AI: {}", ai_endpoint);
    
    Ok(())
}
```

**Benefits**:
- Any primal can provide any capability
- Automatic discovery via env → registry → DNS → mDNS
- Caching with TTL
- Confidence scoring for multiple providers
- Graceful fallback

### Method 2: Environment Variables (Backwards Compatible)

```rust
use std::env;

// ✅ DO: Environment variable (backwards compatible)
let security_endpoint = env::var("CAPABILITY_SECURITY_ENDPOINT")
    .or_else(|_| env::var("BEARDOG_ENDPOINT"))
    .unwrap_or_else(|_| {
        // Fallback to localhost for development only
        if env::var("SONGBIRD_ENV").as_deref() == Ok("development") {
            "http://localhost:8443".to_string()
        } else {
            panic!("CAPABILITY_SECURITY_ENDPOINT must be set in production")
        }
    });
```

### Method 3: Full Discovery with Graceful Degradation

```rust
use songbird_config::capability_based_runtime_discovery::{
    CapabilityResolver, CapabilityRequest
};

async fn get_compute_provider() -> Result<String, Box<dyn std::error::Error>> {
    let resolver = CapabilityResolver::new();
    
    let request = CapabilityRequest {
        capability: "compute".to_string(),
        required_features: vec!["gpu".to_string()],
        preferred_providers: None,
        timeout: std::time::Duration::from_secs(30),
    };
    
    // ✅ DO: Multi-method discovery with fallback
    match resolver.resolve(request).await {
        Ok(provider) => Ok(provider.endpoint),
        Err(e) => {
            // Fallback to environment
            if let Ok(endpoint) = env::var("CAPABILITY_COMPUTE_ENDPOINT") {
                return Ok(endpoint);
            }
            // Last resort for development
            if env::var("SONGBIRD_ENV").as_deref() == Ok("development") {
                return Ok("http://localhost:8082".to_string());
            }
            Err(e)
        }
    }
}
```

---

## 🔧 DISCOVERY METHODS

The `CapabilityEndpointResolver` tries these methods in order:

### 1. Environment Variables (Highest Priority)
```bash
export CAPABILITY_SECURITY_ENDPOINT="https://beardog.example.com:8443"
export CAPABILITY_COMPUTE_ENDPOINT="http://toadstool.example.com:8001"
export CAPABILITY_STORAGE_ENDPOINT="https://nestgate.example.com:8003"
export CAPABILITY_AI_ENDPOINT="http://squirrel.example.com:8002"
```

### 2. Service Registry (Consul, etcd, etc.)
```bash
export SERVICE_REGISTRY_ENDPOINT="http://consul.example.com:8500"
```

### 3. Container Metadata (Kubernetes, Docker)
Automatically discovers services via:
- Kubernetes service DNS
- Docker network discovery
- Container orchestration metadata

### 4. DNS-SD / mDNS
Discovers local services via:
- DNS Service Discovery (RFC 6763)
- Multicast DNS (Bonjour/Avahi)

---

## 📝 ENVIRONMENT VARIABLE REFERENCE

### Capability Endpoints (Recommended)
```bash
# Modern capability-based (any primal can provide these)
CAPABILITY_SECURITY_ENDPOINT=https://security-provider:8443
CAPABILITY_STORAGE_ENDPOINT=https://storage-provider:9000
CAPABILITY_COMPUTE_ENDPOINT=http://compute-provider:8001
CAPABILITY_AI_ENDPOINT=http://ai-provider:8002
CAPABILITY_ORCHESTRATION_ENDPOINT=http://orchestrator:8080
```

### Legacy Primal Endpoints (Backwards Compatible)
```bash
# Legacy primal-specific (still supported, but deprecated)
BEARDOG_ENDPOINT=https://beardog:8443
NESTGATE_ENDPOINT=https://nestgate:9000
TOADSTOOL_ENDPOINT=http://toadstool:8001
SQUIRREL_ENDPOINT=http://squirrel:8002
```

### Discovery Configuration
```bash
# Service registry for discovery
SERVICE_REGISTRY_ENDPOINT=http://consul:8500

# Discovery settings
DISCOVERY_CACHE_TTL_SECS=300
DISCOVERY_TIMEOUT_SECS=30
ENABLE_MDNS_DISCOVERY=true
ENABLE_DNS_SD_DISCOVERY=true
```

---

## 🔄 MIGRATION STEPS

### Step 1: Update Environment Variables
```bash
# Before (hardcoded in code)
# No configuration needed - hardcoded localhost

# After (environment-based)
export CAPABILITY_SECURITY_ENDPOINT="https://beardog.prod:8443"
export CAPABILITY_COMPUTE_ENDPOINT="http://toadstool.prod:8001"
export CAPABILITY_STORAGE_ENDPOINT="https://nestgate.prod:9000"
export CAPABILITY_AI_ENDPOINT="http://squirrel.prod:8002"
```

### Step 2: Update Code
```rust
// Before
#[allow(deprecated)]
let endpoint = DEFAULT_BEARDOG_ENDPOINT;

// After
let resolver = CapabilityEndpointResolver::new();
let endpoint = resolver
    .get_endpoint(CapabilityType::Security)
    .await?;
```

### Step 3: Test Discovery
```bash
# Test discovery is working
cargo run --bin songbird-cli -- discover --capability security
cargo run --bin songbird-cli -- discover --capability compute
```

### Step 4: Deploy Incrementally
1. ✅ Update dev environment first
2. ✅ Test with environment variables
3. ✅ Deploy to staging with service registry
4. ✅ Enable DNS-SD/mDNS for production
5. ✅ Remove hardcoded fallbacks

---

## 🧪 TESTING

### Test Discovery in Development
```bash
# Set up test environment
export CAPABILITY_SECURITY_ENDPOINT="http://localhost:8443"
export CAPABILITY_COMPUTE_ENDPOINT="http://localhost:8001"

# Test capability resolution
cargo test --package songbird-config capability_endpoints
```

### Test with Multiple Providers
```bash
# Multiple providers for same capability
export CAPABILITY_COMPUTE_ENDPOINT="http://compute1:8001,http://compute2:8001"

# Resolver will load balance and failover
```

---

## 📊 MIGRATION STATUS

### Deprecated Constants (Dec 14, 2025)
- ✅ DEFAULT_TOADSTOOL_ENDPOINT
- ✅ DEFAULT_TOADSTOOL_PORT
- ✅ DEFAULT_SQUIRREL_ENDPOINT
- ✅ DEFAULT_SQUIRREL_PORT
- ✅ DEFAULT_NESTGATE_ENDPOINT
- ✅ DEFAULT_NESTGATE_PORT
- ✅ DEFAULT_BEARDOG_ENDPOINT
- ✅ DEFAULT_BEARDOG_PORT

### Infrastructure Ready
- ✅ CapabilityEndpointResolver (full implementation)
- ✅ CapabilityResolver (runtime discovery)
- ✅ ServiceLocator (self-aware config)
- ✅ Multi-method discovery (env, registry, DNS, mDNS)
- ✅ Caching with TTL
- ✅ Confidence scoring

### Next Steps
1. Update `PrimalConfig::default()` to use CapabilityResolver
2. Update examples to show capability discovery
3. Add integration tests for discovery
4. Document service registry setup

---

## 🎓 EXAMPLES

### Example 1: Security Service Discovery
```rust
use songbird_config::capability_endpoints::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let resolver = CapabilityEndpointResolver::new();
    
    // Discover security provider
    let endpoint = resolver
        .get_endpoint(CapabilityType::Security)
        .await?;
    
    println!("Security service: {}", endpoint);
    // Could be BearDog, or ANY other security provider!
    
    Ok(())
}
```

### Example 2: Multi-Capability Application
```rust
use songbird_config::capability_endpoints::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let resolver = CapabilityEndpointResolver::new();
    
    // Discover all needed capabilities
    let security = resolver.get_endpoint(CapabilityType::Security).await?;
    let storage = resolver.get_endpoint(CapabilityType::Storage).await?;
    let compute = resolver.get_endpoint(CapabilityType::Compute).await?;
    let ai = resolver.get_endpoint(CapabilityType::Ai).await?;
    
    // Use them (any provider works!)
    run_application(security, storage, compute, ai).await
}
```

---

## ✅ BENEFITS OF MIGRATION

1. **Sovereignty** - Each primal independent, discovers others
2. **Flexibility** - Any primal can provide any capability
3. **Scalability** - Automatic load balancing across providers
4. **Reliability** - Failover to backup providers
5. **Portability** - Works in any environment (dev, k8s, bare metal)
6. **Zero Hardcoding** - No primal names in code
7. **Future-Proof** - New primals work without code changes

---

## 📚 FURTHER READING

- `crates/songbird-config/src/capability_endpoints.rs` - Full resolver implementation
- `crates/songbird-config/src/capability_based_runtime_discovery.rs` - Discovery architecture
- `specs/UNIVERSAL_PRIMAL_SDK_INTEGRATION_SPECIFICATION.md` - Universal primal patterns
- `docs/DEEP_DEBT_SOLUTIONS.md` - Evolution philosophy

---

**Migration Status**: Infrastructure complete, systematic migration in progress  
**Timeline**: 2-4 weeks for full migration  
**Priority**: High (sovereignty compliance)

🚀 **Start migrating today!**

