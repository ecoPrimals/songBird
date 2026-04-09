// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use std::path::PathBuf;

use super::identity::family_id_with;
use super::{env, runtime_or_tmp_base};

/// Get this primal's IPC socket path (self-knowledge)
///
/// Resolution order (`BiomeOS` XDG Standard):
/// 1. `SONGBIRD_SOCKET` (explicit override - full path)
/// 2. `BIOMEOS_SOCKET_DIR` + socket name (shared socket directory)
/// 3. `/run/user/$UID/biomeos/` + socket name (XDG-compliant default)
/// 4. `{TMPDIR|/tmp}` + socket name (legacy fallback if XDG unavailable)
///
/// **Socket Naming Standard** (bind target — primal filename per PRIMAL_SELF_KNOWLEDGE_STANDARD):
/// - Default: `songbird.sock` (single-family mode)
/// - Multi-family: `songbird-{family_id}.sock` when a non-default family is active
///
/// The capability-domain name ([`socket_name`], e.g. `network.sock`) is **not** the bind path;
/// after bind, [`create_domain_socket_symlink`] creates `network*.sock` → `songbird*.sock` in the same directory.
///
/// This enables multiple Songbird instances serving different families
/// on the same machine, each with its own isolated socket.
#[must_use]
pub fn socket_path() -> PathBuf {
    // Priority 1: Explicit SONGBIRD_SOCKET override
    if let Ok(path) = env("SONGBIRD_SOCKET") {
        return PathBuf::from(path);
    }

    let sock_name = legacy_socket_name();

    // Priority 2: BIOMEOS_SOCKET_DIR + socket name
    if let Ok(socket_dir) = env("BIOMEOS_SOCKET_DIR") {
        let path = PathBuf::from(socket_dir).join(&sock_name);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        return path;
    }

    // Priority 3: XDG-compliant default (/run/user/$UID/biomeos/)
    let xdg_socket = env("XDG_RUNTIME_DIR").map_or_else(
        |_| {
            env("UID").map_or_else(
                |_| PathBuf::from(format!("{}/{}", runtime_or_tmp_base(), sock_name)),
                |uid_str| PathBuf::from(format!("/run/user/{uid_str}/biomeos/{sock_name}")),
            )
        },
        |xdg_runtime_dir| PathBuf::from(xdg_runtime_dir).join("biomeos").join(&sock_name),
    );

    // Ensure directory exists (Pure Rust!)
    if let Some(parent) = xdg_socket.parent()
        && std::fs::create_dir_all(parent).is_ok()
    {
        return xdg_socket;
    }

    // Priority 4: Legacy fallback (if XDG unavailable or directory creation failed)
    PathBuf::from(format!("{}/{}", runtime_or_tmp_base(), sock_name))
}

/// Capability domain stem for socket naming per `PRIMAL_SELF_KNOWLEDGE_STANDARD.md` v1.1.
const DOMAIN_SOCKET_STEM: &str = "network";

/// Get the socket filename based on family configuration.
///
/// Returns domain-based names per `PRIMAL_SELF_KNOWLEDGE_STANDARD.md` v1.1:
/// - `network.sock` in development mode (no `FAMILY_ID`)
/// - `network-{family_id}.sock` in production mode (`FAMILY_ID` set, non-default)
///
/// The domain symlink ([`create_domain_socket_symlink`]) uses this name as the link path.
#[must_use]
pub fn socket_name() -> String {
    socket_name_with(|k| songbird_process_env::var(k))
}

/// [`socket_name`] with an injectable env reader (for unit tests and alternate backends).
#[must_use]
pub fn socket_name_with<F>(env_reader: F) -> String
where
    F: Fn(&str) -> Result<String, std::env::VarError>,
{
    let fid = family_id_with(&env_reader);
    if fid != "default" && !fid.is_empty() {
        format!("{DOMAIN_SOCKET_STEM}-{fid}.sock")
    } else {
        format!("{DOMAIN_SOCKET_STEM}.sock")
    }
}

/// Primal-named socket filename — **canonical bind path** for this orchestrator's UDS.
///
/// Returns `songbird.sock` or `songbird-{family_id}.sock`. The domain-based
/// [`socket_name`] (`network*.sock`) is installed as a symlink to this path.
#[must_use]
pub fn legacy_socket_name() -> String {
    legacy_socket_name_with(|k| songbird_process_env::var(k))
}

/// [`legacy_socket_name`] with an injectable env reader.
#[must_use]
pub fn legacy_socket_name_with<F>(env_reader: F) -> String
where
    F: Fn(&str) -> Result<String, std::env::VarError>,
{
    let fid = family_id_with(&env_reader);
    if fid != "default" && !fid.is_empty() {
        format!("songbird-{fid}.sock")
    } else {
        "songbird.sock".to_string()
    }
}

/// Create the domain-based symlink `network*.sock` → bound primal socket (`songbird*.sock`).
///
/// Per `PRIMAL_SELF_KNOWLEDGE_STANDARD.md` v1.1: primals should expose a domain-named entry
/// for capability discovery; the listening socket is bound at [`socket_path`] (primal name),
/// and this symlink provides `network` domain access to the same socket.
///
/// Best-effort — failure is logged but does not prevent startup.
#[cfg(unix)]
pub fn create_domain_socket_symlink(bound_socket: &std::path::Path) {
    let Some(parent) = bound_socket.parent() else {
        return;
    };
    let domain_name = socket_name();
    let domain_path = parent.join(&domain_name);
    let _ = std::fs::remove_file(&domain_path);
    if let Err(e) = std::os::unix::fs::symlink(bound_socket, &domain_path) {
        tracing::warn!(
            domain = %domain_path.display(),
            bound = %bound_socket.display(),
            "Could not create domain socket symlink: {e}"
        );
    } else {
        tracing::info!(
            domain = %domain_path.display(),
            bound = %bound_socket.display(),
            "Created domain socket symlink (capability discovery)"
        );
    }
}

#[cfg(not(unix))]
pub fn create_domain_socket_symlink(_bound_socket: &std::path::Path) {}

/// Remove the domain symlink created by [`create_domain_socket_symlink`] if it points at `bound_socket`.
#[cfg(unix)]
pub fn remove_domain_socket_symlink_if_matches(bound_socket: &std::path::Path) {
    let Some(parent) = bound_socket.parent() else {
        return;
    };
    let domain_path = parent.join(socket_name());
    let Ok(meta) = std::fs::symlink_metadata(&domain_path) else {
        return;
    };
    if !meta.file_type().is_symlink() {
        return;
    }
    let Ok(link_target) = std::fs::read_link(&domain_path) else {
        return;
    };
    let resolved = if link_target.is_absolute() {
        link_target
    } else {
        parent.join(link_target)
    };
    if resolved == bound_socket {
        let _ = std::fs::remove_file(&domain_path);
    }
}

#[cfg(not(unix))]
pub fn remove_domain_socket_symlink_if_matches(_bound_socket: &std::path::Path) {}
