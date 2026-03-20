// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Default filesystem paths
//!
//! Uses XDG base directory spec where applicable.
//! Actual paths should be discovered via environment or runtime config.

use std::path::PathBuf;

/// Subdirectory under `XDG_RUNTIME_DIR` or `/run/user/<uid>/` for biomeOS sockets
pub const BIOMEOS_RUNTIME_SUBDIR: &str = "biomeos";

/// Crypto/security provider socket basenames (XDG scan order: crypto first)
pub const CRYPTO_PROVIDER_SOCKET_FILENAMES_XDG: &[&str] =
    &["crypto.sock", "security.sock", "beardog.sock"];

/// Crypto/security provider socket basenames (UID scan order: security first)
pub const CRYPTO_PROVIDER_SOCKET_FILENAMES_UID: &[&str] =
    &["security.sock", "crypto.sock", "beardog.sock"];

/// Neural / AI capability socket basenames (XDG and UID scan use the same order)
pub const NEURAL_API_CAPABILITY_SOCKET_FILENAMES: &[&str] =
    &["ai.sock", "neural-api.sock", "squirrel.sock"];

/// `/run/user/{uid}/biomeos/{socket_filename}` for UID-based socket discovery
#[must_use]
pub fn run_user_biomeos_socket(uid: &str, socket_filename: &str) -> PathBuf {
    PathBuf::from(format!("/run/user/{uid}/{BIOMEOS_RUNTIME_SUBDIR}/{socket_filename}"))
}

/// Default BearDog socket paths (tried in order during discovery)
pub const BEARDOG_SOCKET_CANDIDATES: &[&str] =
    &["/tmp/beardog.sock", "/run/user/1000/beardog-default.sock", "/var/run/beardog.sock"];

/// Default biomeOS socket directory
pub const BIOMEOS_SOCKET_DIR: &str = "/tmp/biomeos";

/// biomeOS socket fallback paths (capability names, tried in order)
pub const BIOMEOS_SOCKET_FALLBACK_PATHS: &[&str] = &[
    "/tmp/biomeos/security.sock",
    "/tmp/biomeos/crypto.sock",
    "/tmp/biomeos/beardog.sock",
    "/tmp/beardog.sock",
];

/// Default security socket path (final fallback)
pub const BIOMEOS_SECURITY_SOCKET_DEFAULT: &str = "/tmp/biomeos/security.sock";

/// Legacy BearDog socket path
pub const BEARDOG_SOCKET_LEGACY: &str = "/tmp/beardog.sock";

/// Legacy Neural API socket path pattern (append "{family_id}.sock")
pub const NEURAL_API_SOCKET_LEGACY_PATTERN: &str = "/tmp/neural-api-";

/// Default data directory
pub const DEFAULT_DATA_DIR: &str = "/var/lib/songbird";

/// IPC port file path
pub const IPC_PORT_FILE: &str = "/tmp/songbird-ipc-port";

/// System temp directory for IPC discovery files (last resort, `{primal}-ipc-port`)
pub const IPC_DISCOVERY_TMP_DIR: &str = "/tmp";

/// Neural API legacy fallback paths (tried in order)
pub const NEURAL_API_SOCKET_FALLBACK_PATHS: &[&str] =
    &["/tmp/biomeos/ai.sock", "/tmp/biomeos/neural-api.sock", "/tmp/biomeos/squirrel.sock"];

/// Default Neural API socket when discovery finds nothing (legacy)
pub const NEURAL_API_SOCKET_DEFAULT: &str = "/tmp/biomeos/ai.sock";
