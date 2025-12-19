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
    pub fn new(total: usize) -> Self {
        Self {
            successes: Vec::new(),
            failures: Vec::new(),
            total,
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        self.successes.len() as f64 / self.total as f64
    }

    pub fn is_complete_success(&self) -> bool {
        self.failures.is_empty()
    }

    pub fn is_complete_failure(&self) -> bool {
        self.successes.is_empty()
    }

    pub fn is_partial_success(&self) -> bool {
        !self.successes.is_empty() && !self.failures.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_result() {
        let mut result = BatchResult::<i32>::new(10);

        result.successes.push(1);
        result.successes.push(2);
        result.failures.push((3, "Error".into()));

        assert_eq!(result.success_rate(), 0.2); // 2/10
        assert!(result.is_partial_success());
        assert!(!result.is_complete_success());
        assert!(!result.is_complete_failure());
    }
}
