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
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

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

    #[test]
    fn success_rate_zero_total() {
        let r = BatchResult::<()>::new(0);
        assert_eq!(r.success_rate(), 0.0);
    }

    #[test]
    fn success_rate_all_failures() {
        let mut r = BatchResult::<i32>::new(5);
        r.failures.push((0, "a".into()));
        r.failures.push((1, "b".into()));
        assert_eq!(r.success_rate(), 0.0);
        assert!(r.is_complete_failure());
        assert!(!r.is_partial_success());
    }

    #[test]
    fn partial_success_requires_both_vectors_nonempty() {
        let mut r = BatchResult::<i32>::new(3);
        r.successes.push(1);
        r.failures.push((1, "e".into()));
        assert!(r.is_partial_success());
    }

    #[test]
    fn complete_success_empty_failure_list() {
        let mut r = BatchResult::<String>::new(1);
        r.successes.push("ok".to_string());
        assert!(r.is_complete_success());
        assert!(!r.is_complete_failure());
    }

    #[test]
    fn serde_roundtrip_batch_result() {
        let mut r = BatchResult::<i32>::new(4);
        r.successes.push(1);
        r.failures.push((2, "oops".into()));
        let json = serde_json::to_string(&r).unwrap();
        let back: BatchResult<i32> = serde_json::from_str(&json).unwrap();
        assert_eq!(back.total, 4);
        assert_eq!(back.successes, vec![1]);
        assert_eq!(back.failures.len(), 1);
    }

    #[test]
    fn failures_preserve_indices() {
        let mut r = BatchResult::<()>::new(10);
        r.failures.push((9, "last".into()));
        r.failures.push((0, "first".into()));
        assert_eq!(r.failures[0].0, 9);
        assert_eq!(r.failures[1].0, 0);
    }

    #[test]
    fn success_rate_one_success_large_total() {
        let mut r = BatchResult::<u8>::new(1_000_000);
        r.successes.push(1);
        let rate = r.success_rate();
        assert!((rate - 1e-6).abs() < 1e-12);
    }

    #[test]
    fn is_complete_success_true_when_no_failures_even_if_successes_empty() {
        let r = BatchResult::<i32>::new(5);
        assert!(r.is_complete_success());
    }

    #[test]
    fn total_can_exceed_success_plus_failure_counts() {
        let mut r = BatchResult::<i32>::new(100);
        r.successes.push(1);
        r.failures.push((0, "x".into()));
        assert_eq!(r.success_rate(), 0.01);
    }

    #[test]
    fn batch_result_with_string_type() {
        let mut r = BatchResult::<String>::new(2);
        r.successes.push("a".to_string());
        r.successes.push("b".to_string());
        assert_eq!(r.success_rate(), 1.0);
    }

    #[test]
    fn empty_successes_and_failures_with_positive_total() {
        let r = BatchResult::<i32>::new(99);
        assert!(r.is_complete_success());
        assert!(r.is_complete_failure());
        assert_eq!(r.success_rate(), 0.0);
    }

    #[test]
    fn clone_independent_mutation() {
        let mut a = BatchResult::<i32>::new(3);
        a.successes.push(1);
        let mut b = a.clone();
        b.successes.push(2);
        assert_eq!(a.successes.len(), 1);
        assert_eq!(b.successes.len(), 2);
    }

    #[test]
    fn debug_format_includes_fields() {
        let mut r = BatchResult::<i32>::new(2);
        r.successes.push(7);
        let s = format!("{r:?}");
        assert!(s.contains("successes") && s.contains("failures"));
    }

    #[test]
    fn failure_message_arc_str_shared() {
        let msg: Arc<str> = "shared".into();
        let mut r = BatchResult::<i32>::new(1);
        r.failures.push((0, msg.clone()));
        r.failures.push((1, msg));
        assert_eq!(r.failures[0].1.as_ref(), "shared");
    }

    #[test]
    fn success_rate_half() {
        let mut r = BatchResult::<i32>::new(8);
        for i in 0..4 {
            r.successes.push(i);
        }
        assert_eq!(r.success_rate(), 0.5);
        assert!(!r.is_partial_success());
        assert!(r.is_complete_success());
    }

    #[test]
    fn usize_max_total_success_rate() {
        let mut r = BatchResult::<i32>::new(usize::MAX);
        r.successes.push(1);
        let rate = r.success_rate();
        assert!(rate > 0.0 && rate < f64::INFINITY);
    }
}
