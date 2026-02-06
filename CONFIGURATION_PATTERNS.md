# Configuration Patterns - Zero Hardcoding Standard

**Date**: February 6, 2026  
**Status**: Official Standard for Songbird Configuration  
**Principle**: ZERO HARDCODING in production, smart defaults with env override

---

## Philosophy

**Every configuration value should be discoverable, not hardcoded.**

### The Three-Layer Model

```
┌─────────────────────────────────────────────────────────────┐
│  1. Environment Variables (Highest Priority)                 │
│     - Runtime configuration                                  │
│     - Deployment-specific                                    │
│     - Overrides all defaults                                 │
├─────────────────────────────────────────────────────────────┤
│  2. Capability Discovery (Runtime)                           │
│     - Service discovery (mDNS, DNS-SD)                       │
│     - Primal discovery (biomeOS coordination)                │
│     - Auto-configuration from ecosystem                      │
├─────────────────────────────────────────────────────────────┤
│  3. Smart Defaults (Lowest Priority)                         │
│     - Documented fallbacks                                   │
│     - Safe for development                                   │
│     - Never deployed to production                           │
└─────────────────────────────────────────────────────────────┘
```

---

## When Hardcoding is ACCEPTABLE

### ✅ Test Code

Tests use hardcoded values for **predictability and reproducibility**:

```rust
#[cfg(test)]
mod tests {
    // ✅ ACCEPTABLE: Tests need predictable addresses
    const TEST_HOST: &str = "127.0.0.1";
    const TEST_PORT: u16 = 8080;
    
    #[test]
    fn test_connection() {
        let client = Client::new(TEST_HOST, TEST_PORT);
        assert!(client.connect().is_ok());
    }
}
```

**Why acceptable**:
- Tests run in isolation
- Need deterministic behavior
- Don't affect production
- Can use `localhost` safely

### ✅ Documented Defaults with Override

Smart defaults that are **explicitly documented** and **overridable**:

```rust
impl Default for HostConfig {
    fn default() -> Self {
        Self {
            // ✅ ACCEPTABLE: Documented default, overridable via env
            orchestrator: "localhost".to_string(),
        }
    }
}

impl HostConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            // Environment variable overrides default
            orchestrator: Self::parse_host(
                "SONGBIRD_ORCHESTRATOR_HOST", 
                "localhost"  // ← Documented fallback
            ),
        })
    }
}
```

**Why acceptable**:
- Default is explicitly documented
- Can be overridden via environment
- Fails gracefully in production (uses discovery)
- Clear upgrade path

### ✅ Protocol Constants

Well-known protocol values:

```rust
// ✅ ACCEPTABLE: IPv6 loopback is a protocol constant
const IPV6_LOOPBACK: &str = "::1";

// ✅ ACCEPTABLE: Multicast address for discovery (RFC spec)
const MDNS_MULTICAST: &str = "224.0.0.251";
```

---

## When Hardcoding is UNACCEPTABLE

### ❌ Production Configuration

**NEVER** hardcode in production code without env override:

```rust
// ❌ BAD: No way to override
fn connect_to_orchestrator() -> Result<Connection> {
    let addr = "127.0.0.1:8080";  // ← Hardcoded!
    Connection::new(addr)
}

// ✅ GOOD: Uses configuration system
fn connect_to_orchestrator() -> Result<Connection> {
    let hosts = HostConfig::from_env()?;
    let ports = PortConfig::from_env()?;
    let addr = format!("{}:{}", hosts.orchestrator(), ports.orchestrator());
    Connection::new(&addr)
}
```

### ❌ Cross-Primal Communication

**NEVER** hardcode primal endpoints - use runtime discovery:

```rust
// ❌ BAD: Assumes BearDog is at localhost
fn connect_to_beardog() -> Result<BeardogClient> {
    let socket = "/tmp/beardog.sock";  // ← Hardcoded!
    BeardogClient::connect(socket)
}

// ✅ GOOD: Uses environment discovery
fn connect_to_beardog() -> Result<BeardogClient> {
    // Try environment variable first
    let socket = std::env::var("BEARDOG_SOCKET")
        // Then try Unix socket discovery
        .or_else(|_| discover_unix_socket("beardog"))
        // Finally, standard location (biomeOS coordination)
        .unwrap_or_else(|_| "/run/user/1000/biomeos/beardog.sock".to_string());
    
    BeardogClient::connect(&socket)
}
```

### ❌ Network Binding Without Configuration

**NEVER** bind to hardcoded addresses:

```rust
// ❌ BAD: Can't change bind address
async fn start_server() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    // ...
}

// ✅ GOOD: Configurable binding
async fn start_server() -> Result<()> {
    let bind_host = std::env::var("SONGBIRD_BIND_HOST")
        .unwrap_or_else(|_| "127.0.0.1".to_string());
    let bind_port = std::env::var("SONGBIRD_PORT")
        .unwrap_or_else(|_| "8080".to_string());
    
    let listener = TcpListener::bind(format!("{}:{}", bind_host, bind_port)).await?;
    // ...
}
```

---

## Official Configuration Infrastructure

Songbird provides a complete configuration system in `songbird-config`:

### Using HostConfig

```rust
use songbird_config::canonical::hardcoded_elimination::HostConfig;

// Load from environment or use documented defaults
let hosts = HostConfig::from_env()?;

// Access specific hosts
let orchestrator = hosts.orchestrator();  // SONGBIRD_ORCHESTRATOR_HOST or "localhost"
let discovery = hosts.discovery();        // SONGBIRD_DISCOVERY_HOST or "localhost"

// Check if running locally
if hosts.orchestrator_is_localhost() {
    println!("Running in development mode");
}
```

### Using PortConfig

```rust
use songbird_config::canonical::hardcoded_elimination::PortConfig;

// Load from environment or use documented defaults
let ports = PortConfig::from_env()?;

// Access specific ports
let orchestrator_port = ports.orchestrator();  // SONGBIRD_ORCHESTRATOR_PORT or 8080
let discovery_port = ports.discovery();        // SONGBIRD_DISCOVERY_PORT or 8500
```

### Using EndpointConfig

```rust
use songbird_config::canonical::hardcoded_elimination::EndpointConfig;

// Get full endpoints (host + port)
let config = EndpointConfig::from_env()?;

let orchestrator_url = config.orchestrator_endpoint();
// → "http://localhost:8080" (development)
// → "http://songbird-orchestrator:8080" (production)

let discovery_url = config.discovery_endpoint();
// → "http://localhost:8500" (development)
// → "http://songbird-discovery:8500" (production)
```

---

## Environment Variables

### Standard Variables

All Songbird services support these environment variables:

#### Host Configuration
```bash
export SONGBIRD_ORCHESTRATOR_HOST="localhost"  # or service name
export SONGBIRD_DISCOVERY_HOST="localhost"
export SONGBIRD_REGISTRY_HOST="localhost"
export SONGBIRD_SECURITY_HOST="localhost"
export SONGBIRD_STORAGE_HOST="localhost"
export SONGBIRD_COMPUTE_HOST="localhost"
export SONGBIRD_AI_HOST="localhost"
```

#### Port Configuration
```bash
export SONGBIRD_ORCHESTRATOR_PORT="8080"
export SONGBIRD_DISCOVERY_PORT="8500"
export SONGBIRD_REGISTRY_PORT="8600"
export SONGBIRD_SECURITY_PORT="8443"
export SONGBIRD_STORAGE_PORT="9000"
export SONGBIRD_COMPUTE_PORT="9100"
export SONGBIRD_AI_PORT="9200"
```

#### Binding Configuration
```bash
export SONGBIRD_BIND_HOST="127.0.0.1"  # Development
# export SONGBIRD_BIND_HOST="0.0.0.0"  # Production (all interfaces)
```

#### Primal Discovery
```bash
export BEARDOG_SOCKET="/run/user/1000/biomeos/beardog.sock"
export BIOMEOS_COORDINATION_SOCKET="/run/user/1000/biomeos/coordinator.sock"
```

---

## Migration Guide

### Step 1: Identify Hardcoded Values

```bash
# Find hardcoded IPs in production code (excluding tests)
grep -r "127\.0\.0\.1\|localhost\|0\.0\.0\.0" crates/*/src/*.rs | grep -v test
```

### Step 2: Replace with Configuration

**Before** (hardcoded):
```rust
fn start_server() -> Result<()> {
    let addr = "127.0.0.1:8080";
    TcpListener::bind(addr)?;
    // ...
}
```

**After** (configurable):
```rust
use songbird_config::canonical::hardcoded_elimination::{HostConfig, PortConfig};

fn start_server() -> Result<()> {
    let hosts = HostConfig::from_env()?;
    let ports = PortConfig::from_env()?;
    let addr = format!("{}:{}", hosts.orchestrator(), ports.orchestrator());
    TcpListener::bind(&addr)?;
    // ...
}
```

### Step 3: Document Defaults

Add clear documentation when using defaults:

```rust
impl HostConfig {
    fn parse_host(env_var: &str, default: &str) -> String {
        env::var(env_var).unwrap_or_else(|_| {
            // ✅ Document why this default is acceptable
            tracing::debug!(
                "Environment variable {} not set, using default: {}",
                env_var,
                default
            );
            default.to_string()
        })
    }
}
```

### Step 4: Test Both Paths

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_configuration() {
        // Test that defaults work
        let config = HostConfig::default();
        assert_eq!(config.orchestrator(), "localhost");
    }
    
    #[test]
    fn test_env_override() {
        // Test that environment overrides work
        std::env::set_var("SONGBIRD_ORCHESTRATOR_HOST", "custom-host");
        let config = HostConfig::from_env().unwrap();
        assert_eq!(config.orchestrator(), "custom-host");
        std::env::remove_var("SONGBIRD_ORCHESTRATOR_HOST");
    }
}
```

---

## TRUE PRIMAL Compliance

**Primal code only has self knowledge and discovers other primals at runtime.**

### Self-Knowledge (OK)

A primal knows about itself:

```rust
// ✅ OK: Songbird knows its own defaults
const SONGBIRD_DEFAULT_PORT: u16 = 8080;
```

### Cross-Primal Discovery (Required)

A primal discovers others at runtime:

```rust
// ✅ GOOD: Discover BearDog socket
let beardog = discover_primal_socket("beardog")?;

// ❌ BAD: Assume BearDog location
let beardog = "/tmp/beardog.sock";
```

### biomeOS Coordination

Production deployments use biomeOS for primal coordination:

```rust
// ✅ biomeOS sets environment variables for discovery
// BEARDOG_SOCKET=/run/user/1000/biomeos/beardog.sock
// SONGBIRD_DISCOVERY_PORT=8500

let beardog_socket = std::env::var("BEARDOG_SOCKET")
    .expect("BEARDOG_SOCKET not set - biomeOS coordination required");
```

---

## Deployment Scenarios

### Development (localhost)

```bash
# No configuration needed - uses smart defaults
cargo run
```

### Docker Compose

```yaml
services:
  songbird:
    environment:
      - SONGBIRD_ORCHESTRATOR_HOST=0.0.0.0  # Bind all interfaces
      - SONGBIRD_DISCOVERY_HOST=discovery
      - BEARDOG_SOCKET=/run/beardog/beardog.sock
    volumes:
      - /run/beardog:/run/beardog
```

### Kubernetes

```yaml
env:
  - name: SONGBIRD_ORCHESTRATOR_HOST
    value: "0.0.0.0"
  - name: SONGBIRD_DISCOVERY_HOST
    value: "songbird-discovery.default.svc.cluster.local"
  - name: BEARDOG_SOCKET
    value: "/var/run/beardog/beardog.sock"
```

### Tower (Physical Device)

```bash
# biomeOS sets all coordination variables
# /etc/biomeos/primal-env.d/songbird.env:
export SONGBIRD_BIND_HOST="0.0.0.0"
export BEARDOG_SOCKET="/run/user/1000/biomeos/beardog.sock"
export SONGBIRD_ORCHESTRATOR_PORT="8080"
```

---

## Anti-Patterns to Avoid

### ❌ Magic Numbers

```rust
// BAD: What is 8080? Why 8080?
TcpListener::bind("127.0.0.1:8080")?;

// GOOD: Named, documented, configurable
const DEFAULT_ORCHESTRATOR_PORT: u16 = 8080;
let port = env::var("SONGBIRD_ORCHESTRATOR_PORT")
    .unwrap_or_else(|_| DEFAULT_ORCHESTRATOR_PORT.to_string());
```

### ❌ Hardcoded Primal Assumptions

```rust
// BAD: Assumes BearDog is always at this socket
let beardog = BeardogClient::connect("/tmp/beardog.sock")?;

// GOOD: Discovers BearDog at runtime
let beardog = BeardogClient::from_env()?;
```

### ❌ Configuration in Code Comments

```rust
// BAD: Configuration in comment (not enforceable)
// To change the port, edit this file and recompile:
const PORT: u16 = 8080;

// GOOD: Configuration via environment
let port = env::var("SONGBIRD_PORT")
    .unwrap_or_else(|_| "8080".to_string())
    .parse()?;
```

---

## Quick Reference

| Scenario | Pattern | Example |
|----------|---------|---------|
| **Test code** | ✅ Hardcoded OK | `const TEST_PORT: u16 = 8080;` |
| **Documented default** | ✅ With env override | `env::var("PORT").unwrap_or("8080")` |
| **Production config** | ✅ Use HostConfig | `HostConfig::from_env()?` |
| **Primal discovery** | ✅ Runtime discovery | `discover_primal_socket("beardog")?` |
| **Network binding** | ✅ BIND_HOST env var | `env::var("SONGBIRD_BIND_HOST")` |
| **No override** | ❌ NEVER | `TcpListener::bind("127.0.0.1:8080")` |

---

## Related Documentation

- `crates/songbird-config/src/canonical/hardcoded_elimination.rs` - Full implementation
- `DEEP_DEBT_PHASE_4_ANALYSIS_FEB_06_2026.md` - Analysis
- `CRYPTO_CLEANUP_COMPLETE_FEB_06_2026.md` - TRUE PRIMAL compliance

---

**Standard Adopted**: February 6, 2026  
**Status**: Official Songbird Configuration Pattern  
**Compliance**: Required for all new code, encouraged for existing code

🐦 Songbird | ✅ Zero Hardcoding | 🎯 TRUE PRIMAL | 🦀 Pure Rust
