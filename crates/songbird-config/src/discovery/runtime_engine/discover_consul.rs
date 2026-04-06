// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::net::SocketAddr;
use tracing::debug;

use super::{CapabilityDiscoveryEngine, DiscoveredService};

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

    let url = format!("{}/v1/catalog/service/{capability}", endpoint.trim_end_matches('/'));
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

    let mut discovered = Vec::new();
    for entry in &entries {
        let address = entry
            .get("ServiceAddress")
            .and_then(|v| v.as_str())
            .or_else(|| entry.get("Address").and_then(|v| v.as_str()))
            .unwrap_or("127.0.0.1");
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
            metadata.insert("consul_endpoint".to_string(), endpoint.to_string());

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

    debug!(
        target: "songbird_config::discovery",
        backend = "consul",
        count = discovered.len(),
        "Consul discovery complete"
    );
    Ok(discovered)
}
