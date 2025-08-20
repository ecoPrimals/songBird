# 🔧 Hardcoded Value Elimination Guide

## 🎯 Purpose

This guide shows how to eliminate hardcoded addresses, ports, and configuration values from Songbird codebase using the unified configuration system.

## ✅ Before & After Examples

### ❌ Old Pattern (Hardcoded)
```rust
// BAD: Hardcoded localhost and port
let endpoint = "http://localhost:8080".to_string();
let database_url = "postgresql://localhost:5432/demo".to_string();
let bind_addr = "127.0.0.1:8080";
```

### ✅ New Pattern (Configurable)
```rust
// GOOD: Use unified configuration
use songbird_config::UnifiedSongbirdConfig;

let config = UnifiedSongbirdConfig::from_env();
let endpoint = format!("http://{}:{}", config.network.bind_address, config.network.port);
let database_url = format!("postgresql://{}:5432/demo", config.network.bind_address);
let bind_addr = format!("{}:{}", config.network.bind_address, config.network.port);
```

### ✅ External Services Pattern
```rust
// EXCELLENT: Use external service endpoints
let config = UnifiedSongbirdConfig::from_env();
let beardog_endpoint = config.network.external_services.beardog.full_url();
let squirrel_endpoint = config.network.external_services.squirrel.full_url();
```

## 📋 Migration Checklist

### 1. **Import Unified Config**
```rust
use songbird_config::UnifiedSongbirdConfig;
```

### 2. **Load Configuration**
```rust
let config = UnifiedSongbirdConfig::from_env();
```

### 3. **Replace Common Patterns**

| Hardcoded Value | Replacement |
|----------------|-------------|
| `"127.0.0.1"` | `config.network.bind_address` |
| `"localhost"` | `config.network.bind_address` |
| `:8080` | `config.network.port` |
| `:8443` | `config.network.external_services.beardog.port` |
| `:9000` | `config.network.external_services.nestgate.port` |
| `:8002` | `config.network.external_services.squirrel.port` |

### 4. **Service Discovery Endpoints**
```rust
// Old: Hardcoded consul
let consul_addr = "http://localhost:8500";

// New: Configurable
let consul_endpoints = &config.network.external_services.service_discovery.consul;
let consul_addr = consul_endpoints[0].full_url();
```

## 🔧 Configuration Structure

### Environment Variables
```bash
# Core networking
export SONGBIRD_BIND_ADDRESS="0.0.0.0"
export SONGBIRD_PORT="8080"

# External services
export BEARDOG_HOST="beardog.internal"
export BEARDOG_PORT="8443" 
export BEARDOG_SCHEME="https"

export SQUIRREL_HOST="ai.internal"
export SQUIRREL_PORT="8002"
```

### TOML Configuration
```toml
[network]
bind_address = "0.0.0.0"
port = 8080

[network.external_services.beardog]
host = "beardog.internal"
port = 8443
scheme = "https"

[network.external_services.squirrel]
host = "ai.internal"  
port = 8002
scheme = "http"
```

## 🎯 Common Migration Patterns

### 1. **Simple Endpoint Replacement**
```rust
// Old
format!("http://localhost:{}", port)

// New  
let config = UnifiedSongbirdConfig::from_env();
format!("http://{}:{}", config.network.bind_address, port)
```

### 2. **Default Values with Environment Override**
```rust
// Old
std::env::var("CUSTOM_ENDPOINT")
    .unwrap_or_else(|_| "localhost:8080".to_string())

// New
std::env::var("CUSTOM_ENDPOINT")
    .unwrap_or_else(|_| {
        let config = UnifiedSongbirdConfig::from_env();
        format!("{}:{}", config.network.bind_address, config.network.port)
    })
```

### 3. **Service Discovery Arrays**
```rust
// Old
vec![
    "http://localhost:8500".to_string(),
    "http://127.0.0.1:2379".to_string(),
]

// New
let config = UnifiedSongbirdConfig::from_env();
config.network.external_services.service_discovery.consul
    .iter()
    .map(|endpoint| endpoint.full_url())
    .collect()
```

## 🚀 Benefits

- ✅ **Production Ready**: No hardcoded localhost values
- ✅ **Environment Flexible**: Works in dev, staging, production
- ✅ **Testable**: Easy to override for testing
- ✅ **Maintainable**: Single source of configuration truth
- ✅ **Secure**: No hardcoded credentials or internal addresses

## ⚠️ Important Notes

1. **Always load config once** at the start of functions for performance
2. **Use environment variables** for deployment-specific overrides
3. **Provide sensible defaults** for development environments
4. **Document new configuration options** in the config files
5. **Test with different environments** to ensure flexibility

## 🔍 Finding Hardcoded Values

### Search Commands
```bash
# Find hardcoded localhost
grep -r "localhost\|127\.0\.0\.1" crates/

# Find hardcoded ports  
grep -r ":8080\|:3000\|:5432" crates/

# Find hardcoded URLs
grep -r "http://.*:[0-9]" crates/
```

### Common Locations
- Examples and demos
- Default configurations
- Test fixtures
- Service endpoints
- Health check URLs

## 📝 Testing Your Changes

```bash
# Test with default config
cargo run --example demo_orchestration

# Test with custom config
export SONGBIRD_BIND_ADDRESS="192.168.1.100"
export SONGBIRD_PORT="9090"
cargo run --example demo_orchestration

# Test with TOML config
SONGBIRD_CONFIG_FILE=./examples/config/songbird-demo.toml \
cargo run --example demo_orchestration
```

---

**Remember**: Every hardcoded value eliminated makes Songbird more production-ready! 🎯 