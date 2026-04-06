// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Platform-appropriate log, cache, data, config, and temp directories.

use songbird_types::error_helpers::SafeEnv;

/// Get log directory from environment or calculate default
#[must_use]
pub fn get_log_dir() -> String {
    SafeEnv::get_or_default("SONGBIRD_LOG_DIR", {
        // Use platform-appropriate log directory
        if cfg!(windows) {
            format!(
                "{}\\AppData\\Local\\Songbird\\logs",
                SafeEnv::get_or_default("USERPROFILE", "C:\\Users\\Default".to_string()),
            )
        } else {
            format!(
                "{}/.local/share/songbird/logs",
                SafeEnv::get_or_default("HOME", "/tmp".to_string()),
            )
        }
    })
}

/// Get cache directory from environment or calculate default
#[must_use]
pub fn get_cache_dir() -> String {
    SafeEnv::get_or_default("SONGBIRD_CACHE_DIR", {
        // Use platform-appropriate cache directory
        if cfg!(windows) {
            format!(
                "{}\\AppData\\Local\\Songbird\\cache",
                SafeEnv::get_or_default("USERPROFILE", "C:\\Users\\Default".to_string()),
            )
        } else {
            format!("{}/.cache/songbird", SafeEnv::get_or_default("HOME", "/tmp".to_string()),)
        }
    })
}

/// Get data directory from environment or calculate default
#[must_use]
pub fn get_data_dir() -> String {
    SafeEnv::get_or_default("SONGBIRD_DATA_DIR", {
        // Use platform-appropriate data directory
        if cfg!(windows) {
            format!(
                "{}\\AppData\\Roaming\\Songbird",
                SafeEnv::get_or_default("USERPROFILE", "C:\\Users\\Default".to_string()),
            )
        } else {
            format!("{}/.local/share/songbird", SafeEnv::get_or_default("HOME", "/tmp".to_string()),)
        }
    })
}

/// Get configuration directory from environment or calculate default
#[must_use]
pub fn get_config_dir() -> String {
    SafeEnv::get_or_default("SONGBIRD_CONFIG_DIR", {
        // Use platform-appropriate config directory
        if cfg!(windows) {
            format!(
                "{}\\AppData\\Roaming\\Songbird\\config",
                SafeEnv::get_or_default("USERPROFILE", "C:\\Users\\Default".to_string()),
            )
        } else {
            format!("{}/.config/songbird", SafeEnv::get_or_default("HOME", "/tmp".to_string()),)
        }
    })
}

/// Get temporary directory from environment or use system default
#[must_use]
pub fn get_temp_dir() -> String {
    SafeEnv::get_or_default("SONGBIRD_TEMP_DIR", std::env::temp_dir().to_string_lossy().to_string())
}
