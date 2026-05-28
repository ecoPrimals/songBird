// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Cloudflare DNS API provider for H2-15 DDNS updates.
//!
//! Updates A/AAAA records via the Cloudflare API v4 when the node's public IP
//! changes (detected via STUN). Uses Bearer token authentication.
//!
//! ## Configuration
//!
//! | Env Var                         | Description                    |
//! |---------------------------------|--------------------------------|
//! | `SONGBIRD_DDNS_PROVIDER`        | Must be `"cloudflare"`         |
//! | `SONGBIRD_DDNS_HOSTNAME`        | FQDN to update (e.g. `node.example.com`) |
//! | `SONGBIRD_DDNS_TTL`             | Record TTL (1 = auto)          |
//! | `SONGBIRD_CF_API_TOKEN`         | Cloudflare API Bearer token    |
//! | `SONGBIRD_CF_ZONE_ID`           | Cloudflare Zone ID             |
//!
//! ## HTTP Execution Model
//!
//! The provider builds JSON request descriptions but delegates actual HTTP
//! execution to a caller-provided `HttpExecutor` function. This keeps the
//! STUN crate free of TLS/HTTP-stack deps and allows the orchestrator to
//! plug in whatever client it already uses.

use songbird_types::config::ddns::{DdnsConfig, DdnsError, DdnsProvider, DdnsUpdateResult};
use std::future::Future;
use std::net::IpAddr;
use std::pin::Pin;
use std::sync::Arc;
use tracing::{debug, info};

const CF_API_BASE: &str = "https://api.cloudflare.com/client/v4";

/// HTTP method + URL + headers + body descriptor for the executor.
#[derive(Debug, Clone)]
pub struct HttpReq {
    /// HTTP method (`GET`, `POST`, `PUT`, etc.).
    pub method: &'static str,
    /// Fully-qualified URL.
    pub url: String,
    /// Bearer token for the `Authorization` header.
    pub bearer_token: String,
    /// Optional JSON request body.
    pub body: Option<String>,
}

/// Function signature for HTTP execution.
/// Returns the response body as a string or an error message.
pub type HttpExecutor = Arc<
    dyn Fn(HttpReq) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send>> + Send + Sync,
>;

/// Cloudflare DNS API v4 provider.
#[derive(Clone)]
pub struct CloudflareDdnsProvider {
    api_token: String,
    zone_id: String,
    http: HttpExecutor,
}

impl std::fmt::Debug for CloudflareDdnsProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CloudflareDdnsProvider")
            .field("zone_id", &self.zone_id)
            .field("api_token", &"[redacted]")
            .finish_non_exhaustive()
    }
}

impl CloudflareDdnsProvider {
    /// Create with explicit values and an HTTP executor.
    #[must_use]
    pub fn new(api_token: String, zone_id: String, http: HttpExecutor) -> Self {
        Self {
            api_token,
            zone_id,
            http,
        }
    }

    /// Create from environment variables.
    ///
    /// # Errors
    ///
    /// Returns `DdnsError::ConfigError` if required env vars are missing.
    pub fn from_env(http: HttpExecutor) -> Result<Self, DdnsError> {
        let api_token = songbird_process_env::var("SONGBIRD_CF_API_TOKEN")
            .map_err(|_| DdnsError::ConfigError("SONGBIRD_CF_API_TOKEN not set".to_string()))?;
        let zone_id = songbird_process_env::var("SONGBIRD_CF_ZONE_ID")
            .map_err(|_| DdnsError::ConfigError("SONGBIRD_CF_ZONE_ID not set".to_string()))?;
        Ok(Self::new(api_token, zone_id, http))
    }

    async fn cf_request(&self, req: HttpReq) -> Result<serde_json::Value, DdnsError> {
        let body = (self.http)(req).await.map_err(|e| DdnsError::NetworkError(e))?;

        let parsed: serde_json::Value = serde_json::from_str(&body)
            .map_err(|e| DdnsError::ProviderError(format!("CF JSON parse: {e}")))?;

        if parsed["success"].as_bool() != Some(true) {
            let errors = &parsed["errors"];
            return Err(DdnsError::ProviderError(format!("CF API error: {errors}")));
        }

        Ok(parsed)
    }

    async fn list_records(
        &self,
        hostname: &str,
        record_type: &str,
    ) -> Result<Vec<CfDnsRecord>, DdnsError> {
        let url = format!(
            "{CF_API_BASE}/zones/{}/dns_records?type={record_type}&name={hostname}",
            self.zone_id
        );

        let parsed = self
            .cf_request(HttpReq {
                method: "GET",
                url,
                bearer_token: self.api_token.clone(),
                body: None,
            })
            .await?;

        let records = parsed["result"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|r| {
                        Some(CfDnsRecord {
                            id: r["id"].as_str()?.to_string(),
                            content: r["content"].as_str()?.to_string(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(records)
    }

    async fn upsert_record(
        &self,
        record_id: Option<&str>,
        hostname: &str,
        record_type: &str,
        content: &str,
        ttl: u32,
    ) -> Result<(), DdnsError> {
        let body = serde_json::json!({
            "type": record_type,
            "name": hostname,
            "content": content,
            "ttl": if ttl == 0 { 1 } else { ttl },
        });

        let (method, url) = if let Some(id) = record_id {
            ("PUT", format!("{CF_API_BASE}/zones/{}/dns_records/{id}", self.zone_id))
        } else {
            ("POST", format!("{CF_API_BASE}/zones/{}/dns_records", self.zone_id))
        };

        self.cf_request(HttpReq {
            method,
            url,
            bearer_token: self.api_token.clone(),
            body: Some(body.to_string()),
        })
        .await?;

        Ok(())
    }
}

#[derive(Debug)]
struct CfDnsRecord {
    id: String,
    content: String,
}

impl DdnsProvider for CloudflareDdnsProvider {
    async fn update(
        &self,
        config: &DdnsConfig,
        new_ip: IpAddr,
    ) -> Result<DdnsUpdateResult, DdnsError> {
        let hostname = config
            .hostname
            .as_deref()
            .ok_or_else(|| DdnsError::ConfigError("DDNS hostname not configured".to_string()))?;

        let (record_type, content) = match new_ip {
            IpAddr::V4(v4) => ("A", v4.to_string()),
            IpAddr::V6(v6) => ("AAAA", v6.to_string()),
        };

        debug!("CF DDNS: checking {record_type} record for {hostname}");

        let existing = self.list_records(hostname, record_type).await?;

        if let Some(record) = existing.first() {
            if record.content == content {
                debug!("CF DDNS: {hostname} already points to {content}");
                return Ok(DdnsUpdateResult::Unchanged);
            }
            info!("CF DDNS: updating {hostname} {record_type} → {content}");
            self.upsert_record(Some(&record.id), hostname, record_type, &content, config.ttl)
                .await?;
        } else {
            info!("CF DDNS: creating {hostname} {record_type} → {content}");
            self.upsert_record(None, hostname, record_type, &content, config.ttl).await?;
        }

        Ok(DdnsUpdateResult::Updated {
            new_ip,
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use std::net::Ipv4Addr;

    fn mock_http_ok(response: &str) -> HttpExecutor {
        let resp = response.to_string();
        Arc::new(move |_req| {
            let r = resp.clone();
            Box::pin(async move { Ok(r) })
        })
    }

    fn mock_http_err(msg: &str) -> HttpExecutor {
        let m = msg.to_string();
        Arc::new(move |_req| {
            let e = m.clone();
            Box::pin(async move { Err(e) })
        })
    }

    #[test]
    fn provider_creation() {
        let p = CloudflareDdnsProvider::new(
            "tok".to_string(),
            "zone123".to_string(),
            mock_http_ok("{}"),
        );
        assert_eq!(p.zone_id, "zone123");
    }

    #[test]
    fn provider_debug_redacts_token() {
        let p = CloudflareDdnsProvider::new(
            "super-secret".to_string(),
            "z1".to_string(),
            mock_http_ok("{}"),
        );
        let dbg = format!("{p:?}");
        assert!(dbg.contains("[redacted]"));
        assert!(!dbg.contains("super-secret"));
    }

    #[test]
    fn from_env_missing_token() {
        let result = CloudflareDdnsProvider::from_env(mock_http_ok("{}"));
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn update_creates_new_record() {
        let list_resp = r#"{"success":true,"result":[]}"#;
        let create_resp = r#"{"success":true,"result":{"id":"new"}}"#;
        let call_count = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let cc = call_count.clone();

        let http: HttpExecutor = Arc::new(move |req| {
            let n = cc.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let r = if n == 0 {
                list_resp.to_string()
            } else {
                create_resp.to_string()
            };
            assert!(!req.bearer_token.is_empty());
            Box::pin(async move { Ok(r) })
        });

        let provider = CloudflareDdnsProvider::new("tok".into(), "zone1".into(), http);
        let config = DdnsConfig {
            enabled: true,
            provider: "cloudflare".into(),
            hostname: Some("node.example.com".into()),
            ttl: 60,
            zone: None,
            server: None,
            key_name: None,
        };
        let result = provider
            .update(&config, IpAddr::V4(Ipv4Addr::new(203, 0, 113, 1)))
            .await
            .expect("should succeed");
        assert!(matches!(result, DdnsUpdateResult::Updated { .. }));
        assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn update_returns_unchanged_when_ip_matches() {
        let list_resp = r#"{"success":true,"result":[{"id":"rec1","content":"203.0.113.1"}]}"#;
        let http = mock_http_ok(list_resp);

        let provider = CloudflareDdnsProvider::new("tok".into(), "zone1".into(), http);
        let config = DdnsConfig {
            enabled: true,
            provider: "cloudflare".into(),
            hostname: Some("node.example.com".into()),
            ttl: 60,
            zone: None,
            server: None,
            key_name: None,
        };
        let result = provider
            .update(&config, IpAddr::V4(Ipv4Addr::new(203, 0, 113, 1)))
            .await
            .expect("should succeed");
        assert!(matches!(result, DdnsUpdateResult::Unchanged));
    }

    #[tokio::test]
    async fn update_fails_on_network_error() {
        let http = mock_http_err("connection refused");
        let provider = CloudflareDdnsProvider::new("tok".into(), "zone1".into(), http);
        let config = DdnsConfig {
            enabled: true,
            provider: "cloudflare".into(),
            hostname: Some("node.example.com".into()),
            ttl: 60,
            zone: None,
            server: None,
            key_name: None,
        };
        let err = provider
            .update(&config, IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4)))
            .await
            .expect_err("should fail");
        assert!(matches!(err, DdnsError::NetworkError(_)));
    }

    #[tokio::test]
    async fn update_fails_on_missing_hostname() {
        let http = mock_http_ok("{}");
        let provider = CloudflareDdnsProvider::new("tok".into(), "zone1".into(), http);
        let config = DdnsConfig {
            enabled: true,
            provider: "cloudflare".into(),
            hostname: None,
            ttl: 60,
            zone: None,
            server: None,
            key_name: None,
        };
        let err = provider
            .update(&config, IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4)))
            .await
            .expect_err("should fail");
        assert!(matches!(err, DdnsError::ConfigError(_)));
    }

    #[test]
    fn http_req_debug() {
        let req = HttpReq {
            method: "GET",
            url: "https://example.com".into(),
            bearer_token: "tok".into(),
            body: None,
        };
        assert!(!format!("{req:?}").is_empty());
    }
}
