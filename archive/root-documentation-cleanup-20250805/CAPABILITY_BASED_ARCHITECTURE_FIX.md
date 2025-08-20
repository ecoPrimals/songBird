# 🎯 **CAPABILITY-BASED ARCHITECTURE FIX**

**PRINCIPLE**: Each primal only knows itself. Route by capability, not by name.

---

## 🚨 **CURRENT VIOLATIONS TO FIX**

### **1. Remove ALL Hardcoded Primal Names**
```rust
// ❌ DELETE THESE (Lines 70-74 in config/environment.rs)
beardog_endpoint: get_primal_endpoint("beardog"),
nestgate_endpoint: get_primal_endpoint("nestgate"), 
toadstool_endpoint: get_primal_endpoint("toadstool"),
squirrel_endpoint: get_primal_endpoint("squirrel"),

// ❌ DELETE THESE (Lines 660-679 in config/constants.rs)
pub const DEFAULT_BEARDOG_ENDPOINT: &str = "http://localhost:8004";
pub const DEFAULT_TOADSTOOL_ENDPOINT: &str = "http://localhost:8001";
pub const DEFAULT_NESTGATE_ENDPOINT: &str = "http://localhost:8003";
pub const DEFAULT_SQUIRREL_ENDPOINT: &str = "http://localhost:8002";
```

### **2. Remove Environment Variable Dependencies**
```rust
// ❌ DELETE THESE (in adapter_impl.rs)
if std::env::var("BEARDOG_ENDPOINT").is_ok() {
    providers.push("beardog".to_string());  // HARDCODED KNOWLEDGE
}
if std::env::var("TOADSTOOL_ENDPOINT").is_ok() {
    providers.push("toadstool".to_string()); // HARDCODED KNOWLEDGE
}
```

---

## ✅ **CORRECT ARCHITECTURE: SELF-ONLY + CAPABILITY ROUTING**

### **1. Songbird Self-Discovery Only**
```rust
// crates/songbird-config/src/self_discovery.rs (NEW FILE)
pub struct SongbirdSelfRegistration {
    pub service_id: Uuid,
    pub capabilities: Vec<ServiceCapability>,
    pub endpoints: Vec<ServiceEndpoint>,
}

impl SongbirdSelfRegistration {
    pub fn new() -> Self {
        Self {
            service_id: Uuid::new_v4(),
            capabilities: vec![
                ServiceCapability {
                    capability_type: "orchestration.networking".to_string(),
                    level: "advanced".to_string(),
                    constraints: vec!["service_mesh", "load_balancing"].into_iter().map(String::from).collect(),
                },
                ServiceCapability {
                    capability_type: "networking.discovery".to_string(),
                    level: "native".to_string(),
                    constraints: vec!["mdns", "dns_sd", "manual"].into_iter().map(String::from).collect(),
                },
                ServiceCapability {
                    capability_type: "orchestration.federation".to_string(),
                    level: "distributed".to_string(),
                    constraints: vec!["multi_node", "cluster_management"].into_iter().map(String::from).collect(),
                },
            ],
            endpoints: vec![
                ServiceEndpoint {
                    endpoint_type: "orchestration".to_string(),
                    url: std::env::var("SONGBIRD_ENDPOINT").unwrap_or("http://localhost:8080".to_string()),
                    health_check_path: Some("/health".to_string()),
                },
            ],
        }
    }
}
```

### **2. Pure Capability-Based Routing**
```rust
// crates/songbird-universal/src/capability_router.rs (REPLACE CURRENT)
pub struct CapabilityRouter {
    discovery_client: UniversalDiscoveryClient,
    capability_cache: Arc<RwLock<HashMap<String, Vec<ServiceProvider>>>>,
}

impl CapabilityRouter {
    /// Route request by capability (NO PRIMAL NAMES)
    pub async fn route_by_capability<T>(&self, 
        capability: &str, 
        request: T
    ) -> Result<ServiceResponse> {
        // 1. Discover services with this capability
        let providers = self.discovery_client
            .discover_by_capability(capability)
            .await?;
        
        if providers.is_empty() {
            return Err(RoutingError::NoCapableProviders {
                capability: capability.to_string(),
                suggestion: "Check if any services provide this capability".to_string(),
            });
        }
        
        // 2. Select best provider (QoS-based)
        let best_provider = self.select_optimal_provider(&providers).await?;
        
        // 3. Route request (NO KNOWLEDGE OF WHAT SERVICE THIS IS)
        self.send_capability_request(best_provider, request).await
    }
}
```

### **3. Examples of Capability-Based Usage**
```rust
// ❌ OLD WAY (hardcoded primal names)
let beardog_client = BearDogClient::new(&config.beardog_endpoint);
let result = beardog_client.encrypt(data).await?;

// ✅ NEW WAY (capability-based)
let result = capability_router
    .route_by_capability("security.encryption", EncryptRequest { data })
    .await?;

// ❌ OLD WAY (hardcoded primal names)  
let toadstool_client = ToadStoolClient::new(&config.toadstool_endpoint);
let metrics = toadstool_client.get_resource_usage().await?;

// ✅ NEW WAY (capability-based)
let metrics = capability_router
    .route_by_capability("compute.monitoring", ResourceUsageRequest {})
    .await?;

// ❌ OLD WAY (hardcoded primal names)
let nestgate_client = NestGateClient::new(&config.nestgate_endpoint);
let result = nestgate_client.store_data(data).await?;

// ✅ NEW WAY (capability-based)
let result = capability_router
    .route_by_capability("storage.persistence", StoreRequest { data })
    .await?;
```

---

## 🔄 **DISCOVERY CONFIGURATION (ENVIRONMENT-BASED)**

### **Replace Hardcoded Primal Configs With Discovery Config**
```toml
# OLD config (WRONG - hardcoded primal knowledge)
[primals]
beardog_endpoint = "http://localhost:8004"
toadstool_endpoint = "http://localhost:8001"
nestgate_endpoint = "http://localhost:8003"
squirrel_endpoint = "http://localhost:8002"

# NEW config (CORRECT - capability-based discovery)
[discovery]
# Songbird only needs to know how to discover services
discovery_methods = ["mdns", "dns_sd", "manual"]
discovery_timeout = "30s"
capability_cache_ttl = "5m"

# Manual service registrations (for services that don't self-advertise)
[[manual_services]]
service_id = "security-service-1"
capabilities = ["security.encryption", "security.authentication"]  
endpoint = "http://security-primal:8443"

[[manual_services]]
service_id = "compute-service-1"
capabilities = ["compute.monitoring", "compute.execution"]
endpoint = "http://compute-primal:8080"

[[manual_services]]
service_id = "storage-service-1"
capabilities = ["storage.persistence", "storage.backup"]
endpoint = "http://storage-primal:8080"

# Songbird self-registration
[self]
capabilities = ["orchestration.networking", "networking.discovery", "orchestration.federation"]
bind_address = "0.0.0.0:8080"
advertise_address = "${SONGBIRD_ENDPOINT:-http://localhost:8080}"
```

---

## 🎯 **IMPLEMENTATION STEPS**

### **Step 1: Create Self-Discovery Module**
```bash
# Create Songbird self-discovery (knows only itself)
touch crates/songbird-config/src/self_discovery.rs
```

### **Step 2: Remove All Hardcoded Primal References**
```bash
# Remove hardcoded primal knowledge
sed -i '/beardog_endpoint:/d' crates/songbird-config/src/config/environment.rs
sed -i '/nestgate_endpoint:/d' crates/songbird-config/src/config/environment.rs
sed -i '/toadstool_endpoint:/d' crates/songbird-config/src/config/environment.rs
sed -i '/squirrel_endpoint:/d' crates/songbird-config/src/config/environment.rs

# Remove hardcoded constants
sed -i '/DEFAULT_BEARDOG_ENDPOINT/d' crates/songbird-config/src/config/constants.rs
sed -i '/DEFAULT_TOADSTOOL_ENDPOINT/d' crates/songbird-config/src/config/constants.rs
sed -i '/DEFAULT_NESTGATE_ENDPOINT/d' crates/songbird-config/src/config/constants.rs
sed -i '/DEFAULT_SQUIRREL_ENDPOINT/d' crates/songbird-config/src/config/constants.rs
```

### **Step 3: Implement Pure Capability Router**
```bash
# Replace hardcoded adapter with capability-based router
mv crates/songbird-universal/src/adapter_impl.rs crates/songbird-universal/src/capability_router.rs
```

### **Step 4: Update All Usage Points**
```bash
# Find and replace all hardcoded primal usage
grep -r "beardog\|toadstool\|nestgate\|squirrel" crates/ --include="*.rs" | grep -v test
# Replace each with capability_router.route_by_capability() calls
```

---

## ✅ **RESULT: TRUE PRIMAL AUTONOMY**

After this fix:
- ✅ **Songbird knows only itself** (orchestration.networking capabilities)
- ✅ **No hardcoded primal names** anywhere in codebase
- ✅ **Pure capability-based routing** for all external services
- ✅ **Dynamic service discovery** based on what services can do, not what they're called
- ✅ **True ecosystem compliance** - any service can provide any capability
- ✅ **Future-proof architecture** - new primals integrate without code changes

**Each primal becomes truly autonomous and self-contained!** 