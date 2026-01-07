//! Node Identity Management
//!
//! Provides stable node identity across network changes, restarts, and interface changes.
//! A node's identity remains constant regardless of its transport paths.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use tracing::{debug, info, warn};
use uuid::Uuid;
use songbird_types::{LineageId, LineageProof};

/// Stable node identity with genetic lineage
///
/// This identity remains constant across:
/// - Network interface changes (WiFi → Ethernet)
/// - IP address changes (DHCP)
/// - Process restarts
/// - System reboots (persisted to disk)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeIdentity {
    /// Stable unique identifier for this node
    pub node_id: Uuid,

    /// Human-readable node name (hostname by default)
    pub node_name: String,

    /// All available transport endpoints for this node
    pub endpoints: Vec<TransportEndpoint>,

    /// Genetic lineage identifier (NEW)
    ///
    /// Cryptographic ancestry of this node, enabling automatic
    /// trust establishment with same-lineage peers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genetic_lineage: Option<LineageId>,

    /// Cryptographic lineage proof (NEW)
    ///
    /// Signature chain proving this node's ancestry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lineage_proof: Option<LineageProof>,
}

/// A transport endpoint for reaching this node
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TransportEndpoint {
    /// Interface type (e.g., "ethernet", "wifi", "bluetooth")
    pub interface_type: String,

    /// Network address for this endpoint
    pub address: SocketAddr,

    /// Supported protocols on this endpoint
    pub protocols: Vec<String>,

    /// Relative preference (higher = more preferred)
    pub preference: u8,
}

impl NodeIdentity {
    /// Create a new node identity
    ///
    /// This will either:
    /// 1. Load existing identity from disk (if available)
    /// 2. Generate new identity and persist it
    pub fn new_or_load(node_name: Option<String>) -> Result<Self> {
        // Try to load existing identity
        if let Ok(identity) = Self::load_from_disk() {
            info!("🆔 Loaded existing node identity: {}", identity.node_id);
            return Ok(identity);
        }

        // Generate new identity
        let node_id = Self::generate_stable_id()?;
        
        // CRITICAL FIX (Jan 5, 2026): Prefer SONGBIRD_NODE_ID over hostname
        // This ensures multi-instance deployments have unique discoverable names
        let node_name = node_name.unwrap_or_else(|| {
            std::env::var("SONGBIRD_NODE_ID")
                .or_else(|_| std::env::var("NODE_ID"))
                .ok()
                .or_else(|| {
                    hostname::get()
                        .ok()
                        .and_then(|h| h.into_string().ok())
                })
                .unwrap_or_else(|| "songbird-node".to_string())
        });

        let identity = Self {
            node_id,
            node_name,
            endpoints: Vec::new(),
            genetic_lineage: None,  // Will be set when security provider integration is ready
            lineage_proof: None,
        };

        // Persist to disk
        identity.save_to_disk()?;

        info!("🆔 Generated new node identity: {}", identity.node_id);
        Ok(identity)
    }

    /// Set genetic lineage for this node
    ///
    /// Updates the node identity with genetic lineage information from security provider.
    pub fn set_lineage(&mut self, lineage_id: LineageId, proof: LineageProof) -> Result<()> {
        self.genetic_lineage = Some(lineage_id.clone());
        self.lineage_proof = Some(proof);
        
        // Persist updated identity
        self.save_to_disk()?;
        
        info!("🧬 Updated node identity with genetic lineage: {}", lineage_id);
        Ok(())
    }

    /// Check if this node has genetic lineage
    pub fn has_lineage(&self) -> bool {
        self.genetic_lineage.is_some() && self.lineage_proof.is_some()
    }

    /// Get lineage information
    pub fn get_lineage(&self) -> Option<(&LineageId, &LineageProof)> {
        self.genetic_lineage.as_ref()
            .zip(self.lineage_proof.as_ref())
    }

    /// Generate a stable node ID
    ///
    /// CRITICAL FIX (Jan 5, 2026): Include NODE_ID in UUID generation
    /// to support multiple instances on same machine
    ///
    /// Strategy (in order of preference):
    /// 1. Use /etc/machine-id + NODE_ID (Linux standard, multi-instance)
    /// 2. Use /var/lib/dbus/machine-id + NODE_ID (systemd)
    /// 3. Generate from MAC address + NODE_ID (fallback)
    /// 4. Generate random UUID and persist (last resort)
    fn generate_stable_id() -> Result<Uuid> {
        // Get NODE_ID if available for multi-instance support
        let node_id_suffix = std::env::var("SONGBIRD_NODE_ID")
            .or_else(|_| std::env::var("NODE_ID"))
            .ok();
        
        // Try machine-id (most stable)
        if let Ok(machine_id) = fs::read_to_string("/etc/machine-id") {
            let machine_id = machine_id.trim();
            if !machine_id.is_empty() {
                // CRITICAL: Include NODE_ID in hash for uniqueness
                let hash_input = if let Some(ref suffix) = node_id_suffix {
                    format!("{}:{}", machine_id, suffix)
                } else {
                    machine_id.to_string()
                };
                // Hash machine-id to UUID
                return Ok(Uuid::new_v5(&Uuid::NAMESPACE_DNS, hash_input.as_bytes()));
            }
        }

        // Try dbus machine-id
        if let Ok(machine_id) = fs::read_to_string("/var/lib/dbus/machine-id") {
            let machine_id = machine_id.trim();
            if !machine_id.is_empty() {
                let hash_input = if let Some(ref suffix) = node_id_suffix {
                    format!("{}:{}", machine_id, suffix)
                } else {
                    machine_id.to_string()
                };
                return Ok(Uuid::new_v5(&Uuid::NAMESPACE_DNS, hash_input.as_bytes()));
            }
        }

        // Try MAC address (less stable, but better than random)
        #[cfg(target_os = "linux")]
        {
            if let Ok(interfaces) = Self::get_mac_addresses() {
                if let Some(mac) = interfaces.first() {
                    let hash_input = if let Some(ref suffix) = node_id_suffix {
                        format!("{}:{}", mac, suffix)
                    } else {
                        mac.to_string()
                    };
                    return Ok(Uuid::new_v5(&Uuid::NAMESPACE_DNS, hash_input.as_bytes()));
                }
            }
        }

        // Last resort: random UUID (will be persisted)
        warn!("⚠️  Could not determine stable machine ID, using random UUID");
        warn!("   This ID will change if identity file is deleted");
        Ok(Uuid::new_v4())
    }

    /// Get MAC addresses from network interfaces
    #[cfg(target_os = "linux")]
    fn get_mac_addresses() -> Result<Vec<String>> {
        use std::process::Command;

        let output = Command::new("ip").args(["link", "show"]).output()?;

        let output = String::from_utf8_lossy(&output.stdout);
        let macs: Vec<String> = output
            .lines()
            .filter(|line| line.contains("link/ether"))
            .filter_map(|line| line.split_whitespace().nth(1).map(|mac| mac.to_string()))
            .collect();

        Ok(macs)
    }

    /// Path to identity file
    fn identity_path() -> PathBuf {
        let data_dir = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        
        // CRITICAL FIX (Jan 5, 2026): Support multiple instances on same machine
        // Use SONGBIRD_NODE_ID or NODE_ID to create unique identity files
        let filename = std::env::var("SONGBIRD_NODE_ID")
            .or_else(|_| std::env::var("NODE_ID"))
            .ok()
            .map(|node_id| format!("node_identity-{}.json", node_id))
            .unwrap_or_else(|| "node_identity.json".to_string());
        
        data_dir.join("songbird").join(filename)
    }

    /// Save identity to disk
    fn save_to_disk(&self) -> Result<()> {
        let path = Self::identity_path();

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self)?;
        fs::write(&path, json)?;

        debug!("💾 Saved node identity to: {}", path.display());
        Ok(())
    }

    /// Load identity from disk
    fn load_from_disk() -> Result<Self> {
        let path = Self::identity_path();
        let json = fs::read_to_string(&path)?;
        let identity: NodeIdentity = serde_json::from_str(&json)?;
        Ok(identity)
    }

    /// Add or update a transport endpoint
    pub fn add_endpoint(&mut self, endpoint: TransportEndpoint) {
        // Remove existing endpoint with same address
        self.endpoints.retain(|e| e.address != endpoint.address);

        // Add new endpoint
        self.endpoints.push(endpoint);

        // Sort by preference (highest first)
        self.endpoints.sort_by(|a, b| b.preference.cmp(&a.preference));
    }

    /// Detect all available network interfaces and populate endpoints
    ///
    /// This scans the system for all active network interfaces and creates
    /// transport endpoints for each. It assigns preferences based on interface type:
    /// - Ethernet: 100 (highest preference)
    /// - WiFi: 80
    /// - Other: 50
    /// - Loopback: 10 (lowest preference)
    pub fn detect_all_endpoints(&mut self, port: u16) -> Result<()> {
        info!("🔍 Detecting network interfaces...");

        // Get all network interfaces
        let interfaces = if_addrs::get_if_addrs()
            .map_err(|e| anyhow!("Failed to enumerate network interfaces: {}", e))?;

        let mut detected_count = 0;

        for iface in interfaces {
            // Skip loopback by default (can be enabled explicitly)
            if iface.is_loopback() {
                continue;
            }

            // Determine interface type and preference
            let (interface_type, preference) = Self::classify_interface(&iface.name);

            // Create endpoint
            let address = match iface.addr {
                if_addrs::IfAddr::V4(addr) => SocketAddr::new(IpAddr::V4(addr.ip), port),
                if_addrs::IfAddr::V6(addr) => SocketAddr::new(IpAddr::V6(addr.ip), port),
            };

            let endpoint = TransportEndpoint {
                interface_type: interface_type.clone(),
                address,
                protocols: vec!["https".to_string(), "tarpc".to_string()],
                preference,
            };

            self.add_endpoint(endpoint);
            detected_count += 1;

            info!(
                "  ✅ {} ({}) - {} [preference: {}]",
                iface.name, interface_type, address, preference
            );
        }

        info!("🔍 Detected {} network endpoints", detected_count);

        Ok(())
    }

    /// Classify network interface by name
    ///
    /// Returns (interface_type, preference)
    fn classify_interface(name: &str) -> (String, u8) {
        let name_lower = name.to_lowercase();

        // Ethernet interfaces
        if name_lower.starts_with("eth")
            || name_lower.starts_with("en")
            || name_lower.starts_with("ens")
            || name_lower.starts_with("enp")
        {
            return ("ethernet".to_string(), 100);
        }

        // WiFi interfaces
        if name_lower.starts_with("wlan")
            || name_lower.starts_with("wl")
            || name_lower.starts_with("wifi")
        {
            return ("wifi".to_string(), 80);
        }

        // Loopback
        if name_lower.starts_with("lo") {
            return ("loopback".to_string(), 10);
        }

        // Unknown/Other
        ("other".to_string(), 50)
    }

    /// Get preferred endpoint (highest preference)
    pub fn preferred_endpoint(&self) -> Option<&TransportEndpoint> {
        self.endpoints.first()
    }

    /// Get all IP addresses for this node
    pub fn all_addresses(&self) -> Vec<IpAddr> {
        self.endpoints.iter().map(|e| e.address.ip()).collect()
    }
}

impl Default for NodeIdentity {
    fn default() -> Self {
        Self::new_or_load(None).expect("Failed to create node identity")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stable_id_generation() {
        let id1 = NodeIdentity::generate_stable_id().unwrap();
        let id2 = NodeIdentity::generate_stable_id().unwrap();

        // Should generate same ID on same machine
        assert_eq!(id1, id2, "Stable ID should be consistent");
    }

    #[test]
    fn test_endpoint_management() {
        let mut identity = NodeIdentity {
            node_id: Uuid::new_v4(),
            node_name: "test".to_string(),
            endpoints: Vec::new(),
            genetic_lineage: None,
            lineage_proof: None,
        };

        // Add Ethernet endpoint
        identity.add_endpoint(TransportEndpoint {
            interface_type: "ethernet".to_string(),
            address: "192.168.1.144:8080".parse().unwrap(),
            protocols: vec!["https".to_string()],
            preference: 100,
        });

        // Add WiFi endpoint
        identity.add_endpoint(TransportEndpoint {
            interface_type: "wifi".to_string(),
            address: "192.168.1.185:8080".parse().unwrap(),
            protocols: vec!["https".to_string()],
            preference: 80,
        });

        // Should have 2 endpoints
        assert_eq!(identity.endpoints.len(), 2);

        // Preferred should be Ethernet (higher preference)
        let preferred = identity.preferred_endpoint().unwrap();
        assert_eq!(preferred.interface_type, "ethernet");
        assert_eq!(preferred.preference, 100);
    }

    #[test]
    fn test_endpoint_update() {
        let mut identity = NodeIdentity {
            node_id: Uuid::new_v4(),
            node_name: "test".to_string(),
            endpoints: Vec::new(),
            genetic_lineage: None,
            lineage_proof: None,
        };

        let addr: SocketAddr = "192.168.1.144:8080".parse().unwrap();

        // Add endpoint
        identity.add_endpoint(TransportEndpoint {
            interface_type: "ethernet".to_string(),
            address: addr,
            protocols: vec!["https".to_string()],
            preference: 100,
        });

        // Update same endpoint (new preference)
        identity.add_endpoint(TransportEndpoint {
            interface_type: "ethernet".to_string(),
            address: addr,
            protocols: vec!["https".to_string(), "tarpc".to_string()],
            preference: 90,
        });

        // Should still have 1 endpoint (updated, not duplicated)
        assert_eq!(identity.endpoints.len(), 1);

        // Should have new protocols
        let endpoint = &identity.endpoints[0];
        assert_eq!(endpoint.protocols.len(), 2);
        assert_eq!(endpoint.preference, 90);
    }
}
