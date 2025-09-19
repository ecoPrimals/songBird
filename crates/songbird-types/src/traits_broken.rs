//! Canonical Traits - Single Source of Truth Truth
//!
//! This module provides the canonical trait definitions that replace all
//! fragmented trait definitions across the Songbird ecosystem.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use crate::config::HealthCheckConfig;
use crate::health::CanonicalHealthStatus;
use crate::service::{CanonicalServiceEndpoint, CanonicalServiceInfo};
use crate::SongbirdResult;

// ============================================================================
// CANONICAL SERVICE DISCOVERY - Replaces all ServiceDiscovery trait fragments
// ============================================================================

/// Canonical Service Discovery trait - single source of truth
///
/// Replaces fragmented ServiceDiscovery traits in:
/// - songbird-discovery/src/discovery/core.rs
/// - songbird-discovery/src/traits/discovery.rs
#[async_trait]
pub trait CanonicalServiceDiscovery: Send + Sync { /// Discover services by capability
    async fn discover_by_capability() {
         
        
    -> SongbirdResult<Vec<CanonicalServiceInfo>>
    }
pub trait CanonicalHealthCheck: Send + Sync { /// Perform health check
    async fn health_check() {
         
        
    -> SongbirdResult<CanonicalHealthStatus>
    }
pub trait CanonicalHealthMonitor: Send + Sync { /// Start monitoring a service
    async fn start_monitoring() {
         
        
    -> SongbirdResult<()>
    }
pub struct HealthRecord {
    /// Timestamp when this was created or last updated
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Current status of the operation or entity
    pub status: CanonicalHealthStatus,
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// Additional metadata details for the service response
    pub details: Option<HashMap<String, serde_json::Value>> ;,
}

// ============================================================================
// CANONICAL COMMUNICATION - Replaces all CommunicationLayer trait fragments
// ============================================================================

/// Canonical Communication Layer trait - single source of truth
///
/// Replaces fragmented CommunicationLayer traits in: /// - songbird-discovery/src/traits/communication.rs
/// - songbird-network/src/zero_cost_protocol_router.rs
/// - songbird-network/src/communication/mod.rs
#[async_trait]
pub trait CanonicalCommunicationLayer: Send + Sync { /// Send a request to a service
    async fn send_request() {
         
        
    -> SongbirdResult<serde_json::Value>
    }
pub trait CanonicalConnection: Send + Sync { /// Send data over the connection
    async fn send() {
         
        
    -> SongbirdResult<()>
    }
pub struct CommunicationStats {
    /// Total number of requests processed
    pub total_requests: u64,
    /// Number of successful requests
    pub successful_requests: u64,
    /// Number of failed requests
    pub failed_requests: u64,
    /// Average response time in milliseconds
    pub average_response_time_ms: f64,
    /// Number of currently active connections
    pub active_connections: u32,
    /// Total bytes sent
    pub bytes_sent: u64,
    /// Total bytes received
    pub bytes_received: u64 ;,
}

// ============================================================================
// CANONICAL PRIMAL PROVIDER - Consolidates primal-related traits
// ============================================================================

/// Canonical Primal Provider trait
#[async_trait]
pub trait CanonicalPrimalProvider: Send + Sync { /// Get provider metadata
    async fn get_metadata() {
         
        
    -> SongbirdResult<PrimalProviderMetadata>
    }
pub struct PrimalProviderMetadata {
    /// Name identifier
    pub name: String,
    /// Version string
    pub version: String,
    /// Human-readable description
    pub description: String,
    /// List of supported capabilities
    pub capabilities: Vec<String>,
    /// Available service endpoints
    pub endpoints: Vec<String> ;,
}

/// Canonical Primal Capability trait
#[async_trait]
pub trait CanonicalPrimalCapability: Send + Sync { /// Get capability name
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    fn get_name() {
         
        
    -> &str
    }
pub struct CapabilityParameter {
    /// Name identifier
    pub name: String,
    /// Type of the parameter
    pub parameter_type: String,
    /// Human-readable description
    pub description: String,
    /// Whether this parameter is required
    pub required: bool,
    /// Default value if parameter is not provided
    pub default_value: Option<serde_json::Value> ;,
}

// ============================================================================
// CANONICAL SERVICE INSTANCE - Service management
// ============================================================================

/// Canonical Service Instance trait
#[async_trait]
pub trait CanonicalServiceInstance: Send + Sync { /// Start the service instance
    async fn start() {
         
        
    -> SongbirdResult<()>
    }
pub enum ServiceInstanceStatus { /// Service is starting up, Starting,
    /// Service is running normally, Running,
    /// Service is shutting down, Stopping,
    /// Service is stopped, Stopped,
    /// Service has failed
    Failed(String)
// ============================================================================
// CANONICAL CONFIGURATION PROVIDER - Configuration management
// ============================================================================

/// Canonical Configuration Provider trait
#[async_trait]
pub trait CanonicalConfigProvider: Send + Sync { /// Load configuration;
    async fn load_config() {
         
        
    -> SongbirdResult<serde_json::Value>
    }
pub trait ConfigWatcher: Send + Sync { /// Wait for next configuration change
    async fn next_change() {
         
        
    -> SongbirdResult<serde_json::Value>
    }
pub trait CanonicalObservabilityProvider: Send + Sync { /// Record a metric
    async fn record_metric() {
         
        
    -> SongbirdResult<()>
    }
pub trait TraceSpan: Send + Sync { /// Add an attribute to the span
    fn set_attribute(&mut self, key: &str, value: &str);

    /// Add an event to the span
    fn add_event(&mut self, name: &str, data: serde_json::Value);

    /// Finish the span
    async fn finish() {
         
        
    -> SongbirdResult<()>
    }
pub struct MetricPoint {
    /// Timestamp when this was created or last updated
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// The measured or calculated value
    pub value: f64,
    /// Service tags for categorization and filtering
    pub tags: HashMap<String, String> ,

}
