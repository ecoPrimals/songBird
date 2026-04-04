// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! HTTP registry, container metadata, and DNS-based endpoint probing.

use songbird_http_client::IpcHttpClient;
use songbird_types::{SongbirdError, SongbirdResult};
use tracing::debug;

use super::types::{CapabilityEndpoint, CapabilityType, DiscoveryMethod};

/// Discover from service registry
pub(super) async fn discover_from_registry(
    capability: &CapabilityType,
) -> SongbirdResult<Option<CapabilityEndpoint>> {
    let Ok(registry_endpoint) = songbird_process_env::var("SERVICE_REGISTRY_ENDPOINT") else {
        return Ok(None);
    };

    debug!("Querying service registry for {} capability", capability.as_str());

    let client = IpcHttpClient::new()
        .await
        .map_err(|e| SongbirdError::network(format!("Failed to create HTTP client: {e}")))?;

    let consul_url = format!("{}/v1/catalog/service/{}", registry_endpoint, capability.as_str());
    match client.get(&consul_url).await {
        Ok(response) if response.is_success() => {
            if let Ok(services) = response.json::<Vec<serde_json::Value>>().await
                && let Some(service) = services.first()
                && let (Some(address), Some(port)) = (
                    service
                        .get("ServiceAddress")
                        .and_then(|v| v.as_str())
                        .or_else(|| service.get("Address").and_then(|v| v.as_str())),
                    service
                        .get("ServicePort")
                        .and_then(serde_json::Value::as_u64)
                        .or_else(|| service.get("Port").and_then(serde_json::Value::as_u64)),
                )
            {
                let endpoint = if address.contains("://") {
                    format!("{address}:{port}")
                } else {
                    format!("http://{address}:{port}")
                };

                debug!("Found {} capability at {} via registry", capability.as_str(), endpoint);

                return Ok(Some(CapabilityEndpoint {
                    capability: capability.clone(),
                    endpoint,
                    provider_id: service
                        .get("ServiceName")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    discovery_method: DiscoveryMethod::ServiceRegistry,
                    confidence: 0.9,
                    discovered_at: std::time::SystemTime::now(),
                }));
            }
        }
        Ok(_) => debug!("Registry returned non-success status"),
        Err(e) => debug!("Registry query failed: {}", e),
    }

    Ok(None)
}

/// Discover from container metadata
pub(super) async fn discover_from_container_metadata(
    capability: &CapabilityType,
) -> SongbirdResult<Option<CapabilityEndpoint>> {
    let Ok(metadata_api) = songbird_process_env::var("CONTAINER_METADATA_API") else {
        return Ok(None);
    };

    debug!("Querying container metadata for {} capability", capability.as_str());

    let client = IpcHttpClient::new()
        .await
        .map_err(|e| SongbirdError::network(format!("Failed to create HTTP client: {e}")))?;

    let service_name = format!("{}-service", capability.as_str().to_lowercase());
    let k8s_url = format!("{metadata_api}/api/v1/services/{service_name}");

    match client.get(&k8s_url).await {
        Ok(response) if response.is_success() => {
            if let Ok(service) = response.json::<serde_json::Value>().await
                && let (Some(cluster_ip), Some(ports)) = (
                    service.get("spec").and_then(|s| s.get("clusterIP")).and_then(|v| v.as_str()),
                    service.get("spec").and_then(|s| s.get("ports")).and_then(|v| v.as_array()),
                )
                && let Some(first_port) =
                    ports.first().and_then(|p| p.get("port")).and_then(serde_json::Value::as_u64)
            {
                let endpoint = format!("http://{cluster_ip}:{first_port}");

                debug!(
                    "Found {} capability at {} via container metadata",
                    capability.as_str(),
                    endpoint
                );

                return Ok(Some(CapabilityEndpoint {
                    capability: capability.clone(),
                    endpoint,
                    provider_id: Some(service_name),
                    discovery_method: DiscoveryMethod::ContainerMetadata,
                    confidence: 0.95,
                    discovered_at: std::time::SystemTime::now(),
                }));
            }
        }
        Ok(_) => debug!("Container metadata API returned non-success status"),
        Err(e) => debug!("Container metadata query failed: {}", e),
    }

    Ok(None)
}

/// Discover from DNS
pub(super) async fn discover_from_dns(
    capability: &CapabilityType,
) -> SongbirdResult<Option<CapabilityEndpoint>> {
    let Ok(dns_domain) = songbird_process_env::var("SERVICE_DISCOVERY_DOMAIN") else {
        return Ok(None);
    };

    debug!("Querying DNS for {} capability", capability.as_str());

    let service_name = format!("_{}._tcp.{}", capability.as_str().to_lowercase(), dns_domain);

    match tokio::net::lookup_host(service_name.as_str()).await {
        Ok(mut addrs) => {
            if let Some(addr) = addrs.next() {
                let endpoint = format!("http://{addr}");

                debug!("Found {} capability at {} via DNS SRV", capability.as_str(), endpoint);

                return Ok(Some(CapabilityEndpoint {
                    capability: capability.clone(),
                    endpoint,
                    provider_id: Some(service_name.clone()),
                    discovery_method: DiscoveryMethod::Dns,
                    confidence: 0.8,
                    discovered_at: std::time::SystemTime::now(),
                }));
            }
            debug!("DNS SRV query succeeded but returned no addresses");
        }
        Err(e) => debug!("DNS SRV query failed: {}", e),
    }

    Ok(None)
}
