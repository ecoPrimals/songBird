// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Reusable TURN relay client library (UB-1).
//!
//! Provides [`TurnSession`] — a high-level abstraction over RFC 5766 TURN
//! that gives downstream consumers (e.g. lithoSpore) a simple API for
//! sending and receiving data through Songbird's TURN relay infrastructure.
//!
//! ## Quick Start
//!
//! ```no_run
//! use songbird_turn_client::{TurnSession, TurnSessionConfig};
//! use songbird_stun::StunCredentials;
//!
//! # async fn example() -> Result<(), songbird_turn_client::TurnSessionError> {
//! let config = TurnSessionConfig::new(
//!     "turn.example.com:3478".parse().unwrap(),
//!     StunCredentials { username: "user".into(), key: b"secret".to_vec() },
//!     "192.0.2.5:9200".parse().unwrap(),  // peer address
//! );
//!
//! let mut session = TurnSession::connect(config).await?;
//!
//! // Send JSON-RPC through the relay
//! session.send(b"{\"jsonrpc\":\"2.0\",\"method\":\"health.check\",\"id\":1}\n").await?;
//!
//! // Receive response
//! let mut buf = vec![0u8; 4096];
//! let n = session.recv(&mut buf).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Data Plane
//!
//! After the control-plane handshake (Allocate → `CreatePermission` → `ChannelBind`),
//! data flows via RFC 5766 `ChannelData` framing: `[channel:u16][length:u16][payload]`.
//! This avoids full STUN message overhead for every packet.
//!
//! When no channel is bound, the session falls back to STUN Send/Data Indication
//! framing (higher overhead but works without `ChannelBind`).
#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod error;
mod session;
pub mod transport_impl;

pub use error::TurnSessionError;
pub use session::{TurnSession, TurnSessionConfig};
pub use transport_impl::TurnClientTransport;

pub use songbird_stun::{StunCredentials, TurnAllocation, TurnClient};
