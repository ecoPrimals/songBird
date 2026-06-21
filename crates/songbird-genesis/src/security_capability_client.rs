// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Security Capability Client for Genesis Operations
//!
//! Provider-agnostic client for security capabilities (signing, verification).
//! Discovers security providers at runtime - NO hardcoded primal names!

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use songbird_http_client::IpcHttpClient;

/// Security capability client for cryptographic operations
#[derive(Debug, Clone)]
pub struct SecurityCapabilityClient {
    base_url: String,
    http_client: IpcHttpClient,
}

impl SecurityCapabilityClient {
    /// Create new security client
    ///
    /// Attempts to discover security provider endpoint using:
    /// 1. `SECURITY_ENDPOINT` (capability domain)
    /// 2. `SECURITY_PROVIDER_ENDPOINT`
    /// 3. `BEARDOG_ENDPOINT` (deprecated legacy; logs a migration warning)
    /// 4. Capability discovery (via songbird-config)
    /// 5. Well-known default (localhost:8200, debug builds only)
    ///
    /// # Errors
    ///
    /// Returns an error if endpoint discovery fails or HTTP client creation fails.
    pub async fn new() -> Result<Self> {
        let base_url = Self::discover_endpoint().await?;

        let http_client = IpcHttpClient::new().await.context("Failed to create HTTP client")?;

        tracing::info!("🔐 Security capability client created for endpoint: {}", base_url);

        Ok(Self {
            base_url,
            http_client,
        })
    }

    /// Create client with explicit endpoint
    ///
    /// # Errors
    ///
    /// Returns an error if HTTP client creation fails.
    pub async fn with_endpoint(endpoint: impl Into<String>) -> Result<Self> {
        let base_url = endpoint.into();
        let http_client = IpcHttpClient::new().await.context("Failed to create HTTP client")?;

        Ok(Self {
            base_url,
            http_client,
        })
    }

    /// Sign data using `security provider`
    ///
    /// This is used by genesis witnesses to cryptographically sign new node identities.
    ///
    /// # Errors
    ///
    /// Returns an error if the sign request fails or response parsing fails.
    pub async fn sign_data(&self, node_id: &str, data: &[u8]) -> Result<Vec<u8>> {
        let request = SignRequest {
            node_id: node_id.to_string(),
            data: hex::encode(data),
            signature_type: String::from("genesis"),
        };

        let url = format!("{}/v1/sign", self.base_url);

        tracing::debug!("🔏 Requesting signature from security provider for node: {}", node_id);

        let response = self
            .http_client
            .post(&url)
            .await
            .json(&request)?
            .send()
            .await
            .context("Failed to send sign request to security provider")?;

        if !response.is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("security provider sign request failed ({status}): {error_text}");
        }

        let sign_response: SignResponse = response
            .json()
            .await
            .context("Failed to parse sign response from security provider")?;

        hex::decode(&sign_response.signature)
            .context("Failed to decode signature from security provider")
    }

    /// Verify signature using `security provider`
    ///
    /// This is used to verify that a signature is valid for the given data.
    ///
    /// # Errors
    ///
    /// Returns an error if the verify request fails or response parsing fails.
    pub async fn verify_signature(
        &self,
        node_id: &str,
        data: &[u8],
        signature: &[u8],
    ) -> Result<bool> {
        let request = VerifyRequest {
            node_id: node_id.to_string(),
            data: hex::encode(data),
            signature: hex::encode(signature),
        };

        let url = format!("{}/v1/verify", self.base_url);

        tracing::debug!("🔍 Verifying signature with security provider for node: {}", node_id);

        let response = self
            .http_client
            .post(&url)
            .await
            .json(&request)?
            .send()
            .await
            .context("Failed to send verify request to security provider")?;

        if !response.is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("security provider verify request failed ({status}): {error_text}");
        }

        let verify_response: VerifyResponse = response
            .json()
            .await
            .context("Failed to parse verify response from security provider")?;

        Ok(verify_response.valid)
    }

    /// Get public key fingerprint for a node
    ///
    /// This is used during genesis to establish the node's public key.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails (falls back to deterministic fingerprint on 4xx/5xx).
    pub async fn get_public_key_fingerprint(&self, node_id: &str) -> Result<String> {
        let url = format!("{}/v1/keys/{}/fingerprint", self.base_url, node_id);

        tracing::debug!(
            "🔑 Fetching public key fingerprint from security provider for node: {}",
            node_id
        );

        let response = self
            .http_client
            .get(&url)
            .await
            .context("Failed to fetch public key fingerprint from security provider")?;

        if !response.is_success() {
            // Fallback: Generate deterministic fingerprint from node_id
            tracing::warn!(
                "Failed to fetch public key from security provider, using deterministic fallback"
            );
            return Ok(Self::generate_deterministic_fingerprint(node_id));
        }

        let key_response: KeyFingerprintResponse = response
            .json()
            .await
            .context("Failed to parse key fingerprint response from security provider")?;

        Ok(key_response.fingerprint)
    }

    /// Create lineage data for a new node
    ///
    /// This is used by primals to grant lineage to a new node during genesis.
    ///
    /// # Errors
    ///
    /// Returns an error if the create lineage request fails or response parsing fails.
    pub async fn create_lineage(
        &self,
        primal_name: &str,
        parent_node_id: &str,
        child_node_id: &str,
    ) -> Result<Vec<u8>> {
        let request = CreateLineageRequest {
            primal_name: primal_name.to_string(),
            parent_node_id: parent_node_id.to_string(),
            child_node_id: child_node_id.to_string(),
        };

        let url = format!("{}/v1/lineage/create", self.base_url);

        tracing::debug!(
            "🌳 Creating lineage from {} to {} via primal {}",
            parent_node_id,
            child_node_id,
            primal_name
        );

        let response = self
            .http_client
            .post(&url)
            .await
            .json(&request)?
            .send()
            .await
            .context("Failed to send create lineage request to security provider")?;

        if !response.is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!(
                "security provider create lineage request failed ({status}): {error_text}"
            );
        }

        let lineage_response: CreateLineageResponse = response
            .json()
            .await
            .context("Failed to parse create lineage response from security provider")?;

        hex::decode(&lineage_response.lineage_data)
            .context("Failed to decode lineage data from security provider")
    }

    /// Discover `security provider` endpoint using multiple strategies
    async fn discover_endpoint() -> Result<String> {
        // Strategy 1: SECURITY_ENDPOINT (capability domain — preferred)
        if let Ok(endpoint) = songbird_process_env::var("SECURITY_ENDPOINT") {
            tracing::info!(
                "🐻 Using security provider endpoint from SECURITY_ENDPOINT: {}",
                endpoint
            );
            return Ok(endpoint);
        }

        // Strategy 2: SECURITY_PROVIDER_ENDPOINT
        if let Ok(endpoint) = songbird_process_env::var("SECURITY_PROVIDER_ENDPOINT") {
            tracing::info!(
                "🔐 Using security provider endpoint from SECURITY_PROVIDER_ENDPOINT: {}",
                endpoint
            );
            return Ok(endpoint);
        }

        // Strategy 3: BEARDOG_ENDPOINT (deprecated primal-specific name)
        if let Ok(endpoint) = songbird_process_env::var("BEARDOG_ENDPOINT") {
            tracing::warn!(
                "BEARDOG_ENDPOINT is deprecated — migrate to SECURITY_ENDPOINT or SECURITY_PROVIDER_ENDPOINT"
            );
            return Ok(endpoint);
        }

        // Strategy 4: Capability discovery (via songbird-config)
        #[cfg(feature = "capability-discovery")]
        {
            use songbird_config::discovery_helpers::discover_primal;
            use songbird_types::CanonicalPrimalType;

            if let Ok(service_endpoint) = discover_primal(CanonicalPrimalType::Security).await {
                tracing::info!(
                    "🔐 Discovered security provider via capability discovery: {}",
                    service_endpoint.url
                );
                return Ok(service_endpoint.url);
            }
        }

        // Strategy 5: Well-known default (only in development)
        #[cfg(debug_assertions)]
        {
            let default_endpoint = format!("http://{}:8200", songbird_types::constants::LOCALHOST);
            tracing::warn!(
                "🔓 Using default security provider endpoint (development only): {}",
                default_endpoint
            );
            Ok(default_endpoint)
        }

        #[cfg(not(debug_assertions))]
        {
            anyhow::bail!(
                "security provider endpoint not configured. Set SECURITY_ENDPOINT (or SECURITY_PROVIDER_ENDPOINT), or enable capability discovery."
            );
        }
    }

    /// Generate deterministic fingerprint from node ID (fallback)
    fn generate_deterministic_fingerprint(node_id: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(node_id.as_bytes());
        hasher.update(b"songbird-genesis-fallback-v1");
        hex::encode(hasher.finalize())
    }
}

// Request/Response types for security provider API

#[derive(Debug, Serialize)]
struct SignRequest {
    node_id: String,
    data: String, // hex-encoded
    signature_type: String,
}

#[derive(Debug, Deserialize)]
struct SignResponse {
    signature: String, // hex-encoded
}

#[derive(Debug, Serialize)]
struct VerifyRequest {
    node_id: String,
    data: String,      // hex-encoded
    signature: String, // hex-encoded
}

#[derive(Debug, Deserialize)]
struct VerifyResponse {
    valid: bool,
}

#[derive(Debug, Deserialize)]
struct KeyFingerprintResponse {
    fingerprint: String,
}

#[derive(Debug, Serialize)]
struct CreateLineageRequest {
    primal_name: String,
    parent_node_id: String,
    child_node_id: String,
}

#[derive(Debug, Deserialize)]
struct CreateLineageResponse {
    lineage_data: String, // hex-encoded
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use std::sync::Mutex;

    use super::*;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_deterministic_fingerprint() {
        let fp1 = SecurityCapabilityClient::generate_deterministic_fingerprint("test-node");
        let fp2 = SecurityCapabilityClient::generate_deterministic_fingerprint("test-node");

        // Should be deterministic
        assert_eq!(fp1, fp2);

        // Different nodes should have different fingerprints
        let fp3 = SecurityCapabilityClient::generate_deterministic_fingerprint("other-node");
        assert_ne!(fp1, fp3);

        let empty = SecurityCapabilityClient::generate_deterministic_fingerprint("");
        assert_eq!(empty.len(), 64, "SHA256 hex is 64 chars");
    }

    #[tokio::test]
    async fn test_client_creation_with_explicit_endpoint() {
        let client = SecurityCapabilityClient::with_endpoint("http://localhost:9999").await;
        assert!(client.is_ok());

        let client = client.expect("Failed to create client");
        assert_eq!(client.base_url, "http://localhost:9999");
    }

    #[tokio::test(start_paused = true)]
    async fn with_endpoint_roundtrip_uses_tokio_paused_clock_compat() {
        let fut = SecurityCapabilityClient::with_endpoint("http://127.0.0.1:7");
        tokio::time::advance(std::time::Duration::from_millis(1)).await;
        assert!(
            fut.await.is_ok(),
            "explicit endpoint client should construct under paused runtime"
        );
    }

    #[test]
    fn deterministic_fingerprint_long_node_id_stays_sha256_hex() {
        let fp = SecurityCapabilityClient::generate_deterministic_fingerprint(&"n".repeat(10_000));
        assert_eq!(fp.len(), 64);
        assert!(fp.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn deterministic_fingerprint_whitespace_node_is_distinct() {
        let space = SecurityCapabilityClient::generate_deterministic_fingerprint(" ");
        let empty = SecurityCapabilityClient::generate_deterministic_fingerprint("");
        assert_ne!(space, empty);
    }

    #[tokio::test]
    async fn sign_data_fails_against_unreachable_endpoint() {
        let client =
            SecurityCapabilityClient::with_endpoint("http://127.0.0.1:9").await.expect("client");
        let err = client
            .sign_data("node-a", b"payload")
            .await
            .expect_err("sign should fail without IPC backend");
        let msg = err.to_string();
        assert!(
            msg.contains("sign") || msg.contains("Failed") || msg.contains("security provider"),
            "unexpected error message: {msg}"
        );
    }

    #[tokio::test]
    async fn verify_signature_fails_against_unreachable_endpoint() {
        let client =
            SecurityCapabilityClient::with_endpoint("http://127.0.0.1:9").await.expect("client");
        let err = client
            .verify_signature("node-a", b"payload", b"sig-bytes")
            .await
            .expect_err("verify should fail without IPC backend");
        assert!(!err.to_string().is_empty());
    }

    #[tokio::test]
    async fn create_lineage_fails_against_unreachable_endpoint() {
        let client =
            SecurityCapabilityClient::with_endpoint("http://127.0.0.1:9").await.expect("client");
        let err = client
            .create_lineage("songbird", "parent", "child")
            .await
            .expect_err("create_lineage should fail without IPC backend");
        let msg = err.to_string();
        assert!(
            msg.contains("lineage") || msg.contains("Failed") || msg.contains("security provider"),
            "unexpected error message: {msg}"
        );
    }

    #[tokio::test]
    async fn get_public_key_fingerprint_errors_on_network_failure() {
        let client =
            SecurityCapabilityClient::with_endpoint("http://127.0.0.1:9").await.expect("client");
        let err = client
            .get_public_key_fingerprint("node-z")
            .await
            .expect_err("network failure should not silently fallback");
        assert!(!err.to_string().is_empty());
    }

    #[tokio::test]
    async fn sign_data_empty_payload_still_builds_rpc() {
        let client =
            SecurityCapabilityClient::with_endpoint("http://127.0.0.1:9").await.expect("client");
        assert!(
            client.sign_data("empty-node", &[]).await.is_err(),
            "empty payload should still attempt POST and fail at transport"
        );
    }

    #[tokio::test]
    async fn with_endpoint_trailing_slash_preserved() {
        let client = SecurityCapabilityClient::with_endpoint("http://localhost:8200/")
            .await
            .expect("client");
        assert_eq!(client.base_url, "http://localhost:8200/");
    }

    #[tokio::test]
    async fn client_clone_preserves_endpoint() {
        let client = SecurityCapabilityClient::with_endpoint("http://clone-test.local:8200")
            .await
            .expect("client");
        let cloned = client.clone();
        assert_eq!(cloned.base_url, client.base_url);
    }

    #[tokio::test]
    #[expect(
        clippy::await_holding_lock,
        reason = "env test serialization requires lock held across await"
    )]
    async fn new_uses_security_endpoint_env_when_set() {
        use songbird_process_env::ScopedEnv;
        let _lock = ENV_LOCK.lock().unwrap();
        let _env = ScopedEnv::new("SECURITY_ENDPOINT", "http://env-test.local:9200");
        let client = SecurityCapabilityClient::new().await.expect("new with SECURITY_ENDPOINT");
        assert_eq!(client.base_url, "http://env-test.local:9200");
    }

    #[tokio::test]
    #[expect(
        clippy::await_holding_lock,
        reason = "env test serialization requires lock held across await"
    )]
    async fn new_prefers_security_endpoint_over_legacy_beardog() {
        use songbird_process_env::ScopedEnv;
        let _lock = ENV_LOCK.lock().unwrap();
        let _sec = ScopedEnv::new("SECURITY_ENDPOINT", "http://preferred/");
        let _legacy = ScopedEnv::new("BEARDOG_ENDPOINT", "http://legacy/");
        let client = SecurityCapabilityClient::new().await.expect("new with env precedence");
        assert_eq!(client.base_url, "http://preferred/");
    }
}
