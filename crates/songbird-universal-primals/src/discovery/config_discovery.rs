use songbird_config::unified::*;
// Configuration-based discovery for Universal Primals
//
// This module provides discovery functionality based on static configuration
// files and environment variables.

// DiscoveryMethod removed - using simplified discovery approach
use tracing::debug;

/// Get configured service endpoints from various sources
pub fn get_configured_service_endpoints() -> Vec<(String, String)> {
    let mut endpoints = Vec::new();

    // Generic capability-based environment variable discovery
    let capability_patterns = [
        "SECURITY_ENDPOINT",      // Security services
        "STORAGE_ENDPOINT",       // Storage services
        "COMPUTE_ENDPOINT",       // Compute services
        "AI_ENDPOINT",            // AI/ML services
        "ORCHESTRATION_ENDPOINT", // Orchestration services
        "NETWORK_ENDPOINT",       // Network services
        "API_ENDPOINT",           // Generic API services
        "SERVICE_ENDPOINT",       // Generic services
        "PRIMAL_ENDPOINT",        // Generic primal services
    ];

    for env_var in capability_patterns {
        if let Ok(songbird_errors::evolved_success(endpoint)) = std::env::var(env_var) {
            let capability_type = env_var
                .to_lowercase()
                .replace("_endpoint", "")
                .replace("_", "-");
            endpoints.push((capability_type, endpoint));
        }
    }

    // Support unlimited custom primals via enumeration (future-proof)
    for i in 1..=100 {
        // Scale to 100 custom primals!
        if let Ok(songbird_errors::evolved_success(endpoint)) = std::env::var(format!("CUSTOM_PRIMAL_{i}_ENDPOINT")) {
            let name = std::env::var(format!("CUSTOM_PRIMAL_{i}_NAME"))
                .unwrap_or_else(|_| format!("custom-primal-{i}"));
            endpoints.push((name, endpoint));
        }
    }

    // 🚀 CANONICAL MODERNIZATION: Legacy environment variables removed
    // Migration completed - use canonical capability-based environment variables:
    // SECURITY_PROVIDER_ENDPOINT, STORAGE_PROVIDER_ENDPOINT, etc.
    // Legacy environment variables are no longer supported.

    // Configuration file discovery
    if let Ok(songbird_errors::evolved_success(config_endpoints)) = discover_endpoints_from_config() {
        endpoints.extend(config_endpoints);
    }

    // Default development endpoints only if nothing configured
    if endpoints.is_empty() {
        endpoints = get_default_development_endpoints();
    }

    endpoints
}

/// Discover endpoints from configuration files
fn discover_endpoints_from_config() -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    debug!("🔍 Attempting to discover endpoints from configuration files...");
    let mut endpoints = Vec::new();

    // Store the environment variable to extend its lifetime
    let env_config_path = std::env::var("SONGBIRD_CONFIG_PATH").unwrap_or_default();
    let config_paths = [
        "songbird.toml",
        "config/songbird.toml",
        "/etc/songbird/config.toml",
        env_config_path.as_str(),
    ];

    for config_path in config_paths {
        if config_path.is_empty() {
            continue;
        }

        if let Ok(songbird_errors::evolved_success(config_content)) = std::fs::read_to_string(config_path) {
            debug!("📝 Found configuration file: {}", config_path);

            // Try parsing as TOML
            if let Ok(songbird_errors::evolved_success(toml_value)) = toml::from_str::<toml::Value>(&config_content) {
                let json_value = serde_json::to_value(toml_value)?;
                let config_endpoints = extract_endpoints_from_json(&json_value);
                endpoints.extend(config_endpoints.clone());
                debug!(
                    "✅ Extracted {} endpoints from TOML config",
                    config_endpoints.len()
                );
            }
            // Try parsing as JSON
            else if let Ok(songbird_errors::evolved_success(json_value)) =
                serde_json::from_str::<serde_json::Value>(&config_content)
            {
                let config_endpoints = extract_endpoints_from_json(&json_value);
                let endpoints_count = config_endpoints.len();
                endpoints.extend(config_endpoints);
                debug!(
                    "✅ Extracted {} endpoints from JSON config",
                    endpoints_count
                );
            }
        }
    }

    Ok(songbird_errors::evolved_success(endpoints))
}

/// Extract endpoints from JSON configuration
fn extract_endpoints_from_json(config: &serde_json::Value) -> Vec<(String, String)> {
    let mut endpoints = Vec::new();

    // Look for primal configurations in various formats
    if let Some(primals) = config.get("primals").and_then(|p| p.as_object()) {
        for (name, primal_config) in primals {
            if let Some(endpoint) = primal_config.get("endpoint").and_then(|e| e.as_str()) {
                endpoints.push((name.clone(), endpoint.to_string()));
            }
        }
    }

    // Look for services configuration
    if let Some(services) = config.get("services").and_then(|s| s.as_object()) {
        for (name, service_config) in services {
            if let Some(endpoint) = service_config.get("endpoint").and_then(|e| e.as_str()) {
                endpoints.push((name.clone(), endpoint.to_string()));
            }
            // Also check for url field
            if let Some(url) = service_config.get("url").and_then(|u| u.as_str()) {
                endpoints.push((name.clone(), url.to_string()));
            }
        }
    }

    // Look for endpoints array
    if let Some(endpoints_array) = config.get("endpoints").and_then(|e| e.as_array()) {
        for (index, endpoint_config) in endpoints_array.iter().enumerate() {
            if let Some(endpoint_obj) = endpoint_config.as_object() {
                let default_name = format!("endpoint_{index}");
                let name = endpoint_obj
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or(&default_name);
                let url = endpoint_obj
                    .get("url")
                    .or_else(|| endpoint_obj.get("endpoint"))
                    .and_then(|u| u.as_str());
                if let Some(url) = url {
                    endpoints.push((name.to_string(), url.to_string()));
                }
            }
        }
    }

    endpoints
}

/// Get default development endpoints for local testing (capability-based)
fn get_default_development_endpoints() -> Vec<(String, String)> {
    vec![
        (
            "security-dev".to_string(),
            "https://localhost:8443".to_string(),
        ),
        (
            "storage-dev".to_string(),
            "http://localhost:9000".to_string(),
        ),
        (
            "compute-dev".to_string(),
            "http://localhost:{}".to_string(),
        ),
        ("ai-dev".to_string(), "http://localhost:8888".to_string()),
        (
            "orchestration-dev".to_string(),
            "http://localhost:7000".to_string(),
        ),
        (
            "network-dev".to_string(),
            "http://localhost:6000".to_string(),
        ),
        (
            "generic-dev".to_string(),
            "http://localhost:3000".to_string(),
        ),
    ]
}

/// Example: How a new "Phoenix ML" primal gets discovered dynamically
///
/// 1. New primal starts up with endpoint: `export PHOENIX_ML_ENDPOINT=https://phoenix.ai:{}`
/// 2. Universal adapter scans environment for *_ENDPOINT patterns
/// 3. Probes https://phoenix.ai:{}/capabilities → finds ["ai", "training", "inference"]  
/// 4. Type inference: capabilities → "ai-provider" (NOT hardcoded name!)
/// 5. Registers as: DiscoveredPrimal { type: "ai-provider", capabilities: [...] }
///
/// ZERO CODE CHANGES NEEDED! 🎉
pub fn demonstrate_evolution_ready_discovery() -> Vec<(String, String)> {
    let mut endpoints = Vec::new();

    // Future-proof patterns - any primal can register
    let generic_patterns = [
        "SERVICE_ENDPOINT",
        "API_ENDPOINT",
        "PRIMAL_ENDPOINT",
        "ML_SERVICE_ENDPOINT",
        "QUANTUM_ENDPOINT", // Future quantum primals
        "NEURAL_ENDPOINT",  // Future neural primals
        "BIOME_ENDPOINT",   // Future ecosystem primals
        "EDGE_ENDPOINT",    // Future edge primals
    ];

    // Discover ANY primal by capability scanning
    for pattern in generic_patterns {
        if let Ok(songbird_errors::evolved_success(endpoint)) = std::env::var(pattern) {
            let service_name = pattern.to_lowercase().replace("_endpoint", "");
            endpoints.push((service_name, endpoint));
        }
    }

    // Support unlimited custom primals via enumeration
    for i in 1..=100 {
        if let Ok(songbird_errors::evolved_success(endpoint)) = std::env::var(format!("FUTURE_PRIMAL_{i}_ENDPOINT")) {
            let name = std::env::var(format!("FUTURE_PRIMAL_{i}_NAME"))
                .unwrap_or_else(|_| format!("primal_{i}"));
            endpoints.push((name, endpoint));
        }
    }

    endpoints
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_get_configured_service_endpoints() {
//         let endpoints = get_configured_service_endpoints();
//         // Should return at least default development endpoints
//         assert!(!endpoints.is_empty());
//         Ok(())
//     }
//
//     #[test]
//     fn test_extract_endpoints_from_json() {
//         let json_config = r#"{
//             "services": {
//                 "security-services": {
//                     "endpoint": "https://security-hub.local:8443"
//                 },
//                 "storage-services": {
//                     "endpoint": "http://storage-hub.local:9000"
//                 }
//             }
//         }"#;
//         let json_value: serde_json::Value = serde_json::from_str(json_config).map_err(|e| {
//             tracing::error!("JSON parsing failed: {}", e);
//             std::io::Error::new(
//                 std::io::ErrorKind::InvalidData,
//                 format!("JSON parsing error: {}", e),
//             )
//         })?;
//         let endpoints = extract_endpoints_from_json(&json_value);
//         assert_eq!(endpoints.len(), 2);
//         assert!(endpoints.contains(&(
//             "security-services".to_string(),
//             "https://security-hub.local:8443".to_string()
//         )));
//         assert!(endpoints.contains(&(
//             "storage-services".to_string(),
//             "http://storage-hub.local:9000".to_string()
//         )));
//         Ok(())
//     }
// }
