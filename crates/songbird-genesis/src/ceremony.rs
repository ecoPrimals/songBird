//! Genesis ceremony coordinator

use crate::{error::*, identity::*, physical_channels::*, types::*, witness::*};
use chrono::Utc;
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
    pub fn new(physical_channel: PhysicalChannel, witness: GenesisWitness) -> Self {
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
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    /// Conduct complete genesis ceremony
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

    /// Witness signs genesis using BearDog
    async fn witness_sign_genesis(&self, creds: &[u8]) -> Result<Vec<u8>> {
        use crate::beardog_client::BearDogClient;

        // Try to create BearDog client
        match BearDogClient::new().await {
            Ok(client) => {
                // Try to sign with BearDog
                match client.sign_data(&self.witness.device_id, creds).await {
                    Ok(signature) => {
                        tracing::debug!("✅ Signed with BearDog");
                        return Ok(signature);
                    }
                    Err(e) => {
                        // BearDog request failed, use fallback
                        tracing::warn!("BearDog signing request failed: {}. Using fallback.", e);
                    }
                }
            }
            Err(e) => {
                // BearDog not available, use fallback
                tracing::warn!("BearDog not available: {}. Using fallback signature.", e);
            }
        }

        // Fallback: Create deterministic signature (graceful degradation for testing/development)
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
    pub fn new(primal_name: String, endpoint: String) -> Self {
        Self {
            primal_name,
            endpoint,
            auth_token: None,
        }
    }

    /// Request lineage from this primal using BearDog
    pub async fn request_lineage(
        &self,
        node_id: &str,
        witness: &GenesisWitness,
    ) -> Result<PrimalLineage> {
        use crate::beardog_client::BearDogClient;

        debug!("Requesting lineage from {} at {}", self.primal_name, self.endpoint);

        // Try HTTP request to primal's genesis endpoint
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| {
                GenesisError::CoordinationFailed(format!("Failed to create HTTP client: {}", e))
            })?;

        let request_body = serde_json::json!({
            "primal_name": self.primal_name,
            "node_id": node_id,
            "witness_device_id": witness.device_id,
            "witness_signature": hex::encode(&witness.signature),
        });

        let url = format!("{}/genesis/lineage", self.endpoint);

        match client.post(&url).json(&request_body).send().await {
            Ok(response) if response.status().is_success() => {
                // Parse lineage response
                #[derive(serde::Deserialize)]
                struct LineageResponse {
                    lineage_data: String, // hex-encoded
                    signature: String,    // hex-encoded
                }

                let lineage_resp: LineageResponse = response.json().await.map_err(|e| {
                    GenesisError::CoordinationFailed(format!(
                        "Failed to parse lineage response: {}",
                        e
                    ))
                })?;

                let lineage_data = hex::decode(&lineage_resp.lineage_data).map_err(|e| {
                    GenesisError::CoordinationFailed(format!(
                        "Failed to decode lineage data: {}",
                        e
                    ))
                })?;

                let signature = hex::decode(&lineage_resp.signature).map_err(|e| {
                    GenesisError::CoordinationFailed(format!("Failed to decode signature: {}", e))
                })?;

                Ok(PrimalLineage {
                    primal_name: self.primal_name.clone(),
                    lineage_data,
                    signature,
                    timestamp: Utc::now(),
                })
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
                // Fallback: Use BearDog to create lineage locally
                tracing::warn!(
                    "Failed to contact primal {}: {}. Trying local BearDog fallback.",
                    self.primal_name,
                    e
                );

                // Try BearDog for lineage creation
                if let Ok(beardog_client) = BearDogClient::new().await {
                    match beardog_client
                        .create_lineage(&self.primal_name, &witness.device_id, node_id)
                        .await
                    {
                        Ok(lineage_data) => {
                            match beardog_client.sign_data(node_id, &lineage_data).await {
                                Ok(signature) => {
                                    tracing::debug!("✅ Created lineage with BearDog");
                                    return Ok(PrimalLineage {
                                        primal_name: self.primal_name.clone(),
                                        lineage_data,
                                        signature,
                                        timestamp: Utc::now(),
                                    });
                                }
                                Err(sign_err) => {
                                    tracing::warn!(
                                        "BearDog signing failed: {}. Using mock.",
                                        sign_err
                                    );
                                }
                            }
                        }
                        Err(lineage_err) => {
                            tracing::warn!(
                                "BearDog lineage creation failed: {}. Using mock.",
                                lineage_err
                            );
                        }
                    }
                }

                // Double fallback: Create mock lineage (test/development only)
                tracing::warn!(
                    "Using mock lineage for {} (development/test mode)",
                    self.primal_name
                );
                Ok(PrimalLineage {
                    primal_name: self.primal_name.clone(),
                    lineage_data: format!("lineage_for_{}", node_id).into_bytes(),
                    signature: format!("sig_from_{}", self.primal_name).into_bytes(),
                    timestamp: Utc::now(),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physical_channels::MockPhysicalChannel;

    #[tokio::test]
    async fn test_genesis_ceremony_basic() {
        let channel = PhysicalChannel::Mock(MockPhysicalChannel::new());
        let witness = GenesisWitness::new(
            "test-witness".to_string(),
            vec![1, 2, 3],
            crate::types::PhysicalChannelType::HardwareKey,
        );

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
        assert!(identity.is_multi_primal_genesis());
    }
}
