//! Universal service discovery factory for creating capability-based adapters

use super::backends::{
    StaticServiceDiscovery, UniversalContainerOrchestration, UniversalServiceDiscovery,
};
use super::core::DiscoveryConfig;
use crate::traits::ServiceDiscovery;
use songbird_types::errors::SongbirdResult;
type Result<T> = SongbirdResult<T>;
use tracing::{debug, info};

/// Universal service discovery factory that creates capability-based adapters
pub struct UniversalDiscoveryFactory;

impl UniversalDiscoveryFactory {
    /// Create universal service discovery with auto-detection
    pub async fn create_auto_detect() -> Result<Box<dyn ServiceDiscovery>> {
        info!("🔍 Creating universal service discovery with auto-detection");

        // Try universal service discovery first
        if let Ok(universal) = UniversalServiceDiscovery::new().await {
            info!("✅ Universal service discovery initialized");
            return Ok(Box::new(universal));
        }

        // Fallback to container orchestration discovery
        if let Ok(container) = UniversalContainerOrchestration::new().await {
            info!("✅ Universal container orchestration initialized");
            return Ok(Box::new(container));
        }

        // Final fallback to static discovery
        info!("⚠️ Falling back to static service discovery");
        Ok(Box::new(StaticServiceDiscovery::new()))
    }

    /// Create service discovery based on capability requirements
    pub async fn create_for_capability(capability: &str) -> Result<Box<dyn ServiceDiscovery>> {
        match capability {
            "service_discovery" | "http_registry" => {
                debug!(
                    "Creating universal service discovery for capability: {}",
                    capability
                );
                Ok(Box::new(UniversalServiceDiscovery::new().await?))
            }
            "container_orchestration" | "kubernetes" | "docker" => {
                debug!(
                    "Creating universal container orchestration for capability: {}",
                    capability
                );
                Ok(Box::new(UniversalContainerOrchestration::new().await?))
            }
            "static" | "file_based" => {
                debug!(
                    "Creating static service discovery for capability: {}",
                    capability
                );
                Ok(Box::new(StaticServiceDiscovery::new()))
            }
            _ => {
                debug!("Unknown capability '{}', using auto-detection", capability);
                Self::create_auto_detect().await
            }
        }
    }

    /// Create service discovery with environment-based detection
    pub async fn create_from_environment() -> Result<Box<dyn ServiceDiscovery>> {
        info!("🌍 Creating service discovery from environment detection");

        // Check for service registry environment variables
        if std::env::var("SERVICE_REGISTRY_URL").is_ok()
            || std::env::var("CONSUL_HTTP_ADDR").is_ok()
            || std::env::var("EUREKA_SERVER_URL").is_ok()
        {
            debug!("Detected service registry environment variables");
            return Ok(Box::new(UniversalServiceDiscovery::new().await?));
        }

        // Check for container orchestration environment variables
        if std::env::var("KUBERNETES_SERVICE_HOST").is_ok()
            || std::env::var("DOCKER_HOST").is_ok()
            || std::path::Path::new("/.dockerenv").exists()
        {
            debug!("Detected container orchestration environment");
            return Ok(Box::new(UniversalContainerOrchestration::new().await?));
        }

        // Default to auto-detection
        Self::create_auto_detect().await
    }

    /// Create service discovery based on configuration
    pub async fn create_for_config(config: &DiscoveryConfig) -> Result<Box<dyn ServiceDiscovery>> {
        match config.backend.as_str() {
            "static" => {
                info!("Creating static service discovery backend");
                Ok(Box::new(StaticServiceDiscovery::new()))
            }
            "universal" => {
                info!("Creating universal service discovery backend");
                Ok(Box::new(UniversalServiceDiscovery::new().await?))
            }
            "container" | "kubernetes" | "docker" => {
                info!("Creating universal container orchestration backend");
                Ok(Box::new(UniversalContainerOrchestration::new().await?))
            }
            _ => {
                info!(
                    "Unknown backend '{}' - using auto-detection",
                    config.backend
                );
                Self::create_auto_detect().await
            }
        }
    }
}
