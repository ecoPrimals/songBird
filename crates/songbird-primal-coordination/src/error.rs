// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Error types for primal coordination

use thiserror::Error;

/// Result type for primal coordination operations
pub type Result<T> = std::result::Result<T, PrimalCoordinationError>;

/// Errors that can occur during primal coordination
#[derive(Debug, Error)]
pub enum PrimalCoordinationError {
    /// Primal or capability not found
    #[error("Not found: {0}")]
    NotFound(String),

    /// Failed to connect to a primal
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    /// Communication error with primal
    #[error("Communication error: {0}")]
    CommunicationError(String),

    /// Unexpected response from primal
    #[error("Unexpected response: {0}")]
    UnexpectedResponse(String),

    /// Error from primal
    #[error("Primal error: {0}")]
    PrimalError(String),

    /// No capable primal found for operation
    #[error("No capable primal found: {0}")]
    NoCapablePrimal(String),

    /// Internal coordination error
    #[error("Internal error: {0}")]
    Internal(String),

    /// Discovery error
    #[error("Discovery failed: {0}")]
    DiscoveryFailed(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic error
    #[error("Error: {0}")]
    Other(#[from] anyhow::Error),
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn display_not_found_and_connection() {
        assert!(PrimalCoordinationError::NotFound("x".into()).to_string().contains("Not found"));
        assert!(
            PrimalCoordinationError::ConnectionFailed("e".into())
                .to_string()
                .contains("Connection failed")
        );
    }

    #[test]
    fn display_no_capable_primal_and_internal() {
        assert!(
            PrimalCoordinationError::NoCapablePrimal("cap".into())
                .to_string()
                .contains("No capable primal"),
            "NoCapablePrimal should mention no capable primal"
        );
        assert!(
            PrimalCoordinationError::Internal("bug".into()).to_string().contains("Internal error"),
            "Internal should surface as internal error"
        );
    }

    #[test]
    fn display_communication_unexpected_primal_discovery() {
        assert!(
            PrimalCoordinationError::CommunicationError("c".into())
                .to_string()
                .contains("Communication error"),
            "CommunicationError display"
        );
        assert!(
            PrimalCoordinationError::UnexpectedResponse("u".into())
                .to_string()
                .contains("Unexpected response"),
            "UnexpectedResponse display"
        );
        assert!(
            PrimalCoordinationError::PrimalError("p".into()).to_string().contains("Primal error"),
            "PrimalError display"
        );
        assert!(
            PrimalCoordinationError::DiscoveryFailed("d".into())
                .to_string()
                .contains("Discovery failed"),
            "DiscoveryFailed display"
        );
    }

    #[test]
    fn anyhow_error_maps_to_other() {
        let e: PrimalCoordinationError = anyhow::anyhow!("wrapped").into();
        assert!(matches!(e, PrimalCoordinationError::Other(_)), "anyhow maps to Other");
        assert!(e.to_string().contains("wrapped"), "Underlying message preserved");
    }

    #[test]
    fn serde_json_error_maps_to_serialization() {
        let j = serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
        let e: PrimalCoordinationError = j.into();
        assert!(matches!(e, PrimalCoordinationError::Serialization(_)));
    }

    #[test]
    fn io_error_maps() {
        let io = std::io::Error::other("io");
        let e: PrimalCoordinationError = io.into();
        assert!(matches!(e, PrimalCoordinationError::Io(_)));
    }
}
