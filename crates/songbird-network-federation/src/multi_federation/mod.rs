// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

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

mod discovery_routing;

pub use discovery_routing::{DiscoveryRouter, RoutingMatcher, RoutingRule};

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

use crate::state::NodeRegistration;
use songbird_discovery::anonymous::DiscoveredPeer;

/// Federation identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FederationId(pub Uuid);

impl FederationId {
    /// Create a new random federation ID
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create from existing UUID
    #[must_use]
    pub const fn from_uuid(uuid: Uuid) -> Self {
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

    /// Node's core identity (used for federation membership tracking)
    #[expect(
        dead_code,
        reason = "reserved for future use: federation-specific identity management"
    )]
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
        let node_locks: Vec<_> =
            self.federations.read().await.values().map(|f| Arc::clone(&f.nodes)).collect();
        let mut total = 0;
        for nodes_lock in node_locks {
            total += nodes_lock.read().await.len();
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
        // Registers the peer locally: no remote join protocol runs in this minimal integration.
        let node = NodeRegistration {
            node_id: peer.session_id.clone(),
            node_name: peer.session_id.clone(),
            node_address: endpoint.to_string(),
            endpoints: None, // Will be populated by discovery integration
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
            min_trust_level: String::from("anonymous"),
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
        if let Some(max) = self.max_nodes
            && current_nodes >= max
        {
            debug!("Federation at capacity: {}/{}", current_nodes, max);
            return false;
        }

        // Check IP allowlist/denylist
        let peer_ip = peer.address.ip();

        if let Some(ref allowlist) = self.ip_allowlist
            && !allowlist.iter().any(|net| net.contains(peer_ip))
        {
            debug!("Peer {} not in allowlist", peer_ip);
            return false;
        }

        if self.ip_denylist.iter().any(|net| net.contains(peer_ip)) {
            debug!("Peer {} in denylist", peer_ip);
            return false;
        }

        // Check capabilities
        if !self.required_capabilities.iter().all(|cap| peer.capabilities.contains(cap)) {
            debug!("Peer missing required capabilities: {:?}", self.required_capabilities);
            return false;
        }

        if self.forbidden_capabilities.iter().any(|cap| peer.capabilities.contains(cap)) {
            debug!("Peer has forbidden capabilities: {:?}", self.forbidden_capabilities);
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
    #[must_use]
    pub fn contains(&self, ip: IpAddr) -> bool {
        use std::net::IpAddr;

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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

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
            required_capabilities: vec![String::from("academic")],
            forbidden_capabilities: vec![String::from("personal")],
            max_nodes: Some(10),
            ip_allowlist: None,
            ip_denylist: Vec::new(),
            require_approval: false,
        };

        let peer = DiscoveredPeer {
            node_id: None,
            node_name: None,
            session_id: String::from("test"),
            endpoints: None,
            capabilities: vec![String::from("academic"), String::from("compute")],
            protocols: vec![String::from("https")],
            port: 8080,
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)), 2300),
            last_seen: SystemTime::now(),
            version: String::from("2.1"),
            tags: None,
            timestamp: None,
            identity_attestations: Some(Vec::new()),
        };

        // Should auto-join (has required, no forbidden, under capacity)
        assert!(policy.should_auto_join(&peer, 5));

        // Should not auto-join (at capacity)
        assert!(!policy.should_auto_join(&peer, 10));
    }

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

    #[test]
    fn ip_network_ipv6_contains() {
        let net = IpNetwork {
            address: IpAddr::V6("2001:db8::".parse().unwrap()),
            prefix_len: 32,
        };
        assert!(net.contains("2001:db8::1".parse().unwrap()));
        assert!(!net.contains(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))));
    }

    #[tokio::test]
    async fn multi_federation_add_remove_and_list() {
        let st = MultiFederationState::new(uuid::Uuid::new_v4());
        let ctx = FederationContext::new("fed-a".into());
        let id = ctx.federation_id.clone();
        st.add_federation(ctx).await;
        assert!(st.get_federation(&id).await.is_some());
        let all = st.get_all_federations().await;
        assert_eq!(all.len(), 1);
        assert!(st.remove_federation(&id).await.is_some());
        assert!(st.get_federation(&id).await.is_none());
    }

    #[tokio::test]
    async fn total_nodes_sums_across_federations() {
        let st = MultiFederationState::new(uuid::Uuid::new_v4());
        let a = FederationContext::new("a".into());
        let b = FederationContext::new("b".into());
        let id_a = a.federation_id.clone();
        let id_b = b.federation_id.clone();
        a.nodes.write().await.insert(
            "n1".into(),
            NodeRegistration {
                node_id: "n1".into(),
                node_name: "n1".into(),
                node_address: "x".into(),
                endpoints: None,
                cpu_cores: 0,
                memory_gb: 0,
                gpu_model: None,
                storage_gb: None,
                capabilities: vec![],
                status: crate::state::NodeStatus::Active,
                joined_at: chrono::Utc::now(),
                last_heartbeat: chrono::Utc::now(),
            },
        );
        b.nodes.write().await.insert(
            "n2".into(),
            NodeRegistration {
                node_id: "n2".into(),
                node_name: "n2".into(),
                node_address: "y".into(),
                endpoints: None,
                cpu_cores: 0,
                memory_gb: 0,
                gpu_model: None,
                storage_gb: None,
                capabilities: vec![],
                status: crate::state::NodeStatus::Active,
                joined_at: chrono::Utc::now(),
                last_heartbeat: chrono::Utc::now(),
            },
        );
        st.add_federation(a).await;
        st.add_federation(b).await;
        assert_eq!(st.total_nodes().await, 2);
        st.remove_federation(&id_a).await;
        st.remove_federation(&id_b).await;
    }

    #[test]
    fn auto_join_rejects_forbidden_capability() {
        let policy = AutoJoinPolicy {
            enabled: true,
            required_capabilities: vec![],
            forbidden_capabilities: vec!["bad".into()],
            max_nodes: None,
            ip_allowlist: None,
            ip_denylist: vec![],
            require_approval: false,
        };
        let mut p = sample_peer([8, 8, 8, 8]);
        p.capabilities = vec!["bad".into()];
        assert!(!policy.should_auto_join(&p, 0));
    }

    #[test]
    fn trust_policy_default_allows_anonymous() {
        let t = TrustPolicy::default();
        assert!(t.allow_anonymous);
        assert_eq!(t.min_trust_level, "anonymous");
    }

    #[test]
    fn federation_id_new_and_from_uuid() {
        let a = FederationId::new();
        let b = FederationId::new();
        assert_ne!(a, b);
        let uuid = uuid::Uuid::new_v4();
        assert_eq!(FederationId::from_uuid(uuid), FederationId(uuid));
    }

    #[test]
    fn federation_id_default_is_unique() {
        let a = FederationId::default();
        let b = FederationId::default();
        assert_ne!(a, b);
    }

    #[test]
    fn resource_quota_and_data_policy_defaults() {
        let quota = ResourceQuota::default();
        assert!(quota.max_cpu_cores.is_none());
        assert_eq!(quota.priority, 50);
        let data = DataPolicy::default();
        assert!(data.accessible_paths.is_empty());
        assert!(!data.encryption_required);
    }

    #[test]
    fn auto_join_disabled_returns_false() {
        let policy = AutoJoinPolicy {
            enabled: false,
            ..AutoJoinPolicy::default()
        };
        assert!(!policy.should_auto_join(&sample_peer([10, 0, 0, 1]), 0));
    }

    #[test]
    fn auto_join_ip_allowlist_rejects_outside_network() {
        let policy = AutoJoinPolicy {
            enabled: true,
            ip_allowlist: Some(vec![IpNetwork {
                address: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 0)),
                prefix_len: 24,
            }]),
            ..AutoJoinPolicy::default()
        };
        assert!(!policy.should_auto_join(&sample_peer([10, 0, 0, 1]), 0));
        assert!(policy.should_auto_join(&sample_peer([192, 168, 1, 50]), 0));
    }

    #[test]
    fn auto_join_ip_denylist_blocks_peer() {
        let policy = AutoJoinPolicy {
            enabled: true,
            ip_denylist: vec![IpNetwork {
                address: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 0)),
                prefix_len: 8,
            }],
            ..AutoJoinPolicy::default()
        };
        assert!(!policy.should_auto_join(&sample_peer([10, 1, 2, 3]), 0));
    }

    #[test]
    fn auto_join_missing_required_capability() {
        let policy = AutoJoinPolicy {
            enabled: true,
            required_capabilities: vec!["gpu".into()],
            ..AutoJoinPolicy::default()
        };
        assert!(!policy.should_auto_join(&sample_peer([192, 168, 1, 1]), 0));
    }

    #[test]
    fn auto_join_require_approval_blocks() {
        let policy = AutoJoinPolicy {
            enabled: true,
            require_approval: true,
            ..AutoJoinPolicy::default()
        };
        assert!(!policy.should_auto_join(&sample_peer([192, 168, 1, 1]), 0));
    }

    #[tokio::test]
    async fn federation_context_try_join_registers_peer() {
        let ctx = FederationContext::new("school".into());
        let peer = DiscoveredPeer {
            node_id: None,
            node_name: None,
            session_id: "peer-1".into(),
            endpoints: None,
            capabilities: vec!["compute".into()],
            protocols: vec!["https".into()],
            port: 8080,
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 9000),
            last_seen: SystemTime::now(),
            version: "2.1".into(),
            tags: None,
            timestamp: None,
            identity_attestations: Some(vec![]),
        };
        ctx.try_join(&peer, "127.0.0.1:9000").await.unwrap();
        assert_eq!(ctx.node_count().await, 1);
        let nodes = ctx.nodes.read().await;
        let reg = nodes.get("peer-1").unwrap();
        assert_eq!(reg.node_address, "127.0.0.1:9000");
        assert_eq!(reg.status, crate::state::NodeStatus::Active);
    }

    #[tokio::test]
    async fn remove_federation_missing_returns_none() {
        let st = MultiFederationState::new(uuid::Uuid::new_v4());
        let id = FederationId::new();
        assert!(st.remove_federation(&id).await.is_none());
    }

    #[tokio::test]
    async fn federation_context_new_has_unique_id_and_name() {
        let ctx = FederationContext::new("work".into());
        assert_eq!(ctx.federation_name, "work");
        assert!(ctx.node_count().await == 0);
        assert!(ctx.auto_join_policy.enabled);
    }
}
