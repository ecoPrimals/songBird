//! Kubernetes service discovery implementation

use songbird_errors::SongbirdError;
use std::time::Duration;
use tracing::{debug, info};

/// Discover federation endpoints from Kubernetes services
pub async fn discover_from_kubernetes() -> Result<Vec<String>, SongbirdError> {
    info!("Starting Kubernetes service discovery");

    let mut endpoints = Vec::new();

    // Try different Kubernetes API access methods
    if let Ok(k8s_endpoints) = discover_via_kubernetes_api().await {
        endpoints.extend(k8s_endpoints);
    }

    if let Ok(dns_endpoints) = discover_via_kubernetes_dns().await {
        endpoints.extend(dns_endpoints);
    }

    if let Ok(env_endpoints) = discover_via_kubernetes_env().await {
        endpoints.extend(env_endpoints);
    }

    // Remove duplicates
    endpoints.sort();
    endpoints.dedup();

    info!("Kubernetes discovery found {} endpoints", endpoints.len());
    Ok(endpoints)
}

/// Discover services via Kubernetes API
async fn discover_via_kubernetes_api() -> Result<Vec<String>, SongbirdError> {
    debug!("Attempting Kubernetes API service discovery");

    let mut endpoints = Vec::new();

    // Try to access Kubernetes API from within cluster
    let kubernetes_hosts = vec![
        "https://kubernetes.default.svc.cluster.local",
        "https://kubernetes.default.svc",
        "https://kubernetes.default",
        "https://10.96.0.1",      // Default Kubernetes service IP
        "https://127.0.0.1:8080", // kubectl proxy
    ];

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .danger_accept_invalid_certs(true) // For self-signed cluster certs
        .build()
        .map_err(|e| {
            SongbirdError::service_error(
                "discovery",
                format!("Failed to create Kubernetes client: {e}"),
            )
        })?;

    // Try to read service account token
    let token = tokio::fs::read_to_string("/var/run/secrets/kubernetes.io/serviceaccount/token")
        .await
        .unwrap_or_default();

    for k8s_host in kubernetes_hosts {
        let mut request = client.get(format!("{k8s_host}/api/v1/services"));

        if !token.is_empty() {
            request = request.bearer_auth(&token);
        }

        match request.send().await {
            Ok(response) if response.status().is_success() => {
                match response.json::<serde_json::Value>().await {
                    Ok(services_response) => {
                        endpoints.extend(parse_kubernetes_services(&services_response));
                        debug!("Successfully queried Kubernetes API at {}", k8s_host);
                        break; // Found working API endpoint
                    }
                    Err(e) => {
                        debug!(
                            "Failed to parse Kubernetes services from {}: {}",
                            k8s_host, e
                        );
                    }
                }
            }
            Ok(response) => {
                debug!(
                    "Kubernetes API at {} returned status: {}",
                    k8s_host,
                    response.status()
                );
            }
            Err(e) => {
                debug!("Failed to connect to Kubernetes API at {}: {}", k8s_host, e);
            }
        }
    }

    debug!(
        "Kubernetes API discovery found {} endpoints",
        endpoints.len()
    );
    Ok(endpoints)
}

/// Parse Kubernetes services response to extract endpoints
fn parse_kubernetes_services(services_response: &serde_json::Value) -> Vec<String> {
    let mut endpoints = Vec::new();

    if let Some(items) = services_response.get("items").and_then(|v| v.as_array()) {
        for item in items {
            if let Some(metadata) = item.get("metadata") {
                if let Some(name) = metadata.get("name").and_then(|v| v.as_str()) {
                    // Look for Songbird-related services
                    if name.contains("songbird")
                        || name.contains("federation")
                        || name.contains("primals")
                        || name.contains("mcp")
                    {
                        if let Some(spec) = item.get("spec") {
                            if let Some(cluster_ip) = spec.get("clusterIP").and_then(|v| v.as_str())
                            {
                                if let Some(ports) = spec.get("ports").and_then(|v| v.as_array()) {
                                    for port in ports {
                                        if let Some(port_num) =
                                            port.get("port").and_then(|v| v.as_u64())
                                        {
                                            let endpoint =
                                                format!("http://{cluster_ip}:{port_num}");
                                            endpoints.push(endpoint.clone());
                                            debug!(
                                                "Found Kubernetes service endpoint: {}",
                                                endpoint
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    endpoints
}

/// Discover services via Kubernetes DNS
async fn discover_via_kubernetes_dns() -> Result<Vec<String>, SongbirdError> {
    debug!("Attempting Kubernetes DNS service discovery");

    let mut endpoints = Vec::new();

    // Common Kubernetes DNS patterns for Songbird services
    let dns_patterns = vec![
        "songbird.default.svc.cluster.local",
        "songbird-federation.default.svc.cluster.local",
        "primals.default.svc.cluster.local",
        "mcp.default.svc.cluster.local",
        "songbird.kube-system.svc.cluster.local",
        "songbird-federation.kube-system.svc.cluster.local",
        "songbird-discovery.default.svc.cluster.local",
    ];

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .map_err(|e| {
            SongbirdError::service_error("discovery", format!("Failed to create DNS client: {e}"))
        })?;

    for dns_name in dns_patterns {
        // Try common service ports
        let service_ports = vec![80, 8080, 8081, 8082, 9090, 9091];

        for port in service_ports {
            let endpoint = format!("http://{dns_name}:{port}");

            // Quick health check to see if service is available
            match client.get(format!("{endpoint}/health")).send().await {
                Ok(response) if response.status().is_success() => {
                    endpoints.push(endpoint.clone());
                    debug!("Found Kubernetes DNS service: {}", endpoint);
                }
                Ok(response) if response.status().as_u16() == 404 => {
                    // Service exists but no /health endpoint, try root
                    if client.get(&endpoint).send().await.is_ok() {
                        endpoints.push(endpoint.clone());
                        debug!("Found Kubernetes DNS service (via root): {}", endpoint);
                    }
                }
                _ => {} // Continue
            }
        }
    }

    debug!(
        "Kubernetes DNS discovery found {} endpoints",
        endpoints.len()
    );
    Ok(endpoints)
}

/// Discover services via Kubernetes environment variables
async fn discover_via_kubernetes_env() -> Result<Vec<String>, SongbirdError> {
    debug!("Attempting Kubernetes environment variable service discovery");

    let mut endpoints = Vec::new();

    // Kubernetes automatically injects service environment variables
    // Format: {SERVICE_NAME}_SERVICE_HOST and {SERVICE_NAME}_SERVICE_PORT
    let env_prefixes = vec![
        "SONGBIRD",
        "SONGBIRD_FEDERATION",
        "PRIMALS",
        "MCP",
        "FEDERATION",
    ];

    for prefix in env_prefixes {
        let host_key = format!("{prefix}_SERVICE_HOST");
        let port_key = format!("{prefix}_SERVICE_PORT");

        if let (Ok(host), Ok(port)) = (std::env::var(&host_key), std::env::var(&port_key)) {
            let endpoint = format!("http://{host}:{port}");
            endpoints.push(endpoint.clone());
            debug!(
                "Found Kubernetes service via env vars: {}={}, {}={}",
                host_key, host, port_key, port
            );
        }

        // Also check for TCP port variants
        let tcp_port_key = format!("{prefix}_SERVICE_PORT_HTTP");
        if let (Ok(host), Ok(port)) = (std::env::var(&host_key), std::env::var(&tcp_port_key)) {
            let endpoint = format!("http://{host}:{port}");
            endpoints.push(endpoint.clone());
            debug!(
                "Found Kubernetes service via env vars (TCP): {}={}, {}={}",
                host_key, host, tcp_port_key, port
            );
        }
    }

    // Look for any environment variables that might contain Songbird service info
    for (key, value) in std::env::vars() {
        if (key.contains("SONGBIRD") || key.contains("FEDERATION") || key.contains("PRIMALS"))
            && key.contains("HOST")
        {
            // Try to find corresponding port
            let service_name = key.replace("_SERVICE_HOST", "");
            let port_key = format!("{service_name}_SERVICE_PORT");

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
        "Kubernetes environment discovery found {} endpoints",
        endpoints.len()
    );
    Ok(endpoints)
}

/// Check if running in Kubernetes cluster
pub fn is_running_in_kubernetes() -> bool {
    // Check for Kubernetes service account token
    std::path::Path::new("/var/run/secrets/kubernetes.io/serviceaccount/token").exists()
        || std::env::var("KUBERNETES_SERVICE_HOST").is_ok()
}

/// Get current Kubernetes namespace
pub async fn get_current_namespace() -> Option<String> {
    // Try to read namespace from service account
    if let Ok(namespace) =
        tokio::fs::read_to_string("/var/run/secrets/kubernetes.io/serviceaccount/namespace").await
    {
        return Some(namespace.trim().to_string());
    }

    // Fallback to environment variable
    std::env::var("POD_NAMESPACE")
        .ok()
        .or_else(|| std::env::var("NAMESPACE").ok())
        .or(Some("default".to_string()))
}
