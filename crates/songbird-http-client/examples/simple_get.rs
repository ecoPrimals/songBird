#![allow(
    clippy::clone_on_ref_ptr,
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions"
)]
// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Simple HTTP GET example

use songbird_http_client::SongbirdHttpClient;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create client
    let beardog_socket =
        std::env::var("BEARDOG_SOCKET").unwrap_or_else(|_| "/tmp/beardog-nat0.sock".to_string());

    let client = SongbirdHttpClient::new(beardog_socket);

    // Make GET request
    println!("Making GET request to httpbin.org...");

    let response = client.request("GET", "http://httpbin.org/get", HashMap::new(), None).await?;

    println!("Status: {}", response.status);
    println!("Headers: {:#?}", response.headers);
    println!("Body: {}", serde_json::to_string_pretty(&response.body)?);

    Ok(())
}
