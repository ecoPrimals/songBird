//! mDNS (Multicast DNS) Discovery Backend
//!
//! Complete production implementation for discovering capabilities via mDNS.
//! Enables zero-configuration service discovery on local networks.

use super::{CapabilityProvider, CapabilityRequest, Protocol};
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::net::IpAddr;
use std::time::Duration;
use tracing::{debug, info, warn};

/// mDNS discovery backend
///
/// Discovers services on the local network using Multicast DNS (mDNS/Bonjour/Avahi).
/// Supports automatic discovery of services advertising songbird capabilities.
#[derive(Debug, Clone)]
pub struct MdnsDiscovery {
    /// Service type to query (e.g., "_songbird._tcp.local.")
    service_type: String,
    /// Discovery timeout
    timeout: Duration,
    /// Interface to scan (None = all interfaces)
    interface: Option<String>,
}

impl MdnsDiscovery {
    /// Create a new mDNS discovery backend
    ///
    /// # Arguments
    /// * `service_type` - Optional service type (defaults to "_songbird._tcp.local.")
    ///
    /// # Examples
    /// ```no_run
    /// use songbird_config::capability_based_runtime_discovery::mdns::MdnsDiscovery;
    ///
    /// let discovery = MdnsDiscovery::new(None);
    /// ```
    #[must_use]
    pub fn new(service_type: Option<String>) -> Self {
        Self {
            service_type: service_type.unwrap_or_else(|| "_songbird._tcp.local.".to_string()),
            timeout: Duration::from_secs(3),
            interface: None,
        }
    }

    /// Set the network interface to scan
    #[must_use]
    pub fn with_interface(mut self, interface: impl Into<String>) -> Self {
        self.interface = Some(interface.into());
        self
    }

    /// Set the discovery timeout
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Discover a capability provider via mDNS
    ///
    /// # Errors
    /// Returns error if no services found or mDNS scanning fails
    pub async fn discover(
        &self,
        request: &CapabilityRequest,
    ) -> SongbirdResult<CapabilityProvider> {
        debug!("Scanning local network via mDNS for capability: {}", request.capability);

        // Perform mDNS scan
        let services = self.scan_network(request).await?;

        if services.is_empty() {
            return Err(SongbirdError::discovery(format!(
                "No services found via mDNS for capability: {}",
                request.capability
            )));
        }

        // Select best match
        let best_match = Self::select_best_service(&services, request)?;

        info!(
            "Discovered provider '{}' for capability '{}' via mDNS at {}",
            best_match.name, request.capability, best_match.endpoint
        );

        Ok(best_match)
    }

    /// Discover services by capability (convenience method)
    ///
    /// # Errors
    /// Returns error if no services found or mDNS scanning fails
    pub async fn discover_by_capability(
        &self,
        capability: &str,
        timeout: Option<Duration>,
    ) -> SongbirdResult<Vec<MdnsServiceInfo>> {
        debug!("Discovering capability '{}' via mDNS", capability);

        // Apply custom timeout if provided
        let discovery = if let Some(custom_timeout) = timeout {
            Self {
                service_type: self.service_type.clone(),
                timeout: custom_timeout,
                interface: self.interface.clone(),
            }
        } else {
            self.clone()
        };

        // Create request
        let request = CapabilityRequest {
            capability: capability.to_string(),
            required_features: Vec::new(),
            optional_features: Vec::new(),
            preferences: Default::default(),
            min_sla: None,
        };

        // Scan network
        let services = discovery.scan_network(&request).await?;

        // Convert to simplified service info format
        let service_infos: Vec<MdnsServiceInfo> = services
            .into_iter()
            .map(|s| MdnsServiceInfo {
                name: s.name,
                address: format!("{}:{}", s.address, s.endpoint.rsplit(':').next().unwrap_or("80")),
                endpoint: s.endpoint,
                metadata: s.metadata,
            })
            .collect();

        Ok(service_infos)
    }

    /// Scan the local network for services
    async fn scan_network(&self, request: &CapabilityRequest) -> SongbirdResult<Vec<MdnsService>> {
        // Use mdns-sd crate for production-ready mDNS
        let mdns = mdns_sd::ServiceDaemon::new()
            .map_err(|e| SongbirdError::network(format!("Failed to create mDNS daemon: {e}")))?;

        // Browse for services
        let receiver = mdns
            .browse(&self.service_type)
            .map_err(|e| SongbirdError::discovery(format!("mDNS browse failed: {e}")))?;

        let mut services = Vec::new();
        let deadline = tokio::time::Instant::now() + self.timeout;

        // Collect services until timeout
        while tokio::time::Instant::now() < deadline {
            match tokio::time::timeout_at(deadline, async { receiver.recv_async().await }).await {
                Ok(Ok(event)) => {
                    // Only process ServiceResolved events
                    if let mdns_sd::ServiceEvent::ServiceResolved(info) = event {
                        // Check if service advertises the requested capability
                        if let Some(service) = Self::parse_service_info(&info, request) {
                            services.push(service);
                        }
                    }
                    // ServiceRemoved and other events are ignored
                }
                Ok(Err(e)) => {
                    warn!("mDNS receiver error: {e}");
                    break;
                }
                Err(_) => {
                    // Timeout reached
                    break;
                }
            }
        }

        // Shutdown mDNS daemon
        mdns.shutdown()
            .map_err(|e| SongbirdError::network(format!("mDNS shutdown failed: {e}")))?;

        Ok(services)
    }

    /// Parse mDNS service info into our service structure
    ///
    /// Pure function - doesn't need instance state.
    /// Made an associated function for clarity and to allow calling without self.
    fn parse_service_info(
        info: &mdns_sd::ServiceInfo,
        request: &CapabilityRequest,
    ) -> Option<MdnsService> {
        // Extract capability from TXT records
        let properties = info.get_properties();
        let capability = properties
            .get_property_val_str("capability")
            .or_else(|| properties.get_property_val_str("cap"))?;

        // Filter by requested capability
        if capability != request.capability {
            return None;
        }

        // Extract features
        let features_str = properties.get_property_val_str("features").unwrap_or_default();
        let features: Vec<String> = features_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        // Build endpoint from address and port
        let addresses = info.get_addresses();
        let addr = addresses.iter().next()?;
        let endpoint = format!("http://{}:{}", addr, info.get_port());

        // Extract protocol
        let protocol_str = properties.get_property_val_str("protocol").unwrap_or("http");
        let protocol = match protocol_str {
            "https" => Protocol::Https,
            "grpc" => Protocol::Grpc,
            "ws" | "websocket" => Protocol::WebSocket,
            _ => Protocol::Http,
        };

        // Extract priority (for selection)
        let priority = properties
            .get_property_val_str("priority")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(100);

        // Build metadata from all TXT records
        let mut metadata = HashMap::new();
        for property in properties.iter() {
            let key_str = property.key();
            let value_str = property.val_str();
            metadata.insert(key_str.to_string(), value_str.to_string());
        }

        Some(MdnsService {
            name: info.get_fullname().to_string(),
            endpoint,
            protocol,
            features,
            metadata,
            priority,
            address: *addr,
        })
    }

    /// Select the best service from discovered options
    fn select_best_service(
        services: &[MdnsService],
        request: &CapabilityRequest,
    ) -> SongbirdResult<CapabilityProvider> {
        // Filter by required features
        let mut candidates: Vec<_> = services
            .iter()
            .filter(|s| request.required_features.iter().all(|req| s.features.contains(req)))
            .collect();

        if candidates.is_empty() {
            return Err(SongbirdError::discovery(
                "No mDNS services support required features".to_string(),
            ));
        }

        // Sort by priority (lower is better)
        candidates.sort_by_key(|s| s.priority);

        // Prefer IPv6 localhost, then IPv4 localhost, then any local
        candidates.sort_by_key(|s| match s.address {
            IpAddr::V6(addr) if addr.is_loopback() => 0,
            IpAddr::V4(addr) if addr.is_loopback() => 1,
            IpAddr::V6(_) => 2,
            IpAddr::V4(_) => 3,
        });

        let selected = candidates.first().ok_or_else(|| {
            SongbirdError::discovery("No suitable mDNS service found".to_string())
        })?;

        Ok(CapabilityProvider {
            name: selected.name.clone(),
            capability: request.capability.clone(),
            endpoint: selected.endpoint.clone(),
            protocol: selected.protocol.clone(),
            features: selected.features.clone(),
            metadata: selected.metadata.clone(),
        })
    }
}

/// Service discovered via mDNS
#[derive(Debug, Clone)]
struct MdnsService {
    name: String,
    endpoint: String,
    protocol: Protocol,
    features: Vec<String>,
    metadata: HashMap<String, String>,
    priority: u32,
    address: IpAddr,
}

/// Simplified service info for discovery results
#[derive(Debug, Clone)]
pub struct MdnsServiceInfo {
    /// Service name
    pub name: String,
    /// Service address (IP:port)
    pub address: String,
    /// Full endpoint URL
    pub endpoint: String,
    /// Service metadata
    pub metadata: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mdns_discovery_creation() {
        let discovery = MdnsDiscovery::new(None);
        assert_eq!(discovery.service_type, "_songbird._tcp.local.");
        assert_eq!(discovery.timeout, Duration::from_secs(3));
    }

    #[test]
    fn test_mdns_with_custom_service_type() {
        let discovery = MdnsDiscovery::new(Some("_custom._tcp.local.".to_string()));
        assert_eq!(discovery.service_type, "_custom._tcp.local.");
    }

    #[test]
    fn test_mdns_with_interface() {
        let discovery = MdnsDiscovery::new(None).with_interface("eth0");
        assert_eq!(discovery.interface, Some("eth0".to_string()));
    }

    #[test]
    fn test_mdns_with_timeout() {
        let discovery = MdnsDiscovery::new(None).with_timeout(Duration::from_secs(10));
        assert_eq!(discovery.timeout, Duration::from_secs(10));
    }
}
