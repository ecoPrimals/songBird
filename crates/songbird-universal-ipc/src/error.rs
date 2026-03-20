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

    /// `NestGate` integration error (optional feature)
    #[cfg(feature = "nestgate")]
    #[error("NestGate error: {0}")]
    NestGateError(String),

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
