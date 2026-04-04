// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Shared environment parsing helpers for canonical constants.

/// Process environment lookup (function pointer satisfies HRTB for injectable env readers).
pub fn read_process_env(key: &str) -> Result<String, std::env::VarError> {
    songbird_process_env::var(key)
}

pub fn env_parse_with<T: std::str::FromStr>(
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
    key: &str,
    default: T,
) -> T {
    env(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

pub fn env_get_bool_with(
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
    key: &str,
    default: bool,
) -> bool {
    env(key)
        .ok()
        .and_then(|v| match v.to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => Some(true),
            "false" | "0" | "no" | "off" => Some(false),
            _ => v.parse().ok(),
        })
        .unwrap_or(default)
}

pub fn env_get_or_default_with(
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
    key: &str,
    default: impl Into<String>,
) -> String {
    env(key).unwrap_or_else(|_| default.into())
}

pub fn env_port_with(
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
    key: &str,
    default: u16,
) -> u16 {
    env(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

pub fn env_or_default_with(
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
    key: &str,
    default: impl Into<String>,
) -> String {
    env_get_or_default_with(env, key, default)
}
