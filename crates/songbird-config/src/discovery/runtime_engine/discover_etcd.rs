// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::net::SocketAddr;
use tracing::debug;

use super::{CapabilityDiscoveryEngine, DiscoveredService};

fn etcd_capability_prefix(capability: &str) -> String {
    format!("/songbird/services/{capability}/")
}

fn etcd_range_end_b64_for_prefix(prefix: &str) -> String {
    let mut end = prefix.as_bytes().to_vec();
    if let Some(last) = end.last_mut() {
        *last = last.wrapping_add(1);
    }
    songbird_http_client::base64_encode(&end)
}

fn etcd_kv_range_url(endpoint: &str) -> String {
    format!("{}/v3/kv/range", endpoint.trim_end_matches('/'))
}

fn parse_etcd_kv_entries(
    kvs: &[serde_json::Value],
    capability: &str,
    etcd_endpoint: &str,
) -> Vec<DiscoveredService> {
    let mut discovered = Vec::new();
    for kv in kvs {
        let value_b64 = kv.get("value").and_then(|v| v.as_str()).unwrap_or("");
        let value_bytes = songbird_http_client::base64_decode(value_b64).unwrap_or_default();
        let value = String::from_utf8_lossy(&value_bytes);
        if let Ok(addr) = value.parse::<SocketAddr>() {
            let mut metadata = HashMap::new();
            metadata.insert("source".to_string(), "etcd".to_string());
            metadata.insert("etcd_endpoint".to_string(), etcd_endpoint.to_string());
            discovered.push(DiscoveredService {
                address: addr,
                capabilities: vec![capability.to_string()],
                metadata,
                discovered_at: std::time::SystemTime::now(),
            });
        }
    }
    discovered
}

/// Discover from etcd v3 key-value store by capability prefix.
///
/// Uses [`IpcHttpClient`] (Tower Atomic) to query the etcd v3 HTTP
/// gateway (`POST /v3/kv/range`). Keys under `/songbird/services/<cap>/`
/// store `host:port` values. Tries each endpoint in order.
pub(super) async fn discover_from_etcd(
    _engine: &CapabilityDiscoveryEngine,
    endpoints: &[String],
    capability: &str,
) -> SongbirdResult<Vec<DiscoveredService>> {
    use songbird_http_client::ipc_client::IpcHttpClient;

    debug!(
        target: "songbird_config::discovery",
        backend = "etcd",
        endpoints = ?endpoints,
        %capability,
        "Querying etcd for capability services via Tower Atomic"
    );

    let prefix = etcd_capability_prefix(capability);
    let prefix_b64 = songbird_http_client::base64_encode(prefix.as_bytes());
    let range_end = etcd_range_end_b64_for_prefix(&prefix);

    let body = serde_json::json!({
        "key": prefix_b64,
        "range_end": range_end,
    });

    let client = IpcHttpClient::new()
        .await
        .map_err(|e| SongbirdError::discovery(format!("IPC HTTP client init failed: {e}")))?;

    for ep in endpoints {
        let url = etcd_kv_range_url(ep);
        let Ok(builder) = client.post(&url).await.json(&body) else {
            continue;
        };
        let resp = match builder.send().await {
            Ok(r) if r.is_success() => r,
            _ => continue,
        };

        let json: serde_json::Value = match resp.json().await {
            Ok(v) => v,
            Err(_) => continue,
        };

        let discovered = json
            .get("kvs")
            .and_then(|v| v.as_array())
            .map(|kvs| parse_etcd_kv_entries(kvs, capability, ep))
            .unwrap_or_default();

        debug!(
            target: "songbird_config::discovery",
            backend = "etcd",
            count = discovered.len(),
            "etcd discovery complete"
        );
        return Ok(discovered);
    }

    Err(SongbirdError::discovery(
        "All etcd endpoints unreachable; capability discovery deferred to other backends",
    ))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;
    use std::net::SocketAddr;

    #[test]
    fn etcd_capability_prefix_embeds_capability() {
        assert_eq!(etcd_capability_prefix("alpha"), "/songbird/services/alpha/");
    }

    #[test]
    fn etcd_range_end_increments_last_byte_for_exclusive_range() {
        let prefix = "/songbird/services/x/";
        let end = etcd_range_end_b64_for_prefix(prefix);
        let decoded = songbird_http_client::base64_decode(&end).unwrap();
        let mut expected = prefix.as_bytes().to_vec();
        *expected.last_mut().unwrap() += 1;
        assert_eq!(decoded, expected);
    }

    #[test]
    fn etcd_kv_range_url_trims_trailing_slash() {
        assert_eq!(etcd_kv_range_url("http://etcd:2379/"), "http://etcd:2379/v3/kv/range");
    }

    #[test]
    fn parse_etcd_kv_entries_decodes_host_port() {
        let addr_str = "192.168.1.10:4000";
        let b64 = songbird_http_client::base64_encode(addr_str.as_bytes());
        let kvs = vec![serde_json::json!({ "value": b64 })];
        let out = parse_etcd_kv_entries(&kvs, "cap1", "http://e:2379");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].address, addr_str.parse::<SocketAddr>().unwrap());
        assert_eq!(out[0].capabilities, vec!["cap1"]);
        assert_eq!(out[0].metadata.get("source"), Some(&"etcd".to_string()));
    }
}
