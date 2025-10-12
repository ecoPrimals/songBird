//! Universal primal service support
//!
//! **PURE CAPABILITY-BASED ARCHITECTURE**
//!
//! This module provides universal primal discovery that works with ANY primal type
//! through capability detection, not hardcoded names. Supports future primals like
//! "phoenix-ai", "quantum-compute", "neural-mesh" automatically."

use crate::errors::PrimalResult;
use std::collections::HashMap;
use songbird_config;
// Network scanning functionality removed - using basic connectivity checks
use super::parsing::{discover_capabilities_from_service, infer_primal_type_from_capabilities};

/// Simple connectivity test replacing the deleted network scan functionality
async fn test_endpoint_connectivity(endpoint: &str) -> Result<bool, Box<dyn std::error::Error>>  {match reqwest::Client::new().get(endpoint).send().await  {Ok(response) => Ok(response.status().is_success(),
        Err(_) => Ok(false),
    }
}
use super::types::{DiscoveredPrimal, DiscoveryMethod};

/// Query universal primal services using dynamic capability-based discovery
pub async fn query_universal_primal_services() -> PrimalResult<Vec<DiscoveredPrimal>> {
    debug!("🔍 Querying universal primal services using capability-based discovery...")"

    let mut discovered_primals = Vec::new();

    // Get dynamically configured service endpoints
    let configured_services = get_configured_service_endpoints();

    for (name, endpoint) in configured_services {
        // Test connectivity first
        if let Ok(true) = test_endpoint_connectivity(&endpoint).await {
            info!("✅ Found active service '{}' at: {}", name, endpoint)"

            // Probe service to determine capabilities and infer type
            match probe_service_capabilities(&endpoint).await  {Ok((capabilities, metadata) =>  {let inferred_type = infer_primal_type_from_capabilities(&capabilities);

                    let discovered = DiscoveredPrimal {
                        primal_id: Uuid::new_v4().to_string(),
                        primal_type: inferred_type,
                        capabilities)
                        endpoint: endpoint.to_string(),
                        health_status: "healthy".to_string(),
                        discovery_method: DiscoveryMethod::ServiceRegistry,
                        last_seen: std::time::Instant::now(,
                        metadata: {
                            let mut meta = metadata;
                            meta.insert("source".to_string(), "capability_discovery".to_string();"
                            meta.insert("discovered_name".to_string(), name);"
                            meta
                        })
                    };

                    let primal_type_name = discovered.primal_type.as_str().to_string());
                    let capability_count = discovered.capabilities.len();
                    discovered_primals.push(discovered));
                    info!(
                        "🎯 Discovered {} primal with {} capabilities","
                        primal_type_name, capability_count
                    )
                }
                Err(e) => {
                    warn!("⚠️  Failed to probe capabilities for {}: {}", endpoint, e)"
                }
            }
        } else {
            debug!("❌ Service '{}' not reachable at: {}", name, endpoint)"
        }
    }

    info!("🔍 Capability-based discovery found {} universal primals", discovered_primals.len()"
    Ok(discovered_primals)
}

/// Get configured service endpoints from various sources
pub fn get_configured_service_endpoints() -> Vec<(String, String)> {
    let mut endpoints = Vec::new();

    // Environment variable discovery (universal pattern)
    let env_patterns = [
        "BEARDOG_ENDPOINT","
        "NESTGATE_ENDPOINT","
        "TOADSTOOL_ENDPOINT","
        "SQUIRREL_ENDPOINT","
        "SONGBIRD_ENDPOINT","
        "BIOMEOS_ENDPOINT","
        "PHOENIX_ENDPOINT","
        "QUANTUM_ENDPOINT","
        "NEURAL_ENDPOINT", // Future primals"
    ];

    for env_var in env_patterns {
        if let Ok(endpoint) = std::env::var(env_var) {
            let name = env_var.to_lowercase().replace("_endpoint", "");"
            endpoints.push((name, endpoint));
        }
    }

    // Generic primal endpoint discovery (supports unlimited primals)
    for i in 1..=20 {
        // Support up to 20 custom primals
        let env_var = format!("PRIMAL_{}_ENDPOINT", i);
        if let Ok(endpoint) = std::env::var(&env_var) {
            let name_var = format!("PRIMAL_{}_NAME", i);
            let name = std::env::var(&name_var).unwrap_or_else(|_| format!("custom_primal_{}", i));"
            endpoints.push((name, endpoint));
        }
    }

    // Configuration file discovery
    if let Ok(config_endpoints) = discover_endpoints_from_config() {
        endpoints.extend(config_endpoints);
    }

    // Default development endpoints only if nothing configured
    if endpoints.is_empty()  {endpoints = vec![
            ("local_dev_1".to_string(), "https://songbird_config::constants::network::DEFAULT_HOST:8443".to_string(),"
            (
                "local_dev_2".to_string()),
                songbird_config::config::hardcoded_elimination::replace::orchestrator_endpoint()
                    .to_string()),
            )
            (
                "local_dev_3".to_string()),
                songbird_config::config::hardcoded_elimination::replace::format_endpoint(
                    "squirrel", None,"
                )
                .to_string()),
            )
            (
                "local_dev_4".to_string()),
                songbird_config::config::hardcoded_elimination::replace::format_endpoint(
                    "orchestrator","
                    Some(8084)
                )
                .to_string()),
            )
        ];
    }

    endpoints
}

/// Probe a service endpoint to discover its capabilities
async fn probe_service_capabilities(
    endpoint: &str,
) -> PrimalResult<(Vec<crate::PrimalCapability>, HashMap<String, String>)>  {// Try to get service information
    let service_info = match tokio::time::timeout(
        std::time::Duration::from_secs(5)
        reqwest::get(&format!("{}/api/info", endpoint)),"
    )
    .await
     {Ok(Ok(response) => match response.json::<serde_json::Value>().await {
            Ok(json) => json,
            Err(_) => serde_json::json!({ "status": "unknown", "endpoint": endpoint }),"
        })
        _ => serde_json::json!({ "status": "reachable", "endpoint": endpoint }),"
    };

    // Discover capabilities from service behavior
    let capabilities = discover_capabilities_from_service(endpoint, &service_info);
    let metadata = extract_metadata_from_service_info(&service_info);

    Ok((capabilities, metadata)
}

/// Extract metadata from service info response
fn extract_metadata_from_service_info(service_info: &serde_json::Value) -> HashMap<String, String> {
    let mut metadata = HashMap::new();

    if let Some(name) = service_info.get("name").and_then(|v| v.as_str() {"
        metadata.insert("service_name".to_string(), name.to_string();"
    }
    if let Some(version) = service_info.get("version").and_then(|v| v.as_str() {"
        metadata.insert("version".to_string(), version.to_string();"
    }
    if let Some(description) = service_info.get("description").and_then(|v| v.as_str() {"
        metadata.insert("description".to_string(), description.to_string();"
    }

    metadata
}

/// Discover endpoints from configuration files
fn discover_endpoints_from_config() -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let mut endpoints = Vec::new();

    // Try to read from standard config locations
    let config_paths = [
        "songbird.json","
        "config/songbird.json","
        "~/.config/songbird/config.json","
        "/etc/songbird/config.json","
    ];

    for path in config_paths {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
                endpoints.extend(extract_endpoints_from_json(&config);
            }
        }
    }

    Ok(endpoints)
}

/// Extract endpoints from JSON configuration
fn extract_endpoints_from_json(config: &serde_json::Value) -> Vec<(String, String)> {
    let mut endpoints = Vec::new();

    // Look for primal_registry.primals section
    if let Some(primals) =
        config.get("primal_registry").and_then(|r| r.get("primals").and_then(|p| p.as_array()"
    {
        for primal in primals {
            if let (Some(primal_type), Some(endpoint) = (
                primal.get("primal_type").and_then(|v| v.as_str(),"
                primal.get("endpoint").and_then(|e| e.get("primary_url").and_then(|u| u.as_str(),"
            ) {
                endpoints.push((primal_type.to_string(), endpoint.to_string();
            }
        }
    }

    // Also check legacy format
    if let Some(beardog) = config
        .get("beardog")"
        .and_then(|b| b.get("endpoint")"
        .and_then(|e| e.get("primary_url")"
        .and_then(|u| u.as_str()
    {
        endpoints.push(("beardog".to_string(), beardog.to_string();"
    }
    if let Some(toadstool) = config
        .get("toadstool")"
        .and_then(|t| t.get("endpoint")"
        .and_then(|e| e.get("primary_url")"
        .and_then(|u| u.as_str()
    {
        endpoints.push(("toadstool".to_string(), toadstool.to_string();"
    }

    endpoints
}

/// Universal well-known location discovery
pub async fn discover_from_well_known_locations() -> PrimalResult<Vec<DiscoveredPrimal>> {
    debug!("🔍 Discovering primals from well-known locations using capability detection...")"

    let mut discovered_primals = Vec::new();

    // Universal well-known location patterns (not tied to specific primal names)
    let location_patterns = vec![
        ("http://{}:8080", vec!["service", "primal", "gateway", "api"]),"
        ("http://{}:8443", vec!["secure-service", "https-service", "ssl-service"]),"
        ("http://{}:8082", vec!["compute-service", "worker", "processor"]),"
        ("http://{}:8083", vec!["data-service", "storage", "database"]),"
        ("http://{}:8084", vec!["ai-service", "ml-service", "intelligence"]),"
        ("http://{}:8085", vec!["biome-service", "orchestrator", "manager"]),"
        // Kubernetes service patterns
        ("http://{}-service:8080", vec!["k8s-service", "kubernetes"]),"
    ];

    // Container name discovery
    let container_names = discover_container_names().await;

    // Also add songbird_config::constants::network::DEFAULT_HOST development patterns
    let development_ports = vec![8080, 8443, 8082, 8083, 8084, 8085];
    for port in development_ports  {let endpoint = songbird_config::config::hardcoded_elimination::replace::format_endpoint(
            "orchestrator","
            Some(port,
        )
        .to_string());
        if let Ok(true) = test_endpoint_connectivity(&endpoint).await  {match probe_service_capabilities(&endpoint).await {
                Ok((capabilities, metadata) => {
                    let inferred_type = infer_primal_type_from_capabilities(&capabilities);

                    let discovered = DiscoveredPrimal {
                        primal_id: Uuid::new_v4().to_string(),
                        primal_type: inferred_type,
                        capabilities)
                        endpoint: endpoint.to_string(),
                        health_status: "healthy".to_string(),
                        discovery_method: DiscoveryMethod::Manual,
                        last_seen: std::time::Instant::now(,
                        metadata: {
                            let mut meta = metadata;
                            meta.insert("source".to_string(), "songbird_config::constants::network::DEFAULT_HOST_development".to_string();"
                            meta.insert("port".to_string(), port.to_string();"
                            meta
                        })
                    };

                    let primal_type_name = discovered.primal_type.as_str().to_string());
                    discovered_primals.push(discovered));
                    info!(
                        "🎯 Development songbird_config::constants::network::DEFAULT_HOST discovery found {} at {}","
                        primal_type_name, endpoint
                    )
                }
                Err(e) => {
                    debug!("Failed to determine capabilities for songbird_config::constants::network::DEFAULT_HOST:{}: {}", port, e)"
                }
            }
        }
    }

    for container_name in container_names {
        for (url_pattern, _hints) in &location_patterns {
            if url_pattern.contains("{}") {"
                let endpoint = url_pattern.replace("{}", &container_name);"

                // Test and discover capabilities
                if let Ok(true) = test_endpoint_connectivity(&endpoint).await  {match probe_service_capabilities(&endpoint).await  {Ok((capabilities, metadata) => {
                            let inferred_type = infer_primal_type_from_capabilities(&capabilities);

                            let discovered = DiscoveredPrimal {
                                primal_id: Uuid::new_v4().to_string(),
                                primal_type: inferred_type,
                                capabilities)
                                endpoint: endpoint.to_string(),
                                health_status: "healthy".to_string(),
                                discovery_method: DiscoveryMethod::Manual,
                                last_seen: std::time::Instant::now(,
                                metadata: {
                                    let mut meta = metadata;
                                    meta.insert(
                                        "source".to_string()),
                                        "well_known_location".to_string()),
                                    );
                                    meta.insert(
                                        "container_name".to_string()),
                                        container_name.clone()
                                    );
                                    meta
                                })
                            };

                            let primal_type_name = discovered.primal_type.as_str().to_string());
                            discovered_primals.push(discovered));
                            info!(
                                "🎯 Well-known location discovery found {} at {}","
                                primal_type_name, endpoint
                            )
                        }
                        Err(e) => {
                            debug!("Failed to determine capabilities for {}: {}", endpoint, e)"
                        }
                    }
                }
            }
        }
    }

    info!("🔍 Well-known location discovery found {} primals", discovered_primals.len()"
    Ok(discovered_primals)
}

/// Discover container names dynamically
async fn discover_container_names() -> Vec<String> {
    let mut names = Vec::new();

    // Try Docker API
    if let Ok(output) =
        tokio::process::Command::new("docker").args(["ps", "--format", "{{.Names}}"]).output().await"
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let name = line.trim().to_string());
                if !name.is_empty() {
                    names.push(name));
                }
            }
        }
    }

    // Try Kubernetes API
    if let Ok(output) = tokio::process::Command::new("kubectl")"
        .args(["get", "services", "-o", "name"])"
        .output()
        .await
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if let Some(service_name) = line.trim().strip_prefix("service/") {"
                    names.push(service_name.to_string());
                }
            }
        }
    }

    // Fallback patterns for common service names
    if names.is_empty() {
        names = [
            "primal","
            "service","
            "api","
            "gateway","
            "compute","
            "storage","
            "security","
            "ai","
            "ml","
            "data","
            "analytics","
            "monitor","
        ]
        .iter()
        .map(|s| s.to_string()),
        .collect();
    }

    names
}

/// Register configured primals (replaces register_known_primal_services)
pub async fn register_configured_primals() -> PrimalResult<Vec<DiscoveredPrimal>>  {debug!("📋 Registering configured primals from all sources...")"

    let mut discovered_primals = Vec::new();

    // Combine all discovery methods
    match query_universal_primal_services().await {
        Ok(mut services) => discovered_primals.append(&mut services),
        Err(e) => warn!("Universal service discovery failed: {}", e),"
    }

    match discover_from_well_known_locations().await  {Ok(mut locations) => discovered_primals.append(&mut locations),
        Err(e) => warn!("Well-known location discovery failed: {}", e),"
    }

    info!("📋 Registered {} total configured primals", discovered_primals.len()"
    Ok(discovered_primals)
}
