//! Service registration management for BiomeOS

use super::client::BiomeOSClient;
use super::types::BiomeOSServiceRegistration;
use songbird_errors::SongbirdResult;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Service registration manager
pub struct ServiceRegistrationManager {
    client: BiomeOSClient,
    current_registration: Arc<RwLock<Option<BiomeOSServiceRegistration>>>,
}

impl ServiceRegistrationManager {
    /// Create new service registration manager
    pub fn new(client: BiomeOSClient) -> Self {
        Self {
            client,
            current_registration: Arc::new(RwLock::new(None)),
        }
    }

    /// Register service with BiomeOS
    pub async fn register_service(
        &mut self,
        registration: BiomeOSServiceRegistration,
    ) -> SongbirdResult<()> {
        info!("Registering service: {}", registration.service_name);

        // Register with BiomeOS
        self.client.register_service(&registration).await?;

        // Store current registration
        {
            let mut current = self.current_registration.write().await;
            *current = Some(registration);
        }

        info!("Service registration completed successfully");
        Ok(())
    }

    /// Deregister service from BiomeOS
    pub async fn deregister_service(&self, service_id: &str) -> SongbirdResult<()> {
        info!("Deregistering service: {}", service_id);

        // Deregister from BiomeOS
        self.client.deregister_service(service_id).await?;

        // Clear current registration
        {
            let mut current = self.current_registration.write().await;
            *current = None;
        }

        info!("Service deregistration completed successfully");
        Ok(())
    }

    /// Update service registration
    pub async fn update_registration(
        &mut self,
        mut registration: BiomeOSServiceRegistration,
    ) -> SongbirdResult<()> {
        debug!(
            "Updating service registration: {}",
            registration.service_name
        );

        registration.updated_at = chrono::Utc::now();

        // Re-register with updated information
        self.client.register_service(&registration).await?;

        // Update current registration
        {
            let mut current = self.current_registration.write().await;
            *current = Some(registration);
        }

        debug!("Service registration updated successfully");
        Ok(())
    }

    /// Get current registration
    pub async fn get_current_registration(&self) -> Option<BiomeOSServiceRegistration> {
        let current = self.current_registration.read().await;
        current.clone()
    }

    /// Check if service is registered
    pub async fn is_registered(&self) -> bool {
        let current = self.current_registration.read().await;
        current.is_some()
    }
}
