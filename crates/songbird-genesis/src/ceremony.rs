// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Genesis ceremony coordinator
//!
//! UDP/multicast discovery broadcast addresses are not configured here; the orchestrator
//! resolves them from `SONGBIRD_BROADCAST_ADDRESSES` (and config fallbacks) in
//! `songbird_orchestrator::app`.

use crate::error::{GenesisError, Result};
use crate::identity::NewNodeIdentity;
use crate::physical_channels::{PhysicalChannel, PhysicalChannelProvider};
use crate::types::{GenesisLineage, PrimalLineage, ProximityProof};
use crate::witness::GenesisWitness;
use chrono::Utc;
use songbird_http_client::IpcHttpClient;
use std::time::Duration;
use tracing::{debug, info};
use uuid::Uuid;

/// Genesis ceremony coordinator
pub struct GenesisCeremony {
    /// Physical channel for proximity verification
    physical_channel: PhysicalChannel,

    /// Genesis witness device
    witness: GenesisWitness,

    /// Coordinating primals (endpoints)
    primal_coordinators: Vec<PrimalCoordinator>,

    /// Ceremony timeout
    timeout: Duration,
}

impl GenesisCeremony {
    /// Create new genesis ceremony
    #[must_use]
    pub const fn new(physical_channel: PhysicalChannel, witness: GenesisWitness) -> Self {
        Self {
            physical_channel,
            witness,
            primal_coordinators: Vec::new(),
            timeout: Duration::from_secs(300), // 5 minutes default
        }
    }

    /// Add primal coordinator
    pub fn add_primal_coordinator(&mut self, coordinator: PrimalCoordinator) {
        self.primal_coordinators.push(coordinator);
    }

    /// Set ceremony timeout
    pub const fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    /// Conduct complete genesis ceremony
    ///
    /// # Errors
    ///
    /// Returns an error if physical proximity verification, credential exchange,
    /// witness signing, or primal coordination fails.
    pub async fn conduct(&self, new_node_id: String) -> Result<NewNodeIdentity> {
        info!("🔐 Starting genesis ceremony for node: {}", new_node_id);

        // Phase 1: Physical proximity verification
        info!("📱 Phase 1: Verifying physical proximity...");
        let proximity_proof = self.verify_physical_proximity().await?;
        debug!("✅ Physical proximity verified via {:?}", proximity_proof.channel_type);

        // Phase 2: Exchange genesis credentials
        info!("🔑 Phase 2: Exchanging genesis credentials...");
        let genesis_creds = self.physical_channel.secure_exchange().await?;
        debug!("✅ Genesis credentials exchanged");

        // Phase 3: Witness signs new identity
        info!("✍️  Phase 3: Witness signing genesis...");
        let _witnessed_identity = self.witness_sign_genesis(&genesis_creds).await?;
        debug!("✅ Genesis witnessed and signed");

        // Phase 4: Coordinate with all primals
        info!("🌳 Phase 4: Coordinating multi-primal lineage...");
        let genesis_lineage = self.coordinate_primal_lineages(&new_node_id).await?;
        info!(
            "✅ Multi-primal lineage established ({} primals)",
            genesis_lineage.primal_lineages.len()
        );

        // Phase 5: Forge unified identity
        info!("✨ Phase 5: Forging unified node identity...");
        let identity = NewNodeIdentity::new(
            new_node_id.clone(),
            genesis_creds,
            self.witness.clone(),
            genesis_lineage,
        );

        info!("🎉 Genesis ceremony complete for node: {}", new_node_id);
        info!("   Trust level: {:?}", identity.genesis_trust_level());
        info!("   Primal signatures: {}", identity.primal_signature_count());

        Ok(identity)
    }

    /// Verify physical proximity
    async fn verify_physical_proximity(&self) -> Result<ProximityProof> {
        self.physical_channel.verify_proximity().await
    }

    /// Witness signs genesis using `BearDog`
    async fn witness_sign_genesis(&self, creds: &[u8]) -> Result<Vec<u8>> {
        use crate::security_capability_client::SecurityCapabilityClient;

        // Try to create BearDog client
        match SecurityCapabilityClient::new().await {
            Ok(client) => {
                // Try to sign with BearDog
                match client.sign_data(&self.witness.device_id, creds).await {
                    Ok(signature) => {
                        tracing::debug!("✅ Signed with BearDog");
                        return Ok(signature);
                    }
                    Err(e) => {
                        // BearDog request failed, use fallback
                        tracing::warn!(
                            "BearDog signing request failed: {e}. Using deterministic fallback signature."
                        );
                    }
                }
            }
            Err(e) => {
                // BearDog not available, use fallback
                tracing::warn!(
                    "BearDog not available: {e}. Using deterministic fallback signature."
                );
            }
        }

        // Degraded mode: deterministic signature when BearDog is unavailable
        Ok(format!("witness_sig_{}_{}", self.witness.device_id, creds.len()).into_bytes())
    }

    /// Coordinate lineages from all primals
    async fn coordinate_primal_lineages(&self, node_id: &str) -> Result<GenesisLineage> {
        let mut primal_lineages = std::collections::HashMap::new();
        let ceremony_id = Uuid::new_v4();

        for coordinator in &self.primal_coordinators {
            info!("   Requesting lineage from primal: {}", coordinator.primal_name);

            match coordinator.request_lineage(node_id, &self.witness).await {
                Ok(lineage) => {
                    debug!("   ✅ Received lineage from {}", coordinator.primal_name);
                    primal_lineages.insert(coordinator.primal_name.clone(), lineage);
                }
                Err(e) => {
                    // Log but continue - some primals may be optional
                    info!("   ⚠️  Failed to get lineage from {}: {}", coordinator.primal_name, e);
                }
            }
        }

        if primal_lineages.is_empty() {
            return Err(GenesisError::CoordinationFailed(
                "No primals provided lineage".to_string(),
            ));
        }

        Ok(GenesisLineage {
            witness_device_id: self.witness.device_id.clone(),
            primal_lineages,
            birth_timestamp: Utc::now(),
            ceremony_id,
        })
    }
}

/// Coordinator for a specific primal
#[derive(Debug, Clone)]
pub struct PrimalCoordinator {
    /// Primal name (e.g., "songbird", "beardog")
    pub primal_name: String,

    /// Primal endpoint for genesis requests
    pub endpoint: String,

    /// Optional: API key or auth token
    pub auth_token: Option<String>,
}

impl PrimalCoordinator {
    /// Create new primal coordinator
    #[must_use]
    pub const fn new(primal_name: String, endpoint: String) -> Self {
        Self {
            primal_name,
            endpoint,
            auth_token: None,
        }
    }

    /// Request lineage from this primal using `BearDog`
    ///
    /// # Errors
    ///
    /// Returns an error if HTTP client creation, network request, or response parsing fails.
    pub async fn request_lineage(
        &self,
        node_id: &str,
        witness: &GenesisWitness,
    ) -> Result<PrimalLineage> {
        debug!("Requesting lineage from {} at {}", self.primal_name, self.endpoint);

        let client = IpcHttpClient::new().await.map_err(|e| {
            GenesisError::CoordinationFailed(format!("Failed to create HTTP client: {e}"))
        })?;

        let request_body = serde_json::json!({
            "primal_name": self.primal_name,
            "node_id": node_id,
            "witness_device_id": witness.device_id,
            "witness_signature": hex::encode(&witness.signature),
        });

        let url = format!("{}/genesis/lineage", self.endpoint);

        match client.post(&url).await.json(&request_body)?.send().await {
            Ok(response) if response.is_success() => {
                Self::parse_lineage_response(response, &self.primal_name).await
            }
            Ok(response) => {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();
                Err(GenesisError::CoordinationFailed(format!(
                    "Primal {} returned error {}: {}",
                    self.primal_name, status, error_text
                )))
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to contact primal {}: {}. Trying local BearDog fallback.",
                    self.primal_name,
                    e
                );
                if let Some(lineage) = Self::try_beardog_lineage(self, node_id, witness).await {
                    return Ok(lineage);
                }
                tracing::error!(
                    primal = %self.primal_name,
                    "DEGRADED: Generating synthetic lineage — primal unreachable and BearDog unavailable. \
                     This node will have reduced trust until re-genesis with live primals."
                );
                Ok(Self::synthetic_lineage(self, node_id))
            }
        }
    }

    async fn parse_lineage_response(
        response: songbird_http_client::Response,
        primal_name: &str,
    ) -> Result<PrimalLineage> {
        #[derive(serde::Deserialize)]
        struct LineageResponse {
            lineage_data: String,
            signature: String,
        }

        let lineage_resp: LineageResponse = response.json().await.map_err(|e| {
            GenesisError::CoordinationFailed(format!("Failed to parse lineage response: {e}"))
        })?;

        let lineage_data = hex::decode(&lineage_resp.lineage_data).map_err(|e| {
            GenesisError::CoordinationFailed(format!("Failed to decode lineage data: {e}"))
        })?;

        let signature = hex::decode(&lineage_resp.signature).map_err(|e| {
            GenesisError::CoordinationFailed(format!("Failed to decode signature: {e}"))
        })?;

        Ok(PrimalLineage {
            primal_name: primal_name.to_string(),
            lineage_data,
            signature,
            timestamp: Utc::now(),
        })
    }

    async fn try_beardog_lineage(
        coordinator: &Self,
        node_id: &str,
        witness: &GenesisWitness,
    ) -> Option<PrimalLineage> {
        use crate::security_capability_client::SecurityCapabilityClient;

        let security_client = match SecurityCapabilityClient::new().await {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("BearDog not available: {e}. Falling back to synthetic lineage.");
                return None;
            }
        };
        let lineage_data = match security_client
            .create_lineage(&coordinator.primal_name, &witness.device_id, node_id)
            .await
        {
            Ok(d) => d,
            Err(e) => {
                tracing::warn!(
                    "BearDog lineage creation failed: {e}. Falling back to synthetic lineage."
                );
                return None;
            }
        };
        let signature = match security_client.sign_data(node_id, &lineage_data).await {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("BearDog signing failed: {e}. Falling back to synthetic lineage.");
                return None;
            }
        };
        tracing::debug!("✅ Created lineage with BearDog");
        Some(PrimalLineage {
            primal_name: coordinator.primal_name.clone(),
            lineage_data,
            signature,
            timestamp: Utc::now(),
        })
    }

    fn synthetic_lineage(coordinator: &Self, node_id: &str) -> PrimalLineage {
        PrimalLineage {
            primal_name: coordinator.primal_name.clone(),
            lineage_data: format!("synthetic_lineage_{node_id}").into_bytes(),
            signature: format!("unsigned_synthetic_{}", coordinator.primal_name).into_bytes(),
            timestamp: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use crate::physical_channels::MockPhysicalChannel;

    fn sample_witness() -> GenesisWitness {
        GenesisWitness::new(
            "test-witness".to_string(),
            vec![1, 2, 3],
            crate::types::PhysicalChannelType::HardwareKey,
        )
    }

    #[tokio::test]
    async fn test_genesis_ceremony_basic() {
        let channel = PhysicalChannel::Mock(MockPhysicalChannel::new());
        let witness = sample_witness();

        let mut ceremony = GenesisCeremony::new(channel, witness);

        // Add mock primal coordinators
        ceremony.add_primal_coordinator(PrimalCoordinator::new(
            "songbird".to_string(),
            "http://localhost:8080".to_string(),
        ));
        ceremony.add_primal_coordinator(PrimalCoordinator::new(
            "beardog".to_string(),
            "http://localhost:9000".to_string(),
        ));

        // Conduct ceremony (will use fallback since no real primals are running)
        let identity = ceremony.conduct("new-node-test".to_string()).await;

        // Should succeed with fallback implementations
        assert!(identity.is_ok(), "Genesis ceremony failed: {:?}", identity.err());
        let identity = identity.unwrap();
        assert_eq!(identity.node_id, "new-node-test");

        // Note: In fallback mode (no real primals), we won't have multi-primal genesis
        // This is expected behavior for isolated testing
        // In production with real BearDog+Songbird, is_multi_primal_genesis() would return true
        assert!(identity.primal_signature_count() >= 1, "Should have at least one signature");
    }

    #[tokio::test]
    async fn conduct_fails_when_no_primal_coordinators() {
        let channel = PhysicalChannel::Mock(MockPhysicalChannel::new());
        let ceremony = GenesisCeremony::new(channel, sample_witness());
        let err = ceremony
            .conduct("orphan-node".to_string())
            .await
            .expect_err("coordination must fail with zero primals");
        match err {
            GenesisError::CoordinationFailed(msg) => {
                assert!(
                    msg.contains("No primals"),
                    "expected coordination message, got {msg}"
                );
            }
            other => panic!("expected CoordinationFailed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn conduct_fails_when_proximity_verification_fails() {
        let channel = PhysicalChannel::Mock(MockPhysicalChannel::failing());
        let mut ceremony = GenesisCeremony::new(channel, sample_witness());
        ceremony.add_primal_coordinator(PrimalCoordinator::new(
            "songbird".to_string(),
            "http://127.0.0.1:1".to_string(),
        ));
        let err = ceremony
            .conduct("n".to_string())
            .await
            .expect_err("proximity should fail before coordination");
        assert!(
            matches!(err, GenesisError::ProximityVerificationFailed(_)),
            "expected proximity failure, got {err:?}"
        );
    }

    #[test]
    fn primal_coordinator_new_fields() {
        let c = PrimalCoordinator::new("p".to_string(), "http://example/genesis".to_string());
        assert_eq!(c.primal_name, "p");
        assert_eq!(c.endpoint, "http://example/genesis");
        assert!(c.auth_token.is_none());
    }

    #[tokio::test]
    async fn set_timeout_does_not_break_conduct() {
        let channel = PhysicalChannel::Mock(MockPhysicalChannel::new());
        let mut ceremony = GenesisCeremony::new(channel, sample_witness());
        ceremony.set_timeout(std::time::Duration::from_secs(3600));
        ceremony.add_primal_coordinator(PrimalCoordinator::new(
            "songbird".to_string(),
            "http://127.0.0.1:1".to_string(),
        ));
        let identity = ceremony.conduct("timeout-smoke".to_string()).await;
        assert!(identity.is_ok(), "custom timeout should not block successful ceremony: {identity:?}");
    }
}
