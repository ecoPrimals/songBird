//! Multi-Federation Support
//!
//! Enables a single node to participate in multiple federations simultaneously,
//! each with its own trust policy, resource quota, and data isolation rules.
//!
//! ## Real-World Use Cases
//!
//! - **Family Federation**: High trust, full capabilities, home network
//! - **School Federation**: Medium trust, academic capabilities, university network
//! - **Work Federation**: Capability-verified, work-specific, corporate network
//!
//! ## Key Features
//!
//! - **Context-Aware**: Different capabilities per federation
//! - **Resource Fair**: Quota-based resource allocation
//! - **Secure**: Per-federation data isolation
//! - **Smart Routing**: Automatic discovery → federation mapping

use anyhow::Result;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

use crate::state::NodeRegistration;
use songbird_discovery::anonymous_discovery::DiscoveredPeer;

/// Federation identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FederationId(pub Uuid);

impl FederationId {
    /// Create a new random federation ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create from existing UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for FederationId {
    fn default() -> Self {
        Self::new()
    }
}

/// Multi-federation state manager
pub struct MultiFederationState {
    /// All federations this node participates in
    federations: Arc<RwLock<HashMap<FederationId, FederationContext>>>,

    /// Node's core identity
    node_id: Uuid,
}

impl MultiFederationState {
    /// Create a new multi-federation state
    #[must_use]
    pub fn new(node_id: Uuid) -> Self {
        Self {
            federations: Arc::new(RwLock::new(HashMap::new())),
            node_id,
        }
    }

    /// Add a federation
    pub async fn add_federation(&self, context: FederationContext) {
        let federation_id = context.federation_id.clone();
        self.federations.write().await.insert(federation_id.clone(), context);
        info!("✅ Added federation: {}", federation_id.0);
    }

    /// Get a federation by ID
    pub async fn get_federation(&self, id: &FederationId) -> Option<FederationContext> {
        self.federations.read().await.get(id).cloned()
    }

    /// Get all federations
    pub async fn get_all_federations(&self) -> Vec<FederationContext> {
        self.federations.read().await.values().cloned().collect()
    }

    /// Remove a federation
    pub async fn remove_federation(&self, id: &FederationId) -> Option<FederationContext> {
        let removed = self.federations.write().await.remove(id);
        if removed.is_some() {
            info!("🗑️  Removed federation: {}", id.0);
        }
        removed
    }

    /// Get total node count across all federations
    pub async fn total_nodes(&self) -> usize {
        let federations = self.federations.read().await;
        let mut total = 0;
        for f in federations.values() {
            total += f.nodes.read().await.len();
        }
        total
    }
}

/// Federation context with policies and state
#[derive(Clone)]
pub struct FederationContext {
    /// Unique federation identifier
    pub federation_id: FederationId,

    /// Human-readable name
    pub federation_name: String,

    /// Nodes in this federation
    pub nodes: Arc<RwLock<HashMap<String, NodeRegistration>>>,

    /// Trust policy for this federation
    pub trust_policy: TrustPolicy,

    /// What capabilities we expose in this federation
    pub exposed_capabilities: Vec<String>,

    /// Resource limits for this federation
    pub resource_quota: ResourceQuota,

    /// Data isolation rules
    pub data_policy: DataPolicy,

    /// Auto-join rules
    pub auto_join_policy: AutoJoinPolicy,

    /// When this federation was created
    pub created_at: DateTime<Utc>,
}

impl FederationContext {
    /// Create a new federation context
    #[must_use]
    pub fn new(name: String) -> Self {
        Self {
            federation_id: FederationId::new(),
            federation_name: name,
            nodes: Arc::new(RwLock::new(HashMap::new())),
            trust_policy: TrustPolicy::default(),
            exposed_capabilities: Vec::new(),
            resource_quota: ResourceQuota::default(),
            data_policy: DataPolicy::default(),
            auto_join_policy: AutoJoinPolicy::default(),
            created_at: Utc::now(),
        }
    }

    /// Try to join a discovered peer to this federation
    pub async fn try_join(&self, peer: &DiscoveredPeer, endpoint: &str) -> Result<()> {
        // TODO: Implement actual federation join logic
        // For now, just add to nodes
        let node = NodeRegistration {
            node_id: peer.session_id.clone(),
            node_name: peer.session_id.clone(),
            node_address: endpoint.to_string(),
            cpu_cores: 0,
            memory_gb: 0,
            gpu_model: None,
            storage_gb: None,
            capabilities: peer.capabilities.clone(),
            status: crate::state::NodeStatus::Active,
            joined_at: Utc::now(),
            last_heartbeat: Utc::now(),
        };

        self.nodes.write().await.insert(peer.session_id.clone(), node);
        Ok(())
    }

    /// Get node count in this federation
    pub async fn node_count(&self) -> usize {
        self.nodes.read().await.len()
    }
}

/// Trust policy for a federation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustPolicy {
    /// Minimum trust level required to join
    pub min_trust_level: String, // "anonymous", "capability", "identity", "hardware"

    /// Allow anonymous joins
    pub allow_anonymous: bool,
}

impl Default for TrustPolicy {
    fn default() -> Self {
        Self {
            min_trust_level: "anonymous".to_string(),
            allow_anonymous: true,
        }
    }
}

/// Resource quota for a federation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQuota {
    /// Max CPU cores allocated to this federation
    pub max_cpu_cores: Option<u32>,

    /// Max memory (GB) allocated to this federation
    pub max_memory_gb: Option<u32>,

    /// Max storage (GB) for this federation's data
    pub max_storage_gb: Option<u32>,

    /// Max concurrent tasks from this federation
    pub max_concurrent_tasks: Option<u32>,

    /// Priority (0-100, higher = more resources under contention)
    pub priority: u8,
}

impl Default for ResourceQuota {
    fn default() -> Self {
        Self {
            max_cpu_cores: None,
            max_memory_gb: None,
            max_storage_gb: None,
            max_concurrent_tasks: None,
            priority: 50,
        }
    }
}

/// Data isolation policy for a federation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPolicy {
    /// What data can be accessed by this federation's nodes
    pub accessible_paths: Vec<PathBuf>,

    /// Data that must be kept isolated
    pub isolated_paths: Vec<PathBuf>,

    /// Encryption requirements
    pub encryption_required: bool,

    /// Audit logging enabled
    pub audit_logging: bool,
}

impl Default for DataPolicy {
    fn default() -> Self {
        Self {
            accessible_paths: Vec::new(),
            isolated_paths: Vec::new(),
            encryption_required: false,
            audit_logging: false,
        }
    }
}

/// Auto-join policy for a federation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoJoinPolicy {
    /// Enable auto-join for this federation
    pub enabled: bool,

    /// Required capabilities (peer must have ALL)
    pub required_capabilities: Vec<String>,

    /// Forbidden capabilities (peer must have NONE)
    pub forbidden_capabilities: Vec<String>,

    /// Max nodes in this federation (capacity limit)
    pub max_nodes: Option<usize>,

    /// Allowlist (only these IPs can auto-join)
    pub ip_allowlist: Option<Vec<IpNetwork>>,

    /// Denylist (these IPs cannot auto-join)
    pub ip_denylist: Vec<IpNetwork>,

    /// Require manual approval
    pub require_approval: bool,
}

impl Default for AutoJoinPolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            required_capabilities: Vec::new(),
            forbidden_capabilities: Vec::new(),
            max_nodes: None,
            ip_allowlist: None,
            ip_denylist: Vec::new(),
            require_approval: false,
        }
    }
}

impl AutoJoinPolicy {
    /// Check if a discovered peer should auto-join
    pub fn should_auto_join(&self, peer: &DiscoveredPeer, current_nodes: usize) -> bool {
        if !self.enabled {
            return false;
        }

        // Check capacity
        if let Some(max) = self.max_nodes {
            if current_nodes >= max {
                debug!("Federation at capacity: {}/{}", current_nodes, max);
                return false;
            }
        }

        // Check IP allowlist/denylist
        let peer_ip = peer.address.ip();

        if let Some(ref allowlist) = self.ip_allowlist {
            if !allowlist.iter().any(|net| net.contains(peer_ip)) {
                debug!("Peer {} not in allowlist", peer_ip);
                return false;
            }
        }

        if self.ip_denylist.iter().any(|net| net.contains(peer_ip)) {
            debug!("Peer {} in denylist", peer_ip);
            return false;
        }

        // Check capabilities
        if !self
            .required_capabilities
            .iter()
            .all(|cap| peer.capabilities.contains(cap))
        {
            debug!(
                "Peer missing required capabilities: {:?}",
                self.required_capabilities
            );
            return false;
        }

        if self
            .forbidden_capabilities
            .iter()
            .any(|cap| peer.capabilities.contains(cap))
        {
            debug!(
                "Peer has forbidden capabilities: {:?}",
                self.forbidden_capabilities
            );
            return false;
        }

        // If require approval, don't auto-join
        if self.require_approval {
            debug!("Manual approval required");
            return false;
        }

        true
    }
}

/// IP network for allowlist/denylist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpNetwork {
    /// Network address
    pub address: IpAddr,

    /// Prefix length (e.g., 24 for /24)
    pub prefix_len: u8,
}

impl IpNetwork {
    /// Check if an IP is in this network
    pub fn contains(&self, ip: IpAddr) -> bool {
        use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

        match (self.address, ip) {
            (IpAddr::V4(net_ip), IpAddr::V4(test_ip)) => {
                let net_u32 = u32::from(net_ip);
                let test_u32 = u32::from(test_ip);
                let mask = !0u32 << (32 - self.prefix_len);
                (net_u32 & mask) == (test_u32 & mask)
            }
            (IpAddr::V6(net_ip), IpAddr::V6(test_ip)) => {
                let net_u128 = u128::from(net_ip);
                let test_u128 = u128::from(test_ip);
                let mask = !0u128 << (128 - self.prefix_len);
                (net_u128 & mask) == (test_u128 & mask)
            }
            _ => false, // IPv4 vs IPv6 mismatch
        }
    }
}

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
        let mut matches = Vec::new();
        let rules = self.routing_rules.read().await;

        for rule in rules.iter() {
            if rule.matcher.matches(peer) {
                matches.push(rule.target_federation.clone());
                debug!(
                    "Peer {} matched rule (priority {}) → federation {}",
                    peer.session_id, rule.priority, rule.target_federation.0
                );
            }
        }

        // If no matches, use default
        if matches.is_empty() {
            if let Some(ref default) = self.default_federation {
                debug!(
                    "Peer {} using default federation {}",
                    peer.session_id, default.0
                );
                matches.push(default.clone());
            }
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
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::time::SystemTime;

    #[test]
    fn test_ip_network_contains() {
        let network = IpNetwork {
            address: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 0)),
            prefix_len: 24,
        };

        assert!(network.contains(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))));
        assert!(network.contains(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 255))));
        assert!(!network.contains(IpAddr::V4(Ipv4Addr::new(192, 168, 2, 1))));
        assert!(!network.contains(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))));
    }

    #[test]
    fn test_auto_join_policy() {
        let policy = AutoJoinPolicy {
            enabled: true,
            required_capabilities: vec!["academic".to_string()],
            forbidden_capabilities: vec!["personal".to_string()],
            max_nodes: Some(10),
            ip_allowlist: None,
            ip_denylist: Vec::new(),
            require_approval: false,
        };

        let peer = DiscoveredPeer {
            session_id: "test".to_string(),
            capabilities: vec!["academic".to_string(), "compute".to_string()],
            protocols: vec!["https".to_string()],
            port: 8080,
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)), 2300),
            last_seen: SystemTime::now(),
            version: "2.1".to_string(),
        };

        // Should auto-join (has required, no forbidden, under capacity)
        assert!(policy.should_auto_join(&peer, 5));

        // Should not auto-join (at capacity)
        assert!(!policy.should_auto_join(&peer, 10));
    }

    #[tokio::test]
    async fn test_discovery_router() {
        let family_id = FederationId::new();
        let school_id = FederationId::new();

        let router = DiscoveryRouter::new(Some(family_id.clone()));

        // Add school rule (IP-based)
        router.add_rule(RoutingRule {
            matcher: RoutingMatcher::IpSubnet(IpNetwork {
                address: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 0)),
                prefix_len: 8,
            }),
            target_federation: school_id.clone(),
            priority: 100,
        }).await;

        // Peer from school network
        let school_peer = DiscoveredPeer {
            session_id: "school_peer".to_string(),
            capabilities: vec!["academic".to_string()],
            protocols: vec!["https".to_string()],
            port: 8080,
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 1, 100)), 2300),
            last_seen: SystemTime::now(),
            version: "2.1".to_string(),
        };

        let routes = router.route(&school_peer).await;
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0], school_id);

        // Peer from home network (no match, uses default)
        let home_peer = DiscoveredPeer {
            session_id: "home_peer".to_string(),
            capabilities: vec!["media".to_string()],
            protocols: vec!["https".to_string()],
            port: 8080,
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)), 2300),
            last_seen: SystemTime::now(),
            version: "2.1".to_string(),
        };

        let routes = router.route(&home_peer).await;
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0], family_id);
    }
}

