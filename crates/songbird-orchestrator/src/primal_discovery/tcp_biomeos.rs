// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! TCP discovery files, BiomeOS `*.sock` enumeration, and synchronous multi-strategy resolution.

use anyhow::Result;
use std::path::PathBuf;
use tracing::{debug, info, warn};

use super::capability::Capability;

#[cfg(unix)]
use super::unix_transport;

/// Synchronous discovery: env, TCP files, then biomeos socket probe.
pub fn discover_with_sync<F>(capability: Capability, env_reader: F) -> Result<String>
where
    F: Fn(&str) -> Option<String>,
{
    info!("🔍 Discovering {:?} provider (capability-based discovery)...", capability);

    if let Some(socket_path) = env_reader(capability.env_var_name()) {
        info!("   ✅ Found via {}: {}", capability.env_var_name(), socket_path);
        return Ok(socket_path);
    }

    for alt_var in capability.alt_env_vars() {
        if let Some(socket_path) = env_reader(alt_var) {
            info!("   ✅ Found via {} (compatibility): {}", alt_var, socket_path);
            return Ok(socket_path);
        }
    }

    if let Some(tcp_endpoint) = discover_tcp_from_capability(capability, &env_reader) {
        info!("   ✅ Found {:?} provider via TCP discovery file: {}", capability, tcp_endpoint);
        return Ok(tcp_endpoint);
    }

    if let Some(socket_path) = discover_via_biomeos_probe(capability, &env_reader) {
        info!("   ✅ Found {:?} provider via biomeos probe: {}", capability, socket_path);
        return Ok(socket_path);
    }

    warn!("❌ No {:?} provider found - checked all discovery strategies", capability);
    anyhow::bail!("No {capability:?} provider available")
}

#[cfg(unix)]
#[inline]
fn is_unix_socket_filetype(ft: &std::fs::FileType) -> bool {
    std::os::unix::fs::FileTypeExt::is_socket(ft)
}

#[cfg(not(unix))]
#[inline]
fn is_unix_socket_filetype(_ft: &std::fs::FileType) -> bool {
    false
}

fn list_biomeos_sock_paths<F>(env_reader: &F) -> Vec<PathBuf>
where
    F: Fn(&str) -> Option<String>,
{
    let mut dirs = Vec::new();
    if let Some(xdg) = env_reader("XDG_RUNTIME_DIR") {
        dirs.push(PathBuf::from(xdg).join(songbird_types::primal_names::BIOMEOS_DIR));
    }
    dirs.push(songbird_types::defaults::paths::biomeos_socket_dir_tmp());

    let mut out = Vec::new();
    for dir in dirs {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.extension().is_some_and(|e| e.eq_ignore_ascii_case("sock")) {
                    continue;
                }
                let Ok(ft) = entry.file_type() else {
                    continue;
                };
                if ft.is_file() || is_unix_socket_filetype(&ft) {
                    out.push(path);
                }
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

#[cfg(unix)]
pub fn discover_via_biomeos_probe_filtered<F, P>(env_reader: &F, predicate: P) -> Option<String>
where
    F: Fn(&str) -> Option<String>,
    P: Fn(&[String]) -> bool,
{
    for path in list_biomeos_sock_paths(env_reader) {
        if let Some(tokens) = unix_transport::probe_capabilities_list(&path)
            && predicate(&tokens)
        {
            return Some(path.to_string_lossy().into_owned());
        }
    }
    None
}

#[cfg(not(unix))]
pub fn discover_via_biomeos_probe_filtered<F, P>(_env_reader: &F, _predicate: P) -> Option<String>
where
    F: Fn(&str) -> Option<String>,
    P: Fn(&[String]) -> bool,
{
    None
}

fn discover_via_biomeos_probe<F>(capability: Capability, env_reader: &F) -> Option<String>
where
    F: Fn(&str) -> Option<String>,
{
    discover_via_biomeos_probe_filtered(env_reader, |tokens| {
        capability.matches_capability_tokens(tokens)
    })
}

/// Synchronous entry for non-async callers (e.g. JWT path discovery). Uses blocking Unix I/O only.
#[must_use]
pub fn discover_via_biomeos_probe_blocking(capability: Capability) -> Option<String> {
    discover_via_biomeos_probe(capability, &|k| songbird_process_env::var(k).ok())
}

/// Injectable env reader variant (tests).
#[must_use]
pub fn discover_via_biomeos_probe_blocking_with<F>(
    capability: Capability,
    env_reader: &F,
) -> Option<String>
where
    F: Fn(&str) -> Option<String>,
{
    discover_via_biomeos_probe(capability, env_reader)
}

/// Scan socket directories for sockets matching capability — **deprecated path**: use `discover_via_biomeos_probe`.
#[must_use]
pub fn scan_sockets(capability: Capability) -> Option<String> {
    discover_via_biomeos_probe_blocking(capability)
}

fn discover_tcp_from_capability<F>(capability: Capability, env_reader: &F) -> Option<String>
where
    F: Fn(&str) -> Option<String>,
{
    let names: Vec<&str> = match capability {
        Capability::Crypto => vec!["crypto"],
        Capability::Security => vec!["security"],
        Capability::Http => vec!["http"],
        Capability::Ai => vec!["ai"],
        Capability::Storage => vec!["storage"],
        Capability::Messaging => vec!["messaging"],
    };

    for name in names {
        if let Some(tcp_addr) = check_tcp_discovery_file(name, env_reader) {
            return Some(format!("tcp:{tcp_addr}"));
        }
    }

    None
}

fn check_tcp_discovery_file<F>(primal_name: &str, env_reader: &F) -> Option<String>
where
    F: Fn(&str) -> Option<String>,
{
    let filename = format!("{primal_name}-ipc-port");
    let mut candidates = Vec::new();

    if let Some(runtime_dir) = env_reader("XDG_RUNTIME_DIR") {
        candidates.push(std::path::PathBuf::from(runtime_dir).join(&filename));
    }

    if let Some(home) = env_reader("HOME") {
        candidates.push(std::path::PathBuf::from(home).join(".local/share").join(&filename));
    }

    candidates.push(std::env::temp_dir().join(&filename));

    check_tcp_discovery_from_candidates(&candidates)
}

/// Used by tests and TCP discovery helpers.
pub fn check_tcp_discovery_from_candidates(candidates: &[std::path::PathBuf]) -> Option<String> {
    for path in candidates {
        if let Ok(content) = std::fs::read_to_string(path)
            && let Some(addr_str) = content.strip_prefix("tcp:")
        {
            let addr_trimmed = addr_str.trim();
            if addr_trimmed.parse::<std::net::SocketAddr>().is_ok() {
                debug!("   Found TCP discovery file: {} -> {}", path.display(), addr_trimmed);
                return Some(addr_trimmed.to_string());
            }
        }
    }

    None
}
