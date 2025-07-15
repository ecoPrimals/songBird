//! # Songbird Federation System
//!
//! Re-exports from the songbird-federation crate for backward compatibility.
//!
//! The federation system provides self-contained networking using Songbird
//! coordination + BearDog security with proximity-first discovery that scales
//! to worldwide mesh.

pub use songbird_federation::*;

// Legacy compatibility re-exports
pub use songbird_federation::{
    FederatedDeploymentRequirements, FederatedDeploymentResult, FederationManager, FederationNode,
    NetworkProximity, NodeType,
};

pub use songbird_federation::types::{FederationConfig, FederationStatus};
pub use songbird_federation::{
    DiscoveryProtocol, NodeStatus, RouteInfo, RouteStrategy, SecuritySession,
};

// Additional convenience functions for backward compatibility
pub async fn create_federation_manager(
    config: FederationConfig,
) -> Result<FederationManager, songbird_errors::SongbirdError> {
    FederationManager::new(config).await
}

pub async fn discover_federation_nodes(
    manager: &FederationManager,
) -> Result<Vec<FederationNode>, songbird_errors::SongbirdError> {
    manager.discover_nodes().await
}
