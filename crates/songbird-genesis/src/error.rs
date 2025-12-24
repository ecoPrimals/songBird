//! Error types for genesis operations

use thiserror::Error;

/// Result type for genesis operations
pub type Result<T> = std::result::Result<T, GenesisError>;

/// Errors that can occur during genesis operations
#[derive(Debug, Error)]
pub enum GenesisError {
    /// Physical proximity verification failed
    #[error("Physical proximity verification failed: {0}")]
    ProximityVerificationFailed(String),

    /// Witness signature invalid
    #[error("Witness signature invalid: {0}")]
    InvalidWitnessSignature(String),

    /// Physical channel error
    #[error("Physical channel error: {0}")]
    PhysicalChannelError(String),

    /// Coordination error (when using primal coordination)
    #[error("Coordination failed: {0}")]
    CoordinationFailed(String),

    /// Genesis ceremony timeout
    #[error("Genesis ceremony timed out after {0}s")]
    CeremonyTimeout(u64),

    /// Witness not authorized
    #[error("Witness not authorized: {0}")]
    UnauthorizedWitness(String),

    /// Invalid genesis certificate
    #[error("Invalid genesis certificate: {0}")]
    InvalidCertificate(String),

    /// Lineage establishment failed
    #[error("Lineage establishment failed: {0}")]
    LineageFailed(String),

    /// Hardware key error (SoloKey, YubiKey, etc.)
    #[error("Hardware key error: {0}")]
    HardwareKeyError(String),

    /// QR code error
    #[error("QR code error: {0}")]
    QrCodeError(String),

    /// Bluetooth error
    #[error("Bluetooth error: {0}")]
    BluetoothError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Generic error
    #[error("{0}")]
    Other(String),
}

impl From<anyhow::Error> for GenesisError {
    fn from(err: anyhow::Error) -> Self {
        GenesisError::Other(err.to_string())
    }
}

impl From<String> for GenesisError {
    fn from(msg: String) -> Self {
        GenesisError::Other(msg)
    }
}

impl From<&str> for GenesisError {
    fn from(msg: &str) -> Self {
        GenesisError::Other(msg.to_string())
    }
}
