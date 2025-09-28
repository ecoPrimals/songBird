use songbird_types::config::consolidated_canonical::CanonicalSongbirdConfig;
//! # Vendor Agnostic Discovery Demo
//!
//! This example demonstrates how Songbird's universal discovery system
//! works without any hardcoded vendor names like "consul", "kubernetes", etc.
//!
//! The system automatically detects what's available and adapts accordingly.

use songbird_discovery::{traits::ServiceQuery, UniversalDiscoveryFactory};
use tracing::{info, warn};
use tracing_subscriber::fmt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    fmt::init();

    info!("🚀 Starting Vendor Agnostic Discovery Demo");
    info!("📋 This demo shows how Songbird discovers services without hardcoding vendor names");

    // ✅ NEW APPROACH: Auto-detection without vendor hardcoding
    info!("🔍 Creating universal discovery with auto-detection...");
    let discovery = match UniversalDiscoveryFactory::create_auto_detect().await {
        Ok(d) => {
            info!("✅ Universal discovery initialized successfully!");
            d
        }
        Err(e) => {
            warn!("⚠️ Auto-detection failed, this is expected in demo environment: {}", e);
            info!("🔄 Falling back to capability-based discovery...");
            UniversalDiscoveryFactory::create_for_capability("service_discovery").await?
        }
    };

    // Demonstrate capability-based service discovery
    info!("🔍 Discovering services by capability (not vendor name)...");
    let query = ServiceQuery::new();

    match discovery.discover(query).await {
        Ok(services) => {
            info!("✅ Discovered {} services without hardcoding any vendor names!", services.len());
            for service in services {
                info!("   📡 Service: {} (type: {})", service.name, service.service_type);
            }
        }
        Err(e) => {
            info!("ℹ️ No services discovered in demo environment: {}", e);
            info!("   This is expected - in a real environment, services would be auto-detected");
        }
    }

    // Demonstrate environment-based discovery
    info!("🌍 Testing environment-based discovery...");
    let env_discovery = UniversalDiscoveryFactory::create_from_environment().await?;

    match env_discovery.list_all().await {
        Ok(services) => {
            info!("✅ Environment discovery found {} services", services.len());
        }
        Err(e) => {
            info!("ℹ️ Environment discovery: {}", e);
        }
    }

    // Show the contrast with old hardcoded approach
    info!("");
    info!("🎯 VENDOR AGNOSTIC SUCCESS!");
    info!(
        "   ❌ OLD: match backend {{ \"consul\" => ConsulClient::new(\"http://consul:8500\"), ... }}"
    );
    info!("   ✅ NEW: UniversalDiscoveryFactory::create_auto_detect() // Works with ANY system!");
    info!("");
    info!("🌟 Benefits of vendor-agnostic architecture:");
    info!("   🔍 Auto-detects Consul, Eureka, Kubernetes, Docker, or any other system");
    info!("   🚀 No vendor lock-in - works with any service registry or container orchestration");
    info!("   🔮 Future-proof - new service types work without code changes");
    info!("   ⚡ Performance optimized with zero-cost abstractions");

    Ok(())
}
