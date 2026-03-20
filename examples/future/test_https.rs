// SPDX-License-Identifier: AGPL-3.0-only
//! Simple HTTPS test example for Songbird
//!
//! Tests the TLS 1.3 client against real HTTPS servers.
//!
//! Usage:
//!   cargo run --example test_https -- https://example.com
//!   cargo run --example test_https -- https://github.com
//!   cargo run --example test_https -- https://google.com

use songbird_http_client::SongbirdHttpClient;
use std::env;
use tracing::{error, info};
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    info!("");
    info!("═══════════════════════════════════════════════════════════════");
    info!("🔬 SONGBIRD HTTPS TEST - Pure Rust TLS 1.3");
    info!("═══════════════════════════════════════════════════════════════");
    info!("");

    // Get URL from command line args
    let args: Vec<String> = env::args().collect();
    let url = if args.len() > 1 {
        &args[1]
    } else {
        "https://example.com"
    };

    info!("🌐 Target: {}", url);
    info!("🔧 Mode: From environment (BEARDOG_MODE)");
    info!("");

    // Create HTTP client (uses environment configuration)
    let client = SongbirdHttpClient::from_env();
    info!("✅ Client created");
    info!("");

    info!("═══════════════════════════════════════════════════════════════");
    info!("🚀 SENDING HTTP GET REQUEST");
    info!("═══════════════════════════════════════════════════════════════");
    info!("");

    // Make the request
    match client.get(url).await {
        Ok(response) => {
            info!("");
            info!("═══════════════════════════════════════════════════════════════");
            info!("✅ SUCCESS! HTTP RESPONSE RECEIVED");
            info!("═══════════════════════════════════════════════════════════════");
            info!("");
            info!("Status: {}", response.status);
            info!("Headers: {} headers received", response.headers.len());

            // Show some key headers
            if let Some(content_type) = response.headers.get("content-type") {
                info!("  Content-Type: {}", content_type);
            }
            if let Some(content_length) = response.headers.get("content-length") {
                info!("  Content-Length: {}", content_length);
            }
            if let Some(server) = response.headers.get("server") {
                info!("  Server: {}", server);
            }

            info!("");
            info!("Body: {} bytes", response.body.to_string().len());

            // Show first 200 chars of body
            let body_str = response.body.to_string();
            let preview = if body_str.len() > 200 {
                format!("{}...", &body_str[..200])
            } else {
                body_str.clone()
            };
            info!("");
            info!("Body Preview:");
            info!("{}", preview);
            info!("");
            info!("═══════════════════════════════════════════════════════════════");
            info!("🎉 TEST PASSED! Pure Rust HTTPS Working!");
            info!("═══════════════════════════════════════════════════════════════");
            info!("");

            Ok(())
        }
        Err(e) => {
            error!("");
            error!("═══════════════════════════════════════════════════════════════");
            error!("❌ TEST FAILED");
            error!("═══════════════════════════════════════════════════════════════");
            error!("");
            error!("Error: {}", e);
            error!("");
            error!("This may indicate:");
            error!("  • TLS handshake issue");
            error!("  • Certificate validation problem");
            error!("  • Network connectivity issue");
            error!("  • Server-specific requirements");
            error!("");
            error!("Check logs above for detailed error information.");
            error!("");

            Err(e)
        }
    }
}
