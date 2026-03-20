// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Default resource configuration with environment variable support
//!
//! # Environment Variables
//!
//! - `SONGBIRD_MAX_SERVICES` - Maximum number of services (default: 1000)
//! - `SONGBIRD_MAX_CACHE_SIZE` - Maximum cache size (default: 10000)
//! - `SONGBIRD_BUFFER_POOL_SIZE` - Buffer pool size (default: 100)
//! - `SONGBIRD_MAX_CONNECTIONS` - Maximum connections (default: 1000)
//! - `SONGBIRD_MAX_SESSIONS` - Maximum sessions (default: 1000)

use std::env;

/// Get maximum number of services from environment or default
///
/// # Environment Variable
/// `SONGBIRD_MAX_SERVICES` (default: 1000)
#[must_use]
pub fn max_services() -> u32 {
    env::var("SONGBIRD_MAX_SERVICES").ok().and_then(|t| t.parse().ok()).unwrap_or(1000)
}

/// Get maximum cache size from environment or default
///
/// # Environment Variable
/// `SONGBIRD_MAX_CACHE_SIZE` (default: 10000)
#[must_use]
pub fn max_cache_size() -> usize {
    env::var("SONGBIRD_MAX_CACHE_SIZE").ok().and_then(|t| t.parse().ok()).unwrap_or(10_000)
}

/// Get buffer pool size from environment or default
///
/// # Environment Variable
/// `SONGBIRD_BUFFER_POOL_SIZE` (default: 100)
#[must_use]
pub fn get_buffer_pool_size() -> usize {
    env::var("SONGBIRD_BUFFER_POOL_SIZE").ok().and_then(|t| t.parse().ok()).unwrap_or(100)
}

/// Get maximum connections from environment or default
///
/// # Environment Variable
/// `SONGBIRD_MAX_CONNECTIONS` (default: 1000)
#[must_use]
pub fn get_max_connections() -> usize {
    env::var("SONGBIRD_MAX_CONNECTIONS").ok().and_then(|t| t.parse().ok()).unwrap_or(1000)
}

/// Get maximum sessions from environment or default
///
/// # Environment Variable
/// `SONGBIRD_MAX_SESSIONS` (default: 1000)
#[must_use]
pub fn get_max_sessions() -> usize {
    env::var("SONGBIRD_MAX_SESSIONS").ok().and_then(|t| t.parse().ok()).unwrap_or(1000)
}

#[cfg(test)]
#[path = "resources_tests.rs"]
mod tests;
