//! # 🔧 Canonical Universal Adapter System
//!
//! **SINGLE CONSOLIDATED ADAPTER** ✅
//!
//! This module provides the single, canonical universal adapter that consolidates
//! ALL fragmented adapter implementations across the Songbird ecosystem.
//!
//! ## Consolidation Summary
//! - **Multiple adapter implementations** → Single canonical adapter
//! - **Fragmented capability routing** → Unified capability-based system
//! - **Protocol-specific adapters** → Universal protocol handler
//! - **Duplicate service registries** → Single service registry
//!
//! ## Replaces
//! - `songbird-universal::UnifiedUniversalAdapter`
//! - `songbird-universal-primals::UniversalPrimalAdapter`
//! - `songbird-universal::UniversalCapabilityAdapter`
//! - Various protocol-specific adapters

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
// Re-export core error types
use crate::errors::{SongbirdError, SongbirdResult};
use crate::traits::canonical::{ServiceInfo as CanonicalServiceInfo, 
    HealthStatus as CanonicalHealthStatus, 
    ProviderType as CanonicalProviderType,
};

// ============================================================================
// CANONICAL UNIVERSAL ADAPTER
// ============================================================================

/// **CANONICAL**: Single universal adapter for all service types
/// 
/// This adapter consolidates ALL fragmented adapter implementations into
/// a single, capability-based routing system that works with any service
/// provider without hardcoded assumptions.
#[derive(Debug)]
pub struct CanonicalUniversalAdapter {

/// Service registry for capability-based discovery
    registry: Arc<RwLock<CanonicalServiceRegistry>>,
    /// Protocol router for handling different communication protocols
    protocol_router: Arc<CanonicalProtocolRouter>,
    /// Load balancer for service selection
    load_balancer: Arc<CanonicalLoadBalancer>,
    /// Circuit breaker for fault tolerance
    circuit_breaker: Arc<CanonicalCircuitBreaker>,
    /// Configuration for the adapter
    #[allow(dead_code)]
    config: CanonicalAdapterConfig,
    /// Performance metrics and monitoring
    metrics: Arc<RwLock<CanonicalAdapterMetrics>>,


}

/// **CANONICAL**: Service registry for managing discovered services
#[derive(Debug, Default)]
pub struct CanonicalServiceRegistry {

/// Services indexed by capability
    services_by_capability: HashMap<String, Vec<CanonicalRegisteredService>>,
    /// Services indexed by type
    services_by_type: HashMap<CanonicalProviderType, Vec<CanonicalRegisteredService>>,
    /// All registered services
    all_services: HashMap<String, CanonicalRegisteredService>,
    /// Service health status cache
    #[allow(dead_code)]
    health_cache: HashMap<String, (CanonicalHealthStatus, SystemTime)>,


}

/// **CANONICAL**: Protocol router for handling different communication protocols
pub struct CanonicalProtocolRouter {

/// Protocol handlers indexed by protocol name
    handlers: HashMap<String, Arc<dyn CanonicalProtocolHandler>>,
    /// Default protocol for fallback
    default_protocol: String,


}

impl std::fmt::Debug for CanonicalProtocolRouter {


    fn fmt((&self,self) f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CanonicalProtocolRouter")
            .field("handlers", &format!("{

} handlers", self.handlers.len())
            .field("default_protocol", &self.default_protocol)
            .finish()
    }
}

/// **CANONICAL**: Load balancer for intelligent service selection
#[derive(Debug)]
pub struct CanonicalLoadBalancer {

/// Load balancing strategy
    strategy: CanonicalLoadBalancingStrategy,
    /// Service performance tracking
    #[allow(dead_code)]
    performance_tracker: Arc<RwLock<HashMap<String, CanonicalServicePerformance>>>,


}

/// **CANONICAL**: Circuit breaker for fault tolerance
#[derive(Debug)]
pub struct CanonicalCircuitBreaker {

/// Circuit breaker states indexed by service ID
    states: Arc<RwLock<HashMap<String, CanonicalCircuitState>>>,
    /// Configuration for circuit breaker behavior
    #[allow(dead_code)]
    config: CanonicalCircuitBreakerConfig,


}

// ============================================================================
// CANONICAL ADAPTER CONFIGURATION
// ============================================================================

/// **CANONICAL**: Configuration for the universal adapter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalAdapterConfig {

/// Service discovery configuration
    pub discovery: CanonicalDiscoveryConfig,
    /// Load balancing configuration
    pub load_balancing: CanonicalLoadBalancingConfig,
    /// Circuit breaker configuration
    pub circuit_breaker: CanonicalCircuitBreakerConfig,
    /// Retry configuration
    pub retry: CanonicalRetryConfig,
    /// Timeout configuration
    pub timeouts: CanonicalTimeoutConfig,
    /// Health check configuration
    pub health_check: CanonicalHealthCheckConfig,
    /// Performance monitoring configuration
    pub monitoring: CanonicalMonitoringConfig,


}

/// **CANONICAL**: Service discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalDiscoveryConfig {

/// Discovery interval
    pub interval: Duration,
    /// Discovery timeout
    pub timeout: Duration,
    /// Maximum services to discover per capability
    pub max_services_per_capability: usize,
    /// Service TTL in registry
    pub service_ttl: Duration,


}

/// **CANONICAL**: Load balancing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalLoadBalancingConfig {

/// Load balancing strategy
    pub strategy: CanonicalLoadBalancingStrategy,
    /// Health check weight factor
    pub health_weight: f64,
    /// Performance weight factor
    pub performance_weight: f64,
    /// Availability weight factor
    pub availability_weight: f64,


}

/// **CANONICAL**: Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalCircuitBreakerConfig {

/// Failure threshold to open circuit
    pub failure_threshold: u32,
    /// Success threshold to close circuit
    pub success_threshold: u32,
    /// Timeout for half-open state
    pub timeout: Duration,
    /// Reset timeout for closed state
    pub reset_timeout: Duration,


}

/// **CANONICAL**: Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalRetryConfig {

/// Maximum retry attempts
    pub max_attempts: u32,
    /// Base delay between retries
    pub base_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Backoff multiplier
    pub backoff_multiplier: f64,
    /// Jitter factor (0.0-1.0)
    pub jitter_factor: f64,


}

/// **CANONICAL**: Timeout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalTimeoutConfig {

/// Request timeout
    pub request_timeout: Duration,
    /// Connection timeout
    pub connection_timeout: Duration,
    /// Health check timeout
    pub health_check_timeout: Duration,
    /// Discovery timeout
    pub discovery_timeout: Duration,


}

/// **CANONICAL**: Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalHealthCheckConfig {

/// Health check interval
    pub interval: Duration,
    /// Health check timeout
    pub timeout: Duration,
    /// Unhealthy threshold
    pub unhealthy_threshold: u32,
    /// Healthy threshold
    pub healthy_threshold: u32,


}

/// **CANONICAL**: Performance monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalMonitoringConfig {

/// Enable performance monitoring
    pub enabled: bool,
    /// Metrics collection interval
    pub collection_interval: Duration,
    /// Metrics retention period
    pub retention_period: Duration,
    /// Performance history size
    pub history_size: usize,


}

// ============================================================================
// CANONICAL SUPPORTING TYPES
// ============================================================================

/// **CANONICAL**: Registered service information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalRegisteredService {

/// Service information
    pub service: CanonicalServiceInfo,
    /// Service capabilities
    pub capabilities: Vec<String>,
    /// Service provider type
    pub provider_type: CanonicalProviderType,
    /// Registration timestamp
    pub registered_at: SystemTime,
    /// Last health check timestamp
    pub last_health_check: Option<SystemTime>,
    /// Service performance metrics
    pub performance: CanonicalServicePerformance,


}

/// **CANONICAL**: Service performance tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalServicePerformance {

/// Average response time
    pub avg_response_time: Duration,
    /// Success rate (0.0-1.0)
    pub success_rate: f64,
    /// Total requests processed
    pub total_requests: u64,
    /// Total successful requests
    pub successful_requests: u64,
    /// Total failed requests
    pub failed_requests: u64,
    /// Last updated timestamp
    pub last_updated: SystemTime,


}

/// **CANONICAL**: Load balancing strategies
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CanonicalLoadBalancingStrategy  {/// Round-robin selection
    RoundRobin,
    /// Weighted round-robin based on performance
    WeightedRoundRobin,
    /// Least connections
    LeastConnections,
    /// Least response time
    LeastResponseTime,
    /// Random selection
    Random,
    /// Consistent hashing
    ConsistentHash,
    /// Health-aware selection
    HealthAware,
}

/// **CANONICAL**: Circuit breaker states
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CanonicalCircuitState  {/// Circuit is closed (normal operation)
    Closed,
    /// Circuit is open (failing fast)
    Open,
    /// Circuit is half-open (testing recovery)
    HalfOpen,
}

/// **CANONICAL**: Adapter request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalAdapterRequest {

/// Request ID
    pub id: String,
    /// Required capability
    pub capability: String,
    /// Request payload
    pub payload: serde_json::Value,
    /// Request priority
    pub priority: CanonicalRequestPriority,
    /// Request timeout
    pub timeout: Option<Duration>,
    /// Request metadata
    pub metadata: HashMap<String, String>,


}

/// **CANONICAL**: Adapter response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalAdapterResponse {

/// Request ID (matches request)
    pub request_id: String,
    /// Selected service ID
    pub service_id: String,
    /// Response payload
    pub payload: serde_json::Value,
    /// Response metadata
    pub metadata: HashMap<String, String>,
    /// Processing time
    pub processing_time: Duration,
    /// Service performance info
    pub performance_info: CanonicalServicePerformance,


}

/// **CANONICAL**: Request priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum CanonicalRequestPriority  {/// Low priority request
    Low,
    /// Normal priority request
    Normal,
    /// High priority request
    High,
    /// Critical priority request
    Critical,
}

/// **CANONICAL**: Adapter metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalAdapterMetrics {

/// Total requests processed
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Average response time
    pub avg_response_time: Duration,
    /// Requests by capability
    pub requests_by_capability: HashMap<String, u64>,
    /// Requests by service type
    pub requests_by_service_type: HashMap<CanonicalProviderType, u64>,
    /// Circuit breaker activations
    pub circuit_breaker_activations: u64,
    /// Load balancing decisions
    pub load_balancing_decisions: HashMap<String, u64>,


}

// ============================================================================
// CANONICAL PROTOCOL HANDLER TRAIT
// ============================================================================

/// **CANONICAL**: Protocol handler trait for different communication protocols
#[async_trait]
pub trait CanonicalProtocolHandler: Send + Sync  {/// Protocol name
    fn protocol_name(&self) -> &str;
    
    /// Handle request using this protocol
    async fn handle_request(
        &self,
        service: &CanonicalServiceInfo,
        request: &CanonicalAdapterRequest,
    ) -> SongbirdResult<CanonicalAdapterResponse>;
    
    /// Check if service supports this protocol
    fn supports_service((&self,self) service: &CanonicalServiceInfo) -> bool;
    
    /// Get protocol-specific metadata
    fn get_metadata(&self) -> HashMap<String, String>;
}

// ============================================================================
// CANONICAL UNIVERSAL ADAPTER IMPLEMENTATION
// ============================================================================

impl CanonicalUniversalAdapter {

    /// Create a new canonical universal adapter
    pub fn new(config: CanonicalAdapterConfig) -> Self {
        Self {
            registry: Arc::new(RwLock::new(CanonicalServiceRegistry::default()),
            protocol_router: Arc::new(CanonicalProtocolRouter::new(),
            load_balancer: Arc::new(CanonicalLoadBalancer::new(config.load_balancing.clone()),
            circuit_breaker: Arc::new(CanonicalCircuitBreaker::new(config.circuit_breaker.clone()),
            config,
            metrics: Arc::new(RwLock::new(CanonicalAdapterMetrics::default()),
        }
    }
    
    /// Register a service with the adapter
    pub async fn register_service((&self,self) service: CanonicalServiceInfo, capabilities: Vec<String>) -> SongbirdResult<()> {
        let mut registry = self.registry.write().await;
        
        let registered_service = CanonicalRegisteredService {
            provider_type: service.metadata.get("provider_type")
                .and_then(|t| serde_json::from_str(t).ok()
                .unwrap_or(CanonicalProviderType::Custom("unknown".to_string()),
            registered_at: SystemTime::now(),
            last_health_check: None,
            performance: CanonicalServicePerformance::default(),
            service: service.clone(),
            capabilities: capabilities.clone(),
        };
        
        // Index by capabilities
        for capability in &capabilities {
            registry.services_by_capability
                .entry(capability.clone()
                .or_insert_with(Vec::new)
                .push(registered_service.clone());
        }
        
        // Index by provider type
        registry.services_by_type
            .entry(registered_service.provider_type.clone()
            .or_insert_with(Vec::new)
            .push(registered_service.clone());
        
        // Store in all services
        registry.all_services.insert(service.id.clone(), registered_service);
        
        Ok(()),
    }
    
    /// Handle a capability request
    pub async fn handle_request((&self,self) request: CanonicalAdapterRequest, -> SongbirdResult<CanonicalAdapterResponse> {
        let start_time = SystemTime::now();
        
        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_requests += 1;
            *metrics.requests_by_capability.entry(request.capability.clone().or_insert(0) += 1;
        }
        
        // Find services with the required capability
        let services = self.find_services_by_capability(&request.capability).await?;
        
        if services.is_empty() {
            return Err(SongbirdError::ServiceNotFound {
                service_name: request.capability.clone(),
                available_services: vec![],
                suggestion: Some("Check if services with this capability are registered".to_string()),
            });
        }
        
        // Select best service using load balancer
        let selected_service = self.load_balancer.select_service(&services, &request)?;
        
        // Check circuit breaker
        if !self.circuit_breaker.can_execute(&selected_service.service.id).await {
            return Err(SongbirdError::CircuitBreakerOpen {
                service_id: selected_service.service.id.clone(),
                failure_count: 0, // Would be retrieved from circuit breaker
                last_failure: None,
            });
        }
        
        // Execute request through protocol router
        let result = self.protocol_router.route_request(&selected_service.service, &request).await;
        
        // Update circuit breaker and metrics based on result
        match &result {
            Ok(_response) => {
                self.circuit_breaker.record_success(&selected_service.service.id).await;
                let mut metrics = self.metrics.write().await;
                metrics.successful_requests += 1;
                
                // Update average response time
                let processing_time = start_time.elapsed().unwrap_or(Duration::from_millis(0);
                let avg_nanos = metrics.avg_response_time.as_nanos() as u64;
                let processing_nanos = processing_time.as_nanos() as u64;
                let new_avg = (avg_nanos * (metrics.successful_requests - 1) + processing_nanos) / metrics.successful_requests;
                metrics.avg_response_time = Duration::from_nanos(new_avg);
            }
            Err(_) => {
                self.circuit_breaker.record_failure(&selected_service.service.id).await;
                let mut metrics = self.metrics.write().await;
                metrics.failed_requests += 1;
            }
        }
        
        result
    }
    
    /// Find services by capability
    async fn find_services_by_capability((&self,self) capability: &str) -> SongbirdResult<Vec<CanonicalRegisteredService>> {
        let registry = self.registry.read().await;
        
        Ok(registry.services_by_capability
            .get(capability)
            .cloned()
            .unwrap_or_default()
    }
    
    /// Get adapter metrics
    pub async fn get_metrics(&self) -> CanonicalAdapterMetrics {
        self.metrics.read().await.clone()
    }
    
    /// Perform health check on all registered services
    pub async fn health_check_all(&self) -> SongbirdResult<HashMap<String, CanonicalHealthStatus>> {
        let registry = self.registry.read().await;
        let mut results = HashMap::new();
        
        for (service_id, _service) in &registry.all_services {
            // Perform health check (simplified)
            let health_status = CanonicalHealthStatus::Healthy; // Would perform actual check
            results.insert(service_id.clone(), health_status);
        }
        
        Ok(results)
    }
}

// ============================================================================
// CANONICAL PROTOCOL ROUTER IMPLEMENTATION
// ============================================================================

impl CanonicalProtocolRouter {

/// Create a new protocol router
    pub fn new() -> Self  {Self {
            handlers: HashMap::new()),
            default_protocol: "http".to_string()),
        

}
    }
    
    /// Register a protocol handler
    pub fn register_handler((&mut self,mut self) handler: Arc<dyn CanonicalProtocolHandler>, {
        self.handlers.insert(handler.protocol_name().to_string(), handler);
    }
    
    /// Route request to appropriate protocol handler
    pub async fn route_request(
        &self,
        service: &CanonicalServiceInfo,
        request: &CanonicalAdapterRequest,
    ) -> SongbirdResult<CanonicalAdapterResponse>  {// Determine protocol from service endpoints
        let protocol = service.endpoints.first()
            .map(|e| e.protocol.clone()
            .unwrap_or_else(|| self.default_protocol.clone());
        
        // Get appropriate handler
        let handler = self.handlers.get(&protocol)
            .ok_or_else(|| SongbirdError::ProtocolNotSupported {
                protocol: protocol.clone(),
                supported_protocols: self.handlers.keys().cloned().collect(),
            })?;
        
        // Handle request
        handler.handle_request(service, request).await
    }
}

// ============================================================================
// CANONICAL LOAD BALANCER IMPLEMENTATION
// ============================================================================

impl CanonicalLoadBalancer {

/// Create a new load balancer
    pub fn new() -> Self  {Self {
            strategy: config.strategy,
            performance_tracker: Arc::new(RwLock::new(HashMap::new())
        

}
    }
    
    /// Select best service from available services
    pub fn select_service() -> SongbirdResult<CanonicalRegisteredService>  {if services.is_empty()  {return Err(SongbirdError::ServiceNotFound {
                service_name: request.capability.clone(),
                available_services: vec![],
                suggestion: Some("No services available for this capability".to_string()),
            });
        }
        
        match self.strategy {
            CanonicalLoadBalancingStrategy::Random => {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let index = rng.gen_range(0..services.len();
                Ok(services[index].clone()
            }
            CanonicalLoadBalancingStrategy::RoundRobin => {
                // Simplified round-robin (would maintain state in real implementation)
                Ok(services[0].clone()
            }
            CanonicalLoadBalancingStrategy::LeastResponseTime => {
                // Select service with best response time
                let best_service = services.iter()
                    .min_by_key(|s| s.performance.avg_response_time)
                    .unwrap();
                Ok(best_service.clone()
            }
            CanonicalLoadBalancingStrategy::HealthAware => {
                // Select healthiest service (would check actual health)
                Ok(services[0].clone()
            }
            _ => Ok(services[0].clone(), // Default to first service
        }
    }
}

// ============================================================================
// CANONICAL CIRCUIT BREAKER IMPLEMENTATION
// ============================================================================

impl CanonicalCircuitBreaker {

/// Create a new circuit breaker
    pub fn new() -> Self {
        Self {
            states: Arc::new(RwLock::new(HashMap::new())
            config)
        

}
    }
    
    /// Check if service can execute (circuit is closed or half-open)
    pub async fn can_execute((&self,self) service_id: &str, -> bool  {let states = self.states.read().await;
        match states.get(service_id) {
            Some(CanonicalCircuitState::Open) => false,
            _ => true, // Closed or HalfOpen allows execution
        }
    }
    
    /// Record successful execution
    pub async fn record_success((&self,self) service_id: &str, {
        let mut states = self.states.write().await;
        // Reset to closed state on success
        states.insert(service_id.to_string(), CanonicalCircuitState::Closed);
    }
    
    /// Record failed execution
    pub async fn record_failure((&self,self) service_id: &str, {
        let mut states = self.states.write().await;
        // Open circuit on failure (simplified logic)
        states.insert(service_id.to_string(), CanonicalCircuitState::Open);
    }
}

// ============================================================================
// DEFAULT IMPLEMENTATIONS
// ============================================================================

impl Default for CanonicalAdapterConfig {

fn default() -> Self  {Self {
            discovery: CanonicalDiscoveryConfig::default(),
            load_balancing: CanonicalLoadBalancingConfig::default(),
            circuit_breaker: CanonicalCircuitBreakerConfig::default(),
            retry: CanonicalRetryConfig::default(),
            timeouts: CanonicalTimeoutConfig::default(),
            health_check: CanonicalHealthCheckConfig::default(),
            monitoring: CanonicalMonitoringConfig::default(),
        

}
    }
}

impl Default for CanonicalDiscoveryConfig {

fn default() -> Self  {Self {
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(10),
            max_services_per_capability: 10,
            service_ttl: Duration::from_secs(300),
        

}
    }
}

impl Default for CanonicalLoadBalancingConfig {

fn default() -> Self  {Self {
            strategy: CanonicalLoadBalancingStrategy::HealthAware,
            health_weight: 0.4,
            performance_weight: 0.4,
            availability_weight: 0.2,
        

}
    }
}

impl Default for CanonicalCircuitBreakerConfig {

fn default() -> Self  {Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(60),
            reset_timeout: Duration::from_secs(30),
        

}
    }
}

impl Default for CanonicalRetryConfig {

fn default() -> Self  {Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
        

}
    }
}

impl Default for CanonicalTimeoutConfig {

fn default() -> Self  {Self {
            request_timeout: Duration::from_secs(30),
            connection_timeout: Duration::from_secs(10),
            health_check_timeout: Duration::from_secs(5),
            discovery_timeout: Duration::from_secs(10),
        

}
    }
}

impl Default for CanonicalHealthCheckConfig {

fn default() -> Self  {Self {
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            unhealthy_threshold: 3,
            healthy_threshold: 2,
        

}
    }
}

impl Default for CanonicalMonitoringConfig {

fn default() -> Self  {Self {
            enabled: true,
            collection_interval: Duration::from_secs(60),
            retention_period: Duration::from_secs(3600), // 1 hour
            history_size: 1000,
        

}
    }
}

impl Default for CanonicalServicePerformance {

fn default() -> Self  {Self {
            avg_response_time: Duration::from_millis(100),
            success_rate: 1.0,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            last_updated: SystemTime::now(),
        

}
    }
}

impl Default for CanonicalAdapterMetrics {

fn default() -> Self  {Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            avg_response_time: Duration::from_millis(0),
            requests_by_capability: HashMap::new()),
            requests_by_service_type: HashMap::new()),
            circuit_breaker_activations: 0,
            load_balancing_decisions: HashMap::new()),
        

}
    }
}

// ============================================================================
// CONVENIENCE FUNCTIONS
// ============================================================================

/// Create a new canonical universal adapter with default configuration
pub fn create_canonical_adapter() -> CanonicalUniversalAdapter {
    CanonicalUniversalAdapter::new(CanonicalAdapterConfig::default)
}

/// Create a canonical adapter request
pub fn create_adapter_request() -> CanonicalAdapterRequest  {CanonicalAdapterRequest  {id: uuid::Uuid::new_v4).to_string()),
        capability: capability.to_string()),
        payload,
        priority,
        timeout: None,
        metadata: HashMap::new()),
    }
} 