// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Socket discovery for Neural API and security provider.
//!
//! After Gate 5.2, songbird crates only need to find the Neural API socket.
//! Direct security-provider discovery is kept for legacy direct-mode bootstrap (`BEARDOG_MODE=direct`).
//!
//! ## Capability-Based Discovery (wateringHole v1.2)
//!
//! Songbird discovers crypto/security by **capability**, not by primal name:
//! - Primary env: `SECURITY_PROVIDER_SOCKET` (capability-standard)
//! - Capability symlink: `$XDG_RUNTIME_DIR/biomeos/security.sock`
//! - Fallback: `$XDG_RUNTIME_DIR/biomeos/crypto.sock`
//! - Legacy: `BEARDOG_SOCKET` env var (deprecated, logged)

use songbird_types::defaults::paths::{BIOMEOS_RUNTIME_SUBDIR, ai_provider_socket_legacy_path};
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

/// Security capability socket path under `xdg_runtime_dir`/`biomeos`/
/// (wateringHole v1.2 capability-named symlink pattern).
#[must_use]
pub fn security_socket_path_in_biomeos_runtime(xdg_runtime_dir: &str) -> PathBuf {
    PathBuf::from(xdg_runtime_dir).join(BIOMEOS_RUNTIME_SUBDIR).join("security.sock")
}

/// Family-scoped security socket path under `xdg_runtime_dir`/`biomeos`/
/// e.g. `/run/user/1000/biomeos/security-nucleus01.sock`
#[must_use]
pub fn security_socket_path_in_biomeos_runtime_with_family(
    xdg_runtime_dir: &str,
    family_id: &str,
) -> PathBuf {
    let socket_name = if family_id.is_empty() {
        "security.sock".to_string()
    } else {
        format!("security-{family_id}.sock")
    };
    PathBuf::from(xdg_runtime_dir).join(BIOMEOS_RUNTIME_SUBDIR).join(socket_name)
}

/// Legacy family-scoped BearDog socket path under `xdg_runtime_dir`/`biomeos`/
/// e.g. `/run/user/1000/biomeos/beardog-nucleus01.sock`
#[must_use]
pub fn legacy_beardog_socket_path_in_biomeos_runtime(
    xdg_runtime_dir: &str,
    family_id: &str,
) -> PathBuf {
    let socket_name = if family_id.is_empty() {
        "beardog.sock".to_string()
    } else {
        format!("beardog-{family_id}.sock")
    };
    PathBuf::from(xdg_runtime_dir).join(BIOMEOS_RUNTIME_SUBDIR).join(socket_name)
}

/// Crypto provider socket path under `xdg_runtime_dir`/`biomeos`/ (matches discovery rules).
#[must_use]
pub fn crypto_socket_path_in_biomeos_runtime(xdg_runtime_dir: &str, family_id: &str) -> PathBuf {
    let socket_name = if family_id.is_empty() {
        "crypto.sock".to_string()
    } else {
        format!("crypto-{family_id}.sock")
    };
    PathBuf::from(xdg_runtime_dir).join(BIOMEOS_RUNTIME_SUBDIR).join(socket_name)
}

/// Discover the Neural API socket (preferred for all crypto routing).
///
/// Priority:
/// 1. `$NEURAL_API_SOCKET` / `$NEURALS_SOCKET`
/// 2. `$XDG_RUNTIME_DIR/biomeos/neural-api.sock` (with optional family suffix)
/// 3. `{std::env::temp_dir()}/biomeos/neural-api.sock`
/// 4. `/tmp/neural-api-{family}.sock` (legacy)
#[must_use]
pub fn discover_neural_api_socket() -> String {
    discover_neural_api_socket_with(|k| songbird_process_env::var(k).ok(), Path::exists)
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

    let biomeos_path = std::env::temp_dir().join(BIOMEOS_RUNTIME_SUBDIR).join("neural-api.sock");
    if path_exists(&biomeos_path) {
        return biomeos_path.to_string_lossy().to_string();
    }

    let family_id = get_var("FAMILY_ID").unwrap_or_else(|| "default".to_string());
    let socket = ai_provider_socket_legacy_path(&family_id).to_string_lossy().into_owned();
    warn!("⚠️  Using legacy Neural API path: {}", socket);
    socket
}

/// Discover the security provider socket via capability-based discovery.
///
/// Priority (wateringHole v1.2 compliant):
/// 1. `$SECURITY_PROVIDER_SOCKET` (capability-standard env var)
/// 2. `$SECURITY_SOCKET` (capability domain)
/// 3. `$CRYPTO_PROVIDER_SOCKET` (alternate capability name)
/// 4. `$XDG_RUNTIME_DIR/biomeos/security.sock` (capability symlink)
/// 5. `$XDG_RUNTIME_DIR/biomeos/security-{family_id}.sock` (family-scoped)
/// 6. `$XDG_RUNTIME_DIR/biomeos/crypto-{family_id}.sock` (domain socket)
/// 7. `$XDG_RUNTIME_DIR/biomeos/beardog-{family_id}.sock` (legacy on-disk)
/// 8. `$BEARDOG_SOCKET` (legacy env — logged as deprecated)
/// 9. `{temp_dir}/biomeos/security.sock` (temp fallback)
#[must_use]
pub fn discover_security_socket() -> String {
    discover_security_socket_with(|k| songbird_process_env::var(k).ok(), Path::exists)
}

/// Like [`discover_security_socket`], but with injectable env and path checks.
#[must_use]
pub fn discover_security_socket_with<G, P>(get_var: G, path_exists: P) -> String
where
    G: Fn(&str) -> Option<String>,
    P: Fn(&Path) -> bool,
{
    if let Some(socket) = get_var("SECURITY_PROVIDER_SOCKET")
        && !socket.is_empty()
    {
        info!("✅ Security provider via $SECURITY_PROVIDER_SOCKET: {socket}");
        return socket;
    }

    if let Some(socket) = get_var("SECURITY_SOCKET")
        && !socket.is_empty()
    {
        info!("✅ Security provider via $SECURITY_SOCKET: {socket}");
        return socket;
    }

    if let Some(socket) = get_var("CRYPTO_PROVIDER_SOCKET")
        && !socket.is_empty()
    {
        info!("✅ Security provider via $CRYPTO_PROVIDER_SOCKET: {socket}");
        return socket;
    }

    if let Some(xdg_dir) = get_var("XDG_RUNTIME_DIR") {
        let cap_symlink = security_socket_path_in_biomeos_runtime(&xdg_dir);
        if path_exists(&cap_symlink) {
            let path = cap_symlink.to_string_lossy().to_string();
            info!("✅ Security provider via capability symlink: {path}");
            return path;
        }

        let family_id = get_var("FAMILY_ID").unwrap_or_default();

        let family_security =
            security_socket_path_in_biomeos_runtime_with_family(&xdg_dir, &family_id);
        if path_exists(&family_security) {
            let path = family_security.to_string_lossy().to_string();
            info!("✅ Security provider via family-scoped security socket: {path}");
            return path;
        }

        let crypto_path = crypto_socket_path_in_biomeos_runtime(&xdg_dir, &family_id);
        if path_exists(&crypto_path) {
            let path = crypto_path.to_string_lossy().to_string();
            info!("✅ Security provider via crypto domain socket: {path}");
            return path;
        }

        let legacy_beardog = legacy_beardog_socket_path_in_biomeos_runtime(&xdg_dir, &family_id);
        if path_exists(&legacy_beardog) {
            let path = legacy_beardog.to_string_lossy().to_string();
            warn!(
                "Security provider via legacy beardog socket: {path} — migrate to security-{{family}}.sock"
            );
            return path;
        }
    }

    if let Some(socket) = get_var("BEARDOG_SOCKET")
        && !socket.is_empty()
    {
        warn!(
            "BEARDOG_SOCKET is deprecated — migrate to SECURITY_PROVIDER_SOCKET or SECURITY_SOCKET"
        );
        return socket;
    }

    let fallback = std::env::temp_dir().join(BIOMEOS_RUNTIME_SUBDIR).join("security.sock");
    if path_exists(&fallback) {
        return fallback.to_string_lossy().to_string();
    }

    let legacy = songbird_types::defaults::paths::security_provider_legacy_flat_path();
    warn!("⚠️  Using legacy temp-dir fallback for security provider: {}", legacy.display());
    legacy.to_string_lossy().into_owned()
}

/// Deprecated alias for [`discover_security_socket`].
#[deprecated(note = "Use discover_security_socket (capability-based naming)")]
#[must_use]
pub fn discover_security_provider_socket() -> String {
    discover_security_socket()
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
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
    fn crypto_socket_path_in_biomeos_runtime_empty_family() {
        let p = crypto_socket_path_in_biomeos_runtime("/run/user/1000", "");
        assert_eq!(p, PathBuf::from("/run/user/1000/biomeos/crypto.sock"));
    }

    #[test]
    fn crypto_socket_path_in_biomeos_runtime_with_family() {
        let p = crypto_socket_path_in_biomeos_runtime("/run/user/1000", "beta");
        assert_eq!(p, PathBuf::from("/run/user/1000/biomeos/crypto-beta.sock"));
    }

    #[test]
    fn discover_neural_prefers_neural_api_socket_env() {
        let map: HashMap<&str, String> =
            std::iter::once(("NEURAL_API_SOCKET", "/explicit/neural.sock".to_string())).collect();
        let out = discover_neural_api_socket_with(|k| map.get(k).cloned(), |_p| false);
        assert_eq!(out, "/explicit/neural.sock");
    }

    #[test]
    fn discover_neural_neural_api_socket_wins_over_neurals_when_both_set() {
        let map: HashMap<&str, String> = [
            ("NEURAL_API_SOCKET", "/primary.sock".to_string()),
            ("NEURALS_SOCKET", "/secondary.sock".to_string()),
        ]
        .into_iter()
        .collect();
        let out = discover_neural_api_socket_with(|k| map.get(k).cloned(), |_p| false);
        assert_eq!(
            out, "/primary.sock",
            "$NEURAL_API_SOCKET should take precedence over $NEURALS_SOCKET"
        );
    }

    #[test]
    fn discover_neural_skips_empty_neural_api_socket_env() {
        let map: HashMap<&str, String> =
            std::iter::once(("NEURAL_API_SOCKET", String::new())).collect();
        let out = discover_neural_api_socket_with(|k| map.get(k).cloned(), |_p| false);
        assert_eq!(out, "/tmp/neural-api-default.sock", "empty env should fall through to legacy");
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
    fn discover_neural_uses_temp_biomeos_neural_socket_when_present() {
        let temp = std::env::temp_dir();
        let biomeos_path = temp.join("biomeos").join("neural-api.sock");
        let _ = std::fs::create_dir_all(biomeos_path.parent().expect("biomeos parent"));
        std::fs::write(&biomeos_path, b"x").expect("touch neural socket");
        let map: HashMap<&str, String> = HashMap::new();
        let out = discover_neural_api_socket_with(
            |k| map.get(k).cloned(),
            |p| p == biomeos_path.as_path(),
        );
        assert_eq!(
            PathBuf::from(&out),
            biomeos_path,
            "should prefer temp_dir/biomeos/neural-api.sock when it exists"
        );
        let _ = std::fs::remove_file(&biomeos_path);
    }

    #[test]
    fn discover_neural_xdg_with_family_suffix_in_path() {
        let xdg = "/run/user/7777";
        let expected = neural_api_socket_path_in_biomeos_runtime(xdg, "fam");
        let map: HashMap<&str, String> =
            [("XDG_RUNTIME_DIR", xdg.to_string()), ("FAMILY_ID", "fam".to_string())]
                .into_iter()
                .collect();
        let out =
            discover_neural_api_socket_with(|k| map.get(k).cloned(), |p| p == expected.as_path());
        assert_eq!(out, expected.to_string_lossy());
    }

    #[test]
    fn security_socket_path_capability_symlink() {
        let p = security_socket_path_in_biomeos_runtime("/run/user/1000");
        assert_eq!(p, PathBuf::from("/run/user/1000/biomeos/security.sock"));
    }

    #[test]
    fn security_socket_family_scoped_path() {
        let p = security_socket_path_in_biomeos_runtime_with_family("/run/user/1000", "nucleus01");
        assert_eq!(p, PathBuf::from("/run/user/1000/biomeos/security-nucleus01.sock"));
    }

    #[test]
    fn security_socket_family_scoped_empty_family() {
        let p = security_socket_path_in_biomeos_runtime_with_family("/run/user/1000", "");
        assert_eq!(p, PathBuf::from("/run/user/1000/biomeos/security.sock"));
    }

    #[test]
    fn legacy_beardog_socket_family_scoped() {
        let p = legacy_beardog_socket_path_in_biomeos_runtime("/run/user/1000", "nucleus01");
        assert_eq!(p, PathBuf::from("/run/user/1000/biomeos/beardog-nucleus01.sock"));
    }

    #[test]
    fn discover_security_finds_family_scoped_security_socket() {
        let xdg = "/run/user/5555";
        let family_security = security_socket_path_in_biomeos_runtime_with_family(xdg, "nucleus01");
        let map: HashMap<&str, String> =
            [("XDG_RUNTIME_DIR", xdg.to_string()), ("FAMILY_ID", "nucleus01".to_string())]
                .into_iter()
                .collect();
        let out = discover_security_socket_with(
            |k| map.get(k).cloned(),
            |p| p == family_security.as_path(),
        );
        assert_eq!(
            out,
            family_security.to_string_lossy(),
            "should find security-nucleus01.sock under XDG"
        );
    }

    #[test]
    fn discover_security_finds_legacy_beardog_family_socket() {
        let xdg = "/run/user/6666";
        let beardog = legacy_beardog_socket_path_in_biomeos_runtime(xdg, "nucleus01");
        let map: HashMap<&str, String> =
            [("XDG_RUNTIME_DIR", xdg.to_string()), ("FAMILY_ID", "nucleus01".to_string())]
                .into_iter()
                .collect();
        let out =
            discover_security_socket_with(|k| map.get(k).cloned(), |p| p == beardog.as_path());
        assert_eq!(
            out,
            beardog.to_string_lossy(),
            "should find beardog-nucleus01.sock under XDG as legacy fallback"
        );
    }

    #[test]
    fn discover_security_prefers_family_security_over_legacy_beardog() {
        let xdg = "/run/user/7777";
        let security = security_socket_path_in_biomeos_runtime_with_family(xdg, "nucleus01");
        let beardog = legacy_beardog_socket_path_in_biomeos_runtime(xdg, "nucleus01");
        let map: HashMap<&str, String> =
            [("XDG_RUNTIME_DIR", xdg.to_string()), ("FAMILY_ID", "nucleus01".to_string())]
                .into_iter()
                .collect();
        let out = discover_security_socket_with(
            |k| map.get(k).cloned(),
            |p| p == security.as_path() || p == beardog.as_path(),
        );
        assert_eq!(
            out,
            security.to_string_lossy(),
            "security-nucleus01.sock should beat beardog-nucleus01.sock"
        );
    }

    /// Backward-compat: ensures `SECURITY_PROVIDER_SOCKET` wins when both it and the deprecated
    /// `BEARDOG_SOCKET` env var are set (migration shim for the legacy socket key).
    #[test]
    fn discover_security_prefers_security_provider_socket_env() {
        let map: HashMap<&str, String> = [
            ("SECURITY_PROVIDER_SOCKET", "/cap/security.sock".to_string()),
            ("BEARDOG_SOCKET", "/legacy/security-fallback.sock".to_string()),
        ]
        .into_iter()
        .collect();
        let out = discover_security_socket_with(|k| map.get(k).cloned(), |_p| false);
        assert_eq!(out, "/cap/security.sock", "$SECURITY_PROVIDER_SOCKET beats legacy socket env");
    }

    #[test]
    fn discover_security_uses_crypto_provider_socket_env() {
        let map: HashMap<&str, String> =
            std::iter::once(("CRYPTO_PROVIDER_SOCKET", "/cap/crypto.sock".to_string())).collect();
        let out = discover_security_socket_with(|k| map.get(k).cloned(), |_p| false);
        assert_eq!(out, "/cap/crypto.sock");
    }

    #[test]
    fn discover_security_prefers_capability_symlink_over_crypto_domain() {
        let xdg = "/run/user/8888";
        let security = security_socket_path_in_biomeos_runtime(xdg);
        let crypto = crypto_socket_path_in_biomeos_runtime(xdg, "");
        let map: HashMap<&str, String> =
            std::iter::once(("XDG_RUNTIME_DIR", xdg.to_string())).collect();
        let out = discover_security_socket_with(
            |k| map.get(k).cloned(),
            |p| p == security.as_path() || p == crypto.as_path(),
        );
        assert_eq!(
            out,
            security.to_string_lossy(),
            "security.sock symlink should beat crypto.sock"
        );
    }

    #[test]
    fn discover_security_uses_crypto_domain_when_no_symlink() {
        let xdg = "/run/user/8888";
        let crypto = crypto_socket_path_in_biomeos_runtime(xdg, "");
        let map: HashMap<&str, String> =
            std::iter::once(("XDG_RUNTIME_DIR", xdg.to_string())).collect();
        let out = discover_security_socket_with(|k| map.get(k).cloned(), |p| p == crypto.as_path());
        assert_eq!(out, crypto.to_string_lossy());
    }

    /// Backward-compat: when no canonical env keys match, the deprecated `BEARDOG_SOCKET` value
    /// is still honored as the last-resort env fallback (see production `discover_security_socket`).
    #[test]
    fn discover_security_falls_back_to_legacy_socket_env() {
        let map: HashMap<&str, String> =
            std::iter::once(("BEARDOG_SOCKET", "/legacy/security-fallback.sock".to_string()))
                .collect();
        let out = discover_security_socket_with(|k| map.get(k).cloned(), |_p| false);
        assert_eq!(
            out, "/legacy/security-fallback.sock",
            "legacy security socket env still works as last env fallback"
        );
    }

    #[test]
    fn discover_security_legacy_when_no_match() {
        let map: HashMap<&str, String> = HashMap::new();
        let out = discover_security_socket_with(|k| map.get(k).cloned(), |_p| false);
        let expected = std::env::temp_dir().join("security-provider.sock");
        assert_eq!(PathBuf::from(out), expected);
    }
}
