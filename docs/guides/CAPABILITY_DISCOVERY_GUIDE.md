# 🔧 Capability-Based Discovery Guide

## Overview

Songbird uses **capability-based runtime discovery** instead of hardcoded primal endpoints. This guide shows how to properly discover and connect to other primals.

---

## Principles

1. **🎯 Self-Knowledge Only** - Each primal knows only about itself
2. **🔍 Runtime Discovery** - All inter-primal communication discovered at runtime
3. **🚫 No Hardcoding** - Zero compile-time dependencies on other primals
4. **⚡ Capability-Based** - Route by what you need, not who provides it

---

## Quick Start

### Discover a Primal

```rust
use songbird_config::discovery_helpers::discover_primal;
use songbird_types::CanonicalPrimalType;

// Discover security primal (BearDog)
let security_endpoint = discover_primal(CanonicalPrimalType::Security).await?;
println!("Security primal at: {}", security_endpoint.url);

// Discover compute primal (Toadstool)
let compute_endpoint = discover_primal(CanonicalPrimalType::Compute).await?;

// Discover storage primal (Squirrel)
let storage_endpoint = discover_primal(CanonicalPrimalType::Storage).await?;
```

### Connect to Discovered Primal

```rust
// Full example: Discover and connect
let endpoint = discover_primal(CanonicalPrimalType::Security).await?;
let client = BearDogClient::new(&endpoint.url)?;
let result = client.encrypt(data).await?;
```

---

## Discovery Order

The `discover_primal` function tries multiple methods in order:

1. **✅ Capability Registry** (Preferred)
   - Runtime registered services
   - Dynamic discovery
   - Health-aware

2. **✅ Environment Variables** (Configuration)
   - `SECURITY_URL=http://localhost:8200`
   - `SECURITY_PRIMAL_URL=http://localhost:8200`
   - `SECURITY_ENDPOINT=http://localhost:8200`

3. **⚠️ Development Fallback** (Debug builds only)
   - Localhost with default ports
   - Only in `cfg!(debug_assertions)`
   - Warns when used

4. **❌ Production Failure** (No fallback)
   - Fails with clear error message
   - Tells you which env vars to set
   - Forces explicit configuration

---

## Environment Variables

### Standard Naming

```bash
# Format 1: {CAPABILITY}_URL (recommended)
export SECURITY_URL="http://[::]:8200"
export COMPUTE_URL="http://[::]:7000"
export STORAGE_URL="http://[::]:6000"

# Format 2: {PRIMAL_TYPE}_PRIMAL_URL (alternative)
export SECURITY_PRIMAL_URL="http://[::]:8200"
export COMPUTE_PRIMAL_URL="http://[::]:7000"
export STORAGE_PRIMAL_URL="http://[::]:6000"

# Format 3: {CAPABILITY}_ENDPOINT (alternative)
export SECURITY_ENDPOINT="http://[::]:8200"
export COMPUTE_ENDPOINT="http://[::]:7000"
export STORAGE_ENDPOINT="http://[::]:6000"
```

### Complete List

| Primal Type | Env Var 1 | Env Var 2 | Default Port (Dev) |
|-------------|-----------|-----------|---------------------|
| Security (BearDog) | `SECURITY_URL` | `SECURITY_PRIMAL_URL` | 8200 |
| Compute (Toadstool) | `COMPUTE_URL` | `COMPUTE_PRIMAL_URL` | 7000 |
| Storage (Squirrel) | `STORAGE_URL` | `STORAGE_PRIMAL_URL` | 6000 |
| AI | `AI_URL` | `AI_PRIMAL_URL` | 7100 |
| Orchestration (Songbird) | `ORCHESTRATION_URL` | `ORCHESTRATION_PRIMAL_URL` | 8080 |
| Federation | `FEDERATION_URL` | `FEDERATION_PRIMAL_URL` | 8090 |
| Discovery | `DISCOVERY_URL` | `DISCOVERY_PRIMAL_URL` | 5300 |
| Registry | `REGISTRY_URL` | `REGISTRY_PRIMAL_URL` | 8081 |
| Observability | `OBSERVABILITY_URL` | `OBSERVABILITY_PRIMAL_URL` | 9090 |

---

## Usage Patterns

### Pattern 1: Simple Discovery

```rust
// Just get the endpoint
let endpoint = discover_primal(CanonicalPrimalType::Security).await?;
let url = &endpoint.url;
```

### Pattern 2: Optional Discovery

```rust
// Try to discover, but continue if not found
use songbird_config::discovery_helpers::try_discover_primal;

if let Some(security) = try_discover_primal(CanonicalPrimalType::Security).await {
    // Use security primal
    enable_encryption(&security).await?;
} else {
    // Continue without encryption
    warn!("Security primal not available, continuing unencrypted");
}
```

### Pattern 3: Discover All Instances

```rust
// Get all available instances of a capability
use songbird_config::discovery_helpers::discover_all_primals;

let compute_primals = discover_all_primals(CanonicalPrimalType::Compute).await?;
for primal in compute_primals {
    println!("Compute primal at {} (health: {})", primal.url, primal.health_score);
}

// Use the healthiest one
let best = compute_primals.iter()
    .max_by(|a, b| a.health_score.partial_cmp(&b.health_score).unwrap())
    .unwrap();
```

### Pattern 4: Environment Variable Helpers

```rust
use songbird_config::discovery_helpers::env_var_for_primal;

// Get all possible env var names for a primal
let env_vars = env_var_for_primal(&CanonicalPrimalType::Security);
// Returns: ["SECURITY_URL", "SECURITY_PRIMAL_URL", "SECURITY_ENDPOINT"]

// Use in error messages
eprintln!("Set one of these environment variables: {:?}", env_vars);
```

---

## Migration Guide

### ❌ OLD: Hardcoded

```rust
// BAD: Hardcoded dependency
let beardog_client = BearDogClient::new("http://localhost:8200")?;
```

### ✅ NEW: Discovery-Based

```rust
// GOOD: Runtime discovery
let endpoint = discover_primal(CanonicalPrimalType::Security).await?;
let beardog_client = BearDogClient::new(&endpoint.url)?;
```

### ❌ OLD: Config with Hardcoded Defaults

```rust
// BAD: Hardcoded fallback
let url = env::var("BEARDOG_URL")
    .unwrap_or_else(|_| "http://localhost:8200".to_string());
```

### ✅ NEW: Discovery with Fallback

```rust
// GOOD: Discovery with proper fallback
let endpoint = discover_primal(CanonicalPrimalType::Security).await?;
let url = endpoint.url;
// Fallback handled by discover_primal()
```

---

## Primal Self-Knowledge

### ✅ CORRECT: Know Only Yourself

```rust
// In BearDog (security primal)
pub struct BearDogConfig {
    pub my_name: String,                    // "beardog"
    pub my_capabilities: Vec<String>,       // ["security", "crypto"]
    pub my_endpoint: String,                // "http://[::]:8200"
    pub my_version: String,                 // "0.9.3"
    
    // NO references to other primals!
}

// Discover others at runtime
let orchestrator = discover_primal(CanonicalPrimalType::Orchestration).await?;
```

### ❌ WRONG: Hardcoded Dependencies

```rust
// BAD: Don't do this!
pub struct BearDogConfig {
    pub orchestrator_url: String,  // ❌ Hardcoded dependency
    pub toadstool_url: String,     // ❌ Hardcoded dependency
    pub squirrel_url: String,      // ❌ Hardcoded dependency
}
```

---

## Testing

### Mock Discovery in Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_with_mock_discovery() {
        // Set test environment
        std::env::set_var("SECURITY_URL", "http://localhost:9999");
        
        // Discovery will find the test endpoint
        let endpoint = discover_primal(CanonicalPrimalType::Security).await.unwrap();
        assert_eq!(endpoint.url, "http://localhost:9999");
    }
}
```

### Development Mode

```rust
// Debug builds automatically fall back to localhost
// No need to set env vars during development

#[cfg(debug_assertions)]
{
    // This will use http://[::]:8200 automatically
    let security = discover_primal(CanonicalPrimalType::Security).await?;
}
```

---

## Production Deployment

### Docker Compose

```yaml
version: '3.8'
services:
  songbird:
    image: songbird:latest
    environment:
      - SECURITY_URL=http://beardog:8200
      - COMPUTE_URL=http://toadstool:7000
      - STORAGE_URL=http://squirrel:6000
    depends_on:
      - beardog
      - toadstool
      - squirrel
  
  beardog:
    image: beardog:latest
    ports:
      - "8200:8200"
  
  toadstool:
    image: toadstool:latest
    ports:
      - "7000:7000"
  
  squirrel:
    image: squirrel:latest
    ports:
      - "6000:6000"
```

### Kubernetes

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: songbird-config
data:
  SECURITY_URL: "http://beardog-service:8200"
  COMPUTE_URL: "http://toadstool-service:7000"
  STORAGE_URL: "http://squirrel-service:6000"
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: songbird
spec:
  template:
    spec:
      containers:
      - name: songbird
        image: songbird:latest
        envFrom:
        - configMapRef:
            name: songbird-config
```

---

## Advanced: Capability Registry

### Register Your Primal

```rust
use songbird_config::capability_discovery::CapabilityDiscovery;

// Register your primal's capabilities
let discovery = CapabilityDiscovery::new().await?;
discovery.register_service(
    "beardog-instance-1",
    "http://[::]:8200",
    vec!["security".to_string(), "crypto".to_string()],
).await?;
```

### Health Monitoring

```rust
// Endpoints include health scores
let endpoint = discover_primal(CanonicalPrimalType::Compute).await?;
if endpoint.health_score < 0.5 {
    warn!("Compute primal health is low: {}", endpoint.health_score);
}
```

---

## Troubleshooting

### Error: "No {primal} primal found"

**Solution**: Set environment variable or register in capability registry

```bash
export SECURITY_URL="http://localhost:8200"
# OR
export SECURITY_PRIMAL_URL="http://localhost:8200"
```

### Development: Want automatic fallback

**Solution**: Already works in debug builds! Just use:

```rust
// Automatically uses localhost:8200 in debug builds
let security = discover_primal(CanonicalPrimalType::Security).await?;
```

### Production: Need explicit configuration

**Good!** This is intentional. Production should never use fallbacks.

Set environment variables explicitly in your deployment.

---

## Examples

See working examples:
- `examples/capability_discovery_basic.rs` - Simple discovery
- `examples/capability_discovery_advanced.rs` - Multi-primal coordination
- `crates/songbird-network-federation/src/beardog/mod.rs` - BearDog discovery
- `crates/songbird-genesis/src/ceremony.rs` - Genesis ceremony discovery

---

## Benefits

✅ **Zero Hardcoding** - No compile-time dependencies  
✅ **Flexible Deployment** - Works in any environment  
✅ **Health-Aware** - Automatic health monitoring  
✅ **Development-Friendly** - Automatic fallbacks in debug mode  
✅ **Production-Safe** - Explicit configuration required  
✅ **Testable** - Easy to mock and test  
✅ **Sovereign** - Each primal independent  

---

🦀 **Capability-Based. Runtime Discovery. Human Dignity First.**

