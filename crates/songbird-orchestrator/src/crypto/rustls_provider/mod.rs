//! Pure Rust TLS via capability-based crypto provider
//!
//! This module implements a rustls `CryptoProvider` that delegates all cryptographic
//! operations to a capability-discovered crypto provider (e.g., BearDog).
//!
//! # Architecture
//!
//! ```text
//! rustls (TLS Protocol) → BeardogCryptoProvider → Capability Discovery → BearDog (Crypto)
//! ```
//!
//! # Components
//!
//! - `secure_random`: Pure Rust RNG using `getrandom` crate
//! - `key_provider`: Key loading and signing delegation to crypto provider
//! - `kx_group`: X25519 key exchange via crypto provider
//! - `cipher_suites`: TLS cipher suite definitions (ChaCha20-Poly1305)
//! - `aead`: AEAD encryption/decryption via crypto provider
//! - `provider`: Main `BeardogCryptoProvider` struct
//!
//! # Principles
//!
//! - **100% Pure Rust**: Zero C dependencies
//! - **Capability-Based**: No hardcoded primal names, runtime discovery
//! - **TRUE PRIMAL**: Only self-knowledge, discovers crypto at runtime
//! - **Deep Debt**: Comprehensive solution, not a quick fix
//!
//! # Example
//!
//! ```rust,no_run
//! use songbird_orchestrator::crypto::rustls_provider::BeardogCryptoProvider;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create and install crypto provider (discovers BearDog by capability)
//!     let provider = BeardogCryptoProvider::new().await?;
//!     provider.install_default()?;
//!     
//!     // Now all rustls usage uses Pure Rust crypto via BearDog!
//!     Ok(())
//! }
//! ```

pub mod key_provider;
pub mod kx_group;
pub mod secure_random;

// Re-exports for public API
pub use key_provider::BeardogKeyProvider;
pub use kx_group::{init_runtime_crypto_provider, X25519_GROUP};
pub use secure_random::GETRANDOM_WRAPPER;

