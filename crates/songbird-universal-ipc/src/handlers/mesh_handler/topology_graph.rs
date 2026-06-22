// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Topology graph construction from local mesh state + peer gossip.
//!
//! Merges our directly-observed edges (from `BeaconMesh`) with
//! `reachable_peers` data received from remote gates via `capabilities_announce`.
//! Produces a global view of the mesh topology suitable for:
//!
//! - Visualization (`PetalTongue` dashboards)
//! - Multi-hop route planning (future)
//! - Partition detection across the full mesh

use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};

use super::PeerMetadata;

/// A merged topology graph representing the full mesh as known to this gate.
pub struct TopologyGraph {
    /// All known nodes in the mesh (self + directly observed + gossiped).
    pub nodes: Vec<TopologyNode>,
    /// All edges: both directly observed and inferred from peer gossip.
    pub edges: Vec<TopologyEdge>,
}

/// A node in the topology graph.
pub struct TopologyNode {
    pub id: String,
    pub role: NodeRole,
    pub reachable_from_self: bool,
}

/// The role of a node as perceived by this gate.
#[derive(Clone, Copy)]
pub enum NodeRole {
    /// This gate.
    Self_,
    /// Directly reachable peer.
    DirectPeer,
    /// Known only through gossip from other peers.
    InferredPeer,
}

/// An edge in the topology graph.
pub struct TopologyEdge {
    pub from: String,
    pub to: String,
    pub source: EdgeSource,
    pub latency_ms: Option<u64>,
}

/// How this edge was learned.
#[derive(Clone, Copy)]
pub enum EdgeSource {
    /// Directly observed by this gate (we have an endpoint registered).
    DirectObservation,
    /// Inferred from a peer's `reachable_peers` gossip.
    PeerGossip,
}

/// Build a merged topology graph from local mesh state and peer metadata.
///
/// Combines:
/// - Direct edges: self → each peer we have a path to
/// - Gossip edges: for each peer P, if P reports peer Q in `reachable_peers`,
///   adds edge P → Q (source: gossip)
pub fn build_topology(
    self_node_id: &str,
    directly_reachable: &[String],
    peer_metadata: &HashMap<String, PeerMetadata>,
    local_latencies: &HashMap<String, u64>,
) -> TopologyGraph {
    let mut node_set: HashSet<String> = HashSet::new();
    let mut edges: Vec<TopologyEdge> = Vec::new();

    node_set.insert(String::from(self_node_id));

    // Direct edges from self
    for peer_id in directly_reachable {
        node_set.insert(peer_id.clone());
        edges.push(TopologyEdge {
            from: String::from(self_node_id),
            to: peer_id.clone(),
            source: EdgeSource::DirectObservation,
            latency_ms: local_latencies.get(peer_id).copied(),
        });
    }

    // Gossip edges from peer metadata
    for (reporter_id, meta) in peer_metadata {
        node_set.insert(reporter_id.clone());
        for remote_peer in &meta.reachable_peers {
            node_set.insert(remote_peer.clone());
            if remote_peer != self_node_id {
                edges.push(TopologyEdge {
                    from: reporter_id.clone(),
                    to: remote_peer.clone(),
                    source: EdgeSource::PeerGossip,
                    latency_ms: None,
                });
            }
        }
    }

    let directly_reachable_set: HashSet<&str> =
        directly_reachable.iter().map(String::as_str).collect();

    let nodes: Vec<TopologyNode> = node_set
        .into_iter()
        .map(|id| {
            let role = if id == self_node_id {
                NodeRole::Self_
            } else if directly_reachable_set.contains(id.as_str()) {
                NodeRole::DirectPeer
            } else {
                NodeRole::InferredPeer
            };
            TopologyNode {
                reachable_from_self: id == self_node_id
                    || directly_reachable_set.contains(id.as_str()),
                id,
                role,
            }
        })
        .collect();

    TopologyGraph { nodes, edges }
}

impl TopologyGraph {
    /// Serialize to JSON for the enriched `mesh.topology` response.
    #[must_use]
    pub fn to_json(&self, self_node_id: &str, uptime_secs: u64) -> Value {
        let nodes: Vec<Value> = self
            .nodes
            .iter()
            .map(|n| {
                json!({
                    "id": n.id,
                    "role": match n.role {
                        NodeRole::Self_ => "self",
                        NodeRole::DirectPeer => "peer",
                        NodeRole::InferredPeer => "inferred",
                    },
                    "reachable_from_self": n.reachable_from_self
                })
            })
            .collect();

        let edges: Vec<Value> = self
            .edges
            .iter()
            .map(|e| {
                let mut edge = json!({
                    "from": e.from,
                    "to": e.to,
                    "source": match e.source {
                        EdgeSource::DirectObservation => "direct",
                        EdgeSource::PeerGossip => "gossip",
                    }
                });
                if let Some(ms) = e.latency_ms {
                    edge["latency_ms"] = json!(ms);
                }
                edge
            })
            .collect();

        let direct_count = self
            .edges
            .iter()
            .filter(|e| matches!(e.source, EdgeSource::DirectObservation))
            .count();
        let gossip_count = self
            .edges
            .iter()
            .filter(|e| matches!(e.source, EdgeSource::PeerGossip))
            .count();

        json!({
            "nodes": nodes,
            "edges": edges,
            "node_count": self.nodes.len(),
            "edge_count": self.edges.len(),
            "direct_edges": direct_count,
            "gossip_edges": gossip_count,
            "self_node_id": self_node_id,
            "uptime_seconds": uptime_secs
        })
    }

    /// Detect partitions: nodes known through gossip but not directly reachable.
    #[must_use]
    pub fn partitioned_nodes(&self) -> Vec<&str> {
        self.nodes
            .iter()
            .filter(|n| !n.reachable_from_self && !matches!(n.role, NodeRole::Self_))
            .map(|n| n.id.as_str())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_topology_full_mesh_five_gates() {
        let self_id = "east-gate";
        let directly_reachable = vec![
            String::from("flock-gate"),
            String::from("spore-gate"),
            String::from("iron-gate"),
            String::from("golgi"),
        ];

        let mut peer_metadata = HashMap::new();
        peer_metadata.insert(
            String::from("flock-gate"),
            PeerMetadata {
                version: Some(String::from("0.2.1")),
                reachable_peers: vec![
                    String::from("spore-gate"),
                    String::from("iron-gate"),
                    String::from("golgi"),
                ],
                last_updated: std::time::Instant::now(),
            },
        );
        peer_metadata.insert(
            String::from("spore-gate"),
            PeerMetadata {
                version: Some(String::from("0.2.1")),
                reachable_peers: vec![
                    String::from("flock-gate"),
                    String::from("iron-gate"),
                    String::from("golgi"),
                ],
                last_updated: std::time::Instant::now(),
            },
        );

        let mut latencies = HashMap::new();
        latencies.insert(String::from("flock-gate"), 12);
        latencies.insert(String::from("spore-gate"), 1);
        latencies.insert(String::from("iron-gate"), 2);
        latencies.insert(String::from("golgi"), 25);

        let graph = build_topology(self_id, &directly_reachable, &peer_metadata, &latencies);

        assert_eq!(graph.nodes.len(), 5, "all 5 gates should be nodes");
        assert!(graph.partitioned_nodes().is_empty(), "full mesh has no partitions");

        let direct_edges: Vec<_> = graph
            .edges
            .iter()
            .filter(|e| matches!(e.source, EdgeSource::DirectObservation))
            .collect();
        assert_eq!(direct_edges.len(), 4, "4 direct edges from self");

        let gossip_edges: Vec<_> = graph
            .edges
            .iter()
            .filter(|e| matches!(e.source, EdgeSource::PeerGossip))
            .collect();
        assert!(gossip_edges.len() >= 4, "gossip edges from flock + spore");
    }

    #[test]
    fn build_topology_partial_mesh_detects_inferred_node() {
        let self_id = "gate-a";
        let directly_reachable = vec![String::from("gate-b")];

        let mut peer_metadata = HashMap::new();
        peer_metadata.insert(
            String::from("gate-b"),
            PeerMetadata {
                version: None,
                reachable_peers: vec![String::from("gate-c"), String::from("gate-d")],
                last_updated: std::time::Instant::now(),
            },
        );

        let graph = build_topology(self_id, &directly_reachable, &peer_metadata, &HashMap::new());

        assert_eq!(graph.nodes.len(), 4, "a, b, c, d");

        let partitioned = graph.partitioned_nodes();
        assert_eq!(partitioned.len(), 2, "gate-c and gate-d not directly reachable");
        assert!(partitioned.contains(&"gate-c"));
        assert!(partitioned.contains(&"gate-d"));
    }

    #[test]
    fn topology_json_serialization() {
        let self_id = "hub";
        let directly_reachable = vec![String::from("peer-1")];
        let mut latencies = HashMap::new();
        latencies.insert(String::from("peer-1"), 5);

        let graph =
            build_topology(self_id, &directly_reachable, &HashMap::new(), &latencies);
        let json = graph.to_json(self_id, 120);

        assert_eq!(json["node_count"], 2);
        assert_eq!(json["edge_count"], 1);
        assert_eq!(json["direct_edges"], 1);
        assert_eq!(json["gossip_edges"], 0);
        assert_eq!(json["self_node_id"], "hub");
        assert_eq!(json["uptime_seconds"], 120);
    }
}
