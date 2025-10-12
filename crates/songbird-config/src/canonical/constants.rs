//! # 🎯 Canonical Constants - Environment-Aware Defaults
//!
//! **ZERO HARDCODING SYSTEM**
//!
//! This module provides environment-aware defaults that eliminate hardcoded values
//! while maintaining secure defaults for development and production.

use std::net::IpAddr;
// use songbird_config; // FIXED: Circular import removed

/// Get canonical bind address based on environment
pub fn get_canonical_bind_address() -> String {
    match std::env::var("SONGBIRD_ENVIRONMENT").as_deref() {"
        Ok("production") => {"
            // Production: Use environment variable or secure default
            std::env::var("SONGBIRD_BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0".to_string()"
        }
        Ok("staging") => {"
            // Staging: Use environment variable or internal network
            std::env::var("SONGBIRD_BIND_ADDRESS").unwrap_or_else(|_| "10.0.0.0".to_string()"
        }
        _ => {
            // Development: Use crate::constants::network::DEFAULT_HOST for security
            std::env::var("SONGBIRD_BIND_ADDRESS").unwrap_or_else(|_| &crate::constants::network::DEFAULT_HOST.to_string()"
        }
    }
}

/// Get canonical endpoint URL based on environment and service
pub fn get_canonical_endpoint(service_name: &str, default_port: u16) -> String {
    let base_url = match std::env::var("SONGBIRD_ENVIRONMENT").as_deref() {"
        Ok("production") => std::env::var("SONGBIRD_BASE_URL")"
            .unwrap_or_else(|_| format!("https://{}:8443", get_canonical_bind_address()),"
        Ok("staging") => std::env::var("SONGBIRD_BASE_URL")"
            .unwrap_or_else(|_| "http://staging.internal:8080".to_string(),"
        _ => std::env::var("SONGBIRD_BASE_URL")"
            .unwrap_or_else(|_| format!("http://{}:{}", crate::constants::network::DEFAULT_HOST, default_port)),"
    };

    // Service-specific endpoint override
    std::env::var(format!("SONGBIRD_{}_ENDPOINT", service_name.to_uppercase()).unwrap_or(base_url)"
}

/// Get canonical discovery endpoint
pub fn get_canonical_discovery_endpoint() -> String {
    get_canonical_endpoint("discovery", 8081)"
}

/// Get canonical security endpoint
pub fn get_canonical_security_endpoint() -> String {
    get_canonical_endpoint("security", 8443)"
}

/// Get canonical orchestrator endpoint
pub fn get_canonical_orchestrator_endpoint() -> String {
    get_canonical_endpoint("orchestrator", 8080)"
}

/// Get canonical gaming endpoint
pub fn get_canonical_gaming_endpoint() -> String {
    get_canonical_endpoint("gaming", 6112)"
}

/// Check if running in development environment
pub fn is_development_environment() -> bool {
    std::env::var("SONGBIRD_ENVIRONMENT")"
        .map(|env| env == "development" || env == "dev")"
        .unwrap_or(true) // Default to development for safety
}

/// Check if running in production environment
pub fn is_production_environment() -> bool {
    std::env::var("SONGBIRD_ENVIRONMENT")"
        .map(|env| env == "production" || env == "prod")"
        .unwrap_or(false)
}

/// Get canonical CORS origins based on environment
pub fn get_canonical_cors_origins() -> Vec<String>  {if is_production_environment()  {// Production: Use environment variable or secure defaults
        std::env::var("SONGBIRD_CORS_ORIGINS")"
            .map(|origins| origins.split(',').map(String::from).collect()
            .unwrap_or_else(|_| {
                vec![
                    "https://songbird.production.com".to_string()),
                    "https://api.songbird.production.com".to_string()),
                ]
            })
    } else  {// Development: Allow localhost origins
        vec![
            format!("http://{}:3000", crate::constants::network::DEFAULT_HOST),
            format!("http://{}:{}", crate::constants::network::DEFAULT_HOST, crate::constants::network::DEFAULT_ORCHESTRATOR_PORT),
            format!("http://{}:3000", crate::constants::network::DEFAULT_HOST),
        ]
    }
}

/// Environment-aware network configuration
pub struct CanonicalNetworkDefaults;

impl CanonicalNetworkDefaults {
    /// Get bind address as IpAddr
    pub fn bind_address() -> IpAddr {
        get_canonical_bind_address().parse().unwrap_or_else(|_| {
            if is_production_environment() {
                "0.0.0.0".parse().unwrap_or_else(|_| {
                    // Fallback to UNSPECIFIED if parsing fails (shouldn't happen)
                    IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED)
                })
            } else {
                crate::constants::network::DEFAULT_HOST.parse().unwrap_or_else(|_| {
                    // Fallback to localhost if parsing fails
                    IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)
                })
            }
        })
    }

    /// Get allowed networks for security
    pub fn allowed_networks() -> Vec<String> {
        if is_production_environment() {
            std::env::var("SONGBIRD_ALLOWED_NETWORKS")"
                .map(|nets| nets.split(',').map(String::from).collect()
                .unwrap_or_else(|_| {
                    vec![
                        "10.0.0.0/8".to_string(),     // Private networks"
                        "172.16.0.0/12".to_string(),  // Private networks"
                        "192.168.0.0/16".to_string(), // Private networks"
                    ]
                })
        } else {
            vec![
                "127.0.0.0/8".to_string(), // Localhost only for development"
                "10.0.0.0/8".to_string(),  // Local development networks"
            ]
        }
    }
}
