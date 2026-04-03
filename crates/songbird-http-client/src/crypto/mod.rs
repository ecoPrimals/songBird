// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Cryptographic Capability Module
//!
//! Provides agnostic cryptographic operations for TLS 1.3.
//! Abstracts the underlying provider (`security provider`, etc.) enabling
//! runtime discovery and capability-based communication.
//!
//! ## Architecture
//!
//! - **`CryptoCapability`**: Trait defining all crypto operations
//! - **`SecurityCryptoProvider`**: Implementation using the `security provider` via JSON-RPC
//! - **`discover()`**: Runtime provider discovery
//!
//! ## Usage
//!
//! ```rust,ignore
//! use songbird_http_client::crypto::{discover_crypto_capability, CryptoCapability};
//!
//! // Automatic discovery
//! let crypto = discover_crypto_capability().await?;
//!
//! // Or explicit `SecurityCryptoProvider`
//! let crypto = SecurityCryptoProvider::new("/tmp/beardog.sock");
//! ```

mod capability;
mod discovery;
mod security_provider;
pub mod socket_discovery; // Public for isomorphic IPC

pub use capability::{
    CryptoCapability, CryptoProvider, TlsApplicationSecrets, TlsHandshakeSecrets,
};

pub use security_provider::SecurityCryptoProvider;

/// Deprecated alias for [`SecurityCryptoProvider`].
#[deprecated(note = "use SecurityCryptoProvider (capability-based naming)")]
pub type BearDogProvider = SecurityCryptoProvider;

/// Deprecated alias for `security_provider` module.
#[deprecated(note = "use module `security_provider` (capability-based naming)")]
pub mod beardog_provider {
    pub use super::security_provider::*;
}
pub use discovery::discover_crypto_capability;
/// Deprecated; see [`socket_discovery::discover_security_socket`].
#[allow(deprecated)]
pub use socket_discovery::discover_security_provider_socket;
pub use socket_discovery::{
    IpcEndpoint, discover_ipc_endpoint, discover_neural_api_socket, discover_security_socket,
    discover_socket,
};
