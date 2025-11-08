use songbird_types::config::consolidated_canonical::CanonicalSongbirdConfig;
//! # Discovery Migration Demo
//!
//! This example demonstrates how to migrate from the old hardcoded discovery factory
//! to the new agnostic, configuration-driven approach.

use songbird_discovery::abstraction::{
    ModernizedDiscoveryFactory, DiscoveryConfigBuilder,
    capabilities::{CapabilityMatcher, CapabilityQuery, DiscoveryCapability},
    delegation::DelegationStrategy,
};
use songbird_errors::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔄 Discovery Migration Demo");
    println!("===========================");

    // === BEFORE: Hardcoded approach ===
    println!("\n❌ OLD HARDCODED APPROACH:");
    println!("```rust");
    println!("// Hard-coded string matching");
    println!("match backend {{");
    println!("    \"static\" => Box::new(StaticServiceDiscovery::new()),");
    println!("    \"consul\" => Box::new(ConsulServiceDiscovery::new(url)),");
    println!("    \"kubernetes\" => Box::new(KubernetesServiceDiscovery::new(ns)),");
    println!("    _ => return Err(\"Unsupported backend\"),");
    println!("}}");
    println!("```");

    // === AFTER: Agnostic approach ===
    println!("\n✅ NEW AGNOSTIC APPROACH:");

    // 1. Create the modernized factory
    println!("\n🏭 Creating modernized factory...");
    let factory = ModernizedDiscoveryFactory::new().await?;

    // 2. Configuration-driven setup
    println!("\n⚙️ Configuration-driven setup:");
    let configs = DiscoveryConfigBuilder::new()
        .add_static("dev-static".to_string(), vec![
            serde_json::json!({
                "service_id": "local-api",
                "name": "Local API",
                "version": "1.0.0",
                "service_type": "api",
                "description": "Local development API",
                "endpoints": [],
                "health_check_endpoint": "http://localhost:config.network.http_port/health",
                "metadata": {},
                "tags": ["api", "local"],
                "dependencies": [],
                "status": "Running",
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z",
                "instance_id": "local-api-1",
                "host": "localhost",
                "port": config.network.http_port
            })
        ])
        .add_consul("prod-consul".to_string(), "http://consul.prod:8500".to_string())
        .build();

    println!("📋 Created {} provider configurations", configs.len());
    for config in &configs {
        println!("  - {} ({})", config.name, config.id);
    }

    // 3. Validate configurations
    println!("\n🔍 Validating configurations...");
    factory.validate_configs(&configs).await?;
    println!("✅ All configurations valid!");

    // 4. Create discovery system
    println!("\n🎯 Creating discovery system with BestMatch strategy...");
    let delegator = factory.create_with_strategy(configs, DelegationStrategy::BestMatch).await?;

    // 5. Demonstrate capability-based routing
    println!("\n🎯 Capability-based routing:");

    // Find providers that can register services
    let registration_query = CapabilityQuery::new(
        CapabilityMatcher::new().require(DiscoveryCapability::ServiceRegistration)
    );

    println!("\n🔍 Providers that can register services:");
    if let Ok(providers) = delegator.registry.find_providers(&registration_query).await {
        for provider_id in providers {
            if let Ok(metadata) = delegator.registry.get_provider_metadata(&provider_id).await {
                println!("  ✅ {} ({}): load={}", metadata.name, metadata.id, metadata.load_score);
            }
        }
    }

    // Find providers that can watch for changes
    let watching_query = CapabilityQuery::new(
        CapabilityMatcher::new().require(DiscoveryCapability::ServiceWatching)
    );

    println!("\n👀 Providers that can watch for changes:");
    if let Ok(providers) = delegator.registry.find_providers(&watching_query).await {
        for provider_id in providers {
            if let Ok(metadata) = delegator.registry.get_provider_metadata(&provider_id).await {
                println!("  ✅ {} ({})", metadata.name, metadata.id);
            }
        }
    } else {
        println!("  ⚠️ No providers support watching (static doesn't support watching)");
    }

    // 6. Demonstrate environment-based configuration
    println!("\n🌍 Environment-based configuration:");
    println!("Setting environment variables...");
    std::env::set_var("SONGBIRD_DISCOVERY_STATIC", "true");
    let metrics_port = songbird_config::defaults::ports::metrics_port();
    std::env::set_var("SONGBIRD_STATIC_SERVICES", &format!(r#"[{{"service_id": "env-service", "name": "Environment Service", "host": "localhost", "port": {}}}]"#, metrics_port));

    let env_delegator = factory.create_from_environment().await?;
    println!("✅ Created discovery system from environment variables");

    let env_stats = env_delegator.registry.get_statistics().await;
    println!("📊 Environment-based registry: {} providers", env_stats.total_providers);

    // 7. Demonstrate file-based configuration
    println!("\n📄 File-based configuration:");
    
    // Create a sample config file
    let config_json = serde_json::json!([
        {
            "id": "file-static",
            "name": "File Static Provider",
            "parameters": {
                "type": "static",
                "services": []
            },
            "environment": {},
            "timeout_ms": 1000
        }
    ]);

    std::fs::write("discovery_config.json", serde_json::to_string_pretty(&config_json)?)?;
    println!("📝 Created discovery_config.json");

    let file_delegator = factory.create_from_file("discovery_config.json").await?;
    println!("✅ Created discovery system from JSON file");

    let file_stats = file_delegator.registry.get_statistics().await;
    println!("📊 File-based registry: {} providers", file_stats.total_providers);

    // Clean up
    let _ = std::fs::remove_file("discovery_config.json");

    // 8. Show migration benefits
    println!("\n🎉 MIGRATION BENEFITS ACHIEVED:");
    println!("  ✅ Zero hardcoding - all configuration-driven");
    println!("  ✅ Runtime flexibility - add providers without recompiling");
    println!("  ✅ Capability-based routing - automatic provider selection");
    println!("  ✅ Multiple strategies - BestMatch, LeastLoad, RoundRobin, Broadcast");
    println!("  ✅ Environment integration - works with existing env vars");
    println!("  ✅ File configuration - JSON/YAML support");
    println!("  ✅ Validation - catch config errors early");
    println!("  ✅ Vendor independence - no lock-in to specific services");

    println!("\n🚀 BEFORE vs AFTER:");
    println!("┌─────────────────────┬──────────────────────────────────┐");
    println!("│ Aspect              │ Before → After                   │");
    println!("├─────────────────────┼──────────────────────────────────┤");
    println!("│ Provider Selection  │ String match → Capability-based  │");
    println!("│ Configuration       │ Hardcoded → JSON/YAML/Env       │");
    println!("│ Adding New Provider │ Code change → Config change      │");
    println!("│ Load Balancing      │ Manual → Automatic               │");
    println!("│ Vendor Lock-in      │ Yes → No                         │");
    println!("│ Testing            │ Mock entire system → Register test providers │");
    println!("│ Flexibility        │ Compile-time → Runtime           │");
    println!("└─────────────────────┴──────────────────────────────────┘");

    println!("\n✨ Migration completed successfully!");
    println!("The discovery system is now completely agnostic and vendor-independent! 🎯");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_migration_example() {
        // Test that the migration example works
        let factory = ModernizedDiscoveryFactory::new().await.unwrap();
        
        let configs = DiscoveryConfigBuilder::new()
            .add_static("test".to_string(), vec![])
            .build();

        assert!(factory.validate_configs(&configs).await.is_ok());
        
        let delegator = factory.create_from_config(configs).await.unwrap();
        let stats = delegator.registry.get_statistics().await;
        
        assert_eq!(stats.total_providers, 1);
    }

    #[tokio::test]
    async fn test_capability_based_selection() {
        let factory = ModernizedDiscoveryFactory::new().await.unwrap();
        
        let configs = DiscoveryConfigBuilder::new()
            .add_static("static-1".to_string(), vec![])
            .build();

        let delegator = factory.create_from_config(configs).await.unwrap();
        
        // Test capability-based selection
        let query = CapabilityQuery::new(
            CapabilityMatcher::new().require(DiscoveryCapability::ServiceRegistration)
        );

        let providers = delegator.registry.find_providers(&query).await.unwrap();
        assert!(!providers.is_empty());
    }
} 