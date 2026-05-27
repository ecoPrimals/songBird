// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Network utilities for orchestrator
//!
//! Provides network detection and configuration utilities with modern,
//! idiomatic Rust patterns.

use anyhow::Result;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use tracing::{info, warn};

use crate::network::route_detect::{ROUTE_GET_TARGET_V4, resolve_local_ipv4};

/// Get local IP address for connectivity testing
///
/// Uses a technique that creates a UDP socket to determine which interface
/// would be used for external connectivity, without actually sending data.
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn get_local_ip_for_connectivity_test() -> Result<String> {
    resolve_local_ipv4()
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
/// # Errors
///
/// Returns an error if the operation fails.
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
        addr if addr == songbird_types::constants::PRODUCTION_BIND_ADDRESS => {
            Ok(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port))
        }
        addr if addr == songbird_types::constants::DEVELOPMENT_BIND_ADDRESS => {
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
    if let Ok(ip) = resolve_local_ipv4() {
        info!("🌐 Detected primary network IP: {ip}");
        return Some(ip);
    }

    // Fallback: Try to get from network interfaces
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;

        // Try ip command first (documentation IPv4 — [RFC 5737] TEST-NET-1)
        if let Ok(output) = Command::new("ip").args(["route", "get", ROUTE_GET_TARGET_V4]).output()
            && let Ok(stdout) = String::from_utf8(output.stdout)
        {
            // Parse output like: "192.0.2.1 via X.X.X.X dev eth0 src Y.Y.Y.Y"
            for line in stdout.lines() {
                if let Some(src_pos) = line.find(" src ") {
                    let after_src = &line[src_pos + 5..];
                    if let Some(ip_str) = after_src.split_whitespace().next()
                        && let Ok(ip) = ip_str.parse::<IpAddr>()
                        && !ip.is_loopback()
                        && !ip.is_unspecified()
                    {
                        info!("🌐 Detected primary network IP: {}", ip);
                        return Some(ip.to_string());
                    }
                }
            }
        }

        // Fallback to hostname -I
        if let Ok(output) = Command::new("hostname").arg("-I").output()
            && let Ok(stdout) = String::from_utf8(output.stdout)
        {
            // Get first non-loopback IP
            for ip_str in stdout.split_whitespace() {
                if let Ok(ip) = ip_str.parse::<IpAddr>()
                    && !ip.is_loopback()
                    && !ip.is_unspecified()
                {
                    info!("🌐 Detected primary network IP: {}", ip);
                    return Some(ip.to_string());
                }
            }
        }
    }

    warn!("⚠️  Could not detect primary network IP, using fallback");
    None
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bind_address_ipv6_wildcard() {
        let addr = parse_bind_address("[::]", 8080).unwrap();
        assert_eq!(addr.port(), 8080);
        assert!(addr.is_ipv6());
        assert_eq!(addr.ip(), IpAddr::V6(Ipv6Addr::UNSPECIFIED));
    }

    #[test]
    fn test_parse_bind_address_ipv6_localhost() {
        let addr = parse_bind_address("[::1]", 9090).unwrap();
        assert_eq!(addr.port(), 9090);
        assert!(addr.is_ipv6());
        assert!(addr.ip().is_loopback());
    }

    #[test]
    fn test_parse_bind_address_ipv4_wildcard() {
        let addr = parse_bind_address("0.0.0.0", 8080).unwrap();
        assert_eq!(addr.port(), 8080);
        assert!(addr.is_ipv4());
        assert_eq!(addr.ip(), IpAddr::V4(Ipv4Addr::UNSPECIFIED));
    }

    #[test]
    fn test_parse_bind_address_localhost() {
        let addr = parse_bind_address("127.0.0.1", 3000).unwrap();
        assert_eq!(addr.port(), 3000);
        assert!(addr.ip().is_loopback());
    }

    #[test]
    fn test_parse_bind_address_custom_ipv4() {
        let addr = parse_bind_address("10.0.0.5", 4000).unwrap();
        assert_eq!(addr.port(), 4000);
        assert_eq!(addr.ip().to_string(), "10.0.0.5");
    }

    #[test]
    fn test_parse_bind_address_bracketed_ipv6() {
        let addr = parse_bind_address("[fe80::1]", 5000).unwrap();
        assert_eq!(addr.port(), 5000);
        assert!(addr.is_ipv6());
    }

    #[test]
    fn test_parse_bind_address_invalid_ipv6() {
        let result = parse_bind_address("[not-an-ipv6]", 1234);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid IPv6"));
    }

    #[test]
    fn test_parse_bind_address_invalid_format() {
        let result = parse_bind_address("not.a.valid.address.at.all", 80);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_bind_address_empty_brackets() {
        let result = parse_bind_address("[]", 80);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_bind_address_preserves_port() {
        for port in [0u16, 1, 80, 443, 8080, 65535] {
            let addr = parse_bind_address("127.0.0.1", port).unwrap();
            assert_eq!(addr.port(), port);
        }
    }

    #[test]
    fn test_detect_primary_ip_returns_some_or_none() {
        let result = detect_primary_ip();
        if let Some(ip) = result {
            let parsed: IpAddr = ip.parse().expect("must parse as IP");
            assert!(!parsed.is_loopback());
            assert!(!parsed.is_unspecified());
        }
    }
}
