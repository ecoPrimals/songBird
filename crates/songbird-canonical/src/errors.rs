// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Canonical error types for the Songbird ecosystem

use songbird_types::SongbirdError;

#[cfg(test)]
#[path = "errors_tests.rs"]
mod errors_tests;

/// Canonical error context wrapper
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// The error message
    pub message: String,
    /// Additional context about the error
    pub context: String,
    /// Suggested recovery actions
    pub recovery_suggestions: Vec<String>,
}

impl ErrorContext {
    /// Create a new error context
    pub fn new(message: impl Into<String>, context: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            context: context.into(),
            recovery_suggestions: Vec::new(),
        }
    }

    /// Add a recovery suggestion
    #[must_use]
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.recovery_suggestions.push(suggestion.into());
        self
    }

    /// Add multiple recovery suggestions
    #[must_use]
    pub fn with_suggestions<I, S>(mut self, suggestions: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.recovery_suggestions.extend(suggestions.into_iter().map(Into::into));
        self
    }

    /// Get the message
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Get the context
    #[must_use]
    pub fn context(&self) -> &str {
        &self.context
    }

    /// Get recovery suggestions
    #[must_use]
    pub fn suggestions(&self) -> &[String] {
        &self.recovery_suggestions
    }
}

impl std::fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (Context: {})", self.message, self.context)?;
        if !self.recovery_suggestions.is_empty() {
            write!(f, " Suggestions: {}", self.recovery_suggestions.join(", "))?;
        }
        Ok(())
    }
}

/// Helper function to create successful results
///
/// # Errors
///
/// This function never returns an error - it always creates a successful result.
pub const fn success_result<T>(data: T) -> T {
    data
}

/// Create a successful unit result
///
/// # Errors
/// This function never returns an error - it always succeeds with `Ok(()`
pub fn unit_success() -> Result<(), SongbirdError> {
    Ok(())
}
