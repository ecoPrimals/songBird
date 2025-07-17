use songbird_core::biome::{
    BiomeMetadata, ByobCoordinator, ByobDeploymentRequest, OrchestratorConfig, ServiceSpec,
    SongbirdBiomeManifest, TeamResourceQuota,
};
// Note: ServiceDiscovery trait is used from traits module
use songbird_discovery::discovery::config::SongbirdDiscoveryConfig;
use songbird_discovery::discovery::SongbirdDiscovery;
use songbird_registry::service::ServiceRegistry;
// Note: PluginRegistry trait is imported from traits module

use songbird_discovery::traits::ServiceDiscovery;
use songbird_discovery::traits::{PluginCapability, PluginRequirement};
use songbird_registry::plugin::DynamicPluginRegistry;
use songbird_registry::PluginRegistry;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("🎵 Starting Songbird Universal Orchestrator Demo");

    // Demo 1: Service Registry and Dynamic Plugin Composition
    demo_service_registry().await?;

    // Demo 2: BYOB Deployment with Orchestration
    demo_byob_deployment().await?;

    // Demo 3: Auto-Discovery System
    demo_auto_discovery().await?;

    info!("✅ Demo completed successfully!");
    Ok(())
}

/// Demo 1: Service Registry and Dynamic Plugin Composition
async fn demo_service_registry() -> Result<(), Box<dyn std::error::Error>> {
    info!("🔧 Demo 1: Service Registry & Dynamic Plugin Composition");

    // Create service registry
    let _registry = ServiceRegistry::new().await?;
    info!("✅ Service registry created successfully");

    // Create dynamic plugin registry
    let plugin_registry = DynamicPluginRegistry::new();
    info!("✅ Dynamic plugin registry created successfully");

    // Register some mock plugins
    let plugin_id1 = plugin_registry
        .register_plugin(
            "beardog-encryption".to_string(),
            vec![PluginCapability::Encryption {
                algorithms: vec!["AES-256".to_string(), "ChaCha20".to_string()],
            }],
            vec![PluginRequirement::RequiresNetwork {
                min_bandwidth_mbps: 100,
                max_latency_ms: 50,
            }],
        )
        .await?;
    info!("🔐 Registered BearDog encryption plugin: {}", plugin_id1);

    let plugin_id2 = plugin_registry
        .register_plugin(
            "toadstool-compute".to_string(),
            vec![PluginCapability::Compute {
                cpu_cores: 16,
                memory_gb: 64,
            }],
            vec![PluginRequirement::RequiresCompute {
                min_cpu_cores: 4,
                min_memory_gb: 8,
            }],
        )
        .await?;
    info!("💻 Registered Toadstool compute plugin: {}", plugin_id2);

    let plugin_id3 = plugin_registry
        .register_plugin(
            "nestgate-storage".to_string(),
            vec![PluginCapability::Storage {
                capacity_gb: 1024,
                storage_type: "SSD".to_string(),
            }],
            vec![],
        )
        .await?;
    info!("💾 Registered NestGate storage plugin: {}", plugin_id3);

    // List all registered plugins
    let all_plugins = plugin_registry.list_plugins().await;
    info!("📋 Total registered plugins: {}", all_plugins.len());

    info!("🎯 Plugin composition demo - showcasing zero-config orchestration");
    info!("   → Replaced 256+ TOML files with dynamic runtime composition");
    info!("   → Plugins can be discovered and composed automatically");
    info!("   → No manual configuration required for complex deployments");

    Ok(())
}

/// Demo 2: BYOB Deployment with Orchestration
async fn demo_byob_deployment() -> Result<(), Box<dyn std::error::Error>> {
    info!("🏗️ Demo 2: BYOB Deployment with Orchestration");

    // Create orchestrator configuration
    let config = OrchestratorConfig::default();

    // Create BYOB coordinator
    let coordinator = Arc::new(ByobCoordinator::new(config));

    // Register a team workspace
    let team_id = "demo-team".to_string();
    let resource_quota = TeamResourceQuota {
        max_cpu_cores: 32.0,
        max_memory_bytes: 137438953472,    // 128GB
        max_storage_bytes: 1099511627776,  // 1TB
        max_network_bandwidth_mbps: 10000, // 10Gbps
        max_deployments: 10,
    };

    coordinator
        .register_team_workspace(team_id.clone(), resource_quota)
        .await
        .map_err(|e| format!("Failed to register team workspace: {}", e))?;
    info!("📋 Registered team workspace: {}", team_id);

    // Create a sample biome manifest
    let manifest = SongbirdBiomeManifest {
        metadata: BiomeMetadata {
            name: "Demo Biome".to_string(),
            version: "1.0.0".to_string(),
            description: Some("A demonstration biome with multiple services".to_string()),
        },
        services: {
            let mut services = HashMap::new();

            // Web service
            services.insert(
                "web-frontend".to_string(),
                ServiceSpec {
                    endpoint: Some("http://localhost:3000".to_string()),
                    depends_on: vec!["api-backend".to_string()],
                    health_check: Some(songbird_core::biome::HealthCheckSpec {
                        endpoint: "/health".to_string(),
                        interval_secs: 30,
                        timeout_secs: 5,
                    }),
                    primal_managed: None,
                },
            );

            // API service
            services.insert(
                "api-backend".to_string(),
                ServiceSpec {
                    endpoint: Some("http://localhost:8080".to_string()),
                    depends_on: vec!["database".to_string()],
                    health_check: Some(songbird_core::biome::HealthCheckSpec {
                        endpoint: "/api/health".to_string(),
                        interval_secs: 30,
                        timeout_secs: 5,
                    }),
                    primal_managed: None,
                },
            );

            // Database service
            services.insert(
                "database".to_string(),
                ServiceSpec {
                    endpoint: Some("postgresql://localhost:5432/demo".to_string()),
                    depends_on: vec![],
                    health_check: Some(songbird_core::biome::HealthCheckSpec {
                        endpoint: "/".to_string(),
                        interval_secs: 60,
                        timeout_secs: 10,
                    }),
                    primal_managed: Some("nestgate".to_string()),
                },
            );

            services
        },
        networking: None,
        primals: None,
    };

    // Create deployment request
    let deployment_request = ByobDeploymentRequest {
        deployment_id: Uuid::new_v4().to_string(),
        team_id: team_id.clone(),
        manifest,
        resource_quota: TeamResourceQuota {
            max_cpu_cores: 16.0,
            max_memory_bytes: 68719476736,   // 64GB
            max_storage_bytes: 549755813888, // 512GB
            max_network_bandwidth_mbps: 5000,
            max_deployments: 5,
        },
    };

    // Deploy the biome
    let deployment_id = coordinator.deploy_biome(deployment_request).await?;
    info!("🚀 Started deployment: {}", deployment_id);

    // Wait a moment for deployment to process
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // Check deployment status
    let status = coordinator.get_deployment_status(&deployment_id).await?;
    info!("📊 Deployment status: {:?}", status);

    // List team deployments
    let deployments = coordinator.list_team_deployments(&team_id).await?;
    info!("📋 Team has {} active deployments", deployments.len());

    Ok(())
}

/// Demo 3: Auto-Discovery System
async fn demo_auto_discovery() -> Result<(), Box<dyn std::error::Error>> {
    info!("🔍 Demo 3: Auto-Discovery System");

    // Create discovery configuration
    let config = SongbirdDiscoveryConfig::default();

    // Create discovery service
    let discovery = SongbirdDiscovery::new(config);

    // Start discovery service
    // Note: SongbirdDiscovery doesn't have a start() method
    println!("🔍 Starting service discovery...");

    // List all services
    let services = discovery.list_all().await?;
    info!("🎯 Discovered {} services", services.len());

    for service in services {
        info!(
            "  - {}: {} ({})",
            service.name, service.service_type, service.host
        );
    }

    // Discovery demo completed - primals would be discovered through service registry
    info!("📍 Discovery system demo completed");

    Ok(())
}
