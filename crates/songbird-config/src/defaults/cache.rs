// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Cache configuration defaults with environment variable support
//!
//! Provides configurable cache settings via environment variables.
//!
//! # Environment Variables
//!
//! - `SONGBIRD_CACHE_MAX_ENTRIES` - Maximum cache entries (default: varies by use case)
//! - `SONGBIRD_CACHE_MAX_ENTRIES_SMALL` - Small cache max entries (default: 100)
//! - `SONGBIRD_CACHE_MAX_ENTRIES_MEDIUM` - Medium cache max entries (default: 1000)
//! - `SONGBIRD_CACHE_MAX_ENTRIES_LARGE` - Large cache max entries (default: 10000)
//! - `SONGBIRD_CACHE_MAX_ENTRIES_XLARGE` - Extra large cache max entries (default: 100000)
//! - `SONGBIRD_CACHE_POOL_SIZE` - Connection/resource pool size (default: 100)
//! - `SONGBIRD_BENCHMARK_ITERATIONS_SMALL` - Small benchmark iterations (default: 100)
//! - `SONGBIRD_BENCHMARK_ITERATIONS_LARGE` - Large benchmark iterations (default: 1000)

use songbird_types::SafeEnv;

/// Get default maximum cache entries (standard size)
#[must_use]
pub fn max_entries_default() -> usize {
    SafeEnv::get_usize("SONGBIRD_CACHE_MAX_ENTRIES", 10_000)
}

/// Get maximum cache entries for small caches
#[must_use]
pub fn max_entries_small() -> usize {
    SafeEnv::get_usize("SONGBIRD_CACHE_MAX_ENTRIES_SMALL", 100)
}

/// Get maximum cache entries for medium caches
#[must_use]
pub fn max_entries_medium() -> usize {
    SafeEnv::get_usize("SONGBIRD_CACHE_MAX_ENTRIES_MEDIUM", 1_000)
}

/// Get maximum cache entries for large caches
#[must_use]
pub fn max_entries_large() -> usize {
    SafeEnv::get_usize("SONGBIRD_CACHE_MAX_ENTRIES_LARGE", 10_000)
}

/// Get maximum cache entries for extra large caches
#[must_use]
pub fn max_entries_xlarge() -> usize {
    SafeEnv::get_usize("SONGBIRD_CACHE_MAX_ENTRIES_XLARGE", 100_000)
}

/// Get default pool size (connections, resources, etc.)
#[must_use]
pub fn pool_size_default() -> usize {
    SafeEnv::get_usize("SONGBIRD_CACHE_POOL_SIZE", 100)
}

/// Get benchmark iterations for small/quick benchmarks
#[must_use]
pub fn benchmark_iterations_small() -> usize {
    SafeEnv::get_usize("SONGBIRD_BENCHMARK_ITERATIONS_SMALL", 100)
}

/// Get benchmark iterations for large/thorough benchmarks
#[must_use]
pub fn benchmark_iterations_large() -> usize {
    SafeEnv::get_usize("SONGBIRD_BENCHMARK_ITERATIONS_LARGE", 1_000)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_entries_defaults() {
        assert_eq!(max_entries_small(), 100);
        assert_eq!(max_entries_medium(), 1_000);
        assert_eq!(max_entries_large(), 10_000);
        assert_eq!(max_entries_xlarge(), 100_000);
    }

    #[test]
    fn test_pool_size_default() {
        assert_eq!(pool_size_default(), 100);
    }

    #[test]
    fn test_benchmark_iterations() {
        assert_eq!(benchmark_iterations_small(), 100);
        assert_eq!(benchmark_iterations_large(), 1_000);
    }
}
