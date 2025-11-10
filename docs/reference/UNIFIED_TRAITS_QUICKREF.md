# 🏗️ Unified Traits Quick Reference

**Last Updated**: November 7, 2025  
**Status**: Production-Ready ✅  
**Location**: `crates/songbird-types/src/traits/canonical.rs`

---

## 🎯 Quick Start (2 Minutes)

### Import Canonical Traits

```rust
use songbird_types::traits::canonical::{
    Provider,                 // Base trait for all providers
    ServiceProvider,          // Service-oriented operations
    PrimalProvider,           // Primal-specific functionality
    DiscoveryProvider,        // Service discovery
    CapabilityProvider,       // Capability-based systems
    SecurityProvider,         // Security & authentication
    OrchestrationProvider,    // Service orchestration
    ObservabilityProvider,    // Metrics & monitoring
};
```

### Basic Implementation

```rust
use async_trait::async_trait;
use songbird_types::traits::canonical::*;

pub struct MyProvider {
    id: String,
    name: String,
}

#[async_trait]
impl Provider for MyProvider {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { &self.name }
    fn version(&self) -> &str { "1.0.0" }
    fn provider_type(&self) -> ProviderType { ProviderType::Service }
    
    async fn initialize(&mut self, config: ProviderConfig) -> SongbirdResult<()> {
        // Your init logic
        Ok(())
    }
    
    async fn shutdown(&mut self) -> SongbirdResult<()> {
        // Cleanup logic
        Ok(())
    }
    
    async fn health_check(&self) -> SongbirdResult<HealthStatus> {
        Ok(HealthStatus::Healthy)
    }
    
    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            description: "My provider".to_string(),
            tags: vec![],
            documentation_url: None,
            support_contact: None,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        }
    }
    
    async fn capabilities(&self) -> SongbirdResult<Vec<Capability>> {
        Ok(vec![])
    }
}
```

---

## 📋 Trait Hierarchy

```
Provider (Base Trait)
├── ServiceProvider
├── PrimalProvider
├── DiscoveryProvider
├── CapabilityProvider
├── SecurityProvider
├── OrchestrationProvider
└── ObservabilityProvider
```

**Key Rule**: All specialized traits extend `Provider`

---

## 🔧 1. Provider (Base Trait)

**Purpose**: Foundation for all providers

### Required Methods

```rust
#[async_trait]
pub trait Provider: Send + Sync + 'static {
    // Identity
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn provider_type(&self) -> ProviderType;
    
    // Lifecycle
    async fn initialize(&mut self, config: ProviderConfig) -> SongbirdResult<()>;
    async fn shutdown(&mut self) -> SongbirdResult<()>;
    
    // Health & Capabilities
    async fn health_check(&self) -> SongbirdResult<HealthStatus>;
    fn metadata(&self) -> ProviderMetadata;
    async fn capabilities(&self) -> SongbirdResult<Vec<Capability>>;
}
```

### Provider Types

```rust
pub enum ProviderType {
    Service,              // Service provider
    Primal,               // Primal provider
    Discovery,            // Discovery provider
    Capability,           // Capability provider
    Security,             // Security provider
    Orchestration,        // Orchestration provider
    Observability,        // Observability provider
    Custom(String),       // Custom type
}
```

### Health Status

```rust
pub enum HealthStatus {
    Healthy,      // All good
    Degraded,     // Partially working
    Unhealthy,    // Not working
    Unknown,      // Can't determine
}
```

---

## 🌐 2. ServiceProvider

**Purpose**: Service-oriented operations

### When to Use
- HTTP/gRPC services
- Database connections
- Message queues
- Any service with endpoints

### Required Methods

```rust
#[async_trait]
pub trait ServiceProvider: Provider {
    fn service_type(&self) -> ServiceType;
    
    async fn handle_request(
        &self, 
        request: ServiceRequest
    ) -> SongbirdResult<ServiceResponse>;
    
    async fn metrics(&self) -> SongbirdResult<ServiceMetrics>;
    
    async fn register_service(&self, info: ServiceInfo) -> SongbirdResult<()>;
    async fn unregister_service(&self, service_id: &str) -> SongbirdResult<()>;
    async fn update_health(&self, health: HealthStatus) -> SongbirdResult<()>;
}
```

### Service Types

```rust
pub enum ServiceType {
    WebService,
    Database,
    MessageQueue,
    Cache,
    FileStorage,
    Authentication,
    Custom(String),
}
```

### Example Implementation

```rust
pub struct HttpService {
    id: String,
    endpoint: String,
}

#[async_trait]
impl ServiceProvider for HttpService {
    fn service_type(&self) -> ServiceType {
        ServiceType::WebService
    }
    
    async fn handle_request(
        &self,
        request: ServiceRequest,
    ) -> SongbirdResult<ServiceResponse> {
        // Handle HTTP request
        let response = reqwest::Client::new()
            .request(
                reqwest::Method::from_bytes(request.method.as_bytes()).unwrap(),
                &self.endpoint,
            )
            .json(&request.body)
            .send()
            .await
            .map_err(|e| SongbirdError::network(e.to_string()))?;
            
        Ok(ServiceResponse {
            id: request.id,
            status_code: response.status().as_u16(),
            headers: HashMap::new(),
            body: response.json().await?,
            timestamp: SystemTime::now(),
        })
    }
    
    async fn metrics(&self) -> SongbirdResult<ServiceMetrics> {
        // Return service metrics
        Ok(ServiceMetrics {
            request_count: 1000,
            error_count: 5,
            average_response_time_ms: 45.2,
            uptime_seconds: 86400,
            memory_usage_mb: 128.5,
            cpu_usage_percent: 12.3,
        })
    }
    
    // ... other methods
}
```

---

## 🎯 3. PrimalProvider

**Purpose**: Primal-specific operations (external service integration)

### When to Use
- BearDog integration
- Toadstool integration
- Squirrel integration
- Any primal service

### Required Methods

```rust
#[async_trait]
pub trait PrimalProvider: Provider {
    fn primal_type(&self) -> PrimalType;
    
    async fn execute_capability(
        &self,
        capability: &str,
        context: PrimalContext,
        params: HashMap<String, serde_json::Value>,
    ) -> SongbirdResult<PrimalResponse>;
    
    async fn dependencies(&self) -> SongbirdResult<Vec<PrimalDependency>>;
    
    async fn can_integrate_with(
        &self,
        other_type: &str,
        other_capabilities: &[String],
    ) -> SongbirdResult<bool>;
    
    async fn integrate_with<P: PrimalProvider>(
        &mut self,
        other_primal: Arc<P>,
    ) -> SongbirdResult<IntegrationResult>;
    
    fn config_schema(&self) -> serde_json::Value;
    async fn apply_config(&mut self, config: serde_json::Value) -> SongbirdResult<()>;
}
```

### Primal Types

```rust
pub enum PrimalType {
    Security,         // Security/HSM primals
    Storage,          // Storage primals
    Compute,          // Compute primals
    AI,               // AI/ML primals
    Network,          // Network primals
    Custom(String),   // Custom primals
}
```

### Example Implementation

```rust
pub struct SecurityPrimal {
    id: String,
    hsm_endpoint: String,
}

#[async_trait]
impl PrimalProvider for SecurityPrimal {
    fn primal_type(&self) -> PrimalType {
        PrimalType::Security
    }
    
    async fn execute_capability(
        &self,
        capability: &str,
        context: PrimalContext,
        params: HashMap<String, serde_json::Value>,
    ) -> SongbirdResult<PrimalResponse> {
        match capability {
            "encrypt" => {
                // Encryption logic
                let data = params.get("data")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| SongbirdError::validation("Missing 'data' parameter"))?;
                    
                // Call HSM for encryption
                let encrypted = self.encrypt_data(data).await?;
                
                Ok(PrimalResponse {
                    success: true,
                    data: serde_json::json!({ "encrypted": encrypted }),
                    metadata: HashMap::new(),
                    execution_time_ms: 50,
                })
            }
            _ => Err(SongbirdError::validation(format!("Unknown capability: {}", capability)))
        }
    }
    
    async fn dependencies(&self) -> SongbirdResult<Vec<PrimalDependency>> {
        Ok(vec![
            PrimalDependency {
                service_name: "hsm-service".to_string(),
                required_version: "2.0+".to_string(),
                optional: false,
                capabilities: vec!["encrypt".to_string(), "decrypt".to_string()],
            }
        ])
    }
    
    // ... other methods
}
```

---

## 🔍 4. DiscoveryProvider

**Purpose**: Service discovery operations

### When to Use
- Kubernetes discovery
- Consul discovery
- DNS-based discovery
- Custom service registries

### Required Methods

```rust
#[async_trait]
pub trait DiscoveryProvider: Provider {
    async fn discover_services(
        &self,
        criteria: DiscoveryCriteria,
    ) -> SongbirdResult<Vec<ServiceInfo>>;
    
    async fn discover_primals(
        &self, 
        capability: &str
    ) -> SongbirdResult<Vec<PrimalInfo>>;
    
    async fn register(&self, service: ServiceInfo) -> SongbirdResult<()>;
    async fn unregister(&self, service_id: &str) -> SongbirdResult<()>;
    
    async fn watch_services(
        &self,
        query: DiscoveryQuery,
    ) -> SongbirdResult<Pin<Box<dyn Stream<Item = ServiceEvent> + Send>>>;
    
    async fn update_metadata(
        &self,
        service_id: &str,
        metadata: HashMap<String, String>,
    ) -> SongbirdResult<()>;
    
    async fn is_registered(&self, service_id: &str) -> SongbirdResult<bool>;
    async fn list_all(&self) -> SongbirdResult<Vec<ServiceInfo>>;
    
    fn backend_type(&self) -> &'static str;
}
```

### Example Implementation

```rust
pub struct KubernetesDiscovery {
    client: kube::Client,
}

#[async_trait]
impl DiscoveryProvider for KubernetesDiscovery {
    async fn discover_services(
        &self,
        criteria: DiscoveryCriteria,
    ) -> SongbirdResult<Vec<ServiceInfo>> {
        // Query Kubernetes API for services
        let services = self.client
            .list_services(criteria.tags.as_slice())
            .await
            .map_err(|e| SongbirdError::discovery(e.to_string()))?;
            
        Ok(services.into_iter().map(|s| ServiceInfo {
            id: s.id,
            name: s.name,
            service_type: ServiceType::WebService,
            version: s.version,
            endpoints: s.endpoints,
            health: HealthStatus::Healthy,
            metadata: s.metadata,
            tags: s.labels,
            capabilities: s.capabilities,
            last_updated: SystemTime::now(),
        }).collect())
    }
    
    fn backend_type(&self) -> &'static str {
        "kubernetes"
    }
    
    // ... other methods
}
```

---

## ⚡ 5. CapabilityProvider

**Purpose**: Capability-based operations

### When to Use
- Dynamic capability discovery
- Plugin systems
- Feature toggles

### Required Methods

```rust
#[async_trait]
pub trait CapabilityProvider: Provider {
    async fn get_capabilities(&self) -> SongbirdResult<Vec<Capability>>;
    
    async fn supports_capability(&self, capability: &str) -> SongbirdResult<bool>;
    
    async fn execute_capability(
        &self,
        capability: &str,
        params: HashMap<String, serde_json::Value>,
    ) -> SongbirdResult<serde_json::Value>;
    
    async fn capability_metadata(
        &self, 
        capability: &str
    ) -> SongbirdResult<CapabilityMetadata>;
}
```

---

## 🔒 6. SecurityProvider

**Purpose**: Security & authentication

### Required Methods

```rust
#[async_trait]
pub trait SecurityProvider: Provider {
    async fn authenticate(&self, credentials: Credentials) -> SongbirdResult<AuthToken>;
    
    async fn authorize(
        &self,
        token: &AuthToken,
        resource: &str,
        action: &str,
    ) -> SongbirdResult<bool>;
    
    async fn encrypt(&self, data: &[u8]) -> SongbirdResult<Vec<u8>>;
    async fn decrypt(&self, data: &[u8]) -> SongbirdResult<Vec<u8>>;
    
    async fn generate_token(&self, claims: TokenClaims) -> SongbirdResult<AuthToken>;
    async fn validate_token(&self, token: &AuthToken) -> SongbirdResult<TokenValidation>;
}
```

---

## 🎭 7. OrchestrationProvider

**Purpose**: Service orchestration

### Required Methods

```rust
#[async_trait]
pub trait OrchestrationProvider: Provider {
    async fn deploy(&self, deployment: DeploymentSpec) -> SongbirdResult<DeploymentResult>;
    async fn scale(&self, service_id: &str, replicas: u32) -> SongbirdResult<()>;
    async fn update(&self, service_id: &str, spec: DeploymentSpec) -> SongbirdResult<()>;
    async fn delete(&self, service_id: &str) -> SongbirdResult<()>;
    
    async fn get_status(&self, service_id: &str) -> SongbirdResult<DeploymentStatus>;
    async fn list_deployments(&self) -> SongbirdResult<Vec<DeploymentInfo>>;
    async fn get_logs(&self, service_id: &str, lines: Option<u32>) -> SongbirdResult<Vec<String>>;
}
```

---

## 📊 8. ObservabilityProvider

**Purpose**: Metrics & monitoring

### Required Methods

```rust
#[async_trait]
pub trait ObservabilityProvider: Provider {
    async fn record_metric(
        &self,
        name: &str,
        value: f64,
        labels: HashMap<String, String>,
    ) -> SongbirdResult<()>;
    
    async fn increment_counter(
        &self,
        name: &str,
        labels: HashMap<String, String>,
    ) -> SongbirdResult<()>;
    
    async fn record_histogram(
        &self,
        name: &str,
        value: f64,
        labels: HashMap<String, String>,
    ) -> SongbirdResult<()>;
    
    async fn start_span(
        &self,
        name: &str,
        parent: Option<SpanContext>,
    ) -> SongbirdResult<SpanContext>;
    
    async fn end_span(&self, span: SpanContext) -> SongbirdResult<()>;
    
    async fn query_metrics(&self, query: MetricQuery) -> SongbirdResult<Vec<MetricResult>>;
    async fn system_health(&self) -> SongbirdResult<SystemHealth>;
}
```

---

## 💡 Common Patterns

### Pattern 1: Trait Object Storage

```rust
// Store as trait objects for polymorphism
pub struct ProviderRegistry {
    providers: HashMap<String, Arc<dyn Provider>>,
}

impl ProviderRegistry {
    pub fn register(&mut self, provider: Arc<dyn Provider>) {
        self.providers.insert(provider.id().to_string(), provider);
    }
    
    pub fn get(&self, id: &str) -> Option<Arc<dyn Provider>> {
        self.providers.get(id).cloned()
    }
}
```

### Pattern 2: Downcasting to Specific Trait

```rust
pub async fn call_service(provider: Arc<dyn Provider>) -> SongbirdResult<()> {
    // Check if it's a ServiceProvider
    if let Some(service) = provider.downcast_ref::<dyn ServiceProvider>() {
        let request = ServiceRequest { /* ... */ };
        service.handle_request(request).await?;
    }
    Ok(())
}
```

### Pattern 3: Trait Composition

```rust
// A provider can implement multiple traits
pub struct UniversalProvider {
    /* ... */
}

#[async_trait]
impl Provider for UniversalProvider { /* ... */ }

#[async_trait]
impl ServiceProvider for UniversalProvider { /* ... */ }

#[async_trait]
impl DiscoveryProvider for UniversalProvider { /* ... */ }
```

---

## ⚠️ Important Notes

### Why `#[async_trait]`?

**Architectural Requirement**: Songbird's provider system uses trait objects (`Box<dyn Provider>`, `Arc<dyn Provider>`) for dynamic dispatch and plugin architecture. Native async traits cannot be used with trait objects in current Rust.

**Trade-off**: We accept the small performance overhead (~5-10%) for the architectural flexibility gained.

```rust
// This requires #[async_trait]
let providers: Vec<Arc<dyn Provider>> = vec![
    Arc::new(HttpService::new()),
    Arc::new(DatabaseService::new()),
    Arc::new(CacheService::new()),
];
```

### Dyn-Safety Requirements

- All traits are `Send + Sync + 'static'`
- No associated types or const generics
- Methods use trait objects compatible signatures

---

## ✅ Best Practices

### DO ✅

- Always implement `Provider` first
- Use trait objects for registries
- Provide rich metadata
- Handle errors gracefully
- Implement health checks properly

### DON'T ❌

- Don't panic in trait methods
- Don't block async methods
- Don't ignore initialization errors
- Don't skip cleanup in shutdown

---

## 📚 Related Documentation

- **Error System**: `UNIFIED_ERRORS_QUICKREF.md`
- **Result Types**: `UNIFIED_RESULTS_QUICKREF.md`
- **Full Implementation**: `crates/songbird-types/src/traits/canonical.rs`
- **Architecture**: `ARCHITECTURE_OVERVIEW.md`

---

**Need Help?** Check the trait source code or ask in #songbird-dev!

✅ **This trait system is production-ready and powers the entire Songbird ecosystem!**
