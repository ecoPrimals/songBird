//! Convenient Discovery Helpers
//!
//! High-level utilities for capability-based primal discovery with fallbacks.

use crate::capability_discovery::{CapabilityDiscovery, ServiceEndpoint};
use anyhow::{Context, Result};
use std::env;
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
/// ```rust,no_run
/// use songbird_config::discover_primal;
/// use songbird_types::CanonicalPrimalType;
///
/// # async fn example() -> anyhow::Result<()> {
/// // Discover security primal (BearDog)
/// let endpoint = discover_primal(CanonicalPrimalType::Security).await?;
/// println!("Found security primal at: {}", endpoint.url);
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if the primal cannot be discovered through any available method
/// (capability registry, environment variables, mDNS, or DNS-SD).
pub async fn discover_primal(
    primal_type: songbird_types::CanonicalPrimalType,
) -> Result<ServiceEndpoint> {
    let capability = primal_type_to_capability(&primal_type);

    info!("Discovering primal: {} (capability: {})", primal_type, capability);

    // 1. Try capability registry (preferred method)
    let discovery = CapabilityDiscovery::new();
    if let Ok(endpoints) = discovery.find_providers_by_capability(&capability).await {
        if let Some(endpoint) = endpoints.first() {
            info!("Found {} via capability registry at {}", primal_type, endpoint.url);
            return Ok(endpoint.clone());
        }
    }

    // 2. Try environment variable
    let env_var = format!("{}_URL", capability.to_uppercase().replace('-', "_"));
    if let Ok(url) = env::var(&env_var) {
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
    if let Ok(url) = env::var(&alt_env_var) {
        info!("Found {} via environment variable {} = {}", primal_type, alt_env_var, url);
        return Ok(ServiceEndpoint {
            id: format!("{capability}-from-env-alt"),
            url,
            capabilities: vec![capability.clone()],
            health_score: 1.0,
            last_seen: std::time::SystemTime::now(),
        });
    }

    // 4. Development fallback (debug builds only)
    if cfg!(debug_assertions) {
        let fallback_url = default_url_for_primal(&primal_type);
        warn!(
            "No {} primal found via discovery or environment. Using development fallback: {}",
            primal_type, fallback_url
        );
        warn!("Set {} or {} to override", env_var, alt_env_var);

        return Ok(ServiceEndpoint {
            id: format!("{capability}-fallback"),
            url: fallback_url,
            capabilities: vec![capability],
            health_score: 0.5, // Lower score for fallback
            last_seen: std::time::SystemTime::now(),
        });
    }

    // Production: No fallback
    anyhow::bail!(
        "No {primal_type} primal found. Set {env_var} or {alt_env_var} environment variable, or register via capability discovery"
    )
}

/// Discover multiple primals by capability type
///
/// # Errors
///
/// Returns an error if the primals cannot be discovered via the capability discovery system.
pub async fn discover_all_primals(
    primal_type: songbird_types::CanonicalPrimalType,
) -> Result<Vec<ServiceEndpoint>> {
    let capability = primal_type_to_capability(&primal_type);
    let discovery = CapabilityDiscovery::new();

    discovery
        .find_providers_by_capability(&capability)
        .await
        .context(format!("Failed to discover {primal_type} primals"))
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

/// Get default development URL for a primal type
fn default_url_for_primal(primal_type: &songbird_types::CanonicalPrimalType) -> String {
    use songbird_types::CanonicalPrimalType;

    let port = match primal_type {
        CanonicalPrimalType::Security => 8200,      // BearDog
        CanonicalPrimalType::Storage => 6000,       // Squirrel
        CanonicalPrimalType::Compute => 7000,       // Toadstool
        CanonicalPrimalType::Ai => 7100,            // AI services
        CanonicalPrimalType::Orchestration => 8080, // Songbird
        CanonicalPrimalType::Federation => 8090,    // Federation
        CanonicalPrimalType::Discovery => 5300,     // Discovery
        CanonicalPrimalType::Registry => 8081,      // Registry
        CanonicalPrimalType::Observability => 9090, // Observability
        CanonicalPrimalType::Unknown(_) => 9999,    // Unknown
    };

    format!("http://[::]:{port}")
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
    fn test_default_urls() {
        assert_eq!(default_url_for_primal(&CanonicalPrimalType::Security), "http://[::]:8200");
        assert_eq!(default_url_for_primal(&CanonicalPrimalType::Compute), "http://[::]:7000");
    }

    #[test]
    fn test_env_var_names() {
        let vars = env_var_for_primal(&CanonicalPrimalType::Security);
        assert!(vars.contains(&"SECURITY_URL".to_string()));
        assert!(vars.contains(&"SECURITY_PRIMAL_URL".to_string()));
    }
}
