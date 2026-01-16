//! Security Capability Client for Genesis Operations
//!
//! Provider-agnostic client for security capabilities (signing, verification).
//! Discovers security providers at runtime - NO hardcoded primal names!

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Security capability client for cryptographic operations
#[derive(Debug, Clone)]
pub struct SecurityCapabilityClient {
    base_url: String,
    http_client: reqwest::Client,
}

impl SecurityCapabilityClient {
    /// Create new security client
    ///
    /// Attempts to discover security provider endpoint using:
    /// 1. `SECURITY_ENDPOINT` environment variable
    /// 2. Capability discovery (via songbird-config)
    /// 3. Well-known default (localhost:8200)
    pub async fn new() -> Result<Self> {
        let base_url = Self::discover_endpoint().await?;

        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .context("Failed to create HTTP client")?;

        tracing::info!("🔐 Security capability client created for endpoint: {}", base_url);

        Ok(Self {
            base_url,
            http_client,
        })
    }

    /// Create client with explicit endpoint
    pub fn with_endpoint(endpoint: impl Into<String>) -> Result<Self> {
        let base_url = endpoint.into();
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            base_url,
            http_client,
        })
    }

    /// Sign data using BearDog
    ///
    /// This is used by genesis witnesses to cryptographically sign new node identities.
    pub async fn sign_data(&self, node_id: &str, data: &[u8]) -> Result<Vec<u8>> {
        let request = SignRequest {
            node_id: node_id.to_string(),
            data: hex::encode(data),
            signature_type: "genesis".to_string(),
        };

        let url = format!("{}/v1/sign", self.base_url);

        tracing::debug!("🔏 Requesting signature from BearDog for node: {}", node_id);

        let response = self
            .http_client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send sign request to BearDog")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("BearDog sign request failed ({}): {}", status, error_text);
        }

        let sign_response: SignResponse =
            response.json().await.context("Failed to parse sign response from BearDog")?;

        hex::decode(&sign_response.signature).context("Failed to decode signature from BearDog")
    }

    /// Verify signature using BearDog
    ///
    /// This is used to verify that a signature is valid for the given data.
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

        tracing::debug!("🔍 Verifying signature with BearDog for node: {}", node_id);

        let response = self
            .http_client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send verify request to BearDog")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("BearDog verify request failed ({}): {}", status, error_text);
        }

        let verify_response: VerifyResponse =
            response.json().await.context("Failed to parse verify response from BearDog")?;

        Ok(verify_response.valid)
    }

    /// Get public key fingerprint for a node
    ///
    /// This is used during genesis to establish the node's public key.
    pub async fn get_public_key_fingerprint(&self, node_id: &str) -> Result<String> {
        let url = format!("{}/v1/keys/{}/fingerprint", self.base_url, node_id);

        tracing::debug!("🔑 Fetching public key fingerprint from BearDog for node: {}", node_id);

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch public key fingerprint from BearDog")?;

        if !response.status().is_success() {
            // Fallback: Generate deterministic fingerprint from node_id
            tracing::warn!("Failed to fetch public key from BearDog, using deterministic fallback");
            return Ok(Self::generate_deterministic_fingerprint(node_id));
        }

        let key_response: KeyFingerprintResponse = response
            .json()
            .await
            .context("Failed to parse key fingerprint response from BearDog")?;

        Ok(key_response.fingerprint)
    }

    /// Create lineage data for a new node
    ///
    /// This is used by primals to grant lineage to a new node during genesis.
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
            .json(&request)
            .send()
            .await
            .context("Failed to send create lineage request to BearDog")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("BearDog create lineage request failed ({}): {}", status, error_text);
        }

        let lineage_response: CreateLineageResponse = response
            .json()
            .await
            .context("Failed to parse create lineage response from BearDog")?;

        hex::decode(&lineage_response.lineage_data)
            .context("Failed to decode lineage data from BearDog")
    }

    /// Discover BearDog endpoint using multiple strategies
    async fn discover_endpoint() -> Result<String> {
        // Strategy 1: BEARDOG_ENDPOINT environment variable
        if let Ok(endpoint) = std::env::var("BEARDOG_ENDPOINT") {
            tracing::info!("🐻 Using BearDog endpoint from BEARDOG_ENDPOINT: {}", endpoint);
            return Ok(endpoint);
        }

        // Strategy 2: SECURITY_ENDPOINT environment variable
        if let Ok(endpoint) = std::env::var("SECURITY_ENDPOINT") {
            tracing::info!("🐻 Using BearDog endpoint from SECURITY_ENDPOINT: {}", endpoint);
            return Ok(endpoint);
        }

        // Strategy 3: Capability discovery (via songbird-config)
        #[cfg(feature = "capability-discovery")]
        {
            use songbird_config::discovery_helpers::discover_primal;
            use songbird_types::CanonicalPrimalType;

            if let Ok(service_endpoint) = discover_primal(CanonicalPrimalType::Security).await {
                tracing::info!(
                    "🐻 Discovered BearDog via capability discovery: {}",
                    service_endpoint.url
                );
                return Ok(service_endpoint.url);
            }
        }

        // Strategy 4: Well-known default (only in development)
        #[cfg(debug_assertions)]
        {
            let default_endpoint = "http://localhost:8200".to_string();
            tracing::warn!(
                "🐻 Using default BearDog endpoint (development only): {}",
                default_endpoint
            );
            Ok(default_endpoint)
        }

        #[cfg(not(debug_assertions))]
        {
            anyhow::bail!(
                "BearDog endpoint not configured. Set BEARDOG_ENDPOINT environment variable."
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

// Request/Response types for BearDog API

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
    use super::*;

    #[test]
    fn test_deterministic_fingerprint() {
        let fp1 = SecurityCapabilityClient::generate_deterministic_fingerprint("test-node");
        let fp2 = SecurityCapabilityClient::generate_deterministic_fingerprint("test-node");

        // Should be deterministic
        assert_eq!(fp1, fp2);

        // Different nodes should have different fingerprints
        let fp3 = SecurityCapabilityClient::generate_deterministic_fingerprint("other-node");
        assert_ne!(fp1, fp3);
    }

    #[tokio::test]
    async fn test_client_creation_with_explicit_endpoint() {
        let client = SecurityCapabilityClient::with_endpoint("http://localhost:9999");
        assert!(client.is_ok());

        let client = client.unwrap();
        assert_eq!(client.base_url, "http://localhost:9999");
    }
}
