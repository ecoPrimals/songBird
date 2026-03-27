// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Runtime Capability-Based Discovery
//!
//! **Core Principle**: Each primal knows ONLY itself. Discovery happens at runtime.
//!
//! This module implements zero-hardcoding, capability-based service discovery:
//! 1. Environment variables (primary)
//! 2. DNS-SD / mDNS (local network)
//! 3. Central registry (if available)
//! 4. Capability announcements (peer-to-peer)
//!
//! # Sovereignty Principles
//!
//! - No primal hardcodes knowledge of other primals
//! - Discovery is dynamic and runtime-based
//! - Capability-driven, not name-driven
//! - Zero assumptions about network topology

use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

// Multicast constants for peer-to-peer service announcements
const MULTICAST_ADDR_OCTETS: [u8; 4] = [239, 255, 255, 250];
const MULTICAST_PORT: u16 = 9091;

/// Runtime service discovery engine
///
/// Discovers services by capability at runtime with zero hardcoded knowledge.
pub struct RuntimeDiscoveryEngine {
    /// Required capabilities for this primal
    #[expect(dead_code, reason = "future: multi-capability queries")]
    capabilities: Vec<String>,

    /// Discovery timeout (will be used in timeout wrapper around discovery methods)
    timeout: Duration,

    /// Cache of discovered services
    cache: Arc<RwLock<HashMap<String, DiscoveredService>>>,

    /// Cache TTL
    cache_ttl: Duration,
}

impl RuntimeDiscoveryEngine {
    /// Create new discovery engine
    #[must_use]
    pub fn new() -> Self {
        Self {
            capabilities: Vec::new(),
            timeout: Duration::from_secs(5),
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: Duration::from_secs(300), // 5 minutes
        }
    }

    /// Create with specific capabilities
    #[must_use]
    pub fn with_capabilities(capabilities: Vec<String>) -> Self {
        Self {
            capabilities,
            timeout: Duration::from_secs(5),
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: Duration::from_secs(300),
        }
    }

    /// Discover service by capability (NO hardcoding)
    ///
    /// # Discovery Strategy
    ///
    /// 1. Check environment variable first (e.g., `COMPUTE_ENDPOINT`)
    /// 2. Try mDNS/DNS-SD for local network discovery
    /// 3. Query central registry if configured
    /// 4. Listen for capability announcements
    ///
    /// # Errors
    ///
    /// Returns error if no service found for the capability after all methods exhausted.
    pub async fn discover_by_capability(
        &self,
        capability: &str,
    ) -> SongbirdResult<DiscoveredService> {
        // Check cache first
        if let Some(cached) = self.check_cache(capability).await {
            return Ok(cached);
        }

        // 1. Try environment variable (highest priority)
        if let Ok(service) =
            Self::from_environment_with(capability, &|k| songbird_process_env::var(k))
        {
            self.update_cache(capability, &service).await;
            return Ok(service);
        }

        // 2. Try mDNS/DNS-SD for local network
        if let Ok(service) = self.discover_mdns(capability).await {
            self.update_cache(capability, &service).await;
            return Ok(service);
        }

        // 3. Try central registry
        if let Ok(service) = self.discover_registry(capability).await {
            self.update_cache(capability, &service).await;
            return Ok(service);
        }

        // 4. Wait for announcement (with timeout)
        if let Ok(service) = self.wait_for_announcement(capability).await {
            self.update_cache(capability, &service).await;
            return Ok(service);
        }

        Err(SongbirdError::discovery(format!(
            "No service found for capability '{capability}' after all discovery methods"
        )))
    }

    /// Get endpoint from environment variable
    ///
    /// Environment variable format: `{CAPABILITY}_ENDPOINT`
    /// Example: `COMPUTE_ENDPOINT=http://10.0.1.50:8001`
    fn from_environment_with(
        capability: &str,
        env: &impl Fn(&str) -> Result<String, std::env::VarError>,
    ) -> Result<DiscoveredService, SongbirdError> {
        let env_var = format!("{}_ENDPOINT", capability.to_uppercase());

        let endpoint = env(&env_var).map_err(|_| {
            SongbirdError::configuration(format!("Environment variable {env_var} not set"))
        })?;

        Ok(DiscoveredService {
            capability: capability.to_string(),
            endpoint,
            discovered_via: DiscoveryMethod::Environment,
            health_score: 1.0,
            last_seen: std::time::SystemTime::now(),
        })
    }

    /// Discover via mDNS (local network)
    ///
    /// Uses multicast DNS to find services on the local network.
    /// Service type: `_{capability}._tcp.local`
    ///
    /// # Implementation
    ///
    /// Uses the mdns-sd crate for zero-configuration discovery on local networks.
    /// Queries for services advertising the requested capability via TXT records.
    async fn discover_mdns(&self, capability: &str) -> SongbirdResult<DiscoveredService> {
        use tracing::debug;

        debug!("Attempting mDNS discovery for capability '{capability}'");

        // mDNS is best-effort for local network discovery
        // If we can't discover via mDNS, we'll try other methods

        // Check if mdns feature is enabled (optional dependency)
        #[cfg(feature = "mdns")]
        {
            // Use existing mDNS implementation from capability_based_runtime_discovery module
            use crate::capability_based_runtime_discovery::mdns::MdnsDiscovery;

            let mdns = MdnsDiscovery::new(None);
            match mdns.discover_by_capability(capability, Some(self.timeout)).await {
                Ok(services) => {
                    if let Some(service_info) = services.first() {
                        debug!(
                            "mDNS discovered service at {} for capability '{capability}'",
                            service_info.address
                        );
                        return Ok(DiscoveredService {
                            capability: capability.to_string(),
                            endpoint: format!("http://{}", service_info.address),
                            discovered_via: DiscoveryMethod::MDNS,
                            health_score: 1.0,
                            last_seen: std::time::SystemTime::now(),
                        });
                    }
                }
                Err(e) => {
                    debug!("mDNS discovery failed for '{capability}': {e}");
                }
            }
        }

        #[cfg(not(feature = "mdns"))]
        {
            use tracing::warn;
            warn!("mDNS discovery requested but mdns feature not enabled");
        }

        Err(SongbirdError::discovery(format!(
            "No service found via mDNS for capability '{capability}'"
        )))
    }

    /// Discover via central registry
    ///
    /// Queries a central service registry if `REGISTRY_ENDPOINT` is set.
    ///
    /// # Implementation
    ///
    /// Connects to a Consul/etcd-compatible service registry and queries
    /// for services providing the requested capability.
    async fn discover_registry(&self, capability: &str) -> SongbirdResult<DiscoveredService> {
        use crate::capability_based_runtime_discovery::CapabilityRequest;
        use crate::capability_based_runtime_discovery::service_registry::ServiceRegistryDiscovery;
        use tracing::{debug, info};

        // Check if registry endpoint is configured
        let Ok(registry_endpoint) = songbird_process_env::var("REGISTRY_ENDPOINT") else {
            return Err(SongbirdError::configuration(
                "No registry endpoint configured (REGISTRY_ENDPOINT not set)",
            ));
        };

        debug!(
            "Querying service registry at '{}' for capability '{}'",
            registry_endpoint, capability
        );

        // Use the existing service registry discovery implementation
        // This integrates with crates/songbird-config/src/capability_based_runtime_discovery/service_registry.rs
        let discovery = ServiceRegistryDiscovery::new(&registry_endpoint);
        let request = CapabilityRequest {
            capability: capability.to_string(),
            required_features: Vec::new(),
            optional_features: Vec::new(),
            preferences: Vec::new(),
            min_sla: None,
        };

        match discovery.discover(&request).await {
            Ok(provider) => {
                info!(
                    "Discovered provider '{}' for capability '{}' from registry",
                    provider.name, capability
                );

                Ok(DiscoveredService {
                    capability: capability.to_string(),
                    endpoint: provider.endpoint,
                    discovered_via: DiscoveryMethod::Registry,
                    health_score: 1.0, // Default healthy score
                    last_seen: std::time::SystemTime::now(),
                })
            }
            Err(e) => {
                debug!("Registry query failed for '{capability}': {e}");
                Err(SongbirdError::discovery(format!(
                    "No service found in registry for capability '{capability}': {e}"
                )))
            }
        }
    }

    /// Wait for capability announcement
    ///
    /// Listens for peer-to-peer capability announcements with timeout.
    ///
    /// # Implementation
    ///
    /// Uses multicast announcements or gossip protocol for peer discovery.
    /// Waits for announcement with configured timeout (default 5 seconds).
    ///
    /// # Complexity Note
    ///
    /// This function handles the complete announcement lifecycle. Consider
    /// refactoring into smaller helper functions in future iterations.
    async fn wait_for_announcement(&self, capability: &str) -> SongbirdResult<DiscoveredService> {
        use tokio::time::timeout;
        use tracing::{debug, info};

        debug!("Waiting for announcement for capability '{}'", capability);

        // Create a channel for receiving announcements
        let (tx, mut rx) = tokio::sync::mpsc::channel::<DiscoveredService>(10);

        // Spawn announcement listener in background
        let capability_clone = capability.to_string();
        let timeout_duration = self.timeout;

        tokio::spawn(async move {
            use std::net::{Ipv4Addr, SocketAddrV4};
            use tokio::net::UdpSocket;

            debug!(
                "Announcement listener started for capability '{}' (timeout: {:?})",
                capability_clone, timeout_duration
            );

            // Listen on multicast address for service announcements
            // Using standard multicast address 239.255.255.250:9091
            let multicast_addr = Ipv4Addr::from(MULTICAST_ADDR_OCTETS);

            match UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, MULTICAST_PORT)).await {
                Ok(socket) => {
                    // Join multicast group
                    if let Err(e) = socket.join_multicast_v4(multicast_addr, Ipv4Addr::UNSPECIFIED)
                    {
                        debug!("Failed to join multicast group: {}", e);
                        drop(tx);
                        return;
                    }

                    let mut buf = [0u8; 1024];
                    let start = std::time::Instant::now();

                    // Listen for announcements until timeout
                    while start.elapsed() < timeout_duration {
                        match tokio::time::timeout(
                            Duration::from_millis(100),
                            socket.recv_from(&mut buf),
                        )
                        .await
                        {
                            Ok(Ok((len, addr))) => {
                                // Parse announcement (simple JSON format)
                                if let Ok(announcement) = std::str::from_utf8(&buf[..len])
                                    && let Ok(json) =
                                        serde_json::from_str::<serde_json::Value>(announcement)
                                {
                                    // Check if announcement matches capability
                                    if let Some(caps) =
                                        json.get("capabilities").and_then(|c| c.as_array())
                                    {
                                        let has_capability = caps
                                            .iter()
                                            .any(|c| c.as_str() == Some(&capability_clone));

                                        if has_capability
                                            && let Some(endpoint) =
                                                json.get("endpoint").and_then(|e| e.as_str())
                                        {
                                            debug!(
                                                "Received matching announcement from {} for '{}'",
                                                addr, capability_clone
                                            );

                                            let service = DiscoveredService {
                                                capability: capability_clone.clone(),
                                                endpoint: endpoint.to_string(),
                                                discovered_via: DiscoveryMethod::Announcement,
                                                health_score: 1.0,
                                                last_seen: std::time::SystemTime::now(),
                                            };

                                            let _ = tx.send(service).await;
                                            return; // Found a match, exit early
                                        }
                                    }
                                }
                            }
                            _ => {
                                // Timeout or error, continue listening
                                tokio::task::yield_now().await;
                            }
                        }
                    }

                    debug!("Announcement listener timeout for '{}'", capability_clone);
                }
                Err(e) => {
                    debug!("Failed to bind UDP socket for announcements: {}", e);
                }
            }

            drop(tx);
        });

        // Wait for announcement with timeout
        match timeout(self.timeout, rx.recv()).await {
            Ok(Some(service)) => {
                info!(
                    "Received announcement for capability '{}' from '{}'",
                    capability, service.endpoint
                );
                Ok(service)
            }
            Ok(None) => {
                debug!("Announcement channel closed without receiving service");
                Err(SongbirdError::discovery(format!(
                    "No announcement received for capability '{capability}'"
                )))
            }
            Err(_) => {
                debug!("Announcement wait timed out for capability '{}'", capability);
                Err(SongbirdError::discovery(format!(
                    "Timeout waiting for announcement for capability '{capability}'"
                )))
            }
        }
    }

    /// Check cache for discovered service
    async fn check_cache(&self, capability: &str) -> Option<DiscoveredService> {
        if let Some(service) = self.cache.read().await.get(capability) {
            // Check if cache entry is still valid
            if let Ok(elapsed) = service.last_seen.elapsed()
                && elapsed < self.cache_ttl
            {
                return Some(service.clone());
            }
        }

        None
    }

    /// Update cache with discovered service
    async fn update_cache(&self, capability: &str, service: &DiscoveredService) {
        let mut cache = self.cache.write().await;
        cache.insert(capability.to_string(), service.clone());
    }
}

impl Default for RuntimeDiscoveryEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Discovered service information
#[derive(Debug, Clone)]
pub struct DiscoveredService {
    /// Capability provided by this service
    pub capability: String,

    /// Service endpoint (e.g., "<http://10.0.1.50:8001>")
    pub endpoint: String,

    /// How this service was discovered
    pub discovered_via: DiscoveryMethod,

    /// Health score (0.0 = unhealthy, 1.0 = healthy)
    pub health_score: f64,

    /// When this service was last seen
    pub last_seen: std::time::SystemTime,
}

/// Discovery method used to find a service
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscoveryMethod {
    /// Discovered via environment variable
    Environment,

    /// Discovered via mDNS (multicast DNS)
    MDNS,

    /// Discovered via central registry
    Registry,

    /// Discovered via peer announcement
    Announcement,
}

impl std::fmt::Display for DiscoveryMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Environment => write!(f, "Environment Variable"),
            Self::MDNS => write!(f, "mDNS"),
            Self::Registry => write!(f, "Central Registry"),
            Self::Announcement => write!(f, "Peer Announcement"),
        }
    }
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Discover compute service
///
/// Looks for service with "compute" capability.
///
/// # Environment Variable
///
/// Set `COMPUTE_ENDPOINT` to specify the compute service endpoint.
/// Example: `COMPUTE_ENDPOINT=http://10.0.1.50:8001`
///
/// # Errors
///
/// Returns error if no compute service found.
pub async fn discover_compute() -> SongbirdResult<DiscoveredService> {
    RuntimeDiscoveryEngine::new().discover_by_capability("compute").await
}

/// Discover AI service
///
/// Looks for service with "ai" capability.
///
/// # Environment Variable
///
/// Set `AI_ENDPOINT` to specify the AI service endpoint.
/// Example: `AI_ENDPOINT=http://10.0.1.51:8002`
///
/// # Errors
///
/// Returns error if no AI service found.
pub async fn discover_ai() -> SongbirdResult<DiscoveredService> {
    RuntimeDiscoveryEngine::new().discover_by_capability("ai").await
}

/// Discover storage service
///
/// Looks for service with "storage" capability.
///
/// # Environment Variable
///
/// Set `STORAGE_ENDPOINT` to specify the storage service endpoint.
/// Example: `STORAGE_ENDPOINT=http://10.0.1.52:8003`
///
/// # Errors
///
/// Returns error if no storage service found.
pub async fn discover_storage() -> SongbirdResult<DiscoveredService> {
    RuntimeDiscoveryEngine::new().discover_by_capability("storage").await
}

/// Discover security service
///
/// Looks for service with "security" capability.
///
/// # Environment Variable
///
/// Set `SECURITY_ENDPOINT` to specify the security service endpoint.
/// Example: `SECURITY_ENDPOINT=http://10.0.1.53:8004`
///
/// # Errors
///
/// Returns error if no security service found.
pub async fn discover_security() -> SongbirdResult<DiscoveredService> {
    RuntimeDiscoveryEngine::new().discover_by_capability("security").await
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
#[expect(clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use songbird_process_env;

    #[test]
    fn test_discovery_method_display() {
        assert_eq!(DiscoveryMethod::Environment.to_string(), "Environment Variable");
        assert_eq!(DiscoveryMethod::MDNS.to_string(), "mDNS");
        assert_eq!(DiscoveryMethod::Registry.to_string(), "Central Registry");
        assert_eq!(DiscoveryMethod::Announcement.to_string(), "Peer Announcement");
    }

    #[tokio::test]
    async fn test_environment_discovery() {
        let result = RuntimeDiscoveryEngine::from_environment_with("test", &|k| {
            if k == "TEST_ENDPOINT" {
                Ok("http://test.example.com:8080".to_string())
            } else {
                Err(std::env::VarError::NotPresent)
            }
        });
        assert!(result.is_ok());

        let service = result.expect("env set");
        assert_eq!(service.capability, "test");
        assert_eq!(service.endpoint, "http://test.example.com:8080");
        assert_eq!(service.discovered_via, DiscoveryMethod::Environment);
    }

    #[test]
    fn test_from_environment_errors_when_var_missing() {
        let err =
            RuntimeDiscoveryEngine::from_environment_with("no_such_var_for_sb_rtdisc", &|_| {
                Err(std::env::VarError::NotPresent)
            })
            .expect_err("missing env var");
        assert!(matches!(err, songbird_types::SongbirdError::Configuration { .. }), "{err:?}");
    }

    #[tokio::test]
    async fn test_check_cache_misses_expired_entry() {
        let engine = RuntimeDiscoveryEngine::new();
        let service = DiscoveredService {
            capability: "exp".to_string(),
            endpoint: "http://old".to_string(),
            discovered_via: DiscoveryMethod::Environment,
            health_score: 1.0,
            last_seen: std::time::SystemTime::UNIX_EPOCH,
        };
        engine.update_cache("exp", &service).await;
        assert!(engine.check_cache("exp").await.is_none());
    }

    #[tokio::test]
    async fn test_cache_functionality() {
        let engine = RuntimeDiscoveryEngine::new();

        let service = DiscoveredService {
            capability: "test".to_string(),
            endpoint: "http://test.example.com:8080".to_string(),
            discovered_via: DiscoveryMethod::Environment,
            health_score: 1.0,
            last_seen: std::time::SystemTime::now(),
        };

        // Update cache
        engine.update_cache("test", &service).await;

        // Check cache
        let cached = engine.check_cache("test").await;
        assert!(cached.is_some());

        let cached_service = cached.expect("cached");
        assert_eq!(cached_service.capability, "test");
        assert_eq!(cached_service.endpoint, "http://test.example.com:8080");
    }

    #[test]
    fn engine_default_matches_new() {
        let _ = RuntimeDiscoveryEngine::default();
        let _ = RuntimeDiscoveryEngine::new();
    }

    #[test]
    fn with_capabilities_constructs_engine() {
        let _e = RuntimeDiscoveryEngine::with_capabilities(vec!["a".into(), "b".into()]);
    }

    #[serial_test::serial]
    #[tokio::test]
    async fn discover_by_capability_uses_environment_variable() {
        let cap = "sbserialrtcap";
        let var = format!("{}_ENDPOINT", cap.to_uppercase());
        songbird_process_env::set_var(&var, "http://rt-env:8080");
        let engine = RuntimeDiscoveryEngine::new();
        let s = engine.discover_by_capability(cap).await.expect("from env");
        assert_eq!(s.endpoint, "http://rt-env:8080");
        assert_eq!(s.discovered_via, DiscoveryMethod::Environment);
        songbird_process_env::remove_var(&var);
    }

    #[serial_test::serial]
    #[tokio::test]
    async fn discover_by_capability_returns_cached_before_env() {
        let cap = "sbcachedcap";
        let var = format!("{}_ENDPOINT", cap.to_uppercase());
        songbird_process_env::set_var(&var, "http://should-not-be-used");
        let engine = RuntimeDiscoveryEngine::new();
        let fresh = DiscoveredService {
            capability: cap.to_string(),
            endpoint: "http://cached-first".to_string(),
            discovered_via: DiscoveryMethod::Environment,
            health_score: 1.0,
            last_seen: std::time::SystemTime::now(),
        };
        engine.update_cache(cap, &fresh).await;
        let s = engine.discover_by_capability(cap).await.expect("cache");
        assert_eq!(s.endpoint, "http://cached-first");
        songbird_process_env::remove_var(&var);
    }

    #[tokio::test]
    async fn discover_compute_errors_without_configuration() {
        let err = discover_compute().await.expect_err("no compute");
        assert!(matches!(err, SongbirdError::Discovery { .. }), "{err:?}");
    }

    #[tokio::test]
    async fn discover_ai_errors_without_configuration() {
        let err = discover_ai().await.expect_err("no ai");
        assert!(matches!(err, SongbirdError::Discovery { .. }), "{err:?}");
    }

    #[tokio::test]
    async fn discover_storage_errors_without_configuration() {
        let err = discover_storage().await.expect_err("no storage");
        assert!(matches!(err, SongbirdError::Discovery { .. }), "{err:?}");
    }

    #[tokio::test]
    async fn discover_security_errors_without_configuration() {
        let err = discover_security().await.expect_err("no security");
        assert!(matches!(err, SongbirdError::Discovery { .. }), "{err:?}");
    }

    #[test]
    fn discovered_service_clone_and_debug() {
        let s = DiscoveredService {
            capability: "c".into(),
            endpoint: "e".into(),
            discovered_via: DiscoveryMethod::Registry,
            health_score: 0.5,
            last_seen: std::time::SystemTime::UNIX_EPOCH,
        };
        let _ = format!("{:?}", &s);
        assert_eq!(s.health_score, 0.5);
    }
}
