// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

/// **UNIFIED**: Error types for universal adapter operations
/// Errors that can occur during universal adapter operations
#[derive(Debug, thiserror::Error)]
pub enum UniversalAdapterError {
    /// Network communication error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Failed to parse response or configuration
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Service discovery failed
    #[error("Discovery error: {0}")]
    DiscoveryError(String),

    /// Service-level error
    #[error("Service error: {0}")]
    ServiceError(String),

    /// Required capability is missing
    #[error("Missing required capability")]
    MissingCapability,

    /// No providers available for the requested capability
    #[error("No providers available for capability: {0}")]
    NoProvidersAvailable(String),
}

// Convert UniversalAdapterError to SongbirdError for test compatibility
impl From<UniversalAdapterError> for songbird_types::SongbirdError {
    fn from(err: UniversalAdapterError) -> Self {
        use UniversalAdapterError::{
            DiscoveryError, MissingCapability, NetworkError, NoProvidersAvailable, ParseError,
            ServiceError,
        };
        match err {
            NetworkError(msg) | ParseError(msg) | DiscoveryError(msg) | ServiceError(msg) => {
                Self::from(msg)
            }
            MissingCapability => Self::from("Required capability is missing"),
            NoProvidersAvailable(cap) => {
                Self::from(format!("No providers available for capability: {cap}"))
            }
        }
    }
}
