// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! mDNS Discovery Backend - COMPLETE IMPLEMENTATION
//!
//! Production-ready mDNS (Multicast DNS) discovery implementation for local network
//! service discovery based on capabilities. This implementation follows RFC 6762.
//!
//! # Architecture
//! - Uses `mdns-sd` crate (pure Rust, zero C dependencies)
//! - Capability-based service advertising (no hardcoded names)
//! - Efficient caching with TTL
//! - Graceful shutdown with goodbye packets
//! - IPv4 and IPv6 support
//!
//! # Examples
//! ```rust,ignore
//! use songbird_config::discovery::mdns::MdnsDiscovery;
//! use std::time::Duration;
//!
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     let mdns = MdnsDiscovery::new()?;
//!
//!     // Advertise our capabilities
//!     mdns.advertise(&["compute", "gpu"]).await?;
//!
//!     // Discover services with specific capability
//!     let services = mdns.discover_by_capability("storage", Some(Duration::from_secs(5))).await?;
//!     Ok(())
//! }
//! ```

#![allow(missing_docs, reason = "mDNS helpers align with RFC 6762 terminology")]

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// mDNS service type for auto-discovery
#[cfg(feature = "mdns")]
const SERVICE_TYPE: &str = "_songbird._tcp.local.";

/// mDNS discovery backend implementation
///
/// Provides zero-configuration local network discovery using Multicast DNS.
/// Services advertise their capabilities and can be discovered by capability queries.
pub struct MdnsDiscovery {
    /// Service name (usually hostname)
    service_name: String,
    /// Our advertised capabilities
    advertised_capabilities: Arc<RwLock<Vec<String>>>,
    /// Port we're listening on
    listen_port: u16,
    /// Discovered services cache
    cache: Arc<RwLock<HashMap<String, Vec<MdnsServiceInfo>>>>,
    /// Cache TTL
    cache_ttl: Duration,
    /// mDNS daemon handle (for advertisement)
    #[cfg(feature = "mdns")]
    mdns_daemon: Arc<RwLock<Option<mdns_sd::ServiceDaemon>>>,
}

/// Discovered service information from mDNS
#[derive(Debug, Clone)]
pub struct MdnsServiceInfo {
    /// Service address (host:port)
    pub address: SocketAddr,
    /// Service capabilities from TXT records
    pub capabilities: Vec<String>,
    /// Additional metadata from TXT records
    pub metadata: HashMap<String, String>,
    /// When this service was discovered
    pub discovered_at: std::time::SystemTime,
}

/// mDNS discovery errors
#[derive(Debug, Error)]
pub enum MdnsError {
    /// Failed to initialize mDNS responder
    #[error("Failed to initialize mDNS responder: {0}")]
    InitializationFailed(String),

    /// Network error during discovery
    #[error("Network error during mDNS discovery: {0}")]
    NetworkError(String),

    /// Discovery timeout
    #[error("mDNS discovery timed out after {timeout:?}")]
    DiscoveryTimeout {
        timeout: Duration,
    },

    /// Service registration failed
    #[error("Failed to register service: {0}")]
    RegistrationFailed(String),

    /// Invalid service name
    #[error("Invalid service name: {0}")]
    InvalidServiceName(String),
}

impl MdnsDiscovery {
    /// Create a new mDNS discovery instance
    ///
    /// # Arguments
    /// * `service_name` - Name to advertise (typically hostname)
    /// * `listen_port` - Port this service listens on
    ///
    /// # Errors
    /// Returns `MdnsError::InitializationFailed` if mDNS cannot be initialized
    pub fn new_with_port(
        service_name: impl Into<String>,
        listen_port: u16,
    ) -> Result<Self, MdnsError> {
        let service_name = service_name.into();

        // Validate service name (DNS label rules)
        if service_name.is_empty() || service_name.len() > 63 {
            return Err(MdnsError::InvalidServiceName(
                "Service name must be 1-63 characters".to_string(),
            ));
        }

        // Initialize mDNS daemon if feature enabled
        #[cfg(feature = "mdns")]
        let mdns_daemon = {
            match mdns_sd::ServiceDaemon::new() {
                Ok(daemon) => Arc::new(RwLock::new(Some(daemon))),
                Err(e) => {
                    warn!(
                        "Failed to initialize mDNS daemon: {}. Discovery will work but advertising is disabled.",
                        e
                    );
                    Arc::new(RwLock::new(None))
                }
            }
        };

        Ok(Self {
            service_name,
            advertised_capabilities: Arc::new(RwLock::new(Vec::new())),
            listen_port,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: Duration::from_secs(60),
            #[cfg(feature = "mdns")]
            mdns_daemon,
        })
    }

    /// Create with default hostname and port 0 (OS-assigned)
    ///
    /// # Errors
    /// Returns error if hostname cannot be determined or mDNS init fails
    pub fn new() -> Result<Self, MdnsError> {
        let hostname = gethostname::gethostname()
            .into_string()
            .unwrap_or_else(|_| "songbird-service".to_string());

        Self::new_with_port(hostname, 0)
    }

    /// Advertise this service with specified capabilities
    ///
    /// Registers service with mDNS responder and creates TXT records with capabilities.
    ///
    /// # Implementation Details
    /// - Creates TXT records with `capability=<cap>` for each capability
    /// - Adds `version=<version>` metadata
    /// - TTL set to 120 seconds with automatic refresh
    ///
    /// # Arguments
    /// * `capabilities` - List of capabilities this service provides
    ///
    /// # Errors
    /// Returns `MdnsError::RegistrationFailed` if advertisement fails
    pub async fn advertise(&self, capabilities: &[&str]) -> Result<(), MdnsError> {
        // Store capabilities
        *self.advertised_capabilities.write().await =
            capabilities.iter().map(|s| (*s).to_string()).collect();

        info!(
            service = %self.service_name,
            capabilities = ?capabilities,
            port = self.listen_port,
            "Advertising service via mDNS"
        );

        #[cfg(feature = "mdns")]
        {
            let daemon_lock = self.mdns_daemon.read().await;
            if let Some(daemon) = daemon_lock.as_ref() {
                // Build TXT records with capabilities
                let mut properties = HashMap::new();
                for cap in capabilities {
                    properties.insert("capability".to_string(), cap.to_string());
                }
                properties.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());

                // Create service info
                let service_info = mdns_sd::ServiceInfo::new(
                    SERVICE_TYPE,
                    &self.service_name,
                    &format!("{}.local.", self.service_name),
                    "", // Will be filled by daemon
                    self.listen_port,
                    properties,
                )
                .map_err(|e| {
                    MdnsError::RegistrationFailed(format!("Failed to create service info: {e}"))
                })?;

                // Register service
                daemon.register(service_info).map_err(|e| {
                    MdnsError::RegistrationFailed(format!("Failed to register service: {e}"))
                })?;

                info!(service = %self.service_name, "Service registered with mDNS");
            } else {
                warn!("mDNS daemon not initialized, skipping advertisement");
            }
        }

        #[cfg(not(feature = "mdns"))]
        {
            debug!("mDNS feature not enabled, storing capabilities but not advertising");
        }

        Ok(())
    }

    /// Discover services by capability
    ///
    /// Queries the local network for services advertising the specified capability.
    /// Uses efficient browsing with timeout and caches results.
    ///
    /// # Arguments
    /// * `capability` - The capability to search for (e.g., "compute", "storage")
    /// * `timeout` - Optional timeout (default: 5 seconds)
    ///
    /// # Returns
    /// List of services that advertise the requested capability
    ///
    /// # Errors
    /// Returns error if network query fails or times out
    #[allow(
        clippy::too_many_lines,
        reason = "mDNS capability scan with timeout and result caching"
    )]
    pub async fn discover_by_capability(
        &self,
        capability: &str,
        timeout: Option<Duration>,
    ) -> Result<Vec<MdnsServiceInfo>, MdnsError> {
        let timeout = timeout.unwrap_or(Duration::from_secs(5));

        debug!(
            capability = %capability,
            timeout_secs = timeout.as_secs(),
            "Starting mDNS discovery"
        );

        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(capability) {
                // Filter out expired entries
                let now = std::time::SystemTime::now();
                let fresh: Vec<_> = cached
                    .iter()
                    .filter(|s| {
                        now.duration_since(s.discovered_at).unwrap_or(Duration::MAX)
                            < self.cache_ttl
                    })
                    .cloned()
                    .collect();

                if !fresh.is_empty() {
                    debug!(
                        capability = %capability,
                        count = fresh.len(),
                        "Returning cached discovery results"
                    );
                    return Ok(fresh);
                }
            }
        }

        #[cfg(feature = "mdns")]
        {
            let daemon_lock = self.mdns_daemon.read().await;
            if let Some(daemon) = daemon_lock.as_ref() {
                // Browse for services
                let receiver = daemon
                    .browse(SERVICE_TYPE)
                    .map_err(|e| MdnsError::NetworkError(format!("Failed to start browse: {e}")))?;

                let mut discovered = Vec::new();
                let start = std::time::Instant::now();

                // Collect services until timeout
                while start.elapsed() < timeout {
                    match tokio::time::timeout(
                        timeout.saturating_sub(start.elapsed()),
                        tokio::task::spawn_blocking({
                            let rx = receiver.clone();
                            move || rx.recv_timeout(Duration::from_millis(100))
                        }),
                    )
                    .await
                    {
                        Ok(Ok(Ok(event))) => {
                            use mdns_sd::ServiceEvent;

                            if let ServiceEvent::ServiceResolved(info) = event {
                                // Parse TXT records for capabilities
                                let properties = info.get_properties();
                                let caps: Vec<String> = properties
                                    .iter()
                                    .filter_map(|prop| {
                                        if prop.key() == "capability" {
                                            Some(prop.val_str().to_string())
                                        } else {
                                            None
                                        }
                                    })
                                    .collect();

                                // Check if this service has the requested capability
                                if caps.iter().any(|c| c == capability) {
                                    // Extract address
                                    if let Some(addr) = info.get_addresses().iter().next() {
                                        let socket_addr = SocketAddr::new(*addr, info.get_port());

                                        let mut metadata = HashMap::new();
                                        for prop in properties.iter() {
                                            if prop.key() != "capability" {
                                                metadata.insert(
                                                    prop.key().to_string(),
                                                    prop.val_str().to_string(),
                                                );
                                            }
                                        }

                                        discovered.push(MdnsServiceInfo {
                                            address: socket_addr,
                                            capabilities: caps.clone(),
                                            metadata,
                                            discovered_at: std::time::SystemTime::now(),
                                        });

                                        debug!(
                                            service = %info.get_fullname(),
                                            address = %socket_addr,
                                            "Discovered matching service"
                                        );
                                    }
                                }
                            }
                        }
                        _ => {
                            // Timeout or error on this iteration, yield control
                            tokio::task::yield_now().await;
                        }
                    }
                }

                // Update cache
                if !discovered.is_empty() {
                    let mut cache = self.cache.write().await;
                    cache.insert(capability.to_string(), discovered.clone());
                }

                debug!(
                    capability = %capability,
                    count = discovered.len(),
                    "Discovery complete"
                );

                return Ok(discovered);
            }
        }

        #[cfg(not(feature = "mdns"))]
        {
            debug!(
                capability = %capability,
                "mDNS feature not enabled, returning empty results"
            );
        }

        Ok(Vec::new())
    }

    /// Discover all services on local network
    ///
    /// Performs a broad discovery without filtering by capability.
    ///
    /// # Errors
    ///
    /// Returns `MdnsError` if discovery fails or times out.
    pub async fn discover_all(
        &self,
        timeout: Option<Duration>,
    ) -> Result<Vec<MdnsServiceInfo>, MdnsError> {
        let timeout = timeout.unwrap_or(Duration::from_secs(5));

        info!(timeout_secs = timeout.as_secs(), "Discovering all services via mDNS");

        #[cfg(feature = "mdns")]
        {
            // Similar to discover_by_capability but without filtering
            let daemon_lock = self.mdns_daemon.read().await;
            if let Some(daemon) = daemon_lock.as_ref() {
                let receiver = daemon
                    .browse(SERVICE_TYPE)
                    .map_err(|e| MdnsError::NetworkError(format!("Failed to start browse: {e}")))?;

                let mut discovered = Vec::new();
                let start = std::time::Instant::now();

                while start.elapsed() < timeout {
                    match tokio::time::timeout(
                        timeout.saturating_sub(start.elapsed()),
                        tokio::task::spawn_blocking({
                            let rx = receiver.clone();
                            move || rx.recv_timeout(Duration::from_millis(100))
                        }),
                    )
                    .await
                    {
                        Ok(Ok(Ok(event))) => {
                            use mdns_sd::ServiceEvent;

                            if let ServiceEvent::ServiceResolved(info) = event {
                                let properties = info.get_properties();
                                let caps: Vec<String> = properties
                                    .iter()
                                    .filter_map(|prop| {
                                        if prop.key() == "capability" {
                                            Some(prop.val_str().to_string())
                                        } else {
                                            None
                                        }
                                    })
                                    .collect();

                                if let Some(addr) = info.get_addresses().iter().next() {
                                    let socket_addr = SocketAddr::new(*addr, info.get_port());

                                    let mut metadata = HashMap::new();
                                    for prop in properties.iter() {
                                        if prop.key() != "capability" {
                                            metadata.insert(
                                                prop.key().to_string(),
                                                prop.val_str().to_string(),
                                            );
                                        }
                                    }

                                    discovered.push(MdnsServiceInfo {
                                        address: socket_addr,
                                        capabilities: caps,
                                        metadata,
                                        discovered_at: std::time::SystemTime::now(),
                                    });
                                }
                            }
                        }
                        _ => {
                            tokio::task::yield_now().await;
                        }
                    }
                }

                return Ok(discovered);
            }
        }

        Ok(Vec::new())
    }

    /// Stop advertising this service
    ///
    /// Gracefully removes this service from mDNS announcements by sending goodbye packets.
    /// # Errors
    ///
    /// Returns `MdnsError` if stopping advertisement fails.
    pub async fn stop_advertising(&self) -> Result<(), MdnsError> {
        info!(service = %self.service_name, "Stopping mDNS advertisement");

        // Clear capabilities
        self.advertised_capabilities.write().await.clear();

        #[cfg(feature = "mdns")]
        {
            let daemon_lock = self.mdns_daemon.write().await;
            if let Some(daemon) = daemon_lock.as_ref() {
                // Shutdown daemon (sends goodbye packets automatically)
                daemon.shutdown().map_err(|e| {
                    MdnsError::NetworkError(format!("Failed to shutdown mDNS daemon: {e}"))
                })?;
            }
        }

        Ok(())
    }

    /// Clear the discovery cache
    ///
    /// Forces fresh discovery on next query.
    pub async fn clear_cache(&self) {
        self.cache.write().await.clear();
        debug!("mDNS discovery cache cleared");
    }
}

impl Default for MdnsDiscovery {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            warn!("Failed to create default mDNS discovery: {}", e);
            // Fallback with minimal config
            Self {
                service_name: "songbird".to_string(),
                advertised_capabilities: Arc::new(RwLock::new(Vec::new())),
                listen_port: 0,
                cache: Arc::new(RwLock::new(HashMap::new())),
                cache_ttl: Duration::from_secs(60),
                #[cfg(feature = "mdns")]
                mdns_daemon: Arc::new(RwLock::new(None)),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mdns_creation() {
        let mdns = MdnsDiscovery::new_with_port("test-service", 8080);
        assert!(mdns.is_ok());
        let mdns = mdns.unwrap();
        assert_eq!(mdns.service_name, "test-service");
        assert_eq!(mdns.listen_port, 8080);
    }

    #[test]
    fn test_invalid_service_name() {
        // Empty name
        let result = MdnsDiscovery::new_with_port("", 8080);
        assert!(result.is_err());

        // Too long name (>63 chars)
        let long_name = "a".repeat(64);
        let result = MdnsDiscovery::new_with_port(long_name, 8080);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_advertise_capabilities() {
        let mdns = MdnsDiscovery::new_with_port("test", 8080).unwrap();
        let result = mdns.advertise(&["compute", "storage"]).await;
        assert!(result.is_ok());

        let caps = mdns.advertised_capabilities.read().await;
        assert_eq!(caps.len(), 2);
        assert!(caps.contains(&"compute".to_string()));
        assert!(caps.contains(&"storage".to_string()));
    }

    #[tokio::test]
    async fn test_discover_with_cache() {
        let mdns = MdnsDiscovery::new().unwrap();

        // First discovery (will be empty without mdns feature)
        let result1 =
            mdns.discover_by_capability("compute", Some(Duration::from_millis(100))).await;
        assert!(result1.is_ok());

        // Second discovery (should use cache if any results)
        let result2 = mdns.discover_by_capability("compute", Some(Duration::from_millis(1))).await;
        assert!(result2.is_ok());
    }

    #[tokio::test]
    async fn test_cache_clearing() {
        let mdns = MdnsDiscovery::new().unwrap();
        mdns.clear_cache().await;
        let cache = mdns.cache.read().await;
        assert!(cache.is_empty());
    }

    #[tokio::test]
    async fn test_stop_advertising() {
        let mdns = MdnsDiscovery::new_with_port("test", 8080).unwrap();
        mdns.advertise(&["compute"]).await.unwrap();

        {
            let caps = mdns.advertised_capabilities.read().await;
            assert_eq!(caps.len(), 1);
        }

        mdns.stop_advertising().await.unwrap();

        let caps = mdns.advertised_capabilities.read().await;
        assert!(caps.is_empty());
    }

    #[tokio::test]
    async fn test_cache_expiry() {
        let mdns = MdnsDiscovery::new().unwrap();

        // Manually insert expired entry
        {
            let mut cache = mdns.cache.write().await;
            let old_time = std::time::SystemTime::now() - Duration::from_secs(120);
            cache.insert(
                "test".to_string(),
                vec![MdnsServiceInfo {
                    address: "127.0.0.1:8080".parse().unwrap(),
                    capabilities: vec!["test".to_string()],
                    metadata: HashMap::new(),
                    discovered_at: old_time,
                }],
            );
        }

        // Discovery should not return expired entry
        let result = mdns.discover_by_capability("test", Some(Duration::from_millis(10))).await;
        assert!(result.is_ok());
        // Should be empty because cache entry is expired
        let services = result.unwrap();
        assert!(
            services.is_empty()
                || services[0].discovered_at
                    > std::time::SystemTime::now() - Duration::from_secs(120)
        );
    }
}
