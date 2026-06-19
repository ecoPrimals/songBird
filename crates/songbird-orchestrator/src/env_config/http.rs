// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use songbird_types::defaults::{hosts::DEFAULT_BIND_ALL, ports::DEFAULT_HTTP_PORT};

use super::env;

/// Get HTTP server bind address (self-knowledge)
///
/// Resolution order:
/// 1. `SONGBIRD_HTTP_ADDR` (explicit override)
/// 2. `DEFAULT_BIND_ALL`:`DEFAULT_HTTP_PORT` from `songbird_types::defaults` (bind all interfaces)
#[must_use]
pub fn http_bind_address() -> String {
    env("SONGBIRD_HTTP_ADDR").unwrap_or_else(|_| format!("{DEFAULT_BIND_ALL}:{DEFAULT_HTTP_PORT}"))
}

/// Get HTTP server port (self-knowledge)
///
/// Resolution order:
/// 1. `SONGBIRD_HTTP_PORT` (explicit override)
/// 2. Extract from `SONGBIRD_HTTP_ADDR` if set
/// 3. `DEFAULT_HTTP_PORT` from `songbird_types::defaults` (default)
#[must_use]
pub fn http_port() -> u16 {
    if let Ok(port_str) = env("SONGBIRD_HTTP_PORT") {
        return port_str.parse().unwrap_or(DEFAULT_HTTP_PORT);
    }

    if let Ok(addr) = env("SONGBIRD_HTTP_ADDR")
        && let Some(port_part) = addr.split(':').nth(1)
        && let Ok(port) = port_part.parse()
    {
        return port;
    }

    DEFAULT_HTTP_PORT
}

/// Check if running in production mode
///
/// Resolution order:
/// 1. `SONGBIRD_ENV == "production"`
/// 2. `RUST_ENV == "production"`
/// 3. `false` (default to development)
#[must_use]
pub fn is_production() -> bool {
    env("SONGBIRD_ENV").or_else(|_| env("RUST_ENV")).map(|v| v == "production").unwrap_or(false)
}

/// Get log level (self-knowledge)
///
/// Resolution order:
/// 1. `SONGBIRD_LOG` (explicit override)
/// 2. `RUST_LOG` (Rust standard)
/// 3. `"info"` (default)
#[must_use]
pub fn log_level() -> String {
    env("SONGBIRD_LOG").or_else(|_| env("RUST_LOG")).unwrap_or_else(|_| String::from("info"))
}
