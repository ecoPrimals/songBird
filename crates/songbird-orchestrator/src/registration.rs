//! Node Registration with Genetic Lineage
//!
//! Extends node registration to include cryptographic lineage information.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use songbird_types::{LineageId, LineageProof};
use std::collections::HashMap;
use tracing::info;

/// Node registration information with genetic lineage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeRegistration {
    /// Node identifier
    pub node_id: String,
    
    /// Human-readable node name
    pub node_name: String,
    
    /// Advertised capabilities
    pub capabilities: Vec<String>,
    
    /// Network endpoint
    pub endpoint: String,
    
    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    
    /// Genetic lineage identifier (NEW)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genetic_lineage: Option<LineageId>,
    
    /// Cryptographic lineage proof (NEW)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lineage_proof: Option<LineageProof>,
    
    /// Registration timestamp
    pub registered_at: u64,
    
    /// Time-to-live (seconds)
    pub ttl: u64,
}

impl NodeRegistration {
    /// Create a new node registration
    pub fn new(
        node_id: impl Into<String>,
        node_name: impl Into<String>,
        capabilities: Vec<String>,
        endpoint: impl Into<String>,
    ) -> Self {
        Self {
            node_id: node_id.into(),
            node_name: node_name.into(),
            capabilities,
            endpoint: endpoint.into(),
            metadata: HashMap::new(),
            genetic_lineage: None,
            lineage_proof: None,
            registered_at: Self::current_timestamp(),
            ttl: 300, // 5 minutes default
        }
    }

    /// Create registration with genetic lineage
    pub fn with_lineage(
        node_id: impl Into<String>,
        node_name: impl Into<String>,
        capabilities: Vec<String>,
        endpoint: impl Into<String>,
        lineage_id: LineageId,
        proof: LineageProof,
    ) -> Self {
        let mut registration = Self::new(node_id, node_name, capabilities, endpoint);
        registration.genetic_lineage = Some(lineage_id);
        registration.lineage_proof = Some(proof);
        registration
    }

    /// Set genetic lineage
    pub fn set_lineage(&mut self, lineage_id: LineageId, proof: LineageProof) {
        self.genetic_lineage = Some(lineage_id);
        self.lineage_proof = Some(proof);
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Set TTL
    pub fn with_ttl(mut self, ttl: u64) -> Self {
        self.ttl = ttl;
        self
    }

    /// Check if registration has lineage
    pub fn has_lineage(&self) -> bool {
        self.genetic_lineage.is_some() && self.lineage_proof.is_some()
    }

    /// Check if registration is expired
    pub fn is_expired(&self) -> bool {
        let now = Self::current_timestamp();
        now - self.registered_at > self.ttl
    }

    /// Refresh registration timestamp
    pub fn refresh(&mut self) {
        self.registered_at = Self::current_timestamp();
    }

    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

/// Helper to create registration from NodeIdentity
pub async fn create_registration_from_identity(
    identity: &crate::node_identity::NodeIdentity,
    endpoint: String,
    capabilities: Vec<String>,
) -> Result<NodeRegistration> {
    let node_id = identity.node_id.to_string();
    let node_name = identity.node_name.clone();

    let mut registration = NodeRegistration::new(node_id, node_name, capabilities, endpoint);

    // Add lineage if available
    if let Some((lineage_id, proof)) = identity.get_lineage() {
        registration.set_lineage(lineage_id.clone(), proof.clone());
        info!("✅ Registration includes genetic lineage: {}", lineage_id);
    } else {
        info!("ℹ️  Registration without lineage - BearDog not yet initialized");
    }

    Ok(registration)
}

/// Registration manager that handles lineage-aware registration
pub struct RegistrationManager {
    /// Current registration
    current: Option<NodeRegistration>,
    
    /// Refresh interval (seconds)
    refresh_interval: u64,
}

impl RegistrationManager {
    /// Create a new registration manager
    pub fn new(refresh_interval: u64) -> Self {
        Self {
            current: None,
            refresh_interval,
        }
    }

    /// Register node with lineage
    pub fn register(&mut self, registration: NodeRegistration) {
        info!(
            "📝 Registering node: {} with {} capabilities{}",
            registration.node_name,
            registration.capabilities.len(),
            if registration.has_lineage() { " and genetic lineage" } else { "" }
        );
        
        self.current = Some(registration);
    }

    /// Get current registration
    pub fn current(&self) -> Option<&NodeRegistration> {
        self.current.as_ref()
    }

    /// Check if registration needs refresh
    pub fn needs_refresh(&self) -> bool {
        if let Some(registration) = &self.current {
            let now = NodeRegistration::current_timestamp();
            let age = now - registration.registered_at;
            age >= self.refresh_interval
        } else {
            false
        }
    }

    /// Refresh current registration
    pub fn refresh(&mut self) {
        if let Some(registration) = &mut self.current {
            registration.refresh();
            info!("🔄 Refreshed node registration");
        }
    }

    /// Update lineage in current registration
    pub fn update_lineage(&mut self, lineage_id: LineageId, proof: LineageProof) -> Result<()> {
        let registration = self.current.as_mut()
            .context("No active registration to update")?;
        
        registration.set_lineage(lineage_id.clone(), proof);
        info!("🧬 Updated registration with genetic lineage: {}", lineage_id);
        
        Ok(())
    }
}

impl Default for RegistrationManager {
    fn default() -> Self {
        Self::new(60) // 1 minute refresh interval
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use songbird_types::LineageProof;

    #[test]
    fn test_registration_creation() {
        let reg = NodeRegistration::new(
            "node-123",
            "test-node",
            vec!["compute".to_string()],
            "http://localhost:8080",
        );

        assert_eq!(reg.node_id, "node-123");
        assert_eq!(reg.node_name, "test-node");
        assert!(!reg.has_lineage());
    }

    #[test]
    fn test_registration_with_lineage() {
        let lineage_id = LineageId::new("lineage:tower1:2026:abc");
        let proof = LineageProof::new(lineage_id.clone(), vec![], 1234567890);

        let reg = NodeRegistration::with_lineage(
            "node-123",
            "test-node",
            vec!["compute".to_string()],
            "http://localhost:8080",
            lineage_id,
            proof,
        );

        assert!(reg.has_lineage());
    }

    #[test]
    fn test_registration_expiration() {
        let mut reg = NodeRegistration::new(
            "node-123",
            "test-node",
            vec!["compute".to_string()],
            "http://localhost:8080",
        );

        // Set old timestamp
        reg.registered_at = 0;
        reg.ttl = 60;

        assert!(reg.is_expired());
    }

    #[test]
    fn test_registration_manager() {
        let mut manager = RegistrationManager::new(60);
        
        assert!(manager.current().is_none());
        
        let reg = NodeRegistration::new(
            "node-123",
            "test-node",
            vec!["compute".to_string()],
            "http://localhost:8080",
        );
        
        manager.register(reg);
        assert!(manager.current().is_some());
    }
}

