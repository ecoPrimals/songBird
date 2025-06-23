use std::net::{SocketAddr, UdpSocket};
use std::process::Command;
use std::str;
use std::time::Instant;
use crate::discovery::types::*;
use crate::discovery::config::NetworkConfig;
use crate::errors::Result;

/// Network operations and measurements
pub struct NetworkManager;

impl NetworkManager {
    /// Measure ping latency to target
    pub async fn measure_ping_latency(target_address: &str) -> Result<f64> {
        // Extract hostname/IP from address
        let target_host = if let Some(colon_pos) = target_address.find(':') {
            &target_address[..colon_pos]
        } else {
            target_address
        };

        #[cfg(target_os = "linux")]
        {
            if let Ok(output) = Command::new("ping")
                .args(&["-c", "3", "-W", "2", target_host])
                .output()
            {
                if output.status.success() {
                    if let Ok(output_str) = str::from_utf8(&output.stdout) {
                        // Parse ping output for average latency
                        for line in output_str.lines() {
                            if line.contains("rtt min/avg/max/mdev") {
                                if let Some(stats_part) = line.split('=').nth(1) {
                                    let stats: Vec<&str> = stats_part.split('/').collect();
                                    if stats.len() >= 2 {
                                        if let Ok(avg_latency) = stats[1].parse::<f64>() {
                                            return Ok(avg_latency);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Ok(output) = Command::new("ping")
                .args(&["-c", "3", target_host])
                .output()
            {
                if output.status.success() {
                    if let Ok(output_str) = str::from_utf8(&output.stdout) {
                        // Parse macOS ping output
                        for line in output_str.lines() {
                            if line.contains("round-trip") {
                                if let Some(stats_part) = line.split('=').nth(1) {
                                    let stats: Vec<&str> = stats_part.split('/').collect();
                                    if stats.len() >= 2 {
                                        if let Ok(avg_latency) = stats[1].trim().parse::<f64>() {
                                            return Ok(avg_latency);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Fallback: use basic UDP socket timing
        let start = Instant::now();
        let target_socket: SocketAddr = target_address.parse()
            .map_err(|_| crate::errors::SongbirdError::Service { 
                message: "Invalid target address".to_string()
            })?;
        
        if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
            let _ = socket.send_to(b"ping", target_socket);
        }
        
        let elapsed = start.elapsed();
        Ok(elapsed.as_millis() as f64)
    }

    /// Estimate bandwidth to target (simplified)
    pub async fn estimate_bandwidth(_target_address: &str) -> Result<f64> {
        // In a real implementation, this would perform an actual bandwidth test
        // For now, return a reasonable default based on network interface speed
        
        #[cfg(target_os = "linux")]
        {
            // Try to get network interface speed
            if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
                for entry in entries.flatten() {
                    let interface_name = entry.file_name();
                    let speed_path = format!("/sys/class/net/{}/speed", interface_name.to_string_lossy());
                    
                    if let Ok(speed_str) = std::fs::read_to_string(speed_path) {
                        if let Ok(speed_mbps) = speed_str.trim().parse::<f64>() {
                            if speed_mbps > 0.0 && speed_mbps < 100_000.0 {
                                return Ok(speed_mbps * 0.8); // Assume 80% efficiency
                            }
                        }
                    }
                }
            }
        }
        
        // Default assumption: 1 Gbps with 80% efficiency
        Ok(800.0)
    }

    /// Perform network measurement (ping and basic bandwidth test)
    pub async fn perform_network_measurement(target_address: &str) -> Result<(f64, f64)> {
        let latency_ms = Self::measure_ping_latency(target_address).await?;
        let bandwidth_mbps = Self::estimate_bandwidth(target_address).await?;
        
        Ok((latency_ms, bandwidth_mbps))
    }

    /// Broadcast node discovery request to federation
    pub async fn broadcast_node_discovery(local_node: &LocalNode, config: &NetworkConfig) {
        let discovery_request = FederationMessage::NodeDiscoveryRequest {
            sender_id: local_node.id.clone(),
            timestamp: chrono::Utc::now(),
        };

        if let Ok(request_json) = serde_json::to_string(&discovery_request) {
            // Broadcast to multicast group
            if let Ok(socket) = UdpSocket::bind(&format!("{}:0", config.bind_address)) {
                let multicast_addr: SocketAddr = format!("{}:{}", 
                    config.multicast_address, 
                    config.federation_port
                ).parse().unwrap();
                
                // Enable multicast
                if socket.join_multicast_v4(
                    &config.multicast_address.parse().unwrap(),
                    &config.bind_address.parse().unwrap()
                ).is_ok() {
                    if let Err(e) = socket.send_to(request_json.as_bytes(), multicast_addr) {
                        tracing::warn!("Failed to send discovery request: {}", e);
                    } else {
                        tracing::debug!("Broadcasted node discovery request from {}", local_node.id);
                    }
                }
            }
        }
    }

    /// Send node announcement to federation
    pub async fn send_node_announcement(local_node: &LocalNode, config: &NetworkConfig, current_load: ResourceUsage) {
        let announcement = FederationMessage::NodeAnnouncement {
            node: NodeInfo {
                id: local_node.id.clone(),
                address: format!("{}:{}", 
                    Self::detect_external_ip().unwrap_or_else(|| 
                        Self::detect_internal_ip().unwrap_or_else(|| "localhost".to_string())
                    ),
                    config.service_port
                ),
                node_type: local_node.node_type.clone(),
                institution: local_node.institution.clone(),
                resources: local_node.resources.clone(),
                current_load,
                available_datasets: Vec::new(), // Could be populated from local datasets
                storage_capacity: StorageInfo {
                    total_capacity_gb: local_node.resources.storage_devices
                        .iter().map(|d| d.capacity_gb).sum(),
                    available_capacity_gb: local_node.resources.storage_devices
                        .iter().map(|d| d.available_gb).sum(),
                    performance_tier_breakdown: {
                        let mut breakdown = std::collections::HashMap::new();
                        for device in &local_node.resources.storage_devices {
                            *breakdown.entry(device.performance_tier.clone()).or_insert(0) += device.capacity_gb;
                        }
                        breakdown
                    },
                },
                trust_level: TrustLevel::Institutional,
                reputation_score: 0.95,
                network_location: local_node.network_location.clone(),
                bandwidth_measurements: std::collections::HashMap::new(),
                latency_measurements: std::collections::HashMap::new(),
                last_seen: chrono::Utc::now(),
                health_status: crate::traits::discovery::ServiceHealthStatus::Healthy,
                services: Vec::new(), // Could be populated from local services
            },
            timestamp: chrono::Utc::now(),
        };

        if let Ok(announcement_json) = serde_json::to_string(&announcement) {
            if let Ok(socket) = UdpSocket::bind(&format!("{}:0", config.bind_address)) {
                let multicast_addr: SocketAddr = format!("{}:{}", 
                    config.multicast_address, 
                    config.federation_port
                ).parse().unwrap();
                
                // Enable multicast
                if socket.join_multicast_v4(
                    &config.multicast_address.parse().unwrap(),
                    &config.bind_address.parse().unwrap()
                ).is_ok() {
                    if let Err(e) = socket.send_to(announcement_json.as_bytes(), multicast_addr) {
                        tracing::warn!("Failed to send node announcement: {}", e);
                    } else {
                        tracing::info!("Sent node announcement for {} ({} bytes)", local_node.id, announcement_json.len());
                    }
                }
            }
        }
    }

    /// Send heartbeat message
    pub async fn send_heartbeat(node_id: &str, resource_usage: ResourceUsage, config: &NetworkConfig) {
        let heartbeat = FederationMessage::Heartbeat {
            node_id: node_id.to_string(),
            resource_usage,
            timestamp: chrono::Utc::now(),
        };
        
        if let Ok(heartbeat_json) = serde_json::to_string(&heartbeat) {
            if let Ok(socket) = UdpSocket::bind(&format!("{}:0", config.bind_address)) {
                let multicast_addr: SocketAddr = format!("{}:{}", 
                    config.multicast_address, 
                    config.federation_port
                ).parse().unwrap();
                let _ = socket.send_to(heartbeat_json.as_bytes(), multicast_addr);
            }
        }
    }

    /// Send service advertisement
    pub async fn send_service_advertisement(node_id: &str, services: Vec<crate::traits::service::ServiceInfo>, config: &NetworkConfig) {
        let advertisement = FederationMessage::ServiceAdvertisement {
            node_id: node_id.to_string(),
            services,
            timestamp: chrono::Utc::now(),
        };
        
        if let Ok(ad_json) = serde_json::to_string(&advertisement) {
            if let Ok(socket) = UdpSocket::bind(&format!("{}:0", config.bind_address)) {
                let multicast_addr: SocketAddr = format!("{}:{}", 
                    config.multicast_address, 
                    config.federation_port
                ).parse().unwrap();
                let _ = socket.send_to(ad_json.as_bytes(), multicast_addr);
            }
        }
    }

    /// Send discovery response
    pub async fn send_discovery_response(nodes: Vec<NodeInfo>, sender_addr: SocketAddr, config: &NetworkConfig) {
        let response = FederationMessage::NodeDiscoveryResponse {
            nodes: nodes.clone(),
            timestamp: chrono::Utc::now(),
        };
        
        // Send response back to sender
        if let Ok(response_json) = serde_json::to_string(&response) {
            // Create UDP socket and send response
            if let Ok(socket) = UdpSocket::bind(&format!("{}:0", config.bind_address)) {
                if let Err(e) = socket.send_to(response_json.as_bytes(), sender_addr) {
                    tracing::warn!("Failed to send discovery response to {}: {}", sender_addr, e);
                } else {
                    tracing::debug!("Sent discovery response to {} ({} bytes, {} nodes)", 
                        sender_addr, response_json.len(), nodes.len());
                }
            }
        }
    }

    /// Detect external IP address (simplified)
    fn detect_external_ip() -> Option<String> {
        // In a real implementation, you'd query an external service
        // For now, return None to avoid network calls in this demo
        None
    }

    /// Detect internal IP address
    fn detect_internal_ip() -> Option<String> {
        use std::net::UdpSocket;
        
        // Trick: connect to a remote address to determine local IP
        if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
            if socket.connect("8.8.8.8:80").is_ok() {
                if let Ok(local_addr) = socket.local_addr() {
                    return Some(local_addr.ip().to_string());
                }
            }
        }
        
        None
    }

    /// Estimate latency based on network location
    pub fn estimate_latency_to_node(local_location: &NetworkLocation, node_location: &NetworkLocation) -> f64 {
        let local_region = &local_location.region;
        let node_region = &node_location.region;

        if local_region == node_region {
            // Same region
            if local_location.subnet == node_location.subnet {
                5.0 // Same subnet
            } else {
                15.0 // Same region
            }
        } else if local_region.starts_with("us-") && node_region.starts_with("us-") {
            30.0 // Different US regions
        } else if local_region.starts_with("eu-") && node_region.starts_with("eu-") {
            25.0 // Different EU regions
        } else {
            100.0 // Cross-continental
        }
    }

    /// Create network measurement record
    pub fn create_network_measurement(
        target_node_id: NodeId,
        latency_ms: f64,
        bandwidth_mbps: f64,
    ) -> NetworkMeasurement {
        NetworkMeasurement {
            target_node_id,
            latency_ms,
            bandwidth_mbps,
            packet_loss_percent: 0.0, // Would need actual measurement
            jitter_ms: 0.0, // Would need actual measurement
            measured_at: chrono::Utc::now(),
        }
    }
} 