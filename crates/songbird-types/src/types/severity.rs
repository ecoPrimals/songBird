//! Canonical severity level definitions
//!
//! **CANONICAL**: Single source of truth for error and warning severity levels
//! Used across validation, error handling, and AI-first response systems.

use serde::{Deserialize, Serialize};

/// Canonical error severity levels
///
/// Used for classification and prioritization of errors across the system.
/// Higher severity levels indicate more critical issues requiring immediate attention.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ErrorSeverity {
    /// Informational - no action required
    Info,
    /// Low severity - minor issues
    Low,
    /// Medium severity - notable issues that should be addressed
    Medium,
    /// High severity - significant issues requiring prompt attention
    High,
    /// Critical severity - system-critical issues requiring immediate action
    Critical,
}

impl Default for ErrorSeverity {
    fn default() -> Self {
        Self::Medium
    }
}

impl ErrorSeverity {
    /// Check if this severity level is critical
    pub const fn is_critical(&self) -> bool {
        matches!(self, Self::Critical)
    }

    /// Check if this severity requires immediate attention (High or Critical)
    pub const fn requires_immediate_attention(&self) -> bool {
        matches!(self, Self::High | Self::Critical)
    }

    /// Get numeric priority (higher number = higher severity)
    pub const fn priority(&self) -> u8 {
        match self {
            Self::Info => 1,
            Self::Low => 2,
            Self::Medium => 3,
            Self::High => 4,
            Self::Critical => 5,
        }
    }
}

/// Canonical warning severity levels
///
/// Used for classification of warnings that don't represent errors but require attention.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WarningSeverity {
    /// Low severity warning
    Low,
    /// Medium severity warning
    Medium,
    /// High severity warning
    High,
}

impl Default for WarningSeverity {
    fn default() -> Self {
        Self::Medium
    }
}

impl WarningSeverity {
    /// Get numeric priority (higher number = higher severity)
    pub const fn priority(&self) -> u8 {
        match self {
            Self::Low => 1,
            Self::Medium => 2,
            Self::High => 3,
        }
    }
}

#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::field_reassign_with_default
)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_severity_ordering() {
        assert!(ErrorSeverity::Critical > ErrorSeverity::High);
        assert!(ErrorSeverity::High > ErrorSeverity::Medium);
        assert!(ErrorSeverity::Medium > ErrorSeverity::Low);
        assert!(ErrorSeverity::Low > ErrorSeverity::Info);
    }

    #[test]
    fn test_error_severity_priority() {
        assert_eq!(ErrorSeverity::Critical.priority(), 5);
        assert_eq!(ErrorSeverity::High.priority(), 4);
        assert_eq!(ErrorSeverity::Medium.priority(), 3);
        assert_eq!(ErrorSeverity::Low.priority(), 2);
        assert_eq!(ErrorSeverity::Info.priority(), 1);
    }

    #[test]
    fn test_error_severity_is_critical() {
        assert!(ErrorSeverity::Critical.is_critical());
        assert!(!ErrorSeverity::High.is_critical());
    }

    #[test]
    fn test_error_severity_requires_immediate_attention() {
        assert!(ErrorSeverity::Critical.requires_immediate_attention());
        assert!(ErrorSeverity::High.requires_immediate_attention());
        assert!(!ErrorSeverity::Medium.requires_immediate_attention());
    }

    #[test]
    fn test_warning_severity_ordering() {
        assert!(WarningSeverity::High > WarningSeverity::Medium);
        assert!(WarningSeverity::Medium > WarningSeverity::Low);
    }
}
