// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Network Binding Strategy
//!
//! Intelligent, zero-configuration network interface binding for Songbird.
//! Automatically detects and selects the best binding strategy based on
//! available network interfaces and capabilities.
//!
//! ## Design Principles
//!
//! 1. **Zero Configuration**: No manual address specification required
//! 2. **Capability-Based**: Bind based on what's available, not what's configured
//! 3. **Intelligent Selection**: Auto-detect IPv4/IPv6/dual-stack
//! 4. **Future-Proof**: Foundation for virtual endpoints and hot-swapping

use anyhow::{Context, Result};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use tracing::{debug, info, warn};

/// Network binding strategy determined by intelligent auto-detection
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkBindingStrategy {
    /// Bind to all available IPv4 interfaces (0.0.0.0)
    IPv4All,

    /// Bind to all available IPv6 interfaces (::)
    IPv6All,

    /// Dual-stack: Bind to both IPv4 and IPv6
    /// This is the preferred strategy when both are available
    DualStack,

    /// Bind to specific detected interface (for multi-NIC systems)
    /// String contains interface name (e.g., "eth0", "wlan0")
    Interface(String),
}

impl NetworkBindingStrategy {
    /// Intelligently detect the best binding strategy for this system
    ///
    /// ## Detection Logic
    ///
    /// 1. Detect available network interfaces
    /// 2. Check IPv4 support (non-loopback IPv4 addresses)
    /// 3. Check IPv6 support (non-loopback IPv6 addresses)
    /// 4. Select strategy:
    ///    - Both available → `DualStack` (preferred)
    ///    - IPv4 only → `IPv4All`
    ///    - IPv6 only → `IPv6All`
    ///    - Neither → Error
    ///
    /// ## Examples
    ///
    /// ```no_run
    /// use songbird_orchestrator::network::NetworkBindingStrategy;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let strategy = NetworkBindingStrategy::auto_detect().await?;
    /// println!("Detected strategy: {:?}", strategy);
    /// # Ok(())
    /// # }
    /// ```
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn auto_detect() -> Result<Self> {
        info!("🌐 Auto-detecting optimal network binding strategy...");

        // Detect network capabilities
        let capabilities = NetworkCapabilities::detect().await?;

        // Log detected capabilities
        debug!(
            "Network capabilities: IPv4={}, IPv6={}, interfaces={}",
            capabilities.has_ipv4, capabilities.has_ipv6, capabilities.interface_count
        );

        // Select strategy based on capabilities
        let strategy = match (capabilities.has_ipv4, capabilities.has_ipv6) {
            (true, true) => {
                info!("✅ Dual-stack network detected (IPv4 + IPv6)");
                info!("   Binding to both IPv4 (0.0.0.0) and IPv6 (::)");
                info!("   Maximum compatibility and future-proof");
                Self::DualStack
            }
            (true, false) => {
                info!("✅ IPv4-only network detected");
                info!("   Binding to all IPv4 interfaces (0.0.0.0)");
                Self::IPv4All
            }
            (false, true) => {
                info!("✅ IPv6-only network detected");
                info!("   Binding to all IPv6 interfaces (::)");
                Self::IPv6All
            }
            (false, false) => {
                anyhow::bail!(
                    "No usable network interfaces detected. \
                     Ensure at least one non-loopback interface is available."
                );
            }
        };

        info!("🎯 Selected binding strategy: {:?}", strategy);
        Ok(strategy)
    }

    /// Convert strategy to socket addresses for binding
    ///
    /// Returns a vector of socket addresses to bind to.
    /// `DualStack` returns both IPv4 and IPv6 addresses.
    pub fn to_socket_addrs(&self, port: u16) -> Vec<SocketAddr> {
        match self {
            Self::IPv4All => {
                vec![SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port)]
            }
            Self::IPv6All => {
                vec![SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), port)]
            }
            Self::DualStack => {
                vec![
                    SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port),
                    SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), port),
                ]
            }
            Self::Interface(name) => {
                // Known limitation: we do not call SO_BINDTODEVICE / per-interface bind APIs here.
                // `NetworkBindingStrategy::auto_detect` never selects `Interface`; this arm exists
                // for API completeness. Binding `0.0.0.0` listens on all interfaces (see tracking:
                // per-NIC bind for multi-homed hosts).
                warn!(
                    interface = %name,
                    "Interface-scoped bind not implemented; using IPv4 unspecified (0.0.0.0) on port {}",
                    port
                );
                vec![SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port)]
            }
        }
    }

    /// Get primary socket address (for single-binding scenarios)
    ///
    /// For `DualStack`, returns IPv4 address (broader compatibility).
    /// For single-stack, returns the appropriate address.
    #[must_use]
    pub const fn primary_socket_addr(&self, port: u16) -> SocketAddr {
        match self {
            Self::IPv4All => SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port),
            Self::IPv6All => SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), port),
            Self::DualStack => {
                // Prefer IPv4 for broader compatibility
                SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port)
            }
            Self::Interface(_) => {
                // Matches `to_socket_addrs`: unspecified IPv4 when interface-specific bind is unavailable.
                SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port)
            }
        }
    }

    /// Check if this strategy supports IPv4
    #[must_use]
    pub const fn supports_ipv4(&self) -> bool {
        matches!(self, Self::IPv4All | Self::DualStack | Self::Interface(_))
    }

    /// Check if this strategy supports IPv6
    #[must_use]
    pub const fn supports_ipv6(&self) -> bool {
        matches!(self, Self::IPv6All | Self::DualStack)
    }
}

/// Network capabilities detected on this system
#[derive(Debug, Clone)]
struct NetworkCapabilities {
    /// System has usable IPv4 interfaces
    has_ipv4: bool,

    /// System has usable IPv6 interfaces
    has_ipv6: bool,

    /// Number of non-loopback interfaces detected
    interface_count: usize,

    /// Primary interface name (if detectable)
    #[expect(
        dead_code,
        reason = "populated by detection; exposed when binding UI needs iface name"
    )]
    primary_interface: Option<String>,
}

impl NetworkCapabilities {
    /// Detect network capabilities on this system
    async fn detect() -> Result<Self> {
        // Method 1: Try UDP socket-based detection (fast and portable)
        if let Ok(caps) = Self::detect_via_udp_socket().await {
            return Ok(caps);
        }

        // Method 2: Fallback to interface enumeration (platform-specific)
        if let Ok(caps) = Self::detect_via_interfaces().await {
            return Ok(caps);
        }

        // Method 3: Last resort - assume IPv4 available
        warn!("Could not detect network capabilities, assuming IPv4 available");
        Ok(Self {
            has_ipv4: true,
            has_ipv6: false,
            interface_count: 1,
            primary_interface: None,
        })
    }

    /// Fast detection using UDP socket routing (doesn't actually send data)
    #[expect(
        clippy::unused_async,
        reason = "async signature required by Axum, trait objects, or future I/O"
    )]
    async fn detect_via_udp_socket() -> Result<Self> {
        use std::net::UdpSocket;

        use crate::network::route_detect::{
            primary_ipv4_from_default_interface, primary_ipv6_from_default_interface,
            route_detect_addr_v4, route_detect_addr_v6,
        };

        let mut has_ipv4 = false;
        if let Some(ip) = primary_ipv4_from_default_interface() {
            has_ipv4 = true;
            debug!("IPv4 detected via default interface: {ip}");
        }

        // Test IPv4: route probe to documentation / configured address (no public DNS IPs)
        if !has_ipv4
            && let Ok(socket) = UdpSocket::bind("0.0.0.0:0")
            && socket.connect(route_detect_addr_v4().as_str()).is_ok()
            && let Ok(local_addr) = socket.local_addr()
        {
            let ip = local_addr.ip();
            if !ip.is_loopback() && !ip.is_unspecified() {
                has_ipv4 = true;
                debug!("IPv4 detected via routing check: {}", ip);
            }
        }

        let mut has_ipv6 = false;
        if let Some(ip) = primary_ipv6_from_default_interface() {
            has_ipv6 = true;
            debug!("IPv6 detected via default interface: {ip}");
        }

        // Test IPv6: route probe to documentation IPv6 (RFC 3849) or env override
        if !has_ipv6
            && let Ok(socket) = UdpSocket::bind("[::]:0")
            && socket.connect(route_detect_addr_v6().as_str()).is_ok()
            && let Ok(local_addr) = socket.local_addr()
        {
            let ip = local_addr.ip();
            if !ip.is_loopback() && !ip.is_unspecified() {
                has_ipv6 = true;
                debug!("IPv6 detected via routing check: {}", ip);
            }
        }

        if !has_ipv4 && !has_ipv6 {
            anyhow::bail!("No routable IPv4 or IPv6 addresses detected");
        }

        Ok(Self {
            has_ipv4,
            has_ipv6,
            interface_count: if has_ipv4 && has_ipv6 {
                2
            } else {
                1
            },
            primary_interface: None,
        })
    }

    /// Fallback detection by enumerating network interfaces
    #[cfg(target_os = "linux")]
    #[expect(
        clippy::unused_async,
        reason = "async signature required by Axum, trait objects, or future I/O"
    )]
    async fn detect_via_interfaces() -> Result<Self> {
        use std::process::Command;

        let mut has_ipv4 = false;
        let mut has_ipv6 = false;
        let mut interface_count = 0;
        let mut primary_interface = None;

        // Use `ip addr` to list interfaces and addresses
        let output = Command::new("ip")
            .args(["addr", "show"])
            .output()
            .context("Failed to execute 'ip addr show'")?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        for line in stdout.lines() {
            // Look for inet (IPv4) addresses
            if line.contains("inet ") && !line.contains("127.0.0.1") {
                has_ipv4 = true;
                interface_count += 1;
            }

            // Look for inet6 (IPv6) addresses
            if line.contains("inet6 ") && !line.contains("::1") && !line.contains("fe80") {
                has_ipv6 = true;
            }

            // Try to detect primary interface name
            if primary_interface.is_none()
                && line.contains(": ")
                && let Some(name) = line.split(':').nth(1)
            {
                let name = name.trim();
                if !name.starts_with("lo") {
                    primary_interface = Some(name.to_string());
                }
            }
        }

        if !has_ipv4 && !has_ipv6 {
            anyhow::bail!("No non-loopback IPv4 or IPv6 addresses found in interfaces");
        }

        Ok(Self {
            has_ipv4,
            has_ipv6,
            interface_count,
            primary_interface,
        })
    }

    /// Fallback for non-Linux systems
    #[cfg(not(target_os = "linux"))]
    async fn detect_via_interfaces() -> Result<Self> {
        // For non-Linux, rely on UDP socket detection
        anyhow::bail!("Interface enumeration not implemented for this platform")
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;

    #[tokio::test]
    async fn test_auto_detect() {
        // Should detect something (either IPv4, IPv6, or both)
        let strategy = NetworkBindingStrategy::auto_detect().await;
        assert!(strategy.is_ok(), "Auto-detection should succeed");

        let strategy = strategy.unwrap();
        println!("Detected strategy: {strategy:?}");
    }

    #[test]
    fn test_to_socket_addrs_ipv4() {
        let strategy = NetworkBindingStrategy::IPv4All;
        let addrs = strategy.to_socket_addrs(8080);

        assert_eq!(addrs.len(), 1);
        assert_eq!(addrs[0], "0.0.0.0:8080".parse::<SocketAddr>().unwrap());
    }

    #[test]
    fn test_to_socket_addrs_ipv6() {
        let strategy = NetworkBindingStrategy::IPv6All;
        let addrs = strategy.to_socket_addrs(8080);

        assert_eq!(addrs.len(), 1);
        assert_eq!(addrs[0], "[::]:8080".parse::<SocketAddr>().unwrap());
    }

    #[test]
    fn test_to_socket_addrs_dual_stack() {
        let strategy = NetworkBindingStrategy::DualStack;
        let addrs = strategy.to_socket_addrs(8080);

        assert_eq!(addrs.len(), 2);
        assert!(addrs.contains(&"0.0.0.0:8080".parse::<SocketAddr>().unwrap()));
        assert!(addrs.contains(&"[::]:8080".parse::<SocketAddr>().unwrap()));
    }

    #[test]
    fn test_primary_socket_addr() {
        let ipv4_strategy = NetworkBindingStrategy::IPv4All;
        assert_eq!(
            ipv4_strategy.primary_socket_addr(8080),
            "0.0.0.0:8080".parse::<SocketAddr>().unwrap()
        );

        let ipv6_strategy = NetworkBindingStrategy::IPv6All;
        assert_eq!(
            ipv6_strategy.primary_socket_addr(8080),
            "[::]:8080".parse::<SocketAddr>().unwrap()
        );

        // DualStack prefers IPv4 for compatibility
        let dual_strategy = NetworkBindingStrategy::DualStack;
        assert_eq!(
            dual_strategy.primary_socket_addr(8080),
            "0.0.0.0:8080".parse::<SocketAddr>().unwrap()
        );
    }

    #[test]
    fn test_supports_protocols() {
        let ipv4 = NetworkBindingStrategy::IPv4All;
        assert!(ipv4.supports_ipv4());
        assert!(!ipv4.supports_ipv6());

        let ipv6 = NetworkBindingStrategy::IPv6All;
        assert!(!ipv6.supports_ipv4());
        assert!(ipv6.supports_ipv6());

        let dual = NetworkBindingStrategy::DualStack;
        assert!(dual.supports_ipv4());
        assert!(dual.supports_ipv6());
    }

    #[test]
    fn test_to_socket_addrs_interface_falls_back_to_ipv4_unspecified() {
        let strategy = NetworkBindingStrategy::Interface("eth0".to_string());
        let addrs = strategy.to_socket_addrs(9000);
        assert_eq!(addrs.len(), 1);
        assert_eq!(addrs[0], "0.0.0.0:9000".parse::<SocketAddr>().unwrap());
    }

    #[test]
    fn test_primary_socket_addr_interface_matches_ipv4_all_shape() {
        let strategy = NetworkBindingStrategy::Interface("wlan0".to_string());
        assert_eq!(
            strategy.primary_socket_addr(4444),
            NetworkBindingStrategy::IPv4All.primary_socket_addr(4444)
        );
    }

    #[test]
    fn test_interface_strategy_supports_ipv4_not_ipv6() {
        let strategy = NetworkBindingStrategy::Interface("tailscale0".to_string());
        assert!(strategy.supports_ipv4());
        assert!(!strategy.supports_ipv6());
    }

    #[test]
    fn test_dual_stack_socket_addrs_distinct_families() {
        let addrs = NetworkBindingStrategy::DualStack.to_socket_addrs(3030);
        let v4: SocketAddr = "0.0.0.0:3030".parse().unwrap();
        let v6: SocketAddr = "[::]:3030".parse().unwrap();
        assert!(addrs.contains(&v4));
        assert!(addrs.contains(&v6));
        assert_ne!(addrs[0], addrs[1]);
    }

    #[test]
    fn test_ipv6_all_single_wildcard() {
        let addrs = NetworkBindingStrategy::IPv6All.to_socket_addrs(7);
        assert_eq!(addrs.len(), 1);
        assert!(addrs[0].ip().is_ipv6());
    }
}
