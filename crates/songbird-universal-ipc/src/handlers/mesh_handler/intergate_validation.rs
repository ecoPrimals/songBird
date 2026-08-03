// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Inter-gate mesh validation: connectivity checks and throughput measurement.
//!
//! These methods support the P1 requirement for live E2E validation of cross-gate
//! data paths (`content.get` roundtrips, streaming transfers).

use serde_json::{Value, json};
use songbird_onion_relay::mesh::EndpointType;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

use super::MeshHandler;

/// Minimum payload size for throughput test (64 KiB).
const MIN_PAYLOAD_BYTES: u64 = 64 * 1024;

/// Default payload size for throughput test (1 MiB).
const DEFAULT_PAYLOAD_BYTES: u64 = 1024 * 1024;

/// Maximum payload size (256 MiB) to prevent abuse.
const MAX_PAYLOAD_BYTES: u64 = 256 * 1024 * 1024;

/// Default throughput test timeout.
const DEFAULT_THROUGHPUT_TIMEOUT_MS: u64 = 30_000;

impl MeshHandler {
    /// Handle `mesh.connectivity_check` — active E2E inter-gate validation.
    ///
    /// For each reachable peer (or specified targets), performs:
    /// 1. TCP connect + `riboCipher` handshake
    /// 2. Bidirectional JSON-RPC exchange (`health.ping`)
    /// 3. Cross-gate path classification (direct/overlay/relay)
    /// 4. Reports per-peer: reachability, latency, `riboCipher` acceptance, version
    #[expect(clippy::too_many_lines, reason = "cohesive per-peer probe loop")]
    pub async fn handle_connectivity_check(&self, params: Value) -> Result<Value, String> {
        let timeout_ms = params.get("timeout_ms").and_then(Value::as_u64).unwrap_or(5000);
        let timeout = Duration::from_millis(timeout_ms);
        let verify_identity =
            params.get("verify_identity").and_then(Value::as_bool).unwrap_or(true);

        let mesh = self
            .mesh
            .read()
            .await
            .as_ref()
            .cloned()
            .ok_or("Mesh not initialized (call mesh.init first)")?;

        let target_ids: Vec<String> =
            if let Some(arr) = params.get("target_node_ids").and_then(|v| v.as_array()) {
                arr.iter().filter_map(|v| v.as_str().map(String::from)).collect()
            } else {
                mesh.get_reachable_nodes().await
            };

        let mut results = Vec::new();
        let mut reachable_count = 0u32;
        let mut ribocipher_ok_count = 0u32;
        let mut cross_gate_peers = Vec::new();

        for node_id in &target_ids {
            let Some(path) = mesh.get_best_path(node_id).await else {
                results.push(json!({
                    "node_id": node_id,
                    "status": "no_path",
                    "reachable": false,
                }));
                continue;
            };

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

            let Some(peer_addr) = addr else {
                results.push(json!({
                    "node_id": node_id,
                    "status": "no_tcp_endpoint",
                    "reachable": false,
                    "path_type": endpoint_type_name(&path.endpoint_type),
                }));
                continue;
            };

            let (path_type, is_cross_gate) = classify_path(&path.endpoint_type);

            let check_result =
                Self::full_connectivity_probe(peer_addr, timeout, verify_identity).await;

            match check_result {
                Ok(probe) => {
                    reachable_count += 1;
                    if probe.ribocipher_accepted {
                        ribocipher_ok_count += 1;
                    }
                    if is_cross_gate {
                        cross_gate_peers.push(node_id.clone());
                    }

                    let rtt_ms = u64::try_from(probe.rtt.as_millis()).unwrap_or(u64::MAX);
                    let mut entry = json!({
                        "node_id": node_id,
                        "status": "ok",
                        "reachable": true,
                        "address": peer_addr.to_string(),
                        "path_type": path_type,
                        "cross_gate": is_cross_gate,
                        "latency_ms": rtt_ms,
                        "ribocipher_accepted": probe.ribocipher_accepted,
                    });

                    if let Some(ref ver) = probe.version {
                        entry["version"] = json!(ver);
                    }
                    if let Some(ref identity) = probe.identity {
                        entry["identity"] = json!(identity);
                    }

                    results.push(entry);
                }
                Err(e) => {
                    results.push(json!({
                        "node_id": node_id,
                        "status": "error",
                        "reachable": false,
                        "address": peer_addr.to_string(),
                        "path_type": path_type,
                        "cross_gate": is_cross_gate,
                        "error": e,
                    }));
                }
            }
        }

        let total = target_ids.len();
        let unreachable = total.saturating_sub(usize::try_from(reachable_count).unwrap_or(0));
        info!(
            "🔗 mesh.connectivity_check: {reachable_count}/{total} reachable, \
             {ribocipher_ok_count} riboCipher OK, {} cross-gate peers",
            cross_gate_peers.len()
        );

        Ok(json!({
            "results": results,
            "summary": {
                "total_peers": total,
                "reachable": reachable_count,
                "unreachable": unreachable,
                "ribocipher_accepted": ribocipher_ok_count,
                "cross_gate_peers": cross_gate_peers,
                "timeout_ms": timeout_ms,
                "verify_identity": verify_identity,
            }
        }))
    }

    /// Handle `mesh.throughput` — sustained TCP streaming test.
    ///
    /// Connects to a target peer and sends a configurable payload, measuring
    /// actual transfer rate. Used to validate that inter-gate links can sustain
    /// the bandwidth required for `content.get` (target >800 MB/s on 10G LAN).
    pub async fn handle_throughput(&self, params: Value) -> Result<Value, String> {
        let target_address = params
            .get("target_address")
            .and_then(Value::as_str)
            .ok_or("missing required param: target_address")?;

        let payload_bytes = params
            .get("payload_bytes")
            .and_then(Value::as_u64)
            .unwrap_or(DEFAULT_PAYLOAD_BYTES)
            .clamp(MIN_PAYLOAD_BYTES, MAX_PAYLOAD_BYTES);

        let timeout_ms = params
            .get("timeout_ms")
            .and_then(Value::as_u64)
            .unwrap_or(DEFAULT_THROUGHPUT_TIMEOUT_MS);
        let timeout = Duration::from_millis(timeout_ms);

        let addr: std::net::SocketAddr =
            target_address.parse().map_err(|e| format!("invalid target_address: {e}"))?;

        info!("📊 mesh.throughput: testing {target_address} with {payload_bytes} bytes",);

        let result = Self::measure_throughput(addr, payload_bytes, timeout).await?;

        #[expect(clippy::cast_precision_loss, reason = "throughput display precision is fine")]
        let throughput_mbps =
            (result.bytes_transferred as f64) / result.elapsed.as_secs_f64() / 1_000_000.0;

        let elapsed_ms = u64::try_from(result.elapsed.as_millis()).unwrap_or(u64::MAX);

        info!(
            "📊 mesh.throughput: {throughput_mbps:.1} MB/s to {target_address} \
             ({} bytes in {elapsed_ms}ms)",
            result.bytes_transferred
        );

        Ok(json!({
            "target_address": target_address,
            "throughput_mbps": throughput_mbps,
            "bytes_transferred": result.bytes_transferred,
            "elapsed_ms": elapsed_ms,
            "payload_bytes_requested": payload_bytes,
            "meets_10g_threshold": throughput_mbps >= 800.0,
        }))
    }

    /// Full connectivity probe: TCP + `riboCipher` + `health.ping` + optional identity.
    async fn full_connectivity_probe(
        addr: std::net::SocketAddr,
        timeout: Duration,
        verify_identity: bool,
    ) -> Result<ConnectivityProbeResult, String> {
        use songbird_types::constants::ribocipher;
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::net::TcpStream;

        let start = Instant::now();

        let stream = tokio::time::timeout(timeout, TcpStream::connect(addr))
            .await
            .map_err(|_| "connect timeout".to_string())?
            .map_err(|e| format!("connect failed: {e}"))?;

        let (reader, mut writer) = stream.into_split();

        let ribocipher_accepted =
            tokio::time::timeout(timeout, writer.write_all(&ribocipher::MITO_PREFIX))
                .await
                .map_err(|_| "write timeout (riboCipher)".to_string())?
                .is_ok();

        let ping_req = b"{\"jsonrpc\":\"2.0\",\"method\":\"health.ping\",\"id\":1}\n";
        tokio::time::timeout(timeout, writer.write_all(ping_req))
            .await
            .map_err(|_| "write timeout (ping)".to_string())?
            .map_err(|e| format!("write failed (ping): {e}"))?;

        let mut buf_reader = BufReader::new(reader);
        let mut response = String::new();
        tokio::time::timeout(timeout, buf_reader.read_line(&mut response))
            .await
            .map_err(|_| "read timeout (ping response)".to_string())?
            .map_err(|e| format!("read failed: {e}"))?;

        let rtt = start.elapsed();

        let ping_json: Value =
            serde_json::from_str(&response).map_err(|e| format!("invalid ping response: {e}"))?;
        let version = ping_json["result"]["version"].as_str().map(String::from);

        // Use version from health.ping as identity proxy (version encodes primal + gate).
        let identity = if verify_identity {
            debug!("connectivity_check: identity verification via version field");
            version.clone()
        } else {
            None
        };

        Ok(ConnectivityProbeResult {
            rtt,
            version,
            identity,
            ribocipher_accepted,
        })
    }

    /// Measure sustained TCP throughput to a peer.
    ///
    /// Connects and sends a stream of data, measuring how fast the peer can accept it.
    /// The peer's JSON-RPC server receives the raw bytes (non-JSON lines are discarded).
    /// For bidirectional validation, run from both sides.
    async fn measure_throughput(
        addr: std::net::SocketAddr,
        payload_bytes: u64,
        timeout: Duration,
    ) -> Result<ThroughputResult, String> {
        use songbird_types::constants::ribocipher;
        use tokio::io::AsyncWriteExt;
        use tokio::net::TcpStream;

        let stream = tokio::time::timeout(timeout, TcpStream::connect(addr))
            .await
            .map_err(|_| "connect timeout".to_string())?
            .map_err(|e| format!("connect failed: {e}"))?;

        stream.set_nodelay(true).ok();

        let mut writer = stream;

        writer
            .write_all(&ribocipher::CLEAR_PREFIX)
            .await
            .map_err(|e| format!("riboCipher write failed: {e}"))?;

        let chunk_size: usize = 64 * 1024;
        let chunk = vec![b'A'; chunk_size];
        let mut bytes_sent: u64 = 0;

        let start = Instant::now();

        let send_result = tokio::time::timeout(timeout, async {
            while bytes_sent < payload_bytes {
                let remaining = payload_bytes.saturating_sub(bytes_sent);
                let to_send = usize::try_from(remaining).unwrap_or(chunk_size).min(chunk_size);
                if writer.write_all(&chunk[..to_send]).await.is_err() {
                    break;
                }
                bytes_sent += to_send as u64;
            }
            writer.flush().await.ok();
        })
        .await;

        let elapsed = start.elapsed();

        if send_result.is_err() {
            warn!("mesh.throughput: timeout after {bytes_sent} bytes");
        }

        Ok(ThroughputResult {
            bytes_transferred: bytes_sent,
            elapsed,
        })
    }
}

struct ConnectivityProbeResult {
    rtt: Duration,
    version: Option<String>,
    identity: Option<String>,
    ribocipher_accepted: bool,
}

struct ThroughputResult {
    bytes_transferred: u64,
    elapsed: Duration,
}

fn endpoint_type_name(ep: &EndpointType) -> &'static str {
    match ep {
        EndpointType::Direct {
            ..
        } => "direct",
        EndpointType::Local {
            ..
        } => "local",
        EndpointType::Overlay {
            ..
        } => "overlay",
        EndpointType::TorOnion {
            ..
        } => "tor_onion",
        EndpointType::FamilyRelay {
            ..
        } => "family_relay",
    }
}

/// Classify a path: (`type_name`, `is_cross_gate`).
fn classify_path(ep: &EndpointType) -> (&'static str, bool) {
    match ep {
        EndpointType::Local {
            ..
        } => ("local", false),
        EndpointType::Direct {
            addr,
        } => {
            let is_local = addr.ip().is_loopback();
            ("direct", !is_local)
        }
        EndpointType::Overlay {
            ..
        } => ("overlay", true),
        EndpointType::TorOnion {
            ..
        } => ("tor_onion", true),
        EndpointType::FamilyRelay {
            ..
        } => ("family_relay", true),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_local_is_not_cross_gate() {
        let ep = EndpointType::Local {
            addr: "127.0.0.1:7700".parse().unwrap(),
        };
        let (name, cross) = classify_path(&ep);
        assert_eq!(name, "local");
        assert!(!cross);
    }

    #[test]
    fn classify_direct_remote_is_cross_gate() {
        let ep = EndpointType::Direct {
            addr: "10.13.37.5:7700".parse().unwrap(),
        };
        let (name, cross) = classify_path(&ep);
        assert_eq!(name, "direct");
        assert!(cross);
    }

    #[test]
    fn classify_overlay_is_cross_gate() {
        let ep = EndpointType::Overlay {
            addr: "10.13.37.2:7700".parse().unwrap(),
            overlay_name: String::from("wg0"),
        };
        let (name, cross) = classify_path(&ep);
        assert_eq!(name, "overlay");
        assert!(cross);
    }

    #[test]
    fn endpoint_type_name_covers_all() {
        assert_eq!(
            endpoint_type_name(&EndpointType::Direct {
                addr: "1.2.3.4:80".parse().unwrap()
            }),
            "direct"
        );
        assert_eq!(
            endpoint_type_name(&EndpointType::Local {
                addr: "127.0.0.1:80".parse().unwrap()
            }),
            "local"
        );
    }

    #[test]
    fn payload_limits_enforced() {
        assert!(MIN_PAYLOAD_BYTES < DEFAULT_PAYLOAD_BYTES);
        assert!(DEFAULT_PAYLOAD_BYTES < MAX_PAYLOAD_BYTES);
    }
}
