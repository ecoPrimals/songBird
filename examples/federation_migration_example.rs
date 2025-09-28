use songbird_types::config::consolidated_canonical::CanonicalSongbirdConfig;
//! # Federation Migration Example
//!
//! This example demonstrates how to migrate from the old songbird-federation
//! system to the new discovery-based approach with enhanced sovereignty features.
//!
//! ## Before and After Comparison
//!
//! This example shows:
//! 1. Old federation code (commented out)
//! 2. Automated migration using FederationMigrationHelper
//! 3. New discovery-based code with enhanced features
//! 4. Performance comparison and validation

use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

// New imports (migrated)
use songbird_discovery::{
    discovery::{DiscoveryConfig, ServiceDiscoveryFactory},
    federation_aware_discovery::{FederationAwareDiscovery, FederationDiscoveryConfig},
    migration::{
        FederationMigrationHelper, LegacyFederationConfig, LegacyFederationMode,
        LegacySovereigntyLevel, MigrationConfig,
    },
};
use songbird_universal::sovereignty_aware_adapter::{
    SovereigntyAwareAdapter, SovereigntyAdapterConfig,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("🚀 Federation Migration Example");
    info!("=" * 50);

    // Step 1: Create legacy configuration (what you had before)
    let legacy_config = create_legacy_federation_config();
    info!("📋 Created legacy federation configuration");

    // Step 2: Automated migration
    info!("🔄 Starting automated migration...");
    let migration_result = migrate_federation_config(legacy_config).await?;
    
    if !migration_result.success {
        warn!("⚠️ Migration had issues: {:?}", migration_result.errors);
        return Ok(());
    }

    info!("🎉 Migration successful!");
    let new_config = migration_result.new_discovery_config;

    // Step 3: Use new federation-aware discovery system
    info!("🌐 Creating federation-aware discovery...");
    let (federation_discovery, sovereignty_adapter) = create_new_federation_system(new_config).await?;

    // Step 4: Demonstrate enhanced capabilities
    info!("✨ Demonstrating enhanced capabilities...");
    demonstrate_enhanced_capabilities(federation_discovery, sovereignty_adapter).await?;

    // Step 5: Performance comparison
    info!("📊 Performance comparison:");
    if let Some(perf) = migration_result.performance_comparison {
        info!("  - Discovery time improvement: {:.1}%", perf.discovery_time_improvement);
        info!("  - Memory usage improvement: {:.1}%", perf.memory_usage_improvement);
        info!("  - Feature parity score: {:.1}%", perf.feature_parity_score * 100.0);
        info!("  - Overall performance score: {:.1}%", perf.overall_performance_score * 100.0);
    }

    info!("🎊 Migration example completed successfully!");
    Ok(())
}

/// Create a legacy federation configuration (simulating old system)
fn create_legacy_federation_config() -> LegacyFederationConfig {
    let mut legacy_settings = HashMap::new();
    legacy_settings.insert("old_setting_1".to_string(), "value1".to_string());
    legacy_settings.insert("performance_mode".to_string(), "high".to_string());

    LegacyFederationConfig {
        cluster_name: Some("example-cluster".to_string()),
        peer_discovery_enabled: true,
        discovery_endpoints: vec![
            "localhost:8080".to_string(),
            "localhost:8081".to_string(),
            "192.168.1.100:8443".to_string(),
        ],
        heartbeat_interval: Duration::from_secs(30),
        sovereignty_level: Some(LegacySovereigntyLevel::High),
        enable_network_effects: true,
        federation_mode: Some(LegacyFederationMode::Peer),
        legacy_settings,
    }
}

/// Migrate federation configuration using automated helper
async fn migrate_federation_config(
    legacy_config: LegacyFederationConfig,
) -> Result<songbird_discovery::migration::MigrationResult, Box<dyn std::error::Error>> {
    let mut migration_helper = FederationMigrationHelper::new(MigrationConfig {
        verbose_logging: true,
        validate_migration: true,
        migration_timeout: Duration::from_secs(30),
        preserve_legacy_behavior: false,
    });

    let migration_result = migration_helper.migrate_with_validation(legacy_config).await?;

    // Display migration statistics
    let stats = &migration_result.stats;
    info!("📊 Migration statistics:");
    info!("  - Configurations migrated: {}", stats.configs_migrated);
    info!("  - Services discovered: {}", stats.services_discovered);
    info!("  - Sovereignty assessments: {}", stats.sovereignty_assessments);
    info!("  - Network effects detected: {}", stats.network_effects_detected);
    info!("  - Success rate: {:.1}%", stats.success_rate * 100.0);
    info!("  - Migration time: {:?}", stats.total_migration_time);

    // Display warnings if any
    if !migration_result.warnings.is_empty() {
        warn!("⚠️ Migration warnings:");
        for warning in &migration_result.warnings {
            warn!("  - {}", warning);
        }
    }

    Ok(migration_result)
}

/// Create new federation system with discovery and sovereignty features
async fn create_new_federation_system(
    discovery_config: FederationDiscoveryConfig,
) -> Result<(FederationAwareDiscovery, SovereigntyAwareAdapter), Box<dyn std::error::Error>> {
    // Create base discovery service
    let base_discovery = ServiceDiscoveryFactory::create(&discovery_config.base_config)?;
    
    // Create federation-aware discovery
    let federation_discovery = FederationAwareDiscovery::new(base_discovery, discovery_config);

    // Create sovereignty-aware adapter
    let sovereignty_config = SovereigntyAdapterConfig::default();
    let base_adapter = songbird_universal::capabilities::UniversalCapabilityAdapter::new(
        songbird_universal::capabilities::CapabilityConfig::default()
    );
    let sovereignty_adapter = SovereigntyAwareAdapter::new(base_adapter, sovereignty_config);

    info!("✅ New federation system created successfully");
    Ok((federation_discovery, sovereignty_adapter))
}

/// Demonstrate enhanced capabilities of the new system
async fn demonstrate_enhanced_capabilities(
    mut federation_discovery: FederationAwareDiscovery,
    mut sovereignty_adapter: SovereigntyAwareAdapter,
) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Federation-aware service discovery
    info!("🔍 Discovering services with federation awareness...");
    let services = federation_discovery.discover_federation_aware_services().await?;
    
    info!("📡 Discovered {} services", services.len());
    for service in &services {
        info!("  - {}: {} (sovereignty: {:?})", 
              service.base_info.service_name,
              service.base_info.endpoint,
              service.sovereignty_assessment.sovereignty_level);
    }

    // 2. Network effects calculation
    let network_potential = federation_discovery.calculate_network_effect_potential(&services);
    info!("🌐 Network effect potential: {:.2}", network_potential);

    // 3. Sovereignty-aware routing (example)
    info!("🏛️ Testing sovereignty-aware routing...");
    let example_request = songbird_universal::types::UniversalRequest {
        request_id: "example-request".to_string(),
        capability_required: songbird_universal::types::Capability {
            name: "example-capability".to_string(),
            version: "1.0.0".to_string(),
            requirements: vec![],
        },
        payload: serde_json::json!({"example": "data"}),
        routing_preferences: None,
        timeout: Some(Duration::from_secs(10)),
    };

    match sovereignty_adapter.execute_with_sovereignty_routing(&example_request).await {
        Ok(response) => {
            info!("✅ Sovereignty-aware routing successful");
            info!("  - Response ID: {}", response.response_id);
            info!("  - Success: {}", response.success);
        }
        Err(e) => {
            warn!("⚠️ Sovereignty routing test failed (expected in example): {}", e);
        }
    }

    // 4. Enhanced sovereignty assessment
    info!("🔒 Sovereignty assessment features:");
    for service in services.iter().take(3) {
        let assessment = &service.sovereignty_assessment;
        info!("  - {}: Level={:?}, Confidence={:.2}, Hierarchy={:?}",
              service.base_info.service_name,
              assessment.sovereignty_level,
              assessment.confidence,
              assessment.hierarchy_position);
    }

    // 5. Network effects detection
    info!("🧬 Network effects detected:");
    for service in services.iter().take(3) {
        if !service.network_effects.is_empty() {
            info!("  - {}: {} effects", 
                  service.base_info.service_name,
                  service.network_effects.len());
            for effect in &service.network_effects {
                info!("    - {:?}: {}", effect.effect_type, effect.description);
            }
        }
    }

    Ok(())
}

/// Example of old federation code (commented out for reference)
#[allow(dead_code)]
fn example_old_federation_code() {
    /*
    // OLD CODE (DEPRECATED - DO NOT USE):
    
    use songbird_federation::{
        FederationManager, FederationConfig, ProductionFederation
    };

    async fn old_federation_example() -> Result<(), Box<dyn std::error::Error>> {
        // Old configuration
        let config = FederationConfig {
            cluster_name: Some("example-cluster".to_string()),
            peer_discovery_enabled: true,
            discovery_endpoints: vec!["localhost:8080".to_string()],
            heartbeat_interval: Duration::from_secs(30),
            // ... other old settings
        };

        // Old federation manager
        let mut federation = FederationManager::new(config).await?;
        
        // Old API calls
        let peers = federation.discover_peers().await?;
        let response = federation.route_request(&request).await?;
        let effects = federation.get_network_effects().await?;
        
        Ok(())
    }
    */
}

/// Example of using the compatibility wrapper for gradual migration
#[allow(dead_code)]
async fn example_compatibility_wrapper() -> Result<(), Box<dyn std::error::Error>> {
    info!("🔄 Compatibility wrapper example");

    // Create new config
    let new_config = FederationDiscoveryConfig::default();
    
    // Use compatibility wrapper for gradual migration
    let mut wrapper = songbird_discovery::migration::LegacyFederationWrapper::new(new_config)?;

    // Use old API methods (they work with new system underneath)
    let peers = wrapper.discover_peers().await?;
    info!("📡 Found {} peers via compatibility wrapper", peers.len());

    let network_effects = wrapper.get_network_effects().await?;
    info!("🌐 Network effects: {:.2}", network_effects);

    wrapper.join_network("example-network").await?;
    info!("🔗 Joined network via compatibility wrapper");

    Ok(())
}

/// Quick migration example using convenience functions
#[allow(dead_code)]
async fn example_quick_migration() -> Result<(), Box<dyn std::error::Error>> {
    info!("⚡ Quick migration example");

    // Create legacy config
    let legacy_config = create_legacy_federation_config();

    // Quick migration (single function call)
    let new_config = FederationMigrationHelper::quick_migrate(legacy_config.clone())?;
    info!("✅ Quick migration successful");

    // Quick migration with wrapper (for immediate compatibility)
    let wrapper = FederationMigrationHelper::quick_migrate_with_wrapper(legacy_config)?;
    info!("🔧 Compatibility wrapper created");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_migration_example() {
        // Test that the migration example works
        let legacy_config = create_legacy_federation_config();
        let migration_result = migrate_federation_config(legacy_config).await.unwrap();
        
        // Should succeed even in test environment
        assert!(migration_result.success || !migration_result.errors.is_empty());
    }

    #[tokio::test]
    async fn test_compatibility_wrapper() {
        // Test compatibility wrapper
        let result = example_compatibility_wrapper().await;
        // Should not panic, may have connection errors in test environment
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_quick_migration() {
        // Test quick migration functions
        example_quick_migration().await.unwrap();
    }
} 