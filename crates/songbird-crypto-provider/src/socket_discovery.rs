// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Socket discovery for Neural API and BearDog.
//!
//! After Gate 5.2, songbird crates only need to find the Neural API socket.
//! BearDog discovery is kept for `BEARDOG_MODE=direct` bootstrap.

use songbird_types::defaults::paths::{
    BEARDOG_SOCKET_LEGACY, BIOMEOS_RUNTIME_SUBDIR, NEURAL_API_SOCKET_LEGACY_PATTERN,
};
use std::path::PathBuf;
use tracing::{info, warn};

/// Discover the Neural API socket (preferred for all crypto routing).
///
/// Priority:
/// 1. `$NEURAL_API_SOCKET` / `$NEURALS_SOCKET`
/// 2. `$XDG_RUNTIME_DIR/biomeos/neural-api.sock` (with optional family suffix)
/// 3. `/tmp/biomeos/neural-api.sock`
/// 4. `/tmp/neural-api-{family}.sock` (legacy)
#[must_use]
pub fn discover_neural_api_socket() -> String {
    if let Ok(socket) = std::env::var("NEURAL_API_SOCKET")
        && !socket.is_empty()
    {
        info!("✅ Neural API socket via $NEURAL_API_SOCKET: {}", socket);
        return socket;
    }

    if let Ok(socket) = std::env::var("NEURALS_SOCKET")
        && !socket.is_empty()
    {
        info!("✅ Neural API socket via $NEURALS_SOCKET: {}", socket);
        return socket;
    }

    if let Ok(xdg_dir) = std::env::var("XDG_RUNTIME_DIR") {
        let family_id = std::env::var("FAMILY_ID").unwrap_or_default();
        let socket_name = if family_id.is_empty() {
            "neural-api.sock".to_string()
        } else {
            format!("neural-api-{family_id}.sock")
        };
        let socket_path = PathBuf::from(&xdg_dir).join(BIOMEOS_RUNTIME_SUBDIR).join(&socket_name);
        if socket_path.exists() {
            let path = socket_path.to_string_lossy().to_string();
            info!("✅ Neural API socket via XDG: {}", path);
            return path;
        }
    }

    // /tmp/biomeos/ namespace
    let biomeos_path = "/tmp/biomeos/neural-api.sock";
    if std::path::Path::new(biomeos_path).exists() {
        return biomeos_path.to_string();
    }

    let family_id = std::env::var("FAMILY_ID").unwrap_or_else(|_| "default".to_string());
    let socket = format!("{NEURAL_API_SOCKET_LEGACY_PATTERN}{family_id}.sock");
    warn!("⚠️  Using legacy Neural API path: {}", socket);
    socket
}

/// Discover the BearDog socket (for `BEARDOG_MODE=direct` only).
///
/// Priority:
/// 1. `$BEARDOG_SOCKET`
/// 2. `$XDG_RUNTIME_DIR/biomeos/beardog.sock`
/// 3. `/tmp/beardog.sock` (legacy)
#[must_use]
pub fn discover_beardog_socket() -> String {
    if let Ok(socket) = std::env::var("BEARDOG_SOCKET")
        && !socket.is_empty()
    {
        return socket;
    }

    if let Ok(xdg_dir) = std::env::var("XDG_RUNTIME_DIR") {
        let family_id = std::env::var("FAMILY_ID").unwrap_or_default();
        let socket_name = if family_id.is_empty() {
            "beardog.sock".to_string()
        } else {
            format!("beardog-{family_id}.sock")
        };
        let socket_path = PathBuf::from(&xdg_dir).join(BIOMEOS_RUNTIME_SUBDIR).join(&socket_name);
        if socket_path.exists() {
            return socket_path.to_string_lossy().to_string();
        }
    }

    warn!("⚠️  Using legacy BearDog path: {}", BEARDOG_SOCKET_LEGACY);
    BEARDOG_SOCKET_LEGACY.to_string()
}
