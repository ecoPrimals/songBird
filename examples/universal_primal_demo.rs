//! Universal Primal Integration Demo
//!
//! This example demonstrates how to use the Songbird Universal Primal Integration
//! system to automatically discover and use BearDog for security operations.

use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

use songbird_universal_primals::{
    beardog::BearDogPrimal, nestgate::NestGatePrimalClient, toadstool::ToadstoolPrimal,
    traits::PrimalHealth, PrimalCapability, PrimalContext, PrimalProvider, PrimalResult,
    PrimalType, SecurityLevel, UniversalPrimalConfig, UniversalPrimalRegistry,
    NetworkLocation,
};

// Helper function to create BearDog primal
async fn create_beardog_primal(endpoint: &str, context: PrimalContext) -> PrimalResult<BearDogPrimal> {
    info!("Creating BearDog primal for endpoint: {}", endpoint);
    Ok(BearDogPrimal::with_context(context))
}

// Helper function to create NestGate primal
async fn create_nestgate_primal(endpoint: &str, context: PrimalContext) -> PrimalResult<NestGatePrimalClient> {
    info!("Creating NestGate primal for endpoint: {}", endpoint);
    Ok(NestGatePrimalClient::new())
}

// Helper function to create Toadstool primal
async fn create_toadstool_primal(endpoint: &str, context: PrimalContext) -> PrimalResult<ToadstoolPrimal> {
    info!("Creating Toadstool primal for endpoint: {}", endpoint);
    Ok(ToadstoolPrimal::with_context(context))
}

// Helper function to create Squirrel primal (if available)
async fn create_squirrel_primal(endpoint: &str, context: PrimalContext) -> PrimalResult<BearDogPrimal> {
    info!("Creating Squirrel primal for endpoint: {}", endpoint);
    // For now, using BearDog as a placeholder
    Ok(BearDogPrimal::with_context(context))
}

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
    let beardog: Arc<BearDogPrimal> = Arc::new(create_beardog_primal("http://beardog.demo", context.clone()).await?);
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

    // Create NestGate data primal
    let nestgate: Arc<NestGatePrimalClient> = Arc::new(create_nestgate_primal("http://nestgate.demo", context.clone()).await?);
    info!("💾 Created NestGate primal: {}", nestgate.primal_id());

    // Register the primal
    registry
        .register_primal_for_context(nestgate.clone(), context.clone(), None)
        .await?;
    info!("✅ NestGate primal registered");

    // Create Toadstool compute primal
    let toadstool: Arc<ToadstoolPrimal> =
        Arc::new(create_toadstool_primal("http://toadstool.demo", context.clone()).await?);
    info!("🍄 Created Toadstool primal: {}", toadstool.primal_id());

    // Register the primal
    registry
        .register_primal_for_context(toadstool.clone(), context.clone(), None)
        .await?;
    info!("✅ Toadstool primal registered");

    // Create Squirrel optimization primal
    let squirrel: Arc<BearDogPrimal> = Arc::new(create_squirrel_primal("http://squirrel.demo", context.clone()).await?);
    info!("🐿️ Created Squirrel primal: {}", squirrel.primal_id());

    // Register the primal
    registry
        .register_primal_for_context(squirrel.clone(), context.clone(), None)
        .await?;
    info!("✅ Squirrel primal registered");

    // Demo 2: Health checks and monitoring
    info!("📋 Demo 2: Health checks and monitoring");

    // Check health of all primals
    let beardog_health = beardog.health_check().await;
    info!("🔐 BearDog health: {:?}", beardog_health);

    let nestgate_health = nestgate.health_check().await;
    info!("💾 NestGate health: {:?}", nestgate_health);

    let toadstool_health = toadstool.health_check().await;
    info!("🍄 Toadstool health: {:?}", toadstool_health);

    // Demo 3: Cross-primal operations
    info!("📋 Demo 3: Cross-primal operations");

    // Display primal information
    info!("🔐 BearDog primal ID: {}", beardog.primal_id());
    info!("💾 NestGate primal ID: {}", nestgate.primal_id());
    info!("🍄 Toadstool primal ID: {}", toadstool.primal_id());

    // Demo 4: Dynamic registration and discovery
    info!("📋 Demo 4: Dynamic registration and discovery");

    // Create a new context for a different user
    let user2_context = PrimalContext {
        user_id: "user2".to_string(),
        device_id: "device_b".to_string(),
        session_id: "session_456".to_string(),
        network_location: NetworkLocation {
            ip_address: "192.168.1.100".to_string(),
            subnet: Some("192.168.1.0/24".to_string()),
            network_id: Some("remote_network".to_string()),
            geo_location: Some("Remote Location".to_string()),
        },
        security_level: SecurityLevel::High,
        metadata: HashMap::new(),
    };

    // Create and register a primal for the new context
    let beardog2: Arc<BearDogPrimal> =
        Arc::new(create_beardog_primal("http://beardog2.demo", user2_context.clone()).await?);

    registry
        .register_primal_for_context(beardog2.clone(), user2_context.clone(), None)
        .await?;
    info!("✅ Registered BearDog primal for user2");

    // Display contexts
    info!("👤 User1 context: {:?}", context);
    info!("👤 User2 context: {:?}", user2_context);

    // Demo 5: Primal capability matching
    info!("📋 Demo 5: Primal capability matching");

    // Display capabilities for each primal
    info!("🔐 BearDog capabilities:");
    for capability in beardog.capabilities() {
        info!("  - {:?}", capability);
    }

    info!("💾 NestGate capabilities:");
    for capability in nestgate.capabilities() {
        info!("  - {:?}", capability);
    }

    info!("🍄 Toadstool capabilities:");
    for capability in toadstool.capabilities() {
        info!("  - {:?}", capability);
    }

    // Demo 6: Configuration and optimization
    info!("📋 Demo 6: Configuration and optimization");

    // Display primal contexts
    info!("🔐 BearDog context: {:?}", beardog.context());
    info!("💾 NestGate context: {:?}", nestgate.context());
    info!("🍄 Toadstool context: {:?}", toadstool.context());

    // Demo 7: Cleanup and shutdown
    info!("📋 Demo 7: Cleanup and shutdown");

    // Unregister primals
    registry.unregister_primal(&beardog.primal_id()).await?;
    info!("🗑️ Unregistered BearDog primal");

    registry.unregister_primal(&nestgate.primal_id()).await?;
    info!("🗑️ Unregistered NestGate primal");

    registry.unregister_primal(&toadstool.primal_id()).await?;
    info!("🗑️ Unregistered Toadstool primal");

    registry.unregister_primal(&squirrel.primal_id()).await?;
    info!("🗑️ Unregistered Squirrel primal");

    registry.unregister_primal(&beardog2.primal_id()).await?;
    info!("🗑️ Unregistered BearDog2 primal");

    info!("🎉 Universal Primal Integration Demo completed successfully!");

    Ok(())
}
