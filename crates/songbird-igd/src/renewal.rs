//! TTL renewal task for port mappings
//!
//! Port mappings have TTLs (typically 86400s / 24h for UPnP).
//! This module spawns a background task that renews them at half-TTL.

use crate::gateway::Gateway;
use crate::mapping::PortMapping;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Manages automatic renewal of port mappings
#[derive(Debug)]
pub struct RenewalManager {
    /// Active mappings being renewed
    mappings: Arc<RwLock<Vec<PortMapping>>>,
    /// Whether the renewal task is running
    running: Arc<RwLock<bool>>,
}

impl RenewalManager {
    /// Create a new renewal manager
    pub fn new() -> Self {
        Self {
            mappings: Arc::new(RwLock::new(Vec::new())),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Add a mapping to be renewed
    pub async fn add_mapping(&self, mapping: PortMapping) {
        let mut mappings = self.mappings.write().await;
        mappings.push(mapping);
    }

    /// Remove a mapping from renewal
    pub async fn remove_mapping(&self, external_port: u16) {
        let mut mappings = self.mappings.write().await;
        mappings.retain(|m| m.external_port != external_port);
    }

    /// Get all active mappings
    pub async fn get_mappings(&self) -> Vec<PortMapping> {
        self.mappings.read().await.clone()
    }

    /// Start the renewal background task
    pub fn spawn_renewal_task(&self, gateway: Arc<Gateway>) -> tokio::task::JoinHandle<()> {
        let mappings = self.mappings.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            *running.write().await = true;
            info!("Port mapping renewal task started");

            loop {
                // Check every 60 seconds for mappings that need renewal
                tokio::time::sleep(Duration::from_secs(60)).await;

                if !*running.read().await {
                    info!("Port mapping renewal task stopping");
                    break;
                }

                let current_mappings = mappings.read().await.clone();

                for mapping in &current_mappings {
                    if !mapping.active {
                        continue;
                    }

                    if mapping.needs_renewal() {
                        debug!(
                            "Renewing port mapping: {}:{} (TTL expired at half-life)",
                            mapping.external_port,
                            mapping.protocol.as_str()
                        );

                        match gateway
                            .map_port(
                                mapping.external_port,
                                mapping.internal_port,
                                mapping.protocol.as_str(),
                                mapping.lease_duration,
                            )
                            .await
                        {
                            Ok(renewed) => {
                                info!(
                                    "Port mapping renewed: {}:{} (TTL: {}s)",
                                    renewed.external_port,
                                    renewed.protocol.as_str(),
                                    renewed.lease_duration
                                );

                                // Update the mapping in our list
                                let mut current = mappings.write().await;
                                if let Some(m) = current
                                    .iter_mut()
                                    .find(|m| m.external_port == renewed.external_port)
                                {
                                    m.created_at = renewed.created_at;
                                    m.lease_duration = renewed.lease_duration;
                                }
                            }
                            Err(e) => {
                                warn!(
                                    "Port mapping renewal failed for {}:{}: {}. Will retry.",
                                    mapping.external_port,
                                    mapping.protocol.as_str(),
                                    e
                                );
                            }
                        }
                    }
                }
            }
        })
    }

    /// Stop the renewal task
    pub async fn stop(&self) {
        *self.running.write().await = false;
        info!("Port mapping renewal task signaled to stop");
    }

    /// Cleanup: remove all mappings from the gateway
    pub async fn cleanup(&self, gateway: &Gateway) {
        let mappings = self.mappings.read().await;
        for mapping in mappings.iter() {
            if mapping.active {
                match gateway
                    .unmap_port(mapping.external_port, mapping.protocol.as_str())
                    .await
                {
                    Ok(()) => {
                        info!(
                            "Cleaned up port mapping: {}:{}",
                            mapping.external_port,
                            mapping.protocol.as_str()
                        );
                    }
                    Err(e) => {
                        warn!(
                            "Failed to clean up port mapping {}:{}: {}",
                            mapping.external_port,
                            mapping.protocol.as_str(),
                            e
                        );
                    }
                }
            }
        }
    }
}

impl Default for RenewalManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapping::{PortMappingRequest, Protocol};
    use std::net::{IpAddr, Ipv4Addr};

    #[tokio::test]
    async fn test_renewal_manager_add_remove() {
        let manager = RenewalManager::new();

        let req = PortMappingRequest::new(
            3492,
            3492,
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 144)),
            Protocol::Tcp,
        );
        let mapping = PortMapping::from_request(&req);

        manager.add_mapping(mapping).await;
        assert_eq!(manager.get_mappings().await.len(), 1);

        manager.remove_mapping(3492).await;
        assert_eq!(manager.get_mappings().await.len(), 0);
    }
}

