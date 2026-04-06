// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Discovery → federation routing rules and matchers.

use super::{FederationId, IpNetwork};
use songbird_discovery::anonymous::DiscoveredPeer;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

/// Discovery router - routes discovered peers to appropriate federations
pub struct DiscoveryRouter {
    /// Routing rules (checked in order of priority)
    routing_rules: Arc<RwLock<Vec<RoutingRule>>>,

    /// Default federation (if no rules match)
    default_federation: Option<FederationId>,
}

impl DiscoveryRouter {
    /// Create a new discovery router
    #[must_use]
    pub fn new(default_federation: Option<FederationId>) -> Self {
        Self {
            routing_rules: Arc::new(RwLock::new(Vec::new())),
            default_federation,
        }
    }

    /// Add a routing rule
    pub async fn add_rule(&self, rule: RoutingRule) {
        let mut rules = self.routing_rules.write().await;
        rules.push(rule);
        // Sort by priority (highest first)
        rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Route a discovered peer to appropriate federation(s)
    pub async fn route(&self, peer: &DiscoveredPeer) -> Vec<FederationId> {
        let rules_snapshot: Vec<RoutingRule> = self.routing_rules.read().await.clone();
        let mut matches = Vec::new();
        for rule in rules_snapshot {
            if rule.matcher.matches(peer) {
                let fed_id = rule.target_federation.0;
                matches.push(rule.target_federation);
                debug!(
                    "Peer {} matched rule (priority {}) → federation {}",
                    peer.session_id, rule.priority, fed_id
                );
            }
        }

        // If no matches, use default
        if matches.is_empty()
            && let Some(ref default) = self.default_federation
        {
            debug!("Peer {} using default federation {}", peer.session_id, default.0);
            matches.push(default.clone());
        }

        matches
    }
}

/// Routing rule for discovery → federation mapping
#[derive(Clone)]
pub struct RoutingRule {
    /// Match criteria
    pub matcher: RoutingMatcher,

    /// Target federation
    pub target_federation: FederationId,

    /// Priority (higher = checked first)
    pub priority: u32,
}

/// Routing matcher - determines if a peer matches this rule
#[derive(Clone)]
pub enum RoutingMatcher {
    /// Match by IP subnet (e.g., family = 192.168.1.0/24)
    IpSubnet(IpNetwork),

    /// Match by capabilities (e.g., has "academic" → school)
    HasCapability(String),

    /// Match all (always matches)
    All,
}

impl RoutingMatcher {
    /// Check if a peer matches this matcher
    #[must_use]
    pub fn matches(&self, peer: &DiscoveredPeer) -> bool {
        match self {
            Self::IpSubnet(network) => network.contains(peer.address.ip()),
            Self::HasCapability(cap) => peer.capabilities.contains(cap),
            Self::All => true,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::time::SystemTime;

    fn sample_peer(ip: [u8; 4]) -> DiscoveredPeer {
        DiscoveredPeer {
            node_id: None,
            node_name: None,
            session_id: "s".into(),
            endpoints: None,
            capabilities: vec!["academic".into()],
            protocols: vec!["https".into()],
            port: 8080,
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::from(ip)), 2300),
            last_seen: SystemTime::now(),
            version: "2.1".into(),
            tags: None,
            timestamp: None,
            identity_attestations: Some(vec![]),
        }
    }

    #[tokio::test]
    async fn test_discovery_router() {
        let family_id = FederationId::new();
        let school_id = FederationId::new();

        let router = DiscoveryRouter::new(Some(family_id.clone()));

        // Add school rule (IP-based)
        router
            .add_rule(RoutingRule {
                matcher: RoutingMatcher::IpSubnet(IpNetwork {
                    address: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 0)),
                    prefix_len: 8,
                }),
                target_federation: school_id.clone(),
                priority: 100,
            })
            .await;

        // Peer from school network
        let school_peer = DiscoveredPeer {
            node_id: None,
            node_name: None,
            session_id: "school_peer".to_string(),
            endpoints: None,
            capabilities: vec!["academic".to_string()],
            protocols: vec!["https".to_string()],
            port: 8080,
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 1, 100)), 2300),
            last_seen: SystemTime::now(),
            version: "2.1".to_string(),
            tags: None,
            timestamp: None,
            identity_attestations: Some(Vec::new()),
        };

        let routes = router.route(&school_peer).await;
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0], school_id);

        // Peer from home network (no match, uses default)
        let home_peer = DiscoveredPeer {
            node_id: None,
            node_name: None,
            session_id: "home_peer".to_string(),
            endpoints: None,
            capabilities: vec!["media".to_string()],
            protocols: vec!["https".to_string()],
            port: 8080,
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)), 2300),
            last_seen: SystemTime::now(),
            version: "2.1".to_string(),
            tags: None,
            timestamp: None,
            identity_attestations: Some(Vec::new()),
        };

        let routes = router.route(&home_peer).await;
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0], family_id);
    }

    #[test]
    fn routing_matcher_all_matches_any_peer() {
        let m = RoutingMatcher::All;
        assert!(m.matches(&sample_peer([10, 0, 0, 1])));
    }

    #[test]
    fn routing_matcher_has_capability_requires_tag() {
        let m = RoutingMatcher::HasCapability("academic".into());
        assert!(m.matches(&sample_peer([1, 1, 1, 1])));
        let mut p = sample_peer([1, 1, 1, 1]);
        p.capabilities = vec![];
        assert!(!m.matches(&p));
    }
}
