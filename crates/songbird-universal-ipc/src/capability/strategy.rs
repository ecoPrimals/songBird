// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Discovery strategies for finding capability providers

use crate::capability::provider::Provider;
use crate::error::IpcResult;
use songbird_types::constants::SYSTEM_RUNTIME_DIR;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, info};

/// Discovery strategy (enum dispatch — environment, filesystem, or injected env map for tests).
#[derive(Clone)]
pub enum DiscoveryStrategy {
    Environment,
    Filesystem(FilesystemStrategy),
    #[cfg(test)]
    InjectedEnvironment(Arc<std::collections::HashMap<String, String>>),
}

impl DiscoveryStrategy {
    /// Name of this strategy (for logging)
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Environment => "environment",
            Self::Filesystem(_) => "filesystem",
            #[cfg(test)]
            Self::InjectedEnvironment(_) => "environment(injected)",
        }
    }

    /// Discover providers offering the given capability
    pub async fn discover(&self, capability: &str) -> IpcResult<Vec<Provider>> {
        match self {
            Self::Environment => {
                EnvironmentStrategy::discover_with(capability, |k| songbird_process_env::var(k))
                    .await
            }
            Self::Filesystem(fs) => fs.discover(capability).await,
            #[cfg(test)]
            Self::InjectedEnvironment(map) => {
                let map = Arc::clone(map);
                EnvironmentStrategy::discover_with(capability, move |k| {
                    map.get(k).cloned().ok_or(std::env::VarError::NotPresent)
                })
                .await
            }
        }
    }
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

            // Extract provider ID from path (e.g. .../songbird-test-security.sock → songbird-test-security)
            let provider_id = extract_provider_id(&socket_path);

            let mut provider = Provider::new(
                provider_id.as_ref(),
                vec![capability.to_string()],
                format!("/primal/{}", provider_id.as_ref()),
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
                provider_id.as_ref(),
                vec![capability.to_string()],
                format!("/primal/{}", provider_id.as_ref()),
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
                provider_id.as_ref(),
                vec![capability.to_string()],
                format!("/primal/{}", provider_id.as_ref()),
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

/// Filesystem discovery strategy
///
/// Discovers providers by scanning common socket directories:
/// - `/tmp/` - Temporary sockets
/// - `$XDG_RUNTIME_DIR` - User session runtime (when set)
/// - `/run/user/{uid}/` - User runtime directory (when `UID` is set)
/// - `/var/run/` - System runtime directory
#[derive(Clone)]
pub struct FilesystemStrategy {
    /// Directories to scan
    search_paths: Arc<Vec<PathBuf>>,
}

impl FilesystemStrategy {
    /// Create new filesystem strategy with default search paths
    #[must_use]
    pub fn new() -> Self {
        let mut search_paths = vec![std::env::temp_dir()];

        if let Ok(xdg) = songbird_process_env::var("XDG_RUNTIME_DIR") {
            search_paths.push(PathBuf::from(xdg));
        }

        // Add user runtime directory if available
        if let Ok(uid) = songbird_process_env::var("UID") {
            search_paths.push(PathBuf::from(format!(
                "{}/{uid}",
                songbird_types::constants::USER_RUNTIME_PREFIX
            )));
        }

        search_paths.push(PathBuf::from(SYSTEM_RUNTIME_DIR));

        Self {
            search_paths: Arc::new(search_paths),
        }
    }

    /// Create with custom search paths
    #[must_use]
    pub fn with_paths(search_paths: Vec<PathBuf>) -> Self {
        Self {
            search_paths: Arc::new(search_paths),
        }
    }

    pub async fn discover(&self, capability: &str) -> IpcResult<Vec<Provider>> {
        debug!("🔍 [{}] Discovering {} providers...", self.name(), capability);

        let search_paths = Arc::clone(&self.search_paths);
        let cap = capability.to_string();

        let providers = tokio::task::spawn_blocking(move || {
            discover_on_filesystem(search_paths.as_ref(), &cap)
        })
        .await
        .map_err(|e| crate::error::IpcError::Internal(format!("spawn_blocking join: {e}")))?;

        if providers.is_empty() {
            debug!("   ⏭️  No {} providers found via filesystem", capability);
        }

        Ok(providers)
    }

    fn name(&self) -> &'static str {
        "filesystem"
    }
}

impl Default for FilesystemStrategy {
    fn default() -> Self {
        Self::new()
    }
}

fn discover_on_filesystem(search_paths: &[PathBuf], capability: &str) -> Vec<Provider> {
    let mut providers = Vec::new();

    for search_path in search_paths {
        if !search_path.exists() {
            continue;
        }

        if let Ok(entries) = std::fs::read_dir(search_path) {
            for entry in entries.flatten() {
                if let Ok(file_name) = entry.file_name().into_string()
                    && file_name.contains(capability)
                    && file_name
                        .as_bytes()
                        .get(file_name.len().saturating_sub(5)..)
                        .is_some_and(|ext| ext.eq_ignore_ascii_case(b".sock"))
                {
                    let path = entry.path();
                    info!("   ✅ Found {} socket at: {}", capability, path.display());

                    let provider_id = extract_provider_id(path.to_string_lossy().as_ref());

                    let mut provider = Provider::new(
                        provider_id.as_ref(),
                        vec![capability.to_string()],
                        format!("/primal/{}", provider_id.as_ref()),
                    );
                    provider.metadata.discovery_method =
                        format!("filesystem:{}", search_path.display());

                    providers.push(provider);
                }
            }
        }
    }

    providers
}

/// Extract provider ID from socket path
///
/// Examples:
/// - `/tmp/songbird-test-security.sock` → `songbird-test-security`
/// - `/tmp/songbird-test-nat0.sock` → `songbird-test-nat0`
/// - `/run/user/1000/crypto.sock` → `crypto`
fn extract_provider_id(socket_path: &str) -> Arc<str> {
    PathBuf::from(socket_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .map_or_else(|| Arc::from("unknown"), Arc::from)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn test_extract_provider_id() {
        assert_eq!(
            extract_provider_id("/tmp/songbird-test-security.sock").as_ref(),
            "songbird-test-security"
        );
        assert_eq!(
            extract_provider_id("/tmp/songbird-test-nat0.sock").as_ref(),
            "songbird-test-nat0"
        );
        assert_eq!(extract_provider_id("/run/user/1000/crypto.sock").as_ref(), "crypto");
        assert_eq!(extract_provider_id("storage provider.sock").as_ref(), "storage provider");
    }

    #[tokio::test]
    async fn test_environment_strategy() {
        let providers = EnvironmentStrategy::discover_with("crypto", |k| {
            if k == "CRYPTO_PROVIDER_SOCKET" {
                Ok(String::from("/tmp/test-crypto.sock"))
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
        assert_eq!(extract_provider_id("").as_ref(), "unknown");
    }

    #[test]
    fn extract_provider_id_hidden_file() {
        assert_eq!(extract_provider_id("/tmp/.hidden.sock").as_ref(), ".hidden");
    }

    #[tokio::test]
    async fn environment_strategy_prefers_provider_socket_over_provider() {
        let providers = EnvironmentStrategy::discover_with("ab", |k| match k {
            "AB_PROVIDER_SOCKET" => Ok(String::from("/tmp/first.sock")),
            "AB_PROVIDER" => Ok(String::from("/tmp/second.sock")),
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
            "XY_PROVIDER" => Ok(String::from("/run/xy.sock")),
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
            "ZZ_SOCKET" => Ok(String::from("/var/zz.sock")),
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
        let s = DiscoveryStrategy::Environment;
        assert_eq!(s.name(), "environment");
    }

    #[tokio::test]
    async fn discovery_strategy_filesystem_name() {
        let s = DiscoveryStrategy::Filesystem(FilesystemStrategy::with_paths(vec![]));
        assert_eq!(s.name(), "filesystem");
    }

    #[test]
    fn filesystem_strategy_new_default_constructible() {
        let _ = FilesystemStrategy::new();
        let _ = FilesystemStrategy::default();
    }
}
