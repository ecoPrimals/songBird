//! mDNS-based Service Discovery (Multicast DNS)
//!
//! Implements RFC 6762 Multicast DNS for zero-configuration local network discovery.
//! Perfect for LAN environments, development, and edge deployments.
//!
//! ## Features
//! - Zero-configuration service discovery on local networks
//! - Automatic service announcement
//! - Real-time service updates
//! - No infrastructure dependencies
//! - Cross-platform support
//!
//! ## Usage
//! ```rust,no_run
//! use songbird_discovery::MdnsDiscovery;
//!
//! # async fn example() -> songbird_types::errors::SongbirdResult<()> {
//! let mut discovery = MdnsDiscovery::new(
//!     "songbird-instance-1".to_string(),
//!     "_songbird._tcp".to_string(),
//!     8080,
//! )?;
//!
//! discovery.start().await?;
//! let services = discovery.discover_services().await?;
//! # Ok(())
//! # }
//! ```

use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo as MdnsServiceInfo};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::traits::service::ServiceInfo;
use chrono::Utc;
use songbird_types::errors::{SongbirdError, SongbirdResult};

/// mDNS-based service discovery implementation
pub struct MdnsDiscovery {
    /// mDNS service daemon
    daemon: ServiceDaemon,

    /// Our service instance name
    instance_name: String,

    /// Service type (e.g., "_songbird._tcp")
    service_type: String,

    /// Our service port
    port: u16,

    /// TXT records for service metadata
    txt_records: HashMap<String, String>,

    /// Discovered services
    discovered: Arc<RwLock<HashMap<String, DiscoveredService>>>,

    /// Whether discovery is active
    active: Arc<RwLock<bool>>,
}

/// Internally tracked discovered service
#[derive(Clone, Debug)]
struct DiscoveredService {
    info: ServiceInfo,
    // Keep mdns_info for potential future queries (health checks, etc.)
    _mdns_info: MdnsServiceInfo,
}

impl MdnsDiscovery {
    /// Create a new mDNS discovery instance
    ///
    /// # Arguments
    /// * `instance_name` - Unique name for this service instance
    /// * `service_type` - Service type (e.g., "_songbird._tcp")
    /// * `port` - Port this service listens on
    ///
    /// # Errors
    /// Returns error if mDNS daemon cannot be created
    pub fn new(instance_name: String, service_type: String, port: u16) -> SongbirdResult<Self> {
        info!("Initializing mDNS discovery for {} on port {}", instance_name, port);

        let daemon = ServiceDaemon::new()
            .map_err(|e| SongbirdError::discovery(format!("Failed to create mDNS daemon: {e}")))?;

        Ok(Self {
            daemon,
            instance_name,
            service_type,
            port,
            txt_records: HashMap::new(),
            discovered: Arc::new(RwLock::new(HashMap::new())),
            active: Arc::new(RwLock::new(false)),
        })
    }

    /// Add metadata to advertise via TXT records
    #[must_use]
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.txt_records.insert(key, value);
        self
    }

    /// Add multiple metadata entries
    #[must_use]
    pub fn with_metadata_map(mut self, metadata: HashMap<String, String>) -> Self {
        self.txt_records.extend(metadata);
        self
    }

    /// Start mDNS discovery (announce our service and browse for others)
    ///
    /// This will:
    /// 1. Register and announce our service on the local network
    /// 2. Start browsing for other services of the same type
    /// 3. Continuously update the discovered services list
    ///
    /// # Errors
    /// Returns error if service registration or browsing fails
    pub async fn start(&mut self) -> SongbirdResult<()> {
        let mut active = self.active.write().await;
        if *active {
            return Ok(()); // Already started
        }

        info!("Starting mDNS discovery for {}", self.instance_name);

        // Register our service
        self.register_service().await?;

        // Start browsing for services
        self.start_browsing().await?;

        *active = true;
        info!("mDNS discovery started successfully");
        Ok(())
    }

    /// Register our service on the local network
    #[allow(clippy::unused_async)] // May add async operations in future
    async fn register_service(&self) -> SongbirdResult<()> {
        let service_hostname = format!("{}.local.", self.instance_name);
        let service_type = format!("{}.local.", self.service_type);

        let service_info = MdnsServiceInfo::new(
            &service_type,
            &self.instance_name,
            &service_hostname,
            (), // Auto-detect IP address
            self.port,
            Some(self.txt_records.clone()),
        )
        .map_err(|e| {
            SongbirdError::discovery(format!("Failed to create mDNS service info: {e}"))
        })?;

        self.daemon.register(service_info).map_err(|e| {
            SongbirdError::discovery(format!("Failed to register mDNS service: {e}"))
        })?;

        info!("Registered mDNS service: {} at port {}", self.instance_name, self.port);
        Ok(())
    }

    /// Start browsing for services on the local network
    #[allow(clippy::unused_async)] // Spawns async task
    async fn start_browsing(&self) -> SongbirdResult<()> {
        let service_type = format!("{}.local.", self.service_type);

        let receiver = self
            .daemon
            .browse(&service_type)
            .map_err(|e| SongbirdError::discovery(format!("Failed to start mDNS browsing: {e}")))?;

        // Spawn background task to handle mDNS events
        let discovered = Arc::clone(&self.discovered);
        let instance_name = self.instance_name.clone();
        let service_type_clone = self.service_type.clone();

        tokio::spawn(async move {
            info!("mDNS browser task started");

            loop {
                match receiver.recv_async().await {
                    Ok(event) => {
                        if let Err(e) = Self::handle_mdns_event(
                            event,
                            &discovered,
                            &instance_name,
                            &service_type_clone,
                        )
                        .await
                        {
                            warn!("Error handling mDNS event: {}", e);
                        }
                    }
                    Err(e) => {
                        warn!("mDNS receiver error: {}", e);
                        break;
                    }
                }
            }

            info!("mDNS browser task ended");
        });

        Ok(())
    }

    /// Handle mDNS events (service discovered, removed, etc.)
    async fn handle_mdns_event(
        event: ServiceEvent,
        discovered: &Arc<RwLock<HashMap<String, DiscoveredService>>>,
        our_instance: &str,
        service_type: &str,
    ) -> SongbirdResult<()> {
        match event {
            ServiceEvent::ServiceResolved(mdns_info) => {
                let fullname = mdns_info.get_fullname().to_string();

                // Don't add ourselves to the discovered list
                if mdns_info.get_hostname().trim_end_matches(".local.") == our_instance {
                    debug!("Ignoring our own service announcement");
                    return Ok(());
                }

                debug!("mDNS service resolved: {}", fullname);

                // Convert to our ServiceInfo format (before moving mdns_info)
                let service_info = Self::mdns_to_service_info(&mdns_info, service_type);

                let mut disc = discovered.write().await;
                disc.insert(
                    fullname.clone(),
                    DiscoveredService {
                        info: service_info,
                        _mdns_info: mdns_info,
                    },
                );

                info!("Added mDNS service: {}", fullname);
            }
            ServiceEvent::ServiceRemoved(_, fullname) => {
                debug!("mDNS service removed: {}", fullname);

                let mut disc = discovered.write().await;
                if disc.remove(&fullname).is_some() {
                    info!("Removed mDNS service: {}", fullname);
                }
            }
            ServiceEvent::SearchStarted(_) => {
                debug!("mDNS search started");
            }
            ServiceEvent::SearchStopped(_) => {
                debug!("mDNS search stopped");
            }
            ServiceEvent::ServiceFound(..) => {
                // ServiceFound events are handled by ServiceResolved
            }
        }

        Ok(())
    }

    /// Convert mDNS `ServiceInfo` to our `ServiceInfo` format
    fn mdns_to_service_info(mdns_info: &MdnsServiceInfo, service_type: &str) -> ServiceInfo {
        let hostname = mdns_info.get_hostname().to_string();
        let instance_name = hostname.trim_end_matches(".local.").to_string();
        let port = mdns_info.get_port();

        // Extract IP addresses
        let host = mdns_info
            .get_addresses()
            .iter()
            .next()
            .map_or_else(|| hostname.clone(), ToString::to_string);

        // Parse TXT records into metadata
        let mut metadata = HashMap::new();
        let mut tags = Vec::new();

        // Get properties as a map
        let properties = mdns_info.get_properties();
        for txt_property in properties.iter() {
            let key = txt_property.key();
            metadata.insert(
                key.to_string(),
                serde_json::Value::String(txt_property.val_str().to_string()),
            );
            tags.push(format!("{}={}", key, txt_property.val_str()));
        }

        ServiceInfo {
            service_id: format!("mdns:{hostname}:{port}"),
            name: instance_name.clone(),
            version: metadata
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            service_type: service_type.to_string(),
            description: metadata.get("description").and_then(|v| v.as_str()).map(str::to_string),
            endpoints: vec![],
            health_check_endpoint: metadata
                .get("health")
                .and_then(|v| v.as_str())
                .map(str::to_string),
            metadata,
            tags,
            dependencies: vec![],
            status: crate::traits::service::ServiceStatus::Running,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            instance_id: instance_name,
            host,
            port,
        }
    }

    /// Discover services currently on the local network
    ///
    /// # Errors
    /// Returns error if service discovery fails
    pub async fn discover_services(&self) -> SongbirdResult<Vec<ServiceInfo>> {
        let discovered = self.discovered.read().await;
        Ok(discovered.values().map(|d| d.info.clone()).collect())
    }

    /// Get count of discovered services
    pub async fn service_count(&self) -> usize {
        let discovered = self.discovered.read().await;
        discovered.len()
    }

    /// Check if discovery is active
    pub async fn is_active(&self) -> bool {
        *self.active.read().await
    }

    /// Stop mDNS discovery and unregister service
    ///
    /// # Errors
    /// Returns error if shutdown fails
    pub async fn stop(&mut self) -> SongbirdResult<()> {
        let mut active = self.active.write().await;
        if !*active {
            return Ok(()); // Already stopped
        }

        info!("Stopping mDNS discovery for {}", self.instance_name);

        // Shutdown the daemon (unregisters services)
        self.daemon.shutdown().map_err(|e| {
            SongbirdError::discovery(format!("Failed to shutdown mDNS daemon: {e}"))
        })?;

        *active = false;
        info!("mDNS discovery stopped");
        Ok(())
    }
}

impl Drop for MdnsDiscovery {
    fn drop(&mut self) {
        // Best effort cleanup - shutdown daemon
        let _ = self.daemon.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mdns_discovery_creation() {
        let discovery =
            MdnsDiscovery::new("test-service".to_string(), "_test._tcp".to_string(), 8080);

        assert!(discovery.is_ok());
    }

    #[test]
    fn test_with_metadata() {
        let discovery =
            MdnsDiscovery::new("test-service".to_string(), "_test._tcp".to_string(), 8080)
                .unwrap()
                .with_metadata("version".to_string(), "1.0.0".to_string())
                .with_metadata("env".to_string(), "test".to_string());

        assert_eq!(discovery.txt_records.get("version").unwrap(), "1.0.0");
        assert_eq!(discovery.txt_records.get("env").unwrap(), "test");
    }

    #[tokio::test]
    async fn test_initial_state() {
        let discovery =
            MdnsDiscovery::new("test-service".to_string(), "_test._tcp".to_string(), 8080).unwrap();

        assert!(!discovery.is_active().await);
        assert_eq!(discovery.service_count().await, 0);
    }

    #[tokio::test]
    async fn test_discover_empty() {
        let discovery =
            MdnsDiscovery::new("test-service".to_string(), "_test._tcp".to_string(), 8080).unwrap();

        let services = discovery.discover_services().await.unwrap();
        assert!(services.is_empty());
    }
}
