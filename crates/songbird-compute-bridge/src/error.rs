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
