# 🍼 Zero Hardcoding Migration Guide

**MISSION**: Eliminate ALL vendor and numeric hardcoding from Songbird

## Philosophy: Infant Discovery

> *"Each service knows only itself and discovers others through the universal adapter, like an infant discovering the world."*

### Core Principles

1. **Zero Primal Names**: Never hardcode `beardog`, `toadstool`, `nestgate`, or `squirrel`
2. **Zero Vendor Names**: Never hardcode `kubernetes`, `consul`, `docker`, `redis`, etc.
3. **Zero Numeric Hardcoding**: Never hardcode ports, timeouts, or magic numbers
4. **Capability-Based**: Request `security`, `storage`, `compute`, `ai` - not specific providers
5. **Network Effects**: Complex workflows via universal adapter, not direct connections
6. **Dynamic Discovery**: Learn everything at runtime from environment and discovery

## Current State

**Audit Results** (as of Oct 28, 2025):
- **1,669 hardcoded patterns** found across codebase
- **816 primal name** references (669 critical)
- **300 vendor name** references (282 high severity)
- **553 port number** references

## Migration Strategy

### Phase 1: Configuration Layer ✅

Create zero-touch configuration system that requires NO hardcoded values.

**Key File**: `crates/songbird-config/src/zero_touch/infant_config.rs`

```rust
// ❌ OLD WAY - Hardcoded
let security_endpoint = "http://beardog:8443";

// ✅ NEW WAY - Discovered
let config = ZeroTouchConfig::from_environment()?;
// Reads from SERVICE_PORT env var, fails if not set
```

### Phase 2: Discovery Layer

Replace all hardcoded discovery with capability-based queries.

#### Before (Hardcoded)

```rust
// ❌ BAD: Hardcoded primal name
let beardog_client = BearDogClient::new("http://beardog:8443");
let encrypted = beardog_client.encrypt(data).await?;

// ❌ BAD: Hardcoded vendor
if kubernetes::is_available() {
    kubernetes::deploy(service);
}
```

#### After (Capability-Based)

```rust
// ✅ GOOD: Request capability, not specific provider
let discovery = InfantDiscoveryManager::new();
await discovery.begin_learning()?;

// Security capability (could be beardog, vault, kms, or anything)
let security = discovery.request_capability(
    "security",
    "encrypt",
    &data
).await?;

// Container orchestration capability (could be k8s, docker swarm, nomad, etc.)
let orchestrator = discovery.request_capability(
    "container_orchestration", 
    "deploy",
    &service_spec
).await?;
```

### Phase 3: Network Configuration

Eliminate all hardcoded ports and addresses.

#### Before (Hardcoded)

```rust
// ❌ BAD: Hardcoded ports
const HTTP_PORT: u16 = 8080;
const HEALTH_PORT: u16 = 8081;
const METRICS_PORT: u16 = 9090;

server.bind(format!("0.0.0.0:{}", HTTP_PORT))?;
```

#### After (Environment-Driven)

```rust
// ✅ GOOD: Ports from environment
let config = ZeroTouchConfig::from_environment()?;
// Reads SERVICE_PORT, HEALTH_PORT, METRICS_PORT from env
// FAILS at startup if required ports not configured

server.bind(format!("{}:{}", 
    config.network.bind_address,
    config.network.service_port
))?;
```

### Phase 4: Service Discovery

Replace vendor-specific discovery with universal patterns.

#### Before (Vendor-Specific)

```rust
// ❌ BAD: Hardcoded to Consul
use consul::ConsulClient;

let consul = ConsulClient::new("http://consul:8500");
let services = consul.query_services("api-service")?;
```

#### After (Universal)

```rust
// ✅ GOOD: Works with ANY service registry (Consul, Eureka, etcd, etc.)
let discovery = ZeroTouchConfig::from_environment()?;

// Discovery method determined at runtime from environment:
// - CONTAINER_METADATA_API -> Container orchestrator
// - SERVICE_REGISTRY_ENDPOINT -> HTTP registry (Consul, Eureka, etc.)
// - SERVICE_DISCOVERY_DOMAIN -> DNS SRV
// - ENABLE_NETWORK_DISCOVERY -> Network scanning

let services = discovery.discover_capability("api").await?;
```

## Migration Patterns

### Pattern 1: Primal Name Elimination

```rust
// ❌ BEFORE
use songbird_primal_sdk::beardog::BearDogClient;

let client = BearDogClient::connect("http://beardog:8443").await?;
let token = client.generate_token(user_id).await?;

// ✅ AFTER  
use songbird_universal::InfantDiscoveryManager;

let discovery = InfantDiscoveryManager::new();
let providers = discovery.request_capability(
    "security",           // Capability type
    "generate_token",     // Operation
    &json!({ "user_id": user_id })
).await?;
```

### Pattern 2: Vendor Name Elimination

```rust
// ❌ BEFORE
#[cfg(feature = "kubernetes")]
use k8s_openapi::api::apps::v1::Deployment;

let k8s_client = kubernetes::Client::try_default().await?;

// ✅ AFTER
use songbird_discovery::UniversalServiceDiscovery;

// Works with Kubernetes, Docker Swarm, Nomad, or any container orchestrator
let orchestrator = UniversalServiceDiscovery::from_environment().await?;
let deployment = orchestrator.request_capability(
    "container_orchestration",
    "deploy",
    &service_spec
).await?;
```

### Pattern 3: Port Number Elimination

```rust
// ❌ BEFORE
const DEFAULT_PORT: u16 = 8080;
const HEALTH_CHECK_PORT: u16 = 8081;

let addr = format!("0.0.0.0:{}", DEFAULT_PORT);

// ✅ AFTER
let config = ZeroTouchConfig::from_environment()?;
// SERVICE_PORT=8080 from environment

let addr = format!("{}:{}", 
    config.network.bind_address,
    config.network.service_port
);
```

### Pattern 4: Network Effects Without Direct Connections

The anti-pattern is creating 2^n hardcoded connections between primals:

```rust
// ❌ ANTI-PATTERN: Direct hardcoded connections (2^n complexity)
nestgate.connect_to_beardog();
nestgate.connect_to_toadstool();
beardog.connect_to_squirrel();
toadstool.connect_to_nestgate();
squirrel.connect_to_nestgate();
// ... nightmare of interconnections
```

The correct pattern uses the universal adapter for network effects:

```rust
// ✅ CORRECT: Network effects via universal adapter (O(n) complexity)
// Example: Toadstool needs storage from nestgate and security from beardog
// but doesn't know their names!

let workflow = discovery.execute_network_effect(
    "compute_with_secure_storage",  // Pattern name
    &json!({
        "compute_task": task_spec,
        "requires": ["storage", "security"]
    })
).await?;

// The universal adapter:
// 1. Discovers available security providers (could be beardog, vault, etc.)
// 2. Discovers available storage providers (could be nestgate, s3, etc.)
// 3. Orchestrates the workflow without hardcoded connections
```

## Environment Configuration

### Required Environment Variables

Every service MUST set:

```bash
# Service identity - THE ONLY THING A SERVICE KNOWS ABOUT ITSELF
SERVICE_ID=my-service-instance-1
SERVICE_CAPABILITIES=compute,data-processing  # What THIS service provides

# Network config - NO DEFAULTS
SERVICE_PORT=8080      # Required - no default
HEALTH_PORT=8081       # Optional - defaults to SERVICE_PORT
METRICS_PORT=8082      # Optional - defaults to SERVICE_PORT

# Required capabilities - WHAT you need, not WHO provides it
REQUIRED_CAPABILITIES=security,storage,logging
OPTIONAL_CAPABILITIES=ai,analytics
```

### Discovery Configuration

```bash
# Enable infant discovery (zero-knowledge bootstrap)
ENABLE_INFANT_DISCOVERY=true
DISCOVERY_TIMEOUT_SECS=30
DISCOVERY_REFRESH_SECS=60

# Discovery methods (tried in order)
# Option 1: Service Registry (works with Consul, Eureka, Zookeeper, etc.)
SERVICE_REGISTRY_ENDPOINT=http://localhost:8500

# Option 2: Container Metadata (works with K8s, Docker, Nomad, etc.)
CONTAINER_METADATA_API=http://localhost:10250

# Option 3: DNS Discovery
SERVICE_DISCOVERY_DOMAIN=services.local

# Option 4: Network Scanning (development only)
ENABLE_NETWORK_DISCOVERY=true
DISCOVERY_IP_RANGES=127.0.0.0/8,192.168.0.0/16
DISCOVERY_PORTS=8080,8081,8082,8443
```

### Capability-Specific Configuration

Instead of hardcoded endpoints, optionally provide hints:

```bash
# If you KNOW where security capability is (optional)
CAPABILITY_SECURITY_ENDPOINT=http://security-service:8443
CAPABILITY_SECURITY_SECURITY_LEVEL=encrypted

# If you KNOW where storage capability is (optional)
CAPABILITY_STORAGE_ENDPOINT=http://storage-service:9000
CAPABILITY_STORAGE_MIN_AVAILABILITY=0.99

# But these are OPTIONAL - discovery will find them if not provided
```

## Testing Guidelines

### Test Code vs Production Code

**Test code** CAN have some hardcoding for mocking:

```rust
#[cfg(test)]
mod tests {
    // ✅ OK in tests: Mock specific providers
    use songbird_test_utils::mocks::beardog::MockBearDog;
    
    #[test]
    fn test_security_integration() {
        let mock_security = MockBearDog::new();
        // Testing specific provider behavior
    }
}
```

**Production code** MUST use discovery:

```rust
// ✅ Production: Always use capability discovery
pub async fn secure_operation(discovery: &InfantDiscoveryManager) -> Result<()> {
    let providers = discovery.request_capability("security", "encrypt", &data).await?;
    // Works with ANY security provider
}
```

## Migration Checklist

For each file being migrated:

- [ ] Remove all primal names (beardog, toadstool, nestgate, squirrel)
- [ ] Remove all vendor names (kubernetes, consul, docker, redis)
- [ ] Remove all hardcoded ports (use environment variables)
- [ ] Replace with capability-based discovery
- [ ] Add environment variable requirements to docs
- [ ] Update tests to use mocks where appropriate
- [ ] Verify no hardcoded service URLs

## Tools

### Automated Scanning

```bash
# Scan for all hardcoding
python3 scripts/eliminate_all_hardcoding.py

# Generate environment template
# Output: config/zero-knowledge.env.template

# Generate detailed report
# Output: HARDCODING_ELIMINATION_REPORT.md
```

### Manual Verification

```bash
# Check production code for primal names
grep -r "beardog\|toadstool\|nestgate\|squirrel" crates/*/src/ \
  --include="*.rs" \
  ! -path "*/tests/*" \
  ! -name "*test*.rs"

# Should return 0 results for production code
```

## Benefits

### Before: Hardcoded Hell

```rust
// Nightmare: Everyone knows everyone, 2^n connections
if is_kubernetes() {
    deploy_to_k8s(&beardog_endpoint, &toadstool_endpoint);
} else if is_docker() {
    deploy_to_docker(&beardog_endpoint, &toadstool_endpoint);  
}
// Breaks when: Adding new primal, changing vendor, different environment
```

### After: Zero-Knowledge Zen

```rust
// Zen: Start knowing nothing, learn everything
let discovery = InfantDiscoveryManager::new();
discovery.begin_learning().await?;

let deployment = discovery.execute_network_effect(
    "deploy_with_security",
    &service_spec
).await?;
// Works with: Any vendor, any primal, any environment
```

## Success Metrics

- [ ] Zero hardcoded primal names in production code
- [ ] Zero hardcoded vendor names in production code  
- [ ] Zero hardcoded ports in production code
- [ ] All services can start with only SERVICE_ID and SERVICE_PORT
- [ ] Services discover all dependencies at runtime
- [ ] No 2^n interconnections between services
- [ ] 100% environment-driven configuration

## FAQs

### Q: What about adapter implementations?

**A:** Adapter *implementations* (like `KubernetesAdapter`) can reference their vendor name because they're specifically implementing support for that vendor. But *production code using adapters* should only reference capabilities:

```rust
// ✅ OK: Adapter implementation
pub struct KubernetesAdapter { /* ... */ }

// ❌ BAD: Production code hardcoding vendor
let k8s = KubernetesAdapter::new();

// ✅ GOOD: Production code using discovery
let orchestrator = discovery.request_capability("container_orchestration").await?;
```

### Q: What about constants files?

**A:** Even constants files should avoid hardcoding. Use:

```rust
// ❌ BAD: Hardcoded default
pub const DEFAULT_PORT: u16 = 8080;

// ✅ GOOD: No default, must come from environment
pub fn get_service_port() -> Result<u16> {
    env::var("SERVICE_PORT")?
        .parse()
        .map_err(|_| Error::Configuration("Invalid SERVICE_PORT"))
}
```

### Q: What about development/local testing?

**A:** Use environment files:

```bash
# dev.env
SERVICE_ID=dev-service
SERVICE_PORT=8080
REQUIRED_CAPABILITIES=
ENABLE_INFANT_DISCOVERY=false  # Optional: skip discovery for speed

# Load and run
export $(cat dev.env | xargs) && cargo run
```

## Next Steps

1. ✅ Audit complete (1,669 patterns found)
2. 🚧 Migrate critical production files (config, discovery, SDK)
3. ⏳ Update test utilities to use capability mocks
4. ⏳ Generate environment templates for all services
5. ⏳ Validate zero hardcoding in production code

---

**Remember**: Each service is an infant 🍼 that knows only itself and learns about the world through discovery.

