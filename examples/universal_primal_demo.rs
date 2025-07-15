//! Universal Primal Integration Demo
//!
//! This example demonstrates how to use the Songbird Universal Primal Integration
//! system to automatically discover and use BearDog for security operations.

use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

use songbird_universal_primals::{
    beardog::BearDogPrimal, nestgate::NestGatePrimal, toadstool::ToadstoolPrimal,
    traits::PrimalHealth, PrimalCapability, PrimalContext, PrimalProvider, PrimalResult,
    PrimalType, SecurityLevel, UniversalPrimalConfig, UniversalPrimalRegistry,
};

#[tokio::main]
async fn main() -> PrimalResult<()> {
    tracing_subscriber::fmt::init();

    info!("🚀 Starting Universal Primal Integration Demo");

    // Initialize the primal registry
    let registry = UniversalPrimalRegistry::new();

    // Create default context for the demo
    let context = PrimalContext::default();

    // Demo 1: Create and register individual primals
    info!("📋 Demo 1: Creating and registering individual primals");

    // Create BearDog security primal
    let beardog = Arc::new(create_beardog_primal("http://beardog.demo", context.clone()).await?);
    info!("🔐 Created BearDog primal: {}", beardog.primal_id());

    // Display capabilities
    info!("🔧 BearDog capabilities:");
    for capability in beardog.capabilities() {
        info!("  - {:?}", capability);
    }

    // Display endpoints
    let endpoints = beardog.endpoints();
    info!("🌐 BearDog endpoints: {:?}", endpoints);

    // Register the primal
    registry
        .register_primal_for_context(beardog.clone(), context.clone(), None)
        .await?;
    info!("✅ BearDog primal registered");

    // Create NestGate storage primal
    let nestgate = Arc::new(create_nestgate_primal("http://nestgate.demo", context.clone()).await?);
    info!("💾 Created NestGate primal: {}", nestgate.primal_id());

    // Register the primal
    registry
        .register_primal_for_context(nestgate.clone(), context.clone(), None)
        .await?;
    info!("✅ NestGate primal registered");

    // Create Toadstool compute primal
    let toadstool =
        Arc::new(create_toadstool_primal("http://toadstool.demo", context.clone()).await?);
    info!("🍄 Created Toadstool primal: {}", toadstool.primal_id());

    // Register the primal
    registry
        .register_primal_for_context(toadstool.clone(), context.clone(), None)
        .await?;
    info!("✅ Toadstool primal registered");

    // Create Squirrel AI primal
    let squirrel = Arc::new(create_squirrel_primal("http://squirrel.demo", context.clone()).await?);
    info!("🐿️ Created Squirrel primal: {}", squirrel.primal_id());

    // Register the primal
    registry
        .register_primal_for_context(squirrel.clone(), context.clone(), None)
        .await?;
    info!("✅ Squirrel primal registered");

    // Demo 2: Discovery and querying
    info!("🔍 Demo 2: Discovery and querying");

    // Get all security primals
    let security_primals = registry.get_instances_by_type(PrimalType::Security).await;
    info!("🔐 Security primals found: {}", security_primals.len());

    // Get all storage primals
    let storage_primals = registry.get_instances_by_type(PrimalType::Storage).await;
    info!("💾 Storage primals found: {}", storage_primals.len());

    // Get all compute primals
    let compute_primals = registry.get_instances_by_type(PrimalType::Compute).await;
    info!("🍄 Compute primals found: {}", compute_primals.len());

    // Get all AI primals
    let ai_primals = registry.get_instances_by_type(PrimalType::AI).await;
    info!("🐿️ AI primals found: {}", ai_primals.len());

    // Demo 3: Context-based discovery
    info!("🎯 Demo 3: Context-based discovery");

    let context_primals = registry.find_for_context(&context).await;
    info!("📍 Primals for context: {}", context_primals.len());

    // Demo 4: Capability-based discovery
    info!("🔧 Demo 4: Capability-based discovery");

    let encryption_capability = PrimalCapability::Encryption {
        algorithms: vec!["AES256".to_string()],
    };

    let encryption_primals = registry
        .find_by_capability_for_context(&encryption_capability, &context)
        .await;
    info!(
        "🔐 Encryption-capable primals: {}",
        encryption_primals.len()
    );

    // Demo 5: Health monitoring
    info!("💓 Demo 5: Health monitoring");

    let health_results = registry.health_check_all().await;
    info!("🏥 Health check results:");
    for (primal_id, health) in health_results {
        let status = match health {
            PrimalHealth::Healthy => "✅ Healthy".to_string(),
            PrimalHealth::Degraded { issues } => format!("⚠️ Degraded: {}", issues.join(", ")),
            PrimalHealth::Unhealthy { reason } => format!("❌ Unhealthy: {}", reason),
        };
        info!("  - {}: {}", primal_id, status);
    }

    // Demo 6: Statistics
    info!("📊 Demo 6: Statistics");

    let stats = registry.get_enhanced_statistics().await;
    info!("📈 Registry statistics:");
    info!("  - Total instances: {}", stats.total_instances);
    info!("  - Total users: {}", stats.total_users);
    info!("  - Instances by type:");
    for (primal_type, count) in stats.instances_by_type {
        info!("    - {:?}: {}", primal_type, count);
    }

    // Demo 7: Multi-user scenarios
    info!("👥 Demo 7: Multi-user scenarios");

    // Create a different context for user2
    let user2_context = PrimalContext {
        user_id: "user2".to_string(),
        device_id: "device2".to_string(),
        session_id: "session2".to_string(),
        network_location: songbird_universal_primals::traits::NetworkLocation {
            ip_address: "192.168.1.100".to_string(),
            subnet: Some("192.168.1.0/24".to_string()),
            network_id: Some("home_network".to_string()),
            geo_location: Some("US-West".to_string()),
        },
        security_level: SecurityLevel::Standard,
        metadata: HashMap::new(),
    };

    // Create another BearDog instance for user2
    let beardog2 =
        Arc::new(create_beardog_primal("http://beardog2.demo", user2_context.clone()).await?);
    registry
        .register_primal_for_context(beardog2.clone(), user2_context.clone(), None)
        .await?;
    info!("🔐 Created and registered BearDog for user2");

    // Check instances for different users
    let user1_primals = registry.get_instances_for_user(&context.user_id).await;
    let user2_primals = registry.get_instances_for_user("user2").await;

    info!("👤 User1 primals: {}", user1_primals.len());
    info!("👤 User2 primals: {}", user2_primals.len());

    // Demo 8: Configuration management
    info!("⚙️ Demo 8: Configuration management");

    // Create configuration from environment
    let config = UniversalPrimalConfig::from_env();
    info!("🔧 Configuration loaded from environment");
    info!(
        "  - Auto-discovery enabled: {}",
        config.auto_discovery_enabled
    );
    info!(
        "  - Primal instances configured: {}",
        config.primal_instances.len()
    );

    // Demo 9: Cleanup
    info!("🧹 Demo 9: Cleanup");

    // Unregister primals
    registry.unregister_instance(&beardog.instance_id()).await?;
    info!("🗑️ Unregistered BearDog primal");

    registry
        .unregister_instance(&nestgate.instance_id())
        .await?;
    info!("🗑️ Unregistered NestGate primal");

    registry
        .unregister_instance(&toadstool.instance_id())
        .await?;
    info!("🗑️ Unregistered Toadstool primal");

    registry
        .unregister_instance(&squirrel.instance_id())
        .await?;
    info!("🗑️ Unregistered Squirrel primal");

    registry
        .unregister_instance(&beardog2.instance_id())
        .await?;
    info!("🗑️ Unregistered BearDog2 primal");

    // Verify cleanup
    let final_stats = registry.get_enhanced_statistics().await;
    info!("📊 Final statistics:");
    info!("  - Total instances: {}", final_stats.total_instances);

    info!("🎉 Universal Primal Integration Demo completed successfully!");

    Ok(())
}
