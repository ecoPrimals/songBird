// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Error types for universal IPC

use std::io;
use thiserror::Error;

/// Universal IPC errors
#[derive(Debug, Error)]
pub enum IpcError {
    /// Service not found in registry
    #[error("Service not found: {0}")]
    ServiceNotFound(String),

    /// Service already registered
    #[error("Service already registered: {0}")]
    ServiceAlreadyRegistered(String),

    /// Invalid virtual path format
    #[error("Invalid virtual path: {0}")]
    InvalidVirtualPath(String),

    /// Platform-specific error
    #[error("Platform error: {0}")]
    PlatformError(String),

    /// I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),

    /// Connection failed
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    /// Listener creation failed
    #[error("Listener creation failed: {0}")]
    ListenerFailed(String),

    /// Endpoint cleanup failed
    #[error("Endpoint cleanup failed: {0}")]
    CleanupFailed(String),

    /// Registry error
    #[error("Registry error: {0}")]
    RegistryError(String),

    /// Storage capability provider integration error (optional; enable with `storage_provider` or legacy `nestgate` feature).
    #[cfg(any(feature = "storage_provider", feature = "nestgate"))]
    #[error("Storage provider error: {0}")]
    StorageProviderError(String),

    /// RPC error
    #[error("RPC error: {0}")]
    RpcError(String),

    /// Invalid parameters
    #[error("Invalid parameters: {0}")]
    InvalidParams(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),

    /// Other error
    #[error("Other error: {0}")]
    Other(String),
}

/// Result type for universal IPC operations
pub type IpcResult<T> = Result<T, IpcError>;

#[cfg(any(feature = "storage_provider", feature = "nestgate"))]
impl IpcError {
    /// Deprecated constructor matching the former `NestGateError` variant.
    #[deprecated(note = "use IpcError::StorageProviderError(String)")]
    #[must_use]
    pub fn nest_gate_error(msg: impl Into<String>) -> Self {
        Self::StorageProviderError(msg.into())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::IpcError;
    use std::io;

    #[test]
    fn service_not_found_display() {
        let e = IpcError::ServiceNotFound("beardog".to_string());
        assert!(e.to_string().contains("beardog"));
    }

    #[test]
    fn service_already_registered_display() {
        let e = IpcError::ServiceAlreadyRegistered("/x".to_string());
        assert!(e.to_string().contains("/x"));
    }

    #[test]
    fn invalid_virtual_path_display() {
        let e = IpcError::InvalidVirtualPath("bad".to_string());
        assert!(e.to_string().contains("bad"));
    }

    #[test]
    fn io_error_from_std_io() {
        let inner = io::Error::new(io::ErrorKind::NotFound, "nope");
        let e: IpcError = inner.into();
        assert!(matches!(e, IpcError::IoError(_)));
    }

    #[test]
    fn registry_and_rpc_errors_roundtrip_string() {
        assert!(IpcError::RegistryError("r".to_string()).to_string().contains('r'));
        assert!(IpcError::RpcError("rpc".to_string()).to_string().contains("rpc"));
        assert!(IpcError::InvalidParams("p".to_string()).to_string().contains('p'));
        assert!(IpcError::Internal("i".to_string()).to_string().contains('i'));
        assert!(IpcError::Other("o".to_string()).to_string().contains('o'));
    }

    #[test]
    fn connection_and_listener_errors() {
        assert!(IpcError::ConnectionFailed("c".to_string()).to_string().contains('c'));
        assert!(IpcError::ListenerFailed("l".to_string()).to_string().contains('l'));
        assert!(IpcError::CleanupFailed("u".to_string()).to_string().contains('u'));
        assert!(IpcError::PlatformError("pl".to_string()).to_string().contains("pl"));
    }
}
