// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! # Songbird QUIC Protocol
//!
//! Pure Rust QUIC implementation with `BearDog` crypto delegation.
//!
//! ## Features
//!
//! - **0-RTT Connection**: Faster than TLS 1.3 (zero round-trip time)
//! - **Connection Migration**: Survives IP address changes (mobile roaming)
//! - **Multiplexed Streams**: No head-of-line blocking
//! - **Dark Forest Compliant**: Zero metadata leakage
//! - **BearDog-Only Crypto**: All cryptographic operations delegated
//!
//! ## Architecture
//!
//! ```text
//! Application Data
//!     ↓
//! BearDog ChaCha20Poly1305 (application-level encryption)
//!     ↓
//! QUIC Transport (quinn - connection mgmt, congestion control)
//!     ↓ (QUIC has built-in TLS 1.3)
//! UDP
//!     ↓
//! IPv4/IPv6
//! ```
//!
//! ## Dark Forest Guarantees
//!
//! - No plaintext metadata in QUIC headers
//! - Ephemeral connection IDs (non-correlatable)
//! - Encrypted SNI (no domain name leakage)
//! - All application data BearDog-encrypted
//!
//! ## Usage
//!
//! ```rust,ignore
//! use songbird_quic::{QuicServer, QuicClient};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Server
//! let server = QuicServer::new(
//!     "[::]:4433",
//!     neural_api_socket_path,
//! ).await?;
//!
//! // Client
//! let client = QuicClient::new(neural_api_socket_path).await?;
//! let conn = client.connect("[2600::27]:4433").await?;
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    reason = "intentional pattern; clippy false positive for this API"
)]

mod client;
mod config;
mod connection;
mod error;
mod server;
mod stream;

pub use client::QuicClient;
pub use config::QuicConfig;
pub use connection::QuicConnection;
pub use error::{QuicError, Result};
pub use server::QuicServer;
pub use stream::QuicStream;

/// QUIC protocol version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default QUIC port (IANA recommended)
pub const DEFAULT_QUIC_PORT: u16 = 4433;

/// Maximum transmission unit
pub const MAX_MTU: usize = 1200;

/// Connection ID length (ephemeral, random)
pub const CONNECTION_ID_LEN: usize = 20;
