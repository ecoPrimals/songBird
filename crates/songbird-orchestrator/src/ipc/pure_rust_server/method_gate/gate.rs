// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Core [`MethodGate`] implementation — pre-dispatch authorization check.

use super::caller::CallerContext;
use super::classification::{MethodAccessLevel, classify_method, scope_permits_method};
use super::error_codes;
use super::protocol::JsonRpcError;

#[allow(unused_imports, reason = "used in tracing fields")]
use super::caller::ConnectionOrigin;

/// Enforcement mode for the method gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnforcementMode {
    /// Log violations but allow all calls (backward-compatible default).
    Permissive,
    /// Reject unauthenticated calls to protected methods.
    Enforced,
}

impl EnforcementMode {
    /// Resolve from `SONGBIRD_AUTH_MODE` env var.
    /// Defaults to `Permissive` if unset or unrecognized.
    #[must_use]
    pub fn from_env() -> Self {
        match songbird_process_env::var("SONGBIRD_AUTH_MODE")
            .unwrap_or_default()
            .to_lowercase()
            .as_str()
        {
            "enforced" | "enforce" | "strict" => Self::Enforced,
            _ => Self::Permissive,
        }
    }

    /// Human-readable label for diagnostics and `auth.mode` responses.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Permissive => "permissive",
            Self::Enforced => "enforced",
        }
    }
}

/// Pre-dispatch gate that checks caller authorization before method execution.
#[derive(Debug)]
pub struct MethodGate {
    mode: EnforcementMode,
    /// UID of the process running songBird (for same-user trust bypass).
    /// Read from `/proc/self/status` at startup (Linux) or UID env (macOS).
    #[cfg(unix)]
    own_uid: Option<u32>,
}

impl MethodGate {
    /// Create a gate with the given enforcement mode.
    #[must_use]
    pub fn new(mode: EnforcementMode) -> Self {
        Self {
            mode,
            #[cfg(unix)]
            own_uid: Self::resolve_own_uid(),
        }
    }

    /// Read our own UID from `/proc/self/status` (pure Rust, no libc).
    #[cfg(unix)]
    fn resolve_own_uid() -> Option<u32> {
        // Try /proc/self/status first (Linux)
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if let Some(uid_str) = line.strip_prefix("Uid:")
                    && let Some(real_uid) = uid_str.split_whitespace().next()
                    && let Ok(uid) = real_uid.parse::<u32>()
                {
                    return Some(uid);
                }
            }
        }
        // Fallback: XDG_RUNTIME_DIR contains UID on systemd systems
        songbird_process_env::var("XDG_RUNTIME_DIR")
            .ok()
            .and_then(|dir| dir.strip_prefix("/run/user/").map(String::from))
            .and_then(|uid_str| uid_str.parse::<u32>().ok())
    }

    /// Create a gate from the environment (`SONGBIRD_AUTH_MODE`).
    #[must_use]
    pub fn from_env() -> Self {
        Self::new(EnforcementMode::from_env())
    }

    /// Current enforcement mode.
    #[must_use]
    pub const fn mode(&self) -> EnforcementMode {
        self.mode
    }

    /// Check if the caller is a same-user local process (trusted).
    ///
    /// Returns `true` if the caller connected via UDS with valid peer
    /// credentials AND runs as the same user (uid) as songBird.
    /// Root (uid 0) is always considered trusted.
    #[cfg(unix)]
    fn is_trusted_local_peer(&self, caller: &CallerContext) -> bool {
        if let Some(peer) = &caller.peer {
            // Root is always trusted
            if peer.uid == 0 {
                return true;
            }
            // Same UID as us — trusted primal-to-primal within same ecosystem
            if let Some(own) = self.own_uid {
                return peer.uid == own;
            }
        }
        false
    }

    #[cfg(not(unix))]
    fn is_trusted_local_peer(&self, _caller: &CallerContext) -> bool {
        false
    }

    /// Pre-dispatch authorization check.
    ///
    /// Returns `Ok(())` if the call should proceed.
    ///
    /// Authorization order:
    /// 1. Public methods always pass.
    /// 2. Trusted local peer (same uid via `SO_PEERCRED`) always passes.
    /// 3. If verified claims are present, scope must cover the method.
    /// 4. If only a raw bearer token is present (unverified), allow
    ///    (backward-compatible during rollout).
    /// 5. No token + different uid → permissive logs and allows, enforced rejects.
    ///
    /// # Errors
    ///
    /// Returns `JsonRpcError` with `PERMISSION_DENIED` when a protected
    /// method is called without authorization and the gate is in `Enforced` mode.
    pub fn check(&self, method: &str, caller: &CallerContext) -> Result<(), JsonRpcError> {
        let level = classify_method(method);

        if level == MethodAccessLevel::Public {
            return Ok(());
        }

        // Same-user local callers are trusted (primal-to-primal within ecosystem)
        if self.is_trusted_local_peer(caller) {
            return Ok(());
        }

        if let Some(claims) = &caller.verified_claims {
            if scope_permits_method(&claims.scopes, method) {
                return Ok(());
            }
            return match self.mode {
                EnforcementMode::Permissive => {
                    tracing::warn!(
                        method,
                        subject = %claims.subject,
                        scopes = ?claims.scopes,
                        "method gate: token scope does not cover method (permissive — allowing)"
                    );
                    Ok(())
                }
                EnforcementMode::Enforced => {
                    tracing::warn!(
                        method,
                        subject = %claims.subject,
                        "method gate: REJECTED — token scope does not cover method"
                    );
                    Err(JsonRpcError {
                        code: error_codes::PERMISSION_DENIED,
                        message: format!(
                            "permission denied: token scope does not cover '{method}'"
                        ),
                        data: Some(serde_json::json!({
                            "method": method,
                            "subject": claims.subject,
                        })),
                    })
                }
            };
        }

        // Raw bearer token present (unverified) — backward-compatible allow
        if caller.bearer_token.is_some() {
            return Ok(());
        }

        // No token + not same-uid local peer
        match self.mode {
            EnforcementMode::Permissive => {
                tracing::warn!(
                    method,
                    caller_uid = caller.peer.as_ref().map(|p| p.uid),
                    caller_pid = caller.peer.as_ref().and_then(|p| p.pid),
                    origin = ?caller.origin,
                    "method gate: unauthenticated call to protected method (permissive — allowing)"
                );
                Ok(())
            }
            EnforcementMode::Enforced => {
                tracing::warn!(
                    method,
                    caller_uid = caller.peer.as_ref().map(|p| p.uid),
                    caller_pid = caller.peer.as_ref().and_then(|p| p.pid),
                    origin = ?caller.origin,
                    "method gate: REJECTED — different-uid or unknown caller without token"
                );
                Err(JsonRpcError {
                    code: error_codes::PERMISSION_DENIED,
                    message: format!(
                        "permission denied: method '{method}' requires same-uid peer or capability token"
                    ),
                    data: Some(serde_json::json!({ "method": method })),
                })
            }
        }
    }
}

/// Check if a method is an `auth.*` introspection method handled by the gate.
#[must_use]
pub fn is_gate_handled_method(method: &str) -> bool {
    matches!(method, "auth.check" | "auth.mode" | "auth.peer_info")
}

/// Handle `auth.check` — enriched authentication introspection.
///
/// Returns `{ authenticated, verified, enforcement, scopes, subject, expires_in, origin }`.
#[must_use]
pub fn handle_auth_check(caller: &CallerContext, gate: &MethodGate) -> serde_json::Value {
    let authenticated = caller.bearer_token.is_some();
    let verified = caller.verified_claims.is_some();
    let (scopes, subject, expires_in) = match &caller.verified_claims {
        Some(claims) => {
            let exp = claims.expires_at.and_then(|exp| {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                exp.checked_sub(now)
            });
            (
                serde_json::Value::Array(
                    claims.scopes.iter().map(|s| serde_json::Value::String(s.clone())).collect(),
                ),
                serde_json::Value::String(claims.subject.clone()),
                exp.map_or(serde_json::Value::Null, |s| serde_json::Value::Number(s.into())),
            )
        }
        None => (serde_json::Value::Null, serde_json::Value::Null, serde_json::Value::Null),
    };

    serde_json::json!({
        "authenticated": authenticated,
        "verified": verified,
        "enforcement": gate.mode().as_str(),
        "scopes": scopes,
        "subject": subject,
        "expires_in": expires_in,
        "origin": format!("{:?}", caller.origin).to_lowercase(),
    })
}

/// Handle `auth.mode` — current enforcement mode.
#[must_use]
pub fn handle_auth_mode(gate: &MethodGate) -> serde_json::Value {
    serde_json::json!({
        "mode": gate.mode().as_str(),
        "env_var": "SONGBIRD_AUTH_MODE",
    })
}

/// Handle `auth.peer_info` — peer credential introspection.
#[must_use]
pub fn handle_auth_peer_info(caller: &CallerContext) -> serde_json::Value {
    match &caller.peer {
        Some(creds) => serde_json::json!({
            "available": true,
            "uid": creds.uid,
            "pid": creds.pid,
            "origin": format!("{:?}", caller.origin).to_lowercase(),
        }),
        None => serde_json::json!({
            "available": false,
            "reason": "peer credentials not yet extracted (API unstable or non-Unix transport)",
            "origin": format!("{:?}", caller.origin).to_lowercase(),
        }),
    }
}

/// Dispatch an `auth.*` method. Returns `Some(result)` if handled, `None`
/// if the method is not a gate-handled auth method.
#[must_use]
pub fn dispatch_auth_method(
    method: &str,
    gate: &MethodGate,
    caller: &CallerContext,
) -> Option<serde_json::Value> {
    match method {
        "auth.check" => Some(handle_auth_check(caller, gate)),
        "auth.mode" => Some(handle_auth_mode(gate)),
        "auth.peer_info" => Some(handle_auth_peer_info(caller)),
        _ => None,
    }
}
