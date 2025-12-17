//! Primal Discovery Functions
//!
//! Modern replacement for deprecated hardcoded primal endpoints.
//! These functions discover primals through environment variables and
//! capability-based discovery, respecting sovereignty principles.
//!
//! ## Migration from Deprecated Constants
//!
//! ```rust,ignore
//! // ❌ OLD: Hardcoded endpoints (DEPRECATED)
//! use songbird_config::config::constants::deprecated::DEFAULT_TOADSTOOL_ENDPOINT;
//! let endpoint = DEFAULT_TOADSTOOL_ENDPOINT;
//!
//! // ✅ NEW: Environment + Discovery
//! use songbird_config::primal_discovery::get_compute_endpoint;
//! let endpoint = get_compute_endpoint().await?;
//! ```

use songbird_types::{SongbirdError, SongbirdResult};
use tracing::{debug, warn};

/// Get compute provider endpoint (replaces `DEFAULT_TOADSTOOL_ENDPOINT`)
///
/// Discovery order:
/// 1. `COMPUTE_ENDPOINT` environment variable
/// 2. `TOADSTOOL_ENDPOINT` environment variable (backwards compatibility)
/// 3. Capability-based discovery (future)
/// 4. Error - no hardcoded fallback
///
/// ## Example
///
/// ```no_run
/// use songbird_config::primal_discovery::get_compute_endpoint;
///
/// #[tokio::main]
/// async fn main() -> songbird_types::SongbirdResult<()> {
///     // Set via environment:
///     // export COMPUTE_ENDPOINT=http://my-compute-provider:8001
///     
///     let endpoint = get_compute_endpoint().await?;
///     println!("Compute provider: {}", endpoint);
///     Ok(())
/// }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - No `COMPUTE_ENDPOINT` or `TOADSTOOL_ENDPOINT` environment variable is set
/// - Capability-based discovery fails to find a compute provider
pub async fn get_compute_endpoint() -> SongbirdResult<String> {
    // 1. Try modern COMPUTE_ENDPOINT
    if let Ok(endpoint) = std::env::var("COMPUTE_ENDPOINT") {
        debug!("Using COMPUTE_ENDPOINT from environment: {}", endpoint);
        return Ok(endpoint);
    }

    // 2. Try legacy TOADSTOOL_ENDPOINT (backwards compatibility)
    if let Ok(endpoint) = std::env::var("TOADSTOOL_ENDPOINT") {
        warn!("Using deprecated TOADSTOOL_ENDPOINT - migrate to COMPUTE_ENDPOINT");
        return Ok(endpoint);
    }

    // 3. Try capability-based discovery (RuntimeDiscoveryEngine)
    match crate::runtime_discovery::discover_compute().await {
        Ok(service) => {
            debug!("Discovered compute via RuntimeDiscoveryEngine: {}", service.endpoint);
            return Ok(service.endpoint);
        }
        Err(e) => {
            debug!("Runtime discovery failed: {}", e);
            // Fall through to error
        }
    }

    // 4. No hardcoded fallback - fail with helpful message
    Err(SongbirdError::Configuration {
        message: "No compute provider configured.".to_string(),
        field: Some("compute_endpoint".to_string()),
        suggestion: Some("Set COMPUTE_ENDPOINT environment variable (e.g., export COMPUTE_ENDPOINT=http://your-provider:8001) or enable capability discovery.".to_string()),
    })
}

/// Get storage provider endpoint (replaces `DEFAULT_NESTGATE_ENDPOINT`)
///
/// Discovery order:
/// 1. `STORAGE_ENDPOINT` environment variable
/// 2. `NESTGATE_ENDPOINT` environment variable (backwards compatibility)
/// 3. Capability-based discovery (future)
/// 4. Error - no hardcoded fallback
///
/// # Errors
///
/// Returns an error if:
/// - No `STORAGE_ENDPOINT` or `NESTGATE_ENDPOINT` environment variable is set
/// - Capability-based discovery fails to find a storage provider
pub async fn get_storage_endpoint() -> SongbirdResult<String> {
    // 1. Try modern STORAGE_ENDPOINT
    if let Ok(endpoint) = std::env::var("STORAGE_ENDPOINT") {
        debug!("Using STORAGE_ENDPOINT from environment: {}", endpoint);
        return Ok(endpoint);
    }

    // 2. Try legacy NESTGATE_ENDPOINT (backwards compatibility)
    if let Ok(endpoint) = std::env::var("NESTGATE_ENDPOINT") {
        warn!("Using deprecated NESTGATE_ENDPOINT - migrate to STORAGE_ENDPOINT");
        return Ok(endpoint);
    }

    // 3. Try capability-based discovery (RuntimeDiscoveryEngine)
    match crate::runtime_discovery::discover_storage().await {
        Ok(service) => {
            debug!("Discovered storage via RuntimeDiscoveryEngine: {}", service.endpoint);
            return Ok(service.endpoint);
        }
        Err(e) => {
            debug!("Runtime discovery failed: {}", e);
            // Fall through to error
        }
    }

    Err(SongbirdError::Configuration {
        message: "No storage provider configured.".to_string(),
        field: Some("storage_endpoint".to_string()),
        suggestion: Some("Set STORAGE_ENDPOINT environment variable (e.g., export STORAGE_ENDPOINT=http://your-provider:8003) or enable capability discovery.".to_string()),
    })
}

/// Get security provider endpoint (replaces `DEFAULT_BEARDOG_ENDPOINT`)
///
/// Discovery order:
/// 1. `SECURITY_ENDPOINT` environment variable
/// 2. `BEARDOG_ENDPOINT` environment variable (backwards compatibility)
/// 3. Capability-based discovery (future)
/// 4. Error - no hardcoded fallback
///
/// # Errors
///
/// Returns an error if:
/// - No `SECURITY_ENDPOINT` or `BEARDOG_ENDPOINT` environment variable is set
/// - Capability-based discovery fails to find a security provider
pub async fn get_security_endpoint() -> SongbirdResult<String> {
    // 1. Try modern SECURITY_ENDPOINT
    if let Ok(endpoint) = std::env::var("SECURITY_ENDPOINT") {
        debug!("Using SECURITY_ENDPOINT from environment: {}", endpoint);
        return Ok(endpoint);
    }

    // 2. Try legacy BEARDOG_ENDPOINT (backwards compatibility)
    if let Ok(endpoint) = std::env::var("BEARDOG_ENDPOINT") {
        warn!("Using deprecated BEARDOG_ENDPOINT - migrate to SECURITY_ENDPOINT");
        return Ok(endpoint);
    }

    // 3. Try capability-based discovery (RuntimeDiscoveryEngine)
    match crate::runtime_discovery::discover_security().await {
        Ok(service) => {
            debug!("Discovered security via RuntimeDiscoveryEngine: {}", service.endpoint);
            return Ok(service.endpoint);
        }
        Err(e) => {
            debug!("Runtime discovery failed: {}", e);
            // Fall through to error
        }
    }

    Err(SongbirdError::Configuration {
        message: "No security provider configured.".to_string(),
        field: Some("security_endpoint".to_string()),
        suggestion: Some("Set SECURITY_ENDPOINT environment variable (e.g., export SECURITY_ENDPOINT=http://your-provider:8004) or enable capability discovery.".to_string()),
    })
}

/// Get AI provider endpoint (replaces `DEFAULT_SQUIRREL_ENDPOINT`)
///
/// Discovery order:
/// 1. `AI_ENDPOINT` environment variable
/// 2. `SQUIRREL_ENDPOINT` environment variable (backwards compatibility)
/// 3. Capability-based discovery (future)
/// 4. Error - no hardcoded fallback
///
/// # Errors
///
/// Returns an error if:
/// - No `AI_ENDPOINT` or `SQUIRREL_ENDPOINT` environment variable is set
/// - Capability-based discovery fails to find an AI provider
pub async fn get_ai_endpoint() -> SongbirdResult<String> {
    // 1. Try modern AI_ENDPOINT
    if let Ok(endpoint) = std::env::var("AI_ENDPOINT") {
        debug!("Using AI_ENDPOINT from environment: {}", endpoint);
        return Ok(endpoint);
    }

    // 2. Try legacy SQUIRREL_ENDPOINT (backwards compatibility)
    if let Ok(endpoint) = std::env::var("SQUIRREL_ENDPOINT") {
        warn!("Using deprecated SQUIRREL_ENDPOINT - migrate to AI_ENDPOINT");
        return Ok(endpoint);
    }

    // 3. Try capability-based discovery (RuntimeDiscoveryEngine)
    match crate::runtime_discovery::discover_ai().await {
        Ok(service) => {
            debug!("Discovered AI via RuntimeDiscoveryEngine: {}", service.endpoint);
            return Ok(service.endpoint);
        }
        Err(e) => {
            debug!("Runtime discovery failed: {}", e);
            // Fall through to error
        }
    }

    Err(SongbirdError::Configuration {
        message: "No AI provider configured.".to_string(),
        field: Some("ai_endpoint".to_string()),
        suggestion: Some("Set AI_ENDPOINT environment variable (e.g., export AI_ENDPOINT=http://your-provider:8002) or enable capability discovery.".to_string()),
    })
}

/// Get any primal endpoint by capability
///
/// This is the most flexible function - discovers ANY provider offering
/// the specified capability, not just specific primals.
///
/// ## Example
///
/// ```no_run
/// use songbird_config::primal_discovery::get_endpoint_by_capability;
///
/// #[tokio::main]
/// async fn main() -> songbird_types::SongbirdResult<()> {
///     // Discovers ANY provider with "compute" capability
///     let endpoint = get_endpoint_by_capability("compute").await?;
///     Ok(())
/// }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - No `{CAPABILITY}_ENDPOINT` environment variable is set
/// - Capability-based discovery fails to find a provider with the requested capability
pub async fn get_endpoint_by_capability(capability: &str) -> SongbirdResult<String> {
    // 1. Try environment variable {CAPABILITY}_ENDPOINT
    let env_var = format!("{}_ENDPOINT", capability.to_uppercase());
    if let Ok(endpoint) = std::env::var(&env_var) {
        debug!("Using {} from environment: {}", env_var, endpoint);
        return Ok(endpoint);
    }

    // 2. Try capability-based discovery (RuntimeDiscoveryEngine)
    let engine = crate::runtime_discovery::RuntimeDiscoveryEngine::new();
    match engine.discover_by_capability(capability).await {
        Ok(service) => {
            debug!("Discovered {} via RuntimeDiscoveryEngine: {}", capability, service.endpoint);
            return Ok(service.endpoint);
        }
        Err(e) => {
            debug!("Runtime discovery failed for {}: {}", capability, e);
            // Fall through to error
        }
    }

    Err(SongbirdError::Configuration {
        message: format!("No provider found for capability '{capability}'."),
        field: Some(format!("{capability}_endpoint")),
        suggestion: Some(format!(
            "Set {}_ENDPOINT environment variable (e.g., export {}_ENDPOINT=http://your-provider:PORT)",
            capability.to_uppercase(), capability.to_uppercase()
        )),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn test_compute_endpoint_from_env() {
        // Clear all env vars first
        std::env::remove_var("COMPUTE_ENDPOINT");
        std::env::remove_var("TOADSTOOL_ENDPOINT");
        
        std::env::set_var("COMPUTE_ENDPOINT", "http://test-compute:9000");

        let result = get_compute_endpoint().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "http://test-compute:9000");

        std::env::remove_var("COMPUTE_ENDPOINT");
    }

    #[tokio::test]
    #[serial]
    async fn test_compute_endpoint_legacy_fallback() {
        // Clean up all potentially interfering env vars first
        std::env::remove_var("COMPUTE_ENDPOINT");
        std::env::remove_var("TOADSTOOL_ENDPOINT");

        // Now set the legacy endpoint
        std::env::set_var("TOADSTOOL_ENDPOINT", "http://legacy-toadstool:8001");

        let result = get_compute_endpoint().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "http://legacy-toadstool:8001");

        // Clean up
        std::env::remove_var("TOADSTOOL_ENDPOINT");
    }

    #[tokio::test]
    #[serial]
    async fn test_compute_endpoint_not_configured() {
        std::env::remove_var("COMPUTE_ENDPOINT");
        std::env::remove_var("TOADSTOOL_ENDPOINT");

        let result = get_compute_endpoint().await;
        assert!(result.is_err());

        if let Err(SongbirdError::Configuration {
            message,
            suggestion,
            ..
        }) = result
        {
            assert!(message.contains("No compute provider configured"));
            assert!(suggestion.is_some());
            let suggestion_text = suggestion.unwrap();
            assert!(suggestion_text.contains("COMPUTE_ENDPOINT"));
        }
    }

    #[tokio::test]
    async fn test_capability_based_endpoint() {
        std::env::set_var("MYSERVICE_ENDPOINT", "http://my-service:5000");

        let result = get_endpoint_by_capability("myservice").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "http://my-service:5000");

        std::env::remove_var("MYSERVICE_ENDPOINT");
    }
}
