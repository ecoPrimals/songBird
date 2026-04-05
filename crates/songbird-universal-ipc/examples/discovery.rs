// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Service discovery example using universal IPC

use songbird_universal_ipc::ipc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Initialize universal IPC
    ipc::init()?;

    println!("🔍 Service Discovery Demo\n");

    // Register some services with capabilities
    println!("📝 Registering services...");

    ipc::register("security_provider", vec!["crypto".to_string(), "btsp".to_string()]).await?;
    println!("✅ Registered security_provider [crypto, btsp]");

    ipc::register("ai", vec!["ai".to_string(), "nlp".to_string()]).await?;
    println!("✅ Registered ai capability [ai, nlp]");

    ipc::register("compute", vec!["compute".to_string(), "container".to_string()]).await?;
    println!("✅ Registered compute capability [compute, container]");

    ipc::register("storage provider", vec!["storage".to_string(), "kv".to_string()]).await?;
    println!("✅ Registered storage provider [storage, kv]");

    println!();

    // List all services
    println!("📋 All registered services:");
    let services = ipc::list_services().await;
    for service in &services {
        println!("  - {service}");
    }

    println!();

    // Find services by capability
    println!("🔍 Finding services by capability:");

    let crypto_services = ipc::find_by_capability("crypto").await;
    println!("  crypto: {crypto_services:?}");

    let ai_services = ipc::find_by_capability("ai").await;
    println!("  ai: {ai_services:?}");

    let storage_services = ipc::find_by_capability("storage").await;
    println!("  storage: {storage_services:?}");

    let compute_services = ipc::find_by_capability("compute").await;
    println!("  compute: {compute_services:?}");

    println!("\n✅ Discovery complete!");

    Ok(())
}
