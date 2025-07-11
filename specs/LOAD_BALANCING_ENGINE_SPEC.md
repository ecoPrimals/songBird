---
description: ENFORCE advanced load balancing with health-aware and resource-aware algorithms
globs: ["songbird/src/**/*.rs", "songbird/crates/**/*.rs"]
---

# Load Balancing Engine Specification

## Context
- When implementing dynamic load balancing for service orchestration
- When distributing traffic across multiple service instances
- When optimizing resource utilization and performance
- When ensuring high availability and fault tolerance

## Requirements

### Advanced Load Balancing Algorithms
- Implement multiple load balancing strategies beyond round-robin
- Support health-aware routing with real-time health checks
- Enable resource-aware load balancing (CPU, memory, GPU utilization)
- Provide latency-optimized routing with performance metrics

### Dynamic Service Discovery Integration
- Real-time service instance registration and deregistration
- Automatic health monitoring and instance management
- Integration with service registry for capability-based routing
- Support for weighted routing based on instance capabilities

### Performance Metrics Collection
- Real-time performance metrics gathering from service instances
- Historical performance data analysis and trend detection
- Predictive load balancing based on usage patterns
- Integration with monitoring systems (Prometheus, custom metrics)

### Circuit Breaker and Fault Tolerance
- Automatic circuit breaker activation for failing instances
- Graceful degradation and failover strategies
- Request retry logic with exponential backoff
- Dead letter queue for failed requests

## Architecture

### Core Load Balancer
```rust
pub struct LoadBalancer {
    algorithm_manager: Arc<AlgorithmManager>,
    health_monitor: Arc<HealthMonitor>,
    metrics_collector: Arc<MetricsCollector>,
    circuit_breaker: Arc<CircuitBreakerManager>,
    service_registry: Arc<dyn ServiceRegistry>,
    config: LoadBalancerConfig,
}

impl LoadBalancer {
    pub async fn new(config: LoadBalancerConfig) -> Result<Self>;
    
    // Core load balancing operations
    pub async fn select_instance(&self, service_type: &str, request: &ServiceRequest) -> Result<ServiceInstance>;
    pub async fn select_instances(&self, service_type: &str, count: usize) -> Result<Vec<ServiceInstance>>;
    
    // Instance management
    pub async fn register_instance(&self, instance: ServiceInstance) -> Result<()>;
    pub async fn deregister_instance(&self, instance_id: &str) -> Result<()>;
    pub async fn update_instance_health(&self, instance_id: &str, health: HealthStatus) -> Result<()>;
    
    // Performance monitoring
    pub async fn record_request_metrics(&self, instance_id: &str, metrics: RequestMetrics) -> Result<()>;
    pub async fn get_instance_metrics(&self, instance_id: &str) -> Result<InstanceMetrics>;
    pub async fn get_load_balancer_metrics(&self) -> Result<LoadBalancerMetrics>;
}
```

### Load Balancing Algorithms
```rust
#[async_trait]
pub trait LoadBalancingAlgorithm: Send + Sync {
    async fn select_instance(&self, instances: &[ServiceInstance], request: &ServiceRequest) -> Result<ServiceInstance>;
    async fn select_instances(&self, instances: &[ServiceInstance], count: usize) -> Result<Vec<ServiceInstance>>;
    fn algorithm_type(&self) -> AlgorithmType;
    fn supports_weights(&self) -> bool;
}

#[derive(Debug, Clone)]
pub enum AlgorithmType {
    RoundRobin,
    WeightedRoundRobin,
    LeastConnections,
    HealthBased,
    LatencyOptimized,
    ResourceAware,
    GpuAware,
    Custom(String),
}

pub struct AlgorithmManager {
    algorithms: HashMap<AlgorithmType, Arc<dyn LoadBalancingAlgorithm>>,
    default_algorithm: AlgorithmType,
    algorithm_selection_rules: Vec<AlgorithmSelectionRule>,
}
```

### Specific Algorithm Implementations
```rust
// Round Robin Algorithm
pub struct RoundRobinAlgorithm {
    counter: Arc<AtomicUsize>,
}

// Weighted Round Robin Algorithm
pub struct WeightedRoundRobinAlgorithm {
    weights: Arc<RwLock<HashMap<String, f64>>>,
    counters: Arc<RwLock<HashMap<String, usize>>>,
}

// Least Connections Algorithm
pub struct LeastConnectionsAlgorithm {
    connection_tracker: Arc<ConnectionTracker>,
}

// Health-Based Algorithm
pub struct HealthBasedAlgorithm {
    health_monitor: Arc<HealthMonitor>,
    health_threshold: f64,
    fallback_algorithm: Arc<dyn LoadBalancingAlgorithm>,
}

// Latency-Optimized Algorithm
pub struct LatencyOptimizedAlgorithm {
    latency_tracker: Arc<LatencyTracker>,
    window_size: Duration,
    weight_factor: f64,
}

// Resource-Aware Algorithm
pub struct ResourceAwareAlgorithm {
    resource_monitor: Arc<ResourceMonitor>,
    cpu_weight: f64,
    memory_weight: f64,
    network_weight: f64,
}

// GPU-Aware Algorithm (for AI/ML workloads)
pub struct GpuAwareAlgorithm {
    gpu_monitor: Arc<GpuMonitor>,
    memory_threshold: f64,
    utilization_threshold: f64,
}
```

### Health Monitoring System
```rust
pub struct HealthMonitor {
    health_checkers: HashMap<String, Arc<HealthChecker>>,
    health_cache: Arc<RwLock<HashMap<String, HealthStatus>>>,
    monitoring_tasks: Arc<RwLock<HashMap<String, JoinHandle<()>>>>,
    config: HealthMonitorConfig,
}

#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub healthy: bool,
    pub score: f64, // 0.0 to 1.0
    pub last_check: DateTime<Utc>,
    pub response_time: Duration,
    pub error_rate: f64,
    pub details: HashMap<String, serde_json::Value>,
}

#[async_trait]
pub trait HealthChecker: Send + Sync {
    async fn check_health(&self, instance: &ServiceInstance) -> Result<HealthStatus>;
    fn check_interval(&self) -> Duration;
    fn timeout(&self) -> Duration;
}

// Built-in health checkers
pub struct HttpHealthChecker {
    client: reqwest::Client,
    path: String,
    expected_status: u16,
    timeout: Duration,
}

pub struct TcpHealthChecker {
    timeout: Duration,
}

pub struct CustomHealthChecker {
    check_function: Arc<dyn Fn(&ServiceInstance) -> BoxFuture<'static, Result<HealthStatus>> + Send + Sync>,
}
```

### Metrics Collection System
```rust
pub struct MetricsCollector {
    metrics_store: Arc<dyn MetricsStore>,
    aggregators: HashMap<MetricType, Arc<dyn MetricAggregator>>,
    collection_interval: Duration,
}

#[derive(Debug, Clone)]
pub struct RequestMetrics {
    pub instance_id: String,
    pub request_id: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub response_time: Duration,
    pub status_code: u16,
    pub request_size: usize,
    pub response_size: usize,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct InstanceMetrics {
    pub instance_id: String,
    pub timestamp: DateTime<Utc>,
    pub active_connections: u32,
    pub request_rate: f64, // requests per second
    pub average_response_time: Duration,
    pub error_rate: f64, // percentage
    pub cpu_utilization: f64, // percentage
    pub memory_utilization: f64, // percentage
    pub network_io: NetworkIOMetrics,
    pub custom_metrics: HashMap<String, f64>,
}

#[derive(Debug, Clone)]
pub struct NetworkIOMetrics {
    pub bytes_in_per_sec: f64,
    pub bytes_out_per_sec: f64,
    pub packets_in_per_sec: f64,
    pub packets_out_per_sec: f64,
}
```

### Circuit Breaker System
```rust
pub struct CircuitBreakerManager {
    circuit_breakers: Arc<RwLock<HashMap<String, Arc<CircuitBreaker>>>>,
    default_config: CircuitBreakerConfig,
}

pub struct CircuitBreaker {
    state: Arc<RwLock<CircuitBreakerState>>,
    config: CircuitBreakerConfig,
    metrics: Arc<CircuitBreakerMetrics>,
}

#[derive(Debug, Clone)]
pub enum CircuitBreakerState {
    Closed,
    Open { opened_at: DateTime<Utc> },
    HalfOpen { test_requests: u32 },
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub recovery_timeout: Duration,
    pub test_request_count: u32,
    pub success_threshold: u32,
    pub request_volume_threshold: u32,
}

impl CircuitBreaker {
    pub async fn call<F, T>(&self, instance_id: &str, operation: F) -> Result<T>
    where
        F: Future<Output = Result<T>> + Send,
        T: Send;
    
    pub async fn record_success(&self);
    pub async fn record_failure(&self);
    pub fn get_state(&self) -> CircuitBreakerState;
}
```

## Implementation Tasks

### Phase 1: Core Infrastructure (Week 1-2)
1. **Basic Load Balancer Framework**
   - Core load balancer structure
   - Algorithm trait and manager
   - Service instance management
   - Configuration system

2. **Simple Algorithms Implementation**
   - Round Robin algorithm
   - Weighted Round Robin algorithm
   - Basic health checking integration
   - Request routing logic

### Phase 2: Health Monitoring (Week 3-4)
1. **Health Monitoring System**
   - Health checker trait and implementations
   - Continuous health monitoring
   - Health status caching and updates
   - Integration with load balancing decisions

2. **Metrics Collection**
   - Request metrics collection
   - Instance performance tracking
   - Aggregation and storage
   - Historical data analysis

### Phase 3: Advanced Algorithms (Week 5-6)
1. **Performance-Based Algorithms**
   - Least connections algorithm
   - Latency-optimized routing
   - Resource-aware load balancing
   - GPU-aware routing for AI workloads

2. **Predictive Load Balancing**
   - Traffic pattern analysis
   - Predictive instance selection
   - Load forecasting
   - Adaptive algorithm selection

### Phase 4: Fault Tolerance (Week 7-8)
1. **Circuit Breaker Implementation**
   - Circuit breaker pattern
   - Automatic failover
   - Recovery mechanisms
   - Integration with load balancing

2. **Advanced Fault Tolerance**
   - Request retry with backoff
   - Dead letter queues
   - Graceful degradation
   - Cascading failure prevention

## Configuration

### Load Balancer Configuration
```rust
pub struct LoadBalancerConfig {
    pub default_algorithm: AlgorithmType,
    pub algorithm_configs: HashMap<AlgorithmType, AlgorithmConfig>,
    pub health_monitoring: HealthMonitorConfig,
    pub metrics_collection: MetricsConfig,
    pub circuit_breaker: CircuitBreakerConfig,
    pub service_registry: ServiceRegistryConfig,
}

pub struct AlgorithmConfig {
    pub enabled: bool,
    pub weights: Option<HashMap<String, f64>>,
    pub thresholds: Option<HashMap<String, f64>>,
    pub fallback_algorithm: Option<AlgorithmType>,
}

pub struct HealthMonitorConfig {
    pub default_check_interval: Duration,
    pub default_timeout: Duration,
    pub unhealthy_threshold: u32,
    pub healthy_threshold: u32,
    pub health_check_types: Vec<HealthCheckType>,
}
```

### Algorithm Selection Rules
```rust
pub struct AlgorithmSelectionRule {
    pub condition: SelectionCondition,
    pub algorithm: AlgorithmType,
    pub priority: u32,
}

#[derive(Debug, Clone)]
pub enum SelectionCondition {
    ServiceType(String),
    RequestPath(String),
    RequestHeaders(HashMap<String, String>),
    TimeOfDay(TimeRange),
    LoadLevel(LoadRange),
    Custom(String),
}
```

## Integration Points

### Service Registry Integration
- Automatic service discovery and registration
- Capability-based service selection
- Real-time service updates
- Health status synchronization

### Monitoring Integration
- Prometheus metrics export
- Grafana dashboard support
- Custom metrics collection
- Alerting integration

### BiomeOS Integration
- Team-scoped load balancing
- Resource quota enforcement
- Cross-team traffic isolation
- Performance analytics per team

### Primal Integration
- **Squirrel**: AI-powered load prediction and optimization
- **NestGate**: Storage-aware routing for data-heavy workloads
- **BearDog**: Security-aware routing and policy enforcement
- **ToadStool**: Compute resource optimization and GPU scheduling

## Performance Requirements

### Latency Targets
- Instance selection: < 1ms (cached), < 5ms (with health check)
- Health check response: < 100ms
- Metrics collection: < 10ms overhead per request
- Circuit breaker decision: < 0.1ms

### Throughput Targets
- Load balancing decisions: 100K decisions/second
- Concurrent health checks: 10K instances
- Metrics processing: 1M metrics/second
- Request routing: 50K requests/second

### Availability Targets
- Load balancer uptime: 99.99%
- Health check accuracy: 99.9%
- Failover time: < 1 second
- Recovery time: < 30 seconds

## Security Considerations

### Access Control
- Role-based access to load balancing configuration
- Instance registration authorization
- Metrics access control
- Administrative operation auditing

### Network Security
- TLS termination and passthrough
- Request sanitization
- Rate limiting and DDoS protection
- Secure health check endpoints

## Testing Strategy

### Unit Testing
- Algorithm implementations
- Health checker logic
- Metrics collection and aggregation
- Circuit breaker state transitions

### Integration Testing
- End-to-end load balancing flows
- Health monitoring accuracy
- Failover and recovery scenarios
- Performance under load

### Performance Testing
- Load balancing throughput
- Latency optimization
- Memory usage efficiency
- Scalability testing

## Examples

### Basic Load Balancer Setup
```rust
let config = LoadBalancerConfig {
    default_algorithm: AlgorithmType::HealthBased,
    algorithm_configs: HashMap::from([
        (AlgorithmType::HealthBased, AlgorithmConfig {
            enabled: true,
            weights: None,
            thresholds: Some(HashMap::from([
                ("health_threshold".to_string(), 0.8),
            ])),
            fallback_algorithm: Some(AlgorithmType::RoundRobin),
        }),
    ]),
    health_monitoring: HealthMonitorConfig {
        default_check_interval: Duration::from_secs(30),
        default_timeout: Duration::from_secs(5),
        unhealthy_threshold: 3,
        healthy_threshold: 2,
        health_check_types: vec![HealthCheckType::Http, HealthCheckType::Tcp],
    },
    // ... other configs
};

let load_balancer = LoadBalancer::new(config).await?;
```

### Service Instance Registration
```rust
let instance = ServiceInstance {
    id: "service-1-instance-1".to_string(),
    service_type: "api-gateway".to_string(),
    address: "http://10.0.1.100:8080".to_string(),
    weight: 1.0,
    capabilities: vec!["http".to_string(), "websocket".to_string()],
    metadata: HashMap::from([
        ("zone".to_string(), "us-west-1a".to_string()),
        ("version".to_string(), "1.2.3".to_string()),
    ]),
};

load_balancer.register_instance(instance).await?;
```

### Request Routing
```rust
let request = ServiceRequest {
    service_type: "api-gateway".to_string(),
    path: "/api/v1/users".to_string(),
    method: "GET".to_string(),
    headers: HashMap::new(),
    // ... other request details
};

let selected_instance = load_balancer.select_instance("api-gateway", &request).await?;
println!("Routing request to: {}", selected_instance.address);

// Record metrics after request completion
let metrics = RequestMetrics {
    instance_id: selected_instance.id.clone(),
    request_id: "req-123".to_string(),
    start_time: request_start,
    end_time: Utc::now(),
    response_time: Duration::from_millis(150),
    status_code: 200,
    request_size: 1024,
    response_size: 2048,
    error: None,
};

load_balancer.record_request_metrics(&selected_instance.id, metrics).await?;
```

### Custom Algorithm Implementation
```rust
pub struct CustomGeoAlgorithm {
    geo_resolver: Arc<GeoResolver>,
}

#[async_trait]
impl LoadBalancingAlgorithm for CustomGeoAlgorithm {
    async fn select_instance(&self, instances: &[ServiceInstance], request: &ServiceRequest) -> Result<ServiceInstance> {
        let client_location = self.geo_resolver.get_location(&request.client_ip).await?;
        
        let mut best_instance = None;
        let mut best_distance = f64::MAX;
        
        for instance in instances {
            if let Some(instance_location) = instance.metadata.get("location") {
                let distance = self.calculate_distance(&client_location, instance_location);
                if distance < best_distance {
                    best_distance = distance;
                    best_instance = Some(instance.clone());
                }
            }
        }
        
        best_instance.ok_or_else(|| Error::NoHealthyInstances)
    }
    
    fn algorithm_type(&self) -> AlgorithmType {
        AlgorithmType::Custom("geographic".to_string())
    }
    
    fn supports_weights(&self) -> bool {
        true
    }
}
```

## Monitoring and Observability

### Load Balancer Metrics
- Request distribution across instances
- Algorithm performance statistics
- Health check success/failure rates
- Circuit breaker state changes

### Instance Metrics
- Request latency and throughput
- Error rates and status codes
- Resource utilization (CPU, memory, network)
- Health score trends

### Alerting Rules
- Instance health degradation
- Load balancer algorithm failures
- Circuit breaker activations
- Performance threshold violations

## Version History

- v1.0.0: Initial specification
- v1.1.0: Added GPU-aware algorithms
- v1.2.0: Enhanced circuit breaker functionality
- v1.3.0: BiomeOS integration requirements

<version>1.3.0</version> 