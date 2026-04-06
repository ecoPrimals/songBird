// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::net::SocketAddr;
use tracing::debug;

use super::{CapabilityDiscoveryEngine, DiscoveredService};

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

    let prefix = format!("/songbird/services/{capability}/");
    let prefix_b64 = songbird_http_client::base64_encode(prefix.as_bytes());
    let range_end = {
        let mut end = prefix.as_bytes().to_vec();
        if let Some(last) = end.last_mut() {
            *last = last.wrapping_add(1);
        }
        songbird_http_client::base64_encode(&end)
    };

    let body = serde_json::json!({
        "key": prefix_b64,
        "range_end": range_end,
    });

    let client = IpcHttpClient::new()
        .await
        .map_err(|e| SongbirdError::discovery(format!("IPC HTTP client init failed: {e}")))?;

    for ep in endpoints {
        let url = format!("{}/v3/kv/range", ep.trim_end_matches('/'));
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

        let mut discovered = Vec::new();
        if let Some(kvs) = json.get("kvs").and_then(|v| v.as_array()) {
            for kv in kvs {
                let value_b64 = kv.get("value").and_then(|v| v.as_str()).unwrap_or("");
                let value_bytes =
                    songbird_http_client::base64_decode(value_b64).unwrap_or_default();
                let value = String::from_utf8_lossy(&value_bytes);
                if let Ok(addr) = value.parse::<SocketAddr>() {
                    let mut metadata = HashMap::new();
                    metadata.insert("source".to_string(), "etcd".to_string());
                    metadata.insert("etcd_endpoint".to_string(), ep.clone());
                    discovered.push(DiscoveredService {
                        address: addr,
                        capabilities: vec![capability.to_string()],
                        metadata,
                        discovered_at: std::time::SystemTime::now(),
                    });
                }
            }
        }

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
