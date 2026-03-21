// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! IPC HTTP Client - Self-Delegation Pattern
//!
//! Pure Rust HTTP client that delegates to Songbird's own HTTP service via IPC.
//!
//! ## Architecture: Tower Atomic Self-Delegation
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────┐
//! │  Application Code (Discovery, Config, etc.)              │
//! │  "I need to make an HTTP request"                        │
//! └─────────────────────┬────────────────────────────────────┘
//!                       │
//!                       │ IpcHttpClient::new()
//!                       │ client.get("https://...").await?
//!                       │
//! ┌─────────────────────▼────────────────────────────────────┐
//! │  IpcHttpClient (this module)                             │
//! │  - Provides HTTP client API                              │
//! │  - Delegates via JSON-RPC over Unix socket               │
//! │  - Zero C dependencies                                   │
//! └─────────────────────┬────────────────────────────────────┘
//!                       │
//!                       │ JSON-RPC: {"method": "http.request", ...}
//!                       │ Socket: /primal/songbird
//!                       │
//! ┌─────────────────────▼────────────────────────────────────┐
//! │  Songbird IPC Handler                                    │
//! │  (src/ipc/handlers/http.rs)                             │
//! └─────────────────────┬────────────────────────────────────┘
//!                       │
//! ┌─────────────────────▼────────────────────────────────────┐
//! │  SongbirdHttpClient                                      │
//! │  - Pure Rust TLS 1.3                                    │
//! │  - Tower Atomic with BearDog                            │
//! └──────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```rust,no_run
//! use songbird_http_client::IpcHttpClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create client (connects to Songbird via IPC)
//!     let client = IpcHttpClient::new().await?;
//!
//!     // Make HTTP GET request
//!     let response = client.get("https://api.github.com/repos/rust-lang/rust").await?;
//!     
//!     println!("Status: {}", response.status());
//!     println!("Body: {}", response.text().await?);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Migration from legacy HTTP clients
//!
//! ```rust,ignore
//! // BEFORE (legacy - C dependencies)
//! use legacy_http::Client;
//!
//! let client = Client::new();
//! let response = client.get(url).send().await?;
//! let text = response.text().await?;
//!
//! // AFTER (IpcHttpClient - Pure Rust via IPC)
//! use songbird_http_client::IpcHttpClient;
//!
//! let client = IpcHttpClient::new().await?;
//! let response = client.get(url).await?;
//! let text = response.text().await?;
//! ```
//!
//! ## Benefits
//!
//! - ✅ **Pure Rust**: Zero C dependencies (TRUE ecoBin compliant)
//! - ✅ **Self-Delegation**: Reuses Songbird's own HTTP client
//! - ✅ **Tower Atomic**: `BearDog` crypto via IPC (no ring/openssl)
//! - ✅ **Simple Migration**: Drop-in replacement for legacy HTTP clients
//! - ✅ **Maintained**: Songbird HTTP client is actively developed

mod client_impl;

#[cfg(test)]
#[path = "client_tests.rs"]
mod tests;

pub use client_impl::{IpcHttpClient, IpcHttpClientBuilder, RequestBuilder, Response};
