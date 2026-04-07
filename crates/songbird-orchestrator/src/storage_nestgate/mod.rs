// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! NestGate-delegating storage: JSON-RPC `storage.*` over a capability-discovered Unix socket.

mod nestgate_impl;
mod wire;

pub use nestgate_impl::NestGateStorage;
pub(crate) use wire::storage_socket_path_from_endpoint;
