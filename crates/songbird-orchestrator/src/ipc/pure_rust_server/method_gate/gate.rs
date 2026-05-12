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
}

impl MethodGate {
    /// Create a gate with the given enforcement mode.
    #[must_use]
    pub const fn new(mode: EnforcementMode) -> Self {
        Self {
            mode,
        }
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

    /// Pre-dispatch authorization check.
    ///
    /// Returns `Ok(())` if the call should proceed.
    ///
    /// Authorization order:
    /// 1. Public methods always pass.
    /// 2. If verified claims are present, scope must cover the method.
    /// 3. If only a raw bearer token is present (unverified), allow
    ///    (backward-compatible during rollout).
    /// 4. No token → permissive logs and allows, enforced rejects.
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

        // No token at all
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
                    "method gate: REJECTED unauthenticated call to protected method"
                );
                Err(JsonRpcError {
                    code: error_codes::PERMISSION_DENIED,
                    message: format!(
                        "permission denied: method '{method}' requires a capability token"
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
