//! Primal orchestration traits

use songbird_types::{errors::SongbirdResult, CanonicalRequest, CanonicalResponse};

/// Trait for orchestrating primal services
pub trait PrimalOrchestrator: Send + Sync {
    /// Route a request to appropriate primal
    fn route_request(&self, request: CanonicalRequest) -> impl std::future::Future<Output = SongbirdResult<CanonicalResponse>> + Send;
}
