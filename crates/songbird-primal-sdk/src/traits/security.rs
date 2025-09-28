//! Security traits for Universal Primals
//!
//! Provides security-focused traits for authentication, authorization, and encryption

use songbird_types::{errors::SongbirdResult, CanonicalRequest};

/// Trait for security-focused primal services
#[async_trait::async_trait]
pub trait PrimalSecurity: Send + Sync {
    /// Authenticate a request
    async fn authenticate(&self, request: &CanonicalRequest) -> SongbirdResult<bool>;

    /// Authorize a request
    async fn authorize(&self, request: &CanonicalRequest) -> SongbirdResult<bool>;

    /// Encrypt data
    async fn encrypt(&self, data: &[u8]) -> SongbirdResult<Vec<u8>>;

    /// Decrypt data
    async fn decrypt(&self, encrypted_data: &[u8]) -> SongbirdResult<Vec<u8>>;
}
