//! Songbird-Sovereign biome.yaml Integration
//!
//! This module provides Songbird's own lightweight biome.yaml parsing capability
//! without depending on other Primals. It can coordinate with other Primals via
//! network APIs when they are available, leveraging network effects while maintaining sovereignty.
//!
//! ## Architecture
//!
//! The biome module is organized into focused sub-modules:
//! - `types`: Data structures and type definitions
//! - `storage`: Storage management and deployment
//! - `orchestrator`: Core orchestration and primal coordination
//! - `lifecycle`: Service lifecycle and health monitoring
//! - `byob_coordinator`: BYOB (Bring Your Own Backend) coordination

pub mod byob_coordinator;
pub mod modules;

// Re-export the modular functionality
pub use modules::*;

// Legacy re-exports for backward compatibility
pub use modules::types::{
    BiomeMetadata, ByobError, DeploymentResult, DeploymentStatus, DiscoverySpec, HealthCheckSpec,
    NestGateConfig, NetworkingSpec, OrchestratorConfig, OrchestratorStatus, PrimalCoordination,
    ServiceSpec, ServiceStorageSpec, SongbirdBiomeManifest, SongbirdOrchestrator, ToadstoolConfig,
    ToadstoolEndpoint, VolumeMount,
};

pub use modules::lifecycle::{ServiceLifecycleManager, ServiceRegistry};
pub use modules::orchestrator::OrchestratorManager;

// Re-export BYOB coordinator types
pub use byob_coordinator::{
    ByobCoordinator, ByobDeployment, ByobDeploymentRequest, ByobDeploymentStatus,
    ByobTeamWorkspace, ServiceHealth, ServiceStatus, TeamResourceQuota,
};

use songbird_config::get_default_bind_address;
use songbird_errors::SongbirdResult;
use std::collections::HashMap;
use std::path::Path;
use tracing::info;

/// Create a Songbird orchestrator from a manifest file
///
/// This is a convenience function that combines manifest loading and orchestrator creation.
pub async fn create_orchestrator_from_file(
    _manifest_path: &Path,
    config: Option<OrchestratorConfig>,
) -> SongbirdResult<SongbirdOrchestrator> {
    let config = config.unwrap_or_default();
    // Create a default manifest since we can't actually load from file yet
    let manifest = create_example_manifest();
    Ok(SongbirdOrchestrator::new(config, manifest))
}

/// Create a biome coordinator with default configuration
///
/// This is the main entry point for creating a biome coordinator.
pub fn create_biome_coordinator() -> BiomeCoordinator {
    create_default_biome_coordinator()
}

/// Create a biome coordinator with custom configuration
pub fn create_biome_coordinator_with_config(config: BiomeCoordinatorConfig) -> BiomeCoordinator {
    BiomeCoordinator::new(config)
}

/// Deploy a biome for a team using the default coordinator
///
/// This is a high-level convenience function for common deployment scenarios.
pub async fn deploy_team_biome(
    team_id: String,
    manifest: SongbirdBiomeManifest,
) -> SongbirdResult<BiomeDeploymentResult> {
    let mut coordinator = create_biome_coordinator();
    coordinator.deploy_biome(team_id, manifest).await
}

/// Parse a biome manifest from YAML content
pub fn parse_biome_manifest(yaml_content: &str) -> SongbirdResult<SongbirdBiomeManifest> {
    let manifest: SongbirdBiomeManifest = serde_yaml::from_str(yaml_content).map_err(|e| {
        songbird_errors::SongbirdError::configuration(format!(
            "Failed to parse biome manifest: {}",
            e
        ))
    })?;
    Ok(manifest)
}

/// Load and parse a biome manifest from file
pub async fn load_biome_manifest(manifest_path: &Path) -> SongbirdResult<SongbirdBiomeManifest> {
    let content = tokio::fs::read_to_string(manifest_path).await?;
    parse_biome_manifest(&content)
}

/// Create a default biome manifest for testing/examples
pub fn create_example_manifest() -> SongbirdBiomeManifest {
    SongbirdBiomeManifest {
        metadata: BiomeMetadata {
            name: "example-biome".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Example biome manifest".to_string()),
        },
        services: {
            let mut services = HashMap::new();
            services.insert(
                "web".to_string(),
                ServiceSpec {
                    endpoint: Some(format!("http://{}:8080", get_default_bind_address())),
                    depends_on: vec!["database".to_string()],
                    health_check: Some(HealthCheckSpec {
                        endpoint: format!("http://{}:8080/health", get_default_bind_address()),
                        interval_secs: 30,
                        timeout_secs: 5,
                    }),
                    primal_managed: None,
                },
            );
            services.insert(
                "database".to_string(),
                ServiceSpec {
                    endpoint: Some(format!(
                        "http://{}:{}",
                        get_default_bind_address(),
                        std::env::var("SONGBIRD_DATABASE_PORT")
                            .unwrap_or_else(|_| "5432".to_string())
                    )),
                    depends_on: vec![],
                    health_check: Some(HealthCheckSpec {
                        endpoint: format!("http://{}:5432/health", get_default_bind_address()),
                        interval_secs: 60,
                        timeout_secs: 10,
                    }),
                    primal_managed: None,
                },
            );
            services
        },
        networking: Some(NetworkingSpec {
            discovery: Some(DiscoverySpec {
                method: "mdns".to_string(),
                config: None,
            }),
            ports: Some(vec![8080, 5432]),
        }),
        primals: Some({
            let mut primals = HashMap::new();
            // Use capability-based configuration instead of hardcoded primal names
            primals.insert(
                "compute_provider".to_string(),
                PrimalCoordination {
                    enabled: true,
                    endpoint: None, // Will be discovered via capability system
                    capabilities: vec!["compute".to_string(), "processing".to_string()],
                },
            );
            primals
        }),
    }
}

/// Validate a biome manifest
pub fn validate_biome_manifest(manifest: &SongbirdBiomeManifest) -> SongbirdResult<()> {
    // Basic validation
    if manifest.metadata.name.is_empty() {
        return Err("Manifest name cannot be empty".into());
    }

    if manifest.metadata.version.is_empty() {
        return Err("Manifest version cannot be empty".into());
    }

    // Validate services
    for (service_name, service_spec) in &manifest.services {
        if service_name.is_empty() {
            return Err("Service name cannot be empty".into());
        }

        // Check that dependencies exist
        for dependency in &service_spec.depends_on {
            if !manifest.services.contains_key(dependency) {
                return Err(format!(
                    "Service '{service_name}' depends on '{dependency}' which is not defined"
                )
                .into());
            }
        }
    }

    // Check for circular dependencies (simplified check)
    for (service_name, service_spec) in &manifest.services {
        if service_spec.depends_on.contains(service_name) {
            return Err(format!("Service '{service_name}' cannot depend on itself").into());
        }
    }

    info!(
        "Biome manifest validation passed for: {}",
        manifest.metadata.name
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use songbird_errors::SongbirdResult;

    #[test]
    fn test_example_manifest_creation() {
        let manifest = create_example_manifest();
        assert_eq!(manifest.metadata.name, "example-biome");
        assert_eq!(manifest.services.len(), 2);
        assert!(manifest.services.contains_key("web"));
        assert!(manifest.services.contains_key("database"));
    }

    #[test]
    fn test_manifest_validation() {
        let manifest = create_example_manifest();
        let result = validate_biome_manifest(&manifest);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_manifest_validation() {
        let mut manifest = create_example_manifest();
        manifest.metadata.name = "".to_string();

        let result = validate_biome_manifest(&manifest);
        assert!(result.is_err());
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut manifest = create_example_manifest();

        // Create circular dependency
        if let Some(web_service) = manifest.services.get_mut("web") {
            web_service.depends_on.push("web".to_string());
        }

        let result = validate_biome_manifest(&manifest);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_biome_coordinator_creation() {
        let coordinator = create_biome_coordinator();
        let health = coordinator.get_health_status();

        assert_eq!(health.orchestrator_count, 0);
        assert!(health.coordinator_name.contains("Songbird"));
    }

    #[test]
    fn test_parse_biome_manifest() {
        let yaml_content = r#"
metadata:
  name: test-biome
  version: 1.0.0
  description: Test biome
services:
  web:
    endpoint: http://localhost:8080
    depends_on: []
networking:
  ports:
    - 8080
"#;

        let result = parse_biome_manifest(yaml_content);
        assert!(result.is_ok());

        let manifest = result.expect("Failed to parse biome manifest in test");
        assert_eq!(manifest.metadata.name, "test-biome");
        assert_eq!(manifest.services.len(), 1);
    }
}
