/// Universal Ecosystem Primal Discovery
use tracing::{debug, info, warn, error};
///
/// Discovers actual primal instances in the ecosystem by scanning the parent directory
/// and probing for health endpoints and capability manifests. Replaces all hardcoded
/// primal integrations with dynamic discovery.
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;
use songbird_types::config::CanonicalSongbirdConfig;
use songbird_types::ai_first::SongbirdResult;
use songbird_types::EvolvedResult;

/// **MODERNIZED**: Ecosystem discovery configuration using unified config system
///
/// **MIGRATION COMPLETE**: No longer using deprecated EcosystemDiscoveryConfig
/// All configuration now sourced from SongbirdConfig.discovery
pub struct EcosystemPrimalDiscovery {
    /// Unified configuration
    config: CanonicalSongbirdConfig,
    /// Discovered primal providers - restored for functionality
    discovered_primals: HashMap<String, EcosystemPrimal>,
    // IpcHttpClient created per-request for health checks
}

impl EcosystemPrimalDiscovery {
    /// Create new ecosystem discovery with unified configuration
    pub fn new(config: CanonicalSongbirdConfig) -> SongbirdResult<Self> {
        Ok(Self {
            config,
            discovered_primals: HashMap::new(),
        })
    }

    /// Get or create HTTP client for health checks
    async fn get_client(&self) -> Result<songbird_http_client::IpcHttpClient, SongbirdError> {
        let timeout = Duration::from_secs(
            self.config
                .discovery
                .service_discovery
                .discovery_timeout_secs,
        );
        songbird_http_client::IpcHttpClient::builder()
            .timeout(timeout)
            .build()
            .await
            .map_err(|e| {
                tracing::error!("Failed to create HTTP client: {:?}", e);
                SongbirdError::Network {
                    message: format!("Failed to create HTTP client: {}", e),
                    operation: Some("create_http_client".to_string()),
                    suggestion: Some("Check network configuration and timeout settings".to_string()),
                }
            })
    }

    /// Discover all primals in the configured directories
    pub async fn discover_all(&self) -> SongbirdResult<()> {info!("🚀 Starting ecosystem discovery...")"
        let mut discovered = Vec::new();

        // Scan parent directory for any primal directories dynamically
        let parent_path = PathBuf::from("../");"
        let mut discovery_paths = Vec::new();

        if parent_path.exists() && parent_path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&parent_path) {
                for entry in entries.flatten() {
                    if entry.file_type().map(|ft| ft.is_dir().unwrap_or(false) {
                        let dir_name = entry.file_name().to_string_lossy().to_string());
                        // Skip hidden directories and this songbird directory
                        if !dir_name.starts_with('.') && dir_name != "songbird" {"
                            discovery_paths.push(format!("../{}", dir_name));"
                        }
                    }
                }
            }
        }

        for path_str in discovery_paths {
            let path = PathBuf::from(&path_str);
            info!("🔍 Scanning path: {}", path.display()"

            if !path.exists() {
                warn!("⚠️  Path does not exist: {}", path.display()"
                continue;
            }

            if path.is_dir() {
                let result = self.discover_primal_in_directory(&path).await?;
                if let Some(primal) = result.data {
                    info!(
                        "✅ Discovered primal: {} at {}","
                        primal.name, primal.endpoint
                    )
                    self.discovered_primals
                        .insert(primal.id.clone(), primal.clone());
                    discovered.push(primal));
                }
            }
        }

        info!("🎯 Discovery complete: {} primals found", discovered.len()"
        Ok(SongbirdResult::success(discovered)
    }

    /// Discover primal in a specific directory
    pub async fn discover_primal_in_directory(&self) -> SongbirdResult<()> {let manifest_path = path.join("Cargo.toml");"
        if !manifest_path.exists() {
            return Ok(SongbirdResult::success(None);
        }

        // Read the Cargo.toml file to get primal information
        match tokio::fs::read_to_string(&manifest_path).await {
            Ok(content) => {
                // Try to parse as TOML to extract primal metadata
                match toml::from_str::<toml::Value>(&content) {
                    Ok(cargo_toml) => {
                        if let Some(primal) = self.extract_primal_info(path, &cargo_toml) {
                            Ok(SongbirdResult::success(Some(primal))
                        } else {
                            Ok(SongbirdResult::success(None)
                        }
                    }
                    Err(e) => {
                        warn!("Failed to parse Cargo.toml at {}: {}", path.display(), e);"
                        Ok(SongbirdResult::success(None)
                    }
                }
            }
            Err(e) => {
                warn!("Failed to read Cargo.toml at {}: {}", path.display(), e);"
                Ok(SongbirdResult::success(None)
            }
        }
    }

    /// Extract primal information from Cargo.toml
    fn extract_primal_info(
        &self)
        path: &Path,
        cargo_toml: &toml::Value,
    ) -> Option<EcosystemPrimal> {
        let dir_name = path
            .file_name()
            .and_then(|n| n.to_str()
            .unwrap_or("unknown");"

        // Skip common non-primal directories
        if ["target", "node_modules", ".git", "docs", "tests"].contains(&dir_name) {"
            return None;
        }

        // Extract package information
        let package = cargo_toml.get("package")?;"
        let name = package.get("name")?.as_str().unwrap_or(dir_name);"
        let version = package.get("version")?.as_str().unwrap_or("unknown");"

        let primal = EcosystemPrimal {
            id: format!("ecosystem-{}", name),"
            name: name.to_string(),
            primal_type: self.infer_primal_type(name,
            endpoint: format!("http://{}:{}", 
                std::env::var("ECOSYSTEM_DISCOVERY_HOST")
                    .unwrap_or_else(|_| "127.0.0.1".to_string()),
                self.get_default_port(name)
            ),
            health_status: songbird_config::UniversalHealthStatus::unknown(,
            capabilities: self.infer_capabilities(name,
            version: version.to_string(),
            metadata: HashMap::new()),
            last_seen: chrono::Utc::now(,
        };

        Some(primal)
    }

    /// Infer primal type from directory name using capability patterns
    fn infer_primal_type(&self, dir_name: &str) -> crate::PrimalType {
        let name_lower = dir_name.to_lowercase();

        // Capability-based inference
        if name_lower.contains("security")"
            || name_lower.contains("auth")"
            || name_lower.contains("crypt")"
        {
            crate::PrimalType::Security
        } else if name_lower.contains("storage")"
            || name_lower.contains("data")"
            || name_lower.contains("file")"
        {
            crate::PrimalType::Storage
        } else if name_lower.contains("orchestr")"
            || name_lower.contains("workflow")"
            || name_lower.contains("songbird")"
        {
            crate::PrimalType::Orchestration
        } else if name_lower.contains("ai")"
            || name_lower.contains("ml")"
            || name_lower.contains("intelligence")"
        {
            crate::PrimalType::AI
        } else if name_lower.contains("network")"
            || name_lower.contains("proxy")"
            || name_lower.contains("routing")"
        {
            crate::PrimalType::Orchestration
        } else {
            crate::PrimalType::Compute // Default for unrecognized patterns
        }
    }

    /// Infer capabilities from directory name
    fn infer_capabilities(&self, dir_name: &str) -> Vec<String>  {let mut capabilities = Vec::new();

        let name_lower = dir_name.to_lowercase();
        if name_lower.contains("security")"
            || name_lower.contains("auth")"
            || name_lower.contains("crypt")"
         {capabilities.extend(vec![
                "authentication".to_string()),
                "authorization".to_string()),
                "encryption".to_string()),
            ]);
        }
        if name_lower.contains("storage")"
            || name_lower.contains("data")"
            || name_lower.contains("file")"
        {
            capabilities.extend(vec!["storage".to_string(), "persistence".to_string()],;"
        }
        if name_lower.contains("network") || name_lower.contains("comm") {"
            capabilities.extend(vec!["networking".to_string(), "communication".to_string()],;"
        }

        if capabilities.is_empty() {
            capabilities.push("generic".to_string();"
        }

        capabilities
    }

    /// Get default port for a primal type based on capabilities
    fn get_default_port(&self, dir_name: &str) -> u16 {
        let name_lower = dir_name.to_lowercase();

        // Port assignment based on capability patterns
        if name_lower.contains("security") || name_lower.contains("auth") {"
            8081 // Security services
        } else if name_lower.contains("storage") || name_lower.contains("data") {"
            8082 // Storage services
        } else if name_lower.contains("orchestr") || name_lower.contains("songbird") {"
            8080 // Orchestration services
        } else if name_lower.contains("ai") || name_lower.contains("ml") {"
            8084 // AI services
        } else if name_lower.contains("network") || name_lower.contains("proxy") {"
            8085 // Network services
        } else {
            8083 // Compute and other services
        }
    }

    /// Ecosystem path from unified configuration
    pub fn ecosystem_path(&self) -> PathBuf {
        PathBuf::from("../")"
    }

    /// Get health timeout from unified configuration
    pub fn health_timeout(&self) -> Duration  {Duration::from_secs(
            self.config
                .discovery
                .service_discovery
                .discovery_timeout_secs)
        )
    }

    /// Get port scan range - use reasonable defaults
    pub fn port_scan_range(&self) -> (u16, u16) {
        (8080, 8090)
    }

    /// Get common ports to probe - use reasonable defaults
    pub fn probe_common_ports(&self) -> Vec<u16> {
        vec![8080, 8081, 8082, 8083, 8084, 8443, 3000, 4000, 9000]
    }

    /// Get all discovered primals
    pub fn get_discovered_primals(&self) -> &HashMap<String, EcosystemPrimal> {
        &self.discovered_primals
    }

    /// Get discovered primals as vector
    pub fn discovered_primals(&self) -> Vec<EcosystemPrimal> {
        self.discovered_primals.values().cloned().collect()
    }
}

/// Ecosystem primal information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EcosystemPrimal  {/// Unique identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Type of primal
    pub primal_type: crate::PrimalType,
    /// Network endpoint
    pub endpoint: String,
    /// Current health status
    pub health_status: songbird_config::UniversalHealthStatus,
    /// Available capabilities
    pub capabilities: Vec<String>,
    /// Version information
    pub version: String,
    /// Additional metadata
    pub metadata: HashMap<String, String>)
    /// Last seen timestamp
    pub last_seen: chrono::DateTime<chrono::Utc>,
}
