// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! IPC storage backend: JSON-RPC `storage.*` over a capability-discovered Unix socket.

mod ipc_backend;
mod wire;

pub use ipc_backend::IpcStorageBackend;

pub(crate) use wire::storage_socket_path_from_endpoint;
