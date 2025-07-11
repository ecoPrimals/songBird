---
description: ENFORCE dynamic service discovery and capability-based service registry
globs: ["songbird/src/**/*.rs", "songbird/crates/**/*.rs"]
---

# Service Registry Specification

## Context
- When implementing dynamic service discovery for multi-protocol orchestration
- When managing service capabilities and metadata
- When coordinating service lifecycle across primals
- When enabling capability-based service selection

## Requirements

### Dynamic Service Discovery
- Real-time service registration and deregistration
- Capability-based service discovery with filtering
- Service health monitoring and status updates
- Automatic service cleanup for dead instances

### Capability Management
- Rich capability metadata for services
- Capability validation and enforcement
- Dynamic capability updates
- Capability-based routing and selection

### Cross-Primal Service Coordination
- Standardized service registration across all primals
- Cross-primal service discovery and communication
- Service dependency management
- Primal-specific service policies

### High Availability and Scalability
- Distributed service registry with consensus
- Service data replication and consistency
- Automatic failover and recovery
- Horizontal scaling support

## Architecture

### Core Service Registry
```rust
pub struct ServiceRegistry {
    service_store: Arc<dyn ServiceStore>,
    capability_manager: Arc<CapabilityManager>,
    health_tracker: Arc<HealthTracker>,
    event_publisher: Arc<EventPublisher>,
    discovery_engine: Arc<DiscoveryEngine>,
    registry_config: RegistryConfig,
}

impl ServiceRegistry {
    pub async fn new(config: RegistryConfig) -> Result<Self>;
    
    // Core service operations
    pub async fn register_service(&self, service: ServiceInfo) -> Result<RegistrationResult>;
    pub async fn deregister_service(&self, service_id: &str) -> Result<()>;
    pub async fn update_service(&self, service_id: &str, update: ServiceUpdate) -> Result<()>;
    pub async fn get_service(&self, service_id: &str) -> Result<Option<ServiceInfo>>;
    
    // Service discovery
    pub async fn discover_services(&self, query: ServiceQuery) -> Result<Vec<ServiceInfo>>;
    pub async fn discover_by_capability(&self, capability: &Capability) -> Result<Vec<ServiceInfo>>;
    pub async fn discover_by_primal(&self, primal_type: PrimalType) -> Result<Vec<ServiceInfo>>;
    
    // Health and status
    pub async fn update_service_health(&self, service_id: &str, health: HealthStatus) -> Result<()>;
    pub async fn get_healthy_services(&self, service_type: &str) -> Result<Vec<ServiceInfo>>;
    pub async fn get_service_metrics(&self, service_id: &str) -> Result<ServiceMetrics>;
    
    // Capabilities
    pub async fn update_capabilities(&self, service_id: &str, capabilities: Vec<Capability>) -> Result<()>;
    pub async fn validate_capabilities(&self, service_id: &str, required: &[Capability]) -> Result<ValidationResult>;
    
    // Lifecycle management
    pub async fn start_service_monitoring(&self, service_id: &str) -> Result<()>;
    pub async fn stop_service_monitoring(&self, service_id: &str) -> Result<()>;
    pub async fn cleanup_dead_services(&self) -> Result<Vec<String>>;
}
```

### Service Information Model
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub id: String,
    pub name: String,
    pub service_type: String,
    pub primal_type: PrimalType,
    pub version: String,
    pub capabilities: Vec<Capability>,
    pub endpoints: Vec<ServiceEndpoint>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub health_status: HealthStatus,
    pub registration_time: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    pub protocol: ProtocolType,
    pub address: String,
    pub port: u16,
    pub path: Option<String>,
    pub security: SecurityConfig,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProtocolType {
    HTTP,
    HTTPS,
    WebSocket,
    WebSocketSecure,
    WebSocketJson,
    WebSocketJsonSecure,
    Tarpc,
    TCP,
    UDP,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrimalType {
    Toadstool,
    Songbird,
    NestGate,
    BearDog,
    Squirrel,
    External,
}
```

### Capability System
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Capability {
    pub name: String,
    pub version: String,
    pub parameters: HashMap<String, CapabilityParameter>,
    pub requirements: Vec<CapabilityRequirement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityParameter {
    pub value: serde_json::Value,
    pub parameter_type: ParameterType,
    pub required: bool,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    String,
    Integer,
    Float,
    Boolean,
    Array(Box<ParameterType>),
    Object,
    Enum(Vec<String>),
}

pub struct CapabilityManager {
    capability_definitions: Arc<RwLock<HashMap<String, CapabilityDefinition>>>,
    capability_validators: Arc<RwLock<HashMap<String, Arc<dyn CapabilityValidator>>>>,
    capability_index: Arc<CapabilityIndex>,
}

#[async_trait]
pub trait CapabilityValidator: Send + Sync {
    async fn validate(&self, capability: &Capability, service: &ServiceInfo) -> Result<ValidationResult>;
    fn capability_name(&self) -> &str;
}

// Built-in capability validators
pub struct HttpCapabilityValidator;
pub struct WebSocketCapabilityValidator;
pub struct WebSocketJsonCapabilityValidator;
pub struct TarpcCapabilityValidator;
pub struct StorageCapabilityValidator;
pub struct AICapabilityValidator;
pub struct SecurityCapabilityValidator;
```

### Service Discovery Engine
```rust
pub struct DiscoveryEngine {
    query_processor: Arc<QueryProcessor>,
    result_ranker: Arc<ResultRanker>,
    cache_manager: Arc<CacheManager>,
    load_balancer: Arc<dyn LoadBalancer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceQuery {
    pub service_type: Option<String>,
    pub primal_type: Option<PrimalType>,
    pub capabilities: Vec<Capability>,
    pub tags: Vec<String>,
    pub health_status: Option<HealthStatus>,
    pub metadata_filters: HashMap<String, serde_json::Value>,
    pub location_preferences: Option<LocationPreferences>,
    pub limit: Option<usize>,
    pub ranking_criteria: Option<RankingCriteria>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationPreferences {
    pub preferred_zones: Vec<String>,
    pub preferred_regions: Vec<String>,
    pub latency_threshold: Option<Duration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingCriteria {
    pub health_weight: f64,
    pub performance_weight: f64,
    pub location_weight: f64,
    pub capacity_weight: f64,
    pub custom_weights: HashMap<String, f64>,
}

impl DiscoveryEngine {
    pub async fn execute_query(&self, query: ServiceQuery) -> Result<DiscoveryResult>;
    pub async fn get_best_service(&self, query: ServiceQuery) -> Result<Option<ServiceInfo>>;
    pub async fn get_all_matching_services(&self, query: ServiceQuery) -> Result<Vec<ServiceInfo>>;
}
```

### Health Tracking System
```rust
pub struct HealthTracker {
    health_checkers: HashMap<String, Arc<dyn HealthChecker>>,
    health_cache: Arc<RwLock<HashMap<String, HealthStatus>>>,
    monitoring_tasks: Arc<RwLock<HashMap<String, JoinHandle<()>>>>,
    health_history: Arc<HealthHistory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: HealthState,
    pub score: f64, // 0.0 to 1.0
    pub last_check: DateTime<Utc>,
    pub response_time: Duration,
    pub error_count: u32,
    pub consecutive_failures: u32,
    pub details: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthState {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

#[async_trait]
pub trait HealthChecker: Send + Sync {
    async fn check_health(&self, service: &ServiceInfo) -> Result<HealthStatus>;
    fn check_interval(&self) -> Duration;
    fn supports_service_type(&self, service_type: &str) -> bool;
}
```

### Service Store Interface
```rust
#[async_trait]
pub trait ServiceStore: Send + Sync {
    async fn store_service(&self, service: &ServiceInfo) -> Result<()>;
    async fn get_service(&self, service_id: &str) -> Result<Option<ServiceInfo>>;
    async fn update_service(&self, service_id: &str, update: &ServiceUpdate) -> Result<()>;
    async fn delete_service(&self, service_id: &str) -> Result<()>;
    async fn list_services(&self, filter: Option<&ServiceFilter>) -> Result<Vec<ServiceInfo>>;
    async fn query_services(&self, query: &ServiceQuery) -> Result<Vec<ServiceInfo>>;
    async fn get_services_by_capability(&self, capability: &Capability) -> Result<Vec<ServiceInfo>>;
}

// Implementations
pub struct InMemoryServiceStore {
    services: Arc<RwLock<HashMap<String, ServiceInfo>>>,
    indexes: Arc<ServiceIndexes>,
}

pub struct DistributedServiceStore {
    local_store: Arc<InMemoryServiceStore>,
    consensus_manager: Arc<ConsensusManager>,
    replication_manager: Arc<ReplicationManager>,
}

pub struct PersistentServiceStore {
    database: Arc<dyn Database>,
    cache: Arc<InMemoryServiceStore>,
    consistency_manager: Arc<ConsistencyManager>,
}
```

## Implementation Tasks

### Phase 1: Core Registry (Week 1-2)
1. **Basic Service Registry**
   - Service information model
   - In-memory service store
   - Basic registration/deregistration
   - Simple service discovery

2. **Capability System Foundation**
   - Capability data structures
   - Basic capability validation
   - Capability-based discovery
   - Built-in capability validators

### Phase 2: Health and Discovery (Week 3-4)
1. **Health Tracking System**
   - Health checker implementations
   - Continuous health monitoring
   - Health status caching
   - Health history tracking

2. **Advanced Discovery Engine**
   - Complex query processing
   - Result ranking and optimization
   - Discovery result caching
   - Performance optimizations

### Phase 3: Distributed Registry (Week 5-6)
1. **Distributed Service Store**
   - Consensus-based storage
   - Data replication
   - Consistency guarantees
   - Conflict resolution

2. **Cross-Primal Coordination**
   - Primal-specific service policies
   - Cross-primal discovery
   - Service dependency management
   - Primal lifecycle coordination

### Phase 4: Integration and Optimization (Week 7-8)
1. **BiomeOS Integration**
   - Team-scoped service isolation
   - Resource quota integration
   - Service templates and policies
   - Deployment automation

2. **Performance and Scalability**
   - Query optimization
   - Index maintenance
   - Memory management
   - Horizontal scaling

## Configuration

### Registry Configuration
```rust
pub struct RegistryConfig {
    pub storage: StorageConfig,
    pub health_monitoring: HealthMonitoringConfig,
    pub discovery: DiscoveryConfig,
    pub security: SecurityConfig,
    pub performance: PerformanceConfig,
}

pub struct StorageConfig {
    pub storage_type: StorageType,
    pub consistency_level: ConsistencyLevel,
    pub replication_factor: u32,
    pub backup_config: Option<BackupConfig>,
}

#[derive(Debug, Clone)]
pub enum StorageType {
    InMemory,
    Distributed {
        cluster_config: ClusterConfig,
    },
    Persistent {
        database_url: String,
        cache_config: CacheConfig,
    },
}

pub struct HealthMonitoringConfig {
    pub default_check_interval: Duration,
    pub health_check_timeout: Duration,
    pub unhealthy_threshold: u32,
    pub recovery_threshold: u32,
    pub cleanup_interval: Duration,
}
```

### Service Registration Templates
```rust
pub struct ServiceTemplate {
    pub name: String,
    pub primal_type: PrimalType,
    pub default_capabilities: Vec<Capability>,
    pub required_metadata: Vec<String>,
    pub validation_rules: Vec<ValidationRule>,
    pub health_check_config: HealthCheckConfig,
}

// Built-in templates for each primal
pub const TOADSTOOL_RUNTIME_TEMPLATE: ServiceTemplate = ServiceTemplate {
    name: "toadstool-runtime",
    primal_type: PrimalType::Toadstool,
    default_capabilities: vec![
        Capability::new("compute", "1.0"),
        Capability::new("containerization", "1.0"),
        Capability::new("resource_management", "1.0"),
    ],
    // ... other fields
};

pub const SQUIRREL_AGENT_TEMPLATE: ServiceTemplate = ServiceTemplate {
    name: "squirrel-agent",
    primal_type: PrimalType::Squirrel,
    default_capabilities: vec![
        Capability::new("mcp", "1.0"),
        Capability::new("ai_inference", "1.0"),
        Capability::new("plugin_execution", "1.0"),
    ],
    // ... other fields
};
```

## Integration Points

### BiomeOS Integration
- Team-scoped service registries
- Service templates and policies
- Automated service deployment
- Resource quota enforcement

### Primal Integration
- **Toadstool**: Runtime service registration and lifecycle management
- **Squirrel**: AI agent service discovery and MCP capability validation
- **NestGate**: Storage service registration and data capability management
- **BearDog**: Security policy enforcement and service authentication
- **Songbird**: Load balancing and service mesh coordination

### External Integration
- Kubernetes service discovery integration
- Consul service mesh integration
- DNS-based service discovery
- Cloud provider service discovery

## Security Considerations

### Service Registration Security
- Service identity verification
- Registration authorization
- Capability validation
- Metadata sanitization

### Discovery Security
- Query authorization
- Result filtering based on permissions
- Service access control
- Audit logging

### Cross-Primal Security
- Mutual authentication
- Service-to-service encryption
- Capability-based access control
- Security policy enforcement

## Performance Requirements

### Latency Targets
- Service registration: < 50ms
- Service discovery: < 100ms (cached), < 500ms (uncached)
- Health check updates: < 10ms
- Capability validation: < 25ms

### Throughput Targets
- Service registrations: 1K/second
- Discovery queries: 10K/second
- Health updates: 100K/second
- Capability validations: 5K/second

### Storage Requirements
- Service count: 100K+ services
- Query performance: < 100ms for complex queries
- Data consistency: Strong consistency for critical operations
- Availability: 99.9% uptime

## Testing Strategy

### Unit Testing
- Service registration and deregistration
- Capability validation logic
- Health checking mechanisms
- Discovery query processing

### Integration Testing
- Cross-primal service coordination
- End-to-end service discovery flows
- Health monitoring accuracy
- Performance under load

### Performance Testing
- Query performance optimization
- Registration throughput
- Memory usage efficiency
- Scalability testing

## Examples

### Service Registration
```rust
let registry = ServiceRegistry::new(config).await?;

let service = ServiceInfo {
    id: "squirrel-agent-1".to_string(),
    name: "Squirrel AI Agent".to_string(),
    service_type: "ai-agent".to_string(),
    primal_type: PrimalType::Squirrel,
    version: "1.0.0".to_string(),
    capabilities: vec![
        Capability {
            name: "mcp".to_string(),
            version: "1.0".to_string(),
            parameters: HashMap::from([
                ("max_plugins".to_string(), CapabilityParameter {
                    value: serde_json::Value::Number(serde_json::Number::from(10)),
                    parameter_type: ParameterType::Integer,
                    required: true,
                    description: Some("Maximum number of plugins".to_string()),
                }),
            ]),
            requirements: vec![],
        },
        Capability {
            name: "ai_inference".to_string(),
            version: "1.0".to_string(),
            parameters: HashMap::from([
                ("model_types".to_string(), CapabilityParameter {
                    value: serde_json::Value::Array(vec![
                        serde_json::Value::String("llm".to_string()),
                        serde_json::Value::String("embedding".to_string()),
                    ]),
                    parameter_type: ParameterType::Array(Box::new(ParameterType::String)),
                    required: true,
                    description: Some("Supported model types".to_string()),
                }),
            ]),
            requirements: vec![],
        },
    ],
    endpoints: vec![
        ServiceEndpoint {
            protocol: ProtocolType::HTTP,
            address: "10.0.1.100".to_string(),
            port: 8080,
            path: Some("/api/v1".to_string()),
            security: SecurityConfig::default(),
            metadata: HashMap::new(),
        },
        ServiceEndpoint {
            protocol: ProtocolType::WebSocket,
            address: "10.0.1.100".to_string(),
            port: 8080,
            path: Some("/ws".to_string()),
            security: SecurityConfig::default(),
            metadata: HashMap::new(),
        },
    ],
    metadata: HashMap::from([
        ("team".to_string(), serde_json::Value::String("ai-platform".to_string())),
        ("environment".to_string(), serde_json::Value::String("production".to_string())),
    ]),
    health_status: HealthStatus {
        status: HealthState::Healthy,
        score: 1.0,
        last_check: Utc::now(),
        response_time: Duration::from_millis(50),
        error_count: 0,
        consecutive_failures: 0,
        details: HashMap::new(),
    },
    registration_time: Utc::now(),
    last_updated: Utc::now(),
    tags: vec!["ai".to_string(), "mcp".to_string()],
};

let result = registry.register_service(service).await?;
```

### Service Discovery
```rust
let query = ServiceQuery {
    service_type: Some("ai-agent".to_string()),
    primal_type: Some(PrimalType::Squirrel),
    capabilities: vec![
        Capability {
            name: "mcp".to_string(),
            version: "1.0".to_string(),
            parameters: HashMap::new(),
            requirements: vec![],
        },
    ],
    tags: vec!["ai".to_string()],
    health_status: Some(HealthStatus {
        status: HealthState::Healthy,
        // ... other fields
    }),
    metadata_filters: HashMap::from([
        ("environment".to_string(), serde_json::Value::String("production".to_string())),
    ]),
    location_preferences: None,
    limit: Some(5),
    ranking_criteria: Some(RankingCriteria {
        health_weight: 0.4,
        performance_weight: 0.3,
        location_weight: 0.2,
        capacity_weight: 0.1,
        custom_weights: HashMap::new(),
    }),
};

let discovered_services = registry.discover_services(query).await?;
for service in discovered_services {
    println!("Found service: {} at {}", service.name, service.endpoints[0].address);
}
```

### Health Monitoring
```rust
// Update service health
let health_status = HealthStatus {
    status: HealthState::Healthy,
    score: 0.95,
    last_check: Utc::now(),
    response_time: Duration::from_millis(75),
    error_count: 0,
    consecutive_failures: 0,
    details: HashMap::from([
        ("cpu_usage".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.65).unwrap())),
        ("memory_usage".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.45).unwrap())),
    ]),
};

registry.update_service_health("squirrel-agent-1", health_status).await?;

// Get healthy services
let healthy_services = registry.get_healthy_services("ai-agent").await?;
println!("Found {} healthy AI agent services", healthy_services.len());
```

## Monitoring and Observability

### Registry Metrics
- Service registration/deregistration rates
- Discovery query performance
- Health check success rates
- Capability validation metrics

### Service Metrics
- Service uptime and availability
- Response time distribution
- Error rates and patterns
- Capacity utilization

### Alerting Rules
- Service registration failures
- Discovery query timeouts
- Health check failures
- Capability validation errors

## Version History

- v1.0.0: Initial specification
- v1.1.0: Added capability system
- v1.2.0: Enhanced health monitoring
- v1.3.0: BiomeOS integration requirements

<version>1.3.0</version> 