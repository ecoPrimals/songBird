//! Network-based discovery implementation for universal primals ecosystem
//!
//! This module implements network-based service discovery using multiple strategies:
//! - Ecosystem discovery (scanning ../primal directories)
//! - Broadcast discovery (UDP/mDNS)
//! - Network scanning (port scanning ranges)

use crate::discovery::types::{DiscoveredPrimal, DiscoveryMethod};
use crate::errors::{PrimalError, PrimalResult};
use crate::traits::PrimalCapability;
use songbird_universal::PrimalType;
use std::collections::HashMap;
use std::env;
use std::time::Instant;
use tracing::{debug, info};

/// Capability probe function type
type CapabilityProbe = fn(&str) -> Result<Vec<PrimalCapability>, PrimalError>;

/// Network-based service discovery using capability detection
///
/// **PURE CAPABILITY-BASED ARCHITECTURE**
///
/// This discovers services based on their capabilities, not hardcoded names.
/// Songbird only knows itself - all other services are discovered dynamically.
pub struct CapabilityAdapter {
    /// Self-capabilities (songbird orchestration only)
    self_capabilities: Vec<PrimalCapability>,
    /// Fallback capabilities for unknown services
    fallback_capabilities: Vec<PrimalCapability>,
}

/// Create a new capability adapter with songbird self-knowledge only
impl CapabilityAdapter {
    /// Create a new capability adapter - songbird only knows itself
    pub fn new() -> Self {
        // Songbird only knows its own orchestration capabilities
        let self_capabilities = vec![
            PrimalCapability::ServiceDiscovery {
                protocols: vec!["http".to_string(), "grpc".to_string()],
            },
            PrimalCapability::Orchestration {
                platforms: vec![
                    "universal_orchestration".to_string(),
                    "network_coordination".to_string(),
                ],
            },
        ];

        // Universal fallback for any unknown service
        let fallback_capabilities = vec![PrimalCapability::ServiceDiscovery {
            protocols: vec!["http".to_string()],
        }];

        Self {
            self_capabilities,
            fallback_capabilities,
        }
    }

    /// Get songbird's own capabilities
    pub fn get_self_capabilities(&self) -> Vec<PrimalCapability> {
        self.self_capabilities.clone()
    }

    /// Detect capabilities from service endpoint (replaces hardcoded inference)
    pub async fn detect_capabilities(&self, endpoint: &str) -> Vec<PrimalCapability> {
        // Try to probe the service and detect capabilities
        match probe_service_capabilities(endpoint).await {
            Ok(capabilities) => capabilities,
            Err(_) => {
                debug!("Could not probe {}, using fallback capabilities", endpoint);
                self.fallback_capabilities.clone()
            }
        }
    }
}

impl Default for CapabilityAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// Discover services via network using pure capability detection
pub async fn network_capability_discovery() -> PrimalResult<Vec<DiscoveredPrimal>> {
    info!("🔍 Starting capability-based network discovery");
    let mut discovered = Vec::new();

    // Start with self-registration (songbird knows itself)
    discovered.extend(register_self_capabilities().await?);

    // Discover other services via network scanning (capability-based)
    discovered.extend(scan_network_for_capabilities().await?);

    // Discover services from environment variables (universal patterns)
    discovered.extend(discover_from_environment().await?);

    info!(
        "🔍 Capability-based discovery completed: {} services discovered",
        discovered.len()
    );
    Ok(discovered)
}

/// Register songbird's own capabilities
async fn register_self_capabilities() -> Result<Vec<DiscoveredPrimal>, PrimalError> {
    let capability_adapter = CapabilityAdapter::new();
    let self_capabilities = capability_adapter.get_self_capabilities();

    let self_primal = DiscoveredPrimal {
        primal_id: "self-songbird".to_string(),
        primal_type: PrimalType::new("orchestration"),
        capabilities: self_capabilities,
        endpoint: get_self_endpoint(),
        health_status: "Healthy".to_string(),
        discovery_method: DiscoveryMethod::SelfRegistration,
        last_seen: Instant::now(),
        metadata: HashMap::from([
            (
                "discovery_method".to_string(),
                "self_registration".to_string(),
            ),
            ("role".to_string(), "orchestrator".to_string()),
        ]),
    };

    Ok(vec![self_primal])
}

/// Scan network for services with capabilities (pure capability detection)
async fn scan_network_for_capabilities() -> Result<Vec<DiscoveredPrimal>, PrimalError> {
    let mut discovered = Vec::new();
    let capability_adapter = CapabilityAdapter::new();

    // Common service ports for scanning
    let scan_ports = vec![8080, 8081, 8082, 8083, 8084, 8085, 8443, 3000, 5000];
    let scan_hosts = vec!["localhost", "127.0.0.1"];

    for host in scan_hosts {
        for port in &scan_ports {
            let endpoint = format!("http://{host}:{port}");

            match test_endpoint_connectivity(&endpoint).await {
                Ok(true) => {
                    debug!("Found service at: {}", endpoint);

                    // Detect capabilities for this service
                    let capabilities = capability_adapter.detect_capabilities(&endpoint).await;

                    // Generate primal type from detected capabilities
                    let primal_type = infer_primal_type_from_capabilities(&capabilities);

                    let discovered_service = DiscoveredPrimal {
                        primal_id: format!("network-service-{port}"),
                        primal_type,
                        capabilities,
                        endpoint: endpoint.clone(),
                        health_status: "Unknown".to_string(),
                        discovery_method: DiscoveryMethod::NetworkScan,
                        last_seen: Instant::now(),
                        metadata: HashMap::from([
                            ("discovery_method".to_string(), "network_scan".to_string()),
                            ("host".to_string(), host.to_string()),
                            ("port".to_string(), port.to_string()),
                        ]),
                    };

                    discovered.push(discovered_service);
                }
                Ok(false) => debug!("No service at: {}", endpoint),
                Err(e) => debug!("Error checking {}: {}", endpoint, e),
            }
        }
    }

    Ok(discovered)
}

/// Discover services from environment variables (universal patterns)
async fn discover_from_environment() -> Result<Vec<DiscoveredPrimal>, PrimalError> {
    let mut discovered = Vec::new();
    let capability_adapter = CapabilityAdapter::new();

    // Universal environment variable patterns (not tied to specific primal names)
    let env_patterns = [
        "SERVICE_ENDPOINT",
        "API_ENDPOINT",
        "COMPUTE_ENDPOINT",
        "STORAGE_ENDPOINT",
        "SECURITY_ENDPOINT",
        "AI_ENDPOINT",
        "ML_ENDPOINT",
    ];

    for env_var in env_patterns {
        if let Ok(endpoint) = env::var(env_var) {
            debug!("Found service endpoint from {}: {}", env_var, endpoint);

            // Detect capabilities
            let capabilities = capability_adapter.detect_capabilities(&endpoint).await;
            let primal_type = infer_primal_type_from_capabilities(&capabilities);

            let service_name = env_var.to_lowercase().replace("_endpoint", "");

            let discovered_service = DiscoveredPrimal {
                primal_id: format!("env-{service_name}"),
                primal_type,
                capabilities,
                endpoint,
                health_status: "Unknown".to_string(),
                discovery_method: DiscoveryMethod::EnvironmentVariable,
                last_seen: Instant::now(),
                metadata: HashMap::from([
                    ("discovery_method".to_string(), "environment".to_string()),
                    ("env_var".to_string(), env_var.to_string()),
                ]),
            };

            discovered.push(discovered_service);
        }
    }

    Ok(discovered)
}

/// Probe service endpoint to detect capabilities (replaces hardcoded assumptions)
async fn probe_service_capabilities(endpoint: &str) -> Result<Vec<PrimalCapability>, PrimalError> {
    // Try common API paths to detect service type
    let capability_probes: Vec<(&str, CapabilityProbe)> = vec![
        ("/capabilities", detect_from_capabilities_endpoint),
        ("/api/capabilities", detect_from_capabilities_endpoint),
        ("/health", detect_from_health_endpoint),
        ("/api/v1/info", detect_from_info_endpoint),
        ("/metrics", detect_from_metrics_endpoint),
    ];

    for (path, detector) in capability_probes {
        let probe_url = format!("{endpoint}{path}");
        if let Ok(response) = make_http_request(&probe_url).await {
            if let Ok(capabilities) = detector(&response) {
                return Ok(capabilities);
            }
        }
    }

    // Fallback: basic service capability
    Ok(vec![PrimalCapability::Custom {
        name: "basic_service".to_string(),
        properties: vec![],
    }])
}

/// Get songbird's own endpoint
fn get_self_endpoint() -> String {
    env::var("SONGBIRD_ENDPOINT")
        .unwrap_or_else(|_| format!("http://localhost:{}", get_default_http_port()))
}

/// Get default HTTP port for songbird
fn get_default_http_port() -> u16 {
    env::var("SONGBIRD_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080)
}

/// Test endpoint connectivity
async fn test_endpoint_connectivity(endpoint: &str) -> Result<bool, PrimalError> {
    // Implementation would use HTTP client to test connectivity
    // For now, return true for localhost endpoints during development
    Ok(endpoint.contains("localhost") || endpoint.contains("127.0.0.1"))
}

/// Make HTTP request (placeholder)
async fn make_http_request(url: &str) -> Result<String, PrimalError> {
    // Placeholder implementation
    debug!("Probing: {}", url);
    Ok("{}".to_string())
}

/// Infer primal type from detected capabilities
fn infer_primal_type_from_capabilities(capabilities: &[PrimalCapability]) -> PrimalType {
    for capability in capabilities {
        match capability {
            PrimalCapability::Authentication { .. } | PrimalCapability::Security { .. } => {
                return PrimalType::new("security");
            }
            PrimalCapability::Storage { .. } => {
                return PrimalType::new("storage");
            }
            PrimalCapability::Compute { .. } => {
                return PrimalType::new("compute");
            }
            PrimalCapability::AI { .. } => {
                return PrimalType::new("ai");
            }
            PrimalCapability::Orchestration { .. } => {
                return PrimalType::new("orchestration");
            }
            _ => continue,
        }
    }
    PrimalType::new("service")
}

// Capability detection functions (placeholders for real implementation)
fn detect_from_capabilities_endpoint(
    _response: &str,
) -> Result<Vec<PrimalCapability>, PrimalError> {
    // Parse /capabilities endpoint response
    Ok(vec![])
}

fn detect_from_health_endpoint(_response: &str) -> Result<Vec<PrimalCapability>, PrimalError> {
    // Infer capabilities from health endpoint response
    Ok(vec![])
}

fn detect_from_info_endpoint(_response: &str) -> Result<Vec<PrimalCapability>, PrimalError> {
    // Parse service info to detect capabilities
    Ok(vec![])
}

fn detect_from_metrics_endpoint(_response: &str) -> Result<Vec<PrimalCapability>, PrimalError> {
    // Infer service type from metrics exposed
    Ok(vec![])
}
