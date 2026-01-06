//! Self Knowledge Module
//!
//! This module contains ONLY what this primal knows about itself.
//! NO hardcoded knowledge of other primals - discovery is done at runtime!
//!
//! ## Zero Hardcoding Philosophy
//!
//! - **Self-Identity**: Node ID, hostname, interfaces
//! - **Self-Capabilities**: What we can DO (not what others provide!)
//! - **Self-Config**: Our ports, our settings
//! - **Discovery**: Runtime discovery for EVERYTHING else
//!
//! ## What This Primal Knows
//!
//! ```text
//! ✅ I am Songbird
//! ✅ I provide: discovery, orchestration, federation
//! ✅ I listen on: [ports from environment or auto-select]
//! ✅ I have interfaces: [detected at runtime]
//!
//! ❌ I don't know about BearDog (discover it!)
//! ❌ I don't know about Toadstool (discover it!)
//! ❌ I don't know about k8s (discover it!)
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Self-knowledge about this primal
///
/// Contains ONLY what we know about ourselves.
/// Everything else is discovered at runtime!
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfKnowledge {
    /// Our unique node ID (generated or loaded from persistent storage)
    pub node_id: Uuid,
    
    /// Our hostname (detected at startup)
    pub hostname: String,
    
    /// Our primal type (what we ARE, not what we KNOW ABOUT)
    pub primal_type: PrimalType,
    
    /// Capabilities we provide (what we can DO)
    pub capabilities: Vec<String>,
    
    /// Our network interfaces (detected at runtime)
    pub interfaces: Vec<NetworkInterface>,
    
    /// Our listening endpoints (ports from env or auto-selected)
    pub endpoints: Vec<SelfEndpoint>,
    
    /// Environment-provided configuration
    pub environment: EnvironmentConfig,
}

/// What primal type we are
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimalType {
    /// Discovery and orchestration primal
    Orchestrator,
    /// Security and cryptography primal (discovered, not hardcoded!)
    Security,
    /// Storage primal
    Storage,
    /// Compute primal
    Compute,
    /// Networking primal
    Networking,
    /// Custom primal type
    Custom(String),
}

/// Network interface on this machine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    /// Interface name (e.g., "eth0", "wlan0")
    pub name: String,
    /// IP addresses on this interface
    pub addresses: Vec<IpAddr>,
    /// Interface flags (UP, RUNNING, etc.)
    pub flags: Vec<String>,
    /// MTU
    pub mtu: Option<u32>,
}

/// Our listening endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfEndpoint {
    /// What protocol we're listening on
    pub protocol: String,
    /// Socket address we're bound to
    pub socket_addr: SocketAddr,
    /// Is this the primary endpoint?
    pub is_primary: bool,
}

/// Environment-provided configuration
///
/// All values come from environment variables or runtime detection.
/// NO hardcoded values!
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    /// Log level (from RUST_LOG or default)
    pub log_level: String,
    
    /// Data directory (from env or default: ~/.local/share/songbird)
    pub data_dir: String,
    
    /// Config directory (from env or default: ~/.config/songbird)
    pub config_dir: String,
    
    /// Environment type (dev, staging, production)
    pub environment_type: String,
    
    /// Additional environment variables relevant to this primal
    pub extra: HashMap<String, String>,
}

impl SelfKnowledge {
    /// Discover our self-knowledge at startup
    ///
    /// This is the ONLY place we "know" things - everything is detected!
    ///
    /// # Errors
    ///
    /// Returns error if we can't determine basic facts about ourselves
    /// (hostname, network interfaces, etc.)
    pub async fn discover() -> Result<Self> {
        info!("🔍 Discovering self-knowledge (zero hardcoding!)");
        
        // Generate or load our node ID
        let node_id = Self::discover_node_id()?;
        
        // Detect hostname
        let hostname = Self::discover_hostname()?;
        
        // Detect our primal type (from env or binary name)
        let primal_type = Self::discover_primal_type()?;
        
        // Detect our capabilities (what we can DO)
        let capabilities = Self::discover_capabilities(&primal_type)?;
        
        // Detect network interfaces
        let interfaces = Self::discover_interfaces().await?;
        
        // Detect or select listening endpoints
        let endpoints = Self::discover_endpoints(&interfaces).await?;
        
        // Load environment configuration
        let environment = Self::discover_environment()?;
        
        info!("✅ Self-knowledge discovered: {} ({})", hostname, node_id);
        debug!("   Primal type: {:?}", primal_type);
        debug!("   Capabilities: {:?}", capabilities);
        debug!("   Interfaces: {} detected", interfaces.len());
        debug!("   Endpoints: {} configured", endpoints.len());
        
        Ok(Self {
            node_id,
            hostname,
            primal_type,
            capabilities,
            interfaces,
            endpoints,
            environment,
        })
    }
    
    /// Discover our node ID (generate or load from persistent storage)
    ///
    /// This ensures stable node identity across restarts and supports multi-instance
    /// deployment (each instance gets its own ID based on NODE_ID env var).
    ///
    /// Identity file path: `/var/lib/songbird/identity-{NODE_ID}.json` (or custom via SONGBIRD_IDENTITY_PATH)
    fn discover_node_id() -> Result<Uuid> {
        use std::fs;
        use std::path::PathBuf;

        // Get identity file path (unique per NODE_ID for multi-instance support)
        let node_env = std::env::var("NODE_ID").unwrap_or_else(|_| "default".to_string());
        let identity_path = std::env::var("SONGBIRD_IDENTITY_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                PathBuf::from(format!("/var/lib/songbird/identity-{}.json", node_env))
            });

        // Try to load existing node ID
        if identity_path.exists() {
            match fs::read_to_string(&identity_path) {
                Ok(content) => {
                    if let Ok(stored_id) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(id_str) = stored_id.get("node_id").and_then(|v| v.as_str()) {
                            if let Ok(node_id) = Uuid::parse_str(id_str) {
                                debug!("✅ Loaded node ID from {}: {}", identity_path.display(), node_id);
                                return Ok(node_id);
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("⚠️  Failed to read identity file {}: {}", identity_path.display(), e);
                }
            }
        }

        // Generate new node ID (stable per NODE_ID for deterministic behavior)
        let node_id = if node_env == "default" {
            // Default: pure random UUID
            Uuid::new_v4()
        } else {
            // Multi-instance: deterministic UUID from NODE_ID for reproducibility
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(node_env.as_bytes());
            hasher.update(b"songbird-node-id");
            let hash = hasher.finalize();
            
            // Create UUID v4 from hash (deterministic but looks random)
            Uuid::from_slice(&hash[0..16])
                .unwrap_or_else(|_| Uuid::new_v4())
        };

        debug!("🆕 Generated new node ID: {}", node_id);

        // Try to persist for future restarts
        if let Some(parent) = identity_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                debug!("⚠️  Could not create identity directory: {}", e);
            }
        }

        let identity_data = serde_json::json!({
            "node_id": node_id.to_string(),
            "created_at": chrono::Utc::now().to_rfc3339(),
            "node_env": node_env,
        });

        if let Err(e) = fs::write(&identity_path, serde_json::to_string_pretty(&identity_data)?) {
            debug!("⚠️  Could not persist identity to {}: {}", identity_path.display(), e);
            debug!("   (This is non-fatal - identity will regenerate on restart)");
        } else {
            debug!("💾 Persisted node ID to {}", identity_path.display());
        }

        Ok(node_id)
    }
    
    /// Discover our hostname
    fn discover_hostname() -> Result<String> {
        let hostname = hostname::get()
            .context("Failed to get hostname")?
            .to_string_lossy()
            .to_string();
        
        debug!("Detected hostname: {}", hostname);
        Ok(hostname)
    }
    
    /// Discover what primal type we are
    fn discover_primal_type() -> Result<PrimalType> {
        // Check environment variable first
        if let Ok(primal_type) = std::env::var("PRIMAL_TYPE") {
            return match primal_type.to_lowercase().as_str() {
                "orchestrator" => Ok(PrimalType::Orchestrator),
                "security" => Ok(PrimalType::Security),
                "storage" => Ok(PrimalType::Storage),
                "compute" => Ok(PrimalType::Compute),
                "networking" => Ok(PrimalType::Networking),
                custom => Ok(PrimalType::Custom(custom.to_string())),
            };
        }
        
        // Default: Orchestrator (since we're Songbird)
        Ok(PrimalType::Orchestrator)
    }
    
    /// Discover our capabilities (what we can DO)
    fn discover_capabilities(primal_type: &PrimalType) -> Result<Vec<String>> {
        let mut capabilities = Vec::new();
        
        // Base capabilities by primal type
        match primal_type {
            PrimalType::Orchestrator => {
                capabilities.extend_from_slice(&[
                    "discovery".to_string(),
                    "orchestration".to_string(),
                    "federation".to_string(),
                    "peer-management".to_string(),
                ]);
            },
            PrimalType::Security => {
                capabilities.extend_from_slice(&[
                    "identity".to_string(),
                    "encryption".to_string(),
                    "trust-evaluation".to_string(),
                ]);
            },
            PrimalType::Storage => {
                capabilities.push("storage".to_string());
            },
            PrimalType::Compute => {
                capabilities.push("compute".to_string());
            },
            PrimalType::Networking => {
                capabilities.push("networking".to_string());
            },
            PrimalType::Custom(name) => {
                capabilities.push(name.clone());
            },
        }
        
        // Additional capabilities from environment
        if let Ok(extra_caps) = std::env::var("ADDITIONAL_CAPABILITIES") {
            capabilities.extend(extra_caps.split(',').map(|s| s.trim().to_string()));
        }
        
        debug!("Discovered capabilities: {:?}", capabilities);
        Ok(capabilities)
    }
    
    /// Discover network interfaces
    async fn discover_interfaces() -> Result<Vec<NetworkInterface>> {
        let mut interfaces = Vec::new();
        
        // Use system APIs to detect interfaces
        for iface in netdev::get_interfaces() {
            let addresses: Vec<IpAddr> = iface
                .ipv4
                .iter()
                .map(|ip| IpAddr::V4(ip.addr()))
                .chain(iface.ipv6.iter().map(|ip| IpAddr::V6(ip.addr())))
                .collect();
            
            if addresses.is_empty() {
                continue; // Skip interfaces with no IPs
            }
            
            // Parse interface flags from netdev
            let mut flags = Vec::new();
            if iface.is_up() {
                flags.push("UP".to_string());
            }
            if iface.is_running() {
                flags.push("RUNNING".to_string());
            }
            if iface.is_loopback() {
                flags.push("LOOPBACK".to_string());
            }
            if iface.is_multicast() {
                flags.push("MULTICAST".to_string());
            }
            
            // Get MTU from interface (netdev provides this as Option<u32>)
            let mtu = iface.mtu;
            
            interfaces.push(NetworkInterface {
                name: iface.name.clone(),
                addresses,
                flags,
                mtu,
            });
        }
        
        debug!("Discovered {} network interfaces", interfaces.len());
        Ok(interfaces)
    }
    
    /// Discover or select listening endpoints
    async fn discover_endpoints(_interfaces: &[NetworkInterface]) -> Result<Vec<SelfEndpoint>> {
        let mut endpoints = Vec::new();
        
        // Check for port from environment
        let http_port = std::env::var("HTTP_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(0); // 0 = auto-select
        
        let rpc_port = std::env::var("RPC_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(0); // 0 = auto-select
        
        // HTTP endpoint
        endpoints.push(SelfEndpoint {
            protocol: "http".to_string(),
            socket_addr: SocketAddr::from(([0, 0, 0, 0], http_port)),
            is_primary: true,
        });
        
        // RPC endpoint
        endpoints.push(SelfEndpoint {
            protocol: "rpc".to_string(),
            socket_addr: SocketAddr::from(([0, 0, 0, 0], rpc_port)),
            is_primary: false,
        });
        
        info!("Configured {} endpoints (0 = auto-select)", endpoints.len());
        Ok(endpoints)
    }
    
    /// Discover environment configuration
    fn discover_environment() -> Result<EnvironmentConfig> {
        Ok(EnvironmentConfig {
            log_level: std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
            data_dir: std::env::var("DATA_DIR").unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                format!("{}/.local/share/songbird", home)
            }),
            config_dir: std::env::var("CONFIG_DIR").unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                format!("{}/.config/songbird", home)
            }),
            environment_type: std::env::var("ENVIRONMENT").unwrap_or_else(|_| "production".to_string()),
            extra: HashMap::new(),
        })
    }
    
    /// Get our primary endpoint
    #[must_use]
    pub fn primary_endpoint(&self) -> Option<&SelfEndpoint> {
        self.endpoints.iter().find(|e| e.is_primary)
    }
    
    /// Check if we provide a capability
    #[must_use]
    pub fn provides_capability(&self, capability: &str) -> bool {
        self.capabilities.iter().any(|c| c == capability)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_self_discovery() {
        let self_knowledge = SelfKnowledge::discover().await.unwrap();
        
        // We should know our node ID
        assert!(!self_knowledge.node_id.is_nil());
        
        // We should know our hostname
        assert!(!self_knowledge.hostname.is_empty());
        
        // We should know our capabilities
        assert!(!self_knowledge.capabilities.is_empty());
        
        // We should have endpoints
        assert!(!self_knowledge.endpoints.is_empty());
    }
    
    #[test]
    fn test_capability_check() {
        let self_knowledge = SelfKnowledge {
            node_id: Uuid::new_v4(),
            hostname: "test-node".to_string(),
            primal_type: PrimalType::Orchestrator,
            capabilities: vec!["discovery".to_string(), "orchestration".to_string()],
            interfaces: vec![],
            endpoints: vec![],
            environment: EnvironmentConfig {
                log_level: "info".to_string(),
                data_dir: "/tmp".to_string(),
                config_dir: "/tmp".to_string(),
                environment_type: "test".to_string(),
                extra: HashMap::new(),
            },
        };
        
        assert!(self_knowledge.provides_capability("discovery"));
        assert!(self_knowledge.provides_capability("orchestration"));
        assert!(!self_knowledge.provides_capability("storage"));
    }
}

