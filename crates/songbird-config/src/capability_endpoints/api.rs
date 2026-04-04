// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Convenience entry points for callers that do not hold a [`CapabilityEndpointResolver`](super::CapabilityEndpointResolver).

use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;

use super::resolver::CapabilityEndpointResolver;
use super::types::{CapabilityEndpoint, CapabilityType};

/// Get endpoint for a capability (convenience function)
///
/// # Examples
///
/// ```no_run
/// use songbird_config::capability_endpoints;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let endpoint = capability_endpoints::get_capability_endpoint("security").await?;
///
/// let endpoint = capability_endpoints::get_endpoint_typed(
///     capability_endpoints::CapabilityType::Storage
/// ).await?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
/// Returns error if no endpoint can be discovered for the capability
///
/// Returns an error if capability string parsing fails
pub async fn get_capability_endpoint(capability: &str) -> SongbirdResult<String> {
    let capability_type = capability.parse::<CapabilityType>().map_err(|e| {
        SongbirdError::configuration(format!("Invalid capability type '{capability}': {e}"))
    })?;
    let resolver = CapabilityEndpointResolver::new();
    resolver.get_endpoint(capability_type).await
}

/// Get endpoint for a typed capability
///
/// # Errors
/// Returns error if no endpoint can be discovered for the capability
pub async fn get_endpoint_typed(capability: CapabilityType) -> SongbirdResult<String> {
    let resolver = CapabilityEndpointResolver::new();
    resolver.get_endpoint(capability).await
}

/// Get all available capability endpoints
pub async fn get_all_endpoints() -> HashMap<CapabilityType, CapabilityEndpoint> {
    let resolver = CapabilityEndpointResolver::new();
    resolver.get_all_cached().await
}

/// Clear endpoint cache (force re-discovery)
///
/// Note: With current implementation, this creates a new resolver instance,
/// so cache clearing is implicit. Future versions may use a global instance.
pub const fn clear_cache() {
    // No-op with current architecture - each call creates new resolver
    // This is intentional to avoid global state complexity
}

/// Check if a capability endpoint is available
pub async fn has_capability(capability: &str) -> bool {
    get_capability_endpoint(capability).await.is_ok()
}

/// Get multiple capability endpoints in parallel
///
/// # Errors
/// Returns error if any capability endpoint cannot be discovered
pub async fn get_multiple_endpoints(capabilities: &[&str]) -> SongbirdResult<Vec<String>> {
    let mut endpoints = Vec::new();

    for capability in capabilities {
        let endpoint = get_capability_endpoint(capability).await?;
        endpoints.push(endpoint);
    }

    Ok(endpoints)
}
