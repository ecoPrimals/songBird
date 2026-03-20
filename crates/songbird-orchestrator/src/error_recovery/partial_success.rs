// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Partial Success Handling
//!
//! Handle operations that partially succeed with some failures.

use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Result of a batch operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult<T> {
    pub successes: Vec<T>,
    pub failures: Vec<(usize, Arc<str>)>, // (index, error message)
    pub total: usize,
}

impl<T> BatchResult<T> {
    #[must_use]
    pub const fn new(total: usize) -> Self {
        Self {
            successes: Vec::new(),
            failures: Vec::new(),
            total,
        }
    }

    #[must_use]
    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        self.successes.len() as f64 / self.total as f64
    }

    #[must_use]
    pub const fn is_complete_success(&self) -> bool {
        self.failures.is_empty()
    }

    #[must_use]
    pub const fn is_complete_failure(&self) -> bool {
        self.successes.is_empty()
    }

    #[must_use]
    pub const fn is_partial_success(&self) -> bool {
        !self.successes.is_empty() && !self.failures.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_result_partial() {
        let mut result = BatchResult::<i32>::new(10);

        result.successes.push(1);
        result.successes.push(2);
        result.failures.push((3, "Error".into()));

        assert_eq!(result.success_rate(), 0.2); // 2/10
        assert!(result.is_partial_success());
        assert!(!result.is_complete_success());
        assert!(!result.is_complete_failure());
    }

    #[test]
    fn test_batch_result_complete_success() {
        let mut result = BatchResult::<i32>::new(5);

        result.successes.push(1);
        result.successes.push(2);
        result.successes.push(3);
        result.successes.push(4);
        result.successes.push(5);

        assert_eq!(result.success_rate(), 1.0);
        assert!(result.is_complete_success());
        assert!(!result.is_complete_failure());
        assert!(!result.is_partial_success());
    }

    #[test]
    fn test_batch_result_complete_failure() {
        let mut result = BatchResult::<i32>::new(3);

        result.failures.push((0, "Error 1".into()));
        result.failures.push((1, "Error 2".into()));
        result.failures.push((2, "Error 3".into()));

        assert_eq!(result.success_rate(), 0.0);
        assert!(result.is_complete_failure());
        assert!(!result.is_complete_success());
        assert!(!result.is_partial_success());
    }

    #[test]
    fn test_batch_result_empty() {
        let result = BatchResult::<i32>::new(0);

        assert_eq!(result.success_rate(), 0.0);
        assert!(result.is_complete_success()); // No failures
        assert!(result.is_complete_failure()); // No successes
        assert!(!result.is_partial_success());
    }

    #[test]
    fn test_batch_result_clone() {
        let mut result = BatchResult::<i32>::new(2);
        result.successes.push(42);
        result.failures.push((1, "Failed".into()));

        let cloned = result.clone();
        assert_eq!(cloned.successes.len(), result.successes.len());
        assert_eq!(cloned.failures.len(), result.failures.len());
        assert_eq!(cloned.total, result.total);
    }

    #[test]
    fn test_batch_result_success_rate_precision() {
        let mut result = BatchResult::<String>::new(3);
        result.successes.push("a".to_string());

        let rate = result.success_rate();
        assert!((rate - 0.333_333_333).abs() < 0.001);
    }

    #[test]
    fn test_batch_result_new_initialized() {
        let result = BatchResult::<u64>::new(100);

        assert!(result.successes.is_empty());
        assert!(result.failures.is_empty());
        assert_eq!(result.total, 100);
    }
}
