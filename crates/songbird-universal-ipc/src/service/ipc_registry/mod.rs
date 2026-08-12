// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! IPC registry handlers: register, resolve, discover, lifecycle.

mod lifecycle;
mod query;
mod register;
mod resolve;
mod transport;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

pub(in crate::service) use transport::transport_endpoint_from_native;
