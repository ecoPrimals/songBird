# 🍼 Infant Discovery Examples

This directory contains examples demonstrating the **zero-hardcoding** philosophy where services start with zero knowledge and discover everything dynamically.

---

## 🎯 Philosophy

> **"Each primal only knows itself and discovers others through the universal adapter"**

### What This Means

- ❌ **No hardcoded primal names** (beardog, squirrel, toadstool, nestgate)
- ❌ **No hardcoded ports** (8001, 8002, 8003, 8004)
- ❌ **No hardcoded endpoints**
- ✅ **Pure capability-based discovery**
- ✅ **Zero knowledge bootstrap**
- ✅ **Dynamic network effect formation**

---

## 📚 Available Examples

### 1. **`infant_discovery_demo.rs`** - Core Discovery Demo
Demonstrates the complete infant discovery process.

**What it shows:**
- Starting with zero knowledge
- 6-phase discovery process
- Capability-based discovery
- Network effects emergence

**Run it:**
```bash
# Option 1: Explicit configuration
export SERVICE_PORT=8080
export CAPABILITY_SECURITY_ENDPOINT="http://localhost:8443"
export CAPABILITY_STORAGE_ENDPOINT="http://localhost:9000"
cargo run --example infant_discovery_demo

# Option 2: Service registry
export SERVICE_PORT=8080
export ENABLE_INFANT_DISCOVERY=true
export SERVICE_REGISTRY_ENDPOINT="http://localhost:8500"
cargo run --example infant_discovery_demo

# Option 3: Network scan (dev only)
export SERVICE_PORT=8080
export ENABLE_INFANT_DISCOVERY=true
export ENABLE_NETWORK_DISCOVERY=true
export DISCOVERY_IP_RANGES="127.0.0.1/24"
cargo run --example infant_discovery_demo
```

---

## 🚀 Quick Start

### Minimal Configuration
```bash
# Just set your service port (REQUIRED)
export SERVICE_PORT=8080

# Set what capabilities you need (not who provides them)
export REQUIRED_CAPABILITIES="security,storage"

# Run any example
cargo run --example infant_discovery_demo
```

### With Capability Discovery
```bash
# Configure specific capabilities
export SERVICE_PORT=8080
export CAPABILITY_SECURITY_ENDPOINT="http://security-provider:8443"
export CAPABILITY_AI_ENDPOINT="http://ai-provider:8002"

cargo run --example infant_discovery_demo
```

### With Service Registry
```bash
# Let infant discovery find everything
export SERVICE_PORT=8080
export ENABLE_INFANT_DISCOVERY=true
export SERVICE_REGISTRY_ENDPOINT="http://consul:8500"

cargo run --example infant_discovery_demo
```

---

## 🔄 Migration from Old Examples

### Before (Hardcoded)
```rust
// ❌ OLD: Hardcoded primal names and ports
let beardog_endpoint = "http://localhost:8004";
let squirrel_endpoint = "http://localhost:8002";
let toadstool_endpoint = "http://localhost:8001";
let nestgate_endpoint = "http://localhost:8003";
```

### After (Capability-Based)
```rust
// ✅ NEW: Capability-based discovery
use songbird_config::capability_endpoints;

let security = capability_endpoints::get_capability_endpoint("security").await?;
let ai = capability_endpoints::get_capability_endpoint("ai").await?;
let compute = capability_endpoints::get_capability_endpoint("compute").await?;
let storage = capability_endpoints::get_capability_endpoint("storage").await?;
```

---

## 📖 Capability Mapping

When migrating from hardcoded primal names to capabilities:

| Old Primal | New Capability | Description |
|-----------|---------------|-------------|
| `beardog` | `security` | Authentication, encryption, authorization |
| `squirrel` | `ai` | AI/ML inference, training, analysis |
| `toadstool` | `compute` | Workload execution, container orchestration |
| `nestgate` | `storage` | Data persistence, caching, backup |

---

## 🌐 Discovery Methods

Examples demonstrate multiple discovery methods:

### 1. Environment Variables (Highest Priority)
```bash
export CAPABILITY_SECURITY_ENDPOINT="http://security:8443"
```

### 2. Service Registry (Consul, Eureka, etc.)
```bash
export SERVICE_REGISTRY_ENDPOINT="http://consul:8500"
```

### 3. Container Metadata (Kubernetes, Docker)
```bash
export CONTAINER_METADATA_API="https://kubernetes.default.svc"
```

### 4. DNS SRV Records
```bash
export SERVICE_DISCOVERY_DOMAIN="services.example.com"
```

### 5. Network Scanning (Development Only)
```bash
export ENABLE_NETWORK_DISCOVERY=true
export DISCOVERY_IP_RANGES="127.0.0.1/24"
```

---

## 🧪 Testing Examples

### Test with Mock Services
```bash
# Terminal 1: Run mock security service
cargo run --example mock_security_service -- --port 8443

# Terminal 2: Run mock storage service
cargo run --example mock_storage_service -- --port 9000

# Terminal 3: Run infant discovery demo
export SERVICE_PORT=8080
export CAPABILITY_SECURITY_ENDPOINT="http://localhost:8443"
export CAPABILITY_STORAGE_ENDPOINT="http://localhost:9000"
cargo run --example infant_discovery_demo
```

### Test with Service Registry
```bash
# Start Consul (or use existing)
docker run -d -p 8500:8500 consul

# Register services with Consul
consul services register security-service.json
consul services register storage-service.json

# Run example with discovery
export SERVICE_PORT=8080
export ENABLE_INFANT_DISCOVERY=true
export SERVICE_REGISTRY_ENDPOINT="http://localhost:8500"
cargo run --example infant_discovery_demo
```

---

## 💡 Best Practices

### 1. **Always Use Capability Names**
```rust
// ✅ GOOD
get_capability_endpoint("security").await?

// ❌ BAD
get_primal_endpoint("beardog")  // Deprecated!
```

### 2. **Let Services Discover**
```rust
// ✅ GOOD: Dynamic discovery
let client = SecurityCapabilityClient::new().await?;

// ❌ BAD: Hardcoded connection
let client = BeardogClient::new("http://localhost:8004")?;
```

### 3. **Use Environment for Configuration**
```bash
# ✅ GOOD: Capability-based
export CAPABILITY_SECURITY_ENDPOINT="http://security:8443"

# ❌ BAD: Primal-specific
export BEARDOG_ENDPOINT="http://localhost:8004"
```

### 4. **Enable Infant Discovery**
```bash
# ✅ BEST: Zero configuration
export ENABLE_INFANT_DISCOVERY=true
export SERVICE_REGISTRY_ENDPOINT="http://consul:8500"
```

---

## 🎯 Learning Path

1. **Start with**: `infant_discovery_demo.rs`
   - Understand zero-knowledge bootstrap
   - See capability discovery in action

2. **Then explore**: Capability-specific examples
   - Security capability usage
   - Storage capability usage
   - AI capability usage
   - Compute capability usage

3. **Advanced**: Network effects examples
   - Multi-capability workflows
   - Service mesh formation
   - Dynamic routing

---

## 📞 Troubleshooting

### "SERVICE_PORT not set" Error
```bash
# Fix: Set required environment variable
export SERVICE_PORT=8080
```

### "No capability endpoint found" Error
```bash
# Fix: Either set explicit endpoint
export CAPABILITY_SECURITY_ENDPOINT="http://localhost:8443"

# Or enable discovery
export ENABLE_INFANT_DISCOVERY=true
export SERVICE_REGISTRY_ENDPOINT="http://localhost:8500"
```

### Discovery Not Finding Services
```bash
# Check discovery methods
export RUST_LOG=debug
cargo run --example infant_discovery_demo

# Try explicit configuration first
export CAPABILITY_SECURITY_ENDPOINT="http://localhost:8443"
```

---

## 🔗 Related Documentation

- `ZERO_HARDCODING_MIGRATION_PLAN.md` - Migration strategy
- `CAPABILITY_SHOWCASE_GUIDE.md` - Capability system guide
- `.env.example` - Configuration reference
- `docs/infant-discovery-guide.md` - Detailed discovery guide

---

## 🎉 Success Indicators

You'll know the examples work when you see:

```
✅ Zero-touch configuration created
✅ Found security at: http://localhost:8443
✅ Found storage at: http://localhost:9000
✅ All capabilities available!
🎯 Complex workflow possible through capability composition
🎓 Learning complete: 4 capabilities discovered
🎉 Infant Discovery Demo Complete!
```

---

**Remember**: Each service only knows itself! 🍼


