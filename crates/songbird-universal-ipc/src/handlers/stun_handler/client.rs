// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! STUN **client** JSON-RPC operations: public address discovery, bind, NAT probing, and classification.
//!
//! These methods delegate to [`songbird_stun::StunClient`] and format JSON results for IPC.

use serde_json::{Value, json};
use songbird_stun::{NatType, PortPattern, StunClient, classify_nat_from_dual_probes};
use tokio::net::UdpSocket;
use tracing::{info, warn};

use super::StunHandler;
use super::config::{DEFAULT_PRIMARY_STUN_SERVER, stun_server_list};

/// Build the JSON-RPC payload for a detected [`PortPattern`] (used by [`StunHandler::handle_probe_port_pattern`]).
#[must_use]
fn port_pattern_ipc_value(pattern: &PortPattern) -> Value {
    match pattern {
        PortPattern::Sequential {
            step,
            last_port,
            predicted_next,
            confidence,
        } => json!({
            "pattern": "sequential",
            "step": step,
            "last_port": last_port,
            "predicted_next": predicted_next,
            "confidence": confidence,
            "supports_coordinated_punch": pattern.supports_coordinated_punch()
        }),
        PortPattern::Random {
            observed,
        } => json!({
            "pattern": "random",
            "observed_ports": observed,
            "supports_coordinated_punch": false
        }),
        PortPattern::Unknown => json!({
            "pattern": "unknown",
            "supports_coordinated_punch": false
        }),
    }
}

/// Human-readable label and description for a [`NatType`].
#[must_use]
fn nat_type_label(nat: NatType) -> (&'static str, &'static str) {
    match nat {
        NatType::PortRestrictedCone => {
            ("cone", "Same port for different destinations — likely cone NAT (good for punching)")
        }
        NatType::Symmetric => (
            "symmetric",
            "Different ports for different destinations — symmetric NAT (needs relay-assisted punch)",
        ),
        _ => ("unknown", "Could not classify NAT type"),
    }
}

impl StunHandler {
    /// Handle `stun.get_public_address` method - Discover public IP/port via STUN
    pub async fn handle_get_public_address(&self, params: Value) -> Result<Value, String> {
        info!("🌐 STUN: Discovering public address via racing");

        let servers: Vec<String> = if let Some(servers_val) = params.get("servers") {
            serde_json::from_value(servers_val.clone())
                .map_err(|e| format!("Invalid 'servers' parameter: {e}"))?
        } else {
            stun_server_list()
        };

        if servers.is_empty() {
            return Err("No STUN servers provided".to_string());
        }

        let client = StunClient::new();
        let server_refs: Vec<&str> = servers.iter().map(std::string::String::as_str).collect();
        let servers_count = server_refs.len();

        let public_addr = client
            .discover_public_address_racing(&server_refs)
            .await
            .map_err(|e| format!("STUN discovery failed: {e}"))?;

        info!("✅ STUN discovered public address: {}", public_addr);

        Ok(json!({
            "public_address": public_addr.ip().to_string(),
            "public_port": public_addr.port(),
            "full_address": public_addr.to_string(),
            "nat_type": "unknown",
            "servers_tried": servers_count,
            "method": "stun_racing"
        }))
    }

    /// Handle `stun.bind` method - Bind local port and discover NAT mapping
    pub async fn handle_bind(&self, params: Value) -> Result<Value, String> {
        let stun_server = params.get("stun_server").and_then(|v| v.as_str()).map_or_else(
            || {
                stun_server_list()
                    .into_iter()
                    .next()
                    .unwrap_or_else(|| DEFAULT_PRIMARY_STUN_SERVER.to_string())
            },
            str::to_owned,
        );

        info!("🌐 STUN: Binding and discovering NAT mapping via {}", stun_server);

        let client = StunClient::new();

        let endpoint = client
            .discover_public_endpoint(&stun_server)
            .await
            .map_err(|e| format!("STUN bind failed: {e}"))?;

        info!("✅ STUN bind result: {} (NAT type: {:?})", endpoint.address, endpoint.nat_type);

        Ok(json!({
            "local_address": "0.0.0.0:0",
            "public_address": endpoint.address.to_string(),
            "public_ip": endpoint.address.ip().to_string(),
            "public_port": endpoint.address.port(),
            "nat_type": format!("{:?}", endpoint.nat_type).to_lowercase(),
            "stun_server": stun_server
        }))
    }

    /// Handle `stun.probe_port_pattern` method - Detect NAT port allocation pattern
    pub async fn handle_probe_port_pattern(&self, params: Value) -> Result<Value, String> {
        let stun_server = params.get("stun_server").and_then(|v| v.as_str()).map_or_else(
            || {
                stun_server_list()
                    .into_iter()
                    .next()
                    .unwrap_or_else(|| DEFAULT_PRIMARY_STUN_SERVER.to_string())
            },
            str::to_owned,
        );

        let probes = params
            .get("probes")
            .and_then(serde_json::Value::as_u64)
            .map_or(5, |n| usize::try_from(n).unwrap_or(5));

        info!("🔍 STUN: Probing port pattern ({} probes to {})", probes, stun_server);

        let client = StunClient::new();

        let pattern = client
            .probe_port_pattern(&stun_server, probes)
            .await
            .map_err(|e| format!("Port pattern probing failed: {e}"))?;

        match &pattern {
            PortPattern::Sequential {
                step,
                predicted_next,
                ..
            } => {
                info!("✅ Sequential pattern: step={}, predicted={}", step, predicted_next);
            }
            PortPattern::Random {
                observed,
            } => {
                info!("⚠️ Random pattern: {} ports observed", observed.len());
            }
            PortPattern::Unknown => {
                warn!("⚠️ Could not determine port pattern");
            }
        }

        Ok(port_pattern_ipc_value(&pattern))
    }

    /// Handle `stun.detect_nat_type` method — detect NAT type via shared-socket dual-probe.
    ///
    /// Binds a single UDP socket and sends STUN binding requests to two
    /// different servers, then compares the reflexive addresses. This is
    /// the correct algorithm: using separate sockets (as the pre-H2-13 code
    /// did) defeats symmetric NAT detection because each socket gets its
    /// own NAT mapping regardless of destination.
    pub async fn handle_detect_nat_type(&self, params: Value) -> Result<Value, String> {
        let servers: Vec<String> = if let Some(servers_val) = params.get("servers") {
            serde_json::from_value(servers_val.clone())
                .map_err(|e| format!("Invalid 'servers' parameter: {e}"))?
        } else {
            stun_server_list().into_iter().take(2).collect()
        };

        if servers.len() < 2 {
            return Err("Need at least 2 STUN servers for NAT type detection".to_string());
        }

        info!("🔍 STUN: Detecting NAT type via {} servers (shared socket)", servers.len());

        let socket = UdpSocket::bind("0.0.0.0:0")
            .await
            .map_err(|e| format!("Failed to bind shared probe socket: {e}"))?;

        let client = StunClient::new();

        let addr1 = client
            .discover_on_socket(&socket, &servers[0])
            .await
            .map_err(|e| format!("STUN server 1 failed: {e}"))?;

        let addr2 = client
            .discover_on_socket(&socket, &servers[1])
            .await
            .map_err(|e| format!("STUN server 2 failed: {e}"))?;

        let classified = classify_nat_from_dual_probes(addr1, addr2);
        let (nat_type, description) = nat_type_label(classified);

        info!("✅ NAT type detected: {} — {}", nat_type, description);

        Ok(json!({
            "nat_type": nat_type,
            "description": description,
            "probe_results": {
                "server_1": { "server": &servers[0], "public_addr": addr1.to_string() },
                "server_2": { "server": &servers[1], "public_addr": addr2.to_string() }
            },
            "recommendation": if classified == NatType::Symmetric {
                "Use relay-assisted coordinated punch (punch.coordinate)"
            } else {
                "Direct hole punch should work (punch.request)"
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::{StunHandler, nat_type_label, port_pattern_ipc_value};
    use serde_json::json;
    use songbird_stun::{
        NatType, PortPattern, StunClient, StunError, StunServer, classify_nat_from_dual_probes,
    };
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::time::Duration;
    use tokio::sync::oneshot;
    use tokio::task::JoinHandle;

    async fn start_local_stun_server() -> (JoinHandle<()>, SocketAddr) {
        let (ready_tx, ready_rx) = oneshot::channel();
        let handle = tokio::spawn(async move {
            let server = StunServer::new("127.0.0.1:0".parse().expect("loopback parse"));
            let _ = server.run_with_ready(ready_tx).await;
        });
        let addr = ready_rx.await.expect("STUN server should signal bound address");
        (handle, addr)
    }

    #[test]
    fn port_pattern_ipc_value_sequential_high_confidence() {
        let p = PortPattern::Sequential {
            step: 1,
            last_port: 4000,
            predicted_next: 4001,
            confidence: 0.9,
        };
        let v = port_pattern_ipc_value(&p);
        assert_eq!(v["pattern"], "sequential");
        assert_eq!(v["step"], 1);
        assert_eq!(v["last_port"], 4000);
        assert_eq!(v["predicted_next"], 4001);
        assert_eq!(v["confidence"], 0.9);
        assert_eq!(v["supports_coordinated_punch"], true);
    }

    #[test]
    fn port_pattern_ipc_value_sequential_low_confidence_no_coordinated_punch() {
        let p = PortPattern::Sequential {
            step: 2,
            last_port: 5000,
            predicted_next: 5002,
            confidence: 0.4,
        };
        let v = port_pattern_ipc_value(&p);
        assert_eq!(v["pattern"], "sequential");
        assert_eq!(v["supports_coordinated_punch"], false);
    }

    #[test]
    fn port_pattern_ipc_value_random() {
        let p = PortPattern::Random {
            observed: vec![4100, 9999, 12],
        };
        let v = port_pattern_ipc_value(&p);
        assert_eq!(v["pattern"], "random");
        assert_eq!(v["observed_ports"], json!([4100, 9999, 12]));
        assert_eq!(v["supports_coordinated_punch"], false);
    }

    #[test]
    fn port_pattern_ipc_value_unknown() {
        let v = port_pattern_ipc_value(&PortPattern::Unknown);
        assert_eq!(v["pattern"], "unknown");
        assert_eq!(v["supports_coordinated_punch"], false);
        assert!(v.as_object().unwrap().get("observed_ports").is_none());
    }

    #[test]
    fn port_pattern_library_json_roundtrip_sequential_random_unknown() {
        use serde_json::{from_value, to_value};

        let seq = PortPattern::Sequential {
            step: 1,
            last_port: 100,
            predicted_next: 101,
            confidence: 0.8,
        };
        assert_eq!(seq, from_value(to_value(&seq).unwrap()).unwrap());

        let rand = PortPattern::Random {
            observed: vec![1, 2],
        };
        assert_eq!(rand, from_value(to_value(&rand).unwrap()).unwrap());

        let unk = PortPattern::Unknown;
        assert_eq!(unk, from_value(to_value(&unk).unwrap()).unwrap());
    }

    #[test]
    fn classify_dual_probes_different_public_ips() {
        let a = SocketAddr::from((Ipv4Addr::new(1, 1, 1, 1), 10_000));
        let b = SocketAddr::from((Ipv4Addr::new(2, 2, 2, 2), 10_000));
        assert_eq!(classify_nat_from_dual_probes(a, b), NatType::Unknown);
        assert_eq!(nat_type_label(NatType::Unknown).0, "unknown");
    }

    #[test]
    fn classify_dual_probes_cone_same_ip_and_port() {
        let ip = IpAddr::V4(Ipv4Addr::new(203, 0, 113, 1));
        let a = SocketAddr::new(ip, 50_000);
        let b = SocketAddr::new(ip, 50_000);
        assert_eq!(classify_nat_from_dual_probes(a, b), NatType::PortRestrictedCone);
        assert_eq!(nat_type_label(NatType::PortRestrictedCone).0, "cone");
    }

    #[test]
    fn classify_dual_probes_symmetric_same_ip_different_ports() {
        let ip = IpAddr::V4(Ipv4Addr::new(203, 0, 113, 2));
        let a = SocketAddr::new(ip, 50_000);
        let b = SocketAddr::new(ip, 50_001);
        assert_eq!(classify_nat_from_dual_probes(a, b), NatType::Symmetric);
        assert_eq!(nat_type_label(NatType::Symmetric).0, "symmetric");
    }

    #[test]
    fn map_err_stun_discovery_failed_prefix_matches_all_servers_failed_display() {
        let inner = StunError::AllServersFailed("Last error: Network error: timed out".into());
        assert_eq!(
            format!("STUN discovery failed: {inner}"),
            "STUN discovery failed: All STUN servers failed: Last error: Network error: timed out"
        );
    }

    #[test]
    fn map_err_stun_bind_failed_prefix_matches_timeout_display() {
        let inner = StunError::Timeout(Duration::from_secs(2));
        assert_eq!(
            format!("STUN bind failed: {inner}"),
            format!("STUN bind failed: STUN request timeout after {:?}", Duration::from_secs(2))
        );
    }

    #[test]
    fn map_err_port_pattern_probing_failed_prefix_matches_network_display() {
        let inner = StunError::Network("Failed to resolve STUN server: nxdomain".into());
        assert_eq!(
            format!("Port pattern probing failed: {inner}"),
            "Port pattern probing failed: Network error: Failed to resolve STUN server: nxdomain"
        );
    }

    #[test]
    fn map_err_stun_server_1_and_2_failed_prefixes_match_display() {
        let e1 = StunError::InvalidResponse("bad magic".into());
        assert_eq!(
            format!("STUN server 1 failed: {e1}"),
            "STUN server 1 failed: Invalid STUN response: bad magic"
        );
        let e2 = StunError::ServerError("500".into());
        assert_eq!(
            format!("STUN server 2 failed: {e2}"),
            "STUN server 2 failed: STUN server error: 500"
        );
    }

    #[test]
    fn invalid_servers_parameter_format_matches_handler() {
        let bad = json!("not-an-array");
        let inner = serde_json::from_value::<Vec<String>>(bad.clone()).unwrap_err();
        let expected = format!("Invalid 'servers' parameter: {inner}");
        let got = serde_json::from_value::<Vec<String>>(bad)
            .map_err(|e| format!("Invalid 'servers' parameter: {e}"))
            .unwrap_err();
        assert_eq!(got, expected);
    }

    #[tokio::test]
    async fn get_public_address_success_against_local_stun_server() {
        let (server_handle, stun_addr) = start_local_stun_server().await;
        let handler = StunHandler::new();
        let server_str = stun_addr.to_string();
        let result = handler
            .handle_get_public_address(json!({ "servers": [server_str.as_str()] }))
            .await
            .expect("get_public_address against local STUN");
        assert_eq!(result["method"], "stun_racing");
        assert_eq!(result["servers_tried"], 1);
        assert_eq!(result["nat_type"], "unknown");
        assert!(result["full_address"].as_str().is_some_and(|s| s.contains(':')));
        server_handle.abort();
    }

    #[tokio::test]
    async fn bind_success_against_local_stun_server() {
        let (server_handle, stun_addr) = start_local_stun_server().await;
        let handler = StunHandler::new();
        let server_str = stun_addr.to_string();
        let result =
            handler.handle_bind(json!({ "stun_server": server_str.as_str() })).await.expect("bind");
        assert_eq!(result["nat_type"], "unknown");
        assert_eq!(result["stun_server"], server_str);
        assert_eq!(result["local_address"], "0.0.0.0:0");
        server_handle.abort();
    }

    #[tokio::test]
    async fn probe_port_pattern_success_against_local_stun_server() {
        let (server_handle, stun_addr) = start_local_stun_server().await;
        let handler = StunHandler::new();
        let server_str = stun_addr.to_string();
        let result = handler
            .handle_probe_port_pattern(json!({ "stun_server": server_str.as_str(), "probes": 4 }))
            .await
            .expect("probe");
        let pat = result["pattern"].as_str().expect("pattern");
        assert!(matches!(pat, "sequential" | "random" | "unknown"), "unexpected pattern: {pat}");
        server_handle.abort();
    }

    #[tokio::test]
    async fn detect_nat_type_success_against_local_stun_server() {
        let (server_handle, stun_addr) = start_local_stun_server().await;
        let handler = StunHandler::new();
        let bind = stun_addr.to_string();
        let result = handler
            .handle_detect_nat_type(json!({ "servers": [bind.as_str(), bind.as_str()] }))
            .await
            .expect("detect_nat_type");
        let nat = result["nat_type"].as_str().expect("nat_type");
        assert!(matches!(nat, "cone" | "symmetric" | "unknown"), "unexpected nat_type: {nat}");
        let rec = result["recommendation"].as_str().expect("recommendation");
        if nat == "symmetric" {
            assert!(rec.contains("coordinate"));
        } else {
            assert!(rec.contains("punch.request") || rec.contains("hole punch"));
        }
        server_handle.abort();
    }

    #[tokio::test]
    async fn get_public_address_discovery_error_matches_stun_client_wrapper() {
        let dead = "127.0.0.1:1";
        let client_err = StunClient::new()
            .discover_public_address_racing(&[dead])
            .await
            .expect_err("client should fail against discard port");
        let handler = StunHandler::new();
        let handler_err = handler
            .handle_get_public_address(json!({ "servers": [dead] }))
            .await
            .expect_err("handler should fail");
        assert_eq!(handler_err, format!("STUN discovery failed: {client_err}"));
    }

    #[tokio::test]
    async fn bind_error_matches_stun_client_wrapper() {
        let dead = "127.0.0.1:1";
        let client_err = StunClient::new()
            .discover_public_endpoint(dead)
            .await
            .expect_err("endpoint discovery should fail");
        let handler = StunHandler::new();
        let handler_err = handler
            .handle_bind(json!({ "stun_server": dead }))
            .await
            .expect_err("bind handler should fail");
        assert_eq!(handler_err, format!("STUN bind failed: {client_err}"));
    }

    #[tokio::test]
    async fn detect_nat_type_server1_error_matches_stun_client_wrapper() {
        let dead = "127.0.0.1:1";
        let client_err = StunClient::new()
            .discover_public_address(dead)
            .await
            .expect_err("discover should fail");
        let handler = StunHandler::new();
        let handler_err = handler
            .handle_detect_nat_type(json!({ "servers": [dead, "127.0.0.1:2"] }))
            .await
            .expect_err("first server should fail");
        assert_eq!(handler_err, format!("STUN server 1 failed: {client_err}"));
    }

    #[tokio::test]
    async fn detect_nat_type_server2_error_matches_stun_client_wrapper() {
        let (server_handle, stun_addr) = start_local_stun_server().await;
        let dead = "127.0.0.1:1";
        let server_str = stun_addr.to_string();
        let client_err = StunClient::new()
            .discover_public_address(dead)
            .await
            .expect_err("second discover should fail");
        let handler = StunHandler::new();
        let handler_err = handler
            .handle_detect_nat_type(json!({ "servers": [server_str.as_str(), dead] }))
            .await
            .expect_err("second server should fail");
        assert_eq!(handler_err, format!("STUN server 2 failed: {client_err}"));
        server_handle.abort();
    }

    #[tokio::test]
    async fn probe_port_pattern_error_matches_stun_client_wrapper() {
        let bad_host = "this-host-should-not-resolve.invalid:3478";
        let client_err = StunClient::new()
            .probe_port_pattern(bad_host, 2)
            .await
            .expect_err("probe should fail on DNS");
        let handler = StunHandler::new();
        let handler_err = handler
            .handle_probe_port_pattern(json!({
                "stun_server": bad_host,
                "probes": 2
            }))
            .await
            .expect_err("handler should fail");
        assert_eq!(handler_err, format!("Port pattern probing failed: {client_err}"));
    }

    #[tokio::test]
    async fn get_public_address_invalid_servers_parameter_matches_handler_exactly() {
        let handler = StunHandler::new();
        let bad = json!("not-an-array");
        let inner = serde_json::from_value::<Vec<String>>(bad.clone()).unwrap_err();
        let expected = format!("Invalid 'servers' parameter: {inner}");
        let err = handler
            .handle_get_public_address(json!({ "servers": bad }))
            .await
            .expect_err("invalid servers");
        assert_eq!(err, expected);
    }

    #[tokio::test]
    async fn detect_nat_type_invalid_servers_parameter_matches_handler_exactly() {
        let handler = StunHandler::new();
        let bad = json!(false);
        let inner = serde_json::from_value::<Vec<String>>(bad.clone()).unwrap_err();
        let expected = format!("Invalid 'servers' parameter: {inner}");
        let err = handler
            .handle_detect_nat_type(json!({ "servers": bad }))
            .await
            .expect_err("invalid servers");
        assert_eq!(err, expected);
    }
}
