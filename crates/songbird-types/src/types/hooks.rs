// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Canonical hook error handling definitions
//!
//! **CANONICAL**: Single source of truth for hook error handling strategies
//! Used across hook systems for consistent error handling behavior.

use serde::{Deserialize, Serialize};

/// Canonical hook error handling strategy
///
/// Defines how the hook system should behave when a hook encounters an error.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub enum HookErrorHandling {
    /// Continue executing subsequent hooks even if one fails
    #[default]
    Continue,
    /// Stop execution on the first hook error
    StopOnError,
    /// Retry failed hooks according to retry configuration
    RetryOnError,
    /// Skip failed hooks and continue with remaining hooks
    SkipOnError,
}

impl HookErrorHandling {
    /// Check if this strategy allows continuing after errors
    #[must_use]
    pub const fn allows_continue(&self) -> bool {
        matches!(self, Self::Continue | Self::SkipOnError)
    }

    /// Check if this strategy stops on errors
    #[must_use]
    pub const fn stops_on_error(&self) -> bool {
        matches!(self, Self::StopOnError)
    }

    /// Check if this strategy retries errors
    #[must_use]
    pub const fn retries_on_error(&self) -> bool {
        matches!(self, Self::RetryOnError)
    }
}

#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::field_reassign_with_default,
    reason = "intentional pattern; clippy false positive for this API"
)]
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
