// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Health probing and partition detection for the mesh handler.
//!
//! Extracted from `mod.rs` for SRP: contains `handle_health_check`,
//! `handle_probe_latency`, TCP probing (`probe_peer_rtt` / `probe_peer_full`),
//! and the background `spawn_peer_health_loop`.

use serde_json::{Value, json};
use songbird_onion_relay::mesh::{BeaconMesh, EndpointType};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info};

use super::{MeshHandler, PeerMetadata, ProbeResult, json as mesh_json};

impl MeshHandler {
    /// Handle `mesh.health_check` method - Check peer health
    pub async fn handle_health_check(&self, params: Value) -> Result<Value, String> {
        let (results, all_healthy) = {
            let mesh = self
                .mesh
                .read()
                .await
                .as_ref()
                .cloned()
                .ok_or("Mesh not initialized (call mesh.init first)")?;

            mesh.health_check().await;

            let target_ids: Vec<String> =
                if let Some(arr) = params.get("target_node_ids").and_then(|v| v.as_array()) {
                    arr.iter().filter_map(|v| v.as_str().map(String::from)).collect()
                } else {
                    mesh.get_reachable_nodes().await
                };

            let mut results = Vec::new();
            let mut all_healthy = true;

            for node_id in target_ids {
                if let Some(path) = mesh.get_best_path(&node_id).await {
                    let (path_type, _) = mesh_json::endpoint_to_strings(&path.endpoint_type);
                    let healthy = path.reachable;
                    if !healthy {
                        all_healthy = false;
                    }

                    results.push(json!({
                        "node_id": node_id,
                        "healthy": healthy,
                        "latency_ms": path.latency.map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX)),
                        "path_type": path_type
                    }));
                } else {
                    all_healthy = false;
                    results.push(json!({
                        "node_id": node_id,
                        "healthy": false,
                        "reason": "no_path_known"
                    }));
                }
            }

            Ok::<_, String>((results, all_healthy))
        }?;

        let meta = self.peer_metadata.read().await;
        let locally_reachable: std::collections::HashSet<&str> = results
            .iter()
            .filter_map(|r| {
                if r["healthy"].as_bool() == Some(true) {
                    r["node_id"].as_str()
                } else {
                    None
                }
            })
            .collect();

        let mut partitions: Vec<Value> = Vec::new();
        for (gate_id, pm) in meta.iter() {
            for remote_peer in &pm.reachable_peers {
                if !locally_reachable.contains(remote_peer.as_str()) {
                    partitions.push(json!({
                        "peer": remote_peer,
                        "reachable_from": gate_id,
                        "locally_reachable": false
                    }));
                }
            }
        }
        drop(meta);

        let mut response = json!({
            "results": results,
            "all_healthy": all_healthy
        });
        if !partitions.is_empty() {
            response["partitions"] = json!(partitions);
            response["partition_detected"] = json!(true);
        }
        Ok(response)
    }

    /// Handle `mesh.probe_latency` — actively probe peers to measure RTT.
    ///
    /// Connects to each reachable peer's TCP endpoint, sends a `health.ping` JSON-RPC
    /// request, measures the round-trip time, and updates the mesh with measured latency.
    pub async fn handle_probe_latency(&self, params: Value) -> Result<Value, String> {
        let timeout_ms = params.get("timeout_ms").and_then(Value::as_u64).unwrap_or(5000);
        let timeout = Duration::from_millis(timeout_ms);

        let mesh = self
            .mesh
            .read()
            .await
            .as_ref()
            .cloned()
            .ok_or("Mesh not initialized (call mesh.init first)")?;

        let reachable = mesh.get_reachable_nodes().await;
        let mut results = Vec::new();

        for node_id in &reachable {
            if let Some(path) = mesh.get_best_path(node_id).await {
                let addr = match &path.endpoint_type {
                    EndpointType::Direct {
                        addr,
                    }
                    | EndpointType::Local {
                        addr,
                    }
                    | EndpointType::Overlay {
                        addr,
                        ..
                    } => Some(*addr),
                    _ => None,
                };

                if let Some(peer_addr) = addr {
                    let start = Instant::now();
                    let probe_result = Self::probe_peer_rtt(peer_addr, timeout).await;
                    match probe_result {
                        Ok(rtt) => {
                            match &path.endpoint_type {
                                EndpointType::Overlay { overlay_name, .. } => {
                                    mesh.record_overlay_connection(
                                        node_id.clone(),
                                        peer_addr,
                                        overlay_name,
                                        rtt,
                                    )
                                    .await;
                                }
                                _ => {
                                    mesh.record_direct_connection(
                                        node_id.clone(),
                                        peer_addr,
                                        rtt,
                                    )
                                    .await;
                                }
                            }
                            let rtt_ms = u64::try_from(rtt.as_millis()).unwrap_or(u64::MAX);
                            results.push(json!({
                                "node_id": node_id,
                                "latency_ms": rtt_ms,
                                "address": peer_addr.to_string(),
                                "status": "ok"
                            }));
                            debug!(
                                peer = %node_id,
                                latency_ms = rtt_ms,
                                "Latency probe successful"
                            );
                        }
                        Err(e) => {
                            let elapsed_ms = start.elapsed().as_millis();
                            results.push(json!({
                                "node_id": node_id,
                                "address": peer_addr.to_string(),
                                "status": "error",
                                "error": e,
                                "elapsed_ms": elapsed_ms
                            }));
                        }
                    }
                } else {
                    results.push(json!({
                        "node_id": node_id,
                        "status": "skipped",
                        "reason": "no_tcp_endpoint"
                    }));
                }
            }
        }

        let probed_count = results.iter().filter(|r| r["status"] == "ok").count();
        info!(
            "📡 mesh.probe_latency: {}/{} peers probed successfully",
            probed_count,
            reachable.len()
        );

        Ok(json!({
            "results": results,
            "probed": probed_count,
            "total_peers": reachable.len(),
            "timeout_ms": timeout_ms
        }))
    }

    /// Probe a peer's TCP endpoint with a minimal JSON-RPC ping to measure RTT.
    pub(super) async fn probe_peer_rtt(
        addr: std::net::SocketAddr,
        timeout: Duration,
    ) -> Result<Duration, String> {
        Self::probe_peer_full(addr, timeout).await.map(|r| r.rtt)
    }

    /// Full probe returning RTT + version metadata.
    pub async fn probe_peer_full(
        addr: std::net::SocketAddr,
        timeout: Duration,
    ) -> Result<ProbeResult, String> {
        use songbird_types::constants::ribocipher;
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::net::TcpStream;

        let start = Instant::now();

        let stream = tokio::time::timeout(timeout, TcpStream::connect(addr))
            .await
            .map_err(|_| "connect timeout".to_string())?
            .map_err(|e| format!("connect failed: {e}"))?;

        let (reader, mut writer) = stream.into_split();

        // Send riboCipher mito signal prefix for federation outbound
        tokio::time::timeout(timeout, writer.write_all(&ribocipher::MITO_PREFIX))
            .await
            .map_err(|_| "write timeout (riboCipher signal)".to_string())?
            .map_err(|e| format!("write failed (riboCipher signal): {e}"))?;

        let request = b"{\"jsonrpc\":\"2.0\",\"method\":\"health.ping\",\"id\":1}\n";
        tokio::time::timeout(timeout, writer.write_all(request))
            .await
            .map_err(|_| "write timeout".to_string())?
            .map_err(|e| format!("write failed: {e}"))?;

        let mut buf_reader = BufReader::new(reader);
        let mut response = String::new();
        tokio::time::timeout(timeout, buf_reader.read_line(&mut response))
            .await
            .map_err(|_| "read timeout".to_string())?
            .map_err(|e| format!("read failed: {e}"))?;

        let rtt = start.elapsed();

        let version = serde_json::from_str::<serde_json::Value>(&response)
            .ok()
            .and_then(|v| v["result"]["version"].as_str().map(String::from));

        Ok(ProbeResult {
            rtt,
            version,
        })
    }

    /// Background peer health loop: periodically re-probes bootstrap and overlay peers.
    ///
    /// When a peer is unreachable, applies exponential backoff (30s → 60s → 120s → cap 300s).
    /// When a previously-failed peer responds, records fresh latency and restores reachability.
    /// Also extracts peer version for version-skew detection.
    ///
    /// Overlay peers (`WireGuard`) are probed alongside bootstrap peers and recorded with their
    /// correct endpoint type so `get_best_path` reflects actual overlay latency.
    ///
    /// Filters out self-connections: peers whose `node_id` matches our own or whose
    /// address matches our local bind address are skipped to prevent self-connect loops.
    #[expect(clippy::too_many_lines, reason = "cohesive setup + probe loop with per-kind recording")]
    pub(super) fn spawn_peer_health_loop(
        mesh: Arc<BeaconMesh>,
        bootstrap_peers: Vec<(String, std::net::SocketAddr)>,
        overlay_peers: Vec<(String, std::net::SocketAddr, String)>,
        peer_metadata: Arc<RwLock<HashMap<String, PeerMetadata>>>,
    ) {
        use std::collections::HashMap as StdHashMap;

        let our_node_id = mesh.node_id().to_string();
        let local_addrs = Self::detect_local_addresses();

        #[derive(Clone)]
        enum PeerKind {
            Bootstrap,
            Overlay { name: String },
        }

        let mut all_peers: Vec<(String, std::net::SocketAddr, PeerKind)> = bootstrap_peers
            .into_iter()
            .map(|(id, addr)| (id, addr, PeerKind::Bootstrap))
            .collect();

        for (id, addr, name) in overlay_peers {
            if !all_peers.iter().any(|(existing_id, _, _)| existing_id == &id) {
                all_peers.push((id, addr, PeerKind::Overlay { name }));
            }
        }

        // Filter out self-connections
        let peers: Vec<_> = all_peers
            .into_iter()
            .filter(|(peer_id, addr, _)| {
                if peer_id == &our_node_id {
                    tracing::debug!(peer = %peer_id, "Skipping self in health loop (node_id match)");
                    return false;
                }
                if local_addrs.contains(&addr.ip()) {
                    tracing::debug!(
                        peer = %peer_id,
                        addr = %addr,
                        "Skipping self in health loop (local address match)"
                    );
                    return false;
                }
                true
            })
            .collect();

        if peers.is_empty() {
            tracing::debug!("No remote peers after self-filter — health loop not started");
            return;
        }

        tokio::spawn(async move {
            let base_interval = Duration::from_secs(30);
            let max_interval = Duration::from_secs(300);
            let probe_timeout = Duration::from_secs(5);
            let mut backoff: StdHashMap<String, u32> = StdHashMap::new();
            let mut next_probe: StdHashMap<String, tokio::time::Instant> = peers
                .iter()
                .map(|(id, _, _)| (id.clone(), tokio::time::Instant::now() + base_interval))
                .collect();

            loop {
                tokio::time::sleep(Duration::from_secs(10)).await;

                let now = tokio::time::Instant::now();
                for (peer_id, addr, kind) in &peers {
                    let due = next_probe.get(peer_id).copied().unwrap_or(now);
                    if now < due {
                        continue;
                    }

                    if let Ok(result) = Self::probe_peer_full(*addr, probe_timeout).await {
                        match kind {
                            PeerKind::Bootstrap => {
                                mesh.record_direct_connection(
                                    peer_id.clone(),
                                    *addr,
                                    result.rtt,
                                )
                                .await;
                            }
                            PeerKind::Overlay { name } => {
                                mesh.record_overlay_connection(
                                    peer_id.clone(),
                                    *addr,
                                    name,
                                    result.rtt,
                                )
                                .await;
                            }
                        }
                        backoff.remove(peer_id);
                        next_probe.insert(peer_id.clone(), now + base_interval);

                        if let Some(ref ver) = result.version {
                            let mut meta = peer_metadata.write().await;
                            let entry =
                                meta.entry(peer_id.clone()).or_insert_with(|| PeerMetadata {
                                    version: None,
                                    reachable_peers: Vec::new(),
                                    last_updated: Instant::now(),
                                });
                            entry.version = Some(ver.clone());
                            entry.last_updated = Instant::now();
                        }

                        tracing::debug!(
                            peer = %peer_id,
                            latency_ms = %result.rtt.as_millis(),
                            version = ?result.version,
                            kind = ?match kind {
                                PeerKind::Bootstrap => "direct",
                                PeerKind::Overlay { .. } => "overlay",
                            },
                            "mesh health: peer alive"
                        );
                    } else {
                        let failures = backoff.entry(peer_id.clone()).or_insert(0);
                        *failures = failures.saturating_add(1);
                        let wait = base_interval
                            .saturating_mul(2u32.saturating_pow(*failures))
                            .min(max_interval);
                        next_probe.insert(peer_id.clone(), now + wait);
                        tracing::debug!(
                            peer = %peer_id,
                            failures = *failures,
                            next_retry_s = %wait.as_secs(),
                            "mesh health: peer unreachable, backing off"
                        );
                    }
                }
            }
        });
    }

    /// Detect local IP addresses to filter out self-connections in health probing.
    fn detect_local_addresses() -> Vec<std::net::IpAddr> {
        use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

        let mut addrs = vec![
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            IpAddr::V6(Ipv6Addr::LOCALHOST),
            IpAddr::V4(Ipv4Addr::UNSPECIFIED),
        ];

        if let Ok(hostname_ip) = songbird_process_env::var("SONGBIRD_NODE_ADDRESS")
            && let Ok(ip) = hostname_ip.parse::<IpAddr>()
        {
            addrs.push(ip);
        }

        if let Ok(interfaces) = std::net::UdpSocket::bind("0.0.0.0:0")
            && let Ok(local) = interfaces.local_addr()
        {
            addrs.push(local.ip());
        }

        // Netdev detection: connect to external endpoint to discover our outbound IP
        if let Ok(sock) = std::net::UdpSocket::bind("0.0.0.0:0")
            && sock.connect("8.8.8.8:80").is_ok()
            && let Ok(local) = sock.local_addr()
        {
            addrs.push(local.ip());
        }

        addrs.sort_unstable();
        addrs.dedup();
        addrs
    }
}
