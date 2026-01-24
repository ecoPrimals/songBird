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

mod capability;
mod beardog_provider;
mod discovery;

pub use capability::{
    CryptoCapability,
    CryptoProvider,
    TlsHandshakeSecrets,
    TlsApplicationSecrets,
};

pub use beardog_provider::BearDogProvider;
pub use discovery::discover_crypto_capability;

