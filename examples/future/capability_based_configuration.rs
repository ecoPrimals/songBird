// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals
//! Capability-Based Configuration Example
//!
//! This example demonstrates the evolution from hardcoded configuration
//! to modern capability-based, self-aware service configuration.
//!
//! # Philosophy
//! - **Self-Knowledge**: Service knows only itself
//! - **Runtime Discovery**: Other services discovered by capability
//! - **Zero Hardcoding**: No hardcoded addresses or primal names
//! - **Environment Aware**: Configuration adapts to environment

use songbird_config::defaults::hosts_evolved::{Environment, SelfAwareConfig, ServiceLocator};

/// Old pattern (deprecated) - hardcoded configuration
#[allow(dead_code)]
fn old_pattern_hardcoded() {
    // ❌ OLD PATTERN: Hardcoded addresses
    let _orchestrator_host = "127.0.0.1:8080";
    let _security_provider_host = "127.0.0.1:3000";
    let _compute_provider_host = "127.0.0.1:5000";

    println!("❌ Old Pattern: Hardcoded addresses - not maintainable!");
}

/// Old pattern (improved) - environment variables but still hardcoded defaults
#[allow(dead_code)]
fn old_pattern_with_env_vars() {
    // ⚠️ OLD PATTERN: Environment variables with hardcoded fallbacks
    let _host = std::env::var("SONGBIRD_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

    println!("⚠️ Improved but still has hardcoded fallbacks");
}

/// Modern pattern - self-aware, capability-based
fn modern_pattern_self_aware() {
    println!("\n🌟 MODERN PATTERN: Self-Aware Configuration");
    println!("==========================================\n");

    // ✅ Service knows only itself
    let config = SelfAwareConfig::from_environment();

    println!("Self-Knowledge:");
    println!("  Environment: {:?}", config.environment);
    println!("  Bind Address: {}", config.bind_address());
    println!("  Advertise Address: {}", config.advertise_address());
    println!();

    // ✅ Production-aware behavior
    match config.environment {
        Environment::Development => {
            println!("Development Mode:");
            println!("  - Binding to localhost (isolated)");
            println!("  - Permissive security");
            println!("  - Fast iteration");
        }
        Environment::Production => {
            println!("Production Mode:");
            println!("  - Binding to all interfaces");
            println!("  - High security");
            println!("  - Full monitoring");
        }
        Environment::Staging => {
            println!("Staging Mode:");
            println!("  - Production-like configuration");
            println!("  - Pre-deployment validation");
        }
        Environment::Test => {
            println!("Test Mode:");
            println!("  - Isolated, ephemeral");
            println!("  - OS-assigned ports");
        }
    }
}

/// Modern pattern - capability-based discovery
async fn modern_pattern_capability_discovery() {
    println!("\n🌟 MODERN PATTERN: Capability-Based Discovery");
    println!("=============================================\n");

    let locator = ServiceLocator::new();

    println!("Discovering services by CAPABILITY (not by name!):");

    // ✅ No hardcoded primal names!
    // ✅ No hardcoded addresses!
    // ✅ Discover by what services CAN DO, not what they're CALLED

    let storage_services = locator.discover_by_capability("storage").await;
    println!("  Storage capability: {} providers found", storage_services.len());

    let compute_services = locator.discover_by_capability("compute").await;
    println!("  Compute capability: {} providers found", compute_services.len());

    let ai_services = locator.discover_by_capability("ai").await;
    println!("  AI capability: {} providers found", ai_services.len());

    let security_services = locator.discover_by_capability("security").await;
    println!("  Security capability: {} providers found", security_services.len());

    println!();
    println!("Benefits:");
    println!("  ✅ Zero hardcoded primal names");
    println!("  ✅ Zero hardcoded addresses");
    println!("  ✅ Runtime discovery");
    println!("  ✅ Capability-based routing");
    println!("  ✅ Primal-agnostic design");
}

/// Modern pattern - self-registration
async fn modern_pattern_self_registration() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🌟 MODERN PATTERN: Self-Registration");
    println!("=====================================\n");

    let locator = ServiceLocator::new();

    // ✅ Service registers itself with capabilities
    // ✅ Other services discover it by querying for those capabilities
    let capabilities = vec!["orchestration", "service-mesh", "load-balancing", "discovery"];

    println!("Registering self with capabilities:");
    for cap in &capabilities {
        println!("  - {}", cap);
    }

    locator.register_self(&capabilities).await?;

    println!();
    println!("Self-Registration Complete!");
    println!("  ✅ Announced advertise address");
    println!("  ✅ Tagged with capabilities");
    println!("  ✅ Health check endpoint active");
    println!("  ✅ Other services can now discover us");

    Ok(())
}

/// Comparison: Old vs New
fn comparison() {
    println!("\n📊 COMPARISON: Old Pattern vs Modern Pattern");
    println!("============================================\n");

    println!("OLD PATTERN (Hardcoded):");
    println!("  ❌ Hardcoded IP addresses");
    println!("  ❌ Hardcoded port numbers");
    println!("  ❌ Hardcoded primal names (BEARDOG, TOADSTOOL, etc.)");
    println!("  ❌ Static configuration");
    println!("  ❌ Environment-agnostic");
    println!("  ❌ Tight coupling");
    println!();

    println!("MODERN PATTERN (Capability-Based):");
    println!("  ✅ Zero hardcoded addresses");
    println!("  ✅ Self-aware configuration");
    println!("  ✅ Runtime discovery by capability");
    println!("  ✅ Environment-adaptive");
    println!("  ✅ Loose coupling");
    println!("  ✅ Primal-agnostic design");
    println!("  ✅ Service knows only itself");
    println!();

    println!("IMPACT:");
    println!("  🚀 Easier deployment (no hardcoded config)");
    println!("  🚀 Better scaling (discover new instances)");
    println!("  🚀 Improved resilience (failover to discovered services)");
    println!("  🚀 Ecosystem evolution (add new primals without code changes)");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔═══════════════════════════════════════════════════════╗");
    println!("║   Songbird Capability-Based Configuration Example    ║");
    println!("║                                                       ║");
    println!("║   Evolution from Hardcoded → Self-Aware Discovery    ║");
    println!("╚═══════════════════════════════════════════════════════╝");

    // Show the evolution
    comparison();

    // Demonstrate modern patterns
    modern_pattern_self_aware();
    modern_pattern_capability_discovery().await;
    modern_pattern_self_registration().await?;

    println!("\n✅ Modern capability-based configuration demonstrated!");
    println!("   See code for implementation details.\n");

    Ok(())
}
