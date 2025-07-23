//! Ecosystem Standalone Operation Demo
//!
//! This demo shows how Songbird operates standalone while discovering and
//! leveraging other primals (toadstool, nestgate, squirrel) for network effects.

use songbird_universal_primals::{
    discovery::PrimalDiscoveryEngine,
    registry::UniversalPrimalRegistry,
    traits::{NetworkLocation, PrimalCapability, PrimalContext, SecurityLevel},
};
use std::collections::HashMap;
use tracing::{info, warn};
use tracing_subscriber::fmt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    fmt::init();

    info!("🎼 Starting Songbird in Standalone + Ecosystem mode");

    // 1. Initialize Songbird as standalone orchestrator
    let songbird_registry = UniversalPrimalRegistry::new();

    // 2. Create discovery engine for ecosystem integration
    let mut discovery_engine = PrimalDiscoveryEngine::new(
        songbird_config::config::hardcoded_elimination::PrimalConfig::default(),
    );

    info!("🔍 Discovering ecosystem primals...");

    // 3. Auto-discover available primals (toadstool, nestgate, squirrel, etc.)
    match discovery_engine.start_discovery().await {
        Ok(_) => {
            let discovered = discovery_engine.get_discovered_primals();
            info!("✅ Discovered {} ecosystem primals", discovered.len());

            for primal in discovered {
                info!(
                    "  - {} [{}]: {} capabilities",
                    primal.primal_type.as_str(),
                    primal.endpoint,
                    primal.capabilities.len()
                );
            }
        }
        Err(e) => {
            warn!("⚠️ Discovery failed: {}", e);
            info!("📱 Operating in standalone mode only");
        }
    }

    // 4. Demonstrate capability-based routing
    info!("🧠 Testing capability-based routing...");

    let user_context = PrimalContext {
        user_id: "demo-user".to_string(),
        device_id: "demo-device".to_string(),
        session_id: uuid::Uuid::new_v4().to_string(),
        network_location: NetworkLocation {
            ip_address: "192.168.1.100".to_string(),
            subnet: Some("192.168.1.0/24".to_string()),
            network_id: Some("home-network".to_string()),
            geo_location: Some("local".to_string()),
        },
        security_level: SecurityLevel::Standard,
        metadata: HashMap::new(),
    };

    // 5. Route different workload types to appropriate primals
    route_workload_example(&songbird_registry, &user_context, "compute").await;
    route_workload_example(&songbird_registry, &user_context, "storage").await;
    route_workload_example(&songbird_registry, &user_context, "ai").await;
    route_workload_example(&songbird_registry, &user_context, "security").await;

    info!("🌟 Standalone + Ecosystem operation demonstrated successfully!");

    Ok(())
}

async fn route_workload_example(
    registry: &UniversalPrimalRegistry,
    context: &PrimalContext,
    workload_type: &str,
) {
    info!("🔀 Routing {} workload...", workload_type);

    // Find primals that can handle this workload type
    let available_primals = match workload_type {
        "compute" => {
            let capability = PrimalCapability::ContainerRuntime {
                orchestrators: vec!["docker".to_string()],
            };
            registry
                .find_by_capability_for_context(&capability, context)
                .await
        }
        "storage" => {
            let capability = PrimalCapability::FileSystem {
                supports_zfs: false,
            };
            registry
                .find_by_capability_for_context(&capability, context)
                .await
        }
        "ai" => {
            let capability = PrimalCapability::ModelInference {
                models: vec!["gpt".to_string()],
            };
            registry
                .find_by_capability_for_context(&capability, context)
                .await
        }
        "security" => {
            let capability = PrimalCapability::Encryption {
                algorithms: vec!["aes256".to_string()],
            };
            registry
                .find_by_capability_for_context(&capability, context)
                .await
        }
        _ => Vec::new(),
    };

    if available_primals.is_empty() {
        info!("  📱 No ecosystem primal found, handling locally in Songbird");
        // Songbird handles the workload itself (standalone mode)
    } else {
        info!(
            "  🌐 Found {} capable primals, routing to ecosystem",
            available_primals.len()
        );
        // Route to the most appropriate primal (network effects)
    }
}
