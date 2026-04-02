// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use super::BirdSongHandler;
use songbird_discovery::beardog_birdsong_provider::BearDogBirdSongProvider;
use songbird_types::primal_names;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, info, warn};

impl BirdSongHandler {
    /// Discover `BearDog` socket at runtime (no hardcoding)
    ///
    /// Discovery order:
    /// 1. `BEARDOG_SOCKET` environment variable (supports `tcp:host:port` format for Android)
    /// 2. `XDG_RUNTIME_DIR/biomeos/beardog.sock`
    /// 3. Well-known fallback: /run/user/$(id -u)/biomeos/beardog.sock
    ///
    /// Deep debt: Runtime discovery, agnostic to deployment
    /// Android support: TCP sockets via `tcp:host:port` format (Feb 5, 2026)
    pub(super) async fn discover_beardog_socket(&self) -> Result<PathBuf, String> {
        // Check cache first
        {
            let cached = self.beardog_socket.read().await;
            if let Some(path) = cached.as_ref() {
                // For TCP sockets (tcp:host:port), skip existence check
                let path_str = path.to_string_lossy();
                if path_str.starts_with("tcp:") {
                    return Ok(path.clone());
                }
                if path.exists() {
                    return Ok(path.clone());
                }
            }
        }

        // Discover at runtime (no hardcoding)
        // Discovery order: env BEARDOG_SOCKET → XDG_RUNTIME_DIR → well-known /run/user/{uid}
        let mut tried_paths: Vec<String> = Vec::new();

        let env_socket = songbird_process_env::var("BEARDOG_SOCKET");
        let socket_path = if let Ok(path) = env_socket {
            debug!("🔍 Discovering BearDog via BEARDOG_SOCKET env: {}", path);
            PathBuf::from(path)
        } else {
            tried_paths.push("BEARDOG_SOCKET env (not set)".to_string());
            if let Ok(xdg) = songbird_process_env::var("XDG_RUNTIME_DIR") {
                debug!("🔍 Discovering BearDog via XDG_RUNTIME_DIR");
                let p = PathBuf::from(format!("{xdg}/biomeos/{}.sock", primal_names::BEARDOG));
                tried_paths.push(format!("{} (XDG_RUNTIME_DIR)", p.display()));
                p
            } else {
                tried_paths.push("XDG_RUNTIME_DIR env (not set)".to_string());
                let uid = std::fs::read_to_string("/proc/self/loginuid")
                    .ok()
                    .and_then(|s| s.trim().parse::<u32>().ok())
                    .or_else(|| {
                        std::fs::read_to_string("/proc/self/status").ok().and_then(|content| {
                            content.lines().find(|line| line.starts_with("Uid:")).and_then(|line| {
                                line.split_whitespace().nth(1)?.parse::<u32>().ok()
                            })
                        })
                    })
                    .unwrap_or(1000);

                debug!("🔍 Discovering BearDog via well-known path (UID: {uid}, safe Rust)");
                let p = PathBuf::from(format!(
                    "/run/user/{uid}/biomeos/{}.sock",
                    primal_names::BEARDOG
                ));
                tried_paths.push(format!("{} (well-known)", p.display()));
                p
            }
        };

        // Check if this is a TCP socket (tcp:host:port format)
        let path_str = socket_path.to_string_lossy();
        let is_tcp = path_str.starts_with("tcp:");

        // Verify socket exists (skip for TCP - can't check file existence for network sockets)
        if !is_tcp && !socket_path.exists() {
            let tried = tried_paths.join(", ");
            return Err(format!(
                "BearDog socket not found. Tried: {tried}. \
                 Is BearDog running? Set BEARDOG_SOCKET=/path/to/beardog.sock \
                 or BEARDOG_SOCKET=tcp:host:port for cross-gate deployments"
            ));
        }

        // Cache for future calls
        {
            let mut cached = self.beardog_socket.write().await;
            *cached = Some(socket_path.clone());
        }

        if is_tcp {
            info!("✅ Discovered BearDog TCP socket: {}", path_str);
        } else {
            info!("✅ Discovered BearDog Unix socket: {}", socket_path.display());
        }
        Ok(socket_path)
    }

    /// Get or create `BirdSong` provider (lazy initialization)
    ///
    /// Deep debt: Lazy loading, runtime discovery
    pub(super) async fn get_provider(&self) -> Result<Arc<BearDogBirdSongProvider>, String> {
        // Check cache
        {
            let cached = self.provider.read().await;
            if let Some(provider) = cached.as_ref() {
                return Ok(Arc::clone(provider));
            }
        }

        // Discover and create provider
        let socket_path = self.discover_beardog_socket().await?;

        // Discover family_id from environment (matches biomeOS pattern)
        // Priority: FAMILY_ID > SONGBIRD_FAMILY_ID > NODE_FAMILY_ID
        let family_id = songbird_process_env::var("FAMILY_ID")
            .or_else(|_| songbird_process_env::var("SONGBIRD_FAMILY_ID"))
            .or_else(|_| songbird_process_env::var("NODE_FAMILY_ID"))
            .ok();

        if family_id.is_some() {
            info!("🔒 Using family_id from environment");
        } else {
            warn!("⚠️  No FAMILY_ID environment variable set - BearDog encryption may fail");
        }

        let provider = BearDogBirdSongProvider::new(socket_path, family_id)
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
