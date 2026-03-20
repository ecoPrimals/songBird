// SPDX-License-Identifier: AGPL-3.0-only
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
