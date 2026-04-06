// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Default host/address bindings
//!
//! Primals discover actual addresses via capability-based discovery at runtime.

/// Default bind address for all interfaces
pub const DEFAULT_BIND_ALL: &str = "0.0.0.0";
/// Default loopback address
pub const DEFAULT_LOOPBACK: &str = "127.0.0.1";
/// Default loopback IPv6
pub const DEFAULT_LOOPBACK_V6: &str = "::1";
