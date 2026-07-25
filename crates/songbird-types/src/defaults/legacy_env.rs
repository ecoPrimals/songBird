// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Runtime deprecation warnings for legacy primal-named environment variables.
//!
//! songBird should discover other primals at runtime via capability-based discovery,
//! not by hardcoded primal names. Legacy `BEARDOG_*`, `NESTGATE_*`, and `SQUIRREL_*`
//! env vars remain supported but emit warnings when read.

/// Emit a `tracing::warn!` when a legacy primal-named environment variable is read.
///
/// No-op for env keys that are not legacy primal fallbacks.
#[inline]
pub fn warn_if_legacy_primal_env(env: &str) {
    let migrate = match env {
        "BEARDOG_SOCKET" => {
            "use SECURITY_PROVIDER_SOCKET, SECURITY_SOCKET, or capability discovery instead"
        }
        "BEARDOG_CRYPTO_SOCKET" => "use CRYPTO_PROVIDER_SOCKET or capability discovery instead",
        "BEARDOG_SOCKET_PATH" => "use SECURITY_PROVIDER_SOCKET instead",
        "BEARDOG_MODE" => "use SECURITY_PROVIDER_MODE or capability discovery instead",
        "BEARDOG_ENDPOINT" => {
            "use SECURITY_ENDPOINT, SECURITY_PROVIDER_ENDPOINT, or capability discovery instead"
        }
        "BEARDOG_URL" => "use SECURITY_URL or CAPABILITY_SECURITY_ENDPOINT instead",
        "BEARDOG_PORT" | "SONGBIRD_BEARDOG_PORT" => {
            "use SONGBIRD_SECURITY_PORT or SONGBIRD_SECURITY_PROVIDER_PORT instead"
        }
        "BEARDOG_SECURITY_ENDPOINT" => {
            "use SECURITY_PROVIDER_ENDPOINT, SECURITY_ENDPOINT, or CAPABILITY_SECURITY_ENDPOINT instead"
        }
        "BEARDOG_2FA_ENDPOINT" => {
            "use SONGBIRD_SECURITY_PROVIDER_ENDPOINT or CAPABILITY_SECURITY_ENDPOINT instead"
        }
        "BEARDOG_FAMILY_SEED" => "use FAMILY_SEED or BIOMEOS_FAMILY_SEED instead",
        "SONGBIRD_BEARDOG_ENDPOINT" => "use CAPABILITY_SECURITY_ENDPOINT instead",
        "NESTGATE_SOCKET" => "use STORAGE_PROVIDER_SOCKET or capability discovery instead",
        "NESTGATE_ENDPOINT" => {
            "use STORAGE_ENDPOINT, STORAGE_PROVIDER_ENDPOINT, or capability discovery instead"
        }
        "SONGBIRD_NESTGATE_ENDPOINT" | "SONGBIRD_NESTGATE_PORT" => {
            "use SONGBIRD_STORAGE_PROVIDER_ENDPOINT or SONGBIRD_STORAGE_PORT instead"
        }
        "SQUIRREL_SOCKET" => "use AI_PROVIDER_SOCKET or capability discovery instead",
        "SQUIRREL_ENDPOINT" => {
            "use AI_ENDPOINT, AI_PROVIDER_ENDPOINT, or capability discovery instead"
        }
        "SONGBIRD_SQUIRREL_ENDPOINT" | "SONGBIRD_SQUIRREL_PORT" => {
            "use SONGBIRD_AI_PROVIDER_ENDPOINT or SONGBIRD_AI_PORT instead"
        }
        other if other.starts_with("BEARDOG_") => {
            "use capability-based SECURITY_* env vars or runtime discovery instead"
        }
        other if other.starts_with("NESTGATE_") => {
            "use capability-based STORAGE_* env vars or runtime discovery instead"
        }
        other if other.starts_with("SQUIRREL_") => {
            "use capability-based AI_* env vars or runtime discovery instead"
        }
        _ => return,
    };

    tracing::warn!(env, "deprecated: {migrate}");
}
