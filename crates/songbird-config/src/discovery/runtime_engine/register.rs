// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use songbird_types::{SongbirdError, SongbirdResult};
use std::net::SocketAddr;
use tracing::debug;

use super::{CapabilityDiscoveryEngine, DiscoveryBackend};

/// Register with a specific backend
pub(super) async fn register_with_backend(
    _engine: &CapabilityDiscoveryEngine,
    backend: &DiscoveryBackend,
    capabilities: &[String],
    address: SocketAddr,
) -> SongbirdResult<()> {
    match backend {
        DiscoveryBackend::Environment => {
            // Environment-based doesn't support registration
            Ok(())
        }
        DiscoveryBackend::MDNS | DiscoveryBackend::DNSSD => {
            use crate::discovery::mdns::MdnsDiscovery;
            let mdns = MdnsDiscovery::new().map_err(|e| SongbirdError::discovery(e.to_string()))?;
            let cap_refs: Vec<&str> = capabilities.iter().map(String::as_str).collect();
            mdns.advertise(&cap_refs).await.map_err(|e| SongbirdError::discovery(e.to_string()))
        }
        DiscoveryBackend::Consul {
            endpoint,
        } => {
            let service_id = format!("songbird-{}", address.port());
            let body = serde_json::json!({
                "ID": service_id,
                "Name": "songbird",
                "Address": address.ip().to_string(),
                "Port": address.port(),
                "Tags": capabilities,
                "Check": {
                    "TCP": address.to_string(),
                    "Interval": "10s",
                    "Timeout": "3s",
                }
            });

            let url = format!("{}/v1/agent/service/register", endpoint.trim_end_matches('/'));
            let client = songbird_http_client::IpcHttpClient::new()
                .await
                .map_err(|e| SongbirdError::discovery(format!("IPC client init: {e}")))?;
            client
                .put(&url)
                .await
                .json(&body)
                .map_err(|e| SongbirdError::discovery(format!("JSON encoding failed: {e}")))?
                .send()
                .await
                .map_err(|e| {
                    SongbirdError::discovery(format!(
                        "Consul registration at {endpoint} failed: {e}"
                    ))
                })?;

            debug!(
                target: "songbird_config::discovery",
                backend = "consul",
                %service_id,
                "Registered with Consul"
            );
            Ok(())
        }
        DiscoveryBackend::Etcd {
            endpoints,
        } => {
            let client = songbird_http_client::IpcHttpClient::new()
                .await
                .map_err(|e| SongbirdError::discovery(format!("IPC client init: {e}")))?;

            for cap in capabilities {
                let key = format!("/songbird/services/{cap}/{address}");
                let key_b64 = songbird_http_client::base64_encode(key.as_bytes());
                let value_b64 = songbird_http_client::base64_encode(address.to_string().as_bytes());

                let body = serde_json::json!({
                    "key": key_b64,
                    "value": value_b64,
                    "lease": 0,
                });

                for ep in endpoints {
                    let url = format!("{}/v3/kv/put", ep.trim_end_matches('/'));
                    if let Ok(builder) = client.post(&url).await.json(&body)
                        && builder.send().await.is_ok()
                    {
                        break;
                    }
                }
            }
            Ok(())
        }
        DiscoveryBackend::Kubernetes {
            ..
        } => {
            // Kubernetes uses service definitions, no dynamic registration needed
            Ok(())
        }
    }
}
