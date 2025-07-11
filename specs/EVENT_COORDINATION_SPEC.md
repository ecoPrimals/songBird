---
description: ENFORCE real-time event coordination and cross-primal communication
globs: ["songbird/src/**/*.rs", "songbird/crates/**/*.rs"]
---

# Event Coordination System Specification

## Context
- When implementing real-time cross-primal coordination
- When managing distributed event streaming and subscription
- When coordinating system-wide state changes
- When implementing event-driven orchestration workflows

## Requirements

### Real-Time Event Bus
- Implement high-performance event streaming with <10ms delivery
- Support multiple event transport protocols (WebSocket, SSE, gRPC streams)
- Enable event persistence and replay for reliability
- Provide event ordering guarantees within topics

### Cross-Primal Coordination
- Implement standardized event schemas for primal coordination
- Support event-driven workflows across multiple primals
- Enable distributed consensus for critical operations
- Provide conflict resolution for concurrent operations

### Subscription Management
- Support dynamic subscription patterns and filters
- Implement subscription lifecycle management
- Enable real-time subscription updates
- Provide backpressure handling for slow consumers

## Architecture

### Core Event Coordinator
```rust
pub struct EventCoordinator {
    event_bus: Arc<dyn EventBus>,
    subscription_manager: Arc<SubscriptionManager>,
    event_store: Arc<dyn EventStore>,
    routing_engine: Arc<EventRoutingEngine>,
    metrics_collector: Arc<EventMetrics>,
}

impl EventCoordinator {
    pub async fn new(config: EventConfig) -> Result<Self>;
    
    // Core event operations
    pub async fn publish_event(&self, event: SystemEvent) -> Result<EventId>;
    pub async fn subscribe(&self, pattern: EventPattern) -> Result<EventStream>;
    pub async fn unsubscribe(&self, subscription_id: SubscriptionId) -> Result<()>;
    
    // Event history and replay
    pub async fn replay_events(&self, from: DateTime<Utc>, to: Option<DateTime<Utc>>) -> Result<EventStream>;
    pub async fn get_event_history(&self, pattern: EventPattern, limit: usize) -> Result<Vec<SystemEvent>>;
    
    // Cross-primal coordination
    pub async fn coordinate_operation(&self, operation: CoordinationRequest) -> Result<CoordinationResponse>;
    pub async fn broadcast_system_event(&self, event: SystemEvent) -> Result<()>;
}
```

### Event Bus Implementation
```rust
#[async_trait]
pub trait EventBus: Send + Sync {
    async fn publish(&self, topic: &str, event: &SystemEvent) -> Result<EventId>;
    async fn subscribe(&self, pattern: &EventPattern) -> Result<EventStream>;
    async fn get_topic_stats(&self, topic: &str) -> Result<TopicStats>;
}

pub struct InMemoryEventBus {
    topics: Arc<RwLock<HashMap<String, TopicData>>>,
    subscribers: Arc<RwLock<HashMap<SubscriptionId, SubscriptionData>>>,
    event_ordering: Arc<EventOrdering>,
}

pub struct DistributedEventBus {
    local_bus: Arc<InMemoryEventBus>,
    federation_client: Arc<FederationClient>,
    consensus_engine: Arc<ConsensusEngine>,
}
```

### System Events Schema
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemEvent {
    // Service lifecycle events
    ServiceRegistered { service: ServiceInfo, timestamp: DateTime<Utc> },
    ServiceUnregistered { service_id: String, timestamp: DateTime<Utc> },
    ServiceHealthChanged { service_id: String, health: HealthStatus, timestamp: DateTime<Utc> },
    
    // Primal coordination events
    PrimalStatusChanged { primal_id: String, status: PrimalStatus, timestamp: DateTime<Utc> },
    PrimalCoordinationRequest { request: CoordinationRequest, timestamp: DateTime<Utc> },
    PrimalCoordinationResponse { response: CoordinationResponse, timestamp: DateTime<Utc> },
    
    // Load balancing events
    LoadBalancingUpdate { target: String, metrics: LoadMetrics, timestamp: DateTime<Utc> },
    ScalingDecision { decision: ScalingDecision, timestamp: DateTime<Utc> },
    
    // Security events (integration with BearDog)
    SecurityPolicyUpdate { policy: SecurityPolicy, timestamp: DateTime<Utc> },
    SecurityThreatDetected { threat: ThreatInfo, timestamp: DateTime<Utc> },
    
    // Federation events
    FederationNodeJoined { node: FederationNode, timestamp: DateTime<Utc> },
    FederationNodeLeft { node_id: String, reason: String, timestamp: DateTime<Utc> },
    
    // BiomeOS integration events
    BiomeDeploymentStarted { deployment: BiomeDeployment, timestamp: DateTime<Utc> },
    BiomeDeploymentCompleted { deployment_id: String, status: DeploymentStatus, timestamp: DateTime<Utc> },
    
    // Custom events for extensibility
    CustomEvent { event_type: String, payload: serde_json::Value, timestamp: DateTime<Utc> },
}
```

### Event Patterns and Filtering
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventPattern {
    pub event_types: Option<Vec<String>>,
    pub service_filter: Option<ServiceFilter>,
    pub primal_filter: Option<Vec<String>>,
    pub severity_filter: Option<Vec<EventSeverity>>,
    pub time_range: Option<TimeRange>,
}

pub struct EventFilter {
    pattern: EventPattern,
    compiled_filter: CompiledFilter,
}

impl EventFilter {
    pub fn matches(&self, event: &SystemEvent) -> bool;
    pub fn compile(pattern: EventPattern) -> Result<Self>;
}
```

### Subscription Management
```rust
pub struct SubscriptionManager {
    subscriptions: Arc<RwLock<HashMap<SubscriptionId, ActiveSubscription>>>,
    pattern_index: Arc<PatternIndex>,
    delivery_manager: Arc<DeliveryManager>,
}

pub struct ActiveSubscription {
    pub id: SubscriptionId,
    pub pattern: EventPattern,
    pub subscriber: SubscriberInfo,
    pub delivery_config: DeliveryConfig,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub metrics: SubscriptionMetrics,
}

#[derive(Debug, Clone)]
pub struct DeliveryConfig {
    pub delivery_mode: DeliveryMode,
    pub batch_size: Option<usize>,
    pub batch_timeout: Option<Duration>,
    pub retry_policy: RetryPolicy,
    pub dead_letter_queue: Option<String>,
}

#[derive(Debug, Clone)]
pub enum DeliveryMode {
    AtLeastOnce,
    AtMostOnce,
    ExactlyOnce,
    BestEffort,
}
```

## Implementation Tasks

### Phase 1: Core Event Infrastructure (Week 1-2)
1. **Event Bus Implementation**
   - In-memory event bus with topic partitioning
   - Event serialization and deserialization
   - Basic publish/subscribe functionality
   - Event ordering and delivery guarantees

2. **System Event Schema**
   - Define comprehensive event types
   - Implement event validation and versioning
   - Create event builder patterns
   - Add event metadata and tracing

### Phase 2: Subscription and Filtering (Week 3-4)
1. **Subscription Management**
   - Dynamic subscription registration
   - Pattern-based event filtering
   - Subscription lifecycle management
   - Performance optimization for large subscriber counts

2. **Event Delivery**
   - Multiple delivery modes (at-least-once, exactly-once)
   - Retry mechanisms and dead letter queues
   - Backpressure handling
   - Delivery metrics and monitoring

### Phase 3: Advanced Coordination (Week 5-6)
1. **Cross-Primal Coordination**
   - Distributed event coordination
   - Consensus for critical operations
   - Conflict resolution strategies
   - Event-driven workflow orchestration

2. **Event Persistence and Replay**
   - Event store implementation
   - Event history queries
   - Point-in-time replay
   - Event compaction and archival

### Phase 4: Integration and Optimization (Week 7-8)
1. **Protocol Integration**
   - WebSocket event streaming
   - Server-Sent Events (SSE) support
   - gRPC streaming integration
   - HTTP webhook delivery

2. **Performance Optimization**
   - Event batching and compression
   - Connection pooling for delivery
   - Memory-efficient event storage
   - Horizontal scaling support

## Configuration

### Event System Configuration
```rust
pub struct EventConfig {
    pub bus_type: EventBusType,
    pub delivery_config: GlobalDeliveryConfig,
    pub storage_config: EventStorageConfig,
    pub routing_config: EventRoutingConfig,
    pub metrics_config: EventMetricsConfig,
}

#[derive(Debug, Clone)]
pub enum EventBusType {
    InMemory {
        max_events_per_topic: usize,
        event_ttl: Duration,
    },
    Distributed {
        cluster_config: ClusterConfig,
        replication_factor: usize,
    },
    Hybrid {
        local_config: Box<EventBusType>,
        federation_config: FederationConfig,
    },
}

pub struct EventStorageConfig {
    pub storage_backend: StorageBackend,
    pub retention_policy: RetentionPolicy,
    pub compression: CompressionConfig,
    pub indexing: IndexingConfig,
}
```

### Event Routing Configuration
```rust
pub struct EventRoutingConfig {
    pub routing_strategy: RoutingStrategy,
    pub load_balancing: LoadBalancingConfig,
    pub failover: FailoverConfig,
    pub circuit_breaker: CircuitBreakerConfig,
}

#[derive(Debug, Clone)]
pub enum RoutingStrategy {
    RoundRobin,
    WeightedRoundRobin(HashMap<String, f64>),
    LatencyBased,
    Custom(String),
}
```

## Integration Points

### BiomeOS Integration
- Event-driven biome lifecycle management
- Cross-biome event coordination
- Team-scoped event isolation
- Resource usage event tracking

### Primal Integration
- **Squirrel**: AI-driven event analysis and prediction
- **NestGate**: Storage event coordination and data flow
- **BearDog**: Security event correlation and threat detection
- **ToadStool**: Compute resource event coordination

### External Integration
- Prometheus metrics for event system monitoring
- Grafana dashboards for event visualization
- Webhook integration for external systems
- OIDC integration for event access control

## Performance Requirements

### Latency Targets
- Event publishing: < 1ms (in-memory), < 5ms (distributed)
- Event delivery: < 10ms (local), < 50ms (cross-primal)
- Subscription updates: < 100ms
- Event query response: < 200ms

### Throughput Targets
- Events per second: 100K (single node), 1M (distributed)
- Concurrent subscriptions: 10K (single node), 100K (distributed)
- Event storage: 1TB+ with efficient querying
- Cross-primal coordination: 1K operations/second

### Reliability Targets
- Event delivery success rate: 99.9%
- System availability: 99.95%
- Data consistency: Strong consistency for critical events
- Recovery time: < 30 seconds for failover

## Security Considerations

### Event Access Control
- Role-based access to event topics
- Subscription authorization policies
- Event content encryption for sensitive data
- Audit logging for all event operations

### Cross-Primal Security
- Mutual authentication for primal communication
- Event signature verification
- Encrypted event transmission
- Rate limiting to prevent event flooding

## Testing Strategy

### Unit Testing
- Event serialization/deserialization
- Subscription pattern matching
- Event delivery mechanisms
- Configuration validation

### Integration Testing
- Cross-primal event coordination
- Event persistence and replay
- Subscription lifecycle management
- Performance under load

### Performance Testing
- Event throughput benchmarks
- Subscription scalability tests
- Memory usage optimization
- Network bandwidth utilization

## Examples

### Basic Event Publishing and Subscription
```rust
let coordinator = EventCoordinator::new(config).await?;

// Publish an event
let event = SystemEvent::ServiceRegistered {
    service: service_info,
    timestamp: Utc::now(),
};
let event_id = coordinator.publish_event(event).await?;

// Subscribe to service events
let pattern = EventPattern {
    event_types: Some(vec!["ServiceRegistered".to_string(), "ServiceUnregistered".to_string()]),
    service_filter: None,
    primal_filter: None,
    severity_filter: None,
    time_range: None,
};

let mut event_stream = coordinator.subscribe(pattern).await?;
while let Some(event) = event_stream.next().await {
    println!("Received event: {:?}", event);
}
```

### Cross-Primal Coordination
```rust
// Coordinate a complex operation across multiple primals
let operation = CoordinationRequest {
    operation_id: Uuid::new_v4(),
    operation_type: "deploy_biome".to_string(),
    participants: vec!["toadstool".to_string(), "nestgate".to_string(), "beardog".to_string()],
    payload: serde_json::json!({
        "biome_id": "user-workspace-123",
        "resources": {
            "cpu": "4 cores",
            "memory": "8Gi",
            "storage": "100Gi"
        }
    }),
    timeout: Duration::from_secs(300),
};

let response = coordinator.coordinate_operation(operation).await?;
match response.status {
    CoordinationStatus::Success => println!("Biome deployed successfully"),
    CoordinationStatus::Failure(error) => println!("Deployment failed: {}", error),
    CoordinationStatus::Timeout => println!("Deployment timed out"),
}
```

### Event History and Replay
```rust
// Replay events from the last hour
let start_time = Utc::now() - Duration::from_secs(3600);
let pattern = EventPattern {
    event_types: Some(vec!["ServiceHealthChanged".to_string()]),
    ..Default::default()
};

let events = coordinator.replay_events(start_time, None).await?;
for event in events {
    // Process historical events
    handle_health_event(event).await?;
}
```

## Monitoring and Observability

### Event Metrics
- Event publishing rate and latency
- Subscription count and activity
- Event delivery success/failure rates
- Cross-primal coordination metrics

### Health Indicators
- Event bus health and connectivity
- Subscription manager performance
- Event store utilization
- Network communication status

### Alerting Rules
- High event publishing latency
- Failed event deliveries
- Subscription manager overload
- Cross-primal coordination failures

## Version History

- v1.0.0: Initial specification
- v1.1.0: Added cross-primal coordination
- v1.2.0: Enhanced security integration
- v1.3.0: BiomeOS integration requirements

<version>1.3.0</version> 