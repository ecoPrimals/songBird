---
description: ENFORCE universal load balancing with capability-aware and resource-aware algorithms
globs: ["songbird/src/**/*.rs", "songbird/crates/**/*.rs"]
---

# Universal Load Balancing Engine Specification

## Context
- When implementing dynamic load balancing for universal service orchestration
- When distributing traffic across multiple service instances of any primal type
- When optimizing resource utilization and performance across the ecosystem
- When ensuring high availability and fault tolerance for all service types

## Requirements

### Universal Load Balancing Algorithms
- Implement multiple load balancing strategies that work with any service type
- Support capability-aware routing with real-time capability matching
- Enable resource-aware load balancing (CPU, memory, GPU, custom resources)
- Provide latency-optimized routing with performance metrics
- Support primal-specific load balancing algorithms

### Universal Service Discovery Integration
- Real-time service instance registration and deregistration for any primal type
- Automatic health monitoring and instance management
- Integration with universal service registry for capability-based routing
- Support for weighted routing based on instance capabilities and performance
- Pluggable discovery backends for different deployment environments

### Universal Performance Metrics Collection
- Real-time performance metrics gathering from any service type
- Historical performance data analysis and trend detection
- Predictive load balancing based on usage patterns
- Integration with monitoring systems (Prometheus, custom metrics)
- Capability-specific performance tracking

### Universal Circuit Breaker and Fault Tolerance
- Automatic circuit breaker activation for failing instances
- Graceful degradation and failover strategies
- Request retry logic with exponential backoff
- Dead letter queue for failed requests
- Primal-specific fault tolerance patterns

## Architecture

### Core Universal Load Balancer
```rust
pub struct UniversalLoadBalancer {
    strategies: HashMap<String, Box<dyn LoadBalancingStrategy>>,
    health_monitor: Arc<UniversalHealthMonitor>,
    metrics_collector: Arc<UniversalMetricsCollector>,
    circuit_breaker: Arc<UniversalCircuitBreakerManager>,
    service_registry: Arc<dyn UniversalServiceRegistry>,
    capability_matcher: Arc<CapabilityMatcher>,
    config: UniversalLoadBalancerConfig,
}

impl UniversalLoadBalancer {
    /// Create new universal load balancer with pluggable strategies
    pub fn new(
        config: UniversalLoadBalancerConfig,
        service_registry: Arc<dyn UniversalServiceRegistry>,
    ) -> Self {
        let mut strategies: HashMap<String, Box<dyn LoadBalancingStrategy>> = HashMap::new();
        
        // Register built-in strategies
        strategies.insert("round_robin".to_string(), Box::new(RoundRobinStrategy::new()));
        strategies.insert("least_connections".to_string(), Box::new(LeastConnectionsStrategy::new()));
        strategies.insert("capability_based".to_string(), Box::new(CapabilityBasedStrategy::new()));
        strategies.insert("primal_affinity".to_string(), Box::new(PrimalAffinityStrategy::new()));
        strategies.insert("performance_aware".to_string(), Box::new(PerformanceAwareStrategy::new()));
        strategies.insert("resource_aware".to_string(), Box::new(ResourceAwareStrategy::new()));
        
        // Add custom strategies from config
        for (name, strategy_config) in &config.custom_strategies {
            if let Ok(strategy) = Self::create_custom_strategy(strategy_config) {
                strategies.insert(name.clone(), strategy);
            }
        }
        
        Self {
            strategies,
            health_monitor: Arc::new(UniversalHealthMonitor::new()),
            metrics_collector: Arc::new(UniversalMetricsCollector::new()),
            circuit_breaker: Arc::new(UniversalCircuitBreakerManager::new()),
            service_registry,
            capability_matcher: Arc::new(CapabilityMatcher::new()),
            config,
        }
    }
    
    /// Select optimal service instance based on request requirements
    pub async fn select_instance(
        &self,
        request: &UniversalRequest,
    ) -> Result<RegisteredService, LoadBalancingError> {
        // Step 1: Find services by capability requirements
        let capable_services = self.service_registry
            .find_by_capability(&request.required_capabilities)
            .await?;
        
        if capable_services.is_empty() {
            return Err(LoadBalancingError::NoCapableServices);
        }
        
        // Step 2: Filter by health status
        let healthy_services = self.filter_healthy_services(&capable_services).await?;
        
        if healthy_services.is_empty() {
            return Err(LoadBalancingError::NoHealthyServices);
        }
        
        // Step 3: Filter by circuit breaker status
        let available_services = self.filter_available_services(&healthy_services).await?;
        
        if available_services.is_empty() {
            return Err(LoadBalancingError::NoAvailableServices);
        }
        
        // Step 4: Select strategy based on request preferences
        let strategy_name = request.load_balancing_strategy
            .as_deref()
            .unwrap_or(&self.config.default_strategy);
        
        let strategy = self.strategies.get(strategy_name)
            .ok_or(LoadBalancingError::UnknownStrategy(strategy_name.to_string()))?;
        
        // Step 5: Select instance using strategy
        let selected_instance = strategy.select_instance(&available_services, request).await?;
        
        // Step 6: Record selection for metrics
        self.metrics_collector.record_selection(&selected_instance, strategy_name).await?;
        
        Ok(selected_instance)
    }
    
    /// Register custom load balancing strategy
    pub fn register_strategy(&mut self, name: String, strategy: Box<dyn LoadBalancingStrategy>) {
        self.strategies.insert(name, strategy);
    }
    
    /// Update service weights for weighted strategies
    pub async fn update_weights(&self, weights: HashMap<String, f64>) -> Result<(), LoadBalancingError> {
        for strategy in self.strategies.values() {
            strategy.update_weights(weights.clone()).await?;
        }
        Ok(())
    }
    
    /// Get load balancing statistics
    pub async fn get_statistics(&self) -> Result<LoadBalancingStatistics, LoadBalancingError> {
        Ok(LoadBalancingStatistics {
            total_requests: self.metrics_collector.total_requests().await,
            strategy_usage: self.metrics_collector.strategy_usage().await,
            instance_distribution: self.metrics_collector.instance_distribution().await,
            average_response_time: self.metrics_collector.average_response_time().await,
            error_rates: self.metrics_collector.error_rates().await,
        })
    }
    
    async fn filter_healthy_services(&self, services: &[RegisteredService]) -> Result<Vec<RegisteredService>, LoadBalancingError> {
        let mut healthy_services = Vec::new();
        
        for service in services {
            let health_status = self.health_monitor.check_health(&service.instance_id).await?;
            if health_status == HealthStatus::Healthy {
                healthy_services.push(service.clone());
            }
        }
        
        Ok(healthy_services)
    }
    
    async fn filter_available_services(&self, services: &[RegisteredService]) -> Result<Vec<RegisteredService>, LoadBalancingError> {
        let mut available_services = Vec::new();
        
        for service in services {
            if self.circuit_breaker.is_available(&service.instance_id).await? {
                available_services.push(service.clone());
            }
        }
        
        Ok(available_services)
    }
}
```

### Universal Load Balancing Strategy Trait
```rust
#[async_trait]
pub trait LoadBalancingStrategy: Send + Sync {
    /// Strategy name
    fn name(&self) -> &str;
    
    /// Select service instance from available instances
    async fn select_instance(
        &self,
        instances: &[RegisteredService],
        request: &UniversalRequest,
    ) -> Result<RegisteredService, LoadBalancingError>;
    
    /// Update instance weights (for weighted strategies)
    async fn update_weights(&self, weights: HashMap<String, f64>) -> Result<(), LoadBalancingError> {
        // Default implementation - no-op for strategies that don't use weights
        Ok(())
    }
    
    /// Get strategy configuration
    fn configuration(&self) -> serde_json::Value;
    
    /// Handle strategy-specific metrics
    async fn collect_metrics(&self) -> Result<serde_json::Value, LoadBalancingError> {
        Ok(serde_json::json!({}))
    }
}
```

### Built-in Load Balancing Strategies

#### Capability-Based Strategy
```rust
pub struct CapabilityBasedStrategy {
    capability_scores: Arc<RwLock<HashMap<String, f64>>>,
}

impl CapabilityBasedStrategy {
    pub fn new() -> Self {
        Self {
            capability_scores: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    async fn calculate_capability_score(
        &self,
        instance: &RegisteredService,
        request: &UniversalRequest,
    ) -> Result<f64, LoadBalancingError> {
        let mut score = 0.0;
        
        // Base score for capability matching
        for requirement in &request.required_capabilities {
            if self.instance_has_capability(instance, requirement) {
                score += 1.0;
            }
        }
        
        // Bonus for exact capability match
        if self.exact_capability_match(instance, &request.required_capabilities) {
            score += 2.0;
        }
        
        // Performance bonus
        score += instance.metrics.performance_score;
        
        // Availability bonus
        if instance.health_status == HealthStatus::Healthy {
            score += 0.5;
        }
        
        // Primal-specific bonus
        if let Some(preferred_primal) = &request.preferred_primal_type {
            if instance.registration.primal_type == *preferred_primal {
                score += 1.0;
            }
        }
        
        Ok(score)
    }
    
    fn instance_has_capability(&self, instance: &RegisteredService, requirement: &CapabilityRequirement) -> bool {
        instance.capabilities.iter().any(|cap| self.capability_matches(cap, requirement))
    }
    
    fn capability_matches(&self, capability: &ServiceCapability, requirement: &CapabilityRequirement) -> bool {
        match (capability, requirement) {
            (ServiceCapability::ContainerRuntime { orchestrators }, CapabilityRequirement::ContainerRuntime { required_orchestrator }) => {
                orchestrators.contains(required_orchestrator)
            }
            (ServiceCapability::Authentication { methods }, CapabilityRequirement::Authentication { required_method }) => {
                methods.contains(required_method)
            }
            (ServiceCapability::Custom { name, .. }, CapabilityRequirement::Custom { name: req_name, .. }) => {
                name == req_name
            }
            _ => false,
        }
    }
}

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
    
    fn configuration(&self) -> serde_json::Value {
        serde_json::json!({
            "name": "capability_based",
            "description": "Routes requests based on service capability matching",
            "supports_weights": false,
            "supports_primal_affinity": true,
        })
    }
}
```

#### Performance-Aware Strategy
```rust
pub struct PerformanceAwareStrategy {
    performance_window: Duration,
    latency_weight: f64,
    throughput_weight: f64,
    error_rate_weight: f64,
}

impl PerformanceAwareStrategy {
    pub fn new() -> Self {
        Self {
            performance_window: Duration::from_secs(300), // 5 minute window
            latency_weight: 0.4,
            throughput_weight: 0.3,
            error_rate_weight: 0.3,
        }
    }
    
    async fn calculate_performance_score(
        &self,
        instance: &RegisteredService,
    ) -> Result<f64, LoadBalancingError> {
        let metrics = &instance.metrics;
        
        // Normalize latency score (lower is better)
        let latency_score = if metrics.average_latency_ms > 0.0 {
            1.0 / (1.0 + metrics.average_latency_ms / 100.0)
        } else {
            1.0
        };
        
        // Normalize throughput score (higher is better)
        let throughput_score = metrics.requests_per_second / 1000.0;
        
        // Normalize error rate score (lower is better)
        let error_rate_score = 1.0 - metrics.error_rate;
        
        // Calculate weighted score
        let score = (latency_score * self.latency_weight) +
                   (throughput_score * self.throughput_weight) +
                   (error_rate_score * self.error_rate_weight);
        
        Ok(score)
    }
}

#[async_trait]
impl LoadBalancingStrategy for PerformanceAwareStrategy {
    fn name(&self) -> &str {
        "performance_aware"
    }
    
    async fn select_instance(
        &self,
        instances: &[RegisteredService],
        request: &UniversalRequest,
    ) -> Result<RegisteredService, LoadBalancingError> {
        let mut scored_instances = Vec::new();
        
        for instance in instances {
            let score = self.calculate_performance_score(instance).await?;
            scored_instances.push((instance.clone(), score));
        }
        
        // Sort by score (highest first)
        scored_instances.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Return best performing instance
        scored_instances.into_iter()
            .next()
            .map(|(instance, _)| instance)
            .ok_or(LoadBalancingError::NoSuitableInstance)
    }
    
    fn configuration(&self) -> serde_json::Value {
        serde_json::json!({
            "name": "performance_aware",
            "description": "Routes requests based on historical performance metrics",
            "latency_weight": self.latency_weight,
            "throughput_weight": self.throughput_weight,
            "error_rate_weight": self.error_rate_weight,
            "performance_window_seconds": self.performance_window.as_secs(),
        })
    }
}
```

#### Primal Affinity Strategy
```rust
pub struct PrimalAffinityStrategy {
    primal_preferences: Arc<RwLock<HashMap<String, PrimalType>>>,
    fallback_strategy: Box<dyn LoadBalancingStrategy>,
}

impl PrimalAffinityStrategy {
    pub fn new() -> Self {
        Self {
            primal_preferences: Arc::new(RwLock::new(HashMap::new())),
            fallback_strategy: Box::new(RoundRobinStrategy::new()),
        }
    }
    
    pub async fn set_primal_preference(&self, request_pattern: String, primal_type: PrimalType) {
        let mut preferences = self.primal_preferences.write().await;
        preferences.insert(request_pattern, primal_type);
    }
    
    async fn get_preferred_primal(&self, request: &UniversalRequest) -> Option<PrimalType> {
        let preferences = self.primal_preferences.read().await;
        
        // Check direct preference
        if let Some(preferred) = &request.preferred_primal_type {
            return Some(*preferred);
        }
        
        // Check pattern matching
        for (pattern, primal_type) in preferences.iter() {
            if self.matches_pattern(request, pattern) {
                return Some(*primal_type);
            }
        }
        
        None
    }
    
    fn matches_pattern(&self, request: &UniversalRequest, pattern: &str) -> bool {
        // Simple pattern matching - can be extended
        request.operation.contains(pattern) || 
        request.source_service.contains(pattern) ||
        request.target_service.contains(pattern)
    }
}

#[async_trait]
impl LoadBalancingStrategy for PrimalAffinityStrategy {
    fn name(&self) -> &str {
        "primal_affinity"
    }
    
    async fn select_instance(
        &self,
        instances: &[RegisteredService],
        request: &UniversalRequest,
    ) -> Result<RegisteredService, LoadBalancingError> {
        if let Some(preferred_primal) = self.get_preferred_primal(request).await {
            // Filter instances by preferred primal type
            let preferred_instances: Vec<_> = instances.iter()
                .filter(|instance| instance.registration.primal_type == preferred_primal)
                .cloned()
                .collect();
            
            if !preferred_instances.is_empty() {
                return self.fallback_strategy.select_instance(&preferred_instances, request).await;
            }
        }
        
        // No preference or no instances of preferred type - use fallback
        self.fallback_strategy.select_instance(instances, request).await
    }
    
    fn configuration(&self) -> serde_json::Value {
        serde_json::json!({
            "name": "primal_affinity",
            "description": "Routes requests to preferred primal types with fallback",
            "supports_preferences": true,
            "fallback_strategy": self.fallback_strategy.name(),
        })
    }
}
```

### Universal Configuration
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalLoadBalancerConfig {
    pub default_strategy: String,
    pub health_check_interval: Duration,
    pub metrics_collection_interval: Duration,
    pub circuit_breaker_threshold: u32,
    pub circuit_breaker_timeout: Duration,
    pub custom_strategies: HashMap<String, CustomStrategyConfig>,
    pub primal_preferences: HashMap<String, PrimalType>,
    pub performance_weights: PerformanceWeights,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceWeights {
    pub latency: f64,
    pub throughput: f64,
    pub error_rate: f64,
    pub resource_utilization: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomStrategyConfig {
    pub strategy_type: String,
    pub parameters: serde_json::Value,
}
```

### Universal Metrics Collection
```rust
pub struct UniversalMetricsCollector {
    metrics_store: Arc<RwLock<MetricsStore>>,
    collection_interval: Duration,
}

impl UniversalMetricsCollector {
    pub async fn record_selection(&self, instance: &RegisteredService, strategy: &str) -> Result<(), MetricsError> {
        let mut store = self.metrics_store.write().await;
        
        // Record selection
        store.record_selection(instance.instance_id.clone(), strategy.to_string()).await?;
        
        // Update instance metrics
        store.update_instance_metrics(&instance.instance_id, &instance.metrics).await?;
        
        Ok(())
    }
    
    pub async fn total_requests(&self) -> u64 {
        let store = self.metrics_store.read().await;
        store.total_requests()
    }
    
    pub async fn strategy_usage(&self) -> HashMap<String, u64> {
        let store = self.metrics_store.read().await;
        store.strategy_usage()
    }
    
    pub async fn instance_distribution(&self) -> HashMap<String, u64> {
        let store = self.metrics_store.read().await;
        store.instance_distribution()
    }
}
```

## Implementation Guidelines

### Universal Integration
- All load balancing strategies MUST implement the `LoadBalancingStrategy` trait
- Strategies MUST support universal request/response formats
- Strategies MUST handle capability-based routing
- Strategies MUST integrate with universal health monitoring

### Performance Optimization
- Implement efficient capability matching algorithms
- Use caching for frequently accessed metrics
- Support concurrent request processing
- Optimize for sub-10ms selection latency

### Fault Tolerance
- Implement circuit breaker patterns for failing instances
- Support graceful degradation during service failures
- Enable automatic retry with exponential backoff
- Provide comprehensive error handling

### Extensibility
- Support custom load balancing strategies
- Enable runtime strategy registration
- Support dynamic configuration updates
- Provide strategy-specific metrics

## Success Metrics

### Performance
- Sub-10ms instance selection latency
- 99.9% successful request routing
- Linear scaling with service count
- Efficient resource utilization

### Reliability
- Automatic failover on instance failures
- Circuit breaker protection
- Zero downtime during configuration changes
- Consistent behavior across all primal types

### Flexibility
- Support for custom strategies
- Dynamic strategy switching
- Capability-based routing
- Primal-specific optimizations

This specification establishes universal load balancing patterns that work with any primal type while maintaining optimal performance and flexibility for future expansion. 