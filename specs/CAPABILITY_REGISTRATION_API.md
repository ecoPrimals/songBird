# 🔌 Capability Registration API Specification

**Version**: 1.0  
**Date**: November 10, 2025  
**Status**: Implementation In Progress  
**Owner**: Songbird Orchestrator

---

## 🎯 Overview

The Capability Registration API allows external compute providers (like Toadstool) to dynamically register their capabilities with Songbird, enabling intelligent routing of tasks to specialized compute resources.

**Flow**:
```
External Provider (Toadstool) → POST /api/v1/federation/register → Songbird
                              ← Registration Response ←
                              
Provider sends heartbeats → POST /api/v1/federation/heartbeat → Songbird
                         ← Health acknowledgment ←

User submits task → Songbird Compute API → Complexity Analysis
                 → Capability Registry Lookup → Route to Provider
                 → Provider executes → Results back to user
```

---

## 📋 API Endpoints

### 1. Register Capability Provider

**Endpoint**: `POST /api/v1/federation/register`  
**Purpose**: Register a new capability provider with Songbird

#### Request Format

```json
{
  "provider_id": "toadstool-node-1",
  "provider_name": "Toadstool Compute Node",
  "provider_type": "compute",
  "version": "1.0.0",
  "endpoint": "http://toadstool-node-1.local:9000",
  "capabilities": [
    {
      "name": "compute_gpu",
      "description": "GPU-accelerated computation",
      "metadata": {
        "gpu_count": 4,
        "gpu_type": "NVIDIA RTX 4090",
        "vram_gb": 96,
        "cuda_version": "12.2"
      }
    },
    {
      "name": "compute_heavy",
      "description": "Heavy CPU computation",
      "metadata": {
        "cpu_cores": 64,
        "ram_gb": 256,
        "architecture": "x86_64"
      }
    },
    {
      "name": "ml_training",
      "description": "Machine learning model training",
      "metadata": {
        "frameworks": ["pytorch", "tensorflow", "jax"],
        "max_batch_size": 1024
      }
    }
  ],
  "workload_endpoint": "/api/v1/workload/execute",
  "health_endpoint": "/api/v1/health",
  "metadata": {
    "location": "tower-b",
    "priority": 10,
    "max_concurrent_tasks": 8
  }
}
```

#### Request Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRegistrationRequest {
    /// Unique identifier for the provider
    pub provider_id: String,
    
    /// Human-readable provider name
    pub provider_name: String,
    
    /// Type of provider (compute, storage, security, etc.)
    pub provider_type: String,
    
    /// Provider version
    pub version: String,
    
    /// Base HTTP endpoint for the provider
    pub endpoint: String,
    
    /// List of capabilities this provider offers
    pub capabilities: Vec<CapabilityDescriptor>,
    
    /// Relative path for workload execution
    pub workload_endpoint: String,
    
    /// Relative path for health checks
    pub health_endpoint: String,
    
    /// Additional provider metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityDescriptor {
    /// Capability name (e.g., "compute_gpu", "ml_training")
    pub name: String,
    
    /// Human-readable description
    pub description: String,
    
    /// Capability-specific metadata
    pub metadata: HashMap<String, serde_json::Value>,
}
```

#### Response Format (Success)

```json
{
  "success": true,
  "data": {
    "provider_id": "toadstool-node-1",
    "registration_id": "reg-550e8400-e29b-41d4-a716-446655440000",
    "status": "registered",
    "heartbeat_interval_ms": 5000,
    "heartbeat_endpoint": "/api/v1/federation/heartbeat"
  },
  "message": "Provider successfully registered",
  "timestamp": "2025-11-10T10:30:00Z"
}
```

#### Response Format (Error)

```json
{
  "success": false,
  "error": {
    "code": "DUPLICATE_PROVIDER",
    "message": "Provider with ID 'toadstool-node-1' is already registered",
    "suggestion": "Use a different provider_id or unregister the existing provider first"
  },
  "timestamp": "2025-11-10T10:30:00Z"
}
```

---

### 2. Heartbeat / Keep-Alive

**Endpoint**: `POST /api/v1/federation/heartbeat`  
**Purpose**: Maintain registration and report health status

#### Request Format

```json
{
  "provider_id": "toadstool-node-1",
  "registration_id": "reg-550e8400-e29b-41d4-a716-446655440000",
  "health_status": {
    "status": "healthy",
    "active_tasks": 3,
    "available_capacity": 5,
    "resource_usage": {
      "cpu_percent": 45.2,
      "memory_percent": 62.1,
      "gpu_utilization": [78.5, 82.3, 45.1, 92.0]
    }
  },
  "timestamp": "2025-11-10T10:30:05Z"
}
```

#### Response Format

```json
{
  "success": true,
  "data": {
    "acknowledged": true,
    "next_heartbeat_ms": 5000
  },
  "timestamp": "2025-11-10T10:30:05Z"
}
```

---

### 3. Unregister Provider

**Endpoint**: `DELETE /api/v1/federation/register/{provider_id}`  
**Purpose**: Gracefully remove a provider from the registry

#### Response Format

```json
{
  "success": true,
  "data": {
    "provider_id": "toadstool-node-1",
    "status": "unregistered"
  },
  "message": "Provider successfully unregistered",
  "timestamp": "2025-11-10T10:35:00Z"
}
```

---

### 4. List Registered Providers

**Endpoint**: `GET /api/v1/federation/providers`  
**Purpose**: Query all registered capability providers

#### Response Format

```json
{
  "success": true,
  "data": {
    "providers": [
      {
        "provider_id": "toadstool-node-1",
        "provider_name": "Toadstool Compute Node",
        "status": "healthy",
        "capabilities": ["compute_gpu", "compute_heavy", "ml_training"],
        "last_heartbeat": "2025-11-10T10:30:05Z",
        "active_tasks": 3
      }
    ],
    "total_count": 1
  },
  "timestamp": "2025-11-10T10:30:10Z"
}
```

---

## 🗃️ Data Structures

### Capability Registry (Internal)

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Central registry for capability providers
pub struct CapabilityRegistry {
    /// Map of provider_id -> RegisteredProvider
    providers: Arc<RwLock<HashMap<String, RegisteredProvider>>>,
    
    /// Heartbeat timeout in milliseconds
    heartbeat_timeout_ms: u64,
}

#[derive(Debug, Clone)]
pub struct RegisteredProvider {
    /// Registration request details
    pub registration: CapabilityRegistrationRequest,
    
    /// Unique registration ID
    pub registration_id: String,
    
    /// Current health status
    pub health: ProviderHealth,
    
    /// When this provider was registered
    pub registered_at: DateTime<Utc>,
    
    /// Last successful heartbeat
    pub last_heartbeat: DateTime<Utc>,
    
    /// Number of tasks currently assigned to this provider
    pub active_tasks: usize,
}

#[derive(Debug, Clone)]
pub struct ProviderHealth {
    /// Overall health status
    pub status: HealthStatus,
    
    /// Available capacity for new tasks
    pub available_capacity: usize,
    
    /// Current resource utilization
    pub resource_usage: ResourceUsage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Offline,
}

#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub gpu_utilization: Vec<f64>,
}
```

---

## 🔄 Capability Routing Logic

### Router Enhancement

```rust
use crate::core::routing::CapabilityRouter;

impl CapabilityRouter {
    /// Route a task based on capabilities
    pub async fn route_task(
        &self,
        task: &ComputeTask,
        required_capability: &str,
    ) -> SongbirdResult<TaskRoutingDecision> {
        // 1. Check if capability can be handled locally
        if self.can_handle_locally(required_capability) {
            return Ok(TaskRoutingDecision::Local);
        }
        
        // 2. Query capability registry for external providers
        let providers = self.registry
            .find_providers_with_capability(required_capability)
            .await?;
        
        if providers.is_empty() {
            return Err(SongbirdError::discovery(
                format!("No providers found for capability: {}", required_capability)
            ));
        }
        
        // 3. Select best provider based on load, health, and priority
        let selected = self.select_best_provider(&providers, task).await?;
        
        // 4. Return routing decision
        Ok(TaskRoutingDecision::External {
            provider_id: selected.registration.provider_id.clone(),
            endpoint: format!(
                "{}{}",
                selected.registration.endpoint,
                selected.registration.workload_endpoint
            ),
        })
    }
    
    /// Execute task on external provider
    pub async fn execute_external(
        &self,
        endpoint: &str,
        task: &ComputeTask,
    ) -> SongbirdResult<ComputeResponse> {
        let client = reqwest::Client::new();
        
        let response = client
            .post(endpoint)
            .json(&task)
            .timeout(Duration::from_secs(300))
            .send()
            .await
            .map_err(|e| SongbirdError::network(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(SongbirdError::Service {
                service: "external_provider".to_string(),
                message: format!("Provider returned error: {}", response.status()),
                suggested_alternatives: vec![],
                recovery_actions: vec!["retry".to_string()],
            });
        }
        
        response
            .json()
            .await
            .map_err(|e| SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: e.to_string(),
                debug_info: None,
            })
    }
}

pub enum TaskRoutingDecision {
    Local,
    External {
        provider_id: String,
        endpoint: String,
    },
}
```

---

## ⏱️ Heartbeat Mechanism

### Configuration

```rust
/// Heartbeat configuration
pub struct HeartbeatConfig {
    /// Expected interval between heartbeats (milliseconds)
    pub interval_ms: u64,
    
    /// Number of missed heartbeats before marking provider unhealthy
    pub missed_threshold: u32,
    
    /// Number of missed heartbeats before removing provider
    pub removal_threshold: u32,
}

impl Default for HeartbeatConfig {
    fn default() -> Self {
        Self {
            interval_ms: 5000,        // 5 seconds
            missed_threshold: 3,       // 15 seconds
            removal_threshold: 12,     // 60 seconds
        }
    }
}
```

### Background Monitor

```rust
impl CapabilityRegistry {
    /// Start background task to monitor provider health
    pub fn start_health_monitor(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                Duration::from_millis(self.heartbeat_timeout_ms)
            );
            
            loop {
                interval.tick().await;
                self.check_provider_health().await;
            }
        });
    }
    
    async fn check_provider_health(&self) {
        let mut providers = self.providers.write().await;
        let now = Utc::now();
        
        for (provider_id, provider) in providers.iter_mut() {
            let elapsed = (now - provider.last_heartbeat).num_seconds();
            
            // Update health status based on heartbeat freshness
            if elapsed > 60 {
                tracing::warn!(
                    provider_id = %provider_id,
                    elapsed_seconds = elapsed,
                    "Provider offline - removing from registry"
                );
                provider.health.status = HealthStatus::Offline;
            } else if elapsed > 15 {
                tracing::warn!(
                    provider_id = %provider_id,
                    elapsed_seconds = elapsed,
                    "Provider unhealthy - missing heartbeats"
                );
                provider.health.status = HealthStatus::Unhealthy;
            }
        }
        
        // Remove offline providers
        providers.retain(|_, p| p.health.status != HealthStatus::Offline);
    }
}
```

---

## 🔐 Security Considerations

### Authentication

- **Mutual TLS**: Providers should authenticate with client certificates
- **API Keys**: Temporary solution - providers include API key in registration
- **Token-based**: JWT tokens issued on successful registration

### Rate Limiting

- Limit registration attempts per IP (10 per minute)
- Limit heartbeat frequency (max 1 per second)
- Limit concurrent registrations (max 100 providers)

### Validation

- Validate provider endpoints are reachable before accepting registration
- Verify capabilities match expected schema
- Sanitize all input fields to prevent injection attacks

---

## 🧪 Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_register_provider() {
        let registry = CapabilityRegistry::new();
        
        let request = CapabilityRegistrationRequest {
            provider_id: "test-provider".to_string(),
            provider_name: "Test Provider".to_string(),
            provider_type: "compute".to_string(),
            version: "1.0.0".to_string(),
            endpoint: "http://localhost:9000".to_string(),
            capabilities: vec![
                CapabilityDescriptor {
                    name: "compute_gpu".to_string(),
                    description: "GPU compute".to_string(),
                    metadata: HashMap::new(),
                }
            ],
            workload_endpoint: "/execute".to_string(),
            health_endpoint: "/health".to_string(),
            metadata: HashMap::new(),
        };
        
        let result = registry.register(request).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_heartbeat_timeout() {
        // Test that providers are marked unhealthy after missed heartbeats
    }
    
    #[tokio::test]
    async fn test_capability_routing() {
        // Test that tasks are routed to correct providers
    }
}
```

### Integration Tests

1. **Full Registration Flow**: Provider registers → Heartbeats → Unregisters
2. **Task Routing**: Submit GPU task → Routes to Toadstool → Executes → Returns results
3. **Failover**: Provider goes offline → Tasks route to backup provider
4. **Load Balancing**: Multiple providers with same capability → Even distribution

---

## 📝 Implementation Checklist

- [ ] Add `CapabilityRegistry` struct to `crates/songbird-orchestrator/src/core/registry.rs`
- [ ] Implement registration endpoint in `crates/songbird-orchestrator/src/server/federation_api.rs`
- [ ] Add heartbeat endpoint and background monitor
- [ ] Enhance `CapabilityRouter` to query registry
- [ ] Add external task execution logic
- [ ] Implement provider health monitoring
- [ ] Add authentication/authorization
- [ ] Write unit tests for registry operations
- [ ] Write integration tests for full flow
- [ ] Document API in OpenAPI/Swagger format
- [ ] Update `NEXT_STEPS_HANDOFF.md` with completion status

---

## 🔗 Related Documentation

- **Intelligent Routing**: `specs/INTELLIGENT_CAPABILITY_ROUTING_SPEC.md`
- **Compute API**: `specs/COMPUTE_API_INTEGRATION.md`
- **Integration Plan**: `TOADSTOOL_SONGBIRD_INTEGRATION_PLAN.md`
- **Architecture**: `PRIMAL_RESPONSIBILITY_MATRIX.md`

---

**Status**: 📝 Specification Complete - Ready for Implementation  
**Next Step**: Implement `CapabilityRegistry` and registration endpoint  
**Target Completion**: November 15, 2025

