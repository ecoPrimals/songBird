// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Socket discovery for Neural API and `BearDog`.
//!
//! After Gate 5.2, songbird crates only need to find the Neural API socket.
//! `BearDog` discovery is kept for `BEARDOG_MODE=direct` bootstrap.

use songbird_types::defaults::paths::{
    BEARDOG_SOCKET_LEGACY, BIOMEOS_RUNTIME_SUBDIR, NEURAL_API_SOCKET_LEGACY_PATTERN,
};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Neural API socket path under `xdg_runtime_dir`/`biomeos`/ (matches discovery rules).
#[must_use]
pub fn neural_api_socket_path_in_biomeos_runtime(
    xdg_runtime_dir: &str,
    family_id: &str,
) -> PathBuf {
    let socket_name = if family_id.is_empty() {
        "neural-api.sock".to_string()
    } else {
        format!("neural-api-{family_id}.sock")
    };
    PathBuf::from(xdg_runtime_dir).join(BIOMEOS_RUNTIME_SUBDIR).join(socket_name)
}

/// `BearDog` socket path under `xdg_runtime_dir`/`biomeos`/ (matches discovery rules).
#[must_use]
pub fn beardog_socket_path_in_biomeos_runtime(xdg_runtime_dir: &str, family_id: &str) -> PathBuf {
    let socket_name = if family_id.is_empty() {
        "beardog.sock".to_string()
    } else {
        format!("beardog-{family_id}.sock")
    };
    PathBuf::from(xdg_runtime_dir).join(BIOMEOS_RUNTIME_SUBDIR).join(socket_name)
}

/// Discover the Neural API socket (preferred for all crypto routing).
///
/// Priority:
/// 1. `$NEURAL_API_SOCKET` / `$NEURALS_SOCKET`
/// 2. `$XDG_RUNTIME_DIR/biomeos/neural-api.sock` (with optional family suffix)
/// 3. `/tmp/biomeos/neural-api.sock`
/// 4. `/tmp/neural-api-{family}.sock` (legacy)
#[must_use]
pub fn discover_neural_api_socket() -> String {
    discover_neural_api_socket_with(|k| std::env::var(k).ok(), Path::exists)
}

/// Like [`discover_neural_api_socket`], but with injectable env and path checks (tests / embedding).
#[must_use]
pub fn discover_neural_api_socket_with<G, P>(get_var: G, path_exists: P) -> String
where
    G: Fn(&str) -> Option<String>,
    P: Fn(&Path) -> bool,
{
    if let Some(socket) = get_var("NEURAL_API_SOCKET")
        && !socket.is_empty()
    {
        info!("✅ Neural API socket via $NEURAL_API_SOCKET: {}", socket);
        return socket;
    }

    if let Some(socket) = get_var("NEURALS_SOCKET")
        && !socket.is_empty()
    {
        info!("✅ Neural API socket via $NEURALS_SOCKET: {}", socket);
        return socket;
    }

    if let Some(xdg_dir) = get_var("XDG_RUNTIME_DIR") {
        let family_id = get_var("FAMILY_ID").unwrap_or_default();
        let socket_path = neural_api_socket_path_in_biomeos_runtime(&xdg_dir, &family_id);
        if path_exists(&socket_path) {
            let path = socket_path.to_string_lossy().to_string();
            info!("✅ Neural API socket via XDG: {}", path);
            return path;
        }
    }

    let biomeos_path = Path::new("/tmp/biomeos/neural-api.sock");
    if path_exists(biomeos_path) {
        return biomeos_path.to_string_lossy().to_string();
    }

    let family_id = get_var("FAMILY_ID").unwrap_or_else(|| "default".to_string());
    let socket = format!("{NEURAL_API_SOCKET_LEGACY_PATTERN}{family_id}.sock");
    warn!("⚠️  Using legacy Neural API path: {}", socket);
    socket
}

/// Discover the `BearDog` socket (for `BEARDOG_MODE=direct` only).
///
/// Priority:
/// 1. `$BEARDOG_SOCKET`
/// 2. `$XDG_RUNTIME_DIR/biomeos/beardog.sock`
/// 3. `/tmp/beardog.sock` (legacy)
#[must_use]
pub fn discover_beardog_socket() -> String {
    discover_beardog_socket_with(|k| std::env::var(k).ok(), Path::exists)
}

/// Like [`discover_beardog_socket`], but with injectable env and path checks (tests / embedding).
#[must_use]
pub fn discover_beardog_socket_with<G, P>(get_var: G, path_exists: P) -> String
where
    G: Fn(&str) -> Option<String>,
    P: Fn(&Path) -> bool,
{
    if let Some(socket) = get_var("BEARDOG_SOCKET")
        && !socket.is_empty()
    {
        return socket;
    }

    if let Some(xdg_dir) = get_var("XDG_RUNTIME_DIR") {
        let family_id = get_var("FAMILY_ID").unwrap_or_default();
        let socket_path = beardog_socket_path_in_biomeos_runtime(&xdg_dir, &family_id);
        if path_exists(&socket_path) {
            return socket_path.to_string_lossy().to_string();
        }
    }

    warn!("⚠️  Using legacy BearDog path: {}", BEARDOG_SOCKET_LEGACY);
    BEARDOG_SOCKET_LEGACY.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn neural_api_socket_path_in_biomeos_runtime_empty_family() {
        let p = neural_api_socket_path_in_biomeos_runtime("/run/user/1000", "");
        assert_eq!(p, PathBuf::from("/run/user/1000/biomeos/neural-api.sock"));
    }

    #[test]
    fn neural_api_socket_path_in_biomeos_runtime_with_family() {
        let p = neural_api_socket_path_in_biomeos_runtime("/run/user/1000", "alpha");
        assert_eq!(p, PathBuf::from("/run/user/1000/biomeos/neural-api-alpha.sock"));
    }

    #[test]
    fn beardog_socket_path_in_biomeos_runtime_empty_family() {
        let p = beardog_socket_path_in_biomeos_runtime("/run/user/1000", "");
        assert_eq!(p, PathBuf::from("/run/user/1000/biomeos/beardog.sock"));
    }

    #[test]
    fn beardog_socket_path_in_biomeos_runtime_with_family() {
        let p = beardog_socket_path_in_biomeos_runtime("/run/user/1000", "beta");
        assert_eq!(p, PathBuf::from("/run/user/1000/biomeos/beardog-beta.sock"));
    }

    #[test]
    fn discover_neural_prefers_neural_api_socket_env() {
        let map: HashMap<&str, String> =
            std::iter::once(("NEURAL_API_SOCKET", "/explicit/neural.sock".to_string())).collect();
        let out = discover_neural_api_socket_with(|k| map.get(k).cloned(), |_p| false);
        assert_eq!(out, "/explicit/neural.sock");
    }

    #[test]
    fn discover_neural_falls_back_to_neurals_socket() {
        let map: HashMap<&str, String> =
            std::iter::once(("NEURALS_SOCKET", "/alt/neural.sock".to_string())).collect();
        let out = discover_neural_api_socket_with(|k| map.get(k).cloned(), |_p| false);
        assert_eq!(out, "/alt/neural.sock");
    }

    #[test]
    fn discover_neural_uses_xdg_when_file_exists() {
        let xdg = "/run/user/9999";
        let expected = neural_api_socket_path_in_biomeos_runtime(xdg, "");
        let map: HashMap<&str, String> =
            std::iter::once(("XDG_RUNTIME_DIR", xdg.to_string())).collect();
        let out =
            discover_neural_api_socket_with(|k| map.get(k).cloned(), |p| p == expected.as_path());
        assert_eq!(out, expected.to_string_lossy());
    }

    #[test]
    fn discover_neural_legacy_family_default_when_unset() {
        let map: HashMap<&str, String> = HashMap::new();
        let out = discover_neural_api_socket_with(|k| map.get(k).cloned(), |_p| false);
        assert_eq!(out, "/tmp/neural-api-default.sock");
    }

    #[test]
    fn discover_neural_legacy_family_from_env() {
        let map: HashMap<&str, String> =
            std::iter::once(("FAMILY_ID", "gamma".to_string())).collect();
        let out = discover_neural_api_socket_with(|k| map.get(k).cloned(), |_p| false);
        assert_eq!(out, "/tmp/neural-api-gamma.sock");
    }

    #[test]
    fn discover_beardog_prefers_beardog_socket_env() {
        let map: HashMap<&str, String> =
            std::iter::once(("BEARDOG_SOCKET", "/custom/bd.sock".to_string())).collect();
        let out = discover_beardog_socket_with(|k| map.get(k).cloned(), |_p| false);
        assert_eq!(out, "/custom/bd.sock");
    }

    #[test]
    fn discover_beardog_uses_xdg_when_file_exists() {
        let xdg = "/run/user/8888";
        let expected = beardog_socket_path_in_biomeos_runtime(xdg, "");
        let map: HashMap<&str, String> =
            std::iter::once(("XDG_RUNTIME_DIR", xdg.to_string())).collect();
        let out =
            discover_beardog_socket_with(|k| map.get(k).cloned(), |p| p == expected.as_path());
        assert_eq!(out, expected.to_string_lossy());
    }

    #[test]
    fn discover_beardog_legacy_when_no_match() {
        let map: HashMap<&str, String> = HashMap::new();
        let out = discover_beardog_socket_with(|k| map.get(k).cloned(), |_p| false);
        assert_eq!(out, BEARDOG_SOCKET_LEGACY);
    }
}
