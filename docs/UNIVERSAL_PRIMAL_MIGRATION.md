# Universal Primal Configuration Migration Guide

## Overview

Songbird has migrated from hardcoded primal-specific configurations to a universal, extensible primal registry system. This change enables true universal extensibility where new primal types can be added without any code changes.

## What Changed

### Before (Hardcoded Approach)
```rust
pub struct SongbirdConfig {
    pub beardog: Option<BearDogConfig>,      // Hardcoded BearDog
    pub toadstool: Option<ToadstoolConfig>,  // Hardcoded Toadstool  
    // Adding new primal = code changes required!
}
```

### After (Universal Approach)
```rust
pub struct SongbirdConfig {
    pub primal_registry: Option<PrimalRegistry>,  // Universal for ANY primal
    // Deprecated legacy fields for backward compatibility
    pub beardog: Option<BearDogConfig>,    // [DEPRECATED]
    pub toadstool: Option<ToadstoolConfig>, // [DEPRECATED]
}
```

## Benefits

### ✅ Universal Extensibility
- **Zero Code Changes**: New primals added via configuration only
- **Auto-Discovery**: Primals discovered dynamically on the network
- **Capability-Based**: Match primals by capabilities, not hardcoded names

### ✅ Backward Compatibility
- Existing configurations continue to work unchanged
- Automatic migration from legacy to universal system
- Deprecation warnings guide users to new system

### ✅ Advanced Features
- **Quality of Service**: Latency, throughput, availability metrics
- **Load Balancing**: Health-based, latency-based routing
- **Health Monitoring**: Automatic health checks and failover
- **Flexible Authentication**: API keys, mTLS, OAuth 2.0, custom methods

## Migration Path

### Step 1: Update Configuration Files

**Legacy Configuration (still works, but deprecated):**
```toml
[beardog]
enabled = true
[beardog.endpoint]
primary_url = "https://localhost:8443"

[toadstool] 
enabled = true
[toadstool.endpoint]
primary_url = "http://localhost:8082"
```

**New Universal Configuration:**
```toml
[primal_registry]
# Auto-discovery enabled
[primal_registry.auto_discovery]
enabled = true
discovery_methods = ["environment", "mdns", "network_scan"]

# BearDog as universal primal
[[primal_registry.primals]]
primal_type = "beardog"
display_name = "BearDog Security"
enabled = true

[primal_registry.primals.endpoint]
primary_url = "https://beardog.internal:8443"
fallback_urls = ["https://beardog-backup.internal:8443"]

[[primal_registry.primals.capabilities]]
capability_type = "security"
version = "2.0"

# Toadstool as universal primal
[[primal_registry.primals]]
primal_type = "toadstool" 
display_name = "Toadstool Compute"
enabled = true

[primal_registry.primals.endpoint]
primary_url = "http://toadstool.internal:8082"

[[primal_registry.primals.capabilities]]
capability_type = "compute"
version = "3.0"

# NEW PRIMAL - no code changes needed!
[[primal_registry.primals]]
primal_type = "phoenix"
display_name = "Phoenix AI Assistant"
enabled = true

[primal_registry.primals.endpoint]
primary_url = "https://phoenix.ai.internal:8444"

[[primal_registry.primals.capabilities]]
capability_type = "ai_assistance"
version = "1.0"
```

### Step 2: Update Code Usage

**Legacy API (deprecated but still works):**
```rust
// OLD WAY - hardcoded, deprecated
if config.is_beardog_enabled() {
    let beardog_config = config.get_beardog_config();
    // Use BearDog...
}
```

**New Universal API:**
```rust
// NEW WAY - universal, extensible
if config.is_primal_enabled("beardog") {
    let primal_config = config.get_primal_config("beardog").unwrap();
    // Use any primal...
}

// Find primals by capability - works with any primal type!
let security_primals = config.find_primals_with_capability("security");
let compute_primals = config.find_primals_with_capability("compute");
let ai_primals = config.find_primals_with_capability("ai_assistance");
```

### Step 3: Enable Auto-Discovery

The universal system can automatically discover primals on your network:

```toml
[primal_registry.auto_discovery]
enabled = true
discovery_methods = ["environment", "mdns", "network_scan"] 
scan_ranges = ["127.0.0.0/8", "10.0.0.0/8"]
scan_ports = [8080, 8081, 8082, 8083, 8443]
discovery_interval = "5m"
```

## Adding New Primals

### Adding "Phoenix AI" (Zero Code Changes!)

1. **Configuration Only:**
```toml
[[primal_registry.primals]]
primal_type = "phoenix"
display_name = "Phoenix AI Assistant" 
enabled = true

[primal_registry.primals.endpoint]
primary_url = "https://phoenix.ai.internal:8444"

[[primal_registry.primals.capabilities]]
capability_type = "ai_assistance"
version = "1.0"
```

2. **Use Immediately:**
```rust
// Find AI assistance primals
let ai_primals = config.find_primals_with_capability("ai_assistance");

// Or get Phoenix specifically
if config.is_primal_enabled("phoenix") {
    let phoenix = config.get_primal_config("phoenix").unwrap();
    println!("Phoenix AI available at: {}", phoenix.endpoint.primary_url);
}
```

## Capability-Based Primal Selection

The universal system supports intelligent primal selection based on capabilities:

```rust
// Find best primal for security (could be BearDog, or any security primal)
let security_primals = config.find_primals_with_capability("security");
let best_security = security_primals
    .into_iter()
    .max_by(|a, b| {
        let a_qos = a.get_capability("security").unwrap().qos_metrics.availability.unwrap_or(0.0);
        let b_qos = b.get_capability("security").unwrap().qos_metrics.availability.unwrap_or(0.0);
        a_qos.partial_cmp(&b_qos).unwrap()
    });

// Automatically selects highest-availability security primal
```

## Environment Variables

The universal system supports environment-based configuration:

```bash
# Auto-discover Phoenix AI
export SONGBIRD_PHOENIX_ENDPOINT="https://phoenix.ai.internal:8444"
export SONGBIRD_PHOENIX_ENABLED="true"

# The system will automatically register Phoenix as a primal
```

## Backward Compatibility Timeline

- **Phase 1 (Current)**: Both systems supported, automatic migration
- **Phase 2 (v0.2.0)**: Legacy system deprecated warnings  
- **Phase 3 (v0.3.0)**: Legacy system removed, universal only

## Migration Checklist

- [ ] Update configuration files to use `primal_registry`
- [ ] Replace `is_beardog_enabled()` with `is_primal_enabled("beardog")`
- [ ] Replace `get_beardog_config()` with `get_primal_config("beardog")`
- [ ] Consider capability-based primal selection for flexibility
- [ ] Enable auto-discovery for dynamic primal detection
- [ ] Test with new primal types to verify extensibility

## Example: Complete Migration

See `examples/config/songbird-universal-primals.toml` for a complete example showing:
- Universal primal registry configuration
- Multiple primal types (BearDog, NestGate, Toadstool, Squirrel, Phoenix)  
- Capability-based configuration
- Auto-discovery settings
- Backward compatibility with legacy configs

This migration enables true universal extensibility while maintaining full backward compatibility with existing deployments. 