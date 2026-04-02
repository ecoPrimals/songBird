// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

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

impl EnvironmentStrategy {
    /// Discover using an injectable env reader (e.g. for tests without mutating process env).
    pub async fn discover_with<F>(capability: &str, get_var: F) -> IpcResult<Vec<Provider>>
    where
        F: Fn(&str) -> Result<String, std::env::VarError> + Send,
    {
        debug!("🔍 [environment] Discovering {capability} providers...");

        let mut providers = Vec::new();

        // Strategy 1: {CAPABILITY}_PROVIDER_SOCKET (most specific)
        let env_var_socket = format!("{}_PROVIDER_SOCKET", capability.to_uppercase());
        if let Ok(socket_path) = get_var(&env_var_socket) {
            info!("   ✅ Found {}: {}", env_var_socket, socket_path);

            // Extract provider ID from path (e.g., /tmp/beardog.sock → beardog)
            let provider_id = extract_provider_id(&socket_path);

            let mut provider = Provider::new(
                provider_id.clone(),
                vec![capability.to_string()],
                format!("/primal/{provider_id}"),
            );
            provider.metadata.discovery_method = format!("env:{env_var_socket}");

            providers.push(provider);
        }

        // Strategy 2: {CAPABILITY}_PROVIDER (alternative)
        let env_var = format!("{}_PROVIDER", capability.to_uppercase());
        if let Ok(socket_path) = get_var(&env_var)
            && providers.is_empty()
        {
            // Only add if not already found
            info!("   ✅ Found {}: {}", env_var, socket_path);

            let provider_id = extract_provider_id(&socket_path);

            let mut provider = Provider::new(
                provider_id.clone(),
                vec![capability.to_string()],
                format!("/primal/{provider_id}"),
            );
            provider.metadata.discovery_method = format!("env:{env_var}");

            providers.push(provider);
        }

        // Strategy 3: Generic capability environment variable
        let generic_env = format!("{}_SOCKET", capability.to_uppercase());
        if let Ok(socket_path) = get_var(&generic_env)
            && providers.is_empty()
        {
            info!("   ✅ Found {}: {}", generic_env, socket_path);

            let provider_id = extract_provider_id(&socket_path);

            let mut provider = Provider::new(
                provider_id.clone(),
                vec![capability.to_string()],
                format!("/primal/{provider_id}"),
            );
            provider.metadata.discovery_method = format!("env:{generic_env}");

            providers.push(provider);
        }

        if providers.is_empty() {
            debug!("   ⏭️  No {capability} providers found via environment");
        }

        Ok(providers)
    }
}

#[async_trait]
impl DiscoveryStrategy for EnvironmentStrategy {
    fn name(&self) -> &'static str {
        "environment"
    }

    async fn discover(&self, capability: &str) -> IpcResult<Vec<Provider>> {
        Self::discover_with(capability, |k| songbird_process_env::var(k)).await
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
    #[must_use]
    pub fn new() -> Self {
        let mut search_paths = vec![PathBuf::from("/tmp")];

        // Add user runtime directory if available
        if let Ok(uid) = songbird_process_env::var("UID") {
            search_paths.push(PathBuf::from(format!("/run/user/{uid}")));
        }

        // Add system runtime directory
        search_paths.push(PathBuf::from("/var/run"));

        Self {
            search_paths,
        }
    }

    /// Create with custom search paths
    #[must_use]
    pub const fn with_paths(search_paths: Vec<PathBuf>) -> Self {
        Self {
            search_paths,
        }
    }
}

impl Default for FilesystemStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DiscoveryStrategy for FilesystemStrategy {
    fn name(&self) -> &'static str {
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
                        if file_name.contains(capability)
                            && file_name.to_lowercase().ends_with(".sock")
                        {
                            let path = entry.path();
                            info!("   ✅ Found {} socket at: {}", capability, path.display());

                            let provider_id = extract_provider_id(path.to_string_lossy().as_ref());

                            let mut provider = Provider::new(
                                provider_id.clone(),
                                vec![capability.to_string()],
                                format!("/primal/{provider_id}"),
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
    PathBuf::from(socket_path).file_stem().and_then(|s| s.to_str()).unwrap_or("unknown").to_string()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn test_extract_provider_id() {
        assert_eq!(extract_provider_id("/tmp/beardog.sock"), "beardog");
        assert_eq!(extract_provider_id("/tmp/beardog-nat0.sock"), "beardog-nat0");
        assert_eq!(extract_provider_id("/run/user/1000/crypto.sock"), "crypto");
        assert_eq!(extract_provider_id("nestgate.sock"), "nestgate");
    }

    #[tokio::test]
    async fn test_environment_strategy() {
        let providers = EnvironmentStrategy::discover_with("crypto", |k| {
            if k == "CRYPTO_PROVIDER_SOCKET" {
                Ok("/tmp/test-crypto.sock".to_string())
            } else {
                Err(std::env::VarError::NotPresent)
            }
        })
        .await
        .unwrap();

        assert!(!providers.is_empty());
        assert_eq!(providers[0].capabilities, vec!["crypto"]);
    }

    #[tokio::test]
    async fn test_filesystem_strategy() {
        let strategy = FilesystemStrategy::with_paths(vec![PathBuf::from("/tmp")]);

        // This test depends on system state, so we just verify it doesn't panic
        let result = strategy.discover("crypto").await;
        assert!(result.is_ok());
    }

    #[test]
    fn extract_provider_id_empty_path_uses_unknown() {
        assert_eq!(extract_provider_id(""), "unknown");
    }

    #[test]
    fn extract_provider_id_hidden_file() {
        assert_eq!(extract_provider_id("/tmp/.hidden.sock"), ".hidden");
    }

    #[tokio::test]
    async fn environment_strategy_prefers_provider_socket_over_provider() {
        let providers = EnvironmentStrategy::discover_with("ab", |k| match k {
            "AB_PROVIDER_SOCKET" => Ok("/tmp/first.sock".to_string()),
            "AB_PROVIDER" => Ok("/tmp/second.sock".to_string()),
            _ => Err(std::env::VarError::NotPresent),
        })
        .await
        .unwrap();
        assert_eq!(providers.len(), 1);
        assert!(providers[0].metadata.discovery_method.contains("PROVIDER_SOCKET"));
    }

    #[tokio::test]
    async fn environment_strategy_falls_back_to_capability_provider() {
        let providers = EnvironmentStrategy::discover_with("xy", |k| match k {
            "XY_PROVIDER_SOCKET" => Err(std::env::VarError::NotPresent),
            "XY_PROVIDER" => Ok("/run/xy.sock".to_string()),
            _ => Err(std::env::VarError::NotPresent),
        })
        .await
        .unwrap();
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].id, "xy");
    }

    #[tokio::test]
    async fn environment_strategy_third_tier_generic_socket() {
        let providers = EnvironmentStrategy::discover_with("zz", |k| match k {
            "ZZ_PROVIDER_SOCKET" | "ZZ_PROVIDER" => Err(std::env::VarError::NotPresent),
            "ZZ_SOCKET" => Ok("/var/zz.sock".to_string()),
            _ => Err(std::env::VarError::NotPresent),
        })
        .await
        .unwrap();
        assert_eq!(providers.len(), 1);
        assert!(providers[0].metadata.discovery_method.contains("ZZ_SOCKET"));
    }

    #[tokio::test]
    async fn environment_strategy_empty_when_no_vars() {
        let providers =
            EnvironmentStrategy::discover_with("none", |_| Err(std::env::VarError::NotPresent))
                .await
                .unwrap();
        assert!(providers.is_empty());
    }

    #[tokio::test]
    async fn filesystem_strategy_empty_paths_returns_empty() {
        let strategy = FilesystemStrategy::with_paths(vec![]);
        let providers = strategy.discover("anything").await.unwrap();
        assert!(providers.is_empty());
    }

    #[tokio::test]
    async fn discovery_strategy_environment_name() {
        let s = EnvironmentStrategy;
        assert_eq!(s.name(), "environment");
    }

    #[tokio::test]
    async fn discovery_strategy_filesystem_name() {
        let s = FilesystemStrategy::with_paths(vec![]);
        assert_eq!(s.name(), "filesystem");
    }

    #[test]
    fn filesystem_strategy_new_default_constructible() {
        let _ = FilesystemStrategy::new();
        let _ = FilesystemStrategy::default();
    }
}
