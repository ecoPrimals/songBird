// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Connection timeouts, logging, pool sizing, and performance-related env helpers.

use songbird_types::error_helpers::SafeEnv;
use std::time::Duration;

/// Get connection timeout from environment or calculate based on network conditions
#[must_use]
pub fn get_connection_timeout_ms() -> u64 {
    SafeEnv::parse("SONGBIRD_CONNECTION_TIMEOUT_MS", {
        match SafeEnv::get("SONGBIRD_ENV").as_deref() {
            Ok("production") => 30000,  // 30 seconds for production
            Ok("staging") => 45000,     // 45 seconds for staging
            Ok("development") => 60000, // 60 seconds for development
            _ => calculate_network_based_timeout(),
        }
    })
}

/// Calculate timeout based on detected network conditions
fn calculate_network_based_timeout() -> u64 {
    // Check if we're in a cloud/container environment
    if SafeEnv::get("KUBERNETES_SERVICE_HOST").is_ok()
        || SafeEnv::get("AWS_EXECUTION_ENV").is_ok()
        || SafeEnv::get("GOOGLE_CLOUD_PROJECT").is_ok()
        || SafeEnv::get("AZURE_CLIENT_ID").is_ok()
    {
        15000 // Fast cloud networks
    } else {
        30000 // Conservative default for unknown networks
    }
}

/// Get log level from environment or default
#[must_use]
pub fn get_log_level() -> String {
    SafeEnv::get("SONGBIRD_LOG_LEVEL")
        .or_else(|_| SafeEnv::get("LOG_LEVEL"))
        .or_else(|_| SafeEnv::get("RUST_LOG"))
        .unwrap_or_else(|_| {
            match SafeEnv::get("SONGBIRD_ENV").as_deref() {
                Ok("production") => "warn".to_string(),
                Ok("staging") => "info".to_string(),
                _ => "debug".to_string(), // Testing and development default
            }
        })
}

/// Default cache TTL
pub const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(300); // 5 minutes

/// Default evaluation timeout
pub const DEFAULT_EVALUATION_TIMEOUT: Duration = Duration::from_secs(30);

/// Default metrics interval
pub const DEFAULT_METRICS_INTERVAL: Duration = Duration::from_secs(60);

/// Get maximum connections allowed
#[must_use]
pub fn get_max_connections() -> usize {
    SafeEnv::parse("SONGBIRD_MAX_CONNECTIONS", {
        match SafeEnv::get("SONGBIRD_ENV").as_deref() {
            Ok("production") => 10000,
            Ok("staging") => 5000,
            Ok("testing") => 1000,
            _ => 2000, // Development default
        }
    })
}

/// Get worker thread count based on system resources
#[must_use]
pub fn get_worker_threads() -> usize {
    SafeEnv::parse("SONGBIRD_WORKER_THREADS", {
        // Use CPU count or container limits
        // Fallback to 4 threads
        std::thread::available_parallelism().map(std::num::NonZero::get).unwrap_or(4)
    })
}

/// Get buffer pool size based on available memory
#[must_use]
pub fn get_buffer_pool_size() -> usize {
    SafeEnv::parse("SONGBIRD_BUFFER_POOL_SIZE", {
        // Calculate based on available memory
        let base_size = match SafeEnv::get("SONGBIRD_ENV").as_deref() {
            Ok("production") => 10000,
            Ok("staging") => 5000,
            Ok("development") => 1000,
            _ => 2000,
        };

        // Adjust for container memory limits
        SafeEnv::get("MEMORY_LIMIT")
            .ok()
            .and_then(|memory_limit| memory_limit.parse::<u64>().ok())
            .map_or(base_size, |limit_mb| {
                // Use 1% of available memory for buffer pool
                #[allow(
                    clippy::cast_possible_truncation,
                    reason = "MEMORY_LIMIT parsed as u64; product scaled down for pool sizing"
                )]
                let adjusted_size = (limit_mb as usize * 10) / 1024;
                std::cmp::min(base_size, adjusted_size)
            })
    })
}

/// Get batch processing size based on workload characteristics
#[must_use]
pub fn get_batch_size() -> usize {
    SafeEnv::parse("SONGBIRD_BATCH_SIZE", {
        // Calculate optimal batch size based on system characteristics
        let cpu_count = get_worker_threads();
        let memory_factor = if SafeEnv::get("MEMORY_LIMIT").is_ok() {
            500
        } else {
            1000
        };

        (cpu_count * memory_factor).clamp(100, 5000)
    })
}

/// Check if zero-copy optimizations should be enabled
#[must_use]
pub fn enable_zero_copy() -> bool {
    SafeEnv::get_bool("SONGBIRD_ENABLE_ZERO_COPY", {
        // Enable zero-copy in production and for high-performance environments
        match SafeEnv::get("SONGBIRD_ENV").as_deref() {
            Ok("production" | "staging") => true,
            _ => {
                // Enable if system has sufficient memory
                SafeEnv::get("MEMORY_LIMIT")
                    .ok()
                    .and_then(|s| s.parse::<u64>().ok())
                    .is_none_or(|mb| mb > 2048) // Default to enabled
            }
        }
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;
    use songbird_test_utils::ScopedEnv;

    #[test]
    fn duration_constants_are_positive() {
        assert!(DEFAULT_CACHE_TTL.as_secs() > 0);
        assert!(DEFAULT_EVALUATION_TIMEOUT.as_secs() > 0);
        assert!(DEFAULT_METRICS_INTERVAL.as_secs() > 0);
        assert_eq!(DEFAULT_CACHE_TTL.as_secs(), 300);
        assert_eq!(DEFAULT_EVALUATION_TIMEOUT.as_secs(), 30);
        assert_eq!(DEFAULT_METRICS_INTERVAL.as_secs(), 60);
    }

    #[tokio::test]
    async fn get_connection_timeout_ms_env_specific_defaults() {
        {
            let _e = ScopedEnv::remove_and_set_many(
                ["SONGBIRD_CONNECTION_TIMEOUT_MS", "KUBERNETES_SERVICE_HOST"],
                [("SONGBIRD_ENV", "production")],
            )
            .await;
            assert_eq!(get_connection_timeout_ms(), 30_000);
        }
        {
            let _e = ScopedEnv::remove_and_set_many(
                ["SONGBIRD_CONNECTION_TIMEOUT_MS", "KUBERNETES_SERVICE_HOST"],
                [("SONGBIRD_ENV", "staging")],
            )
            .await;
            assert_eq!(get_connection_timeout_ms(), 45_000);
        }
        {
            let _e = ScopedEnv::remove_and_set_many(
                ["SONGBIRD_CONNECTION_TIMEOUT_MS", "KUBERNETES_SERVICE_HOST"],
                [("SONGBIRD_ENV", "development")],
            )
            .await;
            assert_eq!(get_connection_timeout_ms(), 60_000);
        }
    }

    #[tokio::test]
    async fn get_connection_timeout_ms_cloud_default_when_unspecified_env() {
        let _e = ScopedEnv::remove_and_set_many(
            ["SONGBIRD_CONNECTION_TIMEOUT_MS", "SONGBIRD_ENV"],
            [("KUBERNETES_SERVICE_HOST", "10.0.0.1")],
        )
        .await;
        assert_eq!(get_connection_timeout_ms(), 15_000);
    }

    #[tokio::test]
    async fn get_max_connections_tiers_match_environment() {
        {
            let _e = ScopedEnv::remove_and_set_many(
                ["SONGBIRD_MAX_CONNECTIONS"],
                [("SONGBIRD_ENV", "production")],
            )
            .await;
            assert_eq!(get_max_connections(), 10_000);
        }
        {
            let _e = ScopedEnv::remove_and_set_many(
                ["SONGBIRD_MAX_CONNECTIONS"],
                [("SONGBIRD_ENV", "staging")],
            )
            .await;
            assert_eq!(get_max_connections(), 5_000);
        }
        {
            let _e = ScopedEnv::remove_and_set_many(
                ["SONGBIRD_MAX_CONNECTIONS"],
                [("SONGBIRD_ENV", "testing")],
            )
            .await;
            assert_eq!(get_max_connections(), 1_000);
        }
    }

    #[test]
    fn get_worker_threads_is_positive() {
        assert!(get_worker_threads() > 0);
    }

    #[test]
    fn get_batch_size_is_clamped() {
        let b = get_batch_size();
        assert!((100..=5000).contains(&b));
    }

    #[tokio::test]
    async fn get_log_level_reads_songbird_first() {
        let _e =
            ScopedEnv::set_multiple([("SONGBIRD_LOG_LEVEL", "trace"), ("LOG_LEVEL", "info")]).await;
        assert_eq!(get_log_level(), "trace");
    }

    #[tokio::test]
    async fn enable_zero_copy_true_in_production_by_default() {
        let _e = ScopedEnv::remove_and_set_many(
            ["SONGBIRD_ENABLE_ZERO_COPY", "MEMORY_LIMIT"],
            [("SONGBIRD_ENV", "production")],
        )
        .await;
        assert!(enable_zero_copy());
    }

    #[tokio::test]
    async fn get_buffer_pool_size_respects_memory_limit_cap() {
        let _e = ScopedEnv::remove_and_set_many(
            ["SONGBIRD_BUFFER_POOL_SIZE"],
            [("SONGBIRD_ENV", "production"), ("MEMORY_LIMIT", "2048")],
        )
        .await;
        let size = get_buffer_pool_size();
        assert_eq!(size, 20);
    }
}
