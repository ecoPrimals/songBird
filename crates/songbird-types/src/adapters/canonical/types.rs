// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Configuration, DTOs, and supporting types for the canonical adapter system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use crate::traits::canonical::{
    ProviderType as CanonicalProviderType, ServiceInfo as CanonicalServiceInfo,
};

// ============================================================================
// ADAPTER CONFIGURATION
// ============================================================================

/// Configuration for the universal adapter.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CanonicalAdapterConfig {
    /// Service discovery configuration.
    pub discovery: CanonicalDiscoveryConfig,
    /// Load balancing configuration.
    pub load_balancing: CanonicalLoadBalancingConfig,
    /// Circuit breaker configuration.
    pub circuit_breaker: CanonicalCircuitBreakerConfig,
    /// Retry configuration.
    pub retry: CanonicalRetryConfig,
    /// Timeout configuration.
    pub timeouts: CanonicalTimeoutConfig,
    /// Health check configuration.
    pub health_check: CanonicalHealthCheckConfig,
    /// Performance monitoring configuration.
    pub monitoring: CanonicalMonitoringConfig,
}

/// Service discovery configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalDiscoveryConfig {
    /// Discovery interval.
    pub interval: Duration,
    /// Discovery timeout.
    pub timeout: Duration,
    /// Maximum services to discover per capability.
    pub max_services_per_capability: usize,
    /// Service TTL in registry.
    pub service_ttl: Duration,
}

/// Load balancing configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalLoadBalancingConfig {
    /// Load balancing strategy.
    pub strategy: CanonicalLoadBalancingStrategy,
    /// Health check weight factor.
    pub health_weight: f64,
    /// Performance weight factor.
    pub performance_weight: f64,
    /// Availability weight factor.
    pub availability_weight: f64,
}

/// Circuit breaker configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalCircuitBreakerConfig {
    /// Failure threshold to open circuit.
    pub failure_threshold: u32,
    /// Success threshold to close circuit.
    pub success_threshold: u32,
    /// Timeout for half-open state.
    pub timeout: Duration,
    /// Reset timeout for closed state.
    pub reset_timeout: Duration,
}

/// Retry configuration.
///
/// Foundation definition in `songbird-types`; the authoritative canonical
/// lives in `songbird_config::canonical::resilience::RetryConfig`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalRetryConfig {
    /// Maximum retry attempts.
    pub max_attempts: u32,
    /// Initial delay between retries.
    pub initial_delay: Duration,
    /// Maximum delay between retries.
    pub max_delay: Duration,
    /// Backoff multiplier.
    pub backoff_multiplier: f64,
}

/// Timeout configuration.
///
/// Foundation definition; matches `songbird_config::canonical::network::NetworkTimeouts`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalTimeoutConfig {
    /// Request timeout.
    pub request_timeout: Duration,
    /// Connection timeout.
    pub connection_timeout: Duration,
    /// Health check timeout.
    pub health_check_timeout: Duration,
    /// Discovery timeout.
    pub discovery_timeout: Duration,
}

/// Health check configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalHealthCheckConfig {
    /// Health check interval.
    pub interval: Duration,
    /// Health check timeout.
    pub timeout: Duration,
    /// Unhealthy threshold.
    pub unhealthy_threshold: u32,
    /// Healthy threshold.
    pub healthy_threshold: u32,
}

/// Performance monitoring configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalMonitoringConfig {
    /// Enable performance monitoring.
    pub enabled: bool,
    /// Metrics collection interval.
    pub collection_interval: Duration,
    /// Metrics retention period.
    pub retention_period: Duration,
    /// Performance history size.
    pub history_size: usize,
}

// ============================================================================
// SUPPORTING TYPES
// ============================================================================

/// Registered service information (internal to the adapter registry).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalRegisteredService {
    /// Service information.
    pub service: CanonicalServiceInfo,
    /// Service capabilities.
    pub capabilities: Vec<String>,
    /// Service provider type.
    pub provider_type: CanonicalProviderType,
    /// Registration timestamp.
    pub registered_at: SystemTime,
    /// Last health check timestamp.
    pub last_health_check: Option<SystemTime>,
    /// Service performance metrics.
    pub performance: CanonicalServicePerformance,
}

/// Service performance tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalServicePerformance {
    /// Average response time.
    pub avg_response_time: Duration,
    /// Success rate (0.0–1.0).
    pub success_rate: f64,
    /// Total requests processed.
    pub total_requests: u64,
    /// Total successful requests.
    pub successful_requests: u64,
    /// Total failed requests.
    pub failed_requests: u64,
    /// Last updated timestamp.
    pub last_updated: SystemTime,
}

/// Load balancing strategies.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CanonicalLoadBalancingStrategy {
    /// Round-robin selection.
    RoundRobin,
    /// Weighted round-robin based on performance.
    WeightedRoundRobin,
    /// Least connections.
    LeastConnections,
    /// Least response time.
    LeastResponseTime,
    /// Random selection.
    Random,
    /// Consistent hashing.
    ConsistentHash,
    /// Health-aware selection.
    HealthAware,
}

/// Circuit breaker states.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CanonicalCircuitState {
    /// Circuit is closed (normal operation).
    Closed,
    /// Circuit is open (failing fast).
    Open,
    /// Circuit is half-open (testing recovery).
    HalfOpen,
}

/// Adapter request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalAdapterRequest {
    /// Request ID.
    pub id: String,
    /// Required capability.
    pub capability: String,
    /// Request payload.
    pub payload: serde_json::Value,
    /// Request priority.
    pub priority: CanonicalRequestPriority,
    /// Request timeout.
    pub timeout: Option<Duration>,
    /// Request metadata.
    pub metadata: HashMap<String, String>,
}

/// Adapter response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalAdapterResponse {
    /// Request ID (matches request).
    pub request_id: String,
    /// Selected service ID.
    pub service_id: String,
    /// Response payload.
    pub payload: serde_json::Value,
    /// Response metadata.
    pub metadata: HashMap<String, String>,
    /// Processing time.
    pub processing_time: Duration,
    /// Service performance info.
    pub performance_info: CanonicalServicePerformance,
}

/// Request priority levels.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum CanonicalRequestPriority {
    /// Low priority request.
    Low,
    /// Normal priority request.
    Normal,
    /// High priority request.
    High,
    /// Critical priority request.
    Critical,
}

/// Adapter metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalAdapterMetrics {
    /// Total requests processed.
    pub total_requests: u64,
    /// Successful requests.
    pub successful_requests: u64,
    /// Failed requests.
    pub failed_requests: u64,
    /// Average response time.
    pub avg_response_time: Duration,
    /// Requests by capability.
    pub requests_by_capability: HashMap<String, u64>,
    /// Requests by service type.
    pub requests_by_service_type: HashMap<CanonicalProviderType, u64>,
    /// Circuit breaker activations.
    pub circuit_breaker_activations: u64,
    /// Load balancing decisions.
    pub load_balancing_decisions: HashMap<String, u64>,
}

// ============================================================================
// DEFAULT IMPLEMENTATIONS
// ============================================================================

impl Default for CanonicalDiscoveryConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(10),
            max_services_per_capability: 10,
            service_ttl: Duration::from_secs(300),
        }
    }
}

impl Default for CanonicalLoadBalancingConfig {
    fn default() -> Self {
        Self {
            strategy: CanonicalLoadBalancingStrategy::HealthAware,
            health_weight: 0.4,
            performance_weight: 0.4,
            availability_weight: 0.2,
        }
    }
}

impl Default for CanonicalCircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(60),
            reset_timeout: Duration::from_secs(30),
        }
    }
}

impl Default for CanonicalRetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }
}

impl Default for CanonicalTimeoutConfig {
    fn default() -> Self {
        Self {
            request_timeout: Duration::from_secs(30),
            connection_timeout: Duration::from_secs(10),
            health_check_timeout: Duration::from_secs(5),
            discovery_timeout: Duration::from_secs(10),
        }
    }
}

impl Default for CanonicalHealthCheckConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            unhealthy_threshold: 3,
            healthy_threshold: 2,
        }
    }
}

impl Default for CanonicalMonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            collection_interval: Duration::from_secs(60),
            retention_period: Duration::from_secs(3600),
            history_size: 1000,
        }
    }
}

impl Default for CanonicalServicePerformance {
    fn default() -> Self {
        Self {
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
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            avg_response_time: Duration::from_millis(0),
            requests_by_capability: HashMap::new(),
            requests_by_service_type: HashMap::new(),
            circuit_breaker_activations: 0,
            load_balancing_decisions: HashMap::new(),
        }
    }
}
