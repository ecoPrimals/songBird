# 📝 Configuration Guide - Songbird

**Last Updated**: December 15, 2025  
**Status**: ✅ Capability-Based Discovery & Environment Configuration

---

## 🎯 Overview

Songbird uses **capability-based discovery** with multiple discovery methods for maximum flexibility. All hardcoded endpoints have been eliminated in favor of dynamic runtime discovery.

### Discovery Methods (Priority Order)

1. **Environment Variables** - Highest priority, perfect for containers/K8s
2. **DNS-SD (Service Discovery)** - Standard DNS SRV records
3. **Service Registry** - Centralized capability registry
4. **Config Files** - Static configuration for development

**See**: [Capability Discovery Technical Summary](audits/dec-15-2025/CAPABILITY_DISCOVERY_TECHNICAL_SUMMARY.md) for complete details.

---

## ⚙️ Environment Variables

### **Capability Discovery (NEW)** ⭐

```bash
# Primal Service Discovery (replaces hardcoded endpoints)
COMPUTE_ENDPOINT=http://toadstool:8001   # Compute provider (e.g., ToadStool)
STORAGE_ENDPOINT=http://squirrel:8002    # Storage provider (e.g., Squirrel)
SECURITY_ENDPOINT=http://nestgate:8003   # Security provider (e.g., NestGate)
AI_ENDPOINT=http://beardog:8004          # AI provider (e.g., BearDog)

# Discovery Configuration
REGISTRY_ENDPOINT=http://registry:8000   # Service registry endpoint
DISCOVERY_TIMEOUT=5000                   # Discovery timeout in ms (default: 5000)
DISCOVERY_CACHE_TTL=60                   # Cache TTL in seconds (default: 60)

# DNS-SD Configuration (automatic if DNS records exist)
# No configuration needed - Songbird automatically queries DNS SRV records:
#   _compute._tcp.local.  IN SRV 0 5 8001 toadstool.local.
#   _storage._tcp.local.  IN SRV 0 5 8002 squirrel.local.
```

### **Core Service Ports**

```bash
# API Configuration
SONGBIRD_API_PORT=8080              # API server port (default: 8080)
SONGBIRD_API_HOST=0.0.0.0           # API bind address (default: 127.0.0.1)

# Service Endpoints
SONGBIRD_ORCHESTRATOR_PORT=8081     # Orchestrator port (default: 8081)
SONGBIRD_DISCOVERY_PORT=8082        # Discovery service port (default: 8082)
SONGBIRD_HEALTH_PORT=8083           # Health check port (default: 8083)
SONGBIRD_DASHBOARD_PORT=8084        # Dashboard port (default: 8084)
SONGBIRD_WEBSOCKET_PORT=8085        # WebSocket port (default: 8085)
SONGBIRD_FEDERATION_PORT=8086       # Federation port (default: 8086)
```

### **Network Configuration**

```bash
# Bind Addresses
SONGBIRD_BIND_ADDRESS=0.0.0.0       # Main bind address (default: 0.0.0.0 for prod, 127.0.0.1 for dev)

# Connection Limits
SONGBIRD_MAX_CONNECTIONS=1000       # Max concurrent connections (default: 1000)
SONGBIRD_CONNECTION_TIMEOUT=30      # Connection timeout in seconds (default: 30)
```

### **Environment-Specific Settings**

```bash
# Environment
SONGBIRD_ENV=development            # Environment: development, staging, production (default: development)
RUST_LOG=info                       # Log level: trace, debug, info, warn, error (default: info)

# Features
SONGBIRD_FEDERATION_ENABLED=false   # Enable federation (default: false in dev, true in prod)
SONGBIRD_METRICS_ENABLED=true       # Enable metrics (default: true)
SONGBIRD_OBSERVABILITY_ENABLED=true # Enable observability (default: true)
```

---

## 🔍 Capability Discovery Deployment Patterns

### **Pattern 1: Environment Variables (Recommended for Docker/K8s)**

```bash
# Docker Compose
services:
  songbird:
    environment:
      - COMPUTE_ENDPOINT=http://toadstool:8001
      - STORAGE_ENDPOINT=http://squirrel:8002
      - SECURITY_ENDPOINT=http://nestgate:8003
      - AI_ENDPOINT=http://beardog:8004

# Kubernetes ConfigMap
apiVersion: v1
kind: ConfigMap
metadata:
  name: songbird-endpoints
data:
  COMPUTE_ENDPOINT: "http://toadstool-service:8001"
  STORAGE_ENDPOINT: "http://squirrel-service:8002"
```

**Benefits**:
- ✅ No configuration files needed
- ✅ Standard cloud-native practice
- ✅ Easy to override per environment

---

### **Pattern 2: DNS-SD (Recommended for Enterprise)**

```bash
# Add DNS SRV records to your DNS server
_compute._tcp.local.  IN SRV 0 5 8001 toadstool.local.
_storage._tcp.local.  IN SRV 0 5 8002 squirrel.local.
_security._tcp.local. IN SRV 0 5 8003 nestgate.local.
_ai._tcp.local.       IN SRV 0 5 8004 beardog.local.

# No environment variables needed!
# Songbird automatically discovers services via DNS
```

**Benefits**:
- ✅ Leverages existing DNS infrastructure
- ✅ Standard protocol (RFC 6763)
- ✅ Zero configuration in Songbird
- ✅ Enterprise-ready

---

### **Pattern 3: Service Registry (Recommended for Dynamic Environments)**

```bash
# Start the registry service
REGISTRY_ENDPOINT=http://registry:8000

# Providers register themselves with the registry
# Songbird queries registry for capabilities
```

**Benefits**:
- ✅ Centralized service catalog
- ✅ Dynamic updates
- ✅ Health tracking
- ✅ Multiple providers per capability

---

### **Pattern 4: Config Files (Recommended for Development)**

```toml
# config/endpoints.toml
[endpoints]
compute = "http://localhost:8001"
storage = "http://localhost:8002"
security = "http://localhost:8003"
ai = "http://localhost:8004"
```

**Benefits**:
- ✅ Explicit configuration
- ✅ Version-controlled
- ✅ Easy to understand

---

## 📁 Configuration Files

### **Development Configuration**

**File**: `config/development.env`

```bash
# Development Environment Configuration
SONGBIRD_ENV=development
SONGBIRD_BIND_ADDRESS=127.0.0.1
SONGBIRD_API_PORT=8080
SONGBIRD_ORCHESTRATOR_PORT=8081
SONGBIRD_DISCOVERY_PORT=8082
SONGBIRD_HEALTH_PORT=8083

# Development features
SONGBIRD_FEDERATION_ENABLED=false
RUST_LOG=debug
```

**Usage**:
```bash
# Load development config
source config/development.env
cargo run
```

### **Production Configuration**

**File**: `config/production.env`

```bash
# Production Environment Configuration
SONGBIRD_ENV=production
SONGBIRD_BIND_ADDRESS=0.0.0.0
SONGBIRD_API_PORT=8080
SONGBIRD_ORCHESTRATOR_PORT=8081
SONGBIRD_DISCOVERY_PORT=8082
SONGBIRD_HEALTH_PORT=8083

# Production features
SONGBIRD_FEDERATION_ENABLED=true
SONGBIRD_METRICS_ENABLED=true
RUST_LOG=info
```

**Usage**:
```bash
# Load production config
source config/production.env
cargo run --release
```

### **Staging Configuration**

**File**: `config/staging.env`

```bash
# Staging Environment Configuration
SONGBIRD_ENV=staging
SONGBIRD_BIND_ADDRESS=0.0.0.0
SONGBIRD_API_PORT=8080

# Staging features (mix of dev and prod)
SONGBIRD_FEDERATION_ENABLED=true
RUST_LOG=debug
```

---

## 🚀 Quick Start Examples

### **Local Development**

```bash
# Minimal local setup
export SONGBIRD_ENV=development
export RUST_LOG=debug
cargo run

# Or load full dev config
source config/development.env
cargo run
```

### **Production Deployment**

```bash
# Load production configuration
source config/production.env

# Build and run
cargo build --workspace --release
./target/release/songbird-orchestrator
```

### **Custom Configuration**

```bash
# Override specific settings
export SONGBIRD_API_PORT=9090
export SONGBIRD_API_HOST=0.0.0.0
export SONGBIRD_AI_ENDPOINT=http://ai-service:8002

cargo run --release
```

---

## 📊 Configuration by Component

### **Orchestrator**

```bash
SONGBIRD_ORCHESTRATOR_PORT=8081
SONGBIRD_MAX_WORKERS=10
SONGBIRD_QUEUE_SIZE=1000
```

### **Discovery Service**

```bash
SONGBIRD_DISCOVERY_PORT=8082
SONGBIRD_DISCOVERY_TIMEOUT=5000
SONGBIRD_MDNS_ENABLED=true
SONGBIRD_DNS_SD_ENABLED=true
```

### **Health Monitoring**

```bash
SONGBIRD_HEALTH_PORT=8083
SONGBIRD_HEALTH_CHECK_INTERVAL=30
SONGBIRD_HEALTH_TIMEOUT=10
```

### **Federation**

```bash
SONGBIRD_FEDERATION_PORT=8086
SONGBIRD_FEDERATION_ENABLED=true
SONGBIRD_FEDERATION_NODES="node1:8086,node2:8086"
```

---

## 🛠️ Configuration in Code

### **Reading Configuration**

```rust
use std::env;

// Simple read with default
let port = env::var("SONGBIRD_API_PORT")
    .ok()
    .and_then(|p| p.parse().ok())
    .unwrap_or(8080);

// Read string with default
let host = env::var("SONGBIRD_API_HOST")
    .unwrap_or_else(|_| "127.0.0.1".to_string());

// Boolean configuration
let enabled = env::var("SONGBIRD_FEDERATION_ENABLED")
    .ok()
    .and_then(|v| v.parse().ok())
    .unwrap_or(false);
```

### **Using in Default Implementations**

```rust
impl Default for ApiConfig {
    fn default() -> Self {
        let port = std::env::var("SONGBIRD_API_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);
        
        let host = std::env::var("SONGBIRD_API_HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_string());
        
        Self {
            port,
            host,
            enable_cors: true,
        }
    }
}
```

---

## 🔍 Configuration Discovery

### **Checking Current Configuration**

```bash
# View all Songbird environment variables
env | grep SONGBIRD

# Check specific setting
echo $SONGBIRD_API_PORT

# Verify configuration before running
source config/development.env && env | grep SONGBIRD
```

### **Configuration Validation**

```bash
# Test configuration
cargo run -- --check-config

# Validate all settings
cargo test --package songbird-config
```

---

## 📋 Default Values Reference

| Variable | Default | Environment |
|----------|---------|-------------|
| `SONGBIRD_ENV` | `development` | All |
| `SONGBIRD_API_PORT` | `8080` | All |
| `SONGBIRD_API_HOST` | `127.0.0.1` (dev), `0.0.0.0` (prod) | All |
| `SONGBIRD_ORCHESTRATOR_PORT` | `8081` | All |
| `SONGBIRD_DISCOVERY_PORT` | `8082` | All |
| `SONGBIRD_HEALTH_PORT` | `8083` | All |
| `SONGBIRD_DASHBOARD_PORT` | `8084` | All |
| `SONGBIRD_WEBSOCKET_PORT` | `8085` | All |
| `SONGBIRD_FEDERATION_PORT` | `8086` | All |
| `SONGBIRD_AI_ENDPOINT` | `http://localhost:8002` | All |
| `SONGBIRD_MAX_CONNECTIONS` | `1000` | All |
| `SONGBIRD_CONNECTION_TIMEOUT` | `30` | All |
| `SONGBIRD_FEDERATION_ENABLED` | `false` (dev), `true` (prod) | All |
| `RUST_LOG` | `info` (prod), `debug` (dev) | All |

---

## 🎯 Best Practices

### **Development**

1. **Use Development Config**:
   ```bash
   source config/development.env
   ```

2. **Enable Debug Logging**:
   ```bash
   export RUST_LOG=debug
   ```

3. **Local Bind Address**:
   ```bash
   export SONGBIRD_BIND_ADDRESS=127.0.0.1
   ```

### **Production**

1. **Use Production Config**:
   ```bash
   source config/production.env
   ```

2. **Appropriate Logging**:
   ```bash
   export RUST_LOG=info
   ```

3. **Bind to All Interfaces**:
   ```bash
   export SONGBIRD_BIND_ADDRESS=0.0.0.0
   ```

4. **Enable Federation**:
   ```bash
   export SONGBIRD_FEDERATION_ENABLED=true
   ```

### **Security**

1. **Never commit `.env` files** with secrets
2. **Use environment-specific configs**
3. **Rotate credentials regularly**
4. **Use secure defaults**

---

## 🚨 Troubleshooting

### **Port Already in Use**

```bash
# Change port
export SONGBIRD_API_PORT=9090
cargo run
```

### **Service Not Accessible**

```bash
# Check bind address
export SONGBIRD_API_HOST=0.0.0.0
cargo run
```

### **Configuration Not Loading**

```bash
# Verify environment
env | grep SONGBIRD

# Re-source configuration
source config/development.env
```

---

## 📚 Related Documentation

- **[README.md](../README.md)** - Project overview
- **[DEPLOY.md](../DEPLOY.md)** - Deployment guide
- **[HARDCODING_ELIMINATION_STRATEGY.md](../HARDCODING_ELIMINATION_STRATEGY.md)** - Hardcoding strategy

---

## ✅ Configuration Checklist

### **Before Deployment**

- [ ] Load appropriate environment config
- [ ] Verify all required variables set
- [ ] Check port availability
- [ ] Validate bind addresses
- [ ] Test configuration with `cargo run`
- [ ] Verify logs show correct settings

### **Production Deployment**

- [ ] Use `production.env`
- [ ] Set `SONGBIRD_ENV=production`
- [ ] Enable federation if needed
- [ ] Configure monitoring
- [ ] Set appropriate log level
- [ ] Verify all services accessible

---

*Last Updated: November 22, 2025*  
*Status: All hardcoded values replaced* ✅  
*Configuration: Environment-based* ✅

