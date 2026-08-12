// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Convenient Discovery Helpers
//!
//! High-level utilities for capability-based primal discovery with fallbacks.

use crate::capability_discovery::{CapabilityDiscovery, ServiceEndpoint};
use songbird_types::{SongbirdError, SongbirdResult};
use tracing::{debug, info, warn};

/// Discover a primal by capability with intelligent fallbacks
///
/// Discovery order:
/// 1. Capability registry (runtime discovery)
/// 2. Environment variable
/// 3. Configuration file
/// 4. Default localhost (development only)
///
/// # Example
///
/// ```rust,ignore
/// use songbird_config::discovery_helpers::discover_primal;
/// use songbird_types::{CanonicalPrimalType, SongbirdResult};
///
/// async fn example() -> SongbirdResult<()> {
///     let endpoint = discover_primal(CanonicalPrimalType::Security).await?;
///     println!("Found security primal at: {}", endpoint.url);
///     Ok(())
/// }
/// ```
///
/// # Errors
///
/// Returns an error if the primal cannot be discovered through any available method
/// (capability registry, environment variables, mDNS, or DNS-SD).
pub async fn discover_primal(
    primal_type: songbird_types::CanonicalPrimalType,
) -> SongbirdResult<ServiceEndpoint> {
    let capability = primal_type_to_capability(&primal_type);

    info!("Discovering primal: {} (capability: {})", primal_type, capability);

    // 1. Try capability registry (preferred method)
    let discovery = CapabilityDiscovery::new();
    if let Ok(endpoints) = discovery.find_providers_by_capability(&capability).await
        && let Some(endpoint) = endpoints.first()
    {
        info!("Found {} via capability registry at {}", primal_type, endpoint.url);
        return Ok(endpoint.clone());
    }

    // 2. Try environment variable
    let env_var = format!("{}_URL", capability.to_uppercase().replace('-', "_"));
    if let Ok(url) = songbird_process_env::var(&env_var) {
        info!("Found {} via environment variable {} = {}", primal_type, env_var, url);
        return Ok(ServiceEndpoint {
            id: format!("{capability}-from-env"),
            url,
            capabilities: vec![capability.clone()],
            health_score: 1.0,
            last_seen: std::time::SystemTime::now(),
        });
    }

    // 3. Try alternative environment variable (PRIMAL_TYPE_URL format)
    let alt_env_var = format!("{}_PRIMAL_URL", primal_type.to_string().to_uppercase());
    if let Ok(url) = songbird_process_env::var(&alt_env_var) {
        info!("Found {} via environment variable {} = {}", primal_type, alt_env_var, url);
        return Ok(ServiceEndpoint {
            id: format!("{capability}-from-env-alt"),
            url,
            capabilities: vec![capability.clone()],
            health_score: 1.0,
            last_seen: std::time::SystemTime::now(),
        });
    }

    // 4. Development fallback (debug builds only): songBird self-knowledge for orchestration
    if cfg!(debug_assertions) {
        use songbird_types::CanonicalPrimalType;

        if matches!(primal_type, CanonicalPrimalType::Orchestration) {
            let port = songbird_types::defaults::ports::DEFAULT_ORCHESTRATOR_PORT;
            let fallback_url = format!("http://[::]:{port}");
            warn!(
                "No {primal_type} primal found via discovery or environment. Using songBird self default: {fallback_url}"
            );
            warn!("Set {env_var} or {alt_env_var} to override");

            return Ok(ServiceEndpoint {
                id: format!("{capability}-fallback"),
                url: fallback_url,
                capabilities: vec![capability],
                health_score: 0.5,
                last_seen: std::time::SystemTime::now(),
            });
        }
    }

    // Production (and non-orchestration debug builds): No fallback
    Err(SongbirdError::Configuration {
        message: format!("No {primal_type} primal found"),
        field: Some(String::from("primal_type")),
        suggestion: Some(format!(
            "Set {env_var} or {alt_env_var} environment variable, or register via capability discovery"
        )),
    })
}

/// Discover multiple primals by capability type
///
/// # Errors
///
/// Returns an error if the primals cannot be discovered via the capability discovery system.
pub async fn discover_all_primals(
    primal_type: songbird_types::CanonicalPrimalType,
) -> SongbirdResult<Vec<ServiceEndpoint>> {
    let capability = primal_type_to_capability(&primal_type);
    let discovery = CapabilityDiscovery::new();

    discovery.find_providers_by_capability(&capability).await
}

/// Try to discover a primal, returning None if not found (non-failing)
pub async fn try_discover_primal(
    primal_type: songbird_types::CanonicalPrimalType,
) -> Option<ServiceEndpoint> {
    match discover_primal(primal_type.clone()).await {
        Ok(endpoint) => Some(endpoint),
        Err(e) => {
            debug!("Failed to discover {}: {}", primal_type, e);
            None
        }
    }
}

/// Convert primal type to capability string
fn primal_type_to_capability(primal_type: &songbird_types::CanonicalPrimalType) -> String {
    use songbird_types::CanonicalPrimalType;

    match primal_type {
        CanonicalPrimalType::Security => "security",
        CanonicalPrimalType::Storage => "storage",
        CanonicalPrimalType::Compute => "compute",
        CanonicalPrimalType::Ai => "ai",
        CanonicalPrimalType::Orchestration => "orchestration",
        CanonicalPrimalType::Federation => "federation",
        CanonicalPrimalType::Discovery => "discovery",
        CanonicalPrimalType::Registry => "registry",
        CanonicalPrimalType::Observability => "observability",
        CanonicalPrimalType::Unknown(name) => name.as_str(),
    }
    .to_string()
}

/// Environment variable naming standards
#[must_use]
pub fn env_var_for_primal(primal_type: &songbird_types::CanonicalPrimalType) -> Vec<String> {
    let capability = primal_type_to_capability(primal_type);
    vec![
        format!("{}_URL", capability.to_uppercase().replace('-', "_")),
        format!("{}_PRIMAL_URL", primal_type.to_string().to_uppercase()),
        format!("{}_ENDPOINT", capability.to_uppercase().replace('-', "_")),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use songbird_types::CanonicalPrimalType;

    #[test]
    fn test_primal_to_capability() {
        assert_eq!(primal_type_to_capability(&CanonicalPrimalType::Security), "security");
        assert_eq!(primal_type_to_capability(&CanonicalPrimalType::Compute), "compute");
        assert_eq!(primal_type_to_capability(&CanonicalPrimalType::Storage), "storage");
    }

    #[test]
    fn test_env_var_names() {
        let vars = env_var_for_primal(&CanonicalPrimalType::Security);
        assert!(vars.contains(&String::from("SECURITY_URL")));
        assert!(vars.contains(&String::from("SECURITY_PRIMAL_URL")));
    }
}
