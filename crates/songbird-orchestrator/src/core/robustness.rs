// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! # 🛡️ Robustness & Resilience
//!
//! **MODERN ROBUSTNESS PATTERNS** ✅

use serde::{Deserialize, Serialize};

/// Circuit breaker
#[derive(Debug)]
pub struct CircuitBreaker;

/// Retry policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub backoff_ms: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            backoff_ms: 1000,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn retry_policy_default_matches_specified_values() {
        let p = RetryPolicy::default();
        assert_eq!(p.max_retries, 3);
        assert_eq!(p.backoff_ms, 1000);
    }

    #[test]
    fn retry_policy_serde_roundtrip() {
        let p = RetryPolicy {
            max_retries: 7,
            backoff_ms: 250,
        };
        let j = serde_json::to_string(&p).unwrap();
        let back: RetryPolicy = serde_json::from_str(&j).unwrap();
        assert_eq!(back.max_retries, p.max_retries);
        assert_eq!(back.backoff_ms, p.backoff_ms);
    }

    #[test]
    fn retry_policy_clone_preserves_fields() {
        let a = RetryPolicy {
            max_retries: 1,
            backoff_ms: 99,
        };
        let b = a.clone();
        assert_eq!(a.max_retries, b.max_retries);
        assert_eq!(a.backoff_ms, b.backoff_ms);
    }

    #[test]
    fn circuit_breaker_type_name_in_debug() {
        let cb = CircuitBreaker;
        let s = format!("{cb:?}");
        assert!(s.contains("CircuitBreaker"), "{s}");
    }
}
