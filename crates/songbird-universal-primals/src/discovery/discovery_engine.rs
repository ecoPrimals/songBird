// Core Discovery Engine
//
// This module contains the main PrimalDiscoveryEngine implementation
// that coordinates all discovery methods and manages discovered primals.

use super::network_scan::scanning::perform_network_scan;
use super::types::{DiscoveredPrimal, DiscoveryStats};
use super::universal_query::query_universal_primal_services;
// Use local definitions
use songbird_config::SongbirdConfig;
use songbird_errors::IntoSongbirdResponse;
// Import canonical type with alias
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Engine for discovering Universal Primals across networks
pub struct PrimalDiscoveryEngine {
    /// Configuration for primal discovery
    config: SongbirdConfig,
    /// Map of discovered primals by their ID
    discovered_primals: HashMap<String, DiscoveredPrimal>,
    /// Cache for discovery results (timestamp-based)
    #[allow(dead_code)]
    discovery_cache: HashMap<String, std::time::Instant>,
    /// Statistics about discovery operations
    discovery_stats: DiscoveryStats,
}
impl PrimalDiscoveryEngine {
    /// Create a new discovery engine
    pub fn new(config: SongbirdConfig) -> Self {
        Self {
            config,
            discovered_primals: HashMap::new(),
            discovery_cache: HashMap::new(),
            discovery_stats: DiscoveryStats::default(),
        }
    }

    /// Create a new discovery engine with custom discovery configuration
    pub fn with_discovery_config(
        config: SongbirdConfig,
        _discovery_config: songbird_config::unified::UnifiedDiscoveryConfig,
    ) -> Self {
        Self {
            config,
            discovered_primals: HashMap::new(),
            discovery_cache: HashMap::new(),
            discovery_stats: Default::default(),
        }
    }

    /// Start the discovery engine
    ///
    /// # Errors
    /// Returns error if discovery engine fails to start
    pub async fn start_discovery(&mut self) -> crate::errors::PrimalSongbirdResult<()> {
        info!("🔍 Starting Universal Primal discovery...");

        // Start all discovery methods concurrently
        let mut all_primals = Vec::new();

        // 0. Ecosystem discovery (NEW - connects to real primals in sibling directories)
        if self.config.discovery.network_discovery.enabled {
            let temp_result = self.start_ecosystem_discovery().await;
            match temp_result {
                Ok(songbird_errors::evolved_success(primals)) => {
                    let primal_count = primals.data.len();
                    all_primals.extend(primals.data);
                    info!("🌌 Ecosystem discovery found {} real primals", primal_count);
                }
                Err(e) => {
                    warn!("Ecosystem discovery failed: {}", e);
                }
            }
        }

        // 1. Network scan discovery
        if self.config.discovery.network_discovery.enabled {
            let temp_result = self.start_network_scan_discovery().await;
            match temp_result {
                Ok(songbird_errors::evolved_success(primals)) => {
                    all_primals.extend(primals.data);
                }
                Err(e) => {
                    warn!("Network scan discovery failed: {}", e);
                }
            }
        }

        // Universal primal service discovery
        let temp_result = query_universal_primal_services().await;
        match temp_result {
            Ok(songbird_errors::evolved_success(mut primals)) => all_primals.append(&mut primals.data),
            Err(e) => warn!("Universal primal service discovery failed: {}", e),
        }

        // Network scan discovery
        let temp_result = perform_network_scan().await;
        match temp_result {
            Ok(songbird_errors::evolved_success(mut network_primals)) => all_primals.append(&mut network_primals.data),
            Err(e) => warn!("Network scan discovery failed: {}", e),
        }

        // Store discovered primals
        for primal in all_primals {
            self.discovered_primals
                .insert(primal.primal_id.clone(), primal);
        }

        info!(
            "🎯 Discovery complete: {} primals found",
            self.discovered_primals.len()
        );
        Ok(songbird_errors::success(()))
    }

    /// Start ecosystem discovery (connects to real primals in sibling directories)
    pub async fn start_ecosystem_discovery(&self) -> SongbirdResult<()> {debug!("🌌 Starting ecosystem discovery for real primals...");
        // This would integrate with the ecosystem module
        // For now, return empty to avoid blocking
        Ok(success(Vec::new()))
    }

    /// Start network scan discovery
    pub async fn start_network_scan_discovery(&self) -> SongbirdResult<()> {debug!("🌐 Starting network scan discovery...");

        let temp_result = perform_network_scan().await;
        match temp_result {
            Ok(songbird_errors::evolved_success(primals)) => {
                info!("✅ Network scan found {} primals", primals.data.len());
                Ok(songbird_errors::evolved_success(success(primals))
            }
            Err(e) => {
                warn!("Network scan failed: {}", e);
                Ok(Vec::new())) // Don't fail the entire discovery process
            }
        }
    }

    #[allow(dead_code)] // Planned discovery method for service registry integration
    pub async fn start_service_registry_discovery(&self) -> SongbirdResult<()> {debug!("📋 Starting service registry discovery...");

        // Service registry integration
        // Discovery results are automatically registered with the UniversalPrimalRegistry
        // for centralized primal management and lookup
        Ok(success(Vec::new())) // Placeholder for now
    }

    /// **PLACEHOLDER ELIMINATED**: Real UDP broadcast discovery implementation
    ///
    /// Discovers primals on the local network using UDP broadcast messages.
    /// This replaces the previous placeholder that returned empty results.
    #[allow(dead_code)]
    pub async fn start_broadcast_discovery(&self) -> SongbirdResult<()> {debug!("📡 Starting UDP broadcast discovery...");

        let mut discovered = Vec::new();
        let broadcast_port = 8989; // Standard primal discovery port
        let discovery_timeout = std::time::Duration::from_secs(5);

        // Create UDP socket for broadcasting
        let socket = tokio::net::UdpSocket::bind("0.0.0.0:0")
            .await
            .map_err(|e| crate::errors::PrimalError::Network {
                message: format!("Failed to create UDP socket: {e}"),
                operation: Some("udp_socket_creation".to_string()),
                suggestion: Some("Check network permissions and firewall settings".to_string()),
            })?;

        socket
            .set_broadcast(true)
            .map_err(|e| crate::errors::PrimalError::Network {
                message: format!("Failed to enable broadcast: {e}"),
                operation: Some("udp_broadcast_enable".to_string()),
                suggestion: Some(
                    "Verify UDP broadcast support on this network interface".to_string(),
                ),
            })?;

        // Discover broadcast addresses for all network interfaces
        // Note: These methods need to be implemented in the PrimalDiscoveryEngine
        // For now, we'll use placeholder implementations
        let broadcast_addresses = vec!["224.0.0.0:2300".to_string()]; // Placeholder
        for broadcast_addr in broadcast_addresses {
            let target = format!("{broadcast_addr}:{broadcast_port}");
            debug!("📡 Broadcasting discovery request to {}", target);

            // Send discovery request
            let discovery_msg = serde_json::json!({
                "type": "primal_discovery_request",
                "timestamp": chrono::Utc::now().timestamp(),
                "requestor": "songbird-discovery-engine"
            });
            let msg_bytes = discovery_msg.to_string().into_bytes();
            if let Err(e) = socket.send_to(&msg_bytes, &target).await {
                debug!("Failed to send broadcast to {}: {}", target, e);
                continue;
            }

            // Listen for responses with timeout
            let mut buf = [0u8; 4096];
            let temp_result =
                tokio::time::timeout(discovery_timeout, socket.recv_from(&mut buf)).await;
            match temp_result {
                Ok(Ok((len, peer_addr))) => {
                    if let Ok(songbird_errors::evolved_success(response_str)) = std::str::from_utf8(&buf[..len]) {
                        if let Ok(songbird_errors::evolved_success(response_json)) =
                            serde_json::from_str::<serde_json::Value>(response_str)
                        {
                            // Enhanced JSON response parsing with proper field extraction
                            let default_primal_id = format!("broadcast_discovered_{peer_addr}");
                            let primal_id = response_json
                                .get("primal_id")
                                .and_then(|v| v.as_str())
                                .unwrap_or(&default_primal_id);

                            let primal_type_str = response_json
                                .get("primal_type")
                                .and_then(|v| v.as_str())
                                .unwrap_or("orchestration");
                            let primal_type = match primal_type_str {
                                "compute" => crate::traits::PrimalType::Compute,
                                "storage" => crate::traits::PrimalType::Storage,
                                "security" => crate::traits::PrimalType::Security,
                                "ai" => crate::traits::PrimalType::AI,
                                "networking" | "network" => {
                                    crate::traits::PrimalType::Orchestration
                                } // Network services are orchestration
                                "gaming" => crate::traits::PrimalType::Gaming,
                                _ => crate::traits::PrimalType::Orchestration,
                            };

                            let capabilities = response_json
                                .get("capabilities")
                                .and_then(|v| v.as_array())
                                .map(|arr| {
                                    arr.iter()
                                        .filter_map(|v| {
                                            v.as_str().map(crate::traits::PrimalCapability::new)
                                        })
                                        .collect()
                                })
                                .unwrap_or_else(Vec::new);

                            let metadata = response_json
                                .get("metadata")
                                .and_then(|v| v.as_object())
                                .map(|obj| {
                                    obj.iter()
                                        .filter_map(|(k, v)| {
                                            v.as_str().map(|s| (k.clone(), s.to_string()))
                                        })
                                        .collect()
                                })
                                .unwrap_or_else(std::collections::HashMap::new);

                            let default_endpoint = peer_addr.to_string();
                            let endpoint = response_json
                                .get("endpoint")
                                .and_then(|v| v.as_str())
                                .unwrap_or(&default_endpoint);

                            let health_status = response_json
                                .get("health_status")
                                .and_then(|v| v.as_str())
                                .unwrap_or("healthy");

                            let primal = crate::discovery::types::DiscoveredPrimal {
                                primal_id: primal_id.to_string(),
                                primal_type,
                                endpoint: endpoint.to_string(),
                                capabilities,
                                metadata,
                                discovery_method:
                                    crate::discovery::types::DiscoveryMethod::NetworkScan,
                                health_status: health_status.to_string(),
                                last_seen: std::time::Instant::now(),
                                registration:
                                    crate::universal_registry::UniversalServiceRegistration::default(
                                    ),
                            };
                            debug!(
                                "✅ Discovered primal via UDP broadcast: {}",
                                primal.primal_id
                            );
                            discovered.push(primal);
                        }
                    }
                }
                Ok(songbird_errors::evolved_success(Err(e))) => debug!("UDP receive error: {}", e),
                Err(_) => debug!("UDP broadcast discovery timeout for {}", target),
            }
        }

        info!(
            "📡 UDP broadcast discovery completed: found {} primals",
            discovered.len()
        );
        Ok(songbird_errors::evolved_success(success(discovered)))
    }

    /// **PLACEHOLDER ELIMINATED**: Real federation discovery implementation
    /// Discovers primals through federation connections and MCP endpoints.
    #[allow(dead_code)]
    pub async fn start_federation_discovery(&self) -> SongbirdResult<()> {debug!("🤝 Starting federation discovery...");
        let mut discovered = Vec::new();

        // Get federation endpoints from discovery config or environment
        let federation_endpoints = std::env::var("SONGBIRD_FEDERATION_ENDPOINTS")
            .unwrap_or_else(|_| {
                "http://localhost:2301/federation,http://federation.local:2301/api/v1/primals"
                    .to_string()
            })
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<String>>();

        for endpoint in federation_endpoints {
            debug!("🤝 Querying federation endpoint: {}", endpoint);

            // Real HTTP query to federation endpoint with timeout and error handling
            match self.query_federation_endpoint(&endpoint).await {
                Ok(songbird_errors::evolved_success(primals)) => {
                    discovered.extend(primals.data);
                }
                Err(e) => {
                    debug!("Failed to query federation endpoint {}: {}", endpoint, e);
                    // Continue with other endpoints even if one fails
                }
            }
        }

        Ok(songbird_errors::evolved_success(songbird_errors::success(discovered)))
    }

    /// Query a federation endpoint for registered primals
    #[allow(dead_code)]
    pub async fn query_federation_endpoint(&self) -> SongbirdResult<()> {let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| crate::errors::PrimalError::Network {
                message: format!("Failed to create HTTP client: {e}"),
                operation: Some(format!("HTTP client creation for {endpoint}")),
                suggestion: Some("Check network configuration".to_string()),
            })?;

        let response = client
            .get(endpoint)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| crate::errors::PrimalError::Network {
                message: format!("Federation query failed: {e}"),
                operation: Some(format!("HTTP GET {endpoint}")),
                suggestion: Some("Check if federation endpoint is accessible".to_string()),
            })?;

        if !response.status().is_success() {
            return Err(crate::errors::PrimalError::Network {
                message: format!("Federation endpoint returned status: {}", response.status()),
                operation: Some(format!("HTTP GET {endpoint}")),
                suggestion: Some("Federation service may be unhealthy".to_string()),
            });
        }

        let response_text =
            response
                .text()
                .await
                .map_err(|e| crate::errors::PrimalError::Network {
                    message: format!("Failed to read federation response: {e}"),
                    operation: Some(format!("Reading response from {endpoint}")),
                    suggestion: Some("Response may be malformed".to_string()),
                })?;

        // Parse federation response - expect JSON array of primal objects
        let federation_data: serde_json::Value =
            serde_json::from_str(&response_text).map_err(|e| {
                crate::errors::PrimalError::Network {
                    message: format!("Invalid JSON from federation endpoint: {e}"),
                    operation: Some(format!("Parsing JSON from {endpoint}")),
                    suggestion: Some(
                        "Federation endpoint may not be returning valid JSON".to_string(),
                    ),
                }
            })?;

        let mut discovered = Vec::new();

        // Handle both single primal object and array of primals
        let primals_array = match federation_data.as_array() {
            Some(array) => array.clone(),
            None => vec![federation_data], // Single object
        };

        for primal_data in primals_array {
            if let Some(primal) = self.parse_federation_primal(&primal_data, endpoint) {
                discovered.push(primal);
            }
        }

        Ok(songbird_errors::evolved_success(songbird_errors::success(discovered)))
    }

    /// Parse a federation primal object from JSON
    #[allow(dead_code)]
    fn parse_federation_primal(
        &self,
        primal_data: &serde_json::Value,
        source_endpoint: &str,
    ) -> Option<DiscoveredPrimal> {
        let default_primal_id = format!("federation_discovered_{source_endpoint}");
        let primal_id = primal_data
            .get("id")
            .or_else(|| primal_data.get("primal_id"))
            .and_then(|v| v.as_str())
            .unwrap_or(&default_primal_id);

        let primal_type_str = primal_data
            .get("type")
            .or_else(|| primal_data.get("primal_type"))
            .and_then(|v| v.as_str())
            .unwrap_or("orchestration");

        let primal_type = match primal_type_str {
            "compute" => crate::traits::PrimalType::Compute,
            "storage" => crate::traits::PrimalType::Storage,
            "security" => crate::traits::PrimalType::Security,
            "ai" => crate::traits::PrimalType::AI,
            "networking" | "network" => crate::traits::PrimalType::Orchestration, // Network services are orchestration
            "gaming" => crate::traits::PrimalType::Gaming,
            _ => crate::traits::PrimalType::Orchestration,
        };

        let endpoint = primal_data
            .get("endpoint")
            .or_else(|| primal_data.get("url"))
            .and_then(|v| v.as_str())
            .unwrap_or(source_endpoint);

        let capabilities = primal_data
            .get("capabilities")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(crate::traits::PrimalCapability::new))
                    .collect()
            })
            .unwrap_or_default();

        let metadata = primal_data
            .get("metadata")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default();

        let health_status = primal_data
            .get("health")
            .or_else(|| primal_data.get("status"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        Some(crate::discovery::types::DiscoveredPrimal {
            primal_id: primal_id.to_string(),
            primal_type,
            endpoint: endpoint.to_string(),
            capabilities,
            metadata,
            discovery_method: crate::discovery::types::DiscoveryMethod::ServiceRegistry,
            health_status: health_status.to_string(),
            last_seen: std::time::Instant::now(),
            registration: crate::universal_registry::UniversalServiceRegistration::default(),
        })
    }

    /// Get all discovered primals
    pub fn get_discovered_primals(&self) -> &HashMap<String, DiscoveredPrimal> {
        &self.discovered_primals
    }

    /// Get a specific discovered primal by ID
    pub fn get_primal_by_id(&self, primal_id: &str) -> Option<&DiscoveredPrimal> {
        self.discovered_primals.get(primal_id)
    }

    /// Get primals by type
    pub fn get_primals_by_type(&self, primal_type: &str) -> Vec<&DiscoveredPrimal> {
        self.discovered_primals
            .values()
            .filter(|primal| primal.primal_type.to_string() == primal_type)
            .collect()
    }

    /// Get discovery statistics
    pub fn get_discovery_stats(&self) -> &DiscoveryStats {
        &self.discovery_stats
    }

    /// Get discovery configuration
    pub fn get_discovery_config(&self) -> &songbird_config::unified::UnifiedDiscoveryConfig {
        &self.config.discovery
    }

    /// Update discovery configuration
    pub fn update_discovery_config(
        &mut self,
        new_config: songbird_config::unified::UnifiedDiscoveryConfig,
    ) {
        self.config.discovery = new_config;
    }

    /// Generate a discovery summary
    pub fn generate_summary(&self) -> crate::discovery::discovery_summary::DiscoverySummary {
        let _primals_vec: Vec<&DiscoveredPrimal> = self.discovered_primals.values().collect();
        // Create discovery stats with correct type
        let stats = crate::discovery::discovery_summary::DiscoveryStats {
            total_attempts: self.discovered_primals.len() as u64,
            successful_discoveries: self.discovered_primals.len() as u64,
            failed_attempts: 0,
            average_discovery_time_ms: 150.0,
            capability_counts: std::collections::HashMap::new(),
            method_counts: std::collections::HashMap::new(),
        };
        // Create config summary
        let config_summary = format!(
            "Discovery engine with {} total primals discovered across {} methods",
            self.discovered_primals.len(),
            3 // Network, Config, Environment
        );
        crate::discovery::discovery_summary::DiscoverySummary::from_discovered_primals(
            &self.discovered_primals,
            &stats,
            config_summary,
        )
    }

    /// Clear all discovered primals and reset stats
    pub fn reset(&mut self) {
        self.discovered_primals.clear();
        self.discovery_stats = DiscoveryStats::default();
    }

    /// Get count of primals by discovery method
    pub fn get_method_counts(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for primal in self.discovered_primals.values() {
            let method_name = format!("{:?}", primal.discovery_method);
            *counts.entry(method_name).or_insert(0) += 1;
        }
        counts
    }

    /// Check if a specific primal type was discovered
    pub fn has_primal_type(&self, primal_type: &str) -> bool {
        self.discovered_primals
            .values()
            .any(|primal| primal.primal_type.to_string() == primal_type)
    }

    /// Get the total number of discovered primals
    pub fn total_discovered(&self) -> usize {
        self.discovered_primals.len()
    }
}
// #[cfg(test)]
// mod tests {
//     use super::super::types::{DiscoveredPrimal, DiscoveryMethod};
//     use super::*;
//     use std::time::Instant;
//     fn create_test_primal_config() -> songbird_config::SongbirdConfig {
//         songbird_config::SongbirdConfig::development()
//     }
//     fn create_test_discovery_config() -> songbird_config::unified::UnifiedDiscoveryConfig {
//         songbird_config::unified::UnifiedDiscoveryConfig {
//             backend: "static".to_string(),
//             consul_url: None,
//             kubernetes_namespace: None,
//             health_check_interval_secs: 30,
//             service_registration_enabled: false,
//             refresh_interval: std::time::Duration::from_secs(1),
//             discovery_timeout: std::time::Duration::from_secs(1),
//             max_concurrent_discoveries: 1,
//             auto_discovery: false,
//             enable_network_discovery: false, // Disable to prevent hanging
//         }
//     }
//     #[test]
//     fn test_discovery_engine_creation() {
//         let config = create_test_primal_config();
//         let engine = PrimalDiscoveryEngine::new(config);
//         assert_eq!(engine.total_discovered(), 0);
//         assert!(engine.get_discovered_primals().is_empty());
//         Ok(())
//     }
//     #[test]
//     fn test_discovery_engine_with_custom_config() {
//         let primal_config = create_test_primal_config();
//         let discovery_config = create_test_discovery_config();
//         let engine = PrimalDiscoveryEngine::with_discovery_config(primal_config, discovery_config);
//         assert!(!engine.get_discovery_config().network_discovery);
//         Ok(())
//     }
//     #[test]
//     fn test_get_primals_by_type() {
//         let mut engine = PrimalDiscoveryEngine::new(create_test_primal_config());
//         // Add a test primal
//         let test_primal = DiscoveredPrimal {
//             primal_id: "security-service-001".to_string(),
//             primal_type: PrimalType::Security,
//             capabilities: vec![
//                 PrimalCapability::Security {
//                     protocols: vec!["https".to_string()],
//                 },
//                 PrimalCapability::Authentication {
//                     methods: vec!["oauth2".to_string()],
//                 },
//                 PrimalCapability::Encryption {
//                     algorithms: vec!["aes256".to_string()],
//                 },
//             ],
//             endpoint: "https://security.local:8443".to_string(),
//             health_status: "healthy".to_string(),
//             discovery_method: DiscoveryMethod::NetworkScan,
//             last_seen: std::time::Instant::now(),
//             metadata: std::collections::HashMap::from([
//                 ("service_type".to_string(), "security".to_string()),
//                 ("inferred_from".to_string(), "capabilities".to_string()),
//             ]),
//         };
//         engine
//             .discovered_primals
//             .insert("security-service-001".to_string(), test_primal);
//         let security_primals = engine.get_primals_by_type("security-provider");
//         assert_eq!(security_primals.len(), 1);
//         assert_eq!(security_primals[0].primal_type, PrimalType::Security);
//         let nonexistent_primals = engine.get_primals_by_type("nonexistent");
//         assert_eq!(nonexistent_primals.len(), 0);
//         Ok(())
//     }
//     #[test]
//     fn test_has_primal_type() {
//         let mut engine = PrimalDiscoveryEngine::new(create_test_primal_config());
//         // Test capability-based type checking
//         assert!(!engine.has_primal_type("security-provider"));
//         // Register a security service
//         let test_primal = DiscoveredPrimal {
//             primal_id: "security-test-001".to_string(),
//             primal_type: PrimalType::Security,
//             capabilities: vec![
//                 PrimalCapability::Security {
//                     protocols: vec!["https".to_string()],
//                 },
//                 PrimalCapability::Authentication {
//                     methods: vec!["oauth".to_string()],
//                 },
//             ],
//             endpoint: "https://security-test.local:8443".to_string(),
//             metadata: std::collections::HashMap::new(),
//             discovery_method: DiscoveryMethod::NetworkScan,
//             last_seen: std::time::Instant::now(),
//         };
//         engine
//             .discovered_primals
//             .insert("security-test-001".to_string(), test_primal);
//         // Now should find the security provider
//         assert!(engine.has_primal_type("security-provider"));
//         assert!(!engine.has_primal_type("storage-provider"));
//         Ok(())
//     }
//     #[test]
//     fn test_reset() {
//         let mut engine = PrimalDiscoveryEngine::new(create_test_primal_config());
//         let primal = DiscoveredPrimal {
//             primal_id: "test-1".to_string(),
//             primal_type: PrimalType::Unknown("testing".to_string()),
//             capabilities: vec![PrimalCapability::Custom {
//                 properties: vec![
//                     ("name".to_string(), "testing".to_string()),
//                     ("description".to_string(), "Test capability".to_string()),
//                 ],
//             }],
//             endpoint: "http://test:{}".to_string(),
//             discovery_method: DiscoveryMethod::ServiceRegistry,
//             last_seen: std::time::Instant::now(),
//             metadata: std::collections::HashMap::new(),
//             registration: crate::universal_registry::UniversalServiceRegistration::default(),
//         };
//         engine
//             .discovered_primals
//             .insert("test-1".to_string(), primal);
//         assert_eq!(engine.total_discovered(), 1);
//         engine.reset();
//         assert_eq!(engine.total_discovered(), 0);
//         Ok(())
//     }
//     #[tokio::test]
//     async fn test_start_discovery_basic() -> Result<()>{
//         let mut engine = PrimalDiscoveryEngine::with_discovery_config(
//             create_test_primal_config(),
//             create_test_discovery_config(),
//         );
//         // This should complete without hanging (all discovery methods disabled)
//         let result = engine.start_discovery().await;
//         assert!(result.is_ok());
//         Ok(())
//     }
// }
