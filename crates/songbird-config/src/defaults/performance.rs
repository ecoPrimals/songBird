// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Performance configuration defaults with environment variable support
//!
//! Provides configurable performance settings via environment variables.
//!
//! # Environment Variables
//!
//! - `SONGBIRD_THREAD_POOL_SIZE` - Thread pool size (default: available parallelism or 4)
//! - `SONGBIRD_THREAD_POOL_SIZE_SMALL` - Small thread pool (default: 4)
//! - `SONGBIRD_THREAD_POOL_SIZE_MIN` - Minimal thread pool (default: 2)
//! - `SONGBIRD_THREAD_POOL_SIZE_LARGE` - Large thread pool (default: 10)
//! - `SONGBIRD_MAX_CONNECTIONS_DEFAULT` - Default max connections (default: 1000)
//! - `SONGBIRD_MAX_CONNECTIONS_SMALL` - Small max connections (default: 10)
//! - `SONGBIRD_MAX_CONNECTIONS_MEDIUM` - Medium max connections (default: 100)

use songbird_types::SafeEnv;

/// Get default thread pool size (uses CPU count if available)
#[must_use]
pub fn thread_pool_size() -> usize {
    SafeEnv::get_usize(
        "SONGBIRD_THREAD_POOL_SIZE",
        std::thread::available_parallelism().map_or(1, std::num::NonZero::get),
    )
}

/// Get small thread pool size
#[must_use]
pub fn thread_pool_size_small() -> usize {
    SafeEnv::get_usize("SONGBIRD_THREAD_POOL_SIZE_SMALL", 4)
}

/// Get minimal thread pool size (for testing/constrained environments)
#[must_use]
pub fn thread_pool_size_min() -> usize {
    SafeEnv::get_usize("SONGBIRD_THREAD_POOL_SIZE_MIN", 2)
}

/// Get medium thread pool size
#[must_use]
pub fn thread_pool_size_medium() -> usize {
    SafeEnv::get_usize("SONGBIRD_THREAD_POOL_SIZE_MEDIUM", 10)
}

/// Get large thread pool size
#[must_use]
pub fn thread_pool_size_large() -> usize {
    SafeEnv::get_usize("SONGBIRD_THREAD_POOL_SIZE_LARGE", 10)
}

/// Get default maximum connections
#[must_use]
pub fn max_connections_default() -> usize {
    SafeEnv::get_usize("SONGBIRD_MAX_CONNECTIONS_DEFAULT", 1000)
}

/// Get small maximum connections
#[must_use]
pub fn max_connections_small() -> usize {
    SafeEnv::get_usize("SONGBIRD_MAX_CONNECTIONS_SMALL", 10)
}

/// Get medium maximum connections
#[must_use]
pub fn max_connections_medium() -> usize {
    SafeEnv::get_usize("SONGBIRD_MAX_CONNECTIONS_MEDIUM", 100)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_pool_defaults() {
        assert!(thread_pool_size() >= 1); // At least 1 CPU
        assert_eq!(thread_pool_size_min(), 2);
        assert_eq!(thread_pool_size_small(), 4);
        assert_eq!(thread_pool_size_large(), 10);
    }

    #[test]
    fn test_connection_defaults() {
        assert_eq!(max_connections_default(), 1000);
        assert_eq!(max_connections_small(), 10);
        assert_eq!(max_connections_medium(), 100);
    }
}
