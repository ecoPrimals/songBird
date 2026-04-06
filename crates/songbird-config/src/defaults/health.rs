// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Health check and circuit breaker configuration defaults
//!
//! Provides configurable health and resilience settings via environment variables.
//!
//! # Environment Variables
//!
//! - `SONGBIRD_HEALTH_FAILURE_THRESHOLD` - Consecutive failures before unhealthy (default: 3)
//! - `SONGBIRD_HEALTH_FAILURE_THRESHOLD_HIGH` - Higher failure threshold (default: 5)
//! - `SONGBIRD_HEALTH_SUCCESS_THRESHOLD` - Consecutive successes to recover (default: 2)
//! - `SONGBIRD_HEALTH_SUCCESS_THRESHOLD_HIGH` - Higher success threshold (default: 3)

use songbird_types::SafeEnv;

/// Get default failure threshold (circuit breaker/health checks)
#[must_use]
pub fn failure_threshold() -> u32 {
    SafeEnv::parse("SONGBIRD_HEALTH_FAILURE_THRESHOLD", 3)
}

/// Get higher failure threshold (for less sensitive checks)
#[must_use]
pub fn failure_threshold_high() -> u32 {
    SafeEnv::parse("SONGBIRD_HEALTH_FAILURE_THRESHOLD_HIGH", 5)
}

/// Get default success threshold (recovery from circuit breaker/unhealthy state)
#[must_use]
pub fn success_threshold() -> u32 {
    SafeEnv::parse("SONGBIRD_HEALTH_SUCCESS_THRESHOLD", 2)
}

/// Get higher success threshold (for stricter recovery requirements)
#[must_use]
pub fn success_threshold_high() -> u32 {
    SafeEnv::parse("SONGBIRD_HEALTH_SUCCESS_THRESHOLD_HIGH", 3)
}

/// Get maximum retry attempts for health checks
#[must_use]
pub fn max_retries() -> usize {
    SafeEnv::get_usize("SONGBIRD_HEALTH_MAX_RETRIES", 3)
}

/// Get expected HTTP status code for healthy response
#[must_use]
pub fn expected_status() -> u16 {
    SafeEnv::get_port("SONGBIRD_HEALTH_EXPECTED_STATUS", 200)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_failure_threshold_defaults() {
        assert_eq!(failure_threshold(), 3);
        assert_eq!(failure_threshold_high(), 5);
    }

    #[test]
    fn test_success_threshold_defaults() {
        assert_eq!(success_threshold(), 2);
        assert_eq!(success_threshold_high(), 3);
    }

    #[test]
    fn test_max_retries() {
        assert_eq!(max_retries(), 3);
    }

    #[test]
    fn test_expected_status() {
        assert_eq!(expected_status(), 200);
    }
}
