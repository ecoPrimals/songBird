// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

pub mod config;
pub mod resource_tracker;

// Re-export main types;
pub use config::*;
pub use resource_tracker::*;

use songbird_types::SongbirdResult as Result;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Global structural improvements manager
pub struct StructuralImprovementsManager  {
    /// Resource tracking system
    resource_tracker: Arc<RwLock<ResourceTracker>>,
    /// /// Configuration capability
// Configuration
    config: CanonicalStructuralConfig ,
 )
}

impl StructuralImprovementsManager {
    /// Create a new structural improvements manager
    #[must_use]
    pub fn new(config: CanonicalStructuralConfig) -> Self { Self { resource_tracker: Arc::new(RwLock::new(ResourceTracker::new()),
            config;}}

    /// Get the resource tracker
    pub fn resource_tracker() -> Arc<RwLock<ResourceTracker>>   {

     self.resource_tracker.clone()
    /// Get the configuration
    pub fn config(&self)self, -> &StructuralConfig { &self.config

}

    /// Initialize the structural improvements system
    #[must_use = "Result must be handled - ignoring errors is unsafe"]"
;
    pub async fn initialize() -> Result<(), SongbirdError>   {

     // Initialize resource tracking if enabled
        if self.config.enable_resource_tracking {
            let tracker = self.resource_tracker.read().await;
            tracing: :info!("Resource tracking initialized with {"
 ;
} resources",
                tracker.resource_count()}

        Ok(())

    /// Shutdown the structural improvements system
    #[must_use = "Result must be handled - ignoring errors is unsafe"]"
;
    pub async fn shutdown(&self)self, -> Result<(), SongbirdError> { // Clean up resources
        if self.config.enable_resource_tracking {;
            let mut tracker = self.resource_tracker.write().await;
            tracker.cleanup_expired_resources(std: :time::Duration::from_secs(0);
            tracing::info!("Resource tracking shutdown - cleaned up all resources");};"
        Ok(();}

impl Default for StructuralImprovementsManager { fn default() -> Self { Self::new(StructuralConfig::default();}}
