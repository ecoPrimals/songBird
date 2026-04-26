// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::net::SocketAddr;
use tracing::debug;

use super::{CapabilityDiscoveryEngine, DiscoveredService};

fn consul_catalog_service_url(endpoint: &str, capability: &str) -> String {
    format!("{}/v1/catalog/service/{capability}", endpoint.trim_end_matches('/'))
}

fn parse_consul_catalog_entries(
    entries: &[serde_json::Value],
    capability: &str,
    consul_endpoint: &str,
) -> Vec<DiscoveredService> {
    let mut discovered = Vec::new();
    for entry in entries {
        let address = entry
            .get("ServiceAddress")
            .and_then(|v| v.as_str())
            .or_else(|| entry.get("Address").and_then(|v| v.as_str()))
            .unwrap_or(songbird_types::constants::LOCALHOST);
        let port = entry.get("ServicePort").and_then(serde_json::Value::as_u64).unwrap_or(0);

        if port == 0 {
            continue;
        }

        if let Ok(addr) = format!("{address}:{port}").parse::<SocketAddr>() {
            let tags: Vec<String> = entry
                .get("ServiceTags")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|t| t.as_str().map(String::from)).collect())
                .unwrap_or_default();

            let mut metadata = HashMap::new();
            metadata.insert("source".to_string(), "consul".to_string());
            metadata.insert("consul_endpoint".to_string(), consul_endpoint.to_string());

            discovered.push(DiscoveredService {
                address: addr,
                capabilities: if tags.is_empty() {
                    vec![capability.to_string()]
                } else {
                    tags
                },
                metadata,
                discovered_at: std::time::SystemTime::now(),
            });
        }
    }
    discovered
}

/// Discover from Consul service catalog by capability tag.
///
/// Uses [`IpcHttpClient`] (Tower Atomic: Songbird TLS + `security provider` crypto)
/// to query `GET /v1/catalog/service/<capability>`. Falls back gracefully
/// on network or parsing errors so other backends can still contribute.
pub(super) async fn discover_from_consul(
    _engine: &CapabilityDiscoveryEngine,
    endpoint: &str,
    capability: &str,
) -> SongbirdResult<Vec<DiscoveredService>> {
    debug!(
        target: "songbird_config::discovery",
        backend = "consul",
        endpoint,
        %capability,
        "Querying Consul catalog for capability via Tower Atomic"
    );

    let client = songbird_http_client::IpcHttpClient::new()
        .await
        .map_err(|e| SongbirdError::discovery(format!("IPC HTTP client init failed: {e}")))?;

    let url = consul_catalog_service_url(endpoint, capability);
    let response = client.get(&url).await.map_err(|e| {
        SongbirdError::discovery(format!("Consul HTTP request to {url} failed: {e}"))
    })?;

    if !response.is_success() {
        return Err(SongbirdError::discovery(format!(
            "Consul returned HTTP {} for {url}",
            response.status()
        )));
    }

    let entries: Vec<serde_json::Value> = response
        .json()
        .await
        .map_err(|e| SongbirdError::discovery(format!("Failed to parse Consul response: {e}")))?;

    let discovered = parse_consul_catalog_entries(&entries, capability, endpoint);

    debug!(
        target: "songbird_config::discovery",
        backend = "consul",
        count = discovered.len(),
        "Consul discovery complete"
    );
    Ok(discovered)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;
    use std::net::SocketAddr;

    #[test]
    fn consul_catalog_service_url_trims_slash_and_formats_path() {
        assert_eq!(
            consul_catalog_service_url("http://consul:8500/", "foo-bar"),
            "http://consul:8500/v1/catalog/service/foo-bar"
        );
        assert_eq!(
            consul_catalog_service_url("http://consul:8500", "cap"),
            "http://consul:8500/v1/catalog/service/cap"
        );
    }

    #[test]
    fn parse_consul_catalog_entries_skips_zero_port_and_parses_tags() {
        let cap = "my-cap";
        let entries = vec![
            serde_json::json!({"ServicePort": 0, "ServiceAddress": "127.0.0.1"}),
            serde_json::json!({
                "ServiceAddress": "10.0.0.2",
                "ServicePort": 9300,
                "ServiceTags": ["a", "b"]
            }),
            serde_json::json!({
                "Address": "10.0.0.3",
                "ServicePort": 9400
            }),
        ];
        let out = parse_consul_catalog_entries(&entries, cap, "http://c:8500");
        assert_eq!(out.len(), 2);

        assert_eq!(out[0].address, "10.0.0.2:9300".parse::<SocketAddr>().unwrap());
        assert_eq!(out[0].capabilities, vec!["a", "b"]);
        assert_eq!(out[0].metadata.get("source"), Some(&"consul".to_string()));

        assert_eq!(out[1].address, "10.0.0.3:9400".parse::<SocketAddr>().unwrap());
        assert_eq!(out[1].capabilities, vec![cap.to_string()]);
    }
}
