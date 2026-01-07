//! Security capability client for cryptographic trust evaluation
//!
//! **MODERNIZED v3.12.3**: Now uses protocol-agnostic SecurityAdapter!
//!
//! This module provides a protocol-agnostic API for discovering and using security capabilities
//! without hardcoding specific primal names. Works with ANY primal that provides
//! security capabilities (identity, encryption, trust-evaluation) via ANY protocol.
//!
//! ## Modern Architecture (v3.12.3)
//!
//! - **Security Provider**: ANY primal offering security capabilities (discovered at runtime)
//! - **Protocol Detection**: Automatic (tarpc → JSON-RPC → HTTP)
//! - **Performance**: 10-50x faster with tarpc/JSON-RPC
//! - **Deployment**: Fractal (same code, any protocol)
//!
//! ## Usage (Protocol-Agnostic)
//!
//! ```rust,no_run
//! use songbird_orchestrator::security_capability_client::{SecurityCapabilityClient, TrustEvaluationRequest};
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Discover security provider at runtime (NO hardcoded endpoint!)
//! let endpoint = discover_capability("security").await?;
//! let client = SecurityCapabilityClient::from_endpoint(endpoint);
//!
//! // Get our identity
//! let identity = client.get_identity().await?;
//! println!("Our tag: {}", identity.encryption_tag);
//!
//! // Evaluate peer trust
//! let request = TrustEvaluationRequest {
//!     peer_id: "tower2".to_string(),
//!     peer_family: Some("a3f2".to_string()),
//!     peer_tags: vec!["beardog:family:a3f2".to_string()],
//!     connection_info: None,
//!     context: None,
//! };
//!
//! let decision = client.evaluate_trust(&request).await?;
//! match decision.decision.as_str() {
//!     "auto_accept" => println!("✅ Auto-accepting peer"),
//!     "prompt_user" => println!("⚠️ Prompting user for consent"),
//!     "reject" => println!("❌ Rejecting peer"),
//!     _ => println!("Unknown decision"),
//! }
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use songbird_types::{LineageId, LineageProof};
use songbird_universal::adapters::SecurityAdapter;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, error, info, warn};

use crate::trust::universal_trust_api::{
    IdentityAttestation as UniversalIdentityAttestation,
    UniversalTrustRequest,
    UniversalTrustResponse,
};

/// Wrapper for potentially wrapped API responses (Agnostic Pattern - Jan 3, 2026)
///
/// Some security providers wrap their responses in `{"success": true, "data": {...}}`.
/// This allows graceful handling of both wrapped and unwrapped formats during transition.
#[derive(Debug, Clone, Deserialize)]
struct ApiResponseWrapper<T> {
    pub success: bool,
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Security capability client for trust evaluation
///
/// **MODERNIZED v3.12.3**: Protocol-agnostic! Automatically uses best available protocol:
/// - `tarpc://` → tarpc (PRIMARY - 10-20 μs latency)
/// - `unix://` → JSON-RPC over Unix socket (SECONDARY - 50-100 μs latency)  
/// - `http://` → HTTP (FALLBACK - 500-1000 μs latency)
///
/// Endpoint is discovered at runtime - no hardcoding! Zero configuration needed!
#[derive(Debug, Clone)]
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
    /// **Migration Plan**: These will move to SecurityAdapter when security provider Phase 1.5 is complete.
    /// Until then, HTTP is acceptable as these are specialized genetic lineage operations.
    http_client: Client,
    
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
    /// ```rust,no_run
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
    pub fn from_endpoint(endpoint: impl Into<String>) -> Result<Self> {
        let adapter = SecurityAdapter::new(endpoint.into())
            .context("Failed to create protocol-agnostic security adapter")?;
        
        // Create HTTP client for lineage methods (Phase 1.5)
        let http_client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_default();
        
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
    async fn parse_response<T>(&self, response: reqwest::Response) -> Result<T>
    where
        T: serde::de::DeserializeOwned + std::fmt::Debug,
    {
        // ✅ IDIOMATIC REST: HTTP status code is source of truth
        let status = response.status();
        
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            error!("Security provider returned error: {} - {}", status, body);
            anyhow::bail!("Security provider error: {} - {}", status, body);
        }
        
        // Get response body as text first (so we can try multiple parse attempts)
        let body_text = response.text().await
            .context("Failed to read response body")?;
        
        // Try unwrapped format first (modern, idiomatic)
        if let Ok(data) = serde_json::from_str::<T>(&body_text) {
            debug!("Parsed response as unwrapped format (idiomatic REST)");
            return Ok(data);
        }
        
        // Fall back to wrapped format (backward compatibility)
        if let Ok(wrapped) = serde_json::from_str::<ApiResponseWrapper<T>>(&body_text) {
            if wrapped.success {
                debug!("Parsed response as wrapped format (legacy compatibility)");
                return Ok(wrapped.data);
            } else {
                anyhow::bail!(
                    "Security provider returned success=false: {}",
                    wrapped.error.unwrap_or_else(|| "Unknown error".to_string())
                );
            }
        }
        
        // Neither format worked - show body for debugging
        error!("Failed to parse response in any known format. Body: {}", body_text);
        anyhow::bail!("Failed to parse security provider response")
    }

    /// Get our identity from security provider
    ///
    /// Returns our encryption tag and capabilities.
    /// Results are cached to avoid repeated queries.
    ///
    /// **Agnostic Pattern**: Works with wrapped or unwrapped responses.
    ///
    /// # Errors
    ///
    /// Returns error if security provider is unreachable or returns invalid response.
    /// Get our identity from security provider
    ///
    /// **MODERNIZED v3.12.3**: Protocol-agnostic! Uses tarpc/JSON-RPC/HTTP automatically.
    pub async fn get_identity(&mut self) -> Result<IdentityResponse> {
        // Return cached if available
        if let Some(ref identity) = self.cached_identity {
            debug!("Returning cached identity");
            return Ok(identity.clone());
        }

        // Query security provider using protocol-agnostic adapter
        debug!("Querying security provider identity using protocol-agnostic adapter");

        let universal_identity = self.adapter.get_identity()
            .await
            .context("Failed to connect to security provider")?;

        // Convert from universal format to local format
        let identity = IdentityResponse {
            encryption_tag: universal_identity.encryption_tag,
            capabilities: universal_identity.capabilities,
            family_id: None, // Not in universal format yet
        };

        info!("✅ Retrieved identity from security provider: {}", identity.encryption_tag);
        
        // Cache for future use
        self.cached_identity = Some(identity.clone());
        
        Ok(identity)
    }

    /// Evaluate trust for a discovered peer
    ///
    /// **MODERNIZED v3.12.3**: Protocol-agnostic! Uses tarpc/JSON-RPC/HTTP automatically.
    ///
    /// Asks security provider: "Should I trust this peer?"
    /// Provider responds with auto_accept, prompt_user, or reject.
    ///
    /// # Arguments
    ///
    /// * `request` - Peer information and tags
    ///
    /// # Errors
    ///
    /// Returns error if security provider is unreachable or returns invalid response.
    pub async fn evaluate_trust(&self, request: &TrustEvaluationRequest) -> Result<TrustEvaluationResponse> {
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
            peer_family: request.peer_family.clone(), // ✅ Pass peer_family (v3.14.1)
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
                    trust_level: universal_resp.trust_level.name().to_string(), // Convert TrustLevel enum to string
                    confidence: 0.0, // Not in universal format yet
                    reason: universal_resp.reason,
                    encryption_tag: None, // Not in universal format yet
                    metadata: universal_resp.metadata.unwrap_or_default()
                        .into_iter()
                        .map(|(k, v)| (k, v.to_string()))
                        .collect(),
                }
            },
            Err(e) => {
                warn!("Security provider trust evaluation failed for peer {}: {}", 
                      request.peer_id, e);
                
                // Return reject decision on error (fail-safe)
                TrustEvaluationResponse {
                    decision: "reject".to_string(),
                    trust_level: "none".to_string(),
                    confidence: 0.0,
                    reason: format!("Security provider error: {}", e),
                    encryption_tag: None,
                    metadata: HashMap::new(),
                }
            }
        };

        match decision.decision.as_str() {
            "auto_accept" => {
                info!("✅ Security provider auto-accepts peer {} ({})", 
                      request.peer_id, decision.reason);
            },
            "prompt_user" => {
                warn!("⚠️ Security provider requests user prompt for peer {} ({})", 
                      request.peer_id, decision.reason);
            },
            "reject" => {
                error!("❌ Security provider rejects peer {} ({})", 
                       request.peer_id, decision.reason);
            },
            other => {
                warn!("Unknown security provider decision '{}' for peer {}, treating as reject", 
                      other, request.peer_id);
            },
        }

        Ok(decision)
    }

    /// Check if security provider is available
    ///
    /// **MODERNIZED v3.12.3**: Protocol-agnostic! Uses tarpc/JSON-RPC/HTTP automatically.
    ///
    /// Returns true if security provider responds to health checks.
    pub async fn is_available(&self) -> bool {
        match self.adapter.check_health().await {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    /// Get security provider endpoint
    ///
    /// **MODERNIZED v3.12.3**: Returns endpoint from protocol-agnostic adapter.
    #[must_use]
    pub fn endpoint(&self) -> &str {
        self.adapter.endpoint()
    }
    
    /// Convert identity response to universal attestations
    ///
    /// Creates generic identity attestations from provider-specific identity.
    pub fn identity_to_attestations(identity: &IdentityResponse) -> Vec<UniversalIdentityAttestation> {
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
    
    /// Convert identity response to discovery attestations (CRITICAL FIX - Jan 3, 2026)
    ///
    /// Creates discovery `IdentityAttestation` for inclusion in UDP discovery packets.
    /// This enables genetic lineage auto-trust across the network.
    pub fn identity_to_discovery_attestations(identity: &IdentityResponse) -> Vec<songbird_discovery::IdentityAttestation> {
        let mut attestations = Vec::new();
        
        // Create tag list attestation with family ID
        if !identity.encryption_tag.is_empty() {
            let mut data = json!({
                "tags": vec![identity.encryption_tag.clone()]
            });
            
            // Add family_id if present (CRITICAL for auto-trust)
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
    ///
    /// Generic trust evaluation that works with any security provider.
    ///
    /// # Arguments
    ///
    /// * `request` - Universal trust evaluation request
    ///
    /// # Returns
    ///
    /// Universal trust response with decision, confidence, and reason.
    pub async fn evaluate_trust_universal(&self, request: &UniversalTrustRequest) -> Result<UniversalTrustResponse> {
        let url = format!("{}/api/v1/trust/evaluate", self.adapter.endpoint());
        debug!("Evaluating trust (universal API): {}", url);
        
        let response = self.http_client
            .post(&url)
            .json(request)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .context("Failed to connect to security provider for trust evaluation")?;
        
        // ✅ AGNOSTIC: Gracefully handles wrapped or unwrapped format
        // Falls back to legacy format if universal format fails
        let trust_response = match self.parse_response::<UniversalTrustResponse>(response).await {
            Ok(response) => response,
            Err(e) => {
                warn!("Universal trust evaluation failed: {}. Trying legacy fallback...", e);
                // Try legacy format (for backward compatibility during transition)
                self.evaluate_trust_legacy_fallback(request).await?
            }
        };
        
        info!("✅ Trust evaluation complete: decision={:?}, confidence={}", 
              trust_response.decision, trust_response.confidence);
        
        Ok(trust_response)
    }
    
    /// Fallback to legacy trust evaluation format
    ///
    /// Used during transition period when provider hasn't updated to universal API yet.
    async fn evaluate_trust_legacy_fallback(&self, universal_request: &UniversalTrustRequest) -> Result<UniversalTrustResponse> {
        warn!("Falling back to legacy trust evaluation format");
        
        // Extract tags from attestations
        let mut tags = Vec::new();
        for attestation in &universal_request.evaluator.attestations {
            if attestation.format == "tag_list" {
                if let Some(tag_array) = attestation.data.get("tags").and_then(|t| t.as_array()) {
                    for tag in tag_array {
                        if let Some(tag_str) = tag.as_str() {
                            tags.push(tag_str.to_string());
                        }
                    }
                }
            }
        }
        
        // Build legacy request
        let legacy_request = TrustEvaluationRequest {
            peer_id: universal_request.evaluator.peer_id.clone(),
            peer_family: None, // ✅ v3.14.1: Family extraction implemented in evaluate_peer_trust()
            peer_tags: tags,
            connection_info: Some(ConnectionInfo {
                endpoint: universal_request.context.endpoint.clone(),
                protocol: "tarpc".to_string(),
            }),
            context: None, // Legacy format doesn't use structured context
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
            reason_code: legacy_response.reason.clone(), // Use reason as code
            metadata: legacy_response.metadata.iter()
                .map(|(k, v)| (k.clone(), json!(v)))
                .collect(),
            expires_at: None,
            custom: HashMap::new(),
        })
    }
    
    /// Backward compatibility: alias for from_endpoint
    #[deprecated(note = "Use from_endpoint instead for clarity")]
    /// Create from endpoint (legacy wrapper)
    ///
    /// **MODERNIZED v3.12.3**: Now returns Result due to protocol detection
    pub fn new(endpoint: impl Into<String>) -> Result<Self> {
        Self::from_endpoint(endpoint)
    }

    /// Get our current genetic lineage from security provider
    ///
    /// Returns our lineage ID and proof if available.
    ///
    /// # Errors
    ///
    /// Returns error if security provider is unreachable or returns invalid response.
    pub async fn get_current_lineage(&self) -> Result<Option<CurrentLineageInfo>> {
        let url = format!("{}/api/v1/lineage/current", self.adapter.endpoint());
        debug!("Querying security provider for current lineage: {}", url);

        let response = self.http_client
            .get(&url)
            .send()
            .await
            .context("Failed to connect to security provider for lineage query")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!("Security provider lineage query returned {}: {}", status, body);
            
            // If not found, return None (no lineage configured)
            if status.as_u16() == 404 {
                return Ok(None);
            }
            
            anyhow::bail!("Security provider returned error: {}", status);
        }

        let lineage_info = response.json::<CurrentLineageInfo>()
            .await
            .context("Failed to parse security provider lineage response")?;

        info!("✅ Retrieved current lineage from security provider: {}", lineage_info.lineage_id);
        Ok(Some(lineage_info))
    }
    
    /// Verify a peer's lineage proof
    ///
    /// Asks security provider to cryptographically verify the lineage proof.
    ///
    /// # Errors
    ///
    /// Returns error if security provider is unreachable or returns invalid response.
    pub async fn verify_lineage(&self, proof: &LineageProof) -> Result<VerificationResult> {
        let url = format!("{}/api/v1/lineage/verify", self.adapter.endpoint());
        debug!("Verifying lineage proof with security provider: {}", url);

        let response = self.http_client
            .post(&url)
            .json(proof)
            .send()
            .await
            .context("Failed to connect to security provider for lineage verification")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Security provider lineage verification failed: {} - {}", status, body);
            
            // Return invalid verification result on error  
            // LineageId::new does not return Result - it just wraps the string
            let invalid_lineage = LineageId::new("error-invalid".to_string());
            return Ok(VerificationResult {
                valid: false,
                same_genesis: false,
                lineage_id: invalid_lineage,
                messages: vec![format!("Security provider error: {}", status)],
            });
        }

        let result = response.json::<VerificationResult>()
            .await
            .context("Failed to parse security provider verification response")?;

        if result.valid {
            info!("✅ Lineage proof verified by security provider");
        } else {
            warn!("❌ Lineage proof rejected by security provider: {:?}", result.messages);
        }

        Ok(result)
    }
    
    /// Check if two lineages are from the same genetic family
    ///
    /// Asks security provider to compare lineage origins.
    ///
    /// # Errors
    ///
    /// Returns error if security provider is unreachable or returns invalid response.
    pub async fn same_family(&self, lineage_a: &LineageId, lineage_b: &LineageId) -> Result<bool> {
        let url = format!("{}/api/v1/lineage/same_family", self.adapter.endpoint());
        debug!("Checking if lineages are from same family: {} vs {}", lineage_a, lineage_b);

        #[derive(Serialize)]
        struct SameFamilyRequest {
            lineage_a: String,
            lineage_b: String,
        }

        #[derive(Deserialize)]
        struct SameFamilyResponse {
            same_family: bool,
            confidence: f64,
        }

        let request = SameFamilyRequest {
            lineage_a: lineage_a.to_string(),
            lineage_b: lineage_b.to_string(),
        };

        let response = self.http_client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to connect to security provider for family check")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!("Security provider family check failed: {} - {}", status, body);
            
            // Conservative: assume different families on error
            return Ok(false);
        }

        let result = response.json::<SameFamilyResponse>()
            .await
            .context("Failed to parse security provider family check response")?;

        if result.same_family {
            info!("✅ Lineages are from same genetic family (confidence: {:.2})", result.confidence);
        } else {
            debug!("Different genetic families");
        }

        Ok(result.same_family)
    }
}

/// Identity response from security provider
///
/// Contains our encryption tag and capabilities.
/// Orchestrator doesn't need to understand the tag format,
/// just includes it in discovery packets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityResponse {
    /// Encryption tag for this node
    ///
    /// Format: `{provider}:family:{family_id}:{node_id}` (provider-agnostic!)
    /// Example: `crypto-provider:family:a3f2:tower1`
    pub encryption_tag: String,
    
    /// Security provider capabilities
    ///
    /// Example: `["identity", "encryption", "trust-evaluation"]`
    pub capabilities: Vec<String>,
    
    /// Family ID (optional)
    ///
    /// Example: `ecoPrimals-20260101-a3f2`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family_id: Option<String>,
}

/// Trust evaluation request to security provider
///
/// Orchestrator sends peer information to security provider,
/// asking "should I trust this peer?"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustEvaluationRequest {
    /// Peer node ID
    pub peer_id: String,
    
    /// Peer family ID (v3.14.1 - tag-based identity)
    ///
    /// Extracted from peer tags (e.g., "beardog:family:nat0" → "nat0")
    /// Songbird doesn't interpret this - just extracts and passes to security provider
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer_family: Option<String>,
    
    /// Peer tags (includes security provider encryption tag if present)
    ///
    /// Example: `["crypto:family:a3f2", "encryption_enabled"]`
    pub peer_tags: Vec<String>,
    
    /// Connection information (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_info: Option<ConnectionInfo>,
    
    /// Discovery context (optional, flattened HashMap for security provider compatibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<HashMap<String, String>>,
}

/// Connection information for peer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    /// Peer endpoint
    pub endpoint: String,
    
    /// Protocol used
    pub protocol: String,
}

/// Discovery context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryContext {
    /// How peer was discovered
    pub discovery_method: String,
    
    /// When peer was first seen (Unix timestamp as string for JSON compatibility)
    pub first_seen_at: String,
    
    /// Additional metadata
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, String>,
}

/// Trust evaluation response from security provider
///
/// Provider's decision on whether to trust the peer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustEvaluationResponse {
    /// Decision: "auto_accept", "prompt_user", or "reject"
    pub decision: String,
    
    /// Trust level: "high", "medium", "low", or "none"
    pub trust_level: String,
    
    /// Confidence score (0.0-1.0)
    pub confidence: f64,
    
    /// Human-readable reason
    pub reason: String,
    
    /// Encryption tag for establishing secure connection
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encryption_tag: Option<String>,
    
    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl TrustEvaluationResponse {
    /// Check if decision is to auto-accept
    #[must_use]
    pub fn is_auto_accept(&self) -> bool {
        self.decision == "auto_accept"
    }

    /// Check if decision requires user prompt
    #[must_use]
    pub fn requires_prompt(&self) -> bool {
        self.decision == "prompt_user"
    }

    /// Check if decision is to reject
    #[must_use]
    pub fn is_reject(&self) -> bool {
        self.decision == "reject"
    }
}

/// Current lineage information from security provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentLineageInfo {
    pub lineage_id: LineageId,
    pub proof: LineageProof,
    pub genesis_timestamp: u64,
}

/// Verification result from security provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub valid: bool,
    pub same_genesis: bool,
    pub lineage_id: LineageId,
    pub messages: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        // No hardcoded endpoint! Discovered at runtime
        let client = SecurityCapabilityClient::from_endpoint("http://discovered-security-provider");
        assert_eq!(client.unwrap().endpoint(), "http://discovered-security-provider");
    }

    #[test]
    fn test_trust_decision_helpers() {
        let auto_accept = TrustEvaluationResponse {
            decision: "auto_accept".to_string(),
            trust_level: "high".to_string(),
            confidence: 1.0,
            reason: "same_family".to_string(),
            encryption_tag: Some("crypto-provider:family:a3f2".to_string()),
            metadata: HashMap::new(),
        };
        
        assert!(auto_accept.is_auto_accept());
        assert!(!auto_accept.requires_prompt());
        assert!(!auto_accept.is_reject());

        let prompt = TrustEvaluationResponse {
            decision: "prompt_user".to_string(),
            trust_level: "low".to_string(),
            confidence: 0.5,
            reason: "different_family".to_string(),
            encryption_tag: None,
            metadata: HashMap::new(),
        };
        
        assert!(!prompt.is_auto_accept());
        assert!(prompt.requires_prompt());
        assert!(!prompt.is_reject());

        let reject = TrustEvaluationResponse {
            decision: "reject".to_string(),
            trust_level: "none".to_string(),
            confidence: 0.0,
            reason: "no_lineage".to_string(),
            encryption_tag: None,
            metadata: HashMap::new(),
        };
        
        assert!(!reject.is_auto_accept());
        assert!(!reject.requires_prompt());
        assert!(reject.is_reject());
    }

    #[test]
    fn test_identity_response_serialization() {
        let identity = IdentityResponse {
            encryption_tag: "crypto-provider:family:a3f2:tower1".to_string(),
            capabilities: vec!["identity".to_string(), "encryption".to_string()],
            family_id: Some("ecoPrimals-20260101-a3f2".to_string()),
        };

        let json = serde_json::to_string(&identity).unwrap();
        let deserialized: IdentityResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(identity.encryption_tag, deserialized.encryption_tag);
        assert_eq!(identity.capabilities, deserialized.capabilities);
        assert_eq!(identity.family_id, deserialized.family_id);
    }

    #[test]
    fn test_trust_request_serialization() {
        let mut context = HashMap::new();
        context.insert("discovery_method".to_string(), "udp_multicast".to_string());
        context.insert("first_seen_at".to_string(), "2024-01-01T12:00:00Z".to_string());
        
        let request = TrustEvaluationRequest {
            peer_id: "tower2".to_string(),
            peer_family: Some("a3f2".to_string()), // Extracted from tags
            peer_tags: vec!["crypto-provider:family:a3f2".to_string()],
            connection_info: Some(ConnectionInfo {
                endpoint: "https://192.168.1.134:8080".to_string(),
                protocol: "tarpc".to_string(),
            }),
            context: Some(context),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: TrustEvaluationRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.peer_id, deserialized.peer_id);
        assert_eq!(request.peer_tags, deserialized.peer_tags);
    }
}

