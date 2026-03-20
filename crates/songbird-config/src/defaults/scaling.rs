// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Scaling configuration defaults with environment variable support
//!
//! Provides configurable auto-scaling settings via environment variables.
//!
//! # Environment Variables
//!
//! - `SONGBIRD_MIN_INSTANCES` - Minimum service instances (default: 1)
//! - `SONGBIRD_MAX_INSTANCES` - Maximum service instances (default: 10)
//! - `SONGBIRD_MAX_INSTANCES_TEST` - Max instances for testing (default: 2)
//! - `SONGBIRD_MIN_INSTANCES_TEST` - Min instances for testing (default: 1)
//! - `SONGBIRD_SCALE_UP_THRESHOLD` - CPU/metric threshold to scale up (default: 70.0%)
//! - `SONGBIRD_SCALE_DOWN_THRESHOLD` - CPU/metric threshold to scale down (default: 30.0%)

use songbird_types::SafeEnv;

/// Get minimum instances for auto-scaling
#[must_use]
pub fn min_instances() -> u32 {
    SafeEnv::parse("SONGBIRD_SCALING_MIN_INSTANCES", 1)
}

/// Get maximum instances for auto-scaling
#[must_use]
pub fn max_instances() -> u32 {
    SafeEnv::parse("SONGBIRD_SCALING_MAX_INSTANCES", 10)
}

/// Get minimum instances for testing scenarios
#[must_use]
pub fn min_instances_test() -> usize {
    SafeEnv::get_usize("SONGBIRD_MIN_INSTANCES_TEST", 1)
}

/// Get maximum instances for testing scenarios
#[must_use]
pub fn max_instances_test() -> usize {
    SafeEnv::get_usize("SONGBIRD_MAX_INSTANCES_TEST", 2)
}

/// Get scale-up threshold percentage
#[must_use]
pub fn scale_up_threshold() -> f64 {
    SafeEnv::parse("SONGBIRD_SCALE_UP_THRESHOLD", 70.0)
}

/// Get scale-down threshold percentage
#[must_use]
pub fn scale_down_threshold() -> f64 {
    SafeEnv::parse("SONGBIRD_SCALE_DOWN_THRESHOLD", 30.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instance_defaults() {
        assert_eq!(min_instances(), 1);
        assert_eq!(max_instances(), 10);
        assert_eq!(min_instances_test(), 1);
        assert_eq!(max_instances_test(), 2);
    }

    #[test]
    fn test_threshold_defaults() {
        assert!((scale_up_threshold() - 70.0).abs() < 0.01);
        assert!((scale_down_threshold() - 30.0).abs() < 0.01);
    }
}
