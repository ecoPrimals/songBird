//! Utility functions for orchestrator application

use anyhow::Result;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tracing::debug;

/// Get local IP address for connectivity testing
///
/// This attempts to determine the local IP address by creating a UDP socket
/// and checking which interface would be used to reach the internet.
/// No actual network traffic is generated.
pub async fn get_local_ip_for_connectivity_test() -> Result<String> {
    // Try to get local IP by creating a UDP socket (doesn't actually send data)
    let socket = std::net::UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("8.8.8.8:80")?; // Doesn't actually connect, just determines route

    if let Ok(local_addr) = socket.local_addr() {
        let ip = local_addr.ip();
        if let IpAddr::V4(ipv4) = ip {
            if ipv4 != Ipv4Addr::LOCALHOST {
                debug!("Detected local IP for connectivity test: {}", ip);
                return Ok(ip.to_string());
            }
        }
    }

    Err(anyhow::anyhow!("Could not determine local IP"))
}

/// Parse bind address with support for IPv4, IPv6, and dual-stack
///
/// Supports multiple formats:
/// - `[::]` - IPv6 wildcard (dual-stack, recommended)
/// - `[::1]` - IPv6 localhost
/// - `0.0.0.0` - IPv4 wildcard (legacy)
/// - `127.0.0.1` - IPv4 localhost
/// - Custom IPv4 or IPv6 addresses
pub fn parse_bind_address(addr: &str, port: u16) -> Result<SocketAddr> {
    use std::net::{Ipv6Addr, SocketAddr};

    match addr {
        "[::]" => {
            // Dual-stack: IPv6 wildcard (automatically handles IPv4 via IPv4-mapped addresses)
            Ok(SocketAddr::new(
                IpAddr::V6(Ipv6Addr::UNSPECIFIED),
                port,
            ))
        }
        "[::1]" => {
            // IPv6 localhost
            Ok(SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), port))
        }
        "0.0.0.0" => {
            // IPv4 wildcard (legacy mode)
            Ok(SocketAddr::new(
                IpAddr::V4(Ipv4Addr::UNSPECIFIED),
                port,
            ))
        }
        "127.0.0.1" => {
            // IPv4 localhost
            Ok(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port))
        }
        _ => {
            // Try parsing as custom address
            if let Ok(ip) = addr.parse::<IpAddr>() {
                Ok(SocketAddr::new(ip, port))
            } else {
                Err(anyhow::anyhow!(
                    "Invalid bind address: {}. Use [::]for dual-stack, [::1] for IPv6 localhost, \
                     0.0.0.0 for IPv4 wildcard, or a valid IP address",
                    addr
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bind_address_ipv6_wildcard() {
        let addr = parse_bind_address("[::]", 8080).unwrap();
        assert_eq!(addr.port(), 8080);
        assert!(addr.is_ipv6());
    }

    #[test]
    fn test_parse_bind_address_ipv4_wildcard() {
        let addr = parse_bind_address("0.0.0.0", 8080).unwrap();
        assert_eq!(addr.port(), 8080);
        assert!(addr.is_ipv4());
    }

    #[test]
    fn test_parse_bind_address_localhost() {
        let addr = parse_bind_address("127.0.0.1", 8080).unwrap();
        assert_eq!(addr.port(), 8080);
        assert_eq!(addr.ip(), IpAddr::V4(Ipv4Addr::LOCALHOST));
    }
}

