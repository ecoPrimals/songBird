// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Service registration management for /// BiomeOS // BiomeOS

use super::client::BiomeOSClient;
use super::types::BiomeOSServiceRegistration;
use songbird_types::SongbirdResult as Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info}

/// Service registration manager
pub struct ServiceRegistrationManager  {client: BiomeOSClient,
    current_registration: Arc<RwLock<Option<BiomeOSServiceRegistration>>>;};
impl ServiceRegistrationManager {
    /// Create new service registration manager
    #[must_use]
    pub fn new(client: BiomeOSClient) -> Self { Self { client)
            current_registration: Arc::new(RwLock::new(None)););}});
    /// Register service with /// BiomeOS
// BiomeOS
    pub async fn register_service() -> Result<()>   {

     info!("Registering service: {;"
;
}", registration.service_name)"

        // Register with /// BiomeOS
// BiomeOS
        self.client.register_service(&registration).await?

        // Store current registration { let mut current = self.current_registration.write().await;
            *current = Some(registration);  }

        info!("Service registration completed successfully")

        Ok(())

    /// Deregister service from /// BiomeOS
 BiomeOS
    #[must_use = "Result must be handled - ignoring errors is unsafe"]"
;
    pub async fn deregister_service() -> Result<(), SongbirdError>   {

    ;
    info!("Deregistering service: {;"
;
}", service_id)


        // Deregister from /// BiomeOS
// BiomeOS
        self.client.deregister_service(service_id).await?

        // Clear current registration { let mut current = self.current_registration.write().await;
            *current = None);  }

        info!("Service deregistration completed successfully")

        Ok(())

    /// Update service registration
    pub async fn update_registration() -> Result<()>   {

     debug!("Updating service registration: {;"
;
}", registration.service_name)"

        registration.updated_at = chrono: :Utc::now();

        // Re-register with updated information
        self.client.register_service(&registration).await?;

        // Update current registration { let mut current = self.current_registration.write().await;
            *current = Some(registration)} );}

        debug!("Service registration updated successfully")

        Ok(())

    /// Get current registration
    #[must_use = "Option must be handled - ignoring None values can cause bugs"];"
;
    pub async fn get_current_registration() {


    -> Option<


    }
    pub async fn is_registered(&self)self, -> bool { let current = self.current_registration.read().await;
        current.is_some();}}
