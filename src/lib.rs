/*!
 * Songbird Orchestrator - Production-Grade Service Orchestration Platform
 * 
 * A comprehensive service orchestration platform built in Rust, designed for
 * enterprise-grade deployments with built-in security, observability, and scalability.
 */

// ============================================================================
// CORE MODULES - Fundamental building blocks
// ============================================================================

pub mod config;
pub mod errors;
pub mod traits;

// ============================================================================
// SERVICE MANAGEMENT - Core service orchestration
// ============================================================================

pub mod orchestrator;
pub mod registry;
pub mod discovery;

// ============================================================================
// COMMUNICATION & NETWORKING - Service-to-service communication
// ============================================================================

pub mod communication;
pub mod network;
pub mod http_server;

// ============================================================================
// RELIABILITY & PERFORMANCE - Production-grade reliability
// ============================================================================

pub mod robustness;      // Circuit breakers, retries, timeouts
pub mod load_balancer;   // Load balancing strategies
pub mod scalability;     // Auto-scaling and resource management

// ============================================================================
// INFRASTRUCTURE - Supporting systems
// ============================================================================

pub mod security;        // Authentication, authorization, encryption
pub mod observability;   // Metrics, monitoring, health checks
pub mod federation;      // Multi-cluster federation
pub mod proxy;          // Reverse proxy functionality
pub mod health;         // Health checking systems
pub mod api;            // REST/GraphQL APIs

// ============================================================================
// CORE EXPORTS - Main public interface
// ============================================================================

// Primary orchestrator
pub use orchestrator::{Orchestrator, OrchestratorEvent, OrchestratorMetrics};

// Configuration
pub use config::OrchestratorConfig;

// Error handling
pub use errors::{Result, SongbirdError};

// Service traits and types
pub use traits::service::{
    ServiceEndpoint, ServiceInfo, ServiceMetrics, ServiceRequest, ServiceResponse, UniversalService,
};

// Discovery and registry
pub use discovery::{SongbirdDiscovery, SongbirdDiscoveryConfig};
pub use observability::{ObservabilityEngine, ClusterStatus};

// ============================================================================
// STRUCTURED RE-EXPORTS - Organized by functional area
// ============================================================================

/// Communication and messaging
pub mod communication_types {
    pub use crate::traits::communication::{CommunicationLayer, MessageType, ServiceMessage};
    pub use crate::communication::HttpCommunication;
}

/// Load balancing and routing
pub mod load_balancing {
    pub use crate::load_balancer::{
        DefaultLoadBalancer, LoadBalancer, LoadBalancerConfig, LoadBalancerStats,
        LoadBalancerStrategy, ServiceInstance,
    };
    pub use crate::traits::load_balancer::LoadBalancingAlgorithm;
}

/// Security and authentication
pub mod security_types {
    pub use crate::security::{
        AuthEvent, AuthEventType, Subject, SubjectType,
        AuthenticationProvider, Credentials, 
        ProductionSecurityProvider, SecurityConfig, UserInfo, AuthToken,
        SecurityProvider, Resource, Action, Permission,
    };
}

/// Federation and clustering
pub mod federation_types {
    pub use crate::federation::{
        FederationConfig, FederationManager, FederationMode, FederationStatus,
    };
}

/// Robustness and reliability
pub mod robustness_types {
    pub use crate::robustness::{
        BulkheadConfig, CircuitBreaker, CircuitBreakerConfig, CircuitState, 
        HealthCheckConfig, HealthCheckStrategy, RateLimitConfig, RateLimitStrategy, 
        RateLimiter, RetryConfig, RetryExecutor, RobustnessConfig, RobustnessManager, 
        RobustnessStats, TimeoutConfig as RobustnessTimeoutConfig,
    };
}

/// Proxy and routing
pub mod proxy_types {
    pub use crate::proxy::{
        CircuitBreakerState, ConnectionProxy, LoadBalancerState, LoadBalancingStrategy, 
        ProxyConfig, ProxyRequest, ProxyResponse, ProxyStats,
    };
}

/// Network management
pub mod network_types {
    pub use crate::network::{
        DomainConfig, LoadBalancerStrategy as NetworkLoadBalancerStrategy, NetworkConfig,
        NetworkManager, ProxyHealthCheck, ProxyRoute, ProxyType, SslConfig, TimeoutConfig,
    };
}

/// Scalability and resource management
pub mod scalability_types {
    pub use crate::scalability::{
        InstanceHealth, LoadBalancingAlgorithm as ScalabilityLoadBalancingAlgorithm,
        LoadBalancingConfig as ScalabilityLoadBalancingConfig, PerformanceConfig, 
        PerformanceMetrics, PerformanceThresholds, ResourceConfig, ResourcePool, 
        ResourceUsage, ScalabilityConfig, ScalabilityManager, ScalabilityStats, 
        ScalingAction, ScalingDecision, ScalingStrategy,
        ServiceInstance as ScalabilityServiceInstance, ServiceScalingConfig,
    };
}

// ============================================================================
// CONVENIENCE PRELUDE - For easy imports
// ============================================================================

/// Common imports for most use cases
pub mod prelude {
    // Core types
    pub use crate::{
        Orchestrator, OrchestratorConfig, Result, SongbirdError,
        SongbirdDiscovery, ObservabilityEngine,
    };
    
    // Service traits and types
    pub use crate::traits::service::{
        UniversalService, ServiceInfo, ServiceRequest, ServiceResponse, ServiceMetrics,
        ServiceEndpoint, ResponseStatus, EndpointParameter,
    };
    
    // Common functionality
    pub use crate::communication_types::CommunicationLayer;
    pub use crate::load_balancing::{LoadBalancer, LoadBalancerStrategy};
    pub use crate::security_types::{SecurityProvider, AuthEvent, Subject};
    pub use crate::http_server::HttpServiceExt;
}
