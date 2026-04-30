// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! HTTP registry, container metadata, and DNS-based endpoint probing.

use songbird_http_client::IpcHttpClient;
use songbird_types::{SongbirdError, SongbirdResult};
use tracing::debug;

use super::types::{CapabilityEndpoint, CapabilityType, DiscoveryMethod};
use serde_json::Value;

/// Parses a Consul-style catalog JSON object into `(endpoint, optional_service_name)`.
pub fn parse_consul_catalog_service(service: &Value) -> Option<(String, Option<String>)> {
    let address = service
        .get("ServiceAddress")
        .and_then(Value::as_str)
        .or_else(|| service.get("Address").and_then(Value::as_str))?;
    let port = service
        .get("ServicePort")
        .and_then(Value::as_u64)
        .or_else(|| service.get("Port").and_then(Value::as_u64))?;
    let endpoint = if address.contains("://") {
        format!("{address}:{port}")
    } else {
        format!("http://{address}:{port}")
    };
    let provider_id = service.get("ServiceName").and_then(Value::as_str).map(String::from);
    Some((endpoint, provider_id))
}

/// Parses a Kubernetes `Service` JSON document into `http://cluster_ip:port`.
pub fn parse_kubernetes_service_cluster_endpoint(service: &Value) -> Option<String> {
    let cluster_ip = service.get("spec")?.get("clusterIP")?.as_str()?;
    let ports = service.get("spec")?.get("ports")?.as_array()?;
    let first_port = ports.first()?.get("port")?.as_u64()?;
    Some(format!("http://{cluster_ip}:{first_port}"))
}

/// Discover from service registry
pub async fn discover_from_registry(
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
            if let Ok(services) = response.json::<Vec<Value>>().await
                && let Some(service) = services.first()
                && let Some((endpoint, provider_id)) = parse_consul_catalog_service(service)
            {
                debug!("Found {} capability at {} via registry", capability.as_str(), endpoint);

                return Ok(Some(CapabilityEndpoint {
                    capability: capability.clone(),
                    endpoint,
                    provider_id,
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
            if let Ok(service) = response.json::<Value>().await
                && let Some(endpoint) = parse_kubernetes_service_cluster_endpoint(&service)
            {
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
pub async fn discover_from_dns(
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

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::{
        discover_from_container_metadata, discover_from_dns, discover_from_registry,
        parse_consul_catalog_service, parse_kubernetes_service_cluster_endpoint,
    };
    use crate::capability_endpoints::types::CapabilityType;
    use serde_json::Value;
    use songbird_test_utils::ScopedEnv;

    #[tokio::test]
    async fn discover_from_registry_returns_none_without_endpoint_env() {
        let _e = ScopedEnv::remove("SERVICE_REGISTRY_ENDPOINT").await;
        let out = discover_from_registry(&CapabilityType::Security)
            .await
            .expect("query without registry env");
        assert!(out.is_none());
    }

    #[tokio::test]
    async fn discover_from_container_metadata_returns_none_without_api_env() {
        let _e = ScopedEnv::remove("CONTAINER_METADATA_API").await;
        let out = discover_from_container_metadata(&CapabilityType::Storage)
            .await
            .expect("query without metadata API");
        assert!(out.is_none());
    }

    #[tokio::test]
    async fn discover_from_dns_returns_none_without_domain_env() {
        let _e = ScopedEnv::remove("SERVICE_DISCOVERY_DOMAIN").await;
        let out = discover_from_dns(&CapabilityType::Compute).await.expect("query without domain");
        assert!(out.is_none());
    }

    #[tokio::test]
    async fn discover_from_dns_invalid_srv_name_yields_none() {
        let _e = ScopedEnv::set("SERVICE_DISCOVERY_DOMAIN", "invalid-label-.invalid.").await;
        let out = discover_from_dns(&CapabilityType::Ai).await.expect("dns probe");
        assert!(out.is_none());
    }

    #[test]
    fn parse_consul_prefers_service_address_and_service_port() {
        let v: Value = serde_json::from_str(
            r#"{"ServiceAddress":"10.1.2.3","ServicePort":8500,"ServiceName":"security"}"#,
        )
        .unwrap();
        let (ep, id) = parse_consul_catalog_service(&v).unwrap();
        assert_eq!(ep, "http://10.1.2.3:8500");
        assert_eq!(id.as_deref(), Some("security"));
    }

    #[test]
    fn parse_consul_falls_back_to_address_and_port_fields() {
        let v: Value = serde_json::from_str(r#"{"Address":"192.168.0.5","Port":9999}"#).unwrap();
        let (ep, id) = parse_consul_catalog_service(&v).unwrap();
        assert_eq!(ep, "http://192.168.0.5:9999");
        assert!(id.is_none());
    }

    #[test]
    fn parse_consul_preserves_scheme_when_address_has_uri() {
        let v: Value =
            serde_json::from_str(r#"{"ServiceAddress":"https://edge.example","ServicePort":443}"#)
                .unwrap();
        let (ep, _) = parse_consul_catalog_service(&v).unwrap();
        assert_eq!(ep, "https://edge.example:443");
    }

    #[test]
    fn parse_consul_returns_none_when_port_missing() {
        let v: Value = serde_json::from_str(r#"{"Address":"10.0.0.1"}"#).unwrap();
        assert!(parse_consul_catalog_service(&v).is_none());
    }

    #[test]
    fn parse_kubernetes_cluster_endpoint_from_spec() {
        let v: Value = serde_json::from_str(
            r#"{"spec":{"clusterIP":"10.96.0.42","ports":[{"port":8080,"protocol":"TCP"}]}}"#,
        )
        .unwrap();
        assert_eq!(
            parse_kubernetes_service_cluster_endpoint(&v).as_deref(),
            Some("http://10.96.0.42:8080")
        );
    }

    #[test]
    fn parse_kubernetes_returns_none_without_ports_or_cluster_ip() {
        let bad_ip: Value = serde_json::from_str(r#"{"spec":{"ports":[{"port":80}]}}"#).unwrap();
        assert!(parse_kubernetes_service_cluster_endpoint(&bad_ip).is_none());

        let bad_ports: Value =
            serde_json::from_str(r#"{"spec":{"clusterIP":"10.0.0.1"}}"#).unwrap();
        assert!(parse_kubernetes_service_cluster_endpoint(&bad_ports).is_none());
    }
}
