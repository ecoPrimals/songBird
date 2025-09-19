pub mod config;
pub mod resource_tracker;

// Re-export main types
pub use config::*;
pub use resource_tracker::*;

use songbird_errors::SongbirdResult;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Global structural improvements manager
pub struct StructuralImprovementsManager {
    /// Resource tracking system
    resource_tracker: Arc<RwLock<ResourceTracker>>,
    /// Configuration
    config: StructuralConfig,
}

impl StructuralImprovementsManager {
    /// Create a new structural improvements manager
    pub fn new(config: StructuralConfig) -> Self {
        Self {
            resource_tracker: Arc::new(RwLock::new(ResourceTracker::new())),
            config,
        }
    }

    /// Get the resource tracker
    pub fn resource_tracker(&self) -> Arc<RwLock<ResourceTracker>> {
        self.resource_tracker.clone()
    }

    /// Get the configuration
    pub fn config(&self) -> &StructuralConfig {
        &self.config
    }

    /// Initialize the structural improvements system
    pub async fn initialize(&self) -> SongbirdResult<()> {
        // Initialize resource tracking if enabled
        if self.config.enable_resource_tracking {
            let tracker = self.resource_tracker.read().await;
            tracing::info!(
                "Resource tracking initialized with {} resources",
                tracker.resource_count()
            );
        }

        Ok(())
    }

    /// Shutdown the structural improvements system
    pub async fn shutdown(&self) -> SongbirdResult<()> {
        // Clean up resources
        if self.config.enable_resource_tracking {
            let mut tracker = self.resource_tracker.write().await;
            tracker.cleanup_expired_resources(std::time::Duration::from_secs(0));
            tracing::info!("Resource tracking shutdown - cleaned up all resources");
        }

        Ok(())
    }
}

impl Default for StructuralImprovementsManager {
    fn default() -> Self {
        Self::new(StructuralConfig::default())
    }
}
