//! Genetic Lineage Authentication
//!
//! Implements automatic peer trust based on cryptographic lineage verification.
//! Enables auto-accept for same-lineage peers while requiring user consent for
//! different or unknown lineages.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use songbird_types::{LineageId, LineageProof};
use tracing::{info, warn};

// Import security capability client
use crate::security_capability_client::SecurityCapabilityClient;

/// Decision for peer acceptance based on genetic lineage
#[derive(Debug, Clone)]
pub enum PeerAcceptanceDecision {
    /// Automatically accept (same genetic lineage - cryptographically verified)
    AutoAccept {
        /// Reason for auto-accept
        reason: String,
        /// Verified lineage ID
        lineage_id: LineageId,
        /// Confidence level (0.0-1.0)
        confidence: f64,
    },
    
    /// Prompt user for decision (different or unknown lineage)
    PromptUser {
        /// Peer information for user review
        peer_info: PeerInfo,
        /// Lineage status explanation
        lineage_status: LineageStatus,
        /// Recommended action
        recommendation: UserRecommendation,
    },
    
    /// Reject peer (invalid proof or security concern)
    Reject {
        /// Reason for rejection
        reason: String,
        /// Severity of the issue
        severity: RejectionSeverity,
    },
}

/// Lineage status for peer
#[derive(Debug, Clone)]
pub enum LineageStatus {
    /// Same genesis - cryptographically verified family
    SameGenesis {
        lineage_id: LineageId,
        genesis_timestamp: u64,
    },
    
    /// Different genesis - separate lineage family
    DifferentGenesis {
        their_lineage: LineageId,
        our_lineage: LineageId,
    },
    
    /// Unknown lineage - peer has no lineage information
    UnknownLineage,
    
    /// Invalid proof - cryptographic verification failed
    InvalidProof {
        error: String,
    },
}

/// Recommendation for user when prompting
#[derive(Debug, Clone)]
pub enum UserRecommendation {
    /// Suggest accepting (low risk)
    Accept,
    /// Neutral (user should decide)
    Neutral,
    /// Suggest rejecting (higher risk)
    Reject,
}

/// Severity of rejection
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RejectionSeverity {
    /// Low - Could be temporary issue
    Low,
    /// Medium - Suspicious behavior
    Medium,
    /// High - Security concern
    High,
    /// Critical - Active threat
    Critical,
}

/// Peer information for user review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Peer node ID
    pub node_id: String,
    /// Peer endpoint
    pub endpoint: String,
    /// Advertised capabilities
    pub capabilities: Vec<String>,
    /// When discovered
    pub discovered_at: u64,
}

/// Genetic lineage authenticator
pub struct LineageAuthenticator {
    /// Security capability client for lineage verification
    security_client: Option<SecurityCapabilityClient>,
    
    /// Our local lineage (if available)
    local_lineage: Option<LineageId>,
    
    /// Cache of verified lineages
    verification_cache: std::collections::HashMap<String, CachedVerification>,
    
    /// Cache TTL
    cache_ttl: std::time::Duration,
}

/// Cached verification result
#[derive(Debug, Clone)]
struct CachedVerification {
    valid: bool,
    same_genesis: bool,
    cached_at: std::time::Instant,
}

/// Simplified BearDog client for lineage operations
/// (Will use actual BearDog client when Phase 1.5 is ready)
#[derive(Debug, Clone)]
pub struct BearDogClient {
    endpoint: String,
    http_client: reqwest::Client,
}

impl BearDogClient {
    /// Create a new BearDog client
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            http_client: reqwest::Client::new(),
        }
    }

    /// Verify lineage proof
    pub async fn verify_lineage(&self, proof: &LineageProof) -> Result<VerificationResult> {
        // TODO: Call actual BearDog API when Phase 1.5 is ready
        // For now, implement graceful fallback
        
        info!("🔍 Verifying lineage proof via BearDog (mock implementation)");
        
        // Mock verification - always succeeds for development
        // In production, this will call POST /api/v1/lineage/verify
        Ok(VerificationResult {
            valid: true,
            same_genesis: false,
            lineage_id: proof.lineage_id.clone(),
            messages: vec!["Mock verification - BearDog Phase 1.5 pending".to_string()],
        })
    }

    /// Check if two lineages share the same genesis
    pub async fn same_family(&self, lineage_a: &LineageId, lineage_b: &LineageId) -> Result<bool> {
        // TODO: Call actual BearDog API when Phase 1.5 is ready
        // For now, compare tower IDs as a heuristic
        
        Ok(lineage_a.tower_id() == lineage_b.tower_id())
    }

    /// Get current lineage for this node
    pub async fn get_current_lineage(&self) -> Result<Option<CurrentLineageInfo>> {
        // TODO: Call actual BearDog API when Phase 1.5 is ready
        // For now, return None (graceful degradation)
        
        Ok(None)
    }
}

/// Verification result from BearDog
#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub valid: bool,
    pub same_genesis: bool,
    pub lineage_id: LineageId,
    pub messages: Vec<String>,
}

/// Current lineage information
#[derive(Debug, Clone)]
pub struct CurrentLineageInfo {
    pub lineage_id: LineageId,
    pub proof: LineageProof,
    pub genesis_timestamp: u64,
}

impl LineageAuthenticator {
    /// Create a new lineage authenticator
    pub fn new() -> Self {
        Self {
            security_client: None,
            local_lineage: None,
            verification_cache: std::collections::HashMap::new(),
            cache_ttl: std::time::Duration::from_secs(300), // 5 minutes
        }
    }

    /// Initialize with security capability client
    pub async fn initialize(&mut self, security_endpoint: &str) -> Result<()> {
        info!("🔐 Initializing lineage authenticator with security provider: {}", security_endpoint);
        
        let client = SecurityCapabilityClient::from_endpoint(security_endpoint);
        
        // Try to get our local lineage
        if let Ok(Some(lineage_info)) = client.get_current_lineage().await {
            self.local_lineage = Some(lineage_info.lineage_id.clone());
            info!("✅ Initialized with genetic lineage: {}", lineage_info.lineage_id);
        } else {
            warn!("⚠️ No genetic lineage found - will prompt for all peers");
        }
        
        self.security_client = Some(client);
        
        Ok(())
    }

    /// Evaluate peer for auto-accept based on genetic lineage
    pub async fn evaluate_peer(
        &mut self,
        peer_node_id: &str,
        peer_endpoint: &str,
        peer_capabilities: &[String],
        peer_lineage: Option<&LineageId>,
        peer_proof: Option<&LineageProof>,
    ) -> Result<PeerAcceptanceDecision> {
        // If peer has no lineage, prompt user
        let (lineage, proof) = match (peer_lineage, peer_proof) {
            (Some(l), Some(p)) => (l, p),
            _ => {
                info!("⚠️ Peer {} has no lineage - prompting user", peer_node_id);
                return Ok(PeerAcceptanceDecision::PromptUser {
                    peer_info: PeerInfo {
                        node_id: peer_node_id.to_string(),
                        endpoint: peer_endpoint.to_string(),
                        capabilities: peer_capabilities.to_vec(),
                        discovered_at: Self::current_timestamp(),
                    },
                    lineage_status: LineageStatus::UnknownLineage,
                    recommendation: UserRecommendation::Neutral,
                });
            }
        };

        // Check cache first
        if let Some(cached) = self.get_cached_verification(peer_node_id) {
            if cached.valid && cached.same_genesis {
                return Ok(PeerAcceptanceDecision::AutoAccept {
                    reason: format!("Same genetic lineage: {} (cached)", lineage),
                    lineage_id: lineage.clone(),
                    confidence: 0.95, // Slightly lower for cached
                });
            }
        }

        // Verify lineage proof with security provider
        let security = self.security_client.as_ref()
            .context("Security provider client not initialized")?;
        
        let verification = security.verify_lineage(proof).await
            .context("Failed to verify lineage with security provider")?;
        
        if !verification.valid {
            warn!("❌ Invalid lineage proof from peer {}", peer_node_id);
            return Ok(PeerAcceptanceDecision::Reject {
                reason: format!("Invalid lineage proof: {}", verification.messages.join(", ")),
                severity: RejectionSeverity::High,
            });
        }

        // If we have local lineage, check if same genesis
        let our_lineage_clone = self.local_lineage.clone();
        if let Some(our_lineage) = &our_lineage_clone {
            let same_family = security.same_family(our_lineage, lineage).await?;
            
            // Cache the verification AFTER all beardog operations
            self.cache_verification(peer_node_id, verification.valid, same_family);
            
            if same_family {
                info!("✅ Auto-accepting peer {} - same genetic family", peer_node_id);
                return Ok(PeerAcceptanceDecision::AutoAccept {
                    reason: format!("Same genetic lineage: {}", lineage),
                    lineage_id: lineage.clone(),
                    confidence: 1.0,
                });
            } else {
                info!("⚠️ Different genetic lineage detected for peer {}", peer_node_id);
                return Ok(PeerAcceptanceDecision::PromptUser {
                    peer_info: PeerInfo {
                        node_id: peer_node_id.to_string(),
                        endpoint: peer_endpoint.to_string(),
                        capabilities: peer_capabilities.to_vec(),
                        discovered_at: Self::current_timestamp(),
                    },
                    lineage_status: LineageStatus::DifferentGenesis {
                        their_lineage: lineage.clone(),
                        our_lineage: our_lineage.clone(),
                    },
                    recommendation: UserRecommendation::Neutral,
                });
            }
        }

        // Cache the verification for no local lineage case
        self.cache_verification(peer_node_id, verification.valid, false);

        // No local lineage - prompt user
        Ok(PeerAcceptanceDecision::PromptUser {
            peer_info: PeerInfo {
                node_id: peer_node_id.to_string(),
                endpoint: peer_endpoint.to_string(),
                capabilities: peer_capabilities.to_vec(),
                discovered_at: Self::current_timestamp(),
            },
            lineage_status: LineageStatus::UnknownLineage,
            recommendation: UserRecommendation::Neutral,
        })
    }

    /// Cache a verification result
    fn cache_verification(&mut self, peer_id: &str, valid: bool, same_genesis: bool) {
        self.verification_cache.insert(
            peer_id.to_string(),
            CachedVerification {
                valid,
                same_genesis,
                cached_at: std::time::Instant::now(),
            },
        );
    }

    /// Get cached verification if still valid
    fn get_cached_verification(&self, peer_id: &str) -> Option<&CachedVerification> {
        self.verification_cache.get(peer_id).and_then(|cached| {
            if cached.cached_at.elapsed() < self.cache_ttl {
                Some(cached)
            } else {
                None
            }
        })
    }

    /// Get current Unix timestamp
    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

impl Default for LineageAuthenticator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use songbird_types::LineageProof;

    #[tokio::test]
    async fn test_authenticator_creation() {
        let auth = LineageAuthenticator::new();
        assert!(auth.local_lineage.is_none());
        assert!(auth.security_client.is_none());
    }

    #[tokio::test]
    async fn test_peer_without_lineage_prompts_user() {
        let mut auth = LineageAuthenticator::new();
        
        let decision = auth.evaluate_peer(
            "peer-1",
            "http://192.168.1.100:8080",
            &["compute".to_string()],
            None,
            None,
        ).await.unwrap();
        
        assert!(matches!(decision, PeerAcceptanceDecision::PromptUser { .. }));
    }

    #[test]
    fn test_rejection_severity() {
        assert_eq!(RejectionSeverity::Critical, RejectionSeverity::Critical);
        assert_ne!(RejectionSeverity::Low, RejectionSeverity::High);
    }
}

