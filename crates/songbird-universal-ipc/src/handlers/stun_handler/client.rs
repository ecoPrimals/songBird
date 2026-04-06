// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! STUN **client** JSON-RPC operations: public address discovery, bind, NAT probing, and classification.
//!
//! These methods delegate to [`songbird_stun::StunClient`] and format JSON results for IPC.

use serde_json::{Value, json};
use songbird_stun::StunClient;
use tracing::{info, warn};

use super::StunHandler;
use super::config::{DEFAULT_PRIMARY_STUN_SERVER, stun_server_list};

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

        let response = match &pattern {
            songbird_stun::PortPattern::Sequential {
                step,
                last_port,
                predicted_next,
                confidence,
            } => {
                info!("✅ Sequential pattern: step={}, predicted={}", step, predicted_next);
                json!({
                    "pattern": "sequential",
                    "step": step,
                    "last_port": last_port,
                    "predicted_next": predicted_next,
                    "confidence": confidence,
                    "supports_coordinated_punch": pattern.supports_coordinated_punch()
                })
            }
            songbird_stun::PortPattern::Random {
                observed,
            } => {
                info!("⚠️ Random pattern: {} ports observed", observed.len());
                json!({
                    "pattern": "random",
                    "observed_ports": observed,
                    "supports_coordinated_punch": false
                })
            }
            songbird_stun::PortPattern::Unknown => {
                warn!("⚠️ Could not determine port pattern");
                json!({
                    "pattern": "unknown",
                    "supports_coordinated_punch": false
                })
            }
        };

        Ok(response)
    }

    /// Handle `stun.detect_nat_type` method - Detect NAT type via multiple probes
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

        info!("🔍 STUN: Detecting NAT type via {} servers", servers.len());

        let client = StunClient::new();

        let addr1 = client
            .discover_public_address(&servers[0])
            .await
            .map_err(|e| format!("STUN server 1 failed: {e}"))?;

        let addr2 = client
            .discover_public_address(&servers[1])
            .await
            .map_err(|e| format!("STUN server 2 failed: {e}"))?;

        let (nat_type, description) = if addr1.ip() != addr2.ip() {
            ("unknown", "Different public IPs detected — unusual topology")
        } else if addr1.port() == addr2.port() {
            ("cone", "Same port for different destinations — likely cone NAT (good for punching)")
        } else {
            (
                "symmetric",
                "Different ports for different destinations — symmetric NAT (needs relay-assisted punch)",
            )
        };

        info!("✅ NAT type detected: {} — {}", nat_type, description);

        Ok(json!({
            "nat_type": nat_type,
            "description": description,
            "probe_results": {
                "server_1": { "server": &servers[0], "public_addr": addr1.to_string() },
                "server_2": { "server": &servers[1], "public_addr": addr2.to_string() }
            },
            "recommendation": if nat_type == "symmetric" {
                "Use relay-assisted coordinated punch (punch.coordinate)"
            } else {
                "Direct hole punch should work (punch.request)"
            }
        }))
    }
}
