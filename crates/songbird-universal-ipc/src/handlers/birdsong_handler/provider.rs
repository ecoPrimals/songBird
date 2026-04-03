// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use super::BirdSongHandler;
use songbird_discovery::security_birdsong_provider::SecurityBirdSongProvider;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{info, warn};

impl BirdSongHandler {
    /// Discover the security provider socket at runtime (capability-based; no hardcoding).
    ///
    /// Delegates to [`songbird_http_client::discover_security_socket`] for the
    /// canonical chain (`SECURITY_PROVIDER_SOCKET`, XDG capability symlink, `BEARDOG_SOCKET`, …).
    ///
    /// Deep debt: Runtime discovery, agnostic to deployment. TCP `tcp:host:port` is supported
    /// when returned by discovery.
    pub(super) async fn discover_security_socket(&self) -> Result<PathBuf, String> {
        // Check cache first
        {
            let cached = self.security_socket.read().await;
            if let Some(path) = cached.as_ref() {
                let path_str = path.to_string_lossy();
                if path_str.starts_with("tcp:") {
                    return Ok(path.clone());
                }
                if path.exists() {
                    return Ok(path.clone());
                }
            }
        }

        let socket_str = songbird_http_client::discover_security_socket();
        let socket_path = PathBuf::from(socket_str);

        let path_str = socket_path.to_string_lossy();
        let is_tcp = path_str.starts_with("tcp:");

        if !is_tcp && !socket_path.exists() {
            return Err(format!(
                "Security provider socket not found at {}. \
                 Is the crypto provider running? Set SECURITY_PROVIDER_SOCKET=/path/to/socket.sock \
                 or BEARDOG_SOCKET=/path/to/beardog.sock (legacy), \
                 or use tcp:host:port for cross-gate deployments",
                socket_path.display()
            ));
        }

        {
            let mut cached = self.security_socket.write().await;
            *cached = Some(socket_path.clone());
        }

        if is_tcp {
            info!("✅ Discovered security provider TCP socket: {}", path_str);
        } else {
            info!("✅ Discovered security provider Unix socket: {}", socket_path.display());
        }
        Ok(socket_path)
    }

    /// Get or create `BirdSong` provider (lazy initialization)
    ///
    /// Deep debt: Lazy loading, runtime discovery
    pub(super) async fn get_provider(&self) -> Result<Arc<SecurityBirdSongProvider>, String> {
        // Check cache
        {
            let cached = self.provider.read().await;
            if let Some(provider) = cached.as_ref() {
                return Ok(Arc::clone(provider));
            }
        }

        // Discover and create provider
        let socket_path = self.discover_security_socket().await?;

        // Discover family_id from environment (matches biomeOS pattern)
        // Priority: FAMILY_ID > SONGBIRD_FAMILY_ID > NODE_FAMILY_ID
        let family_id = songbird_process_env::var("FAMILY_ID")
            .or_else(|_| songbird_process_env::var("SONGBIRD_FAMILY_ID"))
            .or_else(|_| songbird_process_env::var("NODE_FAMILY_ID"))
            .ok();

        if family_id.is_some() {
            info!("🔒 Using family_id from environment");
        } else {
            warn!(
                "⚠️  No FAMILY_ID environment variable set - security provider encryption may fail"
            );
        }

        let provider = SecurityBirdSongProvider::new(socket_path, family_id)
            .await
            .map_err(|e| format!("Failed to create BirdSong provider: {e}"))?;

        let provider = Arc::new(provider);

        // Cache provider
        {
            let mut cached = self.provider.write().await;
            *cached = Some(Arc::clone(&provider));
        }

        info!("✅ BirdSong provider initialized (Pure Rust, Zero unsafe)");
        Ok(provider)
    }
}
