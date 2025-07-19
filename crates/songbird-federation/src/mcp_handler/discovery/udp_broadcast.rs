//! UDP broadcast discovery implementation

use songbird_errors::SongbirdError;
use std::time::Duration;
use tracing::{debug, info};

use super::network_utils::{get_federation_ports, send_udp_broadcast};

/// Discover federation endpoints via UDP broadcast
pub async fn discover_via_udp_broadcast() -> Result<Vec<String>, SongbirdError> {
    info!("Starting UDP broadcast discovery for federation endpoints");

    let mut endpoints = Vec::new();
    let federation_ports = get_federation_ports();

    // Create discovery message
    let discovery_message = create_discovery_message();

    // Send broadcasts on various ports
    for &port in &federation_ports {
        match send_udp_broadcast(discovery_message.as_bytes(), port, Duration::from_secs(2)).await {
            Ok(port_endpoints) => {
                let current_len = endpoints.len();
                endpoints.extend(port_endpoints);
                debug!(
                    "UDP broadcast on port {} found {} endpoints",
                    port,
                    endpoints.len() - current_len
                );
            }
            Err(e) => {
                debug!("UDP broadcast failed on port {}: {}", port, e);
            }
        }
    }

    // Also try standard discovery ports
    let discovery_ports = vec![9999, 8888, 7777, 6666, 5555];
    for &port in &discovery_ports {
        match send_udp_broadcast(discovery_message.as_bytes(), port, Duration::from_secs(1)).await {
            Ok(port_endpoints) => {
                let current_len = endpoints.len();
                endpoints.extend(port_endpoints);
                debug!(
                    "Discovery broadcast on port {} found {} endpoints",
                    port,
                    endpoints.len() - current_len
                );
            }
            Err(e) => {
                debug!("Discovery broadcast failed on port {}: {}", port, e);
            }
        }
    }

    // Remove duplicates
    endpoints.sort();
    endpoints.dedup();

    info!(
        "UDP broadcast discovery found {} unique endpoints",
        endpoints.len()
    );
    Ok(endpoints)
}

/// Create discovery message for UDP broadcast
fn create_discovery_message() -> String {
    format!(
        "SONGBIRD_DISCOVERY_v1\n\
        type=federation\n\
        protocol=mcp\n\
        service=songbird\n\
        timestamp={}\n\
        request_id={}\n",
        chrono::Utc::now().timestamp(),
        uuid::Uuid::new_v4()
    )
}

/// Create response message for UDP discovery requests
pub fn create_discovery_response(service_port: u16, service_info: &ServiceInfo) -> String {
    format!(
        "SONGBIRD_RESPONSE_v1\n\
        type=federation\n\
        protocol=mcp\n\
        service=songbird\n\
        port={}\n\
        endpoints={}\n\
        capabilities={}\n\
        version={}\n\
        timestamp={}\n\
        instance_id={}\n",
        service_port,
        service_info.endpoints.join(","),
        service_info.capabilities.join(","),
        service_info.version,
        chrono::Utc::now().timestamp(),
        service_info.instance_id
    )
}

/// Service information for discovery responses
pub struct ServiceInfo {
    pub endpoints: Vec<String>,
    pub capabilities: Vec<String>,
    pub version: String,
    pub instance_id: String,
}

impl ServiceInfo {
    /// Create default service info
    pub fn default() -> Self {
        Self {
            endpoints: vec!["federation".to_string(), "mcp".to_string()],
            capabilities: vec![
                "service_discovery".to_string(),
                "load_balancing".to_string(),
                "health_monitoring".to_string(),
            ],
            version: "1.0.0".to_string(),
            instance_id: uuid::Uuid::new_v4().to_string(),
        }
    }
}

/// Start UDP discovery listener to respond to discovery requests
pub async fn start_discovery_listener(
    port: u16,
    service_info: ServiceInfo,
) -> Result<(), SongbirdError> {
    info!("Starting UDP discovery listener on port {}", port);

    let socket = tokio::net::UdpSocket::bind(format!("0.0.0.0:{}", port))
        .await
        .map_err(|e| {
            SongbirdError::service_error(
                "discovery",
                format!("Failed to bind UDP listener socket: {}", e),
            )
        })?;

    let mut buffer = [0u8; 1024];

    loop {
        match socket.recv_from(&mut buffer).await {
            Ok((len, addr)) => {
                let message = String::from_utf8_lossy(&buffer[..len]);

                // Check if this is a Songbird discovery request
                if message.contains("SONGBIRD_DISCOVERY") {
                    debug!("Received discovery request from {}", addr);

                    // Send response
                    let response = create_discovery_response(port, &service_info);
                    if let Err(e) = socket.send_to(response.as_bytes(), addr).await {
                        debug!("Failed to send discovery response to {}: {}", addr, e);
                    } else {
                        debug!("Sent discovery response to {}", addr);
                    }
                }
            }
            Err(e) => {
                debug!("UDP listener error: {}", e);
                // Continue listening despite errors
            }
        }
    }
}

/// Parse discovery response message
pub fn parse_discovery_response(response: &str) -> Option<DiscoveryResponse> {
    if !response.contains("SONGBIRD_RESPONSE") {
        return None;
    }

    let mut port = 0;
    let mut endpoints = Vec::new();
    let mut capabilities = Vec::new();
    let mut version = String::new();
    let mut instance_id = String::new();

    for line in response.lines() {
        if line.starts_with("port=") {
            if let Ok(p) = line[5..].parse::<u16>() {
                port = p;
            }
        } else if line.starts_with("endpoints=") {
            endpoints = line[10..].split(',').map(|s| s.to_string()).collect();
        } else if line.starts_with("capabilities=") {
            capabilities = line[13..].split(',').map(|s| s.to_string()).collect();
        } else if line.starts_with("version=") {
            version = line[8..].to_string();
        } else if line.starts_with("instance_id=") {
            instance_id = line[12..].to_string();
        }
    }

    if port > 0 {
        Some(DiscoveryResponse {
            port,
            endpoints,
            capabilities,
            version,
            instance_id,
        })
    } else {
        None
    }
}

/// Parsed discovery response information
pub struct DiscoveryResponse {
    pub port: u16,
    pub endpoints: Vec<String>,
    pub capabilities: Vec<String>,
    pub version: String,
    pub instance_id: String,
}
