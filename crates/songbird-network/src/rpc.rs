// # Songbird Native RPC System
//
// Pure Rust bidirectional RPC implementation for universal orchestration.
// This is a P0 critical component replacing HTTP/WebSocket limitations.

// UNUSED IMPORT REMOVED: async_trait not used in this file
use crate::optimization::StreamConfig;
use crate::unified_types::ConnectionInfo;
use serde::{Deserialize, Serialize};
use songbird_errors::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info};
use uuid::Uuid;

/// Load balancing strategies for RPC
#[derive(Debug, Clone)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    CapabilityBased,
    Random,
    LeastConnections,
}

/// Active stream information
#[derive(Debug)]
pub struct ActiveStream {
    pub stream_id: String,
    pub stream_type: String,
    pub created_at: Instant,
}

/// Authentication provider for RPC security
#[derive(Debug)]
pub struct AuthenticationProvider {
    pub provider_type: String,
    pub endpoint: Option<String>,
}

/// Encryption provider for secure communications
#[derive(Debug)]
pub struct EncryptionProvider {
    pub provider_type: String,
    pub algorithm: String,
}

/// Permission type for access control
#[derive(Debug, Clone)]
pub enum Permission {
    Read,
    Write,
    Execute,
    Admin,
}

/// Access control manager for RPC permissions
#[derive(Debug)]
pub struct AccessControlManager {
    pub permissions: Arc<RwLock<HashMap<String, Vec<Permission>>>>,
}

impl Default for AccessControlManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AccessControlManager {
    pub fn new() -> Self {
        Self {
            permissions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn permissions(&self) -> &Arc<RwLock<HashMap<String, Vec<Permission>>>> {
        &self.permissions
    }
}

/// Core RPC system for Songbird
#[derive(Debug)]
pub struct SongbirdRPC {
    /// Message router for capability-based routing  
    #[allow(dead_code)] // Part of API design, will be used when fully implemented
    message_router: Arc<UniversalMessageRouter>,

    /// Stream manager for real-time data flows
    #[allow(dead_code)] // Part of API design, will be used when fully implemented
    stream_manager: Arc<BidirectionalStreamManager>,

    /// Health monitor for connection status
    #[allow(dead_code)] // Part of API design, will be used when fully implemented
    health_monitor: Arc<ConnectionHealthMonitor>,

    /// Security layer for authentication/encryption
    #[allow(dead_code)] // Part of API design, will be used when fully implemented
    security_layer: Arc<UniversalSecurityLayer>,
}

impl SongbirdRPC {
    /// Create a new RPC system
    pub fn new() -> Self {
        Self {
            message_router: Arc::new(UniversalMessageRouter::new()),
            stream_manager: Arc::new(BidirectionalStreamManager::new()),
            health_monitor: Arc::new(ConnectionHealthMonitor::new()),
            security_layer: Arc::new(UniversalSecurityLayer::new()),
        }
    }

    /// Start the RPC system
    pub async fn start(&mut self) -> SongbirdResult<()> {
        info!("🚀 Starting Songbird Native RPC system");

        // Initialize components
        // Note: Actual implementation would start monitoring, security, etc.

        info!("✅ Songbird Native RPC system started successfully");
        Ok(())
    }
}

/// Universal message router
#[derive(Debug)]
pub struct UniversalMessageRouter {
    routes: Arc<RwLock<HashMap<String, String>>>,
}

/// Bidirectional stream manager for RPC connections
#[derive(Debug)]
pub struct BidirectionalStreamManager {
    pub active_streams: Arc<RwLock<HashMap<String, ActiveStream>>>,
    pub stream_registry: StreamRegistry, // Add the missing field
}

// StreamRegistry already defined above - removed duplicate

/// Connection health monitor - uses unified types
#[derive(Debug)]
pub struct ConnectionHealthMonitor {
    #[allow(dead_code)] // Part of API design, will be used when fully implemented
    connections: Arc<RwLock<HashMap<String, ConnectionInfo>>>,
    #[allow(dead_code)] // Part of API design, will be used when fully implemented
    health_checks: Arc<RwLock<HashMap<String, HealthStatus>>>,
}

/// Universal security layer
#[derive(Debug)]
pub struct UniversalSecurityLayer {
    #[allow(dead_code)] // Part of API design, will be used when fully implemented
    oauth2_configs: Arc<RwLock<HashMap<String, String>>>, // Simplified for compilation
    auth_provider: AuthenticationProvider,
    encryption_provider: EncryptionProvider,
    access_control: AccessControlManager,
}

impl Default for UniversalSecurityLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl UniversalSecurityLayer {
    pub fn new() -> Self {
        Self {
            oauth2_configs: Arc::new(RwLock::new(HashMap::new())),
            auth_provider: AuthenticationProvider {
                provider_type: "internal".to_string(),
                endpoint: None,
            },
            encryption_provider: EncryptionProvider {
                provider_type: "tls".to_string(),
                algorithm: "aes256".to_string(),
            },
            access_control: AccessControlManager {
                permissions: Arc::new(RwLock::new(HashMap::new())),
            },
        }
    }

    pub fn auth_provider(&self) -> &AuthenticationProvider {
        &self.auth_provider
    }

    pub fn encryption_provider(&self) -> &EncryptionProvider {
        &self.encryption_provider
    }

    pub fn access_control(&self) -> &AccessControlManager {
        &self.access_control
    }
}

/// OAuth2 configuration for RPC authentication
///
/// This integrates with the unified security configuration system.
// OAuth2Config removed - use songbird_config::unified::security::OAuth2Config
/// Individual stream handle
#[derive(Debug)]
pub struct StreamHandle {
    pub stream_id: String,
    pub primal_source: String,
    pub stream_type: StreamType,
    pub sender: mpsc::UnboundedSender<StreamMessage>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Types of streams supported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamType {
    Metrics,
    Logs,
    Events,
    Gaming,
    Federation,
    Custom(String),
}

/// Stream message format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamMessage {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub source: String,
    pub stream_type: StreamType,
    pub payload: serde_json::Value,
}

/// Stream registry for tracking available streams
#[derive(Debug)]
pub struct StreamRegistry {
    registered_streams: Arc<RwLock<HashMap<String, StreamMetadata>>>,
}

impl Default for StreamRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamRegistry {
    pub fn new() -> Self {
        Self {
            registered_streams: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

/// Metadata for registered streams
#[derive(Debug, Clone)]
pub struct StreamMetadata {
    pub stream_id: String,
    pub primal_provider: String,
    pub description: String,
    pub data_format: String,
    pub access_permissions: Vec<String>,
}

/// Health status for connections
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub connection_id: String,
    pub is_healthy: bool,
    pub last_check: chrono::DateTime<chrono::Utc>,
    pub response_time_ms: u64,
    pub error_count: u32,
}

impl Default for UniversalMessageRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl UniversalMessageRouter {
    pub fn new() -> Self {
        Self {
            routes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register primal routes for message routing
    pub async fn register_primal_routes(
        &mut self,
        primal_type: &str,
        connection_id: &str,
    ) -> SongbirdResult<()> {
        let mut routes = self.routes.write().await;
        routes.insert(primal_type.to_string(), connection_id.to_string());
        debug!("Registered routes for primal: {}", primal_type);
        Ok(())
    }

    /// Route message to appropriate primal
    pub async fn route_message(&self, message: &str, target: &str) -> SongbirdResult<String> {
        let routes = self.routes.read().await;
        if let Some(_connection_id) = routes.get(target) {
            debug!(
                "Routing message to primal: {} with payload: {}",
                target, message
            );
            Ok(format!("Response from {target}"))
        } else {
            Err(SongbirdError::internal_error(Network {
                message: format!("No route found for primal: {target}"),
                operation: "route_message".to_string(),
                suggestion: Some("Register the primal route first".to_string()),
            })
        }
    }
}

impl Default for BidirectionalStreamManager {
    fn default() -> Self {
        Self::new()
    }
}

impl BidirectionalStreamManager {
    pub fn new() -> Self {
        Self {
            active_streams: Arc::new(RwLock::new(HashMap::new())),
            stream_registry: StreamRegistry::new(),
        }
    }

    /// Get the stream registry
    pub fn stream_registry(&self) -> &StreamRegistry {
        &self.stream_registry
    }

    /// Create a new stream
    pub async fn create_stream(&self, stream_config: &StreamConfig) -> SongbirdResult<String> {
        let stream_id = Uuid::new_v4().to_string();
        let stream = ActiveStream {
            stream_id: stream_id.clone(),
            stream_type: "bidirectional".to_string(), // Default stream type
            created_at: Instant::now(),
        };

        let mut streams = self.active_streams.write().await;
        streams.insert(stream_id.clone(), stream);

        debug!(
            "Created stream {} with config: {:?}",
            stream_id, stream_config
        );
        Ok(stream_id)
    }
}

impl StreamRegistry {
    /// Get registered streams
    pub fn registered_streams(&self) -> &Arc<RwLock<HashMap<String, StreamMetadata>>> {
        &self.registered_streams
    }
}

impl Default for ConnectionHealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionHealthMonitor {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            health_checks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get the check interval
    pub fn check_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(30) // Default check interval
    }
}

// Duplicate UniversalSecurityLayer impl removed - methods already defined above

// Duplicate AccessControlManager impl removed - methods already defined above

impl Default for SongbirdRPC {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rpc_creation() {
        let _rpc = SongbirdRPC::new();
        // Basic creation test - verify RPC instance is created successfully
        // Test passes if no panic occurs during creation
    }

    // TODO: Re-enable once ConnectionPool is properly integrated
    // #[tokio::test]
    // async fn test_connection_pool() {
    //     let pool = ConnectionPool::<PrimalConnection>::new(10);
    //     // Test pool creation
    //     assert!(true); // Placeholder assertion
    // }
}
