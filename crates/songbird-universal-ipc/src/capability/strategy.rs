//! Discovery strategies for finding capability providers

use crate::capability::provider::Provider;
use crate::error::IpcResult;
use async_trait::async_trait;
use std::path::PathBuf;
use tracing::{debug, info};

/// Discovery strategy trait
///
/// Each strategy implements a different method for discovering capability providers.
#[async_trait]
pub trait DiscoveryStrategy: Send + Sync {
    /// Name of this strategy (for logging)
    fn name(&self) -> &str;

    /// Discover providers offering the given capability
    ///
    /// # Arguments
    /// * `capability` - Capability to search for (e.g., "crypto", "storage")
    ///
    /// # Returns
    /// List of discovered providers (may be empty)
    async fn discover(&self, capability: &str) -> IpcResult<Vec<Provider>>;
}

/// Environment variable discovery strategy
///
/// Discovers providers via environment variables:
/// - `{CAPABILITY}_PROVIDER_SOCKET` - Specific capability provider
/// - `{CAPABILITY}_PROVIDER` - Alternative
/// - `CAPABILITY_PROVIDERS` - JSON map of capabilities to endpoints
pub struct EnvironmentStrategy;

#[async_trait]
impl DiscoveryStrategy for EnvironmentStrategy {
    fn name(&self) -> &str {
        "environment"
    }

    async fn discover(&self, capability: &str) -> IpcResult<Vec<Provider>> {
        debug!("🔍 [{}] Discovering {} providers...", self.name(), capability);

        let mut providers = Vec::new();

        // Strategy 1: {CAPABILITY}_PROVIDER_SOCKET (most specific)
        let env_var_socket = format!("{}_PROVIDER_SOCKET", capability.to_uppercase());
        if let Ok(socket_path) = std::env::var(&env_var_socket) {
            info!("   ✅ Found {}: {}", env_var_socket, socket_path);

            // Extract provider ID from path (e.g., /tmp/beardog.sock → beardog)
            let provider_id = extract_provider_id(&socket_path);

            let mut provider = Provider::new(
                provider_id.clone(),
                vec![capability.to_string()],
                format!("/primal/{}", provider_id),
            );
            provider.metadata.discovery_method = format!("env:{}", env_var_socket);

            providers.push(provider);
        }

        // Strategy 2: {CAPABILITY}_PROVIDER (alternative)
        let env_var = format!("{}_PROVIDER", capability.to_uppercase());
        if let Ok(socket_path) = std::env::var(&env_var) {
            if providers.is_empty() {
                // Only add if not already found
                info!("   ✅ Found {}: {}", env_var, socket_path);

                let provider_id = extract_provider_id(&socket_path);

                let mut provider = Provider::new(
                    provider_id.clone(),
                    vec![capability.to_string()],
                    format!("/primal/{}", provider_id),
                );
                provider.metadata.discovery_method = format!("env:{}", env_var);

                providers.push(provider);
            }
        }

        // Strategy 3: Generic capability environment variable
        let generic_env = format!("{}_SOCKET", capability.to_uppercase());
        if let Ok(socket_path) = std::env::var(&generic_env) {
            if providers.is_empty() {
                info!("   ✅ Found {}: {}", generic_env, socket_path);

                let provider_id = extract_provider_id(&socket_path);

                let mut provider = Provider::new(
                    provider_id.clone(),
                    vec![capability.to_string()],
                    format!("/primal/{}", provider_id),
                );
                provider.metadata.discovery_method = format!("env:{}", generic_env);

                providers.push(provider);
            }
        }

        if providers.is_empty() {
            debug!("   ⏭️  No {} providers found via environment", capability);
        }

        Ok(providers)
    }
}

/// Filesystem discovery strategy
///
/// Discovers providers by scanning common socket directories:
/// - `/tmp/` - Temporary sockets
/// - `/run/user/{uid}/` - User runtime directory
/// - `/var/run/` - System runtime directory
pub struct FilesystemStrategy {
    /// Directories to scan
    search_paths: Vec<PathBuf>,
}

impl FilesystemStrategy {
    /// Create new filesystem strategy with default search paths
    pub fn new() -> Self {
        let mut search_paths = vec![PathBuf::from("/tmp")];

        // Add user runtime directory if available
        if let Ok(uid) = std::env::var("UID") {
            search_paths.push(PathBuf::from(format!("/run/user/{}", uid)));
        }

        // Add system runtime directory
        search_paths.push(PathBuf::from("/var/run"));

        Self { search_paths }
    }

    /// Create with custom search paths
    pub fn with_paths(search_paths: Vec<PathBuf>) -> Self {
        Self { search_paths }
    }
}

impl Default for FilesystemStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DiscoveryStrategy for FilesystemStrategy {
    fn name(&self) -> &str {
        "filesystem"
    }

    async fn discover(&self, capability: &str) -> IpcResult<Vec<Provider>> {
        debug!("🔍 [{}] Discovering {} providers...", self.name(), capability);

        let mut providers = Vec::new();

        for search_path in &self.search_paths {
            if !search_path.exists() {
                continue;
            }

            if let Ok(entries) = std::fs::read_dir(search_path) {
                for entry in entries.flatten() {
                    if let Ok(file_name) = entry.file_name().into_string() {
                        // Look for sockets matching capability pattern
                        if file_name.contains(capability) && file_name.ends_with(".sock") {
                            let path = entry.path();
                            info!(
                                "   ✅ Found {} socket at: {}",
                                capability,
                                path.display()
                            );

                            let provider_id = extract_provider_id(path.to_string_lossy().as_ref());

                            let mut provider = Provider::new(
                                provider_id.clone(),
                                vec![capability.to_string()],
                                format!("/primal/{}", provider_id),
                            );
                            provider.metadata.discovery_method =
                                format!("filesystem:{}", search_path.display());

                            providers.push(provider);
                        }
                    }
                }
            }
        }

        if providers.is_empty() {
            debug!("   ⏭️  No {} providers found via filesystem", capability);
        }

        Ok(providers)
    }
}

/// Extract provider ID from socket path
///
/// Examples:
/// - `/tmp/beardog.sock` → `beardog`
/// - `/tmp/beardog-nat0.sock` → `beardog-nat0`
/// - `/run/user/1000/crypto.sock` → `crypto`
fn extract_provider_id(socket_path: &str) -> String {
    PathBuf::from(socket_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_provider_id() {
        assert_eq!(extract_provider_id("/tmp/beardog.sock"), "beardog");
        assert_eq!(
            extract_provider_id("/tmp/beardog-nat0.sock"),
            "beardog-nat0"
        );
        assert_eq!(extract_provider_id("/run/user/1000/crypto.sock"), "crypto");
        assert_eq!(extract_provider_id("nestgate.sock"), "nestgate");
    }

    #[tokio::test]
    async fn test_environment_strategy() {
        let strategy = EnvironmentStrategy;

        // Set test environment variable
        std::env::set_var("CRYPTO_PROVIDER_SOCKET", "/tmp/test-crypto.sock");

        let providers = strategy.discover("crypto").await.unwrap();

        assert!(!providers.is_empty());
        assert_eq!(providers[0].capabilities, vec!["crypto"]);

        // Cleanup
        std::env::remove_var("CRYPTO_PROVIDER_SOCKET");
    }

    #[tokio::test]
    async fn test_filesystem_strategy() {
        let strategy = FilesystemStrategy::with_paths(vec![PathBuf::from("/tmp")]);

        // This test depends on system state, so we just verify it doesn't panic
        let result = strategy.discover("crypto").await;
        assert!(result.is_ok());
    }
}

