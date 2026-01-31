//! Cryptographic Capability Module
//!
//! Provides agnostic cryptographic operations for TLS 1.3.
//! Abstracts the underlying provider (BearDog, etc.) enabling
//! runtime discovery and capability-based communication.
//!
//! ## Architecture
//!
//! - **`CryptoCapability`**: Trait defining all crypto operations
//! - **`BearDogProvider`**: Implementation using BearDog via JSON-RPC
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
//! // Or explicit BearDog
//! let crypto = BearDogProvider::new("/tmp/beardog.sock");
//! ```

mod beardog_provider;
mod capability;
mod discovery;
pub mod socket_discovery; // Public for isomorphic IPC

pub use capability::{
    CryptoCapability, CryptoProvider, TlsApplicationSecrets, TlsHandshakeSecrets,
};

pub use beardog_provider::BearDogProvider;
pub use discovery::discover_crypto_capability;
pub use socket_discovery::{
    discover_beardog_socket, discover_ipc_endpoint, discover_neural_api_socket, discover_socket,
    IpcEndpoint,
};
