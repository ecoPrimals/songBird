// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::clone_on_ref_ptr,
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions"
)]
// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Fetch and parse Tor consensus
//!
//! Example: `cargo run --example fetch_consensus`

use songbird_tor_protocol::{Consensus, CryptoProvider};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    println!("🧅 Fetching Tor network consensus...\n");

    // Create security provider client (placeholder)
    let security_provider = CryptoProvider::from_env();

    // Fetch consensus
    let consensus = Consensus::fetch(&security_provider).await?;

    println!("✅ Consensus fetched successfully!\n");
    println!("📊 Network Statistics:");
    println!("   Total relays: {}", consensus.relays.len());

    // Count relay types
    let guards = consensus.relays.iter().filter(|r| r.is_guard()).count();
    let middle = consensus.relays.iter().filter(|r| r.is_middle()).count();
    let hsdirs = consensus.relays.iter().filter(|r| r.is_hsdir()).count();

    println!("   Guards: {guards}");
    println!("   Middle: {middle}");
    println!("   HSDir: {hsdirs}");

    println!("\n🔍 Sample relays:");
    for relay in consensus.relays.iter().take(5) {
        println!("   {} - {} ({})", relay.nickname, relay.address, relay.or_port);
        println!("      Flags: {:?}", relay.flags);
        println!("      Bandwidth: {} KB/s", relay.bandwidth / 1024);
    }

    // Test path selection
    println!("\n🛤️  Selecting circuit path...");
    match consensus.select_path() {
        Ok(path) => {
            println!(
                "   Guard:  {} ({}:{})",
                path.guard.nickname, path.guard.address, path.guard.or_port
            );
            println!(
                "   Middle: {} ({}:{})",
                path.middle.nickname, path.middle.address, path.middle.or_port
            );
            println!(
                "   Exit:   {} ({}:{})",
                path.exit.nickname, path.exit.address, path.exit.or_port
            );
        }
        Err(e) => println!("   ❌ Path selection failed: {e}"),
    }

    println!("\n✨ Consensus is fresh: {}", consensus.is_fresh());
    println!("✨ Consensus is valid: {}", consensus.is_valid());

    Ok(())
}
