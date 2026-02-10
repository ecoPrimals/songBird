//! Integration example: Using capability discovery and primal self-knowledge
//!
//! This example shows how the new architecture eliminates hardcoding.

use songbird_discovery::{CapabilityPortDiscovery, PrimalSelfKnowledge};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("=== Primal Self-Knowledge Example ===\n");

    // STEP 1: Discover own identity (no hardcoding)
    let self_knowledge = PrimalSelfKnowledge::discover_self()?;
    let identity = self_knowledge.identity();

    println!("✓ Discovered self:");
    println!("  Name: {}", identity.name);
    println!("  Capabilities: {:?}\n", identity.capabilities);

    // STEP 2: Set up capability-based port discovery
    let port_discovery = CapabilityPortDiscovery::new();

    println!("=== Capability-Based Port Discovery ===\n");

    // STEP 3: Discover ports for capabilities (no hardcoded ports!)
    for capability in &identity.capabilities {
        match port_discovery.discover_port(capability).await {
            Ok(port) => {
                println!("✓ Capability '{}' -> port {}", capability, port);
                port_discovery.register_local(capability.clone(), port).await;
            }
            Err(e) => {
                println!("✗ Failed to discover port for '{}': {}", capability, e);
            }
        }
    }

    println!("\n=== Discovering Other Primals ===\n");

    // STEP 4: Discover other primals at runtime (no hardcoded knowledge!)
    let needed_capabilities = vec!["storage", "ai", "security"];

    for capability in needed_capabilities {
        match self_knowledge.discover_primal(capability).await {
            Ok(primal_info) => {
                println!("✓ Found primal for '{}':", capability);
                println!("  Name: {}", primal_info.name);
                println!("  Address: {}:{}", primal_info.host, primal_info.port);
                println!("  Method: {}", primal_info.discovery_method);
            }
            Err(e) => {
                println!("✗ Could not discover '{}': {}", capability, e);
                println!("  (This is normal if the primal isn't running)");
            }
        }
        println!();
    }

    // STEP 5: Show all discovered primals
    let discovered = self_knowledge.discovered().await;
    println!("=== Summary ===");
    println!("Total primals discovered: {}", discovered.len());

    for (capability, info) in discovered {
        println!("  {} -> {}:{}", capability, info.host, info.port);
    }

    println!("\n✓ No hardcoded ports or primal knowledge used!");
    println!("✓ All discovery was runtime and capability-based!");

    Ok(())
}
