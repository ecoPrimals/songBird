// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Pre-dispatch capability gate for JSON-RPC methods (JH-0).
//!
//! Every incoming RPC call passes through [`MethodGate::check`] *before*
//! reaching the dispatch table. The gate classifies methods into
//! [`MethodAccessLevel::Public`] (health probes, identity, capability
//! advertisement — always allowed) and [`MethodAccessLevel::Protected`]
//! (require a valid capability token once enforcement is activated).
//!
//! Two enforcement modes:
//! - **Permissive** (default): protected methods are logged but allowed,
//!   preserving backward compatibility during ecosystem rollout.
//! - **Enforced**: protected methods without a valid token are rejected
//!   with `PERMISSION_DENIED` (-32001).
//!
//! Implements the ecosystem standard defined in
//! `primalSpring/wateringHole/METHOD_GATE_STANDARD.md`.

use super::protocol::JsonRpcError;

/// Server-defined error codes (JSON-RPC 2.0 range: -32000 to -32099).
pub mod error_codes {
    /// Caller identity could not be established.
    pub const UNAUTHORIZED: i32 = -32_000;
    /// Caller identity established but lacks scope for the method.
    pub const PERMISSION_DENIED: i32 = -32_001;
}

/// Access level for a JSON-RPC method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MethodAccessLevel {
    /// Health probes, identity, capability advertisement — always allowed.
    Public,
    /// Requires a valid capability token when enforcement is active.
    Protected,
}

/// Prefix patterns that are always public.
const PUBLIC_METHOD_PREFIXES: &[&str] = &["health."];

/// Exact method names that are always public.
const PUBLIC_METHODS: &[&str] = &[
    "identity.get",
    "capabilities.list",
    "capability.list",
    "lifecycle.status",
    "auth.check",
    "auth.mode",
    "auth.peer_info",
];

/// Classify a method string into its access level.
#[must_use]
pub fn classify_method(method: &str) -> MethodAccessLevel {
    if PUBLIC_METHODS.contains(&method) {
        return MethodAccessLevel::Public;
    }
    for prefix in PUBLIC_METHOD_PREFIXES {
        if method.starts_with(prefix) {
            return MethodAccessLevel::Public;
        }
    }
    MethodAccessLevel::Protected
}

/// Peer credentials extracted from `SO_PEERCRED` on Unix sockets.
#[derive(Debug, Clone)]
pub struct PeerCredentials {
    /// Process ID of the caller (if available).
    pub pid: Option<u32>,
    /// User ID of the caller.
    pub uid: u32,
}

/// Identity and authorization context for an incoming RPC call.
#[derive(Debug, Clone)]
pub struct CallerContext {
    /// Optional bearer / capability token sent in the request.
    pub bearer_token: Option<String>,
    /// Peer credentials from `SO_PEERCRED` (Unix socket only).
    pub peer: Option<PeerCredentials>,
    /// Where the connection came from.
    pub origin: ConnectionOrigin,
}

/// How the caller connected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionOrigin {
    /// Local Unix domain socket.
    Unix,
    /// TCP loopback (`127.0.0.1` / `::1`).
    Loopback,
    /// Remote TCP connection.
    Remote,
}

impl CallerContext {
    /// Create a caller context for a Unix domain socket connection.
    ///
    /// Peer credentials (`SO_PEERCRED`) are not extracted here because
    /// `std::os::unix::net::UnixStream::peer_cred()` is still behind the
    /// unstable `peer_credentials_unix_socket` feature gate and the crate
    /// uses `#![forbid(unsafe_code)]`. Once the API stabilizes (or a safe
    /// wrapper like `rustix` is adopted), this method will populate
    /// `PeerCredentials` automatically.
    #[must_use]
    pub const fn from_unix() -> Self {
        Self {
            bearer_token: None,
            peer: None,
            origin: ConnectionOrigin::Unix,
        }
    }

    /// Build a caller context for loopback TCP with no peer credentials.
    #[must_use]
    pub const fn loopback() -> Self {
        Self {
            bearer_token: None,
            peer: None,
            origin: ConnectionOrigin::Loopback,
        }
    }

    /// Build a caller context for a remote TCP connection.
    #[must_use]
    pub const fn remote() -> Self {
        Self {
            bearer_token: None,
            peer: None,
            origin: ConnectionOrigin::Remote,
        }
    }
}

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
    /// # Errors
    ///
    /// Returns `JsonRpcError` with `PERMISSION_DENIED` when a protected
    /// method is called without a valid capability token and the gate is
    /// in `Enforced` mode.
    pub fn check(&self, method: &str, caller: &CallerContext) -> Result<(), JsonRpcError> {
        let level = classify_method(method);

        if level == MethodAccessLevel::Public {
            return Ok(());
        }

        let authorized = caller.bearer_token.is_some();

        if authorized {
            return Ok(());
        }

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

/// Handle `auth.check` — is the caller authenticated?
#[must_use]
pub fn handle_auth_check(caller: &CallerContext) -> serde_json::Value {
    let authenticated = caller.bearer_token.is_some();
    serde_json::json!({
        "authenticated": authenticated,
        "origin": format!("{:?}", caller.origin).to_lowercase(),
        "has_peer_credentials": caller.peer.is_some(),
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
        "auth.check" => Some(handle_auth_check(caller)),
        "auth.mode" => Some(handle_auth_mode(gate)),
        "auth.peer_info" => Some(handle_auth_peer_info(caller)),
        _ => None,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn health_methods_are_public() {
        assert_eq!(classify_method("health.check"), MethodAccessLevel::Public);
        assert_eq!(classify_method("health.liveness"), MethodAccessLevel::Public);
        assert_eq!(classify_method("health.readiness"), MethodAccessLevel::Public);
    }

    #[test]
    fn identity_get_is_public() {
        assert_eq!(classify_method("identity.get"), MethodAccessLevel::Public);
    }

    #[test]
    fn capabilities_list_is_public() {
        assert_eq!(classify_method("capabilities.list"), MethodAccessLevel::Public);
        assert_eq!(classify_method("capability.list"), MethodAccessLevel::Public);
    }

    #[test]
    fn auth_introspection_is_public() {
        assert_eq!(classify_method("auth.check"), MethodAccessLevel::Public);
        assert_eq!(classify_method("auth.mode"), MethodAccessLevel::Public);
        assert_eq!(classify_method("auth.peer_info"), MethodAccessLevel::Public);
    }

    #[test]
    fn lifecycle_status_is_public() {
        assert_eq!(classify_method("lifecycle.status"), MethodAccessLevel::Public);
    }

    #[test]
    fn discovery_methods_are_protected() {
        assert_eq!(classify_method("discovery.peers"), MethodAccessLevel::Protected);
        assert_eq!(classify_method("discovery.content_peers"), MethodAccessLevel::Protected);
    }

    #[test]
    fn birdsong_methods_are_protected() {
        assert_eq!(classify_method("birdsong.announce"), MethodAccessLevel::Protected);
    }

    #[test]
    fn mesh_methods_are_protected() {
        assert_eq!(classify_method("mesh.connect"), MethodAccessLevel::Protected);
    }

    #[test]
    fn ipc_methods_are_protected() {
        assert_eq!(classify_method("ipc.register"), MethodAccessLevel::Protected);
        assert_eq!(classify_method("ipc.resolve"), MethodAccessLevel::Protected);
    }

    #[test]
    fn empty_method_is_protected() {
        assert_eq!(classify_method(""), MethodAccessLevel::Protected);
    }

    #[test]
    fn loopback_context() {
        let ctx = CallerContext::loopback();
        assert!(ctx.peer.is_none());
        assert!(ctx.bearer_token.is_none());
        assert_eq!(ctx.origin, ConnectionOrigin::Loopback);
    }

    #[test]
    fn unix_context() {
        let ctx = CallerContext::from_unix();
        assert_eq!(ctx.origin, ConnectionOrigin::Unix);
    }

    #[test]
    fn remote_context() {
        let ctx = CallerContext::remote();
        assert_eq!(ctx.origin, ConnectionOrigin::Remote);
    }

    #[test]
    fn enforcement_mode_as_str() {
        assert_eq!(EnforcementMode::Permissive.as_str(), "permissive");
        assert_eq!(EnforcementMode::Enforced.as_str(), "enforced");
    }

    #[test]
    fn public_method_always_passes() {
        let gate = MethodGate::new(EnforcementMode::Enforced);
        let caller = CallerContext::loopback();
        assert!(gate.check("health.check", &caller).is_ok());
        assert!(gate.check("identity.get", &caller).is_ok());
        assert!(gate.check("capabilities.list", &caller).is_ok());
        assert!(gate.check("auth.check", &caller).is_ok());
    }

    #[test]
    fn protected_method_passes_in_permissive_mode() {
        let gate = MethodGate::new(EnforcementMode::Permissive);
        let caller = CallerContext::loopback();
        assert!(gate.check("discovery.peers", &caller).is_ok());
    }

    #[test]
    fn protected_method_rejected_in_enforced_mode_without_token() {
        let gate = MethodGate::new(EnforcementMode::Enforced);
        let caller = CallerContext::loopback();
        let result = gate.check("discovery.peers", &caller);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, error_codes::PERMISSION_DENIED);
        assert!(err.message.contains("discovery.peers"));
    }

    #[test]
    fn protected_method_passes_in_enforced_mode_with_token() {
        let gate = MethodGate::new(EnforcementMode::Enforced);
        let caller = CallerContext {
            bearer_token: Some("valid-token".to_owned()),
            peer: None,
            origin: ConnectionOrigin::Unix,
        };
        assert!(gate.check("discovery.peers", &caller).is_ok());
    }

    #[test]
    fn gate_error_includes_method_in_data() {
        let gate = MethodGate::new(EnforcementMode::Enforced);
        let caller = CallerContext::loopback();
        let err = gate.check("mesh.connect", &caller).unwrap_err();
        let method_in_data =
            err.data.as_ref().and_then(|d| d.get("method")).and_then(serde_json::Value::as_str);
        assert_eq!(method_in_data, Some("mesh.connect"));
    }

    #[test]
    fn auth_check_unauthenticated() {
        let caller = CallerContext::loopback();
        let result = handle_auth_check(&caller);
        assert_eq!(result["authenticated"], false);
        assert_eq!(result["origin"], "loopback");
    }

    #[test]
    fn auth_check_authenticated() {
        let caller = CallerContext {
            bearer_token: Some("tok".to_owned()),
            peer: None,
            origin: ConnectionOrigin::Unix,
        };
        let result = handle_auth_check(&caller);
        assert_eq!(result["authenticated"], true);
    }

    #[test]
    fn auth_mode_response() {
        let gate = MethodGate::new(EnforcementMode::Permissive);
        let result = handle_auth_mode(&gate);
        assert_eq!(result["mode"], "permissive");
        assert_eq!(result["env_var"], "SONGBIRD_AUTH_MODE");
    }

    #[test]
    fn auth_peer_info_no_creds() {
        let caller = CallerContext::loopback();
        let result = handle_auth_peer_info(&caller);
        assert_eq!(result["available"], false);
    }

    #[test]
    fn auth_peer_info_with_creds() {
        let caller = CallerContext {
            bearer_token: None,
            peer: Some(PeerCredentials {
                pid: Some(1234),
                uid: 1000,
            }),
            origin: ConnectionOrigin::Unix,
        };
        let result = handle_auth_peer_info(&caller);
        assert_eq!(result["available"], true);
        assert_eq!(result["uid"], 1000);
        assert_eq!(result["pid"], 1234);
    }

    #[test]
    fn dispatch_routes_auth_methods() {
        let gate = MethodGate::new(EnforcementMode::Permissive);
        let caller = CallerContext::loopback();
        assert!(dispatch_auth_method("auth.check", &gate, &caller).is_some());
        assert!(dispatch_auth_method("auth.mode", &gate, &caller).is_some());
        assert!(dispatch_auth_method("auth.peer_info", &gate, &caller).is_some());
    }

    #[test]
    fn dispatch_returns_none_for_non_auth() {
        let gate = MethodGate::new(EnforcementMode::Permissive);
        let caller = CallerContext::loopback();
        assert!(dispatch_auth_method("discovery.peers", &gate, &caller).is_none());
    }

    #[test]
    fn is_gate_handled_method_correct() {
        assert!(is_gate_handled_method("auth.check"));
        assert!(is_gate_handled_method("auth.mode"));
        assert!(is_gate_handled_method("auth.peer_info"));
        assert!(!is_gate_handled_method("auth.issue_ionic"));
        assert!(!is_gate_handled_method("discovery.peers"));
    }
}
