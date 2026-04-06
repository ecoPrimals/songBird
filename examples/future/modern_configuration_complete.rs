// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals
//! Complete Modern Configuration Example
//!
//! This example demonstrates the full evolution of Songbird's configuration
//! system from hardcoded values to capability-based, self-aware, runtime-discovered
//! configuration.
//!
//! # Evolution Stages
//! 1. **Old**: Hardcoded IP addresses and ports
//! 2. **Improved**: Environment variables with hardcoded fallbacks
//! 3. **Modern**: Self-aware, capability-based, runtime discovery

use songbird_config::defaults::{
    hosts_evolved::{Environment, SelfAwareConfig, ServiceLocator},
    ports_evolved::{PortAllocator, ServicePort},
};

/// Stage 1: Old hardcoded configuration (deprecated)
#[allow(dead_code)]
fn stage1_hardcoded() {
    println!("❌ STAGE 1: Hardcoded Configuration (Deprecated)");
    println!("================================================\n");

    // Everything hardcoded - BAD!
    let _orchestrator_addr = "127.0.0.1:8080";
    let _discovery_addr = "127.0.0.1:8081";
    let _beardog_addr = "127.0.0.1:3000"; // Hardcoded primal!
    let _toadstool_addr = "127.0.0.1:5000"; // Hardcoded primal!

    println!("Problems:");
    println!("  ❌ Hardcoded IP addresses");
    println!("  ❌ Hardcoded port numbers");
    println!("  ❌ Hardcoded primal names");
    println!("  ❌ No environment awareness");
    println!("  ❌ Tight coupling");
    println!("  ❌ Deployment hell");
    println!();
}

/// Stage 2: Environment variables with fallbacks (old pattern)
#[allow(dead_code)]
fn stage2_env_vars() {
    println!("⚠️  STAGE 2: Environment Variables (Old Pattern)");
    println!("================================================\n");

    // Better, but still has hardcoded fallbacks
    let _host = std::env::var("SONGBIRD_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let _port: u16 =
        std::env::var("SONGBIRD_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8080);

    println!("Improvements:");
    println!("  ✓ Can override via environment");
    println!("  ✓ Some deployment flexibility");
    println!();

    println!("Remaining Problems:");
    println!("  ⚠️  Still has hardcoded fallbacks");
    println!("  ⚠️  No environment awareness");
    println!("  ⚠️  Static configuration");
    println!("  ⚠️  Manual port management");
    println!();
}

/// Stage 3: Modern capability-based configuration
fn stage3_modern_capability_based() {
    println!("✅ STAGE 3: Modern Capability-Based Configuration");
    println!("================================================\n");

    // Self-aware configuration
    let config = SelfAwareConfig::from_environment();

    println!("Self-Awareness:");
    println!("  Environment: {:?}", config.environment);
    println!("  Bind Address: {}", config.bind_address());
    println!("  Advertise Address: {}", config.advertise_address());
    println!();

    // Capability-based port allocation
    let port_allocator = PortAllocator::new();

    println!("Capability-Based Port Allocation:");
    match port_allocator.allocate_for_capability("orchestration") {
        Ok(listener) => {
            let addr = listener.local_addr().unwrap();
            println!("  Orchestration port: {} (capability-based)", addr.port());
        }
        Err(e) => println!("  Error allocating port: {}", e),
    }

    match port_allocator.allocate_for_capability("discovery") {
        Ok(listener) => {
            let addr = listener.local_addr().unwrap();
            println!("  Discovery port: {} (capability-based)", addr.port());
        }
        Err(e) => println!("  Error allocating port: {}", e),
    }

    println!();

    println!("Port Ranges by Capability:");
    println!("  orchestration: 8000-8099");
    println!("  discovery:     8100-8199");
    println!("  messaging:     8200-8299");
    println!("  storage:       8300-8399");
    println!("  compute:       8400-8499");
    println!("  security:      8500-8599");
    println!("  monitoring:    8600-8699");
    println!("  federation:    8700-8799");
    println!();
}

/// Stage 3 continued: Runtime service discovery
async fn stage3_runtime_discovery() {
    println!("✅ STAGE 3: Runtime Service Discovery");
    println!("=====================================\n");

    let locator = ServiceLocator::new();

    println!("Discovering Services by Capability (Not by Name!):");
    println!();

    // No hardcoded primal names!
    // Discover by what they CAN DO, not what they're CALLED

    let storage_services = locator.discover_by_capability("storage").await;
    println!("  'storage' capability:");
    println!("    Found {} providers", storage_services.len());
    println!("    Could be: NestGate, S3, local filesystem, etc.");
    println!("    ✓ No hardcoding!");
    println!();

    let compute_services = locator.discover_by_capability("compute").await;
    println!("  'compute' capability:");
    println!("    Found {} providers", compute_services.len());
    println!("    Could be: ToadStool, AWS Lambda, local execution, etc.");
    println!("    ✓ No hardcoding!");
    println!();

    let ai_services = locator.discover_by_capability("ai").await;
    println!("  'ai' capability:");
    println!("    Found {} providers", ai_services.len());
    println!("    Could be: Squirrel, OpenAI, local models, etc.");
    println!("    ✓ No hardcoding!");
    println!();

    let security_services = locator.discover_by_capability("security").await;
    println!("  'security' capability:");
    println!("    Found {} providers", security_services.len());
    println!("    Could be: BearDog, Vault, local keystore, etc.");
    println!("    ✓ No hardcoding!");
    println!();
}

/// Stage 3 continued: Self-registration
async fn stage3_self_registration() -> Result<(), Box<dyn std::error::Error>> {
    println!("✅ STAGE 3: Self-Registration");
    println!("==============================\n");

    let locator = ServiceLocator::new();

    // Service registers itself with capabilities
    let capabilities =
        vec!["orchestration", "service-mesh", "load-balancing", "discovery", "routing"];

    println!("Registering Songbird with capabilities:");
    for cap in &capabilities {
        println!("  ✓ {}", cap);
    }
    println!();

    locator.register_self(&capabilities).await?;

    println!("Registration Complete!");
    println!("  ✓ Other services can discover us by capability");
    println!("  ✓ No hardcoded knowledge required");
    println!("  ✓ Runtime binding");
    println!();

    Ok(())
}

/// Complete comparison: Old vs New
fn complete_comparison() {
    println!("\n╔════════════════════════════════════════════════╗");
    println!("║     COMPLETE CONFIGURATION EVOLUTION          ║");
    println!("╚════════════════════════════════════════════════╝\n");

    println!("OLD CONFIGURATION (Stages 1-2):");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  ❌ Hardcoded IP addresses (127.0.0.1)");
    println!("  ❌ Hardcoded port numbers (8080, 8081, 3000, 5000)");
    println!("  ❌ Hardcoded primal names (BEARDOG, TOADSTOOL, SQUIRREL)");
    println!("  ❌ Static configuration files");
    println!("  ❌ Environment-agnostic");
    println!("  ❌ Manual port conflict resolution");
    println!("  ❌ Tight coupling between services");
    println!("  ❌ Deployment complexity");
    println!();

    println!("NEW CONFIGURATION (Stage 3):");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  ✅ Zero hardcoded IP addresses");
    println!("  ✅ Capability-based port allocation");
    println!("  ✅ Zero hardcoded primal names");
    println!("  ✅ Runtime service discovery");
    println!("  ✅ Environment-aware behavior");
    println!("  ✅ Automatic port allocation & conflict avoidance");
    println!("  ✅ Loose coupling via capabilities");
    println!("  ✅ Simple deployment (self-configuring)");
    println!();

    println!("KEY BENEFITS:");
    println!("━━━━━━━━━━━━━");
    println!("  🚀 Easier Deployment");
    println!("     - No config files to manage");
    println!("     - Self-configuring services");
    println!("     - Environment auto-detection");
    println!();

    println!("  🚀 Better Scaling");
    println!("     - Automatic port allocation");
    println!("     - Dynamic service discovery");
    println!("     - No port conflicts");
    println!();

    println!("  🚀 Improved Resilience");
    println!("     - Discover new instances automatically");
    println!("     - Failover to available services");
    println!("     - No single points of failure in config");
    println!();

    println!("  🚀 Ecosystem Evolution");
    println!("     - Add new primals without code changes");
    println!("     - Replace implementations transparently");
    println!("     - Capability-based compatibility");
    println!();
}

/// Environment-specific behavior demonstration
fn environment_adaptation() {
    println!("\n╔════════════════════════════════════════════════╗");
    println!("║       ENVIRONMENT-ADAPTIVE BEHAVIOR            ║");
    println!("╚════════════════════════════════════════════════╝\n");

    let env = Environment::detect();

    println!("Current Environment: {:?}", env);
    println!();

    match env {
        Environment::Development => {
            println!("Development Mode Behavior:");
            println!("  • Bind to localhost (127.0.0.1) - isolated");
            println!("  • Permissive security settings");
            println!("  • Verbose logging");
            println!("  • Fast iteration (hot reload enabled)");
            println!("  • Mock external services");
        }
        Environment::Test => {
            println!("Test Mode Behavior:");
            println!("  • Bind to localhost with OS-assigned ports");
            println!("  • Isolated test environment");
            println!("  • Ephemeral configuration");
            println!("  • Reproducible setup/teardown");
            println!("  • No external dependencies");
        }
        Environment::Staging => {
            println!("Staging Mode Behavior:");
            println!("  • Bind to all interfaces (0.0.0.0)");
            println!("  • Production-like configuration");
            println!("  • Full security enabled");
            println!("  • Pre-deployment validation");
            println!("  • Real service integration");
        }
        Environment::Production => {
            println!("Production Mode Behavior:");
            println!("  • Bind to all interfaces (0.0.0.0)");
            println!("  • Maximum security hardening");
            println!("  • Comprehensive monitoring");
            println!("  • Performance optimization");
            println!("  • Real-time alerting");
        }
    }
    println!();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║                                                           ║");
    println!("║    SONGBIRD CONFIGURATION EVOLUTION COMPLETE EXAMPLE      ║");
    println!("║                                                           ║");
    println!("║    From Hardcoded → Modern Capability-Based              ║");
    println!("║                                                           ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();

    // Show environment adaptation first
    environment_adaptation();

    println!("\n");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    // Show the evolution stages
    stage1_hardcoded();
    println!("───────────────────────────────────────────────────────────\n");

    stage2_env_vars();
    println!("───────────────────────────────────────────────────────────\n");

    stage3_modern_capability_based();
    println!("───────────────────────────────────────────────────────────\n");

    stage3_runtime_discovery().await;
    println!("───────────────────────────────────────────────────────────\n");

    stage3_self_registration().await?;
    println!("───────────────────────────────────────────────────────────\n");

    // Complete comparison
    complete_comparison();

    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║                                                           ║");
    println!("║         ✅ MODERN CONFIGURATION DEMONSTRATED!             ║");
    println!("║                                                           ║");
    println!("║    Zero Hardcoding • Capability-Based • Self-Aware        ║");
    println!("║                                                           ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();

    Ok(())
}
