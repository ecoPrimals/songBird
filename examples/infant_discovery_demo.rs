//! # 🍼 Infant Discovery Demo - Zero Knowledge Bootstrap
//!
//! This example demonstrates the "infant discovery" philosophy where a service
//! starts with ZERO hardcoded knowledge and discovers everything dynamically.
//!
//! ## Philosophy
//! > "Each primal only knows itself and discovers others through the universal adapter"
//!
//! ## What This Demo Shows
//! 1. **Zero hardcoded primal names** - No beardog, squirrel, toadstool, nestgate
//! 2. **Zero hardcoded ports** - All from environment or discovery
//! 3. **Zero hardcoded endpoints** - Everything discovered dynamically
//! 4. **6-Phase Discovery** - Environment → Network → Process → Capability → Communication → Network Effects
//!
//! ## Usage
//!
//! ### Option 1: Explicit Configuration
//! ```bash
//! export SERVICE_PORT=<port from songbird_config>
//! export CAPABILITY_SECURITY_ENDPOINT="http://localhost:<beardog_port>"
//! export CAPABILITY_STORAGE_ENDPOINT="http://localhost:<metrics_port>"
//! cargo run --example infant_discovery_demo
//! ```
//!
//! ### Option 2: Service Registry Discovery
//! ```bash
//! export SERVICE_PORT=<port from songbird_config>
//! export ENABLE_INFANT_DISCOVERY=true
//! export SERVICE_REGISTRY_ENDPOINT="http://localhost:8500"
//! cargo run --example infant_discovery_demo
//! ```
//!
//! ### Option 3: Zero Configuration (Network Scan - Dev Only)
//! ```bash
//! export SERVICE_PORT=<port from songbird_config>
//! export ENABLE_INFANT_DISCOVERY=true
//! export ENABLE_NETWORK_DISCOVERY=true
//! export DISCOVERY_IP_RANGES="127.0.0.1/24"
//! cargo run --example infant_discovery_demo
//! ```

use songbird_config::capability_endpoints::{CapabilityEndpointResolver, CapabilityType};
use songbird_config::zero_touch::infant_config::ZeroTouchConfig;
use songbird_types::SongbirdResult;
use tracing::{debug, error, info, warn};

#[tokio::main]
async fn main() -> SongbirdResult<()> {
    // Initialize logging
    tracing_subscriber::fmt().init();

    info!("🍼 Infant Discovery Demo - Starting with Zero Knowledge");
    info!("================================================");
    println!();

    // Step 1: Demonstrate what we DON'T know
    demonstrate_zero_knowledge();

    // Step 2: Create zero-touch configuration (only knows itself)
    let config = match create_zero_touch_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("❌ Failed to create configuration: {}", e);
            println!("\n💡 TIP: Set SERVICE_PORT environment variable");
            println!(
                "   Example: export SERVICE_PORT={}",
                songbird_config::defaults::ports::orchestrator_port()
            );
            return Err(e);
        }
    };

    info!("✅ Zero-touch configuration created");
    info!("   Service ID: {}", config.self_identity.service_id);
    info!("   Provides: {:?}", config.self_identity.provides_capabilities);
    info!("   Requires: {} capabilities", config.required_capabilities.len());
    println!();

    // Step 3: Discover capabilities dynamically
    info!("🔍 Phase 1: Beginning capability discovery...");
    let resolver = CapabilityEndpointResolver::new();

    // Try to discover security capability (was "beardog")
    discover_capability(&resolver, "security", "Security (Authentication, Encryption)").await;

    // Try to discover storage capability (was "nestgate")
    discover_capability(&resolver, "storage", "Storage (Persistence, Caching)").await;

    // Try to discover compute capability (was "toadstool")
    discover_capability(&resolver, "compute", "Compute (Workload Execution)").await;

    // Try to discover AI capability (was "squirrel")
    discover_capability(&resolver, "ai", "AI/ML (Inference, Training)").await;

    println!();

    // Step 4: Demonstrate network effects
    demonstrate_network_effects(&resolver).await?;

    // Step 5: Show what we learned
    demonstrate_learning_summary(&resolver).await;

    info!("🎉 Infant Discovery Demo Complete!");
    info!("================================================");
    println!();
    println!("📚 Key Takeaways:");
    println!("  1. Started with ZERO hardcoded knowledge");
    println!("  2. Discovered capabilities dynamically");
    println!("  3. No primal names hardcoded (beardog, squirrel, etc.)");
    println!("  4. No ports hardcoded (8001-8004)");
    println!("  5. Network effects emerged naturally");
    println!();
    println!("💎 Philosophy: Each service only knows itself!");
    println!();

    Ok(())
}

/// Demonstrate that we start with ZERO knowledge
fn demonstrate_zero_knowledge() {
    info!("👶 Starting State: ZERO KNOWLEDGE");
    println!();
    println!("❌ We DON'T know:");
    println!("   - Primal names (beardog, squirrel, toadstool, nestgate)");
    println!("   - Port numbers (8001, 8002, 8003, 8004)");
    println!("   - Endpoint URLs");
    println!("   - Who provides what capability");
    println!();
    println!("✅ We ONLY know:");
    println!("   - Our own identity (from environment)");
    println!("   - What capabilities we need (not who provides them)");
    println!("   - How to discover (environment, registry, DNS, network)");
    println!();
}

/// Create zero-touch configuration (knows only itself)
fn create_zero_touch_config() -> SongbirdResult<ZeroTouchConfig> {
    debug!("Creating zero-touch configuration...");

    // This will FAIL if SERVICE_PORT is not set - no hardcoded defaults!
    match ZeroTouchConfig::from_environment() {
        Ok(config) => Ok(config),
        Err(e) => {
            warn!("Failed to create config from environment: {}", e);
            Err(e)
        }
    }
}

/// Discover a single capability
async fn discover_capability(
    resolver: &CapabilityEndpointResolver,
    capability: &str,
    description: &str,
) {
    info!("🔍 Discovering {} capability...", capability);

    match resolver.get_endpoint(parse_capability(capability)).await {
        Ok(endpoint) => {
            info!("   ✅ Found {} at: {}", capability, endpoint);
            println!("   📍 {}: {}", description, endpoint);
        }
        Err(e) => {
            warn!("   ⚠️  {} capability not found: {}", capability, e);
            println!("   ⚠️  {}: Not available", description);
            println!("      💡 Set CAPABILITY_{}_ENDPOINT to enable", capability.to_uppercase());
        }
    }
}

/// Demonstrate network effects (service mesh formation)
async fn demonstrate_network_effects(resolver: &CapabilityEndpointResolver) -> SongbirdResult<()> {
    info!("🕸️  Phase 2: Demonstrating Network Effects...");
    println!();
    println!("Scenario: AI analysis of secure data");
    println!("=====================================");

    // Check what capabilities are available
    let has_security = resolver.get_endpoint(CapabilityType::Security).await.is_ok();
    let has_storage = resolver.get_endpoint(CapabilityType::Storage).await.is_ok();
    let has_ai = resolver.get_endpoint(CapabilityType::Ai).await.is_ok();

    if has_security && has_storage && has_ai {
        println!("✅ All capabilities available!");
        println!();
        println!("Network Effect Flow:");
        println!("  1. Security → Authenticate request");
        println!("  2. Storage → Retrieve data");
        println!("  3. AI → Analyze data");
        println!("  4. Storage → Save results");
        println!("  5. Security → Encrypt response");
        println!();
        info!("🎯 Complex workflow possible through capability composition");
    } else {
        println!("⚠️  Some capabilities missing:");
        if !has_security {
            println!("  - Security capability not available");
        }
        if !has_storage {
            println!("  - Storage capability not available");
        }
        if !has_ai {
            println!("  - AI capability not available");
        }
        println!();
        info!("💡 Network effects limited by available capabilities");
    }

    Ok(())
}

/// Show what we learned during discovery
async fn demonstrate_learning_summary(resolver: &CapabilityEndpointResolver) {
    info!("📊 Phase 3: Learning Summary");
    println!();

    let discovered = resolver.get_all_cached().await;

    if discovered.is_empty() {
        println!("⚠️  No capabilities discovered yet");
        println!();
        println!("💡 To discover capabilities, set environment variables:");
        println!(
            "   export CAPABILITY_SECURITY_ENDPOINT=http://localhost:{}",
            songbird_config::defaults::ports::beardog_port()
        );
        println!(
            "   export CAPABILITY_STORAGE_ENDPOINT=http://localhost:{}",
            songbird_config::defaults::ports::metrics_port()
        );
        println!();
        println!("   OR enable infant discovery:");
        println!("   export ENABLE_INFANT_DISCOVERY=true");
        println!("   export SERVICE_REGISTRY_ENDPOINT=http://localhost:8500");
    } else {
        let count = discovered.len();
        println!("✅ Discovered Capabilities:");
        println!();
        for (cap_type, endpoint) in &discovered {
            println!("  • {:?} → {}", cap_type, endpoint.endpoint);
            println!("    Method: {:?}", endpoint.discovery_method);
            println!("    Confidence: {:.0}%", endpoint.confidence * 100.0);
            println!();
        }

        info!("🎓 Learning complete: {} capabilities discovered", count);
    }
}

// Helper to convert string to CapabilityType
fn parse_capability(s: &str) -> CapabilityType {
    match s.to_lowercase().as_str() {
        "security" => CapabilityType::Security,
        "storage" => CapabilityType::Storage,
        "compute" => CapabilityType::Compute,
        "ai" => CapabilityType::Ai,
        "orchestration" => CapabilityType::Orchestration,
        "observability" => CapabilityType::Observability,
        "networking" => CapabilityType::Networking,
        other => CapabilityType::Custom(other.to_string()),
    }
}
