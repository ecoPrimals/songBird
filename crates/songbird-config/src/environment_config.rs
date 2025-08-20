//! Centralized environment variable handling for Songbird ecosystem integration.
//! Provides fallback defaults and ecosystem-wide configuration patterns.

use std::collections::HashMap;
use std::env;

/// Environment configuration helper for ecosystem integration
pub struct EnvironmentConfig;

impl Default for EnvironmentConfig {
    fn default() -> Self {
        Self
    }
}

impl EnvironmentConfig {
    /// Get the Songbird orchestrator endpoint from environment or calculate from config
    #[must_use]
    pub fn songbird_endpoint() -> String {
        std::env::var("SONGBIRD_ENDPOINT").unwrap_or_else(|_| {
            let bind_addr = crate::config::constants::get_bind_address();
            let port = std::env::var("SONGBIRD_ORCHESTRATOR_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080);
            format!("http://{bind_addr}:{port}")
        })
    }

    /// Get service endpoint by capability instead of hardcoded primal names
    #[must_use]
    pub fn service_endpoint_by_capability(capability_type: &str, default_port: u16) -> String {
        std::env::var("SONGBIRD_ENDPOINT")
            .or_else(|_| env::var(format!("{capability_type}_ENDPOINT")))
            .unwrap_or_else(|_| format!("http://localhost:{default_port}"))
    }

    /// Get the ToadStool compute endpoint from environment or calculate from config
    #[must_use]
    pub fn toadstool_endpoint() -> String {
        std::env::var("TOADSTOOL_ENDPOINT").unwrap_or_else(|_| {
            let bind_addr = crate::config::constants::get_bind_address();
            let port = std::env::var("TOADSTOOL_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8081);
            format!("http://{bind_addr}:{port}")
        })
    }

    /// Get the NestGate storage endpoint from environment or calculate from config
    #[must_use]
    pub fn nestgate_endpoint() -> String {
        std::env::var("NESTGATE_ENDPOINT").unwrap_or_else(|_| {
            let bind_addr = crate::config::constants::get_bind_address();
            let port = std::env::var("NESTGATE_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8082);
            format!("http://{bind_addr}:{port}")
        })
    }

    /// Get the Squirrel AI endpoint from environment or calculate from config
    #[must_use]
    pub fn squirrel_endpoint() -> String {
        std::env::var("SQUIRREL_ENDPOINT").unwrap_or_else(|_| {
            let bind_addr = crate::config::constants::get_bind_address();
            let port = std::env::var("SQUIRREL_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8083);
            format!("http://{bind_addr}:{port}")
        })
    }

    /// Get the BearDog security endpoint from environment or calculate from config
    #[must_use]
    pub fn beardog_endpoint() -> String {
        std::env::var("BEARDOG_ENDPOINT").unwrap_or_else(|_| {
            let bind_addr = crate::config::constants::get_bind_address();
            let port = std::env::var("BEARDOG_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8443);
            let protocol = if port == 8443 { "https" } else { "http" };
            format!("{protocol}://{bind_addr}:{port}")
        })
    }

    /// Get bind address for services
    #[must_use]
    pub fn bind_address() -> String {
        crate::config::constants::get_bind_address()
    }

    /// Get security providers from environment
    #[must_use]
    pub fn security_providers() -> Vec<String> {
        env::var("SECURITY_PROVIDERS")
            .unwrap_or_else(|_| Self::beardog_endpoint())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect()
    }

    /// Get storage providers from environment
    #[must_use]
    pub fn storage_providers() -> Vec<String> {
        env::var("STORAGE_PROVIDERS")
            .unwrap_or_else(|_| Self::nestgate_endpoint())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect()
    }

    /// Get compute providers from environment
    #[must_use]
    pub fn compute_providers() -> Vec<String> {
        env::var("COMPUTE_PROVIDERS")
            .unwrap_or_else(|_| Self::toadstool_endpoint())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect()
    }

    /// Get AI providers from environment
    #[must_use]
    pub fn ai_providers() -> Vec<String> {
        env::var("AI_PROVIDERS")
            .unwrap_or_else(|_| Self::squirrel_endpoint())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect()
    }

    /// Get orchestrator providers from environment
    #[must_use]
    pub fn orchestrator_providers() -> Vec<String> {
        env::var("ORCHESTRATOR_PROVIDERS")
            .unwrap_or_else(|_| Self::songbird_endpoint())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect()
    }

    /// Get ecosystem configuration from environment
    #[must_use]
    pub fn ecosystem_config() -> HashMap<String, String> {
        let mut config = HashMap::new();

        config.insert("songbird".to_string(), Self::songbird_endpoint());
        // 🚀 CANONICAL MODERNIZATION: Dynamic primal discovery
        // Instead of hardcoded primal names, discover from environment
        config.extend(Self::discover_capability_providers());

        // Legacy compatibility - only add if not already discovered
        if !config.contains_key("compute") && !config.contains_key("toadstool") {
            config.insert(
                "compute".to_string(),
                Self::get_capability_endpoint("compute", "toadstool"),
            );
            config.insert("toadstool".to_string(), Self::toadstool_endpoint());
        }
        if !config.contains_key("security") && !config.contains_key("beardog") {
            config.insert(
                "security".to_string(),
                Self::get_capability_endpoint("security", "beardog"),
            );
            config.insert("beardog".to_string(), Self::beardog_endpoint());
        }
        if !config.contains_key("storage") && !config.contains_key("nestgate") {
            config.insert(
                "storage".to_string(),
                Self::get_capability_endpoint("storage", "nestgate"),
            );
            config.insert("nestgate".to_string(), Self::nestgate_endpoint());
        }
        if !config.contains_key("ai") && !config.contains_key("squirrel") {
            config.insert(
                "ai".to_string(),
                Self::get_capability_endpoint("ai", "squirrel"),
            );
            config.insert("squirrel".to_string(), Self::squirrel_endpoint());
        }

        config
    }

    /// Get all endpoints by capability type (preferred over hardcoded primal names)
    #[must_use]
    pub fn get_all_endpoints() -> HashMap<String, String> {
        let mut endpoints = HashMap::new();

        // Use universal capability-based discovery instead of hardcoded primal names
        endpoints.insert("storage".to_string(), Self::nestgate_endpoint());
        endpoints.insert("compute".to_string(), Self::toadstool_endpoint());
        endpoints.insert("orchestration".to_string(), Self::songbird_endpoint());
        endpoints.insert("ai".to_string(), Self::squirrel_endpoint());

        // Security capabilities discovered dynamically via universal adapter
        // No hardcoded "beardog" - let capability discovery find security providers
        if let Ok(security_endpoint) = env::var("SECURITY_ENDPOINT") {
            endpoints.insert("security".to_string(), security_endpoint);
        }

        endpoints
    }

    /// 🚀 CANONICAL MODERNIZATION: Discover capability providers dynamically
    fn discover_capability_providers() -> HashMap<String, String> {
        let mut providers = HashMap::new();

        // Discover from environment variables
        for i in 1..=20 {
            let name_env = format!("CAPABILITY_PROVIDER_{i}_NAME");
            let endpoint_env = format!("CAPABILITY_PROVIDER_{i}_ENDPOINT");
            let caps_env = format!("CAPABILITY_PROVIDER_{i}_CAPABILITIES");

            if let (Ok(name), Ok(endpoint), Ok(capabilities)) = (
                std::env::var(&name_env),
                std::env::var(&endpoint_env),
                std::env::var(&caps_env),
            ) {
                // Register provider by each capability it provides
                for capability in capabilities.split(',').map(|s| s.trim()) {
                    providers.insert(capability.to_string(), endpoint.clone());
                }
                // Also register by provider name for backward compatibility
                providers.insert(name, endpoint);
            }
        }

        // Check for specific capability endpoints
        if let Ok(security_endpoint) = std::env::var("SECURITY_PROVIDER_ENDPOINT") {
            providers.insert("security".to_string(), security_endpoint);
        }
        if let Ok(compute_endpoint) = std::env::var("COMPUTE_PROVIDER_ENDPOINT") {
            providers.insert("compute".to_string(), compute_endpoint);
        }
        if let Ok(storage_endpoint) = std::env::var("STORAGE_PROVIDER_ENDPOINT") {
            providers.insert("storage".to_string(), storage_endpoint);
        }
        if let Ok(ai_endpoint) = std::env::var("AI_PROVIDER_ENDPOINT") {
            providers.insert("ai".to_string(), ai_endpoint);
        }

        providers
    }

    /// Get endpoint for a capability with fallback to legacy primal name
    fn get_capability_endpoint(capability: &str, legacy_primal: &str) -> String {
        // First try capability-based discovery
        let capability_env = format!("{}_PROVIDER_ENDPOINT", capability.to_uppercase());
        if let Ok(endpoint) = std::env::var(&capability_env) {
            return endpoint;
        }

        // Fallback to legacy primal-based lookup
        match legacy_primal {
            "toadstool" => Self::toadstool_endpoint(),
            "beardog" => Self::beardog_endpoint(),
            "nestgate" => Self::nestgate_endpoint(),
            "squirrel" => Self::squirrel_endpoint(),
            _ => format!("http://{legacy_primal}:8080"), // Generic fallback
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endpoint_generation() {
        // Test defaults when no environment variables are set
        assert!(EnvironmentConfig::songbird_endpoint().contains("8080"));
        assert!(
            EnvironmentConfig::service_endpoint_by_capability("beardog", 8443).contains("8443")
        );
        assert!(EnvironmentConfig::nestgate_endpoint().contains("8082"));
        assert!(EnvironmentConfig::toadstool_endpoint().contains("8081"));
    }

    #[test]
    fn test_capability_based_endpoints() {
        let endpoints = EnvironmentConfig::get_all_endpoints();
        assert!(endpoints.contains_key("storage"));
        assert!(endpoints.contains_key("compute"));
        assert!(endpoints.contains_key("orchestration"));
        assert!(endpoints.contains_key("ai"));
    }

    #[test]
    fn test_provider_lists() {
        let security_providers = EnvironmentConfig::security_providers();
        assert!(!security_providers.is_empty());

        let storage_providers = EnvironmentConfig::storage_providers();
        assert!(!storage_providers.is_empty());

        let compute_providers = EnvironmentConfig::compute_providers();
        assert!(!compute_providers.is_empty());
    }
}
