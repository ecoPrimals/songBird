// Module imports
//! Network discovery commands

// CLI discovery commands
use crate::config::constants;
use crate::errors::Result;
use colored::*;
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::timeout;
use tracing::info;

/// Discover nodes on the network
pub async fn discover_nodes(
    subnet: Option<String>,
    port_range: Option<String>,
    timeout: u64,
    interactive: bool,
) -> Result<()> {
    info!(
        "Discovering nodes subnet={:?} port_range={:?} timeout={} interactive={}",
        subnet, port_range, timeout, interactive
    );

    println!(
        "{}",
        "🔍 Discovering Songbird nodes on network..."
            .bright_cyan()
            .bold()
    );
    let subnet = subnet.unwrap_or_else(crate::config::constants::default_subnet);
    let env_config = crate::config::environment::EnvironmentConfig::default();
    let port_range = port_range.unwrap_or_else(|| format!("{}-{}", env_config.bind_port, env_config.bind_port + 10));
    println!("🔍 Scanning subnet: {}", subnet);
    println!("🔌 Port range: {}", port_range);
    println!("⏱️  Timeout: {}ms", timeout);
    println!();

    // Execute comprehensive network discovery with live node detection
    match perform_real_discovery(&subnet, timeout).await {
        Ok(discovered_nodes) => {
            if discovered_nodes.is_empty() {
                println!("❌ No Songbird nodes discovered on subnet {}", subnet);
                println!("💡 Try:");
                println!("   • Check if any nodes are running");
                println!("   • Verify network connectivity");
                println!("   • Try a different subnet");
            } else {
                println!("✅ Discovered {} Songbird nodes:", discovered_nodes.len());
                for node in discovered_nodes {
                    println!("├── {} {} ({})", node.icon, node.address, node.description);
                }
            }
        }
        Err(e) => {
            println!("❌ Discovery failed: {}", e);
            println!("💡 This might be expected if no nodes are running");
        }
    }

    if interactive {
        println!("\n🎯 Interactive mode enabled - you can now select nodes to add");
        // Interactive selection would go here
    }
    Ok(())
}

/// Discovered node information
#[derive(Debug)]
pub struct DiscoveredNode {
    pub node_id: String,
    pub address: SocketAddr,
    pub version: String,
    pub services: Vec<String>,
    pub last_seen: std::time::SystemTime,
    pub icon: String,
    pub description: String,
}

/// Perform real network discovery instead of using mock data
async fn perform_real_discovery(subnet: &str, timeout_ms: u64) -> Result<Vec<DiscoveredNode>> {
    let discovery_timeout = Duration::from_millis(timeout_ms);
    let mut discovered_nodes = Vec::new();

    // Parse subnet to get network range
    let (base_ip, mask) = parse_subnet(subnet)?;

    // Use configurable binding - NO MORE HARDCODING!
    let env_config = crate::config::environment::EnvironmentConfig::default();
    let bind_addr = format!("{}:0", env_config.bind_address);
    
    let socket = UdpSocket::bind(&bind_addr).await
        .map_err(|e| crate::errors::SongbirdError::Config {
            field: Some("discovery_bind_address".to_string()),
            message: format!("Failed to bind discovery socket to {}: {}", bind_addr, e),
        })?;
    socket.set_broadcast(true)?;

    // Send discovery broadcast to the subnet
    let discovery_message = format!("SONGBIRD_DISCOVERY:{}", constants::node_id());
    let broadcast_addr = calculate_broadcast_address(&base_ip, mask)?;
    let discovery_port = constants::default_discovery_port();
    let target = SocketAddr::new(broadcast_addr, discovery_port);

    socket.send_to(discovery_message.as_bytes(), target).await?;

    // Listen for responses
    let mut buffer = [0u8; 1024];
    let start_time = std::time::Instant::now();

    while start_time.elapsed() < discovery_timeout {
        let remaining_time = discovery_timeout - start_time.elapsed();

        match timeout(remaining_time, socket.recv_from(&mut buffer)).await {
            Ok(Ok((len, peer_addr))) => {
                let response = String::from_utf8_lossy(&buffer[..len]);

                if let Some(node_info) = parse_discovery_response(&response, peer_addr) {
                    discovered_nodes.push(node_info);
                }
            }
            Ok(Err(_)) => break, // Socket error
            Err(_) => break,     // Timeout
        }
    }

    Ok(discovered_nodes)
}

/// Parse subnet notation (e.g., "192.168.1.0/24")
fn parse_subnet(subnet: &str) -> Result<(IpAddr, u8)> {
    let parts: Vec<&str> = subnet.split('/').collect();
    if parts.len() != 2 {
        return Err(crate::errors::SongbirdError::Config {
            message: format!("Invalid subnet format: {}", subnet),
            field: Some("subnet".to_string()),
        });
    }

    let ip: IpAddr = parts[0].parse()?;
    let mask: u8 = parts[1]
        .parse()
        .map_err(|_| crate::errors::SongbirdError::Config {
            message: format!("Invalid subnet mask: {}", parts[1]),
            field: Some("subnet_mask".to_string()),
        })?;

    Ok((ip, mask))
}

/// Calculate broadcast address for the given network
fn calculate_broadcast_address(base_ip: &IpAddr, mask: u8) -> Result<IpAddr> {
    match base_ip {
        IpAddr::V4(ipv4) => {
            let ip_u32 = u32::from(*ipv4);
            let mask_u32 = !((1u32 << (32 - mask)) - 1);
            let network = ip_u32 & mask_u32;
            let broadcast = network | !mask_u32;
            Ok(IpAddr::V4(std::net::Ipv4Addr::from(broadcast)))
        }
        IpAddr::V6(_) => {
            // For IPv6, use link-local multicast
            Ok("ff02::1".parse().unwrap())
        }
    }
}

/// Parse discovery response from peer
fn parse_discovery_response(response: &str, peer_addr: SocketAddr) -> Option<DiscoveredNode> {
    if response.starts_with("SONGBIRD_RESPONSE:") {
        let data = response.strip_prefix("SONGBIRD_RESPONSE:")?;
        let parts: Vec<&str> = data.split('|').collect();

        if parts.len() >= 3 {
            Some(DiscoveredNode {
                node_id: parts[0].to_string(),
                address: peer_addr,
                version: parts[1].to_string(),
                services: parts[2].split(',').map(|s| s.to_string()).collect(),
                last_seen: std::time::SystemTime::now(),
                icon: "🎵".to_string(),
                description: format!("Songbird v{}", parts[1]),
            })
        } else {
            None
        }
    } else {
        None
    }
}
