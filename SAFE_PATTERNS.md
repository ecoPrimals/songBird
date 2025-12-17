# 🛡️ Safe Patterns Guide - Songbird
**Modern, Idiomatic, Production-Ready Rust**

---

## 🎯 Core Principles

### 1. **Zero Hardcoded Knowledge**
Primals know ONLY themselves. All other primals discovered at runtime.

### 2. **Fast AND Safe**
Performance without unsafe code. Modern Rust is fast enough.

### 3. **Capability-Based Discovery**
Discover by what services DO, not what they're CALLED.

### 4. **Fail Fast with Guidance**
Errors are explicit, logged, and guide users to solutions.

---

## 📋 Pattern Categories

- [Zero Hardcoding Patterns](#zero-hardcoding-patterns)
- [Discovery Patterns](#discovery-patterns)
- [Error Handling Patterns](#error-handling-patterns)
- [Configuration Patterns](#configuration-patterns)
- [Testing Patterns](#testing-patterns)

---

## 🚫 Zero Hardcoding Patterns

### Pattern 1: No Hardcoded Fallbacks

#### ❌ Anti-Pattern: Hardcoded Localhost Fallback
```rust
// SOVEREIGNTY VIOLATION: Assumes primal location
let endpoint = env::var("COMPUTE_ENDPOINT")
    .unwrap_or_else(|_| "http://127.0.0.1:8001".to_string());
```

**Problems**:
- Assumes primal at specific IP/port
- Silent failure in production
- Violates sovereignty principles
- Not portable across environments

#### ✅ Safe Pattern: Discovery Signal
```rust
let endpoint = env::var("COMPUTE_ENDPOINT")
    .unwrap_or_else(|_| {
        tracing::warn!(
            "COMPUTE_ENDPOINT not set. Use RuntimeDiscoveryEngine::discover_by_capability(\"compute\") for dynamic discovery"
        );
        String::new() // Empty string signals discovery needed
    });
```

**Benefits**:
- No assumptions about primal location
- Clear guidance in logs
- Signals need for discovery
- Fail fast with actionable feedback

---

### Pattern 2: Runtime Capability Discovery

#### ❌ Anti-Pattern: Name-Based Hardcoding
```rust
// Assumes service name and location
let url = format!("http://toadstool-service:8080/api");
```

**Problems**:
- Couples to specific service name
- Assumes network topology
- Brittle across environments

#### ✅ Safe Pattern: Capability-Based Discovery
```rust
use songbird_config::runtime_discovery::RuntimeDiscoveryEngine;

// Discover ANY service providing the capability
let discovery = RuntimeDiscoveryEngine::new();
let service = discovery
    .discover_by_capability("compute")
    .await?;

// Use discovered endpoint
let url = format!("{}/api", service.endpoint);
```

**Benefits**:
- Works with any compute provider
- Environment agnostic
- Automatic failover to different providers
- True sovereignty compliance

---

### Pattern 3: Environment-First with Discovery Fallback

#### ✅ Best Practice: Hybrid Approach
```rust
use songbird_config::runtime_discovery::{RuntimeDiscoveryEngine, DiscoveredService};
use tokio::sync::OnceCell;

static COMPUTE_ENDPOINT: OnceCell<String> = OnceCell::const_new();

async fn get_compute_endpoint() -> Result<String, Box<dyn std::error::Error>> {
    COMPUTE_ENDPOINT
        .get_or_try_init(|| async {
            // Try environment variable first (explicit user config)
            if let Ok(endpoint) = std::env::var("COMPUTE_ENDPOINT") {
                tracing::info!("Using compute endpoint from environment: {}", endpoint);
                return Ok(endpoint);
            }

            // Fall back to runtime discovery
            tracing::info!("Discovering compute provider via RuntimeDiscoveryEngine");
            let discovery = RuntimeDiscoveryEngine::new();
            let service = discovery
                .discover_by_capability("compute")
                .await?;

            tracing::info!(
                "Discovered compute provider at {} via {}",
                service.endpoint,
                service.discovered_via
            );

            Ok(service.endpoint)
        })
        .await
        .map(|s| s.clone())
}
```

**Benefits**:
- Respects explicit user configuration
- Falls back to automatic discovery
- Caches result for performance
- Full observability via logging

---

## 🔍 Discovery Patterns

### Pattern 4: Discovery by Capability (Primary Method)

```rust
use songbird_config::runtime_discovery::RuntimeDiscoveryEngine;

// Create discovery engine
let discovery = RuntimeDiscoveryEngine::new();

// Discover services by capability
let compute = discovery.discover_by_capability("compute").await?;
let storage = discovery.discover_by_capability("storage").await?;
let ai = discovery.discover_by_capability("ai").await?;
let security = discovery.discover_by_capability("security").await?;

// Use discovered services
println!("Compute: {} (via {})", compute.endpoint, compute.discovered_via);
println!("Storage: {} (via {})", storage.endpoint, storage.discovered_via);
```

**Discovery Order**:
1. Environment variables (highest priority)
2. mDNS local network discovery
3. Central service registry
4. Peer announcements

---

### Pattern 5: Discovery with Caching

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

struct ServiceCache {
    cache: Arc<RwLock<HashMap<String, String>>>,
    discovery: RuntimeDiscoveryEngine,
}

impl ServiceCache {
    fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            discovery: RuntimeDiscoveryEngine::new(),
        }
    }

    async fn get_endpoint(&self, capability: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(endpoint) = cache.get(capability) {
                return Ok(endpoint.clone());
            }
        }

        // Discover and cache
        let service = self.discovery.discover_by_capability(capability).await?;
        let endpoint = service.endpoint.clone();

        {
            let mut cache = self.cache.write().await;
            cache.insert(capability.to_string(), endpoint.clone());
        }

        Ok(endpoint)
    }
}
```

**Benefits**:
- Reduces discovery overhead
- Improves performance
- Still supports re-discovery if needed

---

## ⚠️ Error Handling Patterns

### Pattern 6: Proper Error Context

#### ❌ Anti-Pattern: Generic Errors
```rust
.map_err(|_| "discovery failed")?
```

#### ✅ Safe Pattern: Rich Error Context
```rust
.map_err(|e| SongbirdError::discovery(format!(
    "Failed to discover {} provider: {}. \
     Set {}_ENDPOINT environment variable or ensure service is advertising via mDNS/registry.",
    capability, e, capability.to_uppercase()
)))?
```

**Benefits**:
- Clear error messages
- Actionable guidance
- Easy debugging
- Professional UX

---

### Pattern 7: No Unwrap in Production

#### ❌ Anti-Pattern: Unwrap Everywhere
```rust
let endpoint = env::var("ENDPOINT").unwrap();
let service = discover().await.unwrap();
```

#### ✅ Safe Pattern: Proper Error Propagation
```rust
let endpoint = env::var("ENDPOINT")
    .map_err(|e| SongbirdError::configuration(
        format!("ENDPOINT environment variable not set: {e}")
    ))?;

let service = discover().await
    .map_err(|e| SongbirdError::discovery(
        format!("Service discovery failed: {e}")
    ))?;
```

**Benefits**:
- No panics in production
- Proper error types
- Error context preserved
- Graceful degradation possible

---

## ⚙️ Configuration Patterns

### Pattern 8: Environment-Aware Defaults

#### ✅ Safe Pattern: Detect Environment
```rust
use songbird_types::error_helpers::SafeEnv;

pub fn get_bind_address() -> String {
    // Try explicit configuration first
    if let Ok(addr) = SafeEnv::get("SONGBIRD_BIND_ADDRESS") {
        return addr;
    }

    // Detect environment and provide appropriate default
    if SafeEnv::get("KUBERNETES_SERVICE_HOST").is_ok() {
        "0.0.0.0".to_string() // Kubernetes: bind to all interfaces
    } else if SafeEnv::get("CONTAINER").is_ok() {
        "0.0.0.0".to_string() // Docker: bind to all interfaces
    } else if SafeEnv::get("SONGBIRD_ENV").as_deref() == Ok("production") {
        "0.0.0.0".to_string() // Production: bind to all interfaces
    } else {
        "127.0.0.1".to_string() // Development: localhost only
    }
}
```

**Benefits**:
- Secure by default (localhost in dev)
- Production-ready (0.0.0.0 in prod)
- User override always possible
- No hardcoded magic values

---

### Pattern 9: Structured Configuration

#### ✅ Safe Pattern: Typed Configuration
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Service endpoint (from env or discovery)
    #[serde(default)]
    pub endpoint: Option<String>,
    
    /// Discovery capability if endpoint not set
    #[serde(default)]
    pub capability: Option<String>,
    
    /// Timeout for service calls
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    
    /// Enable automatic failover
    #[serde(default = "default_true")]
    pub enable_failover: bool,
}

fn default_timeout() -> u64 { 30 }
fn default_true() -> bool { true }

impl ServiceConfig {
    pub async fn resolve_endpoint(&self) -> Result<String, Box<dyn std::error::Error>> {
        // Explicit endpoint takes priority
        if let Some(ref endpoint) = self.endpoint {
            return Ok(endpoint.clone());
        }

        // Discover by capability
        if let Some(ref capability) = self.capability {
            let discovery = RuntimeDiscoveryEngine::new();
            let service = discovery.discover_by_capability(capability).await?;
            return Ok(service.endpoint);
        }

        Err("Neither endpoint nor capability specified".into())
    }
}
```

**Benefits**:
- Type-safe configuration
- Serializable (TOML/JSON/YAML)
- Clear defaults
- Self-documenting

---

## 🧪 Testing Patterns

### Pattern 10: Test Isolation

#### ✅ Safe Pattern: Isolated Test Fixtures
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    // Test fixtures are ONLY in test modules
    fn test_endpoint() -> String {
        "http://localhost:8080".to_string() // OK in tests
    }

    #[tokio::test]
    async fn test_discovery() {
        // Set test environment
        std::env::set_var("COMPUTE_ENDPOINT", test_endpoint());
        
        // Test discovery
        let result = discover_compute().await;
        assert!(result.is_ok());
        
        // Cleanup
        std::env::remove_var("COMPUTE_ENDPOINT");
    }
}
```

**Benefits**:
- Hardcoding only in tests
- Clean separation
- Production code stays clean
- Easy to maintain

---

### Pattern 11: Mock Services for Testing

#### ✅ Safe Pattern: Test-Only Mocks
```rust
#[cfg(test)]
pub mod mock {
    use super::*;
    
    pub struct MockDiscovery {
        responses: HashMap<String, String>,
    }

    impl MockDiscovery {
        pub fn new() -> Self {
            let mut responses = HashMap::new();
            responses.insert("compute".to_string(), "http://localhost:9001".to_string());
            responses.insert("storage".to_string(), "http://localhost:9002".to_string());
            Self { responses }
        }

        pub async fn discover(&self, capability: &str) -> Result<String, Box<dyn std::error::Error>> {
            self.responses
                .get(capability)
                .cloned()
                .ok_or_else(|| format!("No mock for capability: {}", capability).into())
        }
    }
}
```

**Benefits**:
- Mocks only in test code
- Clear #[cfg(test)] guards
- No production mock leakage

---

## 🚀 Performance Patterns

### Pattern 12: Zero-Copy Where It Matters

#### ✅ Safe Pattern: Bytes for Network Data
```rust
use bytes::Bytes;

// Use Bytes for zero-copy network buffers
pub struct NetworkMessage {
    payload: Bytes, // Reference-counted, cheap to clone
}

impl NetworkMessage {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            payload: Bytes::from(data), // Single allocation
        }
    }

    pub fn clone_payload(&self) -> Bytes {
        self.payload.clone() // Just increments refcount, no copy
    }
}
```

**Benefits**:
- Zero-copy when possible
- Safe (no unsafe code)
- Performance equivalent to unsafe
- Easier to maintain

---

### Pattern 13: Smart Cloning

#### ❌ Anti-Pattern: Clone Everything
```rust
fn process(data: String) {
    for _ in 0..1000 {
        let copy = data.clone(); // Expensive!
        do_something(copy);
    }
}
```

#### ✅ Safe Pattern: Borrow Where Possible
```rust
fn process(data: &str) {
    for _ in 0..1000 {
        do_something(data); // Zero-cost borrowing
    }
}
```

#### ✅ Safe Pattern: Arc for Shared Data
```rust
use std::sync::Arc;

fn share(data: Arc<String>) {
    for _ in 0..1000 {
        let shared = data.clone(); // Just refcount increment
        do_something(shared);
    }
}
```

---

## 📊 Migration Checklist

When migrating code to safe patterns:

- [ ] Replace hardcoded IPs with env vars or discovery
- [ ] Replace hardcoded ports with env vars or discovery
- [ ] Add clear warning logs when discovery is needed
- [ ] Use RuntimeDiscoveryEngine for capability-based discovery
- [ ] Remove all `.unwrap()` from production code paths
- [ ] Add proper error context to all errors
- [ ] Use structured logging (tracing)
- [ ] Isolate any test hardcoding to #[cfg(test)]
- [ ] Document configuration with examples
- [ ] Verify builds pass
- [ ] Verify tests pass
- [ ] Check for no new clippy warnings

---

## 🎯 Quick Reference

### Environment Variables
```bash
# Explicit endpoint configuration
export COMPUTE_ENDPOINT="http://192.168.1.50:8001"
export STORAGE_ENDPOINT="http://192.168.1.51:8002"
export AI_ENDPOINT="http://192.168.1.52:8003"
export SECURITY_ENDPOINT="https://192.168.1.53:8443"

# Discovery configuration
export REGISTRY_ENDPOINT="http://consul.local:8500"
export SONGBIRD_BIND_ADDRESS="0.0.0.0:8080"
export SONGBIRD_ENV="production"
```

### Discovery Code
```rust
use songbird_config::runtime_discovery::RuntimeDiscoveryEngine;

// Discover by capability
let discovery = RuntimeDiscoveryEngine::new();
let compute = discovery.discover_by_capability("compute").await?;
let storage = discovery.discover_by_capability("storage").await?;

// Use discovered endpoints
println!("Compute: {}", compute.endpoint);
println!("Storage: {}", storage.endpoint);
```

---

## 🔗 Related Documentation

- `COMPREHENSIVE_AUDIT_REPORT_DEC_16_2025_FINAL.md` - Full audit
- `HARDCODING_MIGRATION_PHASE1_DEC16.md` - Migration guide
- `crates/songbird-config/src/runtime_discovery.rs` - Reference impl
- `specs/INDIVIDUAL_HUMAN_DIGNITY_SPECIFICATION.md` - Sovereignty principles

---

**Version**: 1.0  
**Date**: December 16, 2025  
**Status**: ESTABLISHED - Use these patterns for all new code

**Bottom Line**: Zero hardcoded knowledge. Capability-based discovery. Fail fast with guidance. Modern idiomatic Rust.

