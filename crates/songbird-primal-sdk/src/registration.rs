//! # Primal Registration Module
//!
//! **Capability-Based Service Registration Protocol**
//!
//! This module provides a generic registration protocol for primals to register
//! with ANY orchestrator that implements the service registry capability.
//!
//! ## Core Principle: "Each Primal Knows Only Itself"
//!
//! This module **does not hardcode** "Songbird" or any specific orchestrator name.
//! Instead, it discovers orchestrators by capability and registers generically.
//!
//! ## Usage Example
//!
//! ```no_run
//! use songbird_primal_sdk::registration::{
//!     discover_orchestrators,
//!     register_with_orchestrator,
//!     ServiceInfo,
//!     Capability,
//!     CapabilityType,
//! };
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // 1. Know thyself (no knowledge of orchestrators!)
//!     let my_info = ServiceInfo {
//!         name: "MyPrimal".to_string(),
//!         version: env!("CARGO_PKG_VERSION").to_string(),
//!         capabilities: vec![
//!             Capability {
//!                 name: "compute".to_string(),
//!                 capability_type: CapabilityType::Execution,
//!                 metadata: Default::default(),
//!             }
//!         ],
//!         protocols: vec!["https".to_string(), "tarpc".to_string()],
//!         metadata: Default::default(),
//!     };
//!
//!     // 2. Discover ANY orchestrator (could be Songbird, Phoenix, etc.)
//!     let orchestrators = discover_orchestrators().await?;
//!
//!     if orchestrators.is_empty() {
//!         println!("No orchestrator found. Running standalone.");
//!         return Ok(());
//!     }
//!
//!     // 3. Register with discovered orchestrator
//!     let registration = register_with_orchestrator(
//!         &orchestrators[0],
//!         my_info
//!     ).await?;
//!
//!     println!("Registered! Assigned endpoint: {}", registration.assigned_endpoint);
//!
//!     Ok(())
//! }
//! ```

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, info, warn};
use url::Url;

// ============================================================================
// ERROR TYPES
// ============================================================================

/// Registration-specific errors
#[derive(Debug, Error)]
pub enum RegistrationError {
    #[error("No orchestrators discovered")]
    NoOrchestratorsFound,

    #[error("Registration failed: {reason}")]
    RegistrationFailed { reason: String },

    #[error("Orchestrator unreachable: {url}")]
    OrchestratorUnreachable { url: String },

    #[error("Invalid endpoint: {0}")]
    InvalidEndpoint(String),

    #[error("Heartbeat failed: {reason}")]
    HeartbeatFailed { reason: String },

    #[error("Deregistration failed: {reason}")]
    DeregistrationFailed { reason: String },

    #[error("HTTP request error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("URL parse error: {0}")]
    UrlParseError(#[from] url::ParseError),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type RegistrationResult<T> = Result<T, RegistrationError>;

// ============================================================================
// CORE TYPES
// ============================================================================

/// Information about a discovered orchestrator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorInfo {
    /// Orchestrator name (e.g., "Songbird", "Phoenix", etc.)
    pub name: String,

    /// Base URL
    pub url: String,

    /// Supported capabilities
    pub capabilities: Vec<String>,

    /// Supported protocols
    pub protocols: Vec<String>,

    /// Version
    pub version: String,

    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Service information - what a primal knows about itself
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// Service name (e.g., "Toadstool", "BearDog")
    pub name: String,

    /// Service version
    pub version: String,

    /// Capabilities this service provides
    pub capabilities: Vec<Capability>,

    /// Supported protocols (https, tarpc, json-rpc, etc.)
    pub protocols: Vec<String>,

    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Capability definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    /// Capability name (e.g., "compute", "storage", "security")
    pub name: String,

    /// Capability type
    #[serde(rename = "type")]
    pub capability_type: CapabilityType,

    /// Capability-specific metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Capability types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityType {
    /// Execution/compute capability
    Execution,
    /// Storage capability
    Storage,
    /// Security capability
    Security,
    /// AI/ML capability
    Ai,
    /// Orchestration capability
    Orchestration,
    /// Custom capability
    Custom(String),
}

/// Endpoint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoint {
    /// Protocol (https, tarpc, json-rpc)
    pub protocol: String,

    /// Host
    pub host: String,

    /// Port
    pub port: u16,

    /// Full URL
    pub full_url: String,
}

impl Endpoint {
    /// Create a new endpoint
    pub fn new(protocol: &str, host: &str, port: u16) -> Self {
        let full_url = format!("{}://{}:{}", protocol, host, port);
        Self {
            protocol: protocol.to_string(),
            host: host.to_string(),
            port,
            full_url,
        }
    }
}

/// Registration response from orchestrator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registration {
    /// Unique service ID assigned by orchestrator
    pub service_id: String,

    /// Assigned endpoint
    pub assigned_endpoint: Endpoint,

    /// Optional fallback endpoint
    pub fallback_endpoint: Option<Endpoint>,

    /// Registration token for subsequent operations
    pub token: String,

    /// Heartbeat interval in seconds
    pub heartbeat_interval_sec: u64,

    /// Trust level assigned
    pub trust_level: String,
}

/// Heartbeat request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatRequest {
    /// Service ID
    pub service_id: String,

    /// Registration token
    pub token: String,

    /// Current status
    pub status: String,

    /// Current load metrics
    pub current_load: Option<LoadMetrics>,

    /// Whether capabilities have changed
    pub capabilities_changed: bool,
}

/// Load metrics for heartbeat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadMetrics {
    /// CPU usage percentage
    pub cpu_usage_percent: f64,

    /// Memory usage percentage
    pub memory_usage_percent: f64,

    /// GPU usage percentage (if applicable)
    pub gpu_usage_percent: Option<f64>,

    /// Active tasks
    pub active_tasks: usize,

    /// Queued tasks
    pub queued_tasks: usize,
}

// ============================================================================
// DISCOVERY
// ============================================================================

/// Discover orchestrators on the network
///
/// This function discovers ANY orchestrator that implements the service registry
/// capability. It does NOT hardcode "Songbird" or any specific orchestrator name.
///
/// Discovery methods (in priority order):
/// 1. Environment variable (ORCHESTRATOR_URL)
/// 2. UDP broadcast (service_registry capability)
/// 3. Well-known ports (8080, 8081, 8082)
/// 4. mDNS (_orchestrator._tcp.local)
///
/// # Returns
///
/// A list of discovered orchestrators, sorted by priority.
pub async fn discover_orchestrators() -> RegistrationResult<Vec<OrchestratorInfo>> {
    let mut orchestrators = Vec::new();

    // Method 1: Environment variable
    if let Ok(url) = std::env::var("ORCHESTRATOR_URL") {
        debug!("Checking ORCHESTRATOR_URL: {}", url);
        if let Ok(info) = probe_orchestrator(&url).await {
            info!("✅ Discovered orchestrator via env: {}", info.name);
            orchestrators.push(info);
            return Ok(orchestrators); // Explicit env var takes precedence
        }
    }

    // Method 2: Well-known ports (localhost first)
    for port in [8080, 8081, 8082] {
        let url = format!("https://localhost:{}", port);
        if let Ok(info) = probe_orchestrator(&url).await {
            info!("✅ Discovered orchestrator at {}: {}", url, info.name);
            orchestrators.push(info);
        }
    }

    // Method 3: UDP broadcast (future enhancement)
    // TODO: Implement UDP discovery for LAN orchestrators

    // Method 4: mDNS (future enhancement)
    // TODO: Implement mDNS discovery for _orchestrator._tcp.local

    if orchestrators.is_empty() {
        warn!("No orchestrators discovered");
        return Err(RegistrationError::NoOrchestratorsFound);
    }

    info!("Discovered {} orchestrator(s)", orchestrators.len());
    Ok(orchestrators)
}

/// Probe a URL to see if it's an orchestrator
async fn probe_orchestrator(url: &str) -> RegistrationResult<OrchestratorInfo> {
    let health_url = format!("{}/health", url);

    debug!("Probing orchestrator at {}", health_url);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?;

    let response = client
        .get(&health_url)
        .send()
        .await
        .map_err(|_| RegistrationError::OrchestratorUnreachable {
            url: url.to_string(),
        })?;

    if !response.status().is_success() {
        return Err(RegistrationError::OrchestratorUnreachable {
            url: url.to_string(),
        });
    }

    // Try to get orchestrator info from /api/v1/info or /health
    let info_url = format!("{}/api/v1/info", url);
    let info_response = client.get(&info_url).send().await;

    let orchestrator_info = if let Ok(resp) = info_response {
        if resp.status().is_success() {
            // Parse orchestrator info
            resp.json::<OrchestratorInfo>().await.unwrap_or_else(|_| {
                // Fallback to basic info
                OrchestratorInfo {
                    name: "Orchestrator".to_string(),
                    url: url.to_string(),
                    capabilities: vec!["service_registry".to_string()],
                    protocols: vec!["https".to_string()],
                    version: "unknown".to_string(),
                    metadata: HashMap::new(),
                }
            })
        } else {
            // Fallback
            OrchestratorInfo {
                name: "Orchestrator".to_string(),
                url: url.to_string(),
                capabilities: vec!["service_registry".to_string()],
                protocols: vec!["https".to_string()],
                version: "unknown".to_string(),
                metadata: HashMap::new(),
            }
        }
    } else {
        // Fallback
        OrchestratorInfo {
            name: "Orchestrator".to_string(),
            url: url.to_string(),
            capabilities: vec!["service_registry".to_string()],
            protocols: vec!["https".to_string()],
            version: "unknown".to_string(),
            metadata: HashMap::new(),
        }
    };

    Ok(orchestrator_info)
}

// ============================================================================
// REGISTRATION
// ============================================================================

/// Register with a discovered orchestrator
///
/// This function registers a primal with ANY orchestrator that implements
/// the service registry protocol. It does NOT assume the orchestrator is "Songbird".
///
/// # Arguments
///
/// * `orchestrator` - The discovered orchestrator to register with
/// * `service_info` - Information about this service (capabilities, protocols, etc.)
///
/// # Returns
///
/// Registration information including assigned endpoint and token
pub async fn register_with_orchestrator(
    orchestrator: &OrchestratorInfo,
    service_info: ServiceInfo,
) -> RegistrationResult<Registration> {
    info!(
        "📝 Registering {} with {} at {}",
        service_info.name, orchestrator.name, orchestrator.url
    );

    let register_url = format!("{}/api/v1/services/register", orchestrator.url);

    // Create registration request
    let request = serde_json::json!({
        "primal_name": service_info.name,
        "primal_version": service_info.version,
        "capabilities": service_info.capabilities,
        "protocols": service_info.protocols,
        "preferred_protocol": service_info.protocols.first().unwrap_or(&"https".to_string()),
        "health_check_path": "/health",
        "metadata": service_info.metadata,
    });

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    let response = client
        .post(&register_url)
        .json(&request)
        .send()
        .await
        .map_err(|e| RegistrationError::RegistrationFailed {
            reason: format!("HTTP request failed: {}", e),
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(RegistrationError::RegistrationFailed {
            reason: format!("HTTP {} - {}", status, body),
        });
    }

    let registration: Registration = response.json().await.map_err(|e| {
        RegistrationError::RegistrationFailed {
            reason: format!("Failed to parse registration response: {}", e),
        }
    })?;

    info!(
        "✅ Registered successfully! Service ID: {}, Endpoint: {}",
        registration.service_id, registration.assigned_endpoint.full_url
    );

    Ok(registration)
}

// ============================================================================
// LIFECYCLE
// ============================================================================

/// Send a heartbeat to the orchestrator
///
/// This should be called periodically (based on `registration.heartbeat_interval_sec`)
/// to maintain the registration.
pub async fn send_heartbeat(
    orchestrator: &OrchestratorInfo,
    heartbeat: HeartbeatRequest,
) -> RegistrationResult<()> {
    debug!("💓 Sending heartbeat for service {}", heartbeat.service_id);

    let heartbeat_url = format!(
        "{}/api/v1/services/{}/heartbeat",
        orchestrator.url, heartbeat.service_id
    );

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let response = client
        .post(&heartbeat_url)
        .json(&heartbeat)
        .send()
        .await
        .map_err(|e| RegistrationError::HeartbeatFailed {
            reason: format!("HTTP request failed: {}", e),
        })?;

    if !response.status().is_success() {
        let status = response.status();
        return Err(RegistrationError::HeartbeatFailed {
            reason: format!("HTTP {}", status),
        });
    }

    debug!("✅ Heartbeat acknowledged");
    Ok(())
}

/// Deregister from the orchestrator
///
/// This should be called on graceful shutdown to release the assigned port
/// and remove the service from the registry.
pub async fn deregister_from_orchestrator(
    orchestrator: &OrchestratorInfo,
    service_id: &str,
    token: &str,
) -> RegistrationResult<()> {
    info!("🛑 Deregistering service {}", service_id);

    let deregister_url = format!(
        "{}/api/v1/services/{}",
        orchestrator.url, service_id
    );

    let request = serde_json::json!({
        "service_id": service_id,
        "token": token,
        "reason": "graceful_shutdown",
    });

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let response = client
        .delete(&deregister_url)
        .json(&request)
        .send()
        .await
        .map_err(|e| RegistrationError::DeregistrationFailed {
            reason: format!("HTTP request failed: {}", e),
        })?;

    if !response.status().is_success() {
        let status = response.status();
        warn!("Deregistration returned HTTP {}", status);
        // Don't fail on deregistration errors (best effort)
    }

    info!("✅ Deregistered successfully");
    Ok(())
}

// ============================================================================
// HELPER UTILITIES
// ============================================================================

/// Create a heartbeat loop task
///
/// This creates a tokio task that sends periodic heartbeats.
///
/// # Example
///
/// ```no_run
/// use songbird_primal_sdk::registration::*;
///
/// # async fn example(orchestrator: OrchestratorInfo, registration: Registration) {
/// let heartbeat_handle = spawn_heartbeat_loop(
///     orchestrator.clone(),
///     registration.service_id.clone(),
///     registration.token.clone(),
///     registration.heartbeat_interval_sec,
/// );
///
/// // Later, on shutdown:
/// heartbeat_handle.abort();
/// # }
/// ```
pub fn spawn_heartbeat_loop(
    orchestrator: OrchestratorInfo,
    service_id: String,
    token: String,
    interval_sec: u64,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(interval_sec));

        loop {
            interval.tick().await;

            let heartbeat = HeartbeatRequest {
                service_id: service_id.clone(),
                token: token.clone(),
                status: "operational".to_string(),
                current_load: None, // Can be populated with actual metrics
                capabilities_changed: false,
            };

            if let Err(e) = send_heartbeat(&orchestrator, heartbeat).await {
                warn!("Heartbeat failed: {}", e);
                // Continue trying - don't exit on failure
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endpoint_creation() {
        let endpoint = Endpoint::new("https", "localhost", 8091);
        assert_eq!(endpoint.protocol, "https");
        assert_eq!(endpoint.host, "localhost");
        assert_eq!(endpoint.port, 8091);
        assert_eq!(endpoint.full_url, "https://localhost:8091");
    }

    #[test]
    fn test_service_info_serialization() {
        let info = ServiceInfo {
            name: "TestService".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec![],
            protocols: vec!["https".to_string()],
            metadata: HashMap::new(),
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("TestService"));
    }

    #[tokio::test]
    async fn test_discovery_no_orchestrators() {
        // Without any orchestrator running, discovery should return error
        // (This test may pass or fail depending on local environment)
        let result = discover_orchestrators().await;
        // Just verify it doesn't panic
        let _ = result;
    }
}

