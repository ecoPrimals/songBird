// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Connection timeouts, default intervals, and runtime resource limits (workers, buffers, I/O).

use std::time::Duration;

use super::{env_get_bool_with, env_parse_with, read_process_env};

/// Get connection timeout from environment or calculate based on network conditions
#[must_use]
pub fn get_connection_timeout_ms() -> u64 {
    get_connection_timeout_ms_with(&read_process_env)
}

/// Same as [`get_connection_timeout_ms`] with an injectable env reader.
#[must_use]
pub fn get_connection_timeout_ms_with(
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
) -> u64 {
    env_parse_with(env, "SONGBIRD_CONNECTION_TIMEOUT_MS", {
        match env("SONGBIRD_ENV").as_deref() {
            Ok("production") => 30000,  // 30 seconds for production
            Ok("staging") => 45000,     // 45 seconds for staging
            Ok("development") => 60000, // 60 seconds for development
            _ => calculate_network_based_timeout_with(env),
        }
    })
}

/// Calculate timeout based on detected network conditions
fn calculate_network_based_timeout_with(
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
) -> u64 {
    // Check if we're in a cloud/container environment
    if env("KUBERNETES_SERVICE_HOST").is_ok()
        || env("AWS_EXECUTION_ENV").is_ok()
        || env("GOOGLE_CLOUD_PROJECT").is_ok()
        || env("AZURE_CLIENT_ID").is_ok()
    {
        15000 // Fast cloud networks
    } else {
        30000 // Conservative default for unknown networks
    }
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
    get_max_connections_with(&read_process_env)
}

/// Same as [`get_max_connections`] with an injectable env reader.
#[must_use]
pub fn get_max_connections_with(
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
) -> usize {
    env_parse_with(env, "SONGBIRD_MAX_CONNECTIONS", {
        match env("SONGBIRD_ENV").as_deref() {
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
    get_worker_threads_with(&read_process_env)
}

/// Same as [`get_worker_threads`] with an injectable env reader.
#[must_use]
pub fn get_worker_threads_with(env: &impl Fn(&str) -> Result<String, std::env::VarError>) -> usize {
    env_parse_with(env, "SONGBIRD_WORKER_THREADS", {
        // Use CPU count or container limits
        // Fallback to 4 threads
        std::thread::available_parallelism().map(std::num::NonZero::get).unwrap_or(4)
    })
}

/// Get buffer pool size based on available memory
#[must_use]
pub fn get_buffer_pool_size() -> usize {
    get_buffer_pool_size_with(&read_process_env)
}

/// Same as [`get_buffer_pool_size`] with an injectable env reader.
#[must_use]
pub fn get_buffer_pool_size_with(
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
) -> usize {
    env_parse_with(env, "SONGBIRD_BUFFER_POOL_SIZE", {
        // Calculate based on available memory
        let base_size = match env("SONGBIRD_ENV").as_deref() {
            Ok("production") => 10000,
            Ok("staging") => 5000,
            Ok("development") => 1000,
            _ => 2000,
        };

        // Adjust for container memory limits
        env("MEMORY_LIMIT").ok().and_then(|memory_limit| memory_limit.parse::<u64>().ok()).map_or(
            base_size,
            |limit_mb| {
                // Use 1% of available memory for buffer pool
                #[allow(
                    clippy::cast_possible_truncation,
                    reason = "MEMORY_LIMIT parsed as u64; product scaled down for pool sizing"
                )]
                let adjusted_size = (limit_mb as usize * 10) / 1024;
                std::cmp::min(base_size, adjusted_size)
            },
        )
    })
}

/// Get batch processing size based on workload characteristics
#[must_use]
pub fn get_batch_size() -> usize {
    get_batch_size_with(&read_process_env)
}

/// Same as [`get_batch_size`] with an injectable env reader.
#[must_use]
pub fn get_batch_size_with(env: &impl Fn(&str) -> Result<String, std::env::VarError>) -> usize {
    env_parse_with(env, "SONGBIRD_BATCH_SIZE", {
        // Calculate optimal batch size based on system characteristics
        let cpu_count = get_worker_threads_with(env);
        let memory_factor = if env("MEMORY_LIMIT").is_ok() {
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
    enable_zero_copy_with(&read_process_env)
}

/// Same as [`enable_zero_copy`] with an injectable env reader.
#[must_use]
pub fn enable_zero_copy_with(env: &impl Fn(&str) -> Result<String, std::env::VarError>) -> bool {
    env_get_bool_with(env, "SONGBIRD_ENABLE_ZERO_COPY", {
        // Enable zero-copy in production and for high-performance environments
        match env("SONGBIRD_ENV").as_deref() {
            Ok("production" | "staging") => true,
            _ => {
                // Enable if system has sufficient memory
                env("MEMORY_LIMIT")
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

    #[test]
    fn default_durations_match_documented_policy() {
        assert_eq!(DEFAULT_CACHE_TTL, std::time::Duration::from_secs(300));
        assert_eq!(DEFAULT_EVALUATION_TIMEOUT, std::time::Duration::from_secs(30));
        assert_eq!(DEFAULT_METRICS_INTERVAL, std::time::Duration::from_secs(60));
    }

    #[test]
    fn get_connection_timeout_ms_with_tiered_defaults() {
        let ms = get_connection_timeout_ms_with(&|k| match k {
            "SONGBIRD_ENV" => Ok("staging".to_string()),
            _ => Err(std::env::VarError::NotPresent),
        });
        assert_eq!(ms, 45_000);
    }

    #[test]
    fn get_connection_timeout_ms_with_cloud_fallback() {
        let ms = get_connection_timeout_ms_with(&|k| match k {
            "KUBERNETES_SERVICE_HOST" => Ok("10.0.0.1".to_string()),
            _ => Err(std::env::VarError::NotPresent),
        });
        assert_eq!(ms, 15_000);
    }

    #[test]
    fn get_max_connections_with_respects_environment_tier() {
        let n = get_max_connections_with(&|k| match k {
            "SONGBIRD_ENV" => Ok("testing".to_string()),
            _ => Err(std::env::VarError::NotPresent),
        });
        assert_eq!(n, 1_000);
    }

    #[test]
    fn get_worker_threads_with_parses_override() {
        let n = get_worker_threads_with(&|k| {
            if k == "SONGBIRD_WORKER_THREADS" {
                Ok("8".to_string())
            } else {
                Err(std::env::VarError::NotPresent)
            }
        });
        assert_eq!(n, 8);
    }

    #[test]
    fn get_buffer_pool_size_with_caps_against_memory_limit() {
        let n = get_buffer_pool_size_with(&|k| match k {
            "SONGBIRD_ENV" => Ok("production".to_string()),
            "MEMORY_LIMIT" => Ok("2048".to_string()),
            _ => Err(std::env::VarError::NotPresent),
        });
        assert_eq!(n, 20);
    }

    #[test]
    fn get_batch_size_with_clamps_to_bounds() {
        let n = get_batch_size_with(&|k| match k {
            "SONGBIRD_WORKER_THREADS" => Ok("1000".to_string()),
            _ => Err(std::env::VarError::NotPresent),
        });
        assert_eq!(n, 5000);
    }

    #[test]
    fn enable_zero_copy_with_reads_explicit_false() {
        let b = enable_zero_copy_with(&|k| {
            if k == "SONGBIRD_ENABLE_ZERO_COPY" {
                Ok("false".to_string())
            } else {
                Err(std::env::VarError::NotPresent)
            }
        });
        assert!(!b);
    }

    #[test]
    fn enable_zero_copy_with_production_default_true() {
        let b = enable_zero_copy_with(&|k| match k {
            "SONGBIRD_ENV" => Ok("production".to_string()),
            _ => Err(std::env::VarError::NotPresent),
        });
        assert!(b);
    }
}
