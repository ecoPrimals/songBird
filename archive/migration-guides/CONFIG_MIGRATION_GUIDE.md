# 🔧 Configuration Migration Guide

## Overview

We're migrating from **80+ fragmented configuration files** to a **single, unified configuration system** that eliminates hardcoding and provides consistent environment variable support.

## 🎯 Benefits

- **Single Source of Truth**: One configuration structure for the entire system
- **Zero Hardcoding**: All values configurable via environment variables
- **Hierarchical Loading**: Environment → File → Defaults
- **Type Safety**: Full Rust type checking for all configuration
- **Validation**: Built-in configuration validation
- **Multiple Formats**: Support for TOML, YAML, and JSON

## 🚀 Migration Process

### Phase 1: Import Unified Config

Replace scattered config imports with the unified system:

```rust
// ❌ OLD: Multiple config imports
use songbird_federation::config::FederationConfig;
use songbird_network::management::config::NetworkConfig;
use songbird_discovery::config::DiscoveryConfig;

// ✅ NEW: Single unified import
use songbird_config::UnifiedSongbirdConfig;
```

### Phase 2: Update Configuration Loading

Replace individual config loading with unified loading:

```rust
// ❌ OLD: Loading multiple configs separately
let federation_config = FederationConfig::load()?;
let network_config = NetworkConfig::load()?; 
let discovery_config = DiscoveryConfig::load()?;

// ✅ NEW: Single config load with validation
let config = UnifiedSongbirdConfig::load()?;
config.validate()?;

// Access all subsystems through unified structure
let federation = &config.federation;
let network = &config.network;
let discovery = &config.discovery;
```

### Phase 3: Environment Variable Migration

Standardize environment variables to use consistent `SONGBIRD_` prefix:

```bash
# ❌ OLD: Inconsistent environment variables
export FEDERATION_ENABLED=true
export NETWORK_PORT=8080
export DISCOVERY_BACKEND=consul
export BEARDOG_ENDPOINT=https://beardog:8443

# ✅ NEW: Consistent SONGBIRD_ prefix
export SONGBIRD_FEDERATION_ENABLED=true
export SONGBIRD_NETWORK_PORT=8080
export SONGBIRD_DISCOVERY_BACKEND=consul
export SONGBIRD_BEARDOG_ENDPOINT=https://beardog:8443
```

### Phase 4: Update Code Usage

Update code to use the unified configuration structure:

```rust
// ❌ OLD: Scattered config usage
impl FederationManager {
    pub fn new(config: FederationConfig) -> Self {
        Self {
            enabled: config.enabled,
            mode: config.mode,
            // ...
        }
    }
}

// ✅ NEW: Unified config usage
impl FederationManager {
    pub fn new(config: &UnifiedSongbirdConfig) -> Self {
        Self {
            enabled: config.federation.enabled,
            mode: config.federation.mode.clone(),
            heartbeat_interval: Duration::from_secs(config.federation.heartbeat_interval_secs),
            // ...
        }
    }
}
```

## 📋 Migration Checklist

### Files to Update

- [ ] `crates/songbird-federation/src/mod.rs` - Use unified federation config
- [ ] `crates/songbird-network/src/lib.rs` - Use unified network config  
- [ ] `crates/songbird-discovery/src/lib.rs` - Use unified discovery config
- [ ] `crates/songbird-security/src/lib.rs` - Use unified security config
- [ ] `crates/songbird-universal-primals/src/lib.rs` - Use unified primals config
- [ ] `crates/songbird-observability/src/lib.rs` - Use unified observability config

### Environment Variables to Standardize

```bash
# Federation
SONGBIRD_FEDERATION_ENABLED=false
SONGBIRD_FEDERATION_MODE=standalone
SONGBIRD_FEDERATION_PORT=8082
SONGBIRD_CLUSTER_NAME=songbird-cluster

# Network  
SONGBIRD_BIND_ADDRESS=127.0.0.1
SONGBIRD_PORT=8080
SONGBIRD_GAMING_ENABLED=true
SONGBIRD_GAMING_PORT_START=7000
SONGBIRD_GAMING_PORT_END=7100

# Discovery
SONGBIRD_DISCOVERY_BACKEND=static
SONGBIRD_CONSUL_URL=http://localhost:8500
SONGBIRD_K8S_NAMESPACE=default

# Security & Primals
SONGBIRD_BEARDOG_ENABLED=false
SONGBIRD_BEARDOG_ENDPOINT=https://beardog:8443
SONGBIRD_NESTGATE_ENDPOINT=http://nestgate:8080
SONGBIRD_TOADSTOOL_ENDPOINT=http://toadstool:8080
SONGBIRD_SQUIRREL_ENDPOINT=http://squirrel:8080

# Observability
SONGBIRD_LOG_LEVEL=info
SONGBIRD_METRICS_ENABLED=true
SONGBIRD_METRICS_PORT=9090
```

## 🔄 Backward Compatibility

During migration, we maintain backward compatibility:

1. **Existing configs** continue to work
2. **Gradual migration** - update one crate at a time
3. **Environment variable aliases** - old vars still work during transition
4. **Deprecation warnings** - clear guidance on what to update

## 🧪 Testing Migration

Test the migration thoroughly:

```rust
#[cfg(test)]
mod migration_tests {
    use super::*;
    
    #[test]
    fn test_unified_config_load() {
        let config = UnifiedSongbirdConfig::load().unwrap();
        config.validate().unwrap();
        
        // Test that all subsystems are properly configured
        assert!(!config.network.port == 0);
        assert!(!config.federation.max_nodes == 0);
    }
    
    #[test]
    fn test_environment_override() {
        std::env::set_var("SONGBIRD_NETWORK_PORT", "9999");
        let config = UnifiedSongbirdConfig::load().unwrap();
        assert_eq!(config.network.port, 9999);
        std::env::remove_var("SONGBIRD_NETWORK_PORT");
    }
}
```

## 🎉 Post-Migration Cleanup

After successful migration:

1. **Remove old config files** from individual crates
2. **Update documentation** to reference unified config
3. **Remove deprecated environment variables**
4. **Simplify configuration examples**
5. **Update deployment scripts** to use new environment variables

## 🌍 Environment-Specific Configurations

Create environment-specific config files:

- `config/development.toml` - Development settings
- `config/staging.toml` - Staging settings  
- `config/production.toml` - Production settings

Load with:
```bash
export SONGBIRD_CONFIG_FILE=config/production.toml
```

## 🔍 Validation & Debugging

The unified config provides built-in validation:

```rust
match config.validate() {
    Ok(()) => println!("✅ Configuration is valid"),
    Err(e) => {
        eprintln!("❌ Configuration error: {}", e);
        std::process::exit(1);
    }
}
```

Debug configuration loading:
```bash
export SONGBIRD_LOG_LEVEL=debug
# Shows exactly which values are loaded from where
```

This migration eliminates the root cause of hardcoding issues by providing a single, comprehensive configuration system with full environment variable support. 