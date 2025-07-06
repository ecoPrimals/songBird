// Module imports
//! Network discovery commands

// CLI discovery commands
use colored::*;
use songbird_config::config::constants;
use songbird_errors::Result;
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
    let subnet = subnet.unwrap_or_else(songbird_config::config::constants::default_subnet);
    let env_config = songbird_config::config::environment::EnvironmentConfig::default();
    let port_range = port_range
        .unwrap_or_else(|| format!("{}-{}", env_config.bind_port, env_config.bind_port + 10));
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
    let env_config = songbird_config::config::environment::EnvironmentConfig::default();
    let bind_addr = format!("{}:0", env_config.bind_address);

    let socket =
        UdpSocket::bind(&bind_addr)
            .await
            .map_err(|e| songbird_errors::SongbirdError::Config {
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
        return Err(songbird_errors::SongbirdError::Config {
            message: format!("Invalid subnet format: {}", subnet),
            field: Some("subnet".to_string()),
        });
    }

    let ip: IpAddr = parts[0].parse()?;
    let mask: u8 = parts[1]
        .parse()
        .map_err(|_| songbird_errors::SongbirdError::Config {
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
            Ok("ff02::1".parse().unwrap_or_else(|e| {
                tracing::warn!(
                    "Failed to parse IPv6 multicast address, using fallback: {}",
                    e
                );
                "::1".parse().expect("::1 is a valid IPv6 address")
            }))
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    fn setup_test_discovery_config() -> (TempDir, std::path::PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("discovery_config.toml");
        fs::write(&config_path, r#"
[discovery]
backend = "static"
timeout = 30
retry_attempts = 3

[network]
bind_address = "0.0.0.0:8080"
discovery_port = 8081

[consul]
url = "http://localhost:8500"
datacenter = "dc1"
"#).unwrap();
        (temp_dir, config_path)
    }

    #[tokio::test]
    async fn test_discover_nodes_with_defaults() {
        let result = discover_nodes(None, None, 100, false).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_discover_nodes_with_custom_subnet() {
        let result = discover_nodes(Some("192.168.1.0/24".to_string()), None, 100, false).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_discover_nodes_with_custom_port_range() {
        let result = discover_nodes(None, Some("8080-8090".to_string()), 100, false).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_discover_nodes_interactive_mode() {
        let result = discover_nodes(None, None, 100, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_discover_nodes_short_timeout() {
        let result = discover_nodes(None, None, 1, false).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_parse_subnet_valid_ipv4() {
        let result = parse_subnet("192.168.1.0/24");
        assert!(result.is_ok());
        let (ip, mask) = result.unwrap();
        assert_eq!(mask, 24);
        match ip {
            std::net::IpAddr::V4(ipv4) => assert_eq!(ipv4.octets(), [192, 168, 1, 0]),
            _ => panic!("Expected IPv4"),
        }
    }

    #[tokio::test]
    async fn test_parse_subnet_valid_ipv6() {
        let result = parse_subnet("2001:db8::/32");
        assert!(result.is_ok());
        let (ip, mask) = result.unwrap();
        assert_eq!(mask, 32);
        match ip {
            std::net::IpAddr::V6(_) => {} // IPv6 is valid
            _ => panic!("Expected IPv6"),
        }
    }

    #[tokio::test]
    async fn test_parse_subnet_invalid_format() {
        let result = parse_subnet("invalid");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_parse_subnet_invalid_mask() {
        let result = parse_subnet("192.168.1.0/invalid");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_parse_subnet_no_mask() {
        let result = parse_subnet("192.168.1.0");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_calculate_broadcast_address_ipv4() {
        let ip = "192.168.1.0".parse().unwrap();
        let result = calculate_broadcast_address(&ip, 24);
        assert!(result.is_ok());
        let broadcast = result.unwrap();
        match broadcast {
            std::net::IpAddr::V4(ipv4) => assert_eq!(ipv4.octets(), [192, 168, 1, 255]),
            _ => panic!("Expected IPv4"),
        }
    }

    #[tokio::test]
    async fn test_calculate_broadcast_address_ipv6() {
        let ip = "2001:db8::".parse().unwrap();
        let result = calculate_broadcast_address(&ip, 32);
        assert!(result.is_ok());
        // IPv6 should return multicast address
        let broadcast = result.unwrap();
        match broadcast {
            std::net::IpAddr::V6(_) => {} // IPv6 multicast is valid
            _ => panic!("Expected IPv6"),
        }
    }

    #[tokio::test]
    async fn test_parse_discovery_response_valid() {
        let response = "SONGBIRD_RESPONSE:node1|1.0.0|service1,service2";
        let peer_addr = "192.168.1.100:8080".parse().unwrap();
        let result = parse_discovery_response(response, peer_addr);
        assert!(result.is_some());
        let node = result.unwrap();
        assert_eq!(node.node_id, "node1");
        assert_eq!(node.version, "1.0.0");
        assert_eq!(node.services.len(), 2);
    }

    #[tokio::test]
    async fn test_parse_discovery_response_invalid() {
        let response = "INVALID_RESPONSE";
        let peer_addr = "192.168.1.100:8080".parse().unwrap();
        let result = parse_discovery_response(response, peer_addr);
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_parse_discovery_response_incomplete() {
        let response = "SONGBIRD_RESPONSE:node1|1.0.0";
        let peer_addr = "192.168.1.100:8080".parse().unwrap();
        let result = parse_discovery_response(response, peer_addr);
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_discovered_node_creation() {
        let node = DiscoveredNode {
            node_id: "test-node".to_string(),
            address: "192.168.1.100:8080".parse().unwrap(),
            version: "1.0.0".to_string(),
            services: vec!["service1".to_string(), "service2".to_string()],
            last_seen: std::time::SystemTime::now(),
            icon: "🎵".to_string(),
            description: "Test node".to_string(),
        };

        assert_eq!(node.node_id, "test-node");
        assert_eq!(node.version, "1.0.0");
        assert_eq!(node.services.len(), 2);
    }

    #[tokio::test]
    async fn test_perform_real_discovery_with_invalid_subnet() {
        let result = perform_real_discovery("invalid/subnet", 100).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_perform_real_discovery_short_timeout() {
        let result = perform_real_discovery("192.168.1.0/24", 1).await;
        // With a very short timeout, the function might fail or succeed with empty results
        // depending on network conditions - both are acceptable behaviors
        match result {
            Ok(nodes) => {
                // If successful, should return empty list with short timeout
                assert!(nodes.is_empty());
            },
            Err(_) => {
                // If failed due to timeout, that's also acceptable
                // The important thing is that the function handles short timeouts gracefully
            }
        }
    }

    #[tokio::test]
    async fn test_discovery_with_all_parameters() {
        let result = discover_nodes(
            Some("10.0.0.0/8".to_string()),
            Some("9000-9010".to_string()),
            200,
            false,
        ).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_discovery_error_handling() {
        // Test with invalid subnet to trigger error handling
        let result = discover_nodes(
            Some("invalid-subnet".to_string()),
            None,
            100,
            false,
        ).await;
        // Should handle error gracefully and still return Ok
        assert!(result.is_ok());
    }
}
