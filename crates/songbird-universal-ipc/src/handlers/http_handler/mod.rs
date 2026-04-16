// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! HTTP/HTTPS IPC Handler - Deep Solution Implementation
//!
//! This module provides JSON-RPC 2.0 handlers for HTTP/HTTPS requests via IPC,
//! exposing Songbird's Pure Rust TLS 1.3 capability to the ecosystem.
//!
//! ## Architecture
//!
//! ```text
//! biomeOS → JSON-RPC → IPC Handler → HTTP Client Factory → security provider (crypto)
//!                                  ↓
//!                        Pure Rust TLS 1.3 (Tower Atomic)
//! ```
//!
//! ## Design Principles
//!
//! 1. **Capability-Based Discovery** - No hardcoded `security provider` endpoints
//! 2. **Factory Pattern** - Dependency injection for testability
//! 3. **Enum dispatch** - Concrete and test doubles without dynamic dispatch
//! 4. **Proper Error Handling** - No unwrap/expect
//! 5. **Modern Async** - tokio, async/await throughout

mod client;
mod env_discovery;
mod handler;
mod http_dispatch;
#[cfg(test)]
mod test_support;
mod types;

pub use client::SongbirdHttpClient;
pub use env_discovery::EnvCryptoDiscovery;
pub use handler::HttpHandler;
pub use http_dispatch::{CryptoDiscovery, DefaultHttpClientFactory, HttpClient, HttpClientFactory};
pub use types::{HttpRequestParams, HttpResponse, HttpResponseResult};

#[cfg(test)]
pub use test_support::RotatingMockClient;

#[cfg(test)]
mod tests;
