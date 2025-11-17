//! Canonical hook error handling definitions
//!
//! **CANONICAL**: Single source of truth for hook error handling strategies
//! Used across hook systems for consistent error handling behavior.

use serde::{Deserialize, Serialize};

/// Canonical hook error handling strategy
///
/// Defines how the hook system should behave when a hook encounters an error.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum HookErrorHandling {
    /// Continue executing subsequent hooks even if one fails
    Continue,
    /// Stop execution on the first hook error
    StopOnError,
    /// Retry failed hooks according to retry configuration
    RetryOnError,
    /// Skip failed hooks and continue with remaining hooks
    SkipOnError,
}

impl Default for HookErrorHandling {
    fn default() -> Self {
        Self::Continue
    }
}

impl HookErrorHandling {
    /// Check if this strategy allows continuing after errors
    pub const fn allows_continue(&self) -> bool {
        matches!(self, Self::Continue | Self::SkipOnError)
    }

    /// Check if this strategy stops on errors
    pub const fn stops_on_error(&self) -> bool {
        matches!(self, Self::StopOnError)
    }

    /// Check if this strategy retries errors
    pub const fn retries_on_error(&self) -> bool {
        matches!(self, Self::RetryOnError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_error_handling_default() {
        assert_eq!(HookErrorHandling::default(), HookErrorHandling::Continue);
    }

    #[test]
    fn test_allows_continue() {
        assert!(HookErrorHandling::Continue.allows_continue());
        assert!(HookErrorHandling::SkipOnError.allows_continue());
        assert!(!HookErrorHandling::StopOnError.allows_continue());
        assert!(!HookErrorHandling::RetryOnError.allows_continue());
    }

    #[test]
    fn test_stops_on_error() {
        assert!(HookErrorHandling::StopOnError.stops_on_error());
        assert!(!HookErrorHandling::Continue.stops_on_error());
    }

    #[test]
    fn test_retries_on_error() {
        assert!(HookErrorHandling::RetryOnError.retries_on_error());
        assert!(!HookErrorHandling::Continue.retries_on_error());
    }
}
