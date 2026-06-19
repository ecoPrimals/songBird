// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::net::SocketAddr;
use tracing::debug;

use super::{CapabilityDiscoveryEngine, DiscoveredService};

fn kubernetes_dns_service_hostname(capability: &str, ns: &str, cluster_domain: &str) -> String {
    format!("{capability}.{ns}.svc.{cluster_domain}")
}

fn kubernetes_endpoints_list_url(api_base: &str, ns: &str, capability: &str) -> String {
    format!(
        "{}/api/v1/namespaces/{ns}/endpoints?labelSelector=songbird/capability={capability}",
        api_base.trim_end_matches('/')
    )
}

/// Discover from Kubernetes in-cluster service API.
///
/// Uses the in-cluster service account token and API server to list
/// endpoints for services labeled with `songbird/capability=<cap>`.
/// Falls back to DNS-based service resolution when the API is
/// unavailable (SRV records: `_<cap>._tcp.<ns>.svc.cluster.local`).
pub(super) async fn discover_from_kubernetes(
    engine: &CapabilityDiscoveryEngine,
    namespace: Option<&str>,
    capability: &str,
) -> SongbirdResult<Vec<DiscoveredService>> {
    debug!(
        target: "songbird_config::discovery",
        backend = "kubernetes",
        ?namespace,
        %capability,
        "Attempting Kubernetes in-cluster discovery"
    );

    let ns = namespace.unwrap_or("default");

    // Attempt DNS-based discovery first (works without API access)
    let cluster_domain = engine
        .read_env("SONGBIRD_K8S_CLUSTER_DOMAIN")
        .unwrap_or_else(|_| String::from("cluster.local"));
    let dns_name = kubernetes_dns_service_hostname(capability, ns, &cluster_domain);
    if let Ok(addrs) = tokio::net::lookup_host(format!("{dns_name}:0")).await {
        let discovered: Vec<DiscoveredService> = addrs
            .filter(|a| a.port() > 0)
            .map(|addr| {
                let mut metadata = HashMap::new();
                metadata.insert(String::from("source"), String::from("kubernetes-dns"));
                metadata.insert(String::from("namespace"), ns.to_string());
                DiscoveredService {
                    address: addr,
                    capabilities: vec![capability.to_string()],
                    metadata,
                    discovered_at: std::time::SystemTime::now(),
                }
            })
            .collect();

        if !discovered.is_empty() {
            debug!(
                target: "songbird_config::discovery",
                backend = "kubernetes",
                count = discovered.len(),
                "Kubernetes DNS discovery returned results"
            );
            return Ok(discovered);
        }
    }

    // Fall back to Kubernetes API if in-cluster service account is available
    let token_path = engine
        .read_env("SONGBIRD_K8S_TOKEN_PATH")
        .unwrap_or_else(|_| String::from("/var/run/secrets/kubernetes.io/serviceaccount/token"));
    let k8s_host = engine.read_env("KUBERNETES_SERVICE_HOST");

    if std::path::Path::new(&token_path).exists() && k8s_host.is_ok() {
        let host = k8s_host.unwrap_or_default();
        let port = engine.read_env("KUBERNETES_SERVICE_PORT").unwrap_or_else(|_| "443".into());
        let token = tokio::fs::read_to_string(&token_path).await.map_err(|e| {
            SongbirdError::discovery(format!("Failed to read K8s service account token: {e}"))
        })?;

        let api_base = engine
            .read_env("SONGBIRD_K8S_API_BASE_URL")
            .unwrap_or_else(|_| format!("https://{host}:{port}"));
        let url = kubernetes_endpoints_list_url(&api_base, ns, capability);

        let client = songbird_http_client::IpcHttpClient::new()
            .await
            .map_err(|e| SongbirdError::discovery(format!("IPC HTTP client init: {e}")))?;

        // Use POST-style builder to attach Authorization header
        let resp = client
            .post(&url)
            .await
            .header("Authorization", format!("Bearer {}", token.trim()))
            .header("X-HTTP-Method-Override", "GET")
            .send()
            .await
            .map_err(|e| SongbirdError::discovery(format!("Kubernetes API request failed: {e}")))?;

        if resp.is_success() {
            let body: serde_json::Value = resp.json().await.map_err(|e| {
                SongbirdError::discovery(format!("Failed to parse K8s response: {e}"))
            })?;

            let mut discovered = Vec::new();
            if let Some(items) = body.get("items").and_then(|v| v.as_array()) {
                for item in items {
                    extract_k8s_endpoints(item, capability, ns, &mut discovered);
                }
            }

            debug!(
                target: "songbird_config::discovery",
                backend = "kubernetes",
                count = discovered.len(),
                "Kubernetes API discovery complete"
            );
            return Ok(discovered);
        }
    }

    debug!(
        target: "songbird_config::discovery",
        backend = "kubernetes",
        "No Kubernetes in-cluster environment detected; returning empty"
    );
    Ok(Vec::new())
}

/// Extract endpoints from a Kubernetes API `items[]` entry.
fn extract_k8s_endpoints(
    item: &serde_json::Value,
    capability: &str,
    ns: &str,
    out: &mut Vec<DiscoveredService>,
) {
    let Some(subsets) = item.get("subsets").and_then(|v| v.as_array()) else {
        return;
    };
    for subset in subsets {
        let ports: Vec<u16> = subset
            .get("ports")
            .and_then(|v| v.as_array())
            .map(|ps| {
                ps.iter()
                    .filter_map(|p| {
                        p.get("port")
                            .and_then(serde_json::Value::as_u64)
                            .and_then(|v| u16::try_from(v).ok())
                    })
                    .collect()
            })
            .unwrap_or_default();

        let Some(addresses) = subset.get("addresses").and_then(|v| v.as_array()) else {
            continue;
        };
        for addr_obj in addresses {
            let Some(ip) = addr_obj.get("ip").and_then(|v| v.as_str()) else {
                continue;
            };
            for &port in &ports {
                if let Ok(addr) = format!("{ip}:{port}").parse::<SocketAddr>() {
                    let mut metadata = HashMap::new();
                    metadata.insert(String::from("source"), String::from("kubernetes-api"));
                    metadata.insert(String::from("namespace"), ns.to_string());
                    out.push(DiscoveredService {
                        address: addr,
                        capabilities: vec![capability.to_string()],
                        metadata,
                        discovered_at: std::time::SystemTime::now(),
                    });
                }
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;
    use std::net::SocketAddr;

    #[test]
    fn kubernetes_dns_hostname_matches_cluster_pattern() {
        assert_eq!(
            kubernetes_dns_service_hostname("payments", "prod", "cluster.local"),
            "payments.prod.svc.cluster.local"
        );
    }

    #[test]
    fn kubernetes_endpoints_list_url_encodes_selector() {
        assert_eq!(
            kubernetes_endpoints_list_url("https://k8s:443/", "default", "foo"),
            "https://k8s:443/api/v1/namespaces/default/endpoints?labelSelector=songbird/capability=foo"
        );
    }

    #[test]
    fn extract_k8s_endpoints_reads_subsets_addresses_and_ports() {
        let item = serde_json::json!({
            "subsets": [{
                "ports": [{"port": 8080}, {"port": 65535}],
                "addresses": [{"ip": "10.20.30.40"}]
            }]
        });
        let mut out = Vec::new();
        extract_k8s_endpoints(&item, "cap-x", "ns1", &mut out);
        assert_eq!(out.len(), 2);
        let addrs: Vec<SocketAddr> = out.iter().map(|d| d.address).collect();
        assert!(addrs.contains(&"10.20.30.40:8080".parse().unwrap()));
        assert!(addrs.contains(&"10.20.30.40:65535".parse().unwrap()));
        assert!(
            out.iter().all(|d| d.metadata.get("source") == Some(&String::from("kubernetes-api")))
        );
        assert!(out.iter().all(|d| d.metadata.get("namespace") == Some(&String::from("ns1"))));
    }

    #[test]
    fn extract_k8s_endpoints_ignores_missing_subsets_or_ip() {
        let mut out = Vec::new();
        extract_k8s_endpoints(&serde_json::json!({}), "c", "n", &mut out);
        assert!(out.is_empty());

        let item = serde_json::json!({
            "subsets": [{
                "ports": [{"port": 80}],
                "addresses": [{}]
            }]
        });
        extract_k8s_endpoints(&item, "c", "n", &mut out);
        assert!(out.is_empty());
    }
}
