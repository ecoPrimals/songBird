// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Platform-aware directory resolution for logs, cache, data, config, and temp.
//!
//! Every path is overridable via `SONGBIRD_*_DIR` environment variables.
//! Fallback paths follow XDG / AppData conventions per platform.

use songbird_types::constants::HOME_FALLBACK_DIR;

use super::read_process_env;

fn env_get_or_default_with(
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
    key: &str,
    default: impl Into<String>,
) -> String {
    env(key).unwrap_or_else(|_| default.into())
}

// ==================== DIRECTORY CONFIGURATION ====================

/// Get log directory from environment or calculate default
#[must_use]
pub fn get_log_dir() -> String {
    get_log_dir_with(&read_process_env)
}

/// Same as [`get_log_dir`] with an injectable env reader.
#[must_use]
pub fn get_log_dir_with(env: &impl Fn(&str) -> Result<String, std::env::VarError>) -> String {
    env_get_or_default_with(env, "SONGBIRD_LOG_DIR", {
        if cfg!(windows) {
            format!(
                "{}\\AppData\\Local\\Songbird\\logs",
                env_get_or_default_with(env, "USERPROFILE", "C:\\Users\\Default".to_string()),
            )
        } else {
            format!(
                "{}/.local/share/songbird/logs",
                env_get_or_default_with(env, "HOME", HOME_FALLBACK_DIR.to_string()),
            )
        }
    })
}

/// Get cache directory from environment or calculate default
#[must_use]
pub fn get_cache_dir() -> String {
    get_cache_dir_with(&read_process_env)
}

/// Same as [`get_cache_dir`] with an injectable env reader.
#[must_use]
pub fn get_cache_dir_with(env: &impl Fn(&str) -> Result<String, std::env::VarError>) -> String {
    env_get_or_default_with(env, "SONGBIRD_CACHE_DIR", {
        if cfg!(windows) {
            format!(
                "{}\\AppData\\Local\\Songbird\\cache",
                env_get_or_default_with(env, "USERPROFILE", "C:\\Users\\Default".to_string()),
            )
        } else {
            format!(
                "{}/.cache/songbird",
                env_get_or_default_with(env, "HOME", HOME_FALLBACK_DIR.to_string()),
            )
        }
    })
}

/// Get data directory from environment or calculate default
#[must_use]
pub fn get_data_dir() -> String {
    get_data_dir_with(&read_process_env)
}

/// Same as [`get_data_dir`] with an injectable env reader.
#[must_use]
pub fn get_data_dir_with(env: &impl Fn(&str) -> Result<String, std::env::VarError>) -> String {
    env_get_or_default_with(env, "SONGBIRD_DATA_DIR", {
        if cfg!(windows) {
            format!(
                "{}\\AppData\\Roaming\\Songbird",
                env_get_or_default_with(env, "USERPROFILE", "C:\\Users\\Default".to_string()),
            )
        } else {
            format!(
                "{}/.local/share/songbird",
                env_get_or_default_with(env, "HOME", HOME_FALLBACK_DIR.to_string()),
            )
        }
    })
}

/// Get configuration directory from environment or calculate default
#[must_use]
pub fn get_config_dir() -> String {
    get_config_dir_with(&read_process_env)
}

/// Same as [`get_config_dir`] with an injectable env reader.
#[must_use]
pub fn get_config_dir_with(env: &impl Fn(&str) -> Result<String, std::env::VarError>) -> String {
    env_get_or_default_with(env, "SONGBIRD_CONFIG_DIR", {
        if cfg!(windows) {
            format!(
                "{}\\AppData\\Roaming\\Songbird\\config",
                env_get_or_default_with(env, "USERPROFILE", "C:\\Users\\Default".to_string()),
            )
        } else {
            format!(
                "{}/.config/songbird",
                env_get_or_default_with(env, "HOME", HOME_FALLBACK_DIR.to_string()),
            )
        }
    })
}

/// Get temporary directory from environment or use system default
#[must_use]
pub fn get_temp_dir() -> String {
    get_temp_dir_with(&read_process_env)
}

/// Same as [`get_temp_dir`] with an injectable env reader.
#[must_use]
pub fn get_temp_dir_with(env: &impl Fn(&str) -> Result<String, std::env::VarError>) -> String {
    env_get_or_default_with(
        env,
        "SONGBIRD_TEMP_DIR",
        std::env::temp_dir().to_string_lossy().to_string(),
    )
}
