//! Network scanning discovery for Universal Primals

use crate::errors::PrimalResult;
use songbird_errors::NetworkError;
use songbird_universal::PrimalType;
use std::time::Duration;
use tracing::{debug, info, warn};
use uuid::Uuid;

use super::parsing::{discover_capabilities_from_service, extract_metadata_from_info, infer_primal_type_from_capabilities};
use super::types::{DiscoveredPrimal, DiscoveryMethod};

/// Perform comprehensive network scan for primals
pub async fn perform_network_scan() -> PrimalResult<Vec<DiscoveredPrimal>> {
    debug!("Starting comprehensive network scan for Universal Primals...");

    let mut discovered_primals = Vec::new();

    // Common IP ranges to scan
    let ip_ranges = vec![
        "127.0.0.1".to_string(),      // Localhost
        "10.0.0.0/24".to_string(),    // Private network
        "192.168.1.0/24".to_string(), // Home network
        "172.16.0.0/24".to_string(),  // Docker networks
    ];

    for ip_range in &ip_ranges {
        match scan_network_range(ip_range, &get_common_primal_ports()).await {
            Ok(mut range_primals) => {
                discovered_primals.append(&mut range_primals);
            }
            Err(e) => {
                warn!("Failed to scan network range {}: {}", ip_range, e);
            }
        }
    }

    info!(
        "🔍 Network scan discovered {} primals",
        discovered_primals.len()
    );
    Ok(discovered_primals)
}

/// Scan a specific network range for primal endpoints
pub async fn scan_network_range(
    ip_range: &str,
    ports: &[u16],
) -> PrimalResult<Vec<DiscoveredPrimal>> {
    debug!("Scanning network range: {} on ports {:?}", ip_range, ports);

    let mut discovered_primals = Vec::new();

    // For localhost or single IP
    if ip_range == "127.0.0.1" || !ip_range.contains("/") {
        let ip = ip_range.split("/").next().unwrap_or(ip_range);
        for &port in ports {
            let endpoint = format!("http://{}:{}", ip, port);
            if let Ok(primal) = probe_primal_endpoint(&endpoint).await {
                discovered_primals.push(primal);
            }
        }
        return Ok(discovered_primals);
    }

    // For network ranges like 192.168.1.0/24
    let base_ip = ip_range.split("/").next().unwrap_or(ip_range);
    let ip_parts: Vec<&str> = base_ip.split(".").collect();

    if ip_parts.len() == 4 {
        let base = format!("{}.{}.{}", ip_parts[0], ip_parts[1], ip_parts[2]);

        // Scan first 10 IPs in the range for performance
        for i in 1..=10 {
            let ip = format!("{}.{}", base, i);
            for &port in ports {
                let endpoint = format!("http://{}:{}", ip, port);
                if let Ok(primal) = probe_primal_endpoint(&endpoint).await {
                    discovered_primals.push(primal);
                }
            }
        }
    }

    debug!(
        "Network range {} scan found {} primals",
        ip_range,
        discovered_primals.len()
    );
    Ok(discovered_primals)
}

/// Probe a specific endpoint to see if it's a primal service
pub async fn probe_primal_endpoint(endpoint: &str) -> PrimalResult<DiscoveredPrimal> {
    debug!("Probing endpoint: {}", endpoint);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| {
            songbird_errors::SongbirdError::Network(Box::new(NetworkError {
                message: format!("Failed to create HTTP client: {}", e),
                endpoint: Some(endpoint.to_string()),
                port: extract_port_from_endpoint(endpoint),
                protocol: Some("HTTP".to_string()),
            }))
        })?;

    // Try different health/info endpoints
    let probe_paths = vec!["/health", "/info", "/status", "/", "/api/v1/info"];

    for path in &probe_paths {
        let url = format!("{}{}", endpoint.trim_end_matches('/'), path);

        match client.get(&url).send().await {
            Ok(response) if response.status().is_success() => {
                match response.text().await {
                    Ok(body) => {
                        // Check if response indicates this is a primal service
                        if is_primal_service_response(&body) {
                            return create_discovered_primal_from_response(endpoint, &body);
                        }
                    }
                    Err(e) => {
                        debug!("Failed to read response body from {}: {}", url, e);
                    }
                }
            }
            Ok(response) => {
                debug!("Endpoint {} returned status: {}", url, response.status());
            }
            Err(e) => {
                debug!("Failed to connect to {}: {}", url, e);
            }
        }
    }

    Err(crate::errors::PrimalError::Network(format!(
        "Endpoint {} is not a primal service",
        endpoint
    )))
}

/// Test connectivity to an endpoint
pub async fn test_endpoint_connectivity(endpoint: &str) -> PrimalResult<bool> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| {
            songbird_errors::SongbirdError::Network(Box::new(NetworkError {
                message: format!(
                    "Universal Primals Discovery - connectivity test failed: {}",
                    e
                ),
                endpoint: None,
                port: None,
                protocol: Some("HTTP".to_string()),
            }))
        })?;

    let health_url = if endpoint.ends_with("/health") {
        endpoint.to_string()
    } else {
        format!("{}/health", endpoint.trim_end_matches('/'))
    };

    match client.get(&health_url).send().await {
        Ok(response) => Ok(response.status().is_success()),
        Err(_) => Ok(false),
    }
}

/// Get common ports used by primal services
pub fn get_common_primal_ports() -> Vec<u16> {
    vec![
        8080, 8081, 8082, 8083, 8084, 8085, // Common HTTP ports
        9090, 9091, 9092, 9093, 9094, 9095, // Common service ports
        3000, 3001, 3002, 3003, // Node.js common ports
        4000, 4001, 4002, 4003, // Alternative service ports
        7000, 7001, 7002, 7003, // Custom service ports
        8443, // HTTPS
    ]
}

/// Check if a response body indicates a primal service
fn is_primal_service_response(body: &str) -> bool {
    let body_lower = body.to_lowercase();

    // Look for primal-specific keywords
    body_lower.contains("primal") ||
    body_lower.contains("beardog") ||
    body_lower.contains("nestgate") ||
    body_lower.contains("toadstool") ||
    body_lower.contains("squirrel") ||
    body_lower.contains("songbird") ||
    body_lower.contains("biomeos") ||
    // Generic service health indicators that might be primals
    (body_lower.contains("healthy") && (
        body_lower.contains("service") ||
        body_lower.contains("api") ||
        body_lower.contains("server")
    ))
}

/// Create a discovered primal from endpoint response
fn create_discovered_primal_from_response(
    endpoint: &str,
    body: &str,
) -> PrimalResult<DiscoveredPrimal> {
    // Parse response as JSON to extract service info
    let service_info = match serde_json::from_str::<serde_json::Value>(body) {
        Ok(json) => json,
        Err(_) => {
            // Fallback to minimal JSON for text responses
            serde_json::json!({
                "description": body,
                "endpoints": [endpoint]
            })
        }
    };

    // Discover capabilities based on service behavior
    let capabilities = discover_capabilities_from_service(endpoint, &service_info);
    let metadata = extract_metadata_from_info(&service_info);
    let primal_type = infer_primal_type_from_capabilities(&capabilities);

    let mut primal = DiscoveredPrimal::new(
        Uuid::new_v4().to_string(),
        primal_type,
        endpoint.to_string(),
        DiscoveryMethod::NetworkScan,
    );

    primal.capabilities = capabilities;
    primal.health_status = "healthy".to_string();

    for (key, value) in metadata {
        primal.add_metadata(key, value);
    }

    debug!(
        "✅ Discovered primal via network scan: {} ({})",
        endpoint, primal.primal_type
    );
    Ok(primal)
}

/// Extract port number from endpoint URL
fn extract_port_from_endpoint(endpoint: &str) -> Option<u16> {
    if let Some(start) = endpoint.rfind(':') {
        let port_part = &endpoint[start + 1..];
        // Remove any trailing path
        let port_str = port_part.split('/').next().unwrap_or(port_part);
        port_str.parse().ok()
    } else {
        None
    }
}

/// Perform targeted scan for specific primal types
pub async fn scan_for_primal_type(primal_type: PrimalType) -> PrimalResult<Vec<DiscoveredPrimal>> {
    let mut discovered_primals = Vec::new();

    // Get specific ports for this primal type
    let ports = get_ports_for_primal_type(&primal_type);

    // Scan localhost with specific ports
    for &port in &ports {
        let endpoint = format!("http://127.0.0.1:{}", port);
        if let Ok(primal) = probe_primal_endpoint(&endpoint).await {
            if primal.primal_type.as_str() == primal_type.as_str() {
                discovered_primals.push(primal);
            }
        }
    }

    debug!(
        "Targeted scan for {} found {} instances",
        primal_type.as_str(),
        discovered_primals.len()
    );
    Ok(discovered_primals)
}

/// Get specific ports typically used by different primal types
fn get_ports_for_primal_type(primal_type: &PrimalType) -> Vec<u16> {
    match primal_type.as_str() {
        "beardog" => vec![8443, 8080],
        "nestgate" => vec![8080, 8081],
        "toadstool" => vec![8083, 9090],
        "squirrel" => vec![8084, 9091],
        "songbird" => vec![8082, 9092],
        "biomeos" => vec![8085, 9093],
        _ => get_common_primal_ports(),
    }
}
