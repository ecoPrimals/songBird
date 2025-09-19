//! Network utility functions for discovery operations

use if_addrs;
use songbird_errors::{NetworkError, SongbirdError};
use std::net::UdpSocket;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, warn};

/// Get local network subnets for scanning
pub async fn get_local_subnets() -> Result<Vec<String>, SongbirdError> {
    let mut subnets = Vec::new();

    // Get network interfaces
    let interfaces = if_addrs::get_if_addrs().map_err(|e| {
        songbird_errors::SongbirdError::network(&format!("Failed to get network interfaces: {e}"))
    })?;

    for interface in interfaces {
        if !interface.is_loopback() {
            if let std::net::IpAddr::V4(ipv4) = interface.ip() {
                let subnet = format!(
                    "{}.{}.{}.0/24",
                    ipv4.octets()[0],
                    ipv4.octets()[1],
                    ipv4.octets()[2]
                );
                subnets.push(subnet);
            }
        }
    }

    Ok(subnets)
}

/// Scan subnet for federation endpoints
pub async fn scan_subnet_for_federation(
    subnet: &str,
    federation_ports: &[u16],
) -> Result<Vec<String>, SongbirdError> {
    let mut endpoints = Vec::new();

    // Parse subnet (simple implementation for /24 networks)
    let parts: Vec<&str> = subnet.split('.').collect();
    if parts.len() >= 3 {
        let base = format!("{}.{}.{}", parts[0], parts[1], parts[2]);

        // Scan IPs 1-254 in the subnet (skip .0 and .255)
        for i in 1..255 {
            let ip = format!("{base}.{i}");

            // Check each federation port
            for &port in federation_ports {
                if is_port_open(&ip, port).await {
                    let endpoint = format!("http://{ip}:{port}");
                    endpoints.push(endpoint);
                    debug!("Found potential federation endpoint: {}:{}", ip, port);
                }
            }
        }
    }

    debug!("Subnet {} scan found {} endpoints", subnet, endpoints.len());
    Ok(endpoints)
}

/// Check if a port is open on the given IP address
pub async fn is_port_open(ip: &str, port: u16) -> bool {
    let address = format!("{ip}:{port}");

    match timeout(
        Duration::from_millis(100),
        tokio::net::TcpStream::connect(&address),
    )
    .await
    {
        Ok(Ok(_)) => {
            debug!("Port {}:{} is open", ip, port);
            true
        }
        _ => false,
    }
}

/// Verify if an endpoint supports federation protocol
pub async fn verify_federation_endpoint(endpoint: &str) -> Result<bool, SongbirdError> {
    debug!("Verifying federation endpoint: {}", endpoint);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| {
            SongbirdError::service_error("discovery", format!("Failed to create HTTP client: {e}"))
        })?;

    // Try to get federation info
    let info_url = format!("{endpoint}/federation/info");
    match client.get(&info_url).send().await {
        Ok(response) if response.status().is_success() => {
            // Check if response contains federation identifiers
            match response.text().await {
                Ok(body) => {
                    let is_federation = body.contains("songbird")
                        || body.contains("federation")
                        || body.contains("primals")
                        || body.contains("mcp");

                    if is_federation {
                        debug!("Verified federation endpoint: {}", endpoint);
                    }
                    Ok(is_federation)
                }
                Err(_) => Ok(false),
            }
        }
        _ => {
            // Try alternative endpoints
            let alt_urls = vec![
                format!("{}/health", endpoint),
                format!("{}/status", endpoint),
                format!("{}/api/v1/info", endpoint),
            ];

            for url in alt_urls {
                match client.get(&url).send().await {
                    Ok(response) if response.status().is_success() => {
                        if let Ok(body) = response.text().await {
                            if body.contains("songbird") || body.contains("federation") {
                                debug!("Verified federation endpoint (alt): {}", endpoint);
                                return Ok(true);
                            }
                        }
                    }
                    _ => continue,
                }
            }

            debug!("Could not verify federation endpoint: {}", endpoint);
            Ok(false)
        }
    }
}

/// Send UDP broadcast message and collect responses
pub async fn send_udp_broadcast(
    message: &[u8],
    port: u16,
    timeout_duration: Duration,
) -> Result<Vec<String>, SongbirdError> {
    let mut discovered_endpoints = Vec::new();

    // Create broadcast socket
    let socket = UdpSocket::bind("0.0.0.0:0").map_err(|e| {
        SongbirdError::service_error("discovery", format!("Failed to create UDP socket: {e}"))
    })?;

    socket.set_broadcast(true).map_err(|e| {
        SongbirdError::service_error("discovery", format!("Failed to enable broadcast: {e}"))
    })?;

    socket
        .set_read_timeout(Some(timeout_duration))
        .map_err(|e| {
            SongbirdError::service_error("discovery", format!("Failed to set socket timeout: {e}"))
        })?;

    // Send broadcast to common broadcast addresses
    let broadcast_addresses = vec![
        format!("255.255.255.255:{}", port),
        format!("224.0.0.1:{}", port), // All Systems multicast
    ];

    for addr in &broadcast_addresses {
        if let Err(e) = socket.send_to(message, addr) {
            warn!("Failed to send broadcast to {}: {}", addr, e);
        } else {
            debug!("Sent UDP broadcast to {}", addr);
        }
    }

    // Collect responses
    let mut buffer = [0u8; 1024];
    let start_time = std::time::Instant::now();

    while start_time.elapsed() < timeout_duration {
        match socket.recv_from(&mut buffer) {
            Ok((len, addr)) => {
                let response = String::from_utf8_lossy(&buffer[..len]);
                if response.contains("songbird") || response.contains("federation") {
                    let endpoint = format!("http://{}", addr.ip());
                    discovered_endpoints.push(endpoint);
                    debug!("Discovered endpoint via UDP broadcast: {}", addr);
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // Timeout reached, continue
                break;
            }
            Err(e) => {
                debug!("UDP receive error: {}", e);
                break;
            }
        }
    }

    debug!(
        "UDP broadcast discovery found {} endpoints",
        discovered_endpoints.len()
    );
    Ok(discovered_endpoints)
}

/// Get common federation ports to scan
pub fn get_federation_ports() -> Vec<u16> {
    vec![
        8080, 8081, 8082, 8083, 8084, 8085, // Common HTTP ports
        9090, 9091, 9092, 9093, 9094, 9095, // Common service ports
        3000, 3001, 3002, 3003, // Node.js common ports
        4000, 4001, 4002, 4003, // Alternative service ports
        7000, 7001, 7002, 7003, // Custom service ports
        6379, // Redis (sometimes used for discovery)
        2379, 2380, // etcd
        8500, // Consul
    ]
}
