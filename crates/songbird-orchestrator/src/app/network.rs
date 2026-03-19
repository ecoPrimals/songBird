//! Network utilities for orchestrator
//!
//! Provides network detection and configuration utilities with modern,
//! idiomatic Rust patterns.

use anyhow::Result;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket};
use tracing::{info, warn};

/// Get local IP address for connectivity testing
///
/// Uses a technique that creates a UDP socket to determine which interface
/// would be used for external connectivity, without actually sending data.
pub async fn get_local_ip_for_connectivity_test() -> Result<String> {
    // Try to get local IP by creating a UDP socket (doesn't actually send data)
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("8.8.8.8:80")?; // Doesn't actually connect, just determines route

    if let Ok(local_addr) = socket.local_addr() {
        let ip = local_addr.ip();
        if let std::net::IpAddr::V4(ipv4) = ip {
            if ipv4 != Ipv4Addr::LOCALHOST {
                return Ok(ip.to_string());
            }
        }
    }

    Err(anyhow::anyhow!("Could not determine local IP"))
}

/// Parse bind address with support for IPv4, IPv6, and dual-stack
///
/// # Supported Formats
///
/// - `[::]` - IPv6 wildcard (dual-stack, **recommended**)
/// - `[::1]` - IPv6 localhost
/// - `0.0.0.0` - IPv4 wildcard (legacy)
/// - `127.0.0.1` - IPv4 localhost
/// - Custom IPv4 or IPv6 addresses
///
/// # Examples
///
/// ```rust,ignore
/// use songbird_orchestrator::app::network::parse_bind_address;
///
/// // Dual-stack (recommended)
/// let addr = parse_bind_address("[::]", 8080)?;
///
/// // IPv4 localhost
/// let addr = parse_bind_address("127.0.0.1", 8080)?;
/// ```
pub fn parse_bind_address(addr: &str, port: u16) -> Result<SocketAddr> {
    match addr {
        "[::]" => {
            // Dual-stack: IPv6 wildcard (automatically handles IPv4 via IPv4-mapped addresses)
            Ok(SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), port))
        }
        "[::1]" => {
            // IPv6 localhost
            Ok(SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), port))
        }
        "0.0.0.0" => {
            // IPv4 wildcard (legacy mode)
            Ok(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port))
        }
        "127.0.0.1" => {
            // IPv4 localhost
            Ok(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port))
        }
        _ => {
            // Try to parse as IPv6 format: [addr] or custom address
            if addr.starts_with('[') && addr.ends_with(']') {
                let ip_part = addr.trim_start_matches('[').trim_end_matches(']');
                let ip: IpAddr = ip_part
                    .parse()
                    .map_err(|e| anyhow::anyhow!("Invalid IPv6 address '{ip_part}': {e}"))?;
                Ok(SocketAddr::new(ip, port))
            } else {
                // Try as IPv4 address or parse full socket address
                format!("{addr}:{port}")
                    .parse()
                    .map_err(|e| anyhow::anyhow!("Invalid bind address '{addr}': {e}"))
            }
        }
    }
}

/// Detect primary network interface IP address
///
/// Attempts multiple strategies to find the primary network interface:
/// 1. UDP socket trick to find routing interface
/// 2. Platform-specific commands (Linux: `ip route`, `hostname -I`)
///
/// Returns `None` if no suitable interface is found.
pub fn detect_primary_ip() -> Option<String> {
    // Try to detect by creating a UDP socket to a public DNS server
    // This doesn't actually send data, just determines which interface would be used
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
        if matches!(socket.connect("8.8.8.8:80"), Ok(())) {
            if let Ok(addr) = socket.local_addr() {
                let ip = addr.ip();
                // Only return if it's a real IP (not 0.0.0.0 or loopback)
                if !ip.is_loopback() && !ip.is_unspecified() {
                    info!("🌐 Detected primary network IP: {}", ip);
                    return Some(ip.to_string());
                }
            }
        }
    }

    // Fallback: Try to get from network interfaces
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;

        // Try ip command first
        if let Ok(output) = Command::new("ip").args(["route", "get", "1.1.1.1"]).output() {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                // Parse output like: "1.1.1.1 via X.X.X.X dev eth0 src Y.Y.Y.Y"
                for line in stdout.lines() {
                    if let Some(src_pos) = line.find(" src ") {
                        let after_src = &line[src_pos + 5..];
                        if let Some(ip_str) = after_src.split_whitespace().next() {
                            if let Ok(ip) = ip_str.parse::<IpAddr>() {
                                if !ip.is_loopback() && !ip.is_unspecified() {
                                    info!("🌐 Detected primary network IP: {}", ip);
                                    return Some(ip.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        // Fallback to hostname -I
        if let Ok(output) = Command::new("hostname").arg("-I").output() {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                // Get first non-loopback IP
                for ip_str in stdout.split_whitespace() {
                    if let Ok(ip) = ip_str.parse::<IpAddr>() {
                        if !ip.is_loopback() && !ip.is_unspecified() {
                            info!("🌐 Detected primary network IP: {}", ip);
                            return Some(ip.to_string());
                        }
                    }
                }
            }
        }
    }

    warn!("⚠️  Could not detect primary network IP, using fallback");
    None
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
        let addr = parse_bind_address("127.0.0.1", 3000).unwrap();
        assert_eq!(addr.port(), 3000);
        assert!(addr.ip().is_loopback());
    }
}
