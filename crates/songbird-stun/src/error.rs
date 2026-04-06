// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! STUN error types

use std::net::AddrParseError;
use thiserror::Error;

/// STUN result type
pub type StunResult<T> = Result<T, StunError>;

/// STUN error types
#[derive(Debug, Error)]
pub enum StunError {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Address parse error
    #[error("Address parse error: {0}")]
    AddrParse(#[from] AddrParseError),

    /// Timeout error
    #[error("STUN request timeout after {0:?}")]
    Timeout(std::time::Duration),

    /// Invalid STUN response
    #[error("Invalid STUN response: {0}")]
    InvalidResponse(String),

    /// STUN server error
    #[error("STUN server error: {0}")]
    ServerError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Network error
    #[error("Network error: {0}")]
    Network(String),

    /// All STUN servers failed during concurrent racing
    #[error("All STUN servers failed: {0}")]
    AllServersFailed(String),
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::StunError;
    use std::io;

    #[test]
    fn io_error_from_displays() {
        let e: StunError = io::Error::new(io::ErrorKind::NotFound, "missing").into();
        assert!(e.to_string().contains("IO error"));
        assert!(e.to_string().contains("missing"));
    }

    #[test]
    fn addr_parse_from_displays() {
        let inner: std::net::AddrParseError = "not-an-ip".parse::<std::net::IpAddr>().unwrap_err();
        let e: StunError = inner.into();
        assert!(e.to_string().contains("Address parse"));
    }

    #[test]
    fn timeout_displays() {
        let e = StunError::Timeout(std::time::Duration::from_secs(5));
        assert!(e.to_string().contains("timeout"));
        assert!(e.to_string().contains("5s") || e.to_string().contains('5'));
    }

    #[test]
    fn invalid_response_displays() {
        let e = StunError::InvalidResponse("bad".to_string());
        assert!(e.to_string().contains("Invalid STUN response"));
        assert!(e.to_string().contains("bad"));
    }

    #[test]
    fn server_error_displays() {
        let e = StunError::ServerError("code 500".to_string());
        assert!(e.to_string().contains("STUN server error"));
    }

    #[test]
    fn config_network_all_servers_displays() {
        assert!(StunError::Config("x".into()).to_string().contains("Configuration"));
        assert!(StunError::Network("n".into()).to_string().contains("Network"));
        assert!(
            StunError::AllServersFailed("all".into())
                .to_string()
                .contains("All STUN servers failed")
        );
    }

    #[test]
    fn stun_error_debug_is_nonempty() {
        let e = StunError::InvalidResponse("x".into());
        assert!(
            format!("{e:?}").contains("InvalidResponse"),
            "Debug should mention variant: {e:?}"
        );
    }
}
