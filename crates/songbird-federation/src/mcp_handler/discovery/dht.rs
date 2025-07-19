//! DHT-based discovery and network scanning implementation

use crate::config::FederationConfig;
use songbird_errors::SongbirdError;
use tracing::{debug, info};

use super::mdns::discover_via_mdns;
use super::network_utils::{
    get_federation_ports, get_local_subnets, scan_subnet_for_federation, verify_federation_endpoint,
};

/// Discover federation endpoints via DHT-like network scanning
pub async fn discover_via_dht(config: &FederationConfig) -> Result<Vec<String>, SongbirdError> {
    debug!("Starting DHT-like discovery for federation endpoints");

    let mut endpoints = Vec::new();

    // 1. Scan common federation ports on local network
    let local_subnets = get_local_subnets().await?;
    let federation_ports = get_federation_ports();

    for subnet in &local_subnets {
        let subnet_endpoints = scan_subnet_for_federation(subnet, &federation_ports).await?;
        let current_len = endpoints.len();
        endpoints.extend(subnet_endpoints);
        debug!(
            "Subnet {} scan found {} endpoints",
            subnet,
            endpoints.len() - current_len
        );
    }

    // 2. Check configured bootstrap nodes
    for bootstrap_node in &config.cluster_endpoints {
        if verify_federation_endpoint(bootstrap_node).await? {
            endpoints.push(bootstrap_node.clone());
            debug!("Verified bootstrap node: {}", bootstrap_node);
        }
    }

    // 3. Query known federation discovery services
    let discovery_endpoints = query_federation_discovery_services().await?;
    let current_len = endpoints.len();
    endpoints.extend(discovery_endpoints);
    debug!(
        "Federation discovery services found {} endpoints",
        endpoints.len() - current_len
    );

    // 4. Use mDNS for local network discovery
    let mdns_endpoints = discover_via_mdns().await?;
    let current_len = endpoints.len();
    endpoints.extend(mdns_endpoints);
    debug!(
        "mDNS discovery found {} endpoints",
        endpoints.len() - current_len
    );

    // 5. Perform peer discovery through known endpoints
    let peer_endpoints = discover_peers_from_known_endpoints(&endpoints).await?;
    endpoints.extend(peer_endpoints);

    // Remove duplicates and validate endpoints
    endpoints.sort();
    endpoints.dedup();

    let mut validated_endpoints = Vec::new();
    for endpoint in endpoints {
        if verify_federation_endpoint(&endpoint).await? {
            validated_endpoints.push(endpoint);
        }
    }

    debug!(
        "DHT-like discovery found {} validated endpoints",
        validated_endpoints.len()
    );
    Ok(validated_endpoints)
}

/// Query federation discovery services for more endpoints
async fn query_federation_discovery_services() -> Result<Vec<String>, SongbirdError> {
    let mut endpoints = Vec::new();

    // Known federation discovery services
    let discovery_services = vec![
        "https://discovery.songbird.local/endpoints",
        "https://primals.discovery.io/federation",
        "https://mcp.discovery.net/services",
    ];

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| {
            SongbirdError::service_error(
                "discovery",
                format!("Failed to create HTTP client: {}", e),
            )
        })?;

    for service_url in discovery_services {
        match client.get(service_url).send().await {
            Ok(response) if response.status().is_success() => {
                match response.json::<serde_json::Value>().await {
                    Ok(discovery_response) => {
                        if let Some(service_endpoints) = discovery_response
                            .get("endpoints")
                            .and_then(|v| v.as_array())
                        {
                            for endpoint in service_endpoints {
                                if let Some(endpoint_str) = endpoint.as_str() {
                                    endpoints.push(endpoint_str.to_string());
                                    debug!("Found discovery service endpoint: {}", endpoint_str);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        debug!(
                            "Failed to parse discovery service response from {}: {}",
                            service_url, e
                        );
                    }
                }
            }
            Ok(response) => {
                debug!(
                    "Discovery service {} returned status: {}",
                    service_url,
                    response.status()
                );
            }
            Err(e) => {
                debug!("Failed to query discovery service {}: {}", service_url, e);
            }
        }
    }

    debug!(
        "Federation discovery services found {} endpoints",
        endpoints.len()
    );
    Ok(endpoints)
}

/// Discover peers through known federation endpoints
async fn discover_peers_from_known_endpoints(
    known_endpoints: &[String],
) -> Result<Vec<String>, SongbirdError> {
    let mut peer_endpoints = Vec::new();

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| {
            SongbirdError::service_error(
                "discovery",
                format!("Failed to create HTTP client: {}", e),
            )
        })?;

    for endpoint in known_endpoints {
        // Try to get peer information from each known endpoint
        let peers_url = format!("{}/federation/peers", endpoint);
        match client.get(&peers_url).send().await {
            Ok(response) if response.status().is_success() => {
                match response.json::<serde_json::Value>().await {
                    Ok(peers_response) => {
                        if let Some(peers) = peers_response.get("peers").and_then(|v| v.as_array())
                        {
                            for peer in peers {
                                if let Some(peer_endpoint) =
                                    peer.get("endpoint").and_then(|v| v.as_str())
                                {
                                    peer_endpoints.push(peer_endpoint.to_string());
                                    debug!("Discovered peer endpoint: {}", peer_endpoint);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        debug!("Failed to parse peers response from {}: {}", endpoint, e);
                    }
                }
            }
            Ok(response) => {
                debug!(
                    "Peers query to {} returned status: {}",
                    endpoint,
                    response.status()
                );
            }
            Err(e) => {
                debug!("Failed to query peers from {}: {}", endpoint, e);
            }
        }

        // Also try alternative peer discovery endpoints
        let alt_urls = vec![
            format!("{}/api/v1/peers", endpoint),
            format!("{}/cluster/members", endpoint),
            format!("{}/status/peers", endpoint),
        ];

        for url in alt_urls {
            match client.get(&url).send().await {
                Ok(response) if response.status().is_success() => {
                    if let Ok(body) = response.text().await {
                        // Simple parsing - look for HTTP endpoints in the response
                        for line in body.lines() {
                            if line.contains("http://") || line.contains("https://") {
                                let words: Vec<&str> = line.split_whitespace().collect();
                                for word in words {
                                    if word.starts_with("http://") || word.starts_with("https://") {
                                        peer_endpoints.push(word.to_string());
                                        debug!("Discovered peer endpoint (alt): {}", word);
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {
                    // Ignore failures on alternative endpoints
                }
            }
        }
    }

    // Remove duplicates
    peer_endpoints.sort();
    peer_endpoints.dedup();

    debug!("Peer discovery found {} endpoints", peer_endpoints.len());
    Ok(peer_endpoints)
}

/// Perform comprehensive network scan for federation services
pub async fn comprehensive_network_scan() -> Result<Vec<String>, SongbirdError> {
    info!("Starting comprehensive network scan for federation services");

    let mut endpoints = Vec::new();

    // Get local network subnets
    let subnets = get_local_subnets().await?;
    let federation_ports = get_federation_ports();

    // Scan all subnets in parallel for better performance
    let mut scan_handles = Vec::new();

    for subnet in subnets {
        let ports = federation_ports.clone();
        let handle = tokio::spawn(async move { scan_subnet_for_federation(&subnet, &ports).await });
        scan_handles.push(handle);
    }

    // Collect results from all scans
    for handle in scan_handles {
        match handle.await {
            Ok(Ok(subnet_endpoints)) => {
                endpoints.extend(subnet_endpoints);
            }
            Ok(Err(e)) => {
                debug!("Subnet scan failed: {}", e);
            }
            Err(e) => {
                debug!("Subnet scan task failed: {}", e);
            }
        }
    }

    // Validate all discovered endpoints
    let mut validated_endpoints = Vec::new();
    for endpoint in endpoints {
        if verify_federation_endpoint(&endpoint).await? {
            validated_endpoints.push(endpoint);
        }
    }

    info!(
        "Comprehensive network scan found {} validated endpoints",
        validated_endpoints.len()
    );
    Ok(validated_endpoints)
}

/// Perform targeted scan for specific service types
pub async fn targeted_service_scan(service_types: &[&str]) -> Result<Vec<String>, SongbirdError> {
    let mut endpoints = Vec::new();

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .map_err(|e| {
            SongbirdError::service_error(
                "discovery",
                format!("Failed to create HTTP client: {}", e),
            )
        })?;

    let subnets = get_local_subnets().await?;

    for subnet in &subnets {
        // Parse subnet to get IP range
        let parts: Vec<&str> = subnet.split('.').collect();
        if parts.len() >= 3 {
            let base = format!("{}.{}.{}", parts[0], parts[1], parts[2]);

            // Scan common ports for each service type
            for i in 1..255 {
                let ip = format!("{}.{}", base, i);

                for &service_type in service_types {
                    let service_ports = get_service_specific_ports(service_type);

                    for port in service_ports {
                        let endpoint = format!("http://{}:{}", ip, port);

                        // Quick check if the service matches the type we're looking for
                        match client.get(&format!("{}/health", endpoint)).send().await {
                            Ok(response) if response.status().is_success() => {
                                if let Ok(body) = response.text().await {
                                    if body.contains(service_type) {
                                        endpoints.push(endpoint.clone());
                                        debug!("Found {} service at {}", service_type, endpoint);
                                    }
                                }
                            }
                            _ => {
                                // Continue scanning
                            }
                        }
                    }
                }
            }
        }
    }

    debug!("Targeted service scan found {} endpoints", endpoints.len());
    Ok(endpoints)
}

/// Get service-specific ports for different types of services
fn get_service_specific_ports(service_type: &str) -> Vec<u16> {
    match service_type {
        "songbird" => vec![8080, 8081, 9090, 7000],
        "federation" => vec![8082, 9091, 7001],
        "mcp" => vec![8083, 9092, 7002],
        "primals" => vec![8084, 9093, 7003],
        "consul" => vec![8500],
        "etcd" => vec![2379, 2380],
        "kubernetes" => vec![6443, 8080, 10250],
        "docker" => vec![2375, 2376],
        _ => get_federation_ports(),
    }
}
