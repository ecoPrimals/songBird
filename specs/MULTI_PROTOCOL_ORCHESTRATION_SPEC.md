---
description: ENFORCE universal multi-protocol orchestration with dynamic service discovery and coordination
globs: ["songbird/src/**/*.rs", "songbird/crates/**/*.rs"]
---

# Multi-Protocol Orchestration Specification

## Context
- When implementing songbird's orchestration capabilities
- When coordinating between primals in the ecosystem
- When managing dynamic service discovery and load balancing
- When implementing real-time coordination protocols

## Requirements

### Hybrid Protocol Architecture
- **External Layer**: WebSocket JSON for web UIs, HTTP/REST for APIs, SSE for real-time updates
- **Internal Layer**: tarpc for high-performance service-to-service communication
- **Streaming Layer**: MCP protocol extensions for AI agent coordination
- **Event Layer**: Reactive event system for cross-primal coordination
- Provide protocol translation bridges between layers
- Enable dynamic protocol selection based on use case

### Dynamic Service Discovery
- Implement self-organizing service registry
- Support primal capability advertisement
- Enable automatic load balancing
- Provide health monitoring and failover

### Real-Time Coordination
- Implement bidirectional event streaming
- Support subscription-based event delivery
- Enable cross-primal coordination workflows
- Provide real-time status monitoring

## Architecture

### Core Orchestration Engine
```rust
pub struct UniversalOrchestrator {
    protocol_router: Arc<ProtocolRouter>,
    service_registry: Arc<ServiceRegistry>,
    event_coordinator: Arc<EventCoordinator>,
    load_balancer: Arc<LoadBalancer>,
    health_monitor: Arc<HealthMonitor>,
}

impl UniversalOrchestrator {
    pub async fn new(config: OrchestratorConfig) -> Result<Self>;
    pub async fn start(&self) -> Result<()>;
    pub async fn register_primal(&self, primal: PrimalInfo) -> Result<()>;
    pub async fn coordinate_request(&self, request: CoordinationRequest) -> Result<CoordinationResponse>;
    pub async fn broadcast_event(&self, event: SystemEvent) -> Result<()>;
}
```

### Protocol Router
```rust
pub struct ProtocolRouter {
    handlers: HashMap<ProtocolType, Box<dyn ProtocolHandler>>,
    translator: Arc<ProtocolTranslator>,
}

#[async_trait]
pub trait ProtocolHandler {
    async fn handle_request(&self, request: UniversalRequest) -> Result<UniversalResponse>;
    async fn stream_events(&self, subscription: EventSubscription) -> Result<EventStream>;
    fn supported_features(&self) -> Vec<ProtocolFeature>;
}
```

### Service Registry
```rust
pub struct ServiceRegistry {
    services: Arc<RwLock<HashMap<String, ServiceInfo>>>,
    capabilities: Arc<RwLock<HashMap<String, Vec<Capability>>>>,
    health_tracker: Arc<HealthTracker>,
}

impl ServiceRegistry {
    pub async fn register_service(&self, service: ServiceInfo) -> Result<()>;
    pub async fn discover_services(&self, query: ServiceQuery) -> Result<Vec<ServiceInfo>>;
    pub async fn update_capabilities(&self, service_id: &str, capabilities: Vec<Capability>) -> Result<()>;
    pub async fn get_healthy_services(&self, service_type: &str) -> Result<Vec<ServiceInfo>>;
}
```

### Event Coordination
```rust
pub struct EventCoordinator {
    event_bus: Arc<EventBus>,
    subscriptions: Arc<RwLock<HashMap<String, Vec<Subscription>>>>,
    event_history: Arc<EventHistory>,
}

#[derive(Debug, Clone)]
pub enum SystemEvent {
    PrimalStatusChanged { primal_id: String, status: PrimalStatus },
    ServiceRegistered { service: ServiceInfo },
    LoadBalancingUpdate { target: String, metrics: LoadMetrics },
    CoordinationRequest { request: CoordinationRequest },
    HealthCheck { service_id: String, status: HealthStatus },
}
```

## Implementation Tasks

### Phase 1: Core Infrastructure
1. **Universal Protocol Framework**
   - Implement protocol abstraction layer
   - Create unified request/response types
   - Build protocol translation matrix
   - Enable dynamic protocol switching

2. **Service Discovery System**
   - Create self-organizing service registry
   - Implement capability advertisement
   - Build service health monitoring
   - Enable automatic failover

### Phase 2: Advanced Coordination
1. **Real-Time Event System**
   - Implement bidirectional event streaming
   - Create subscription management
   - Build event history and replay
   - Enable cross-primal coordination

2. **Load Balancing Engine**
   - Implement dynamic load balancing
   - Create performance metrics collection
   - Build adaptive routing algorithms
   - Enable predictive scaling

### Phase 3: Ecosystem Integration
1. **Primal Coordination**
   - Implement primal lifecycle management
   - Create coordination workflows
   - Build dependency resolution
   - Enable distributed consensus

2. **Federation Support**
   - Implement federation protocols
   - Create cross-federation discovery
   - Build federation health monitoring
   - Enable federated coordination

## Configuration

### Universal Configuration
```rust
pub struct OrchestratorConfig {
    pub service_discovery: ServiceDiscoveryConfig,
    pub protocols: HashMap<ProtocolType, ProtocolConfig>,
    pub event_system: EventSystemConfig,
    pub load_balancing: LoadBalancingConfig,
    pub health_monitoring: HealthMonitoringConfig,
}

pub struct ServiceDiscoveryConfig {
    pub discovery_interval: Duration,
    pub health_check_interval: Duration,
    pub service_timeout: Duration,
    pub max_retries: usize,
}
```

### Protocol Configuration
```rust
pub struct ProtocolConfig {
    pub enabled: bool,
    pub bind_interface: Option<String>,
    pub port_range: Option<(u16, u16)>,
    pub max_connections: usize,
    pub timeout: Duration,
    pub features: Vec<ProtocolFeature>,
}
```

## Integration Points

### Primal Integration
- **Squirrel**: AI coordination and MCP protocol bridging
- **NestGate**: Storage orchestration and data flow coordination
- **BearDog**: Security policy enforcement and encrypted coordination
- **ToadStool**: Compute resource orchestration and execution coordination
- **BiomeOS**: UI coordination and federation management

### Event Integration
- Subscribe to all primal lifecycle events
- Coordinate cross-primal operations
- Manage distributed state consistency
- Handle federation events

## Performance Requirements

### Latency Targets
- Service discovery: < 100ms
- Event delivery: < 10ms
- Protocol translation: < 5ms
- Load balancing decisions: < 50ms

### Throughput Targets
- Event processing: 100K events/second
- Service queries: 10K queries/second
- Protocol translations: 50K translations/second
- Coordination requests: 5K requests/second

## Testing Strategy

### Unit Testing
- Protocol handler implementations
- Service registry operations
- Event coordination logic
- Load balancing algorithms

### Integration Testing
- Cross-primal coordination workflows
- Protocol translation accuracy
- Service discovery reliability
- Event delivery consistency

### Performance Testing
- Latency under load
- Throughput benchmarks
- Memory usage optimization
- Connection scaling

## Examples

### Service Registration
```rust
let orchestrator = UniversalOrchestrator::new(config).await?;

let service = ServiceInfo {
    id: "squirrel-mcp".to_string(),
    service_type: "ai-platform".to_string(),
    capabilities: vec![
        Capability::MCP,
        Capability::AIInference,
        Capability::PluginExecution,
    ],
    protocols: vec![
        ProtocolEndpoint::HTTP { path: "/api/v1" },
        ProtocolEndpoint::WebSocket { path: "/ws" },
        ProtocolEndpoint::GRPC { service: "ai.v1.AIService" },
    ],
    metadata: HashMap::new(),
};

orchestrator.register_primal(service).await?;
```

### Event Coordination
```rust
let event = SystemEvent::PrimalStatusChanged {
    primal_id: "nestgate".to_string(),
    status: PrimalStatus::Ready,
};

orchestrator.broadcast_event(event).await?;
```

### Cross-Primal Coordination
```rust
let request = CoordinationRequest {
    target: "nestgate".to_string(),
    operation: "allocate_storage".to_string(),
    parameters: json!({
        "size": "1GB",
        "type": "high_performance"
    }),
};

let response = orchestrator.coordinate_request(request).await?;
```

## Security Considerations

### Authentication
- Integrate with BearDog for all authentication
- Use cryptographic primal identification
- Implement request signing and verification
- Support mutual authentication

### Authorization
- Implement capability-based access control
- Use fine-grained permission system
- Support role-based coordination
- Enable audit logging

### Communication Security
- All inter-primal communication encrypted
- Use secure protocol variants by default
- Implement certificate-based authentication
- Support zero-trust networking

## Best Practices

1. **Protocol Agnostic Design**
   - Use universal request/response types
   - Implement protocol-specific adapters
   - Enable runtime protocol switching
   - Support protocol capability negotiation

2. **Self-Organizing Architecture**
   - Implement automatic service discovery
   - Use adaptive load balancing
   - Enable automatic failover
   - Support dynamic reconfiguration

3. **Event-Driven Coordination**
   - Use event-based primal coordination
   - Implement asynchronous processing
   - Enable event replay and recovery
   - Support distributed event consistency

4. **Performance Optimization**
   - Use connection pooling
   - Implement request batching
   - Enable adaptive caching
   - Support predictive scaling

## Version History

- v1.0.0: Initial specification
- v1.1.0: Added federation support
- v1.2.0: Enhanced security integration
- v1.3.0: Performance optimization requirements

<version>1.3.0</version> 