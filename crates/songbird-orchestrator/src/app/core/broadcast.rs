// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Broadcast address discovery for cross-interface peer discovery.

use tracing::{info, warn};

use super::SongbirdOrchestrator;

impl SongbirdOrchestrator {
    /// Discover broadcast addresses with capability-based fallback (NEW - Jan 28, 2026)
    ///
    /// **Zero Hardcoding Philosophy**: Discovers broadcast addresses at runtime from:
    /// 1. Environment variable: `SONGBIRD_BROADCAST_ADDRESSES` (comma-separated)
    /// 2. Configuration file: `discovery.broadcast_addresses`
    /// 3. Automatic fallback: Subnet broadcast for cross-interface discovery
    ///
    /// **Cross-Interface Discovery**: Automatically adds subnet broadcast addresses
    /// to handle eth ↔ wifi boundaries that multicast can't cross on consumer routers.
    pub(crate) fn discover_broadcast_addresses(
        configured_addrs: &[String],
    ) -> Vec<std::net::SocketAddr> {
        use std::net::{IpAddr, Ipv4Addr, SocketAddr};

        let port = songbird_types::defaults::network::broadcast_discovery_port();

        // Priority 1: Environment variable (runtime override)
        if let Ok(env_addrs) = songbird_process_env::var("SONGBIRD_BROADCAST_ADDRESSES")
            && !env_addrs.is_empty()
        {
            info!("🌐 Using broadcast addresses from SONGBIRD_BROADCAST_ADDRESSES");
            let addrs: Vec<SocketAddr> =
                env_addrs.split(',').filter_map(|s| s.trim().parse().ok()).collect();

            if !addrs.is_empty() {
                info!("   Addresses: {:?}", addrs);
                return addrs;
            }
        }

        // Priority 2: Configuration file
        let mut addrs: Vec<SocketAddr> =
            configured_addrs.iter().filter_map(|addr| addr.parse().ok()).collect();

        // Priority 3: Add subnet broadcast fallback if not already present
        let default_fallbacks = [
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 255)), port),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 0, 255)), port),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 255)), port),
        ];

        for fallback_addr in &default_fallbacks {
            if !addrs.iter().any(|a| a.ip() == fallback_addr.ip()) {
                addrs.push(*fallback_addr);
            }
        }

        if addrs.is_empty() {
            warn!("⚠️  No broadcast addresses configured, using defaults");
            addrs = vec![
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(224, 0, 0, 251)), port),
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 255)), port),
            ];
        }

        info!("🌐 Discovery broadcast addresses: {:?}", addrs);
        addrs
    }
}
