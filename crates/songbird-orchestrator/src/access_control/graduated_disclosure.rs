//! Graduated Information Disclosure
//!
//! Implements information filtering based on trust level.
//! Shares only appropriate information at each trust level.
//!
//! ## Information Disclosure Rules
//!
//! ```text
//! Level 0 (Anonymous):
//!   ✅ Capabilities
//!   ✅ Protocols
//!   ❌ Identity, hostname, IP, topology
//!
//! Level 1 (Capability-Verified):
//!   ✅ Capabilities
//!   ✅ Protocols
//!   ✅ Role
//!   ❌ Hostname, IP, topology
//!
//! Level 2 (Role-Verified):
//!   ✅ Capabilities
//!   ✅ Protocols
//!   ✅ Role
//!   ✅ Service registry
//!   ❌ Hostname, IP (still anonymous)
//!
//! Level 3 (Identity-Verified):
//!   ✅ Capabilities
//!   ✅ Protocols
//!   ✅ Role
//!   ✅ Service registry
//!   ✅ Identity
//!   ✅ Hostname
//!   ❌ Internal IP (not yet)
//!
//! Level 4 (Hardware-Verified):
//!   ✅ EVERYTHING (full admin)
//!   ✅ Internal IP
//!   ✅ Topology
//!   ✅ Configuration
//! ```

use crate::trust::{TrustEscalationManager, TrustLevel};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Tower information with graduated disclosure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TowerInfo {
    /// Capabilities (shared at all levels)
    pub capabilities: Vec<String>,

    /// Supported protocols (shared at all levels)
    pub protocols: Vec<String>,

    /// Role (shared at Level 1+)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,

    /// Service registry (shared at Level 2+)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub services: Option<Vec<ServiceInfo>>,

    /// Identity (shared at Level 3+)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity: Option<TowerIdentity>,

    /// Hostname (shared at Level 3+)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,

    /// Internal IP (shared at Level 4 only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub internal_ip: Option<String>,

    /// Topology (shared at Level 4 only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topology: Option<TopologyInfo>,

    /// Configuration (shared at Level 4 only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
}

impl Default for TowerInfo {
    fn default() -> Self {
        Self {
            capabilities: Vec::new(),
            protocols: Vec::new(),
            role: None,
            services: None,
            identity: None,
            hostname: None,
            internal_ip: None,
            topology: None,
            config: None,
        }
    }
}

/// Service information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub service_type: String,
    pub status: String,
}

/// Tower identity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TowerIdentity {
    pub node_id: String,
    pub organization: Option<String>,
}

/// Topology information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyInfo {
    pub connected_towers: Vec<String>,
    pub network_segments: Vec<String>,
}

/// Graduated disclosure manager
pub struct GraduatedDisclosure {
    trust_manager: Arc<TrustEscalationManager>,
}

impl GraduatedDisclosure {
    /// Create a new graduated disclosure manager
    #[must_use]
    pub fn new(trust_manager: Arc<TrustEscalationManager>) -> Self {
        Self { trust_manager }
    }

    /// Get tower information based on trust level
    ///
    /// This is the main entry point for graduated information disclosure.
    /// It filters the tower information based on the requester's trust level.
    pub async fn get_tower_info(&self, session_id: &str, tower_id: &str) -> Result<TowerInfo> {
        let trust_level = self.trust_manager.get_trust_level(session_id).await?;

        match trust_level {
            TrustLevel::Anonymous => {
                // Share only capabilities, no identity
                Ok(TowerInfo {
                    capabilities: self.get_capabilities(tower_id).await?,
                    protocols: self.get_protocols(tower_id).await?,
                    ..Default::default()
                })
            }

            TrustLevel::CapabilityVerified => {
                // Share capabilities + role
                Ok(TowerInfo {
                    capabilities: self.get_capabilities(tower_id).await?,
                    protocols: self.get_protocols(tower_id).await?,
                    role: Some(self.get_role(tower_id).await?),
                    ..Default::default()
                })
            }

            TrustLevel::RoleVerified => {
                // Share capabilities + role + service registry
                Ok(TowerInfo {
                    capabilities: self.get_capabilities(tower_id).await?,
                    protocols: self.get_protocols(tower_id).await?,
                    role: Some(self.get_role(tower_id).await?),
                    services: Some(self.get_services(tower_id).await?),
                    ..Default::default()
                })
            }

            TrustLevel::IdentityVerified => {
                // Share capabilities + identity + hostname
                Ok(TowerInfo {
                    capabilities: self.get_capabilities(tower_id).await?,
                    protocols: self.get_protocols(tower_id).await?,
                    role: Some(self.get_role(tower_id).await?),
                    services: Some(self.get_services(tower_id).await?),
                    identity: Some(self.get_identity(tower_id).await?),
                    hostname: Some(self.get_hostname(tower_id).await?),
                    ..Default::default()
                })
            }

            TrustLevel::HardwareVerified => {
                // Share EVERYTHING (full admin)
                Ok(TowerInfo {
                    capabilities: self.get_capabilities(tower_id).await?,
                    protocols: self.get_protocols(tower_id).await?,
                    role: Some(self.get_role(tower_id).await?),
                    services: Some(self.get_services(tower_id).await?),
                    identity: Some(self.get_identity(tower_id).await?),
                    hostname: Some(self.get_hostname(tower_id).await?),
                    internal_ip: Some(self.get_internal_ip(tower_id).await?),
                    topology: Some(self.get_topology(tower_id).await?),
                    config: Some(self.get_config(tower_id).await?),
                })
            }
        }
    }

    // ========================================================================
    // Information Retrieval Methods (Placeholders)
    // ========================================================================
    //
    // These methods retrieve actual tower information.
    // In production, these would query the federation state, service registry, etc.
    // For now, they return placeholder data.

    async fn get_capabilities(&self, _tower_id: &str) -> Result<Vec<String>> {
        Ok(vec![
            "orchestration".to_string(),
            "gpu-compute".to_string(),
            "storage".to_string(),
        ])
    }

    async fn get_protocols(&self, _tower_id: &str) -> Result<Vec<String>> {
        Ok(vec![
            "https".to_string(),
            "tarpc-tls".to_string(),
            "websocket-tls".to_string(),
        ])
    }

    async fn get_role(&self, _tower_id: &str) -> Result<String> {
        Ok("compute-node".to_string())
    }

    async fn get_services(&self, _tower_id: &str) -> Result<Vec<ServiceInfo>> {
        Ok(vec![
            ServiceInfo {
                name: "orchestrator".to_string(),
                service_type: "core".to_string(),
                status: "healthy".to_string(),
            },
            ServiceInfo {
                name: "gpu-worker".to_string(),
                service_type: "compute".to_string(),
                status: "healthy".to_string(),
            },
        ])
    }

    async fn get_identity(&self, tower_id: &str) -> Result<TowerIdentity> {
        Ok(TowerIdentity {
            node_id: tower_id.to_string(),
            organization: Some("ecoPrimals".to_string()),
        })
    }

    async fn get_hostname(&self, tower_id: &str) -> Result<String> {
        Ok(format!("{}.local", tower_id))
    }

    async fn get_internal_ip(&self, _tower_id: &str) -> Result<String> {
        Ok("192.168.1.100".to_string())
    }

    async fn get_topology(&self, _tower_id: &str) -> Result<TopologyInfo> {
        Ok(TopologyInfo {
            connected_towers: vec!["tower-a".to_string(), "tower-b".to_string()],
            network_segments: vec!["lan".to_string(), "compute".to_string()],
        })
    }

    async fn get_config(&self, _tower_id: &str) -> Result<serde_json::Value> {
        Ok(serde_json::json!({
            "max_tasks": 100,
            "gpu_count": 2,
            "memory_gb": 64
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trust::types::CapabilityProof;
    use std::time::SystemTime;

    #[tokio::test]
    async fn test_graduated_disclosure_anonymous() {
        let trust_manager = Arc::new(TrustEscalationManager::with_defaults());
        let disclosure = GraduatedDisclosure::new(Arc::clone(&trust_manager));

        let session_id = "test-session".to_string();
        trust_manager.establish_anonymous(session_id.clone()).await.unwrap();

        let info = disclosure.get_tower_info(&session_id, "tower-a").await.unwrap();

        // Should have capabilities and protocols
        assert!(!info.capabilities.is_empty());
        assert!(!info.protocols.is_empty());

        // Should NOT have identity, hostname, or internal IP
        assert!(info.role.is_none());
        assert!(info.services.is_none());
        assert!(info.identity.is_none());
        assert!(info.hostname.is_none());
        assert!(info.internal_ip.is_none());
    }

    #[tokio::test]
    async fn test_graduated_disclosure_capability_verified() {
        let trust_manager = Arc::new(TrustEscalationManager::with_defaults());
        let disclosure = GraduatedDisclosure::new(Arc::clone(&trust_manager));

        let session_id = "test-session".to_string();
        trust_manager.establish_anonymous(session_id.clone()).await.unwrap();

        // Escalate to capability-verified
        let proof = CapabilityProof {
            capabilities: vec!["orchestration".to_string()],
            proof: "test-proof".to_string(),
            timestamp: SystemTime::now(),
        };
        trust_manager.verify_capabilities(&session_id, proof).await.unwrap();

        let info = disclosure.get_tower_info(&session_id, "tower-a").await.unwrap();

        // Should have capabilities, protocols, and role
        assert!(!info.capabilities.is_empty());
        assert!(!info.protocols.is_empty());
        assert!(info.role.is_some());

        // Should NOT have identity, hostname, or internal IP
        assert!(info.services.is_none());
        assert!(info.identity.is_none());
        assert!(info.hostname.is_none());
        assert!(info.internal_ip.is_none());
    }

    #[tokio::test]
    async fn test_graduated_disclosure_role_verified() {
        let trust_manager = Arc::new(TrustEscalationManager::with_defaults());
        let disclosure = GraduatedDisclosure::new(Arc::clone(&trust_manager));

        let session_id = "test-session".to_string();
        trust_manager.establish_anonymous(session_id.clone()).await.unwrap();

        // Escalate to capability-verified
        let proof = CapabilityProof {
            capabilities: vec!["orchestration".to_string()],
            proof: "test-proof".to_string(),
            timestamp: SystemTime::now(),
        };
        trust_manager.verify_capabilities(&session_id, proof).await.unwrap();

        // Escalate to role-verified
        trust_manager.verify_role(&session_id, "admin".to_string()).await.unwrap();

        let info = disclosure.get_tower_info(&session_id, "tower-a").await.unwrap();

        // Should have capabilities, protocols, role, and services
        assert!(!info.capabilities.is_empty());
        assert!(!info.protocols.is_empty());
        assert!(info.role.is_some());
        assert!(info.services.is_some());

        // Should NOT have identity, hostname, or internal IP
        assert!(info.identity.is_none());
        assert!(info.hostname.is_none());
        assert!(info.internal_ip.is_none());
    }
}

