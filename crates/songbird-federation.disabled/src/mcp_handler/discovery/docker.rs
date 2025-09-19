//! Docker Swarm service discovery implementation

use songbird_errors::SongbirdError;
use std::time::Duration;
use tracing::{debug, info};

/// Discover federation endpoints from Docker Swarm services
pub async fn discover_from_docker_swarm() -> Result<Vec<String>, SongbirdError> {
    info!("Starting Docker Swarm service discovery");

    let mut endpoints = Vec::new();

    // Try different Docker API access methods
    if let Ok(swarm_endpoints) = discover_via_docker_api().await {
        endpoints.extend(swarm_endpoints);
    }

    if let Ok(dns_endpoints) = discover_via_docker_dns().await {
        endpoints.extend(dns_endpoints);
    }

    if let Ok(env_endpoints) = discover_via_docker_env().await {
        endpoints.extend(env_endpoints);
    }

    // Remove duplicates
    endpoints.sort();
    endpoints.dedup();

    info!("Docker Swarm discovery found {} endpoints", endpoints.len());
    Ok(endpoints)
}

/// Discover services via Docker API
async fn discover_via_docker_api() -> Result<Vec<String>, SongbirdError> {
    debug!("Attempting Docker API service discovery");

    let mut endpoints = Vec::new();

    // Try different Docker API endpoints
    let docker_hosts = vec![
        "http://localhost:2375",  // Docker daemon without TLS
        "https://localhost:2376", // Docker daemon with TLS
        "http://docker:2375",
        "http://127.0.0.1:2375",
    ];

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .danger_accept_invalid_certs(true) // For self-signed Docker certs
        .build()
        .map_err(|e| {
            SongbirdError::service_error(
                "discovery",
                format!("Failed to create Docker client: {e}"),
            )
        })?;

    for docker_host in docker_hosts {
        // Try to list services (Docker Swarm)
        match client.get(format!("{docker_host}/services")).send().await {
            Ok(response) if response.status().is_success() => {
                match response.json::<serde_json::Value>().await {
                    Ok(services_response) => {
                        endpoints.extend(parse_docker_services(&services_response));
                        debug!("Successfully queried Docker API at {}", docker_host);
                        break; // Found working API endpoint
                    }
                    Err(e) => {
                        debug!(
                            "Failed to parse Docker services from {}: {}",
                            docker_host, e
                        );
                    }
                }
            }
            Ok(response) => {
                debug!(
                    "Docker API at {} returned status: {}",
                    docker_host,
                    response.status()
                );

                // If /services doesn't work, try /containers (standalone Docker)
                match client
                    .get(format!("{docker_host}/containers/json"))
                    .send()
                    .await
                {
                    Ok(containers_response) if containers_response.status().is_success() => {
                        match containers_response.json::<serde_json::Value>().await {
                            Ok(containers_response) => {
                                endpoints.extend(parse_docker_containers(&containers_response));
                                debug!("Successfully queried Docker containers at {}", docker_host);
                                break;
                            }
                            Err(e) => {
                                debug!(
                                    "Failed to parse Docker containers from {}: {}",
                                    docker_host, e
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }
            Err(e) => {
                debug!("Failed to connect to Docker API at {}: {}", docker_host, e);
            }
        }
    }

    debug!("Docker API discovery found {} endpoints", endpoints.len());
    Ok(endpoints)
}

/// Parse Docker services response to extract endpoints
fn parse_docker_services(services_response: &serde_json::Value) -> Vec<String> {
    let mut endpoints = Vec::new();

    if let Some(services) = services_response.as_array() {
        for service in services {
            if let Some(spec) = service.get("Spec") {
                if let Some(name) = spec.get("Name").and_then(|v| v.as_str()) {
                    // Look for Songbird-related services
                    if name.contains("songbird")
                        || name.contains("federation")
                        || name.contains("primals")
                        || name.contains("mcp")
                    {
                        // Extract endpoint from service
                        if let Some(endpoint) = extract_docker_service_endpoint(service) {
                            endpoints.push(endpoint.clone());
                            debug!("Found Docker service endpoint: {}", endpoint);
                        }
                    }
                }
            }
        }
    }

    endpoints
}

/// Parse Docker containers response to extract endpoints
fn parse_docker_containers(containers_response: &serde_json::Value) -> Vec<String> {
    let mut endpoints = Vec::new();

    if let Some(containers) = containers_response.as_array() {
        for container in containers {
            // Check container names and labels
            if let Some(names) = container.get("Names").and_then(|v| v.as_array()) {
                for name in names {
                    if let Some(name_str) = name.as_str() {
                        if name_str.contains("songbird")
                            || name_str.contains("federation")
                            || name_str.contains("primals")
                            || name_str.contains("mcp")
                        {
                            if let Some(endpoint) = extract_docker_container_endpoint(container) {
                                endpoints.push(endpoint.clone());
                                debug!("Found Docker container endpoint: {}", endpoint);
                            }
                        }
                    }
                }
            }

            // Check labels
            if let Some(labels) = container.get("Labels").and_then(|v| v.as_object()) {
                for (label_key, label_value) in labels {
                    if let Some(label_str) = label_value.as_str() {
                        if (label_key.contains("songbird") || label_str.contains("songbird"))
                            || (label_key.contains("federation")
                                || label_str.contains("federation"))
                        {
                            if let Some(endpoint) = extract_docker_container_endpoint(container) {
                                endpoints.push(endpoint.clone());
                                debug!("Found Docker container endpoint via labels: {}", endpoint);
                            }
                        }
                    }
                }
            }
        }
    }

    endpoints
}

/// Extract endpoint from Docker service definition
fn extract_docker_service_endpoint(service: &serde_json::Value) -> Option<String> {
    // Try to get published ports
    if let Some(endpoint) = service.get("Endpoint") {
        if let Some(ports) = endpoint.get("Ports").and_then(|v| v.as_array()) {
            for port in ports {
                if let Some(published_port) = port.get("PublishedPort").and_then(|v| v.as_u64()) {
                    return Some(format!("http://localhost:{published_port}"));
                }
            }
        }
    }

    // Try to get service name for Docker DNS resolution
    if let Some(spec) = service.get("Spec") {
        if let Some(name) = spec.get("Name").and_then(|v| v.as_str()) {
            // Default port assumption for Songbird services
            return Some(format!("http://{name}:8080"));
        }
    }

    None
}

/// Extract endpoint from Docker container definition
fn extract_docker_container_endpoint(container: &serde_json::Value) -> Option<String> {
    // Try to get port mappings
    if let Some(ports) = container.get("Ports").and_then(|v| v.as_array()) {
        for port in ports {
            if let Some(public_port) = port.get("PublicPort").and_then(|v| v.as_u64()) {
                return Some(format!("http://localhost:{public_port}"));
            }
        }
    }

    // Try to get network settings
    if let Some(network_settings) = container.get("NetworkSettings") {
        if let Some(networks) = network_settings.get("Networks").and_then(|v| v.as_object()) {
            for (_network_name, network_info) in networks {
                if let Some(ip_address) = network_info.get("IPAddress").and_then(|v| v.as_str()) {
                    if !ip_address.is_empty() {
                        // Try common ports
                        return Some(format!("http://{ip_address}:8080"));
                    }
                }
            }
        }
    }

    None
}

/// Discover services via Docker DNS
async fn discover_via_docker_dns() -> Result<Vec<String>, SongbirdError> {
    debug!("Attempting Docker DNS service discovery");

    let mut endpoints = Vec::new();

    // Common Docker service names for Songbird services
    let service_names = vec![
        "songbird",
        "songbird-federation",
        "primals",
        "mcp",
        "federation",
        "songbird-discovery",
    ];

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .map_err(|e| {
            SongbirdError::service_error("discovery", format!("Failed to create DNS client: {e}"))
        })?;

    for service_name in service_names {
        // Try common service ports
        let service_ports = vec![80, 8080, 8081, 8082, 9090, 9091];

        for port in service_ports {
            let endpoint = format!("http://{service_name}:{port}");

            // Quick health check to see if service is available
            match client.get(format!("{endpoint}/health")).send().await {
                Ok(response) if response.status().is_success() => {
                    endpoints.push(endpoint.clone());
                    debug!("Found Docker DNS service: {}", endpoint);
                }
                Ok(response) if response.status().as_u16() == 404 => {
                    // Service exists but no /health endpoint, try root
                    if client.get(&endpoint).send().await.is_ok() {
                        endpoints.push(endpoint.clone());
                        debug!("Found Docker DNS service (via root): {}", endpoint);
                    }
                }
                _ => {} // Continue
            }
        }
    }

    debug!("Docker DNS discovery found {} endpoints", endpoints.len());
    Ok(endpoints)
}

/// Discover services via Docker environment variables
async fn discover_via_docker_env() -> Result<Vec<String>, SongbirdError> {
    debug!("Attempting Docker environment variable service discovery");

    let mut endpoints = Vec::new();

    // Docker Compose automatically creates environment variables for services
    // Format: {SERVICE_NAME}_PORT_{PORT}_TCP_ADDR and {SERVICE_NAME}_PORT_{PORT}_TCP_PORT
    let env_prefixes = vec![
        "SONGBIRD",
        "SONGBIRD_FEDERATION",
        "PRIMALS",
        "MCP",
        "FEDERATION",
    ];

    for prefix in env_prefixes {
        // Try different port patterns
        let ports = vec![8080, 8081, 8082, 9090, 9091];

        for port in ports {
            let addr_key = format!("{prefix}_PORT_{port}_TCP_ADDR");
            let port_key = format!("{prefix}_PORT_{port}_TCP_PORT");

            if let (Ok(addr), Ok(port_env)) = (std::env::var(&addr_key), std::env::var(&port_key)) {
                let endpoint = format!("http://{addr}:{port_env}");
                endpoints.push(endpoint.clone());
                debug!(
                    "Found Docker service via env vars: {}={}, {}={}",
                    addr_key, addr, port_key, port_env
                );
            }
        }

        // Also check for simple service environment variables
        let service_host_key = format!("{prefix}_SERVICE_HOST");
        let service_port_key = format!("{prefix}_SERVICE_PORT");

        if let (Ok(host), Ok(port)) = (
            std::env::var(&service_host_key),
            std::env::var(&service_port_key),
        ) {
            let endpoint = format!("http://{host}:{port}");
            endpoints.push(endpoint.clone());
            debug!(
                "Found Docker service via service env vars: {}={}, {}={}",
                service_host_key, host, service_port_key, port
            );
        }
    }

    // Look for any environment variables that might contain Docker service info
    for (key, value) in std::env::vars() {
        if (key.contains("SONGBIRD") || key.contains("FEDERATION") || key.contains("PRIMALS"))
            && (key.contains("TCP_ADDR") || key.contains("HOST"))
        {
            // Try to find corresponding port
            let port_key = if key.contains("TCP_ADDR") {
                key.replace("_TCP_ADDR", "_TCP_PORT")
            } else if key.contains("_HOST") {
                key.replace("_HOST", "_PORT")
            } else {
                continue;
            };

            if let Ok(port) = std::env::var(&port_key) {
                let endpoint = format!("http://{value}:{port}");
                endpoints.push(endpoint.clone());
                debug!("Found service via env scan: {}", endpoint);
            }
        }
    }

    // Remove duplicates
    endpoints.sort();
    endpoints.dedup();

    debug!(
        "Docker environment discovery found {} endpoints",
        endpoints.len()
    );
    Ok(endpoints)
}

/// Check if running in Docker container
pub fn is_running_in_docker() -> bool {
    // Check for Docker-specific files
    std::path::Path::new("/.dockerenv").exists()
        || std::fs::read_to_string("/proc/1/cgroup")
            .map(|content| content.contains("docker") || content.contains("containerd"))
            .unwrap_or(false)
}

/// Get Docker container ID
pub fn get_container_id() -> Option<String> {
    // Try to read container ID from cgroup
    if let Ok(cgroup_content) = std::fs::read_to_string("/proc/self/cgroup") {
        for line in cgroup_content.lines() {
            if line.contains("docker") || line.contains("containerd") {
                // Extract container ID from cgroup path
                let parts: Vec<&str> = line.split('/').collect();
                if let Some(last_part) = parts.last() {
                    if last_part.len() >= 12 {
                        return Some(last_part.to_string());
                    }
                }
            }
        }
    }

    None
}
