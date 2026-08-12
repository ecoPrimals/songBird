// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! PID file path resolution and legacy cleanup.

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Resolve the default PID file location scoped by family and node identity.
pub(super) fn default_pid_file() -> Result<PathBuf> {
    let family_id = songbird_process_env::var("SONGBIRD_FAMILY_ID")
        .or_else(|_| songbird_process_env::var("FAMILY_ID"))
        .ok();
    let node_id = songbird_process_env::var("SONGBIRD_NODE_ID")
        .or_else(|_| songbird_process_env::var("NODE_ID"))
        .or_else(|_| songbird_process_env::var("SPORE_ID"))
        .ok();

    let filename_suffix = match (family_id.as_ref(), node_id.as_ref()) {
        (Some(family), Some(node)) => format!("-{family}-{node}"),
        (Some(family), None) => format!("-{family}"),
        (None, Some(node)) => format!("-{node}"),
        (None, None) => String::new(),
    };

    let filename = format!("songbird{filename_suffix}.pid");

    if let Ok(pid_dir) = songbird_process_env::var("SONGBIRD_PID_DIR") {
        let custom_path = PathBuf::from(&pid_dir).join(&filename);
        if let Some(parent) = custom_path.parent()
            && fs::create_dir_all(parent).is_ok()
        {
            debug!("Using SONGBIRD_PID_DIR: {}", custom_path.display());
            return Ok(custom_path);
        }
    }

    if let Ok(state_dir) = songbird_process_env::var("SONGBIRD_STATE_DIR") {
        let run_path = PathBuf::from(&state_dir).join("run").join(&filename);
        if let Some(parent) = run_path.parent()
            && fs::create_dir_all(parent).is_ok()
        {
            debug!("Using SONGBIRD_STATE_DIR/run: {}", run_path.display());
            return Ok(run_path);
        }
    }

    let system_path = songbird_types::constants::songbird_runtime_dir().join(&filename);
    if let Some(parent) = system_path.parent()
        && (parent.exists() || fs::create_dir_all(parent).is_ok())
    {
        return Ok(system_path);
    }

    let home = dirs::home_dir().context("Could not determine home directory")?;
    let user_path = home.join(".local/share/songbird").join(&filename);

    if let Some(parent) = user_path.parent() {
        fs::create_dir_all(parent).context("Failed to create PID file directory")?;
    }

    Ok(user_path)
}

/// Clean up stale PID files from deprecated locations (Wave 157d P2 fix).
pub(super) fn cleanup_legacy_pid_files(current_path: &Path) {
    let legacy_paths: &[&str] =
        &["/tmp/songbird.pid", "/var/run/songbird.pid", "/var/run/songbird/songbird.pid"];

    for path_str in legacy_paths {
        let path = Path::new(path_str);
        if path == current_path || !path.exists() {
            continue;
        }
        try_remove_stale_pid(path, path_str);
    }

    if let Some(home) = dirs::home_dir() {
        let user_legacy = home.join(".local/share/songbird/songbird.pid");
        if user_legacy != current_path && user_legacy.exists() {
            try_remove_stale_pid(&user_legacy, &user_legacy.display().to_string());
        }
    }
}

/// Attempt to remove a PID file if the referenced process is no longer running.
pub(super) fn try_remove_stale_pid(path: &Path, label: &str) {
    let Ok(contents) = fs::read_to_string(path) else {
        return;
    };
    let Ok(pid) = contents.trim().parse::<u32>() else {
        if fs::remove_file(path).is_ok() {
            info!("🧹 Removed corrupt legacy PID file: {label}");
        }
        return;
    };
    if !songbird_types::process_ops::is_process_running(pid) && fs::remove_file(path).is_ok() {
        info!("🧹 Removed stale legacy PID file: {label} (PID {pid} not running)");
    }
}
