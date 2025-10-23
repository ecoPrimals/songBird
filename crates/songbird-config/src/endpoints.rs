//! Endpoint configuration with zero hardcoding
//!
//! This module provides environment-driven endpoint configuration for all Songbird components.
//! All endpoints support environment variable overrides for flexible deployment.
//!
//! # Environment Variables
//!
//! ## Individual Primal Endpoints
//! - `BEARDOG_ENDPOINT` - `BearDog` security service endpoint
//! - `NESTGATE_ENDPOINT` - `NestGate` system management endpoint
//! - `TOADSTOOL_ENDPOINT` - `ToadStool` data storage endpoint
//! - `SQUIRREL_ENDPOINT` - `Squirrel` AI service endpoint
//! - `SONGBIRD_ENDPOINT` - `Songbird` orchestrator endpoint
//!
//! ## Discovery
//! - `SONGBIRD_DISCOVERY_ENDPOINTS` - Comma-separated list of discovery endpoints
//!
//! ## Host and Port Components
//! - `SONGBIRD_HOST` - Default host for all services
//! - `SONGBIRD_*_PORT` - Individual port overrides (see `defaults::ports`)
//!
//! # Examples
//!
//! ```no_run
//! use songbird_config::endpoints;
//!
//! // Get endpoint with automatic env var support
//! let endpoint = endpoints::get_primal_endpoint("toadstool");
//! // Returns: $TOADSTOOL_ENDPOINT if set, otherwise http://$SONGBIRD_HOST:$TOADSTOOL_PORT
//!
//! // Get all discovery endpoints
//! let endpoints = endpoints::get_discovery_endpoints();
//! // Returns: Comma-separated $SONGBIRD_DISCOVERY_ENDPOINTS or constructed list
//! ```

use crate::defaults::{hosts, ports};
use std::env;

/// Get endpoint URL for any primal service by name
///
/// Tries environment variables in this order:
/// 1. `{PRIMAL}_ENDPOINT` - Specific primal endpoint (e.g., `BEARDOG_ENDPOINT`)
/// 2. `PRIMAL_{PRIMAL}_ENDPOINT` - Alternative format (e.g., `PRIMAL_BEARDOG_ENDPOINT`)
/// 3. Constructs from `SONGBIRD_HOST` + primal-specific port
/// 4. Falls back to `127.0.0.1` + default port
///
/// # Arguments
///
/// * `primal_name` - Name of the primal (case-insensitive): beardog, nestgate, toadstool, squirrel, songbird
///
/// # Examples
///
/// ```no_run
/// use songbird_config::endpoints;
///
/// // With BEARDOG_ENDPOINT set
/// std::env::set_var("BEARDOG_ENDPOINT", "http://beardog-prod:8443");
/// let endpoint = endpoints::get_primal_endpoint("beardog");
/// assert_eq!(endpoint, "http://beardog-prod:8443");
///
/// // Without env var - uses defaults
/// std::env::remove_var("TOADSTOOL_ENDPOINT");
/// let endpoint = endpoints::get_primal_endpoint("toadstool");
/// // Returns: http://127.0.0.1:8080 (or configured values)
/// ```
#[must_use]
pub fn get_primal_endpoint(primal_name: &str) -> String {
    let primal_upper = primal_name.to_uppercase();

    // Try specific env var first: BEARDOG_ENDPOINT
    let env_var = format!("{primal_upper}_ENDPOINT");
    if let Ok(endpoint) = env::var(&env_var) {
        return endpoint;
    }

    // Try generic primal endpoint pattern: PRIMAL_BEARDOG_ENDPOINT
    let generic_env = format!("PRIMAL_{primal_upper}_ENDPOINT");
    if let Ok(endpoint) = env::var(&generic_env) {
        return endpoint;
    }

    // Construct from host and port
    let host = hosts::default_host();
    let port = get_primal_port(primal_name);
    format!("http://{host}:{port}")
}

/// Get the default port for a primal service
///
/// Uses the centralized port configuration from `defaults::ports`
fn get_primal_port(primal_name: &str) -> u16 {
    match primal_name.to_lowercase().as_str() {
        "beardog" => ports::beardog_port(),
        "nestgate" => ports::nestgate_port(),
        "toadstool" => ports::toadstool_port(),
        "squirrel" => ports::squirrel_port(),
        "songbird" | "orchestrator" => ports::orchestrator_port(),
        "discovery" => ports::discovery_port(),
        "dashboard" => ports::dashboard_port(),
        "metrics" => ports::metrics_port(),
        "federation" => ports::federation_port(),
        "health" => ports::health_port(),
        _ => {
            // Unknown primal - use discovery port as fallback
            ports::discovery_port()
        }
    }
}

/// Get list of endpoints for service discovery
///
/// Returns a list of endpoints to query for service discovery.
/// Supports environment variable override for custom discovery topology.
///
/// # Environment Variables
///
/// * `SONGBIRD_DISCOVERY_ENDPOINTS` - Comma-separated list of endpoints
///
/// # Examples
///
/// ```no_run
/// use songbird_config::endpoints;
///
/// // With custom discovery endpoints
/// std::env::set_var("SONGBIRD_DISCOVERY_ENDPOINTS", "http://consul:8500,http://etcd:2379");
/// let endpoints = endpoints::get_discovery_endpoints();
/// assert_eq!(endpoints.len(), 2);
///
/// // Without env var - uses primal endpoints
/// std::env::remove_var("SONGBIRD_DISCOVERY_ENDPOINTS");
/// let endpoints = endpoints::get_discovery_endpoints();
/// // Returns: List of all known primal endpoints
/// ```
#[must_use]
pub fn get_discovery_endpoints() -> Vec<String> {
    // Try env var first for custom discovery topology
    if let Ok(endpoints_str) = env::var("SONGBIRD_DISCOVERY_ENDPOINTS") {
        return endpoints_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }

    // Build from known primals
    vec![
        get_primal_endpoint("orchestrator"),
        get_primal_endpoint("discovery"),
        get_primal_endpoint("beardog"),
        get_primal_endpoint("nestgate"),
        get_primal_endpoint("toadstool"),
        get_primal_endpoint("squirrel"),
    ]
}

/// Get endpoint for a specific service by capability
///
/// Similar to `get_primal_endpoint` but for capability-based routing.
/// Falls back to discovery endpoint if capability is unknown.
///
/// # Arguments
///
/// * `capability` - Service capability name (e.g., "authentication", "storage", "compute")
///
/// # Examples
///
/// ```no_run
/// use songbird_config::endpoints;
///
/// let endpoint = endpoints::get_capability_endpoint("authentication");
/// // Returns: BearDog endpoint (security/auth capability)
///
/// let endpoint = endpoints::get_capability_endpoint("storage");
/// // Returns: ToadStool endpoint (storage capability)
/// ```
#[must_use]
pub fn get_capability_endpoint(capability: &str) -> String {
    // Map capabilities to primals
    let primal = match capability.to_lowercase().as_str() {
        "authentication" | "security" | "auth" => "beardog",
        "storage" | "database" | "persistence" | "compute" | "execution" | "runtime" => "toadstool", // ToadStool handles storage and compute
        "ai" | "ml" | "inference" | "intelligence" => "squirrel",
        "system" | "management" | "orchestration" => "nestgate",
        "coordination" | "discovery" | "registry" => "orchestrator",
        _ => "discovery", // Unknown capabilities go to discovery
    };

    get_primal_endpoint(primal)
}

/// Check if endpoint is localhost/loopback
///
/// Useful for determining if service is running locally or remote.
#[must_use]
pub fn is_localhost_endpoint(endpoint: &str) -> bool {
    endpoint.contains("localhost")
        || endpoint.contains("127.0.0.1")
        || endpoint.contains("::1")
        || endpoint.contains("0.0.0.0")
}

/// Parse endpoint into components (scheme, host, port)
///
/// # Examples
///
/// ```
/// use songbird_config::endpoints;
///
/// let (scheme, host, port) = endpoints::parse_endpoint("http://service:8080");
/// assert_eq!(scheme, "http");
/// assert_eq!(host, "service");
/// assert_eq!(port, Some(8080));
/// ```
#[must_use]
pub fn parse_endpoint(endpoint: &str) -> (String, String, Option<u16>) {
    // Simple parser - could be enhanced with url crate
    let endpoint = endpoint.trim();

    // Extract scheme
    let (scheme, rest) = if let Some(pos) = endpoint.find("://") {
        let scheme = endpoint[..pos].to_string();
        let rest = &endpoint[pos + 3..];
        (scheme, rest)
    } else {
        ("http".to_string(), endpoint)
    };

    // Extract host and port
    let (host, port) = if let Some(pos) = rest.rfind(':') {
        let host = rest[..pos].to_string();
        let port_str = &rest[pos + 1..];
        // Remove path if present
        let port_str = port_str.split('/').next().unwrap_or(port_str);
        let port = port_str.parse().ok();
        (host, port)
    } else {
        (rest.to_string(), None)
    };

    // Remove path from host if present
    let host = host.split('/').next().unwrap_or(&host).to_string();

    (scheme, host, port)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_primal_endpoint_from_env() {
        env::set_var("BEARDOG_ENDPOINT", "http://custom-beardog:9443");
        let endpoint = get_primal_endpoint("beardog");
        assert_eq!(endpoint, "http://custom-beardog:9443");
        env::remove_var("BEARDOG_ENDPOINT");
    }

    #[test]
    fn test_get_primal_endpoint_default() {
        env::remove_var("NESTGATE_ENDPOINT");
        env::remove_var("PRIMAL_NESTGATE_ENDPOINT");
        let endpoint = get_primal_endpoint("nestgate");
        assert!(endpoint.starts_with("http://"));
        assert!(endpoint.contains(":")); // Should have port
    }

    #[test]
    fn test_get_discovery_endpoints_from_env() {
        env::set_var("SONGBIRD_DISCOVERY_ENDPOINTS", "http://consul:8500,http://etcd:2379");
        let endpoints = get_discovery_endpoints();
        assert_eq!(endpoints.len(), 2);
        assert_eq!(endpoints[0], "http://consul:8500");
        assert_eq!(endpoints[1], "http://etcd:2379");
        env::remove_var("SONGBIRD_DISCOVERY_ENDPOINTS");
    }

    #[test]
    fn test_get_discovery_endpoints_default() {
        env::remove_var("SONGBIRD_DISCOVERY_ENDPOINTS");
        let endpoints = get_discovery_endpoints();
        assert_eq!(endpoints.len(), 6); // All known primals
        assert!(endpoints[0].starts_with("http://"));
    }

    #[test]
    fn test_get_capability_endpoint() {
        let endpoint = get_capability_endpoint("authentication");
        assert!(endpoint.contains(":")); // Should have port

        let endpoint = get_capability_endpoint("storage");
        assert!(endpoint.contains(":")); // Should have port
    }

    #[test]
    fn test_is_localhost_endpoint() {
        assert!(is_localhost_endpoint("http://localhost:8080"));
        assert!(is_localhost_endpoint("http://127.0.0.1:8080"));
        assert!(is_localhost_endpoint("http://[::1]:8080"));
        assert!(!is_localhost_endpoint("http://production-server:8080"));
    }

    #[test]
    fn test_parse_endpoint() {
        let (scheme, host, port) = parse_endpoint("http://service:8080");
        assert_eq!(scheme, "http");
        assert_eq!(host, "service");
        assert_eq!(port, Some(8080));

        let (scheme, host, port) = parse_endpoint("https://api.example.com:443/path");
        assert_eq!(scheme, "https");
        assert_eq!(host, "api.example.com");
        assert_eq!(port, Some(443));

        let (scheme, host, port) = parse_endpoint("http://localhost");
        assert_eq!(scheme, "http");
        assert_eq!(host, "localhost");
        assert_eq!(port, None);
    }

    #[test]
    fn test_case_insensitive_primal_names() {
        env::remove_var("BEARDOG_ENDPOINT");
        let endpoint1 = get_primal_endpoint("beardog");
        let endpoint2 = get_primal_endpoint("BEARDOG");
        let endpoint3 = get_primal_endpoint("BearDog");
        assert_eq!(endpoint1, endpoint2);
        assert_eq!(endpoint2, endpoint3);
    }
}
