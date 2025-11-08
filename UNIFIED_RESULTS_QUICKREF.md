# 📊 Unified Result Types Quick Reference

**Last Updated**: November 7, 2025  
**Status**: Production-Ready ✅  
**Location**: `crates/songbird-types/src/results.rs`

---

## 🎯 Quick Start (1 Minute)

### Import Result Types

```rust
use songbird_types::{
    SongbirdResult,           // Universal result type
    ValidationResult,         // Validation operations
    DeploymentResult,         // Deployment operations
    HealthCheckResult,        // Health checks
    DiscoveryResult,          // Service discovery
    ConfigurationResult,      // Config operations
    ServiceOperationResult,   // Service operations
    NetworkOperationResult,   // Network operations
    SecurityOperationResult,  // Security operations
    FederationOperationResult,// Federation operations
};
```

### Basic Usage

```rust
// Standard function signature
pub async fn my_function() -> SongbirdResult<Data> {
    let data = fetch_data().await?;
    Ok(data)
}

// Domain-specific function
pub async fn validate_input(input: &str) -> ValidationResult<()> {
    if input.is_empty() {
        return Err(SongbirdError::validation("Input cannot be empty"));
    }
    Ok(())
}
```

---

## 📋 Canonical Result Types (11 Types)

### 🏆 1. **`SongbirdResult<T>`** - Universal Result Type

**Purpose**: Default result type for all operations

```rust
pub type SongbirdResult<T> = Result<T, SongbirdError>;
```

**When to Use**:
- ✅ Default choice for any function
- ✅ When no specific domain applies
- ✅ Generic error handling

**Example**:
```rust
pub async fn fetch_data(url: &str) -> SongbirdResult<Vec<u8>> {
    let response = reqwest::get(url)
        .await
        .map_err(|e| SongbirdError::network(e.to_string()))?;
        
    let bytes = response.bytes()
        .await
        .map_err(|e| SongbirdError::network(e.to_string()))?;
        
    Ok(bytes.to_vec())
}
```

---

### ✅ 2. **`ValidationResult<T>`** - Validation Operations

```rust
pub type ValidationResult<T = ()> = SongbirdResult<T>;
```

**When to Use**:
- Input validation
- Schema validation
- Data integrity checks

**Example**:
```rust
pub fn validate_email(email: &str) -> ValidationResult<()> {
    if !email.contains('@') {
        return Err(SongbirdError::validation("Invalid email format"));
    }
    if email.len() > 255 {
        return Err(SongbirdError::validation("Email too long"));
    }
    Ok(())
}

pub fn validate_config(config: &Config) -> ValidationResult<()> {
    if config.port == 0 {
        return Err(SongbirdError::validation("Port cannot be 0"));
    }
    if config.max_connections == 0 {
        return Err(SongbirdError::validation("Max connections must be > 0"));
    }
    Ok(())
}
```

---

### 🚀 3. **`DeploymentResult<T>`** - Deployment Operations

```rust
pub type DeploymentResult<T = ()> = SongbirdResult<T>;
```

**When to Use**:
- Service deployment
- Container orchestration
- Infrastructure provisioning

**Example**:
```rust
pub async fn deploy_service(spec: DeploymentSpec) -> DeploymentResult<DeploymentInfo> {
    // Validate spec
    validate_deployment_spec(&spec)?;
    
    // Deploy to orchestrator
    let deployment = orchestrator.deploy(spec)
        .await
        .map_err(|e| SongbirdError::service("orchestrator", e.to_string()))?;
    
    Ok(DeploymentInfo {
        id: deployment.id,
        status: DeploymentStatus::Running,
        replicas: deployment.replicas,
        ready_replicas: deployment.ready_replicas,
        created_at: SystemTime::now(),
        updated_at: SystemTime::now(),
    })
}
```

---

### 💚 4. **`HealthCheckResult`** - Health Check Operations

```rust
pub type HealthCheckResult = SongbirdResult<UnifiedHealthStatus>;
```

**When to Use**:
- Service health checks
- Component status checks
- System diagnostics

**Health Status Options**:
```rust
pub enum UnifiedHealthStatus {
    Healthy,      // ✅ All systems operational
    Degraded,     // ⚠️  Partially working
    Unhealthy,    // ❌ Not working
    Unknown,      // ❓ Can't determine
}
```

**Example**:
```rust
pub async fn check_database_health() -> HealthCheckResult {
    match database.ping().await {
        Ok(_) => Ok(UnifiedHealthStatus::Healthy),
        Err(e) if e.is_timeout() => Ok(UnifiedHealthStatus::Degraded),
        Err(_) => Ok(UnifiedHealthStatus::Unhealthy),
    }
}

pub async fn check_service_health(service_id: &str) -> HealthCheckResult {
    let endpoint = format!("http://{}/health", service_id);
    
    match reqwest::get(&endpoint).await {
        Ok(response) if response.status().is_success() => {
            Ok(UnifiedHealthStatus::Healthy)
        }
        Ok(_) => Ok(UnifiedHealthStatus::Degraded),
        Err(_) => Ok(UnifiedHealthStatus::Unhealthy),
    }
}
```

---

### 🔍 5. **`DiscoveryResult<T>`** - Service Discovery

```rust
pub type DiscoveryResult<T> = SongbirdResult<T>;
```

**When to Use**:
- Service discovery operations
- Primal discovery
- Endpoint lookups

**Example**:
```rust
pub async fn discover_services(
    criteria: DiscoveryCriteria
) -> DiscoveryResult<Vec<ServiceInfo>> {
    let backend = get_discovery_backend()?;
    
    let services = backend.discover_services(criteria)
        .await
        .map_err(|e| SongbirdError::discovery(e.to_string()))?;
    
    if services.is_empty() {
        return Err(SongbirdError::discovery("No services found"));
    }
    
    Ok(services)
}

pub async fn discover_primals(
    capability: &str
) -> DiscoveryResult<Vec<PrimalInfo>> {
    let discovery = UniversalDiscoveryFactory::auto_detect_and_create().await?;
    
    discovery.discover_primals(capability)
        .await
        .map_err(|e| SongbirdError::discovery(e.to_string()))
}
```

---

### ⚙️ 6. **`ConfigurationResult<T>`** - Configuration Operations

```rust
pub type ConfigurationResult<T = ()> = SongbirdResult<T>;
```

**When to Use**:
- Loading configuration
- Parsing config files
- Environment variable access

**Example**:
```rust
pub fn load_config(path: &str) -> ConfigurationResult<AppConfig> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| SongbirdError::configuration(format!("Failed to read config: {}", e)))?;
    
    let config: AppConfig = toml::from_str(&content)
        .map_err(|e| SongbirdError::configuration(format!("Failed to parse config: {}", e)))?;
    
    // Validate loaded config
    validate_config(&config)?;
    
    Ok(config)
}

pub fn get_env_config(key: &str) -> ConfigurationResult<String> {
    std::env::var(key)
        .map_err(|_| SongbirdError::configuration(format!("Missing env var: {}", key)))
}
```

---

### 🌐 7. **`ServiceOperationResult<T>`** - Service Operations

```rust
pub type ServiceOperationResult<T = ()> = SongbirdResult<T>;
```

**When to Use**:
- Service registration
- Service updates
- Service lifecycle

**Example**:
```rust
pub async fn register_service(info: ServiceInfo) -> ServiceOperationResult<()> {
    let registry = get_service_registry()?;
    
    registry.register(info)
        .await
        .map_err(|e| SongbirdError::service("registry", e.to_string()))
}

pub async fn update_service_metadata(
    service_id: &str,
    metadata: HashMap<String, String>,
) -> ServiceOperationResult<()> {
    let registry = get_service_registry()?;
    
    if !registry.is_registered(service_id).await? {
        return Err(SongbirdError::service(
            service_id,
            "Service not registered",
        ));
    }
    
    registry.update_metadata(service_id, metadata).await
}
```

---

### 🔌 8. **`NetworkOperationResult<T>`** - Network Operations

```rust
pub type NetworkOperationResult<T = ()> = SongbirdResult<T>;
```

**When to Use**:
- Network requests
- Connection management
- Protocol operations

**Example**:
```rust
pub async fn connect_to_service(endpoint: &str) -> NetworkOperationResult<Connection> {
    let connection = tokio::net::TcpStream::connect(endpoint)
        .await
        .map_err(|e| SongbirdError::network(format!("Connection failed: {}", e)))?;
    
    Ok(Connection::new(connection))
}

pub async fn send_request(
    conn: &mut Connection,
    request: &[u8],
) -> NetworkOperationResult<Vec<u8>> {
    conn.write_all(request)
        .await
        .map_err(|e| SongbirdError::network(format!("Write failed: {}", e)))?;
    
    let mut buffer = vec![0u8; 4096];
    let n = conn.read(&mut buffer)
        .await
        .map_err(|e| SongbirdError::network(format!("Read failed: {}", e)))?;
    
    buffer.truncate(n);
    Ok(buffer)
}
```

---

### 🔒 9. **`SecurityOperationResult<T>`** - Security Operations

```rust
pub type SecurityOperationResult<T = ()> = SongbirdResult<T>;
```

**When to Use**:
- Authentication
- Authorization
- Encryption/decryption
- Token operations

**Example**:
```rust
pub async fn authenticate_user(
    credentials: Credentials
) -> SecurityOperationResult<AuthToken> {
    let security_provider = get_security_provider()?;
    
    security_provider.authenticate(credentials)
        .await
        .map_err(|e| SongbirdError::security(format!("Authentication failed: {}", e)))
}

pub async fn encrypt_data(data: &[u8]) -> SecurityOperationResult<Vec<u8>> {
    let security_provider = get_security_provider()?;
    
    security_provider.encrypt(data)
        .await
        .map_err(|e| SongbirdError::security(format!("Encryption failed: {}", e)))
}
```

---

### 🌍 10. **`FederationOperationResult<T>`** - Federation Operations

```rust
pub type FederationOperationResult<T = ()> = SongbirdResult<T>;
```

**When to Use**:
- Cross-network operations
- Federation management
- Distributed operations

**Example**:
```rust
pub async fn join_federation(network_id: &str) -> FederationOperationResult<()> {
    let federation = get_federation_manager()?;
    
    federation.join(network_id)
        .await
        .map_err(|e| SongbirdError::network(format!("Federation join failed: {}", e)))
}

pub async fn sync_federated_services() -> FederationOperationResult<Vec<ServiceInfo>> {
    let federation = get_federation_manager()?;
    
    federation.discover_federated_services()
        .await
        .map_err(|e| SongbirdError::discovery(format!("Federation sync failed: {}", e)))
}
```

---

### 📦 11. **`MigrationResult<T>`** - Migration Operations

```rust
pub type MigrationResult<T = ()> = SongbirdResult<T>;
```

**When to Use**:
- Database migrations
- Data migrations
- Schema updates

**Example**:
```rust
pub async fn run_migration(migration: Migration) -> MigrationResult<()> {
    let db = get_database()?;
    
    // Start transaction
    let mut tx = db.begin()
        .await
        .map_err(|e| SongbirdError::service("database", e.to_string()))?;
    
    // Apply migration
    migration.apply(&mut tx)
        .await
        .map_err(|e| {
            SongbirdError::service("database", format!("Migration failed: {}", e))
        })?;
    
    // Commit
    tx.commit()
        .await
        .map_err(|e| SongbirdError::service("database", e.to_string()))?;
    
    Ok(())
}
```

---

## 🏗️ Specialized Result Structures

### `UnifiedOperationResult<T>`

For operations needing rich contextual information:

```rust
pub struct UnifiedOperationResult<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub duration_ms: u64,
    pub timestamp: u64,
    pub metadata: HashMap<String, String>,
}
```

**Example**:
```rust
pub async fn complex_operation() -> SongbirdResult<UnifiedOperationResult<Data>> {
    let start = std::time::Instant::now();
    
    match perform_operation().await {
        Ok(data) => {
            Ok(UnifiedOperationResult {
                success: true,
                data: Some(data),
                error: None,
                duration_ms: start.elapsed().as_millis() as u64,
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
                metadata: HashMap::new(),
            })
        }
        Err(e) => {
            Ok(UnifiedOperationResult {
                success: false,
                data: None,
                error: Some(e.to_string()),
                duration_ms: start.elapsed().as_millis() as u64,
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
                metadata: HashMap::new(),
            })
        }
    }
}
```

### `ServiceHealthResult`

For detailed health information:

```rust
pub struct ServiceHealthResult {
    pub status: UnifiedHealthStatus,
    pub components: HashMap<String, UnifiedHealthStatus>,
    pub timestamp: u64,
    pub check_duration_ms: u64,
    pub metadata: HashMap<String, String>,
}
```

### `DeploymentStatusResult`

For deployment progress tracking:

```rust
pub struct DeploymentStatusResult {
    pub phase: DeploymentPhase,
    pub progress: f32,            // 0.0 to 1.0
    pub services_deployed: u32,
    pub services_total: u32,
    pub started_at: u64,
    pub metadata: HashMap<String, String>,
}
```

---

## 💡 Common Patterns

### Pattern 1: Chain Multiple Results

```rust
pub async fn complex_workflow() -> SongbirdResult<Output> {
    // All errors auto-convert to SongbirdError via ?
    let config = load_config("app.toml")?;           // ConfigurationResult
    validate_config(&config)?;                       // ValidationResult
    let services = discover_services(criteria).await?; // DiscoveryResult
    let deployment = deploy_service(spec).await?;    // DeploymentResult
    let health = check_health().await?;              // HealthCheckResult
    
    Ok(Output { /* ... */ })
}
```

### Pattern 2: Convert Between Result Types

```rust
// All result types are aliases of SongbirdResult<T>
pub async fn convert_example() -> SongbirdResult<()> {
    let validation: ValidationResult<()> = validate_input("data");
    let deployment: DeploymentResult<()> = deploy().await;
    
    // Can mix freely - they're all SongbirdResult underneath
    validation?;
    deployment?;
    
    Ok(())
}
```

### Pattern 3: Match on Success/Failure

```rust
pub async fn handle_result() {
    match perform_operation().await {
        Ok(data) => {
            info!("Operation succeeded: {:?}", data);
            process_data(data).await;
        }
        Err(SongbirdError::Network { message, .. }) => {
            warn!("Network error: {}. Retrying...", message);
            retry_operation().await;
        }
        Err(SongbirdError::Validation { field, suggestion, .. }) => {
            error!("Validation failed on field: {:?}", field);
            if let Some(hint) = suggestion {
                info!("Hint: {}", hint);
            }
        }
        Err(e) => {
            error!("Operation failed: {}", e);
        }
    }
}
```

### Pattern 4: Early Return with Custom Result Type

```rust
pub async fn typed_operation() -> DeploymentResult<ServiceInfo> {
    // Validate first (returns ValidationResult, auto-converts)
    validate_deployment_spec(&spec)?;
    
    // Deploy (returns DeploymentResult)
    let deployment = deploy_service(spec).await?;
    
    // Type-specific return
    Ok(ServiceInfo {
        id: deployment.id,
        /* ... */
    })
}
```

---

## 📊 Result Type Selection Guide

| Operation Type | Use This Result | Example |
|----------------|----------------|---------|
| Generic | `SongbirdResult<T>` | Any function |
| Validation | `ValidationResult<T>` | Input validation |
| Deployment | `DeploymentResult<T>` | Service deployment |
| Health Check | `HealthCheckResult` | System health |
| Discovery | `DiscoveryResult<T>` | Service/primal lookup |
| Configuration | `ConfigurationResult<T>` | Config loading |
| Service Ops | `ServiceOperationResult<T>` | Service lifecycle |
| Network Ops | `NetworkOperationResult<T>` | Network calls |
| Security Ops | `SecurityOperationResult<T>` | Auth/encryption |
| Federation | `FederationOperationResult<T>` | Cross-network ops |
| Migration | `MigrationResult<T>` | Data migrations |

---

## ✅ Best Practices

### DO ✅

- Use domain-specific result types for clarity
- Chain results with `?` operator
- Provide rich error context
- Use `UnifiedOperationResult` for complex operations
- Match on specific error variants when needed

### DON'T ❌

- Don't use `unwrap()` or `expect()` in production
- Don't create new result type aliases
- Don't lose error information when mapping
- Don't ignore errors silently
- Don't mix panic-based and result-based error handling

---

## 🔧 Utility Functions

### Convert Standard Result to Unified Result

```rust
use songbird_types::results::utils::to_unified_result;

let standard_result: Result<Data, Error> = operation();
let unified: UnifiedOperationResult<Data> = to_unified_result(standard_result);
```

### Check If Operational

```rust
use songbird_types::results::utils::is_operational;

let health = check_health().await?;
if is_operational(&health) {
    // Service is healthy or degraded (still usable)
    proceed_with_operation().await?;
}
```

---

## 📚 Related Documentation

- **Error System**: `UNIFIED_ERRORS_QUICKREF.md`
- **Trait System**: `UNIFIED_TRAITS_QUICKREF.md`
- **Full Implementation**: `crates/songbird-types/src/results.rs`
- **Architecture**: `ARCHITECTURE_OVERVIEW.md`

---

## 📈 Consolidation Summary

**Before**: 66+ fragmented result types across crates  
**After**: 11 canonical result types (-85% reduction)

**Benefits**:
- ✅ Single source of truth
- ✅ Consistent patterns
- ✅ Type-safe
- ✅ Easy to understand
- ✅ Reduced maintenance

---

**Need Help?** Check the results source code or ask in #songbird-dev!

✅ **This result system is production-ready and powers all Songbird operations!**
