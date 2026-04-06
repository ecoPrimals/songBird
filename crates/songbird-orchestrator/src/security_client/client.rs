// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Security capability client implementation
//!
//! **MODERNIZED v3.12.3**: Now uses protocol-agnostic `SecurityAdapter`!
//!
//! This module provides the main `SecurityCapabilityClient` struct that handles
//! communication with security providers for trust evaluation and identity attestation.

use anyhow::{Context, Result};
use serde_json::json;
use songbird_http_client::SongbirdHttpClient;
use songbird_types::{LineageId, LineageProof};
use songbird_universal::adapters::SecurityAdapter;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use super::types::{
    ApiResponseWrapper, ConnectionInfo, CurrentLineageInfo, IdentityResponse,
    TrustEvaluationRequest, TrustEvaluationResponse, VerificationResult,
};
use crate::trust::universal_trust_api::{
    IdentityAttestation as UniversalIdentityAttestation, UniversalTrustRequest,
    UniversalTrustResponse,
};

/// Security capability client for trust evaluation
///
/// **MODERNIZED v3.12.3**: Protocol-agnostic! Automatically uses best available protocol:
/// - `tarpc://` → tarpc (PRIMARY - 10-20 μs latency)
/// - `unix://` → JSON-RPC over Unix socket (SECONDARY - 50-100 μs latency)  
/// - `http://` → HTTP (FALLBACK - 500-1000 μs latency)
///
/// Endpoint is discovered at runtime - no hardcoding! Zero configuration needed!
#[derive(Debug)]
pub struct SecurityCapabilityClient {
    /// Protocol-agnostic security adapter (v3.12.3)
    /// Handles tarpc, JSON-RPC, and HTTP automatically
    adapter: SecurityAdapter,

    /// HTTP client for lineage methods (v3.14.2)
    ///
    /// **Status**: Used ONLY for lineage API endpoints which are Phase 1.5 features:
    /// - `evaluate_trust_universal()` - Universal trust API (transitional)
    /// - `get_current_lineage()` - Query our genetic lineage
    /// - `verify_lineage()` - Verify lineage proof cryptographically
    /// - `same_family()` - Check if two lineages share ancestry
    ///
    /// **✅ PURE RUST**: Now uses songbird-http-client (Zero C dependencies!)
    /// **Migration Plan**: These will move to `SecurityAdapter` when security provider Phase 1.5 is complete.
    http_client: Arc<SongbirdHttpClient>,

    /// Optional: Cached identity
    cached_identity: Option<IdentityResponse>,
}

impl SecurityCapabilityClient {
    /// Create from a discovered endpoint
    ///
    /// **MODERNIZED v3.12.3**: Protocol-agnostic! Automatically detects and uses best protocol.
    ///
    /// # Arguments
    ///
    /// * `endpoint` - Security provider URL (discovered at runtime, not hardcoded!)
    ///   - `tarpc://host:port` → Uses tarpc (10-20 μs latency)
    ///   - `unix:///path/to/socket` → Uses JSON-RPC over Unix socket (50-100 μs latency)
    ///   - `http://host` or `https://host` → Uses HTTP (500-1000 μs latency)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use songbird_orchestrator::security_capability_client::SecurityCapabilityClient;
    /// # async fn example() -> anyhow::Result<()> {
    /// // Discover security provider (protocol detected automatically!)
    /// let endpoint = discover_capability("security").await?;
    /// let client = SecurityCapabilityClient::from_endpoint(endpoint)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns error if endpoint URL is invalid or protocol cannot be determined
    pub async fn from_endpoint(endpoint: impl Into<String>) -> Result<Self> {
        let adapter = SecurityAdapter::new(endpoint.into())
            .await
            .context("Failed to create protocol-agnostic security adapter")?;

        // Create Pure Rust HTTP client for lineage methods (Phase 1.5) ✅
        // Uses capability-based discovery for crypto provider (XDG-compliant)
        let crypto_socket = crate::env_config::security_crypto_ipc_socket_from_env(|| {
            if let Ok(runtime_dir) = songbird_process_env::var("XDG_RUNTIME_DIR") {
                format!("{runtime_dir}/biomeos/security.sock")
            } else {
                songbird_types::defaults::paths::security_socket_default_path()
                    .to_string_lossy()
                    .into_owned()
            }
        });

        let http_client = Arc::new(SongbirdHttpClient::new(crypto_socket));

        Ok(Self {
            adapter,
            http_client,
            cached_identity: None,
        })
    }

    /// Parse response gracefully (agnostic to wrapped/unwrapped format)
    ///
    /// **Modern Idiomatic Pattern** (Jan 3, 2026):
    /// - HTTP status codes indicate success/failure (REST standard)
    /// - Try unwrapped format first (idiomatic, clean)
    /// - Fall back to wrapped format (backward compatibility)
    /// - Works with any security provider during transition
    ///
    /// **✅ PURE RUST** (Jan 21, 2026): Now uses songbird-http-client responses
    fn parse_response_body<T>(&self, status: u16, body: &str) -> Result<T>
    where
        T: serde::de::DeserializeOwned + std::fmt::Debug,
    {
        // ✅ IDIOMATIC REST: HTTP status code is source of truth
        if !(200..300).contains(&status) {
            error!("Security provider returned error: {} - {}", status, body);
            anyhow::bail!("Security provider error: {status} - {body}");
        }

        // Try unwrapped format first (modern, idiomatic)
        if let Ok(data) = serde_json::from_str::<T>(body) {
            debug!("Parsed response as unwrapped format (idiomatic REST)");
            return Ok(data);
        }

        // Fall back to wrapped format (backward compatibility)
        if let Ok(wrapped) = serde_json::from_str::<ApiResponseWrapper<T>>(body) {
            if wrapped.success {
                debug!("Parsed response as wrapped format (legacy compatibility)");
                return Ok(wrapped.data);
            }
            anyhow::bail!(
                "Security provider returned success=false: {}",
                wrapped.error.unwrap_or_else(|| "Unknown error".to_string())
            );
        }

        // Neither format worked - show body for debugging
        error!("Failed to parse response in any known format. Body: {}", body);
        anyhow::bail!("Failed to parse security provider response")
    }

    /// Get our identity from security provider
    ///
    /// **MODERNIZED v3.12.3**: Protocol-agnostic! Uses tarpc/JSON-RPC/HTTP automatically.
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn get_identity(&mut self) -> Result<IdentityResponse> {
        // Return cached if available
        if let Some(ref identity) = self.cached_identity {
            debug!("Returning cached identity");
            return Ok(identity.clone());
        }

        // Query security provider using protocol-agnostic adapter
        debug!("Querying security provider identity using protocol-agnostic adapter");

        let universal_identity =
            self.adapter.get_identity().await.context("Failed to connect to security provider")?;

        // Convert from universal format to local format
        let identity = IdentityResponse {
            encryption_tag: universal_identity.encryption_tag,
            capabilities: universal_identity.capabilities,
            family_id: None, // Not in universal format yet
        };

        info!("✅ Retrieved identity from security provider: {}", identity.encryption_tag);

        let result = identity.clone();
        self.cached_identity = Some(identity);
        Ok(result)
    }

    /// Evaluate trust for a discovered peer
    ///
    /// **MODERNIZED v3.12.3**: Protocol-agnostic! Uses tarpc/JSON-RPC/HTTP automatically.
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn evaluate_trust(
        &self,
        request: &TrustEvaluationRequest,
    ) -> Result<TrustEvaluationResponse> {
        debug!("Evaluating trust for peer {} using protocol-agnostic adapter", request.peer_id);

        // Convert to universal format
        let connection_info_map = request.connection_info.as_ref().map(|info| {
            let mut map = HashMap::new();
            map.insert("endpoint".to_string(), info.endpoint.clone());
            map.insert("protocol".to_string(), info.protocol.clone());
            map
        });

        let universal_req = songbird_universal::TrustEvaluationRequest {
            peer_id: request.peer_id.clone(),
            peer_family: request.peer_family.clone(),
            peer_tags: request.peer_tags.clone(),
            connection_info: connection_info_map,
            context: request.context.clone(),
        };

        // Use protocol-agnostic adapter (automatically selects tarpc/JSON-RPC/HTTP)
        let decision = match self.adapter.evaluate_trust(&universal_req).await {
            Ok(universal_resp) => {
                // Convert from universal format to local format
                TrustEvaluationResponse {
                    decision: universal_resp.decision,
                    trust_level: universal_resp.trust_level.name().to_string(),
                    confidence: 0.0,
                    reason: universal_resp.reason,
                    encryption_tag: None,
                    metadata: universal_resp
                        .metadata
                        .unwrap_or_default()
                        .into_iter()
                        .map(|(k, v)| (k, v.to_string()))
                        .collect(),
                }
            }
            Err(e) => {
                warn!(
                    "Security provider trust evaluation failed for peer {}: {}",
                    request.peer_id, e
                );

                // Return reject decision on error (fail-safe)
                TrustEvaluationResponse {
                    decision: "reject".to_string(),
                    trust_level: "none".to_string(),
                    confidence: 0.0,
                    reason: format!("Security provider error: {e}"),
                    encryption_tag: None,
                    metadata: HashMap::new(),
                }
            }
        };

        match decision.decision.as_str() {
            "auto_accept" => {
                info!(
                    "✅ Security provider auto-accepts peer {} ({})",
                    request.peer_id, decision.reason
                );
            }
            "prompt_user" => {
                warn!(
                    "⚠️ Security provider requests user prompt for peer {} ({})",
                    request.peer_id, decision.reason
                );
            }
            "reject" => {
                error!(
                    "❌ Security provider rejects peer {} ({})",
                    request.peer_id, decision.reason
                );
            }
            other => {
                warn!(
                    "Unknown security provider decision '{}' for peer {}, treating as reject",
                    other, request.peer_id
                );
            }
        }

        Ok(decision)
    }

    /// Check if security provider is available
    ///
    /// **MODERNIZED v3.12.3**: Protocol-agnostic! Uses tarpc/JSON-RPC/HTTP automatically.
    pub async fn is_available(&self) -> bool {
        (self.adapter.check_health().await).is_ok()
    }

    /// Get security provider endpoint
    ///
    /// **MODERNIZED v3.12.3**: Returns endpoint from protocol-agnostic adapter.
    #[must_use]
    pub fn endpoint(&self) -> &str {
        self.adapter.endpoint()
    }

    /// Test hook for [`parse_response_body`](Self::parse_response_body) (no network I/O).
    #[cfg(test)]
    pub(crate) fn test_parse_response_body<T>(&self, status: u16, body: &str) -> anyhow::Result<T>
    where
        T: serde::de::DeserializeOwned + std::fmt::Debug,
    {
        self.parse_response_body(status, body)
    }

    /// Convert identity response to universal attestations
    #[must_use]
    pub fn identity_to_attestations(
        identity: &IdentityResponse,
    ) -> Vec<UniversalIdentityAttestation> {
        let mut attestations = Vec::new();

        // Create tag list attestation
        if !identity.encryption_tag.is_empty() {
            let mut data = json!({
                "tags": vec![identity.encryption_tag.clone()]
            });

            // Add family_id if present
            if let Some(ref family_id) = identity.family_id {
                data["family_id"] = json!(family_id);
            }

            attestations.push(UniversalIdentityAttestation {
                provider: Some("security/identity".to_string()),
                format: "tag_list".to_string(),
                data,
            });
        }

        attestations
    }

    /// Convert identity response to discovery attestations
    #[must_use]
    pub fn identity_to_discovery_attestations(
        identity: &IdentityResponse,
    ) -> Vec<songbird_discovery::IdentityAttestation> {
        let mut attestations = Vec::new();

        // Create tag list attestation with family ID
        if !identity.encryption_tag.is_empty() {
            let mut data = json!({
                "tags": vec![identity.encryption_tag.clone()]
            });

            // Add family_id if present
            if let Some(ref family_id) = identity.family_id {
                data["family_id"] = json!(family_id);
            }

            attestations.push(songbird_discovery::IdentityAttestation {
                provider_capability: "security/identity".to_string(),
                format: "tag_list".to_string(),
                data,
            });
        }

        attestations
    }

    /// Evaluate trust using universal API format
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn evaluate_trust_universal(
        &self,
        request: &UniversalTrustRequest,
    ) -> Result<UniversalTrustResponse> {
        let url = format!("{}/api/v1/trust/evaluate", self.adapter.endpoint());
        debug!("Evaluating trust (universal API): {}", url);

        // ✅ PURE RUST: Using songbird-http-client
        let request_json = serde_json::to_value(request)?;
        let http_response = self
            .http_client
            .request("POST", &url, HashMap::new(), Some(request_json))
            .await
            .context("Failed to connect to security provider for trust evaluation")?;

        // ✅ AGNOSTIC: Gracefully handles wrapped or unwrapped format
        let body_str = http_response.body.to_string();
        let trust_response = match self
            .parse_response_body::<UniversalTrustResponse>(http_response.status, &body_str)
        {
            Ok(response) => response,
            Err(e) => {
                warn!("Universal trust evaluation failed: {}. Trying legacy fallback...", e);
                self.evaluate_trust_legacy_fallback(request).await?
            }
        };

        info!(
            "✅ Trust evaluation complete: decision={:?}, confidence={}",
            trust_response.decision, trust_response.confidence
        );

        Ok(trust_response)
    }

    /// Fallback to legacy trust evaluation format
    async fn evaluate_trust_legacy_fallback(
        &self,
        universal_request: &UniversalTrustRequest,
    ) -> Result<UniversalTrustResponse> {
        warn!("Falling back to legacy trust evaluation format");

        // Extract tags from attestations
        let mut tags = Vec::new();
        for attestation in &universal_request.evaluator.attestations {
            if attestation.format == "tag_list"
                && let Some(tag_array) = attestation.data.get("tags").and_then(|t| t.as_array())
            {
                for tag in tag_array {
                    if let Some(tag_str) = tag.as_str() {
                        tags.push(tag_str.to_string());
                    }
                }
            }
        }

        // Build legacy request
        let legacy_request = TrustEvaluationRequest {
            peer_id: universal_request.evaluator.peer_id.clone(),
            peer_family: None,
            peer_tags: tags,
            connection_info: Some(ConnectionInfo {
                endpoint: universal_request.context.endpoint.clone(),
                protocol: "tarpc".to_string(),
            }),
            context: None,
        };

        // Call legacy API
        let legacy_response = self.evaluate_trust(&legacy_request).await?;

        // Convert to universal format
        use crate::trust::universal_trust_api::TrustDecision as UniversalTrustDecision;

        let decision = match legacy_response.decision.as_str() {
            "auto_accept" => UniversalTrustDecision::AutoAccept,
            "prompt_user" => UniversalTrustDecision::PromptUser,
            _ => UniversalTrustDecision::Reject,
        };

        Ok(UniversalTrustResponse {
            response_format: "universal_trust_v1".to_string(),
            decision,
            confidence: legacy_response.confidence,
            reason: legacy_response.reason.clone(),
            reason_code: legacy_response.reason.clone(),
            metadata: legacy_response.metadata.iter().map(|(k, v)| (k.clone(), json!(v))).collect(),
            expires_at: None,
            custom: HashMap::new(),
        })
    }

    /// Backward compatibility: alias for `from_endpoint`
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    #[deprecated(note = "Use from_endpoint instead for clarity")]
    pub async fn new(endpoint: impl Into<String>) -> Result<Self> {
        Self::from_endpoint(endpoint).await
    }

    /// Get our current genetic lineage from security provider
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn get_current_lineage(&self) -> Result<Option<CurrentLineageInfo>> {
        let url = format!("{}/api/v1/lineage/current", self.adapter.endpoint());
        debug!("Querying security provider for current lineage: {}", url);

        // ✅ PURE RUST: Using songbird-http-client
        let response = self
            .http_client
            .request("GET", &url, HashMap::new(), None)
            .await
            .context("Failed to connect to security provider for lineage query")?;

        // If not found, return None (no lineage configured)
        if response.status == 404 {
            return Ok(None);
        }

        // Parse response using our agnostic parser
        let body_str = response.body.to_string();
        let lineage_info =
            self.parse_response_body::<CurrentLineageInfo>(response.status, &body_str)?;

        info!("✅ Retrieved current lineage from security provider: {}", lineage_info.lineage_id);
        Ok(Some(lineage_info))
    }

    /// Verify a peer's lineage proof
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn verify_lineage(&self, proof: &LineageProof) -> Result<VerificationResult> {
        let url = format!("{}/api/v1/lineage/verify", self.adapter.endpoint());
        debug!("Verifying lineage proof with security provider: {}", url);

        // ✅ PURE RUST: Using songbird-http-client
        let proof_json = serde_json::to_value(proof)?;
        let response = self
            .http_client
            .request("POST", &url, HashMap::new(), Some(proof_json))
            .await
            .context("Failed to connect to security provider for lineage verification")?;

        // Return invalid verification result on error
        if response.status < 200 || response.status >= 300 {
            error!(
                "Security provider lineage verification failed: {} - {}",
                response.status, response.body
            );
            let invalid_lineage = LineageId::new("error-invalid".to_string());
            return Ok(VerificationResult {
                valid: false,
                same_genesis: false,
                lineage_id: invalid_lineage,
                messages: vec![format!("Security provider error: {}", response.status)],
            });
        }

        let body_str = response.body.to_string();
        let result = self
            .parse_response_body::<VerificationResult>(response.status, &body_str)
            .context("Failed to parse security provider verification response")?;

        if result.valid {
            info!("✅ Lineage proof verified by security provider");
        } else {
            warn!("❌ Lineage proof rejected by security provider: {:?}", result.messages);
        }

        Ok(result)
    }

    /// Check if two lineages are from the same genetic family
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn same_family(&self, lineage_a: &LineageId, lineage_b: &LineageId) -> Result<bool> {
        let url = format!("{}/api/v1/lineage/same_family", self.adapter.endpoint());
        debug!("Checking if lineages are from same family: {} vs {}", lineage_a, lineage_b);

        #[derive(serde::Serialize)]
        struct SameFamilyRequest {
            lineage_a: String,
            lineage_b: String,
        }

        #[derive(Debug, serde::Deserialize)]
        struct SameFamilyResponse {
            same_family: bool,
            confidence: f64,
        }

        let request = SameFamilyRequest {
            lineage_a: lineage_a.to_string(),
            lineage_b: lineage_b.to_string(),
        };

        // ✅ PURE RUST: Using songbird-http-client
        let request_json = serde_json::to_value(&request)?;
        let response = self
            .http_client
            .request("POST", &url, HashMap::new(), Some(request_json))
            .await
            .context("Failed to connect to security provider for family check")?;

        // Conservative: assume different families on error
        if response.status < 200 || response.status >= 300 {
            warn!("Security provider family check failed: {} - {}", response.status, response.body);
            return Ok(false);
        }

        let body_str = response.body.to_string();
        let result = self
            .parse_response_body::<SameFamilyResponse>(response.status, &body_str)
            .context("Failed to parse security provider family check response")?;

        if result.same_family {
            info!(
                "✅ Lineages are from same genetic family (confidence: {:.2})",
                result.confidence
            );
        } else {
            debug!("Different genetic families");
        }

        Ok(result.same_family)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[derive(Debug, serde::Deserialize)]
    struct SimpleOk {
        ok: bool,
    }

    #[tokio::test]
    async fn test_client_creation() {
        // No hardcoded endpoint! Discovered at runtime
        let client =
            SecurityCapabilityClient::from_endpoint("http://discovered-security-provider").await;
        assert_eq!(client.unwrap().endpoint(), "http://discovered-security-provider");
    }

    #[tokio::test]
    async fn parse_response_body_rejects_non_success_http() {
        let client = SecurityCapabilityClient::from_endpoint("http://discovered-security-provider")
            .await
            .unwrap();
        let err =
            client.test_parse_response_body::<SimpleOk>(500, "{}").expect_err("should fail on 5xx");
        assert!(err.to_string().contains("500") || err.to_string().contains("Security provider"));
    }

    #[tokio::test]
    async fn parse_response_body_unwrapped_json() {
        let client = SecurityCapabilityClient::from_endpoint("http://discovered-security-provider")
            .await
            .unwrap();
        let v: SimpleOk =
            client.test_parse_response_body(200, r#"{"ok":true}"#).expect("unwrapped");
        assert!(v.ok);
    }

    #[tokio::test]
    async fn parse_response_body_rejects_garbage() {
        let client = SecurityCapabilityClient::from_endpoint("http://discovered-security-provider")
            .await
            .unwrap();
        client.test_parse_response_body::<SimpleOk>(200, "not-json {{{").expect_err("invalid json");
    }
}
