// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Error types for compute bridge

use thiserror::Error;

/// Compute bridge errors
#[derive(Debug, Error)]
pub enum ComputeError {
    /// No compute provider available
    #[error("No compute provider available: {0}")]
    NoProviderAvailable(String),

    /// Deployment failed
    #[error("Deployment failed: {0}")]
    DeploymentFailed(String),

    /// Provider error
    #[error("Provider error: {0}")]
    ProviderError(String),

    /// Communication error
    #[error("Communication error: {0}")]
    CommunicationError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),

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
    use super::ComputeError;
    use std::error::Error as _;

    #[test]
    fn no_provider_available_display() {
        let e = ComputeError::NoProviderAvailable("missing".into());
        assert!(
            e.to_string().contains("No compute provider available"),
            "unexpected message: {}",
            e
        );
        assert!(e.to_string().contains("missing"));
    }

    #[test]
    fn deployment_failed_display() {
        let e = ComputeError::DeploymentFailed("bad deploy".into());
        assert!(e.to_string().contains("Deployment failed"));
        assert!(e.to_string().contains("bad deploy"));
    }

    #[test]
    fn provider_error_display() {
        let e = ComputeError::ProviderError("upstream".into());
        assert!(e.to_string().contains("Provider error"));
    }

    #[test]
    fn communication_error_display() {
        let e = ComputeError::CommunicationError("timeout".into());
        assert!(e.to_string().contains("Communication error"));
    }

    #[test]
    fn configuration_error_display() {
        let e = ComputeError::ConfigurationError("bad yaml".into());
        assert!(e.to_string().contains("Configuration error"));
    }

    #[test]
    fn internal_display() {
        let e = ComputeError::Internal("oops".into());
        assert!(e.to_string().contains("Internal error"));
    }

    #[test]
    fn io_error_from_std_io() {
        let io = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let e: ComputeError = io.into();
        assert!(matches!(e, ComputeError::Io(_)));
        assert!(e.to_string().contains("I/O error"));
        assert!(e.source().is_some(), "Io variant should preserve source");
    }

    #[test]
    fn other_from_anyhow() {
        let e: ComputeError = anyhow::anyhow!("wrapped").into();
        assert!(matches!(e, ComputeError::Other(_)));
        assert!(e.to_string().contains("wrapped"));
    }
}
