# API Migration Guide: Legacy Code to Phase 1 Architecture

## Overview

This guide helps developers migrate legacy Songbird Orchestrator code to work with the Phase 1 modernized architecture. The core system has been significantly improved with better error handling, cleaner APIs, and more robust patterns.

## Quick Reference: Major Changes

### Error System Migration
**Old (Tuple-style)** → **New (Struct-style)**

### API Signature Updates
**ScalingGroup**, **ServiceInfo**, **LoadBalancerConfig** structure changes

### Module Reorganization
New modules: `errors`, `load_balancer` with updated imports

---

## 1. Error Format Migration

### SongbirdError Changes

#### Configuration Errors
```rust
// ❌ OLD - Tuple-style
SongbirdError::Configuration("message".to_string())

// ✅ NEW - Struct-style
SongbirdError::Configuration {
    field: "field_name".to_string(),
    message: "message".to_string(),
}
```

#### Rate Limit Errors
```rust
// ❌ OLD
SongbirdError::RateLimit("Rate limit exceeded".to_string())

// ✅ NEW
SongbirdError::RateLimit {
    message: "Rate limit exceeded".to_string(),
}
```

#### Health Check Errors
```rust
// ❌ OLD
SongbirdError::HealthCheck {
    service: "service-id".to_string(),
    reason: "Health check failed".to_string(),
}

// ✅ NEW
SongbirdError::HealthCheck {
    message: "Health check failed for service-id".to_string(),
}
```

#### Internal Errors
```rust
// ❌ OLD
SongbirdError::Internal("Internal error occurred".to_string())

// ✅ NEW
SongbirdError::Internal {
    message: "Internal error occurred".to_string(),
}
```

#### Service Errors
```rust
// ❌ OLD
SongbirdError::Service {
    service: "service-id".to_string(),
    // other fields
}

// ✅ NEW - Use helper methods
SongbirdError::service_error("service-id", "Error message".to_string())
```

### Error Pattern Matching

#### Configuration Error Matching
```rust
// ❌ OLD
match error {
    SongbirdError::Configuration(msg) => {
        println!("Config error: {}", msg);
    }
}

// ✅ NEW
match error {
    SongbirdError::Configuration { field, message } => {
        println!("Config error in {}: {}", field, message);
    }
}
```

#### Rate Limit Error Matching
```rust
// ❌ OLD
match error {
    SongbirdError::RateLimit(_) => {
        println!("Rate limited");
    }
}

// ✅ NEW
match error {
    SongbirdError::RateLimit { message: _ } => {
        println!("Rate limited");
    }
}
```

---

## 2. ScalingGroup API Updates

### Constructor Changes
```rust
// ❌ OLD
let group = ScalingGroup::new(config);

// ✅ NEW - Requires service_id parameter
let group = ScalingGroup::new("service-id".to_string(), config);
```

### Field Access Changes
```rust
// ❌ OLD
assert_eq!(group.stats.total_instances, 0);

// ✅ NEW - stats renamed to metrics
assert_eq!(group.metrics.total_instances, 0);
```

### Service Instance Creation
```rust
// ❌ OLD - ServiceInstance with simple fields
let instance = ServiceInstance {
    id: "instance-1".to_string(),
    address: "127.0.0.1".to_string(),
    port: 8080,
    // ...
};

// ✅ NEW - ServiceInstance with ServiceInfo
let instance = ServiceInstance {
    service_info: ServiceInfo {
        id: "service-id".to_string(),
        name: "Service Name".to_string(),
        version: "1.0.0".to_string(),
        service_type: "hpc-service".to_string(),
        description: "Service description".to_string(),
        endpoints: vec![],
        capabilities: vec!["compute".to_string()],
        tags: HashMap::new(),
        metadata: HashMap::new(),
    },
    instance_id: "instance-1".to_string(),
    weight: 1,
    current_connections: 0,
    is_healthy: true,
    last_health_check: Some(chrono::Utc::now()),
};
```

---

## 3. ResourceUsage Field Updates

### Field Name Changes
```rust
// ❌ OLD
let usage = ResourceUsage {
    cpu_cores: 2.0,
    memory_mb: 1024,
    disk_mb: 5120,
    network_mbps: 100.0,
};

// ✅ NEW
let usage = ResourceUsage {
    cpu_percentage: 75.0,           // Changed from cpu_cores
    memory_usage_mb: 1024,          // Changed from memory_mb
    network_bytes_per_sec: 104857600, // Changed from network_mbps (100 Mbps)
    disk_io_bytes_per_sec: 5368709120, // Changed from disk_mb
};
```

### Field Access Updates
```rust
// ❌ OLD
println!("CPU: {} cores", usage.cpu_cores);
println!("Memory: {} MB", usage.memory_mb);
println!("Disk: {} MB", usage.disk_mb);
println!("Network: {} Mbps", usage.network_mbps);

// ✅ NEW
println!("CPU: {}%", usage.cpu_percentage);
println!("Memory: {} MB", usage.memory_usage_mb);
println!("Disk I/O: {} bytes/sec", usage.disk_io_bytes_per_sec);
println!("Network: {} bytes/sec", usage.network_bytes_per_sec);
```

---

## 4. LoadBalancerConfig Updates

### Configuration Field Changes
```rust
// ❌ OLD
let config = LoadBalancerConfig {
    health_check_interval: Duration::from_secs(30),
    retry_delay: Duration::from_secs(1),
    // other fields
};

// ✅ NEW
let config = LoadBalancerConfig {
    health_check_interval_ms: 30000, // Changed to milliseconds
    // retry_delay removed
    strategy: LoadBalancerStrategy::RoundRobin,
    health_check_enabled: true,
    max_retries: 3,
};
```

### Field Access Updates
```rust
// ❌ OLD
assert_eq!(config.health_check_interval.as_secs(), 30);
assert_eq!(config.retry_delay.as_secs(), 1);

// ✅ NEW
assert_eq!(config.health_check_interval_ms, 30000);
// retry_delay no longer available
```

---

## 5. ServiceInfo Structure Updates

### ServiceEndpoint Requirements
```rust
// ❌ OLD - Missing required fields
ServiceEndpoint {
    path: "/api/process".to_string(),
    method: "POST".to_string(),
    description: "Process data".to_string(),
}

// ✅ NEW - All required fields
ServiceEndpoint {
    path: "/api/process".to_string(),
    method: "POST".to_string(),
    description: "Process data".to_string(),
    parameters: vec![], // Required
    response_schema: None, // Required
}
```

### Removed Fields
```rust
// ❌ OLD - These fields no longer exist
let service = ServiceInfo {
    id: "service-1".to_string(),
    name: "Service".to_string(),
    address: "127.0.0.1".to_string(), // ❌ REMOVED
    port: 8080,                       // ❌ REMOVED
    // ...
};

// ✅ NEW - Use endpoints for address/port info
let service = ServiceInfo {
    id: "service-1".to_string(),
    name: "Service".to_string(),
    version: "1.0.0".to_string(),
    service_type: "hpc-service".to_string(),
    description: "Service description".to_string(),
    endpoints: vec![
        ServiceEndpoint {
            path: "http://127.0.0.1:8080".to_string(),
            method: "GET".to_string(),
            description: "Service endpoint".to_string(),
            parameters: vec![],
            response_schema: None,
        }
    ],
    capabilities: vec![],
    tags: HashMap::new(),
    metadata: HashMap::new(),
};
```

---

## 6. Orchestrator API Changes

### Service Lifecycle Management
```rust
// ❌ OLD - stop_service method removed
orchestrator.stop_service(&service_id).await?;

// ✅ NEW - Use service lifecycle directly
// Services manage their own lifecycle through UniversalService trait
// Or use the registry to unregister services
orchestrator.registry.unregister(&service_id).await?;
```

### Service Registration
```rust
// ❌ OLD - May have different signature expectations
let service_id = orchestrator.register_service(service, config).await?;

// ✅ NEW - Ensure service implements all required traits
// The service must implement UniversalService with proper Error type
let service_id = orchestrator.register_service(service, config).await?;
```

---

## 7. Import Updates

### Module Path Changes
```rust
// ❌ OLD - May not exist or be in different locations
use songbird_orchestrator::load_balancer::LoadBalancingStrategy;

// ✅ NEW - Updated import paths
use songbird_orchestrator::LoadBalancingStrategy; // Moved to crate root
use songbird_orchestrator::load_balancer::LoadBalancerStrategy; // Alternative name
```

### Discovery Module Updates
```rust
// ❌ OLD - May reference removed types
use songbird_orchestrator::discovery::{DatasetType, AccessLevel};

// ✅ NEW - Updated paths
use songbird_orchestrator::discovery::types::{DatasetType, AccessLevel};
```

---

## 8. UniversalService Trait Updates

### Error Type Requirements
```rust
// ❌ OLD - Boxed error types may not work
impl UniversalService for MyService {
    type Error = Box<dyn std::error::Error + Send + Sync>;
    // ...
}

// ✅ NEW - Use SongbirdError or concrete error types
impl UniversalService for MyService {
    type Error = SongbirdError;
    // ...
}
```

### Required Methods
```rust
// ❌ OLD - May be missing required methods
impl UniversalService for MyService {
    // Only some methods implemented
}

// ✅ NEW - All methods required
impl UniversalService for MyService {
    type Config = serde_json::Value;
    type Health = serde_json::Value;
    type Error = SongbirdError;

    async fn initialize(&mut self, config: Self::Config) -> Result<(), Self::Error> { /* */ }
    async fn start(&mut self) -> Result<(), Self::Error> { /* */ }
    async fn stop(&mut self) -> Result<(), Self::Error> { /* */ }
    async fn health_check(&self) -> Result<Self::Health, Self::Error> { /* */ }
    async fn handle_request(&self, request: ServiceRequest) -> Result<ServiceResponse, Self::Error> { /* */ }
    async fn update_config(&mut self, config: Self::Config) -> Result<(), Self::Error> { /* */ }
    async fn get_metrics(&self) -> Result<ServiceMetrics, Self::Error> { /* */ }
    fn service_info(&self) -> ServiceInfo { /* */ }
    async fn can_handle_load(&self) -> Result<bool, Self::Error> { /* */ }
    async fn get_load_factor(&self) -> Result<f64, Self::Error> { /* */ }
}
```

---

## 9. Configuration Updates

### SongbirdDiscoveryConfig
```rust
// ❌ OLD - Missing required fields
let config = SongbirdDiscoveryConfig {
    discovery_interval_ms: 5000,
    cache_ttl_ms: 30000,
    // Missing required fields
};

// ✅ NEW - All required fields
let config = SongbirdDiscoveryConfig {
    discovery_interval_ms: 5000,
    cache_ttl_ms: 30000,
    monitoring: MonitoringConfig::default(),
    network: NetworkConfig::default(),
    trust: TrustConfig::default(),
};
```

---

## 10. Testing Updates

### Test Helper Functions
```rust
// ❌ OLD - May use old error formats
fn create_temp_dir() -> Result<PathBuf, SongbirdError> {
    std::fs::create_dir_all(&path)
        .map_err(|e| SongbirdError::Configuration(format!("Failed: {}", e)))?;
}

// ✅ NEW - Use struct-style errors
fn create_temp_dir() -> Result<PathBuf, SongbirdError> {
    std::fs::create_dir_all(&path)
        .map_err(|e| SongbirdError::Configuration {
            field: "temp_dir".to_string(),
            message: format!("Failed: {}", e),
        })?;
}
```

---

## Common Migration Patterns

### 1. Error Creation Helper
```rust
// Helper function for common error patterns
fn config_error(field: &str, message: &str) -> SongbirdError {
    SongbirdError::Configuration {
        field: field.to_string(),
        message: message.to_string(),
    }
}

// Usage
return Err(config_error("service_id", "Service ID cannot be empty"));
```

### 2. ServiceInfo Builder Pattern
```rust
// Helper for creating ServiceInfo
fn create_hpc_service_info(id: &str, name: &str) -> ServiceInfo {
    ServiceInfo {
        id: id.to_string(),
        name: name.to_string(),
        version: "1.0.0".to_string(),
        service_type: "hpc-service".to_string(),
        description: format!("HPC service: {}", name),
        endpoints: vec![],
        capabilities: vec!["compute".to_string()],
        tags: HashMap::new(),
        metadata: HashMap::new(),
    }
}
```

### 3. ResourceUsage Conversion
```rust
// Helper for converting old resource format
fn convert_resource_usage(
    cpu_cores: f64,
    memory_mb: u64,
    disk_mb: u64,
    network_mbps: f64
) -> ResourceUsage {
    ResourceUsage {
        cpu_percentage: cpu_cores * 50.0, // Estimate percentage
        memory_usage_mb: memory_mb,
        network_bytes_per_sec: (network_mbps * 1_048_576.0) as u64, // Mbps to bytes/sec
        disk_io_bytes_per_sec: disk_mb * 1_048_576, // MB to bytes
    }
}
```

---

## Validation Checklist

After migration, verify:

- [ ] All compilation errors resolved
- [ ] Error patterns use struct-style syntax
- [ ] ScalingGroup constructor includes service_id
- [ ] ResourceUsage uses new field names
- [ ] LoadBalancerConfig uses new field names
- [ ] ServiceInfo includes all required ServiceEndpoint fields
- [ ] UniversalService implements all required methods
- [ ] Import paths updated for moved modules
- [ ] Tests pass with new API signatures
- [ ] Configuration objects include all required fields

---

## Getting Help

If you encounter issues during migration:

1. **Check Integration Tests**: Run `cargo test --test integration_test` to ensure core functionality works
2. **Review Examples**: Look at updated examples in the `examples/` directory
3. **Check Documentation**: Refer to the Phase 2 status documentation
4. **Incremental Migration**: Update one file at a time and test frequently

The core system is stable and tested - migration issues are typically just API signature updates that can be resolved systematically using this guide.

---
*Last Updated: Phase 2 Initiation*
*Status: Complete Migration Guide* 