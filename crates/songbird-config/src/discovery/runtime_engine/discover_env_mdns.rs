// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use songbird_types::{SongbirdError, SongbirdResult};
use tracing::debug;

use super::{CapabilityDiscoveryEngine, DiscoveredService};

/// Discover from environment variables
pub(super) async fn discover_from_environment(
    engine: &CapabilityDiscoveryEngine,
    capability: &str,
) -> SongbirdResult<Vec<DiscoveredService>> {
    let env_key = format!("{}_ENDPOINT", capability.to_uppercase());

    if let Ok(endpoint) = engine.read_env(&env_key) {
        // Parse address
        let addr: std::net::SocketAddr = endpoint
            .trim_start_matches("http://")
            .trim_start_matches("https://")
            .parse()
            .map_err(|e| SongbirdError::validation(format!("Invalid endpoint format: {e}")))?;

        let service = DiscoveredService {
            address: addr,
            capabilities: vec![capability.to_string()],
            metadata: std::collections::HashMap::from([(
                String::from("source"),
                String::from("environment"),
            )]),
            discovered_at: std::time::SystemTime::now(),
        };

        Ok(vec![service])
    } else {
        Ok(Vec::new())
    }
}

/// Discover from mDNS (local network)
pub(super) async fn discover_from_mdns(
    _engine: &CapabilityDiscoveryEngine,
    capability: &str,
) -> SongbirdResult<Vec<DiscoveredService>> {
    // Use our production mDNS implementation
    use crate::discovery::mdns::MdnsDiscovery;

    let mdns = MdnsDiscovery::new().map_err(|e| SongbirdError::discovery(e.to_string()))?;
    let services = mdns
        .discover_by_capability(capability, Some(std::time::Duration::from_secs(5)))
        .await
        .map_err(|e| SongbirdError::discovery(e.to_string()))?;

    // Convert to DiscoveredService format
    let discovered: Vec<DiscoveredService> = services
        .into_iter()
        .map(|s| DiscoveredService {
            address: s.address,
            capabilities: s.capabilities,
            metadata: s.metadata,
            discovered_at: s.discovered_at,
        })
        .collect();

    Ok(discovered)
}

/// Discover from DNS-SD (delegates to mDNS with DNS-SD semantics).
///
/// DNS-SD (RFC 6763) is built on top of mDNS — the `mdns-sd` crate
/// handles both protocols. This backend uses the same mDNS discovery
/// infrastructure with DNS-SD service type resolution.
pub(super) async fn discover_from_dnssd(
    engine: &CapabilityDiscoveryEngine,
    capability: &str,
) -> SongbirdResult<Vec<DiscoveredService>> {
    debug!(
        target: "songbird_config::discovery",
        backend = "dnssd",
        %capability,
        "DNS-SD discovery delegating to mDNS infrastructure (RFC 6763)"
    );
    discover_from_mdns(engine, capability).await
}
