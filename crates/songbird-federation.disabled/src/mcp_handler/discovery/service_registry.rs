//! Service registry discovery (Consul/etcd) implementation

use songbird_errors::SongbirdError;
use std::time::Duration;
use tracing::{debug, info};

/// Discover federation endpoints via service registries
pub async fn discover_via_service_registry() -> Result<Vec<String>, SongbirdError> {
    info!("Starting service registry discovery for federation endpoints");

    let mut endpoints = Vec::new();

    // Check Consul
    if let Ok(consul_endpoints) = discover_from_consul().await {
        let current_len = endpoints.len();
        endpoints.extend(consul_endpoints);
        info!(
            "Consul discovery found {} endpoints",
            endpoints.len() - current_len
        );
    }

    // Check for etcd
    if let Ok(etcd_endpoints) = discover_from_etcd().await {
        let current_len = endpoints.len();
        endpoints.extend(etcd_endpoints);
        info!(
            "etcd discovery found {} endpoints",
            endpoints.len() - current_len
        );
    }

    // Check for other service registries
    if let Ok(other_endpoints) = discover_from_other_registries().await {
        let current_len = endpoints.len();
        endpoints.extend(other_endpoints);
        info!(
            "Other registry discovery found {} endpoints",
            endpoints.len() - current_len
        );
    }

    // Remove duplicates
    endpoints.sort();
    endpoints.dedup();

    info!(
        "Service registry discovery found {} unique endpoints",
        endpoints.len()
    );
    Ok(endpoints)
}

/// Discover from Consul service registry
pub async fn discover_from_consul() -> Result<Vec<String>, SongbirdError> {
    debug!("Querying Consul for Songbird federation services");

    let consul_endpoints = vec![
        "http://localhost:8500",
        "http://consul:8500",
        "http://consul.service.consul:8500",
        "http://127.0.0.1:8500",
    ];

    let mut discovered = Vec::new();

    for consul_url in consul_endpoints {
        match query_consul_services(consul_url).await {
            Ok(services) => {
                discovered.extend(services);
                debug!(
                    "Found {} services from Consul at {}",
                    discovered.len(),
                    consul_url
                );
            }
            Err(e) => {
                debug!("Failed to query Consul at {}: {}", consul_url, e);
            }
        }
    }

    Ok(discovered)
}

/// Query Consul for services
async fn query_consul_services(consul_url: &str) -> Result<Vec<String>, SongbirdError> {
    debug!("Querying Consul at {} for services", consul_url);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| {
            SongbirdError::service_error("discovery", format!("Failed to create HTTP client: {e}"))
        })?;

    let mut discovered = Vec::new();

    // Query Consul catalog for services
    let catalog_url = format!("{consul_url}/v1/catalog/services");

    match client.get(&catalog_url).send().await {
        Ok(response) if response.status().is_success() => {
            match response.json::<serde_json::Value>().await {
                Ok(services) => {
                    if let Some(services_obj) = services.as_object() {
                        // Look for Songbird-related services
                        for service_name in services_obj.keys() {
                            if service_name.contains("songbird")
                                || service_name.contains("federation")
                                || service_name.contains("primals")
                                || service_name.contains("mcp")
                            {
                                // Get service details
                                let service_url =
                                    format!("{consul_url}/v1/catalog/service/{service_name}");

                                match client.get(&service_url).send().await {
                                    Ok(detail_response)
                                        if detail_response.status().is_success() =>
                                    {
                                        match detail_response.json::<serde_json::Value>().await {
                                            Ok(service_details) => {
                                                discovered.extend(
                                                    extract_endpoints_from_consul_service(
                                                        &service_details,
                                                    ),
                                                );
                                            }
                                            Err(e) => {
                                                debug!(
                                                    "Failed to parse service details for {}: {}",
                                                    service_name, e
                                                );
                                            }
                                        }
                                    }
                                    Ok(_) => {
                                        debug!(
                                            "Service query returned non-success status for {}",
                                            service_name
                                        );
                                    }
                                    Err(e) => {
                                        debug!(
                                            "Failed to query service details for {}: {}",
                                            service_name, e
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    debug!("Failed to parse Consul services response: {}", e);
                }
            }
        }
        Ok(response) => {
            debug!("Consul returned non-success status: {}", response.status());
        }
        Err(e) => {
            debug!("Failed to query Consul catalog: {}", e);
        }
    }

    Ok(discovered)
}

/// Extract endpoints from Consul service details
fn extract_endpoints_from_consul_service(service_details: &serde_json::Value) -> Vec<String> {
    let mut endpoints = Vec::new();

    if let Some(instances) = service_details.as_array() {
        for instance in instances {
            if let (Some(address), Some(port)) = (
                instance.get("Address").and_then(|v| v.as_str()),
                instance.get("ServicePort").and_then(|v| v.as_u64()),
            ) {
                let endpoint = format!("http://{address}:{port}");
                endpoints.push(endpoint.clone());
                debug!("Found Consul service endpoint: {}", endpoint);
            } else if let (Some(address), Some(port)) = (
                instance.get("ServiceAddress").and_then(|v| v.as_str()),
                instance.get("ServicePort").and_then(|v| v.as_u64()),
            ) {
                let endpoint = format!("http://{address}:{port}");
                endpoints.push(endpoint.clone());
                debug!("Found Consul service endpoint (alt): {}", endpoint);
            }
        }
    }

    endpoints
}

/// Discover from etcd service registry
pub async fn discover_from_etcd() -> Result<Vec<String>, SongbirdError> {
    debug!("Querying etcd for Songbird federation services");

    let etcd_endpoints = vec![
        "http://localhost:2379",
        "http://etcd:2379",
        "http://127.0.0.1:2379",
        "http://localhost:2380",
        "http://etcd:2380",
    ];

    let mut discovered = Vec::new();

    for etcd_url in etcd_endpoints {
        match query_etcd_services(etcd_url).await {
            Ok(services) => {
                discovered.extend(services);
                debug!(
                    "Found {} services from etcd at {}",
                    discovered.len(),
                    etcd_url
                );
            }
            Err(e) => {
                debug!("Failed to query etcd at {}: {}", etcd_url, e);
            }
        }
    }

    Ok(discovered)
}

/// Query etcd for services
async fn query_etcd_services(etcd_url: &str) -> Result<Vec<String>, SongbirdError> {
    debug!("Querying etcd at {} for services", etcd_url);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| {
            SongbirdError::service_error("discovery", format!("Failed to create HTTP client: {e}"))
        })?;

    let mut discovered = Vec::new();

    // Query etcd keys for Songbird services
    let keys_to_check = vec![
        "/songbird/",
        "/federation/",
        "/primals/",
        "/services/songbird/",
        "/services/federation/",
        "/mcp/",
    ];

    for key_prefix in keys_to_check {
        let keys_url = format!("{etcd_url}/v2/keys{key_prefix}");

        match client.get(&keys_url).send().await {
            Ok(response) if response.status().is_success() => {
                match response.json::<serde_json::Value>().await {
                    Ok(etcd_response) => {
                        extract_endpoints_from_etcd_response(&etcd_response, &mut discovered);
                    }
                    Err(e) => {
                        debug!("Failed to parse etcd response for {}: {}", key_prefix, e);
                    }
                }
            }
            Ok(response) => {
                debug!(
                    "etcd returned status {} for key {}",
                    response.status(),
                    key_prefix
                );
            }
            Err(e) => {
                debug!("Failed to query etcd key {}: {}", key_prefix, e);
            }
        }
    }

    Ok(discovered)
}

/// Extract endpoints from etcd response
fn extract_endpoints_from_etcd_response(response: &serde_json::Value, endpoints: &mut Vec<String>) {
    if let Some(node) = response.get("node") {
        extract_endpoints_from_etcd_node(node, endpoints);
    }
}

/// Recursively extract endpoints from etcd node
fn extract_endpoints_from_etcd_node(node: &serde_json::Value, endpoints: &mut Vec<String>) {
    // Check if this node has a value that looks like an endpoint
    if let Some(value) = node.get("value").and_then(|v| v.as_str()) {
        if value.starts_with("http://") || value.starts_with("https://") {
            endpoints.push(value.to_string());
            debug!("Found etcd service endpoint: {}", value);
        } else if let Ok(parsed_value) = serde_json::from_str::<serde_json::Value>(value) {
            // Try to parse as JSON and extract endpoint info
            if let Some(endpoint) = parsed_value.get("endpoint").and_then(|v| v.as_str()) {
                endpoints.push(endpoint.to_string());
                debug!("Found etcd service endpoint (JSON): {}", endpoint);
            }
        }
    }

    // Recursively check child nodes
    if let Some(nodes) = node.get("nodes").and_then(|v| v.as_array()) {
        for child_node in nodes {
            extract_endpoints_from_etcd_node(child_node, endpoints);
        }
    }
}

/// Discover from other service registries
async fn discover_from_other_registries() -> Result<Vec<String>, SongbirdError> {
    let mut endpoints = Vec::new();

    // Try Kubernetes service discovery
    if let Ok(k8s_endpoints) = discover_from_kubernetes().await {
        endpoints.extend(k8s_endpoints);
    }

    // Try Docker Swarm service discovery
    if let Ok(swarm_endpoints) = discover_from_docker_swarm().await {
        endpoints.extend(swarm_endpoints);
    }

    Ok(endpoints)
}

/// Discover from Kubernetes service registry
async fn discover_from_kubernetes() -> Result<Vec<String>, SongbirdError> {
    debug!("Attempting Kubernetes service discovery");

    // This would typically use the kubernetes-client crate
    // For now, return empty as this is a complex implementation
    Ok(vec![])
}

/// Discover from Docker Swarm service registry
async fn discover_from_docker_swarm() -> Result<Vec<String>, SongbirdError> {
    debug!("Attempting Docker Swarm service discovery");

    // This would typically use the docker API
    // For now, return empty as this is a complex implementation
    Ok(vec![])
}
