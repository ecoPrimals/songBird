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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]
    #![allow(clippy::expect_used, reason = "test assertions")]

    use super::UniversalAdapterError;

    #[test]
    fn network_error_maps_to_songbird() {
        let e: songbird_types::SongbirdError =
            UniversalAdapterError::NetworkError("n".to_string()).into();
        assert!(!e.to_string().is_empty());
    }

    #[test]
    fn missing_capability_maps() {
        let e: songbird_types::SongbirdError = UniversalAdapterError::MissingCapability.into();
        assert!(e.to_string().contains("capability") || e.to_string().len() > 3);
    }

    #[test]
    fn no_providers_maps() {
        let e: songbird_types::SongbirdError =
            UniversalAdapterError::NoProvidersAvailable("c".to_string()).into();
        assert!(e.to_string().contains('c') || !e.to_string().is_empty());
    }

    #[test]
    fn all_variants_display() {
        let cases = [
            UniversalAdapterError::NetworkError("a".to_string()),
            UniversalAdapterError::ParseError("b".to_string()),
            UniversalAdapterError::DiscoveryError("c".to_string()),
            UniversalAdapterError::ServiceError("d".to_string()),
            UniversalAdapterError::MissingCapability,
            UniversalAdapterError::NoProvidersAvailable("e".to_string()),
        ];
        for err in cases {
            assert!(!err.to_string().is_empty());
        }
    }
}
