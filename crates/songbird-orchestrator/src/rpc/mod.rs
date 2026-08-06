// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Multi-protocol RPC module for Songbird
//!
//! ## Protocols
//!
//! - **JSON-RPC 2.0** (HTTP): `server/jsonrpc_api.rs` — universal access
//! - **JSON-RPC 2.0** (Unix socket): `ipc/pure_rust_server/` — inter-primal IPC
//! - **tarpc**: High-performance binary RPC for primal-to-primal
pub mod tarpc_server;

pub use self::tarpc_server::{
    TarpcConfig, TarpcServer, TarpcServerSimple, start_tarpc_server, start_tarpc_server_simple,
};

#[cfg(unix)]
pub use self::tarpc_server::start_tarpc_uds_server;

// Re-export SongbirdRpc from songbird-universal (v3.12.0)
pub use songbird_universal::tarpc_types::SongbirdRpc;
