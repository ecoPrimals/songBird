---
description: ENFORCE universal multi-protocol orchestration with dynamic service discovery and coordination
globs: ["songbird/src/**/*.rs", "songbird/crates/**/*.rs"]
---

# Universal Multi-Protocol Orchestration Specification

## Context
- When implementing songbird's universal orchestration capabilities
- When coordinating between any primals in the ecosystem
- When managing dynamic service discovery across deployment models
- When implementing real-time coordination protocols for any service type

## Requirements

### Universal Protocol Architecture
- **External Layer**: WebSocket JSON for web UIs, HTTP/REST for APIs, SSE for real-time updates
- **Internal Layer**: Universal request/response format for high-performance service-to-service communication
- **Streaming Layer**: Protocol-agnostic streaming for AI agents, real-time data, and events
- **Event Layer**: Universal event system for cross-primal coordination
- **Agnostic Protocol Bridge**: Enable seamless translation between any protocol types
- **Dynamic Protocol Selection**: Choose optimal protocol based on service capabilities and requirements

### Universal Service Discovery
- Implement agnostic service registry that works with any primal type
- Support primal capability advertisement and matching
- Enable automatic load balancing based on service capabilities
- Provide universal health monitoring and failover
- Support pluggable discovery backends (Kubernetes, Consul, DNS, Static, Custom)

### Universal Real-Time Coordination
- Implement bidirectional event streaming for any service type
- Support subscription-based event delivery with universal filtering
- Enable cross-primal coordination workflows
- Provide real-time status monitoring for all ecosystem services
- Support protocol-agnostic pub/sub patterns

## Architecture

### Core Universal Orchestration Engine
```rust
pub struct UniversalOrchestrator {
    protocol_router: Arc<UniversalProtocolRouter>,
    service_registry: Arc<UniversalServiceRegistry>,
    event_coordinator: Arc<UniversalEventCoordinator>,
    load_balancer: Arc<UniversalLoadBalancer>,
    health_monitor: Arc<UniversalHealthMonitor>,
    discovery_backends: Vec<Box<dyn DiscoveryBackend>>,
    security_manager: Arc<UniversalSecurityManager>,
}

impl UniversalOrchestrator {
    /// Register any service with universal metadata
    pub async fn register_service(&self, registration: UniversalServiceRegistration) -> Result<String, OrchestrationError> {
        // Universal service registration logic
        let service_id = self.service_registry.register(registration).await?;
        
        // Notify all discovery backends
        for backend in &self.discovery_backends {
            backend.notify_service_registered(&service_id).await?;
        }
        
        Ok(service_id)
    }
    
    /// Route requests based on service capabilities
    pub async fn route_request(&self, request: UniversalRequest) -> Result<UniversalResponse, OrchestrationError> {
        // Find services by capability matching
        let services = self.service_registry.find_by_capability(&request.required_capabilities).await?;
        
        // Select optimal service instance
        let instance = self.load_balancer.select_instance(&services, &request).await?;
        
        // Route request through appropriate protocol
        self.protocol_router.route_request(request, instance).await
    }
    
    /// Coordinate cross-primal operations
    pub async fn coordinate_primals(&self, coordination: CrossPrimalCoordination) -> Result<CoordinationResult, OrchestrationError> {
        // Universal primal coordination logic
        let participants = self.discover_primal_participants(&coordination.required_capabilities).await?;
        
        // Execute coordination workflow
        self.event_coordinator.execute_coordination(coordination, participants).await
    }
}
```

### Universal Protocol Router
```rust
pub struct UniversalProtocolRouter {
    protocol_handlers: HashMap<String, Box<dyn ProtocolHandler>>,
    translator: Arc<UniversalProtocolTranslator>,
}

#[async_trait]
pub trait ProtocolHandler: Send + Sync {
    /// Protocol name (e.g., "http", "websocket", "tarpc", "grpc")
    fn protocol_name(&self) -> &str;
    
    /// Handle universal request in protocol-specific way
    async fn handle_request(&self, request: UniversalRequest, endpoint: &str) -> Result<UniversalResponse, ProtocolError>;
    
    /// Check if protocol can handle specific service type
    fn can_handle_service(&self, service_type: &str) -> bool;
    
    /// Get optimal protocol characteristics
    fn characteristics(&self) -> ProtocolCharacteristics;
}

pub struct ProtocolCharacteristics {
    pub latency: ProtocolLatency,
    pub throughput: ProtocolThroughput,
    pub streaming: bool,
    pub bidirectional: bool,
    pub security: ProtocolSecurity,
}

/// Built-in protocol handlers
pub struct HttpProtocolHandler;
pub struct WebSocketProtocolHandler;
pub struct TarpcProtocolHandler;
pub struct GrpcProtocolHandler;
pub struct McpProtocolHandler;
pub struct CustomProtocolHandler;
```

### Universal Service Registry
```rust
pub struct UniversalServiceRegistry {
    services: Arc<RwLock<HashMap<String, RegisteredService>>>,
    capability_index: Arc<RwLock<CapabilityIndex>>,
    primal_index: Arc<RwLock<PrimalIndex>>,
    health_tracker: Arc<HealthTracker>,
}

#[derive(Debug, Clone)]
pub struct RegisteredService {
    pub registration: UniversalServiceRegistration,
    pub instance_id: String,
    pub health_status: HealthStatus,
    pub metrics: ServiceMetrics,
    pub last_seen: DateTime<Utc>,
    pub capabilities: Vec<ServiceCapability>,
}

impl UniversalServiceRegistry {
    /// Find services by capability requirements
    pub async fn find_by_capability(&self, requirements: &[CapabilityRequirement]) -> Result<Vec<RegisteredService>, RegistryError> {
        let capability_index = self.capability_index.read().await;
        let mut matching_services = Vec::new();
        
        for requirement in requirements {
            if let Some(services) = capability_index.get_services_with_capability(requirement) {
                matching_services.extend(services);
            }
        }
        
        // Filter by health status
        let healthy_services = matching_services.into_iter()
            .filter(|service| service.health_status == HealthStatus::Healthy)
            .collect();
            
        Ok(healthy_services)
    }
    
    /// Find services by primal type
    pub async fn find_by_primal_type(&self, primal_type: PrimalType) -> Result<Vec<RegisteredService>, RegistryError> {
        let primal_index = self.primal_index.read().await;
        Ok(primal_index.get_services_by_primal(primal_type).unwrap_or_default())
    }
    
    /// Register universal service
    pub async fn register(&self, registration: UniversalServiceRegistration) -> Result<String, RegistryError> {
        let service_id = format!("{}-{}-{}", 
            registration.primal_type.as_str(), 
            registration.service.name,
            registration.instance_id
        );
        
        let registered_service = RegisteredService {
            registration,
            instance_id: service_id.clone(),
            health_status: HealthStatus::Healthy,
            metrics: ServiceMetrics::default(),
            last_seen: Utc::now(),
            capabilities: Vec::new(),
        };
        
        // Update indices
        self.update_capability_index(&registered_service).await?;
        self.update_primal_index(&registered_service).await?;
        
        // Store service
        let mut services = self.services.write().await;
        services.insert(service_id.clone(), registered_service);
        
        Ok(service_id)
    }
}
```

### Universal Event Coordination
```rust
pub struct UniversalEventCoordinator {
    event_bus: Arc<UniversalEventBus>,
    coordination_engine: Arc<CoordinationEngine>,
    subscription_manager: Arc<SubscriptionManager>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalEvent {
    pub event_id: Uuid,
    pub event_type: String,
    pub source_service: String,
    pub target_services: Vec<String>,
    pub payload: serde_json::Value,
    pub metadata: HashMap<String, serde_json::Value>,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossPrimalCoordination {
    pub coordination_id: Uuid,
    pub required_capabilities: Vec<CapabilityRequirement>,
    pub workflow_steps: Vec<CoordinationStep>,
    pub timeout: Duration,
    pub retry_policy: RetryPolicy,
}

impl UniversalEventCoordinator {
    /// Execute cross-primal coordination
    pub async fn execute_coordination(
        &self,
        coordination: CrossPrimalCoordination,
        participants: Vec<RegisteredService>,
    ) -> Result<CoordinationResult, CoordinationError> {
        let mut results = Vec::new();
        
        for step in coordination.workflow_steps {
            let step_result = self.execute_coordination_step(step, &participants).await?;
            results.push(step_result);
        }
        
        Ok(CoordinationResult {
            coordination_id: coordination.coordination_id,
            results,
            completed_at: Utc::now(),
        })
    }
    
    /// Publish universal event
    pub async fn publish_event(&self, event: UniversalEvent) -> Result<(), CoordinationError> {
        self.event_bus.publish(event).await
    }
    
    /// Subscribe to events with universal filtering
    pub async fn subscribe(&self, 
        subscription: EventSubscription,
        callback: Box<dyn Fn(UniversalEvent) -> BoxFuture<'_, Result<(), EventError>> + Send + Sync>,
    ) -> Result<SubscriptionHandle, CoordinationError> {
        self.subscription_manager.subscribe(subscription, callback).await
    }
}
```

### Universal Load Balancer
```rust
pub struct UniversalLoadBalancer {
    strategies: HashMap<String, Box<dyn LoadBalancingStrategy>>,
    health_checker: Arc<HealthChecker>,
    capability_matcher: Arc<CapabilityMatcher>,
}

impl UniversalLoadBalancer {
    /// Select optimal service instance based on capabilities and health
    pub async fn select_instance(
        &self,
        services: &[RegisteredService],
        request: &UniversalRequest,
    ) -> Result<RegisteredService, LoadBalancingError> {
        // Filter by capability requirements
        let capable_services = self.capability_matcher.filter_by_requirements(services, &request.required_capabilities)?;
        
        // Filter by health status
        let healthy_services = capable_services.into_iter()
            .filter(|service| service.health_status == HealthStatus::Healthy)
            .collect::<Vec<_>>();
        
        if healthy_services.is_empty() {
            return Err(LoadBalancingError::NoHealthyServices);
        }
        
        // Select strategy based on request preferences
        let strategy_name = request.load_balancing_strategy.as_deref().unwrap_or("capability_based");
        let strategy = self.strategies.get(strategy_name)
            .ok_or(LoadBalancingError::UnknownStrategy)?;
        
        // Select instance using strategy
        strategy.select_instance(&healthy_services, request).await
    }
}

/// Capability-based load balancing strategy
pub struct CapabilityBasedStrategy;

#[async_trait]
impl LoadBalancingStrategy for CapabilityBasedStrategy {
    fn name(&self) -> &str {
        "capability_based"
    }
    
    async fn select_instance(
        &self,
        instances: &[RegisteredService],
        request: &UniversalRequest,
    ) -> Result<RegisteredService, LoadBalancingError> {
        // Score instances based on capability match
        let mut scored_instances = Vec::new();
        
        for instance in instances {
            let score = self.calculate_capability_score(instance, request).await?;
            scored_instances.push((instance.clone(), score));
        }
        
        // Sort by score (highest first)
        scored_instances.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Return best match
        scored_instances.into_iter()
            .next()
            .map(|(instance, _)| instance)
            .ok_or(LoadBalancingError::NoSuitableInstance)
    }
    
    async fn calculate_capability_score(&self, instance: &RegisteredService, request: &UniversalRequest) -> Result<f64, LoadBalancingError> {
        // Calculate score based on capability matching
        let mut score = 0.0;
        
        for requirement in &request.required_capabilities {
            if instance.capabilities.iter().any(|cap| self.capability_matches(cap, requirement)) {
                score += 1.0;
            }
        }
        
        // Add performance bonus
        score += instance.metrics.performance_score;
        
        // Add availability bonus
        if instance.health_status == HealthStatus::Healthy {
            score += 0.5;
        }
        
        Ok(score)
    }
}
```

## Implementation Guidelines

### Universal Service Integration
- All services MUST implement `UniversalServiceProvider` trait
- Services MUST register with universal metadata and capabilities
- Services MUST support health checks and metrics reporting
- Services MUST handle universal request/response formats

### Protocol Agnostic Communication
- Use universal request/response formats for all inter-service communication
- Support protocol negotiation based on service capabilities
- Implement graceful fallback between protocols
- Maintain protocol-specific optimizations where beneficial

### Capability-Based Discovery
- Define clear capability requirements for all operations
- Use capability matching for service discovery
- Support dynamic capability updates
- Implement capability-based load balancing

### Cross-Primal Coordination
- Use universal event system for cross-primal communication
- Implement coordination workflows for complex operations
- Support distributed transactions where needed
- Provide monitoring and observability for all coordination

## Success Metrics

### Universal Integration
- Zero-configuration integration for new primals
- Sub-100ms service discovery across all primal types
- 99.9% success rate for capability-based routing
- Protocol-agnostic communication with automatic fallback

### Performance
- Sub-10ms request routing overhead
- Linear scaling with service count
- Efficient capability matching algorithms
- Minimal protocol translation overhead

### Reliability
- Automatic failover between service instances
- Circuit breaker protection for failing services
- Graceful degradation during partial system failures
- Consistent behavior across all protocol types

This specification establishes universal and agnostic patterns that enable Songbird to orchestrate any service type while maintaining optimal performance and reliability. 