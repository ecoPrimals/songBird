# Communication Layer Implementation Plan

**Priority:** ⭐⭐⭐⭐⭐ (Critical Path)  
**Timeline:** Weeks 1-3  
**Dependencies:** Core orchestrator, Load balancer integration  

## Problem Statement

The current communication layer is the biggest blocker to alpha functionality. While we have trait definitions and basic WebSocket scaffolding, the orchestrator cannot:

1. **Route requests between services** - No request proxying mechanism
2. **Handle service-to-service communication** - Missing protocol implementations
3. **Integrate with load balancer** - No service instance selection in request flow
4. **Support multiple protocols** - Only partial WebSocket implementation

## Implementation Architecture

### Core Request Flow
```
Client Request → Orchestrator HTTP API → RequestRouter → LoadBalancer 
    → ServiceInstance → CommunicationLayer → Target Service
    → ServiceResponse → Response Processing → Client Response
```

### Component Hierarchy
```rust
// New RequestRouter component
pub struct RequestRouter {
    load_balancer: Arc<dyn LoadBalancer>,
    communication: Arc<dyn CommunicationLayer>,
    service_registry: Arc<ServiceRegistry>,
    metrics: Arc<RequestMetrics>,
}

// Enhanced Orchestrator with routing
impl Orchestrator {
    pub async fn route_request(
        &self,
        target_service: &str,
        request: ServiceRequest,
    ) -> Result<ServiceResponse> {
        self.request_router.route(target_service, request).await
    }
}
```

## Phase 1: Request Router Core (Week 1)

### 1.1 Create RequestRouter Component

**File:** `src/orchestrator/request_router.rs`

```rust
use std::sync::Arc;
use tokio::time::{timeout, Duration};
use tracing::{info, warn, error};

use crate::load_balancer::{LoadBalancer, ServiceInstance};
use crate::traits::communication::CommunicationLayer;
use crate::traits::service::{ServiceRequest, ServiceResponse};
use crate::errors::{Result, SongbirdError};
use crate::registry::ServiceRegistry;

#[derive(Clone)]
pub struct RequestRouter {
    load_balancer: Arc<dyn LoadBalancer>,
    communication: Arc<dyn CommunicationLayer>,
    registry: Arc<ServiceRegistry>,
    config: RequestRouterConfig,
    metrics: Arc<RequestMetrics>,
}

#[derive(Debug, Clone)]
pub struct RequestRouterConfig {
    pub default_timeout: Duration,
    pub max_retries: u32,
    pub retry_delay: Duration,
    pub circuit_breaker_enabled: bool,
    pub enable_request_tracing: bool,
}

impl Default for RequestRouterConfig {
    fn default() -> Self {
        Self {
            default_timeout: Duration::from_secs(30),
            max_retries: 3,
            retry_delay: Duration::from_millis(100),
            circuit_breaker_enabled: true,
            enable_request_tracing: true,
        }
    }
}

impl RequestRouter {
    pub fn new(
        load_balancer: Arc<dyn LoadBalancer>,
        communication: Arc<dyn CommunicationLayer>,
        registry: Arc<ServiceRegistry>,
    ) -> Self {
        Self {
            load_balancer,
            communication,
            registry,
            config: RequestRouterConfig::default(),
            metrics: Arc::new(RequestMetrics::default()),
        }
    }

    pub async fn route_request(
        &self,
        service_id: &str,
        mut request: ServiceRequest,
    ) -> Result<ServiceResponse> {
        let start_time = std::time::Instant::now();
        
        // Add tracing information
        if self.config.enable_request_tracing {
            request.headers.insert(
                "x-trace-id".to_string(),
                uuid::Uuid::new_v4().to_string(),
            );
            request.headers.insert(
                "x-orchestrator-timestamp".to_string(),
                chrono::Utc::now().to_rfc3339(),
            );
        }

        // Get available service instances
        let instances = self.load_balancer.get_healthy_instances(service_id).await?;
        if instances.is_empty() {
            return Err(SongbirdError::ServiceUnavailable {
                service: service_id.to_string(),
                message: "No healthy instances available".to_string(),
            });
        }

        // Attempt request with retries
        let mut last_error = None;
        for attempt in 0..=self.config.max_retries {
            // Select service instance via load balancer
            let instance = self.load_balancer.select_instance(&instances).await?;
            
            // Route request to selected instance
            match self.send_request_to_instance(&instance, &request).await {
                Ok(response) => {
                    // Update metrics
                    self.metrics.record_success(
                        service_id,
                        &instance.id,
                        start_time.elapsed(),
                    );
                    
                    // Update load balancer stats
                    self.load_balancer.record_success(&instance.id).await?;
                    
                    return Ok(response);
                }
                Err(e) => {
                    last_error = Some(e);
                    
                    // Update failure metrics
                    self.metrics.record_failure(service_id, &instance.id);
                    self.load_balancer.record_failure(&instance.id).await?;
                    
                    // Wait before retry (except on last attempt)
                    if attempt < self.config.max_retries {
                        tokio::time::sleep(self.config.retry_delay).await;
                    }
                }
            }
        }

        // All retries failed
        Err(last_error.unwrap_or_else(|| {
            SongbirdError::RequestFailed {
                service: service_id.to_string(),
                message: "All retry attempts failed".to_string(),
            }
        }))
    }

    async fn send_request_to_instance(
        &self,
        instance: &ServiceInstance,
        request: &ServiceRequest,
    ) -> Result<ServiceResponse> {
        let service_address = ServiceAddress {
            service_id: instance.service_id.clone(),
            instance_id: Some(instance.id.clone()),
            endpoint: instance.endpoint.clone(),
        };

        // Convert ServiceRequest to ServiceMessage for communication layer
        let message = ServiceMessage {
            id: request.id.clone(),
            message_type: MessageType::Request,
            topic: None,
            payload: request.payload.clone(),
            headers: request.headers.clone(),
            timestamp: request.timestamp,
            correlation_id: Some(uuid::Uuid::new_v4().to_string()),
            reply_to: None,
            ttl: request.timeout.map(|d| d.as_secs()),
        };

        // Send request with timeout
        let timeout_duration = request.timeout.unwrap_or(self.config.default_timeout);
        let comm_response = timeout(
            timeout_duration,
            self.communication.send_message(service_address, message),
        )
        .await
        .map_err(|_| SongbirdError::RequestTimeout {
            service: instance.service_id.clone(),
            timeout: timeout_duration,
        })?
        .map_err(|e| SongbirdError::CommunicationFailed {
            service: instance.service_id.clone(),
            message: e.to_string(),
        })?;

        // Convert CommunicationResponse back to ServiceResponse
        let response = ServiceResponse {
            request_id: request.id.clone(),
            status: if comm_response.success {
                ResponseStatus::Success
            } else {
                ResponseStatus::Error {
                    code: 500,
                    message: comm_response
                        .error
                        .unwrap_or_else(|| "Unknown error".to_string()),
                }
            },
            headers: std::collections::HashMap::new(),
            payload: comm_response.payload.unwrap_or(serde_json::json!(null)),
            timestamp: comm_response.timestamp,
            duration: chrono::Utc::now()
                .signed_duration_since(request.timestamp)
                .to_std()
                .unwrap_or_default(),
            processing_time: 0, // Will be calculated by receiving service
            metadata: std::collections::HashMap::new(),
        };

        Ok(response)
    }

    pub fn get_metrics(&self) -> RequestMetrics {
        (*self.metrics).clone()
    }
}

#[derive(Debug, Clone, Default)]
pub struct RequestMetrics {
    pub total_requests: std::sync::atomic::AtomicU64,
    pub successful_requests: std::sync::atomic::AtomicU64,
    pub failed_requests: std::sync::atomic::AtomicU64,
    pub average_response_time: std::sync::atomic::AtomicU64, // milliseconds
    pub requests_by_service: dashmap::DashMap<String, ServiceRequestMetrics>,
}

#[derive(Debug, Clone, Default)]
pub struct ServiceRequestMetrics {
    pub total: u64,
    pub successful: u64,
    pub failed: u64,
    pub average_response_time: u64,
}

impl RequestMetrics {
    fn record_success(&self, service_id: &str, instance_id: &str, duration: Duration) {
        use std::sync::atomic::Ordering;
        
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.successful_requests.fetch_add(1, Ordering::Relaxed);
        
        let duration_ms = duration.as_millis() as u64;
        // Simple moving average approximation
        let current_avg = self.average_response_time.load(Ordering::Relaxed);
        let new_avg = if current_avg == 0 {
            duration_ms
        } else {
            (current_avg * 9 + duration_ms) / 10 // Exponential moving average
        };
        self.average_response_time.store(new_avg, Ordering::Relaxed);
        
        // Update per-service metrics
        let mut service_metrics = self.requests_by_service
            .entry(service_id.to_string())
            .or_insert_with(ServiceRequestMetrics::default);
        service_metrics.total += 1;
        service_metrics.successful += 1;
        service_metrics.average_response_time = 
            (service_metrics.average_response_time * (service_metrics.total - 1) + duration_ms) 
            / service_metrics.total;
    }

    fn record_failure(&self, service_id: &str, instance_id: &str) {
        use std::sync::atomic::Ordering;
        
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.failed_requests.fetch_add(1, Ordering::Relaxed);
        
        // Update per-service metrics
        let mut service_metrics = self.requests_by_service
            .entry(service_id.to_string())
            .or_insert_with(ServiceRequestMetrics::default);
        service_metrics.total += 1;
        service_metrics.failed += 1;
    }
}
```

### 1.2 Integrate RequestRouter into Orchestrator

**Modify:** `src/orchestrator/mod.rs`

```rust
// Add to Orchestrator struct
pub struct Orchestrator {
    // ... existing fields ...
    request_router: Arc<RequestRouter>,
    load_balancer: Arc<dyn LoadBalancer>,
    communication: Arc<dyn CommunicationLayer>,
}

impl Orchestrator {
    pub async fn new(config: OrchestratorConfig) -> Result<Self> {
        let registry = ServiceRegistry::new().await?;
        let (event_sender, _) = broadcast::channel(1000);

        // Initialize load balancer based on config
        let load_balancer: Arc<dyn LoadBalancer> = match config.load_balancer.strategy {
            LoadBalancingStrategy::RoundRobin => Arc::new(RoundRobinLoadBalancer::new()),
            LoadBalancingStrategy::LeastConnections => Arc::new(LeastConnectionsLoadBalancer::new()),
            LoadBalancingStrategy::WeightedRoundRobin => Arc::new(WeightedRoundRobinLoadBalancer::new()),
            // Add other strategies as needed
        };

        // Initialize communication layer based on config
        let communication: Arc<dyn CommunicationLayer> = match config.communication.protocol {
            CommunicationProtocol::WebSocket => {
                Arc::new(WebSocketCommunication::new(
                    config.communication.host.clone(),
                    config.communication.port,
                ))
            }
            CommunicationProtocol::Http => {
                Arc::new(HttpCommunication::new(
                    format!("http://{}:{}", config.communication.host, config.communication.port)
                ))
            }
            // Add other protocols as needed
        };

        // Create request router
        let request_router = Arc::new(RequestRouter::new(
            Arc::clone(&load_balancer),
            Arc::clone(&communication),
            Arc::clone(&registry),
        ));

        Ok(Self {
            config: Arc::new(config),
            registry: Arc::new(registry),
            services: Arc::new(DashMap::new()),
            metrics: Arc::new(RwLock::new(OrchestratorMetrics::default())),
            event_sender,
            shutdown_signal: Arc::new(AtomicBool::new(false)),
            started_at: Instant::now(),
            request_router,
            load_balancer,
            communication,
        })
    }

    /// Handle a service request by routing it through the load balancer
    pub async fn handle_service_request(
        &self,
        service_id: &str,
        request: ServiceRequest,
    ) -> Result<ServiceResponse> {
        // Validate service exists
        if !self.services.contains_key(service_id) {
            return Err(SongbirdError::ServiceNotFound {
                service: service_id.to_string(),
            });
        }

        // Route request
        self.request_router.route_request(service_id, request).await
    }

    /// Register multiple instances of the same service
    pub async fn register_service_instance<S>(
        &self,
        service: S,
        config: S::Config,
        instance_id: Option<String>,
    ) -> Result<String>
    where
        S: UniversalService + 'static,
        S::Config: Clone + Send + Sync + for<'de> serde::de::Deserialize<'de> + std::fmt::Debug,
        S::Health: Send + Sync + serde::Serialize + std::fmt::Debug,
        S::Error: std::error::Error + Send + Sync + 'static,
    {
        // Generate instance ID if not provided
        let instance_id = instance_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        
        // Register with base register_service method
        let service_id = self.register_service(service, config).await?;
        
        // Add instance to load balancer
        let service_info = self.services.get(&service_id)
            .ok_or_else(|| SongbirdError::ServiceNotFound { 
                service: service_id.clone() 
            })?;
        
        let service_instance = load_balancer::ServiceInstance {
            id: instance_id.clone(),
            service_id: service_id.clone(),
            endpoint: service_info.info.endpoints.first().map(|e| e.path.clone()),
            health: load_balancer::InstanceHealth::Healthy,
            weight: 1.0,
            metadata: std::collections::HashMap::new(),
        };
        
        self.load_balancer.add_instance(service_instance).await?;
        
        Ok(instance_id)
    }

    pub fn get_request_metrics(&self) -> RequestMetrics {
        self.request_router.get_metrics()
    }
}
```

## Phase 2: HTTP Communication Implementation (Week 2)

### 2.1 Complete HTTP Communication Backend

**Enhance:** `src/communication/mod.rs`

```rust
use reqwest::Client;
use std::collections::HashMap;
use std::time::Duration;

pub struct HttpCommunication {
    client: Client,
    base_url: String,
    timeout: Duration,
    default_headers: HashMap<String, String>,
}

impl HttpCommunication {
    pub fn new(base_url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("songbird-orchestrator/0.1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url,
            timeout: Duration::from_secs(30),
            default_headers: HashMap::new(),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self.client = Client::builder()
            .timeout(timeout)
            .user_agent("songbird-orchestrator/0.1.0")
            .build()
            .expect("Failed to create HTTP client");
        self
    }

    fn build_url(&self, address: &ServiceAddress) -> String {
        match &address.endpoint {
            Some(endpoint) => {
                if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
                    endpoint.clone()
                } else {
                    format!("{}/{}", self.base_url.trim_end_matches('/'), endpoint.trim_start_matches('/'))
                }
            }
            None => format!("{}/services/{}", self.base_url, address.service_id),
        }
    }
}

#[async_trait]
impl CommunicationLayer for HttpCommunication {
    async fn send_message(
        &self,
        target: ServiceAddress,
        message: ServiceMessage,
    ) -> Result<CommunicationResponse> {
        let url = self.build_url(&target);
        
        // Build request
        let mut request = self.client.post(&url);
        
        // Add headers
        for (key, value) in &self.default_headers {
            request = request.header(key, value);
        }
        for (key, value) in &message.headers {
            request = request.header(key, value);
        }
        
        // Add correlation headers
        request = request.header("x-message-id", &message.id);
        if let Some(correlation_id) = &message.correlation_id {
            request = request.header("x-correlation-id", correlation_id);
        }
        request = request.header("x-message-type", format!("{:?}", message.message_type));
        
        // Send request
        let response = request
            .json(&message)
            .send()
            .await
            .map_err(|e| SongbirdError::Network(std::io::Error::other(e)))?;

        let success = response.status().is_success();
        let status_code = response.status().as_u16();
        
        // Parse response
        let payload = if success {
            response
                .json::<serde_json::Value>()
                .await
                .map_err(|e| SongbirdError::Serialization(e.into()))?
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            serde_json::json!({ "error": error_text, "status": status_code })
        };

        Ok(CommunicationResponse {
            message_id: message.id,
            success,
            payload: Some(payload),
            error: if success {
                None
            } else {
                Some(format!("HTTP {} error", status_code))
            },
            timestamp: Utc::now(),
        })
    }

    async fn broadcast(&self, message: ServiceMessage) -> Result<Vec<CommunicationResponse>> {
        // HTTP doesn't support native broadcast - would need service registry integration
        // For now, return empty vec or error
        Err(SongbirdError::UnsupportedOperation {
            operation: "broadcast".to_string(),
            reason: "HTTP communication layer doesn't support broadcast without service registry".to_string(),
        })
    }

    async fn listen(&self) -> impl Stream<Item = (ServiceAddress, ServiceMessage)> {
        // HTTP is request/response, not streaming
        // Return empty stream
        futures_util::stream::empty()
    }

    async fn subscribe(&self, _topic: &str) -> Result<()> {
        // HTTP doesn't support pub/sub
        Ok(())
    }

    async fn unsubscribe(&self, _topic: &str) -> Result<()> {
        Ok(())
    }

    async fn connect(&self) -> Result<()> {
        // HTTP is connectionless, but we can do a health check
        match self.client.get(&self.base_url).send().await {
            Ok(_) => Ok(()),
            Err(e) => Err(SongbirdError::Network(std::io::Error::other(e))),
        }
    }

    async fn disconnect(&self) -> Result<()> {
        // HTTP is connectionless
        Ok(())
    }

    async fn is_connected(&self) -> bool {
        // Always considered "connected" for HTTP
        true
    }

    async fn get_stats(&self) -> Result<CommunicationStats> {
        Ok(CommunicationStats {
            messages_sent: 0, // Would need internal tracking
            messages_received: 0,
            bytes_sent: 0,
            bytes_received: 0,
            active_connections: 1, // HTTP is connectionless, but indicate ready
            failed_connections: 0,
            last_activity: Some(Utc::now()),
        })
    }
}
```

## Phase 3: Load Balancer Integration (Week 3)

### 3.1 Enhanced LoadBalancer Trait Integration

**Enhance:** `src/load_balancer.rs`

```rust
// Add methods required by RequestRouter
#[async_trait]
pub trait LoadBalancer: Send + Sync {
    async fn add_instance(&self, instance: ServiceInstance) -> Result<()>;
    async fn remove_instance(&self, instance_id: &str) -> Result<()>;
    async fn get_healthy_instances(&self, service_id: &str) -> Result<Vec<ServiceInstance>>;
    async fn select_instance(&self, instances: &[ServiceInstance]) -> Result<ServiceInstance>;
    async fn record_success(&self, instance_id: &str) -> Result<()>;
    async fn record_failure(&self, instance_id: &str) -> Result<()>;
    async fn update_instance_health(&self, instance_id: &str, health: InstanceHealth) -> Result<()>;
    async fn get_stats(&self) -> Result<LoadBalancerStats>;
}

#[derive(Debug, Clone)]
pub struct ServiceInstance {
    pub id: String,
    pub service_id: String,
    pub endpoint: Option<String>,
    pub health: InstanceHealth,
    pub weight: f64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InstanceHealth {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}
```

### 3.2 Round Robin Load Balancer Implementation

```rust
use dashmap::DashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct RoundRobinLoadBalancer {
    instances: Arc<DashMap<String, Vec<ServiceInstance>>>, // service_id -> instances
    counters: Arc<DashMap<String, AtomicUsize>>, // service_id -> counter
    stats: Arc<LoadBalancerStats>,
}

impl RoundRobinLoadBalancer {
    pub fn new() -> Self {
        Self {
            instances: Arc::new(DashMap::new()),
            counters: Arc::new(DashMap::new()),
            stats: Arc::new(LoadBalancerStats::default()),
        }
    }
}

#[async_trait]
impl LoadBalancer for RoundRobinLoadBalancer {
    async fn add_instance(&self, instance: ServiceInstance) -> Result<()> {
        let service_id = instance.service_id.clone();
        
        self.instances
            .entry(service_id.clone())
            .or_insert_with(Vec::new)
            .push(instance);
            
        // Initialize counter if not exists
        self.counters
            .entry(service_id)
            .or_insert_with(|| AtomicUsize::new(0));
            
        Ok(())
    }

    async fn remove_instance(&self, instance_id: &str) -> Result<()> {
        for mut entry in self.instances.iter_mut() {
            entry.value_mut().retain(|instance| instance.id != instance_id);
        }
        Ok(())
    }

    async fn get_healthy_instances(&self, service_id: &str) -> Result<Vec<ServiceInstance>> {
        match self.instances.get(service_id) {
            Some(instances) => {
                let healthy: Vec<ServiceInstance> = instances
                    .iter()
                    .filter(|instance| instance.health == InstanceHealth::Healthy)
                    .cloned()
                    .collect();
                Ok(healthy)
            }
            None => Ok(vec![]),
        }
    }

    async fn select_instance(&self, instances: &[ServiceInstance]) -> Result<ServiceInstance> {
        if instances.is_empty() {
            return Err(SongbirdError::NoAvailableInstances);
        }

        if instances.len() == 1 {
            return Ok(instances[0].clone());
        }

        let service_id = &instances[0].service_id;
        let counter = self.counters
            .get(service_id)
            .ok_or_else(|| SongbirdError::ServiceNotFound { 
                service: service_id.clone() 
            })?;

        let index = counter.fetch_add(1, Ordering::Relaxed) % instances.len();
        Ok(instances[index].clone())
    }

    async fn record_success(&self, instance_id: &str) -> Result<()> {
        // Update statistics
        self.stats.successful_requests.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    async fn record_failure(&self, instance_id: &str) -> Result<()> {
        // Update statistics and potentially mark instance as unhealthy
        self.stats.failed_requests.fetch_add(1, Ordering::Relaxed);
        
        // Could implement circuit breaker logic here
        // For now, just record the failure
        Ok(())
    }

    async fn update_instance_health(&self, instance_id: &str, health: InstanceHealth) -> Result<()> {
        for mut entry in self.instances.iter_mut() {
            for instance in entry.value_mut().iter_mut() {
                if instance.id == instance_id {
                    instance.health = health;
                    return Ok(());
                }
            }
        }
        
        Err(SongbirdError::InstanceNotFound {
            instance: instance_id.to_string(),
        })
    }

    async fn get_stats(&self) -> Result<LoadBalancerStats> {
        Ok((*self.stats).clone())
    }
}
```

## Testing Strategy

### Unit Tests
- RequestRouter routing logic
- Load balancer selection algorithms  
- Communication layer protocol handling
- Error handling and retries

### Integration Tests
- End-to-end request flow: Client → Orchestrator → Service
- Multiple service instances with load balancing
- Service failure scenarios and recovery
- Communication protocol switching

### Example Test
```rust
#[tokio::test]
async fn test_request_routing_with_load_balancing() {
    // Setup orchestrator with mock services
    let mut orchestrator = create_test_orchestrator().await;
    
    // Register service with multiple instances
    let service_id = orchestrator.register_service_instance(
        MockWebService::new(), 
        MockConfig::default(),
        Some("instance-1".to_string())
    ).await.unwrap();
    
    orchestrator.register_service_instance(
        MockWebService::new(), 
        MockConfig::default(),
        Some("instance-2".to_string())
    ).await.unwrap();
    
    // Send multiple requests and verify load balancing
    let requests = (0..10).map(|i| ServiceRequest {
        id: format!("req-{}", i),
        method: "GET".to_string(),
        path: "/test".to_string(),
        headers: HashMap::new(),
        payload: serde_json::json!({"test": i}),
        timestamp: Utc::now(),
        timeout: Some(Duration::from_secs(5)),
        client_info: None,
        metadata: HashMap::new(),
    });
    
    for request in requests {
        let response = orchestrator.handle_service_request(&service_id, request).await.unwrap();
        assert_eq!(response.status, ResponseStatus::Success);
    }
    
    // Verify both instances received requests
    let metrics = orchestrator.get_request_metrics();
    assert!(metrics.requests_by_service.contains_key(&service_id));
}
```

## Success Criteria

By the end of Week 3:
- [ ] Complete request routing from client to service
- [ ] Load balancing working with multiple service instances  
- [ ] HTTP and WebSocket communication protocols functional
- [ ] Request/response correlation and tracing
- [ ] Error handling with retries and timeouts
- [ ] Integration tests covering main request flows
- [ ] Performance testing showing <10ms orchestration overhead

This implementation provides the critical foundation for service orchestration and addresses the primary feedback about missing communication capabilities. 