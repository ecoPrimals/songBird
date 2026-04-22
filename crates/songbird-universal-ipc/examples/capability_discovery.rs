// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Capability-based discovery + Universal IPC example
//!
//! This demonstrates the complete integration:
//! 1. Discover providers by capability (no hardcoded primal names!)
//! 2. Connect via Universal IPC (platform-agnostic!)
//! 3. Communicate (works everywhere!)

use songbird_universal_ipc::{capability, ipc};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🌍 Capability Discovery + Universal IPC Demo\n");

    // Initialize both systems
    capability::discovery::init_capability_registry();
    ipc::init()?;

    // Set up example: simulate a crypto provider
    let example_socket = std::env::temp_dir().join("example-crypto.sock");
    songbird_process_env::set_var(
        "CRYPTO_PROVIDER_SOCKET",
        example_socket.to_string_lossy().as_ref(),
    );

    println!("📝 Step 1: Discover crypto provider by capability");
    println!("   (No hardcoded primal names - pure capability-based!)");

    match capability::discovery::discover("crypto").await {
        Ok(provider) => {
            println!("   ✅ Found crypto provider: {}", provider.id);
            println!("      Virtual endpoint: {}", provider.virtual_endpoint);
            println!("      Capabilities: {:?}", provider.capabilities);
            println!("      Discovery method: {}", provider.metadata.discovery_method);
            println!();

            // Register the provider with Universal IPC
            println!("📝 Step 2: Register provider with Universal IPC");
            let endpoint = ipc::register(&provider.id, provider.capabilities.clone()).await?;
            println!("   ✅ Registered at: {}", endpoint.path);
            println!();

            // Now any primal can connect using just the capability!
            println!("📝 Step 3: Connect to provider (platform-agnostic!)");
            println!("   Application code:");
            println!("   ```rust");
            println!("   // Discover by capability");
            println!("   let provider = capability::discover(\"crypto\").await?;");
            println!();
            println!("   // Connect via Universal IPC");
            println!("   let stream = ipc::connect(&provider.virtual_endpoint).await?;");
            println!("   ```");
            println!();

            println!("✅ Integration complete!");
            println!();
            println!("🎯 Key Benefits:");
            println!("   ✅ No hardcoded primal names");
            println!("   ✅ Platform-agnostic (works on Linux, macOS, Windows)");
            println!("   ✅ Runtime discovery");
            println!("   ✅ Graceful fallbacks");
            println!("   ✅ TRUE PRIMAL self-knowledge");
        }
        Err(e) => {
            println!("   ❌ No crypto provider found: {e}");
            println!("      This is expected if no provider is running");
            println!("      Demonstrates graceful fallback!");
        }
    }

    // Clean up
    songbird_process_env::remove_var("CRYPTO_PROVIDER_SOCKET");

    println!();
    println!("🌍 Demo complete!");

    Ok(())
}
