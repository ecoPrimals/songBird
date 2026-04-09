// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::env;

/// Resolve security/crypto provider Unix socket from environment (capability-first).
///
/// Order: `SECURITY_PROVIDER_SOCKET`, `CRYPTO_PROVIDER_SOCKET`, `SECURITY_SOCKET`, then
/// deprecated `BEARDOG_SOCKET` (emits [`tracing::warn!`]).
///
/// Prefer `CAPABILITY_SECURITY_ENDPOINT` (capability discovery) or `SECURITY_PROVIDER_*` /
/// `SECURITY_*` variables over legacy primal-named env keys.
#[must_use]
pub fn security_crypto_ipc_socket_from_env(default_fn: impl FnOnce() -> String) -> String {
    if let Ok(p) = env("SECURITY_PROVIDER_SOCKET") {
        return p;
    }
    if let Ok(p) = env("CRYPTO_PROVIDER_SOCKET") {
        return p;
    }
    if let Ok(p) = env("SECURITY_SOCKET") {
        return p;
    }
    if let Ok(p) = env("BEARDOG_SOCKET") {
        tracing::warn!(
            "DEPRECATED: BEARDOG_SOCKET is deprecated — migrate to SECURITY_PROVIDER_SOCKET, SECURITY_SOCKET, or CRYPTO_PROVIDER_SOCKET; prefer CAPABILITY_SECURITY_ENDPOINT or SECURITY_PROVIDER_* for capability-first configuration"
        );
        return p;
    }
    default_fn()
}
