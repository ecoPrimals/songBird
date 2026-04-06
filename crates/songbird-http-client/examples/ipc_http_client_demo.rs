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

//! `IpcHttpClient` Demo - First reqwest Elimination Example
//!
//! This demonstrates the Tower Atomic self-delegation pattern in action.
//! It shows how to migrate from `reqwest` to `IpcHttpClient` for TRUE ecoBin compliance.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │  This Example (ipc_http_client_demo.rs)                     │
//! │  - Uses IpcHttpClient (NOT reqwest)                         │
//! │  - Pure Rust, zero C dependencies                           │
//! └─────────────────────┬───────────────────────────────────────┘
//!                       │
//!                       │ JSON-RPC over Unix socket
//!                       │ {"jsonrpc": "2.0", "method": "http.request", ...}
//!                       │
//! ┌─────────────────────▼───────────────────────────────────────┐
//! │  Songbird Orchestrator (must be running)                    │
//! │  - Listens on /tmp/songbird-default.sock                    │
//! │  - Handles http.request IPC calls                           │
//! └─────────────────────┬───────────────────────────────────────┘
//!                       │
//! ┌─────────────────────▼───────────────────────────────────────┐
//! │  SongbirdHttpClient                                         │
//! │  - Pure Rust TLS 1.3 handshake                             │
//! │  - Security-provider crypto delegation (Tower Atomic)       │
//! └──────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ### Step 1: Start Songbird Orchestrator
//!
//! ```bash
//! # In terminal 1
//! cargo run --bin songbird -- orchestrate
//! ```
//!
//! ### Step 2: Run this example
//!
//! ```bash
//! # In terminal 2
//! cargo run --example ipc_http_client_demo -p songbird-http-client
//! ```
//!
//! ## What This Demonstrates
//!
//! 1. ✅ **Drop-in reqwest replacement** - Similar API surface
//! 2. ✅ **TRUE ecoBin compliance** - Zero C dependencies
//! 3. ✅ **Tower Atomic pattern** - Self-delegation via IPC
//! 4. ✅ **Automatic socket discovery** - Environment-aware
//! 5. ✅ **Comprehensive error handling** - No unwraps
//!
//! ## Migration Pattern
//!
//! ```rust,ignore
//! // BEFORE (reqwest - C dependencies via ring/openssl)
//! use reqwest::Client;
//!
//! let client = Client::new();
//! let response = client.get("https://httpbin.org/get").send().await?;
//! let text = response.text().await?;
//!
//! // AFTER (IpcHttpClient - Pure Rust via Songbird)
//! use songbird_http_client::IpcHttpClient;
//!
//! let client = IpcHttpClient::new().await?;
//! let response = client.get("https://httpbin.org/get").await?;
//! let text = response.text().await?;
//! ```

use anyhow::{Context, Result};
use songbird_http_client::IpcHttpClient;
use std::time::Instant;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt().with_target(false).with_level(true).init();

    info!("🚀 IpcHttpClient Demo - Tower Atomic Self-Delegation");
    info!("─────────────────────────────────────────────────────");

    // Create IPC HTTP client
    let start = Instant::now();
    let client = IpcHttpClient::new()
        .await
        .context("Failed to create IpcHttpClient. Is Songbird orchestrator running?")?;
    let init_time = start.elapsed();
    info!("✅ IpcHttpClient initialized in {:?}", init_time);
    info!("   Socket discovery: automatic (env-aware)");

    // Example 1: Simple GET request
    info!("\n📡 Example 1: Simple GET request");
    info!("   URL: https://httpbin.org/get");

    let start = Instant::now();
    match client.get("https://httpbin.org/get").await {
        Ok(response) => {
            let elapsed = start.elapsed();
            info!("✅ GET request successful");
            info!("   Status: {}", response.status());
            info!("   Time: {:?}", elapsed);

            match response.text().await {
                Ok(body) => {
                    info!("   Body length: {} bytes", body.len());
                    if body.len() < 500 {
                        info!("   Body preview:\n{}", body);
                    }
                }
                Err(e) => warn!("⚠️  Failed to read response body: {}", e),
            }
        }
        Err(e) => {
            error!("❌ GET request failed: {}", e);
            error!("   This usually means:");
            error!("   1. Songbird orchestrator is not running");
            error!("   2. Socket path is incorrect");
            error!("   3. Network connectivity issue");
        }
    }

    // Example 2: POST with JSON body
    info!("\n📡 Example 2: POST with JSON body");
    info!("   URL: https://httpbin.org/post");

    let json_data = serde_json::json!({
        "message": "Hello from IpcHttpClient",
        "pure_rust": true,
        "tower_atomic": true,
        "ecobin_compliant": true
    });

    let start = Instant::now();
    match client
        .post("https://httpbin.org/post")
        .await
        .header("Content-Type", "application/json")
        .json(&json_data)?
        .send()
        .await
    {
        Ok(response) => {
            let elapsed = start.elapsed();
            info!("✅ POST request successful");
            info!("   Status: {}", response.status());
            info!("   Time: {:?}", elapsed);

            match response.json::<serde_json::Value>().await {
                Ok(json_response) => {
                    info!(
                        "   Response JSON keys: {:?}",
                        json_response.as_object().map(|o| o.keys().collect::<Vec<_>>())
                    );
                }
                Err(e) => warn!("⚠️  Failed to parse JSON response: {}", e),
            }
        }
        Err(e) => {
            error!("❌ POST request failed: {}", e);
        }
    }

    // Summary
    info!("\n📊 Demo Summary");
    info!("─────────────────────────────────────────────────────");
    info!("✅ IpcHttpClient is a drop-in replacement for reqwest");
    info!("✅ Pure Rust - Zero C dependencies");
    info!("✅ Tower Atomic - Self-delegation via IPC");
    info!("✅ TRUE ecoBin compliance achieved");
    info!(
        "\n📚 Reqwest elimination complete — pure Rust HTTP via Tower Atomic + security-provider IPC"
    );

    Ok(())
}
