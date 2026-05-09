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

// ─── Token verification infrastructure (JH-11 prep) ─────────────────────────

/// Claims extracted from a verified ionic token.
#[derive(Debug, Clone)]
pub struct TokenClaims {
    /// Subject identifier (the entity the token was issued to).
    pub subject: String,
    /// Scope patterns: `"*"`, `"domain.*"`, or exact method names.
    pub scopes: Vec<String>,
    /// Unix timestamp when the token expires (if bounded).
    pub expires_at: Option<u64>,
}

/// Errors from token verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenVerifyError {
    /// No verifier is configured (development/permissive mode).
    NotConfigured,
    /// The token's signature or structure is invalid.
    Invalid(String),
    /// The token has expired.
    Expired,
    /// The upstream verification endpoint is unreachable.
    Unavailable(String),
}

/// Abstraction over ionic token verification.
///
/// Production deployments wire [`BearDogVerifier`] which calls
/// `auth.verify_ionic` on the security provider. Tests use [`NoopVerifier`].
pub trait TokenVerifier: Send + Sync {
    /// Verify an ionic token and extract its claims.
    fn verify(
        &self,
        token: &str,
    ) -> impl std::future::Future<Output = Result<TokenClaims, TokenVerifyError>> + Send;
}

/// Verifier that always returns `NotConfigured` — used in tests and when
/// no security provider is available.
#[derive(Debug, Clone, Copy)]
pub struct NoopVerifier;

impl TokenVerifier for NoopVerifier {
    async fn verify(&self, _token: &str) -> Result<TokenClaims, TokenVerifyError> {
        Err(TokenVerifyError::NotConfigured)
    }
}

/// Verifier that will call BearDog's `auth.verify_ionic` via IPC once
/// BearDog ships key distribution (JH-11).
///
/// Currently returns `Unavailable` — the trait infrastructure is in place so
/// that wiring is a single-line change when the upstream capability lands.
/// Expected BearDog response shape:
/// ```json
/// { "subject": "primal-name", "scopes": ["domain.*"], "expires_at": 1717000000 }
/// ```
#[derive(Debug, Clone)]
pub struct BearDogVerifier {
    _security_client: std::sync::Arc<songbird_http_client::SecurityRpcClient>,
}

impl BearDogVerifier {
    /// Create a verifier backed by the given security provider client.
    #[must_use]
    pub fn new(client: std::sync::Arc<songbird_http_client::SecurityRpcClient>) -> Self {
        Self {
            _security_client: client,
        }
    }
}

impl TokenVerifier for BearDogVerifier {
    async fn verify(&self, _token: &str) -> Result<TokenClaims, TokenVerifyError> {
        // JH-11: Once BearDog ships `auth.verify_ionic`, this becomes:
        //   let params = serde_json::json!({ "token": token });
        //   let result = self._security_client.verify_ionic(params).await?;
        //   Ok(TokenClaims { subject, scopes, expires_at })
        Err(TokenVerifyError::Unavailable(
            "BearDog auth.verify_ionic not yet available (pending JH-11)".to_owned(),
        ))
    }
}

/// Check if any scope in `scopes` permits access to `method`.
///
/// Scope patterns:
/// - `"*"` — permits all methods
/// - `"domain.*"` — permits all methods starting with `domain.`
/// - `"exact.method"` — permits only that exact method
#[must_use]
pub fn scope_permits_method(scopes: &[String], method: &str) -> bool {
    scopes.iter().any(|scope| match scope.as_str() {
        "*" => true,
        s if s.ends_with(".*") => {
            let prefix = &s[..s.len() - 2];
            method.starts_with(prefix) && method.as_bytes().get(prefix.len()) == Some(&b'.')
        }
        s => s == method,
    })
}

/// Extract `_bearer_token` from a JSON-RPC `params` object.
///
/// Clients pass the ionic token as `"_bearer_token"` inside `params`.
/// The field is stripped before forwarding to the method handler.
#[must_use]
pub fn extract_bearer_token(params: &mut serde_json::Value) -> Option<String> {
    params.as_object_mut().and_then(|map| map.remove("_bearer_token")).and_then(|v| match v {
        serde_json::Value::String(s) if !s.is_empty() => Some(s),
        _ => None,
    })
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
    /// Verified claims from the token (populated after async verification).
    pub verified_claims: Option<TokenClaims>,
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
            verified_claims: None,
            peer: None,
            origin: ConnectionOrigin::Unix,
        }
    }

    /// Build a caller context for loopback TCP with no peer credentials.
    #[must_use]
    pub const fn loopback() -> Self {
        Self {
            bearer_token: None,
            verified_claims: None,
            peer: None,
            origin: ConnectionOrigin::Loopback,
        }
    }

    /// Build a caller context for a remote TCP connection.
    #[must_use]
    pub const fn remote() -> Self {
        Self {
            bearer_token: None,
            verified_claims: None,
            peer: None,
            origin: ConnectionOrigin::Remote,
        }
    }

    /// Build a caller context from a TCP peer address.
    ///
    /// Checks whether the peer IP is a loopback address (`127.0.0.1`, `::1`)
    /// and sets `ConnectionOrigin` accordingly.
    #[must_use]
    pub fn from_tcp(addr: std::net::SocketAddr) -> Self {
        let origin = if addr.ip().is_loopback() {
            ConnectionOrigin::Loopback
        } else {
            ConnectionOrigin::Remote
        };
        Self {
            bearer_token: None,
            verified_claims: None,
            peer: None,
            origin,
        }
    }

    /// Attach a bearer token (extracted from `_bearer_token` in params) to
    /// this context. Returns a new context with the token set.
    #[must_use]
    pub fn with_bearer_token(mut self, token: String) -> Self {
        self.bearer_token = Some(token);
        self
    }

    /// Attach verified claims to this context (after async token verification).
    #[must_use]
    pub fn with_verified_claims(mut self, claims: TokenClaims) -> Self {
        self.verified_claims = Some(claims);
        self
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
    /// Authorization order:
    /// 1. Public methods always pass.
    /// 2. If verified claims are present, scope must cover the method.
    /// 3. If only a raw bearer token is present (unverified), allow
    ///    (backward-compatible during JH-11 rollout).
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

        // Verified claims: check scopes
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
    fn from_tcp_loopback_ipv4() {
        let addr: std::net::SocketAddr = "127.0.0.1:12345".parse().unwrap();
        let ctx = CallerContext::from_tcp(addr);
        assert_eq!(ctx.origin, ConnectionOrigin::Loopback);
    }

    #[test]
    fn from_tcp_loopback_ipv6() {
        let addr: std::net::SocketAddr = "[::1]:9000".parse().unwrap();
        let ctx = CallerContext::from_tcp(addr);
        assert_eq!(ctx.origin, ConnectionOrigin::Loopback);
    }

    #[test]
    fn from_tcp_remote() {
        let addr: std::net::SocketAddr = "192.168.1.50:8080".parse().unwrap();
        let ctx = CallerContext::from_tcp(addr);
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
            verified_claims: None,
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
        let gate = MethodGate::new(EnforcementMode::Permissive);
        let caller = CallerContext::loopback();
        let result = handle_auth_check(&caller, &gate);
        assert_eq!(result["authenticated"], false);
        assert_eq!(result["verified"], false);
        assert_eq!(result["enforcement"], "permissive");
        assert_eq!(result["origin"], "loopback");
    }

    #[test]
    fn auth_check_authenticated() {
        let gate = MethodGate::new(EnforcementMode::Enforced);
        let caller = CallerContext {
            bearer_token: Some("tok".to_owned()),
            verified_claims: None,
            peer: None,
            origin: ConnectionOrigin::Unix,
        };
        let result = handle_auth_check(&caller, &gate);
        assert_eq!(result["authenticated"], true);
        assert_eq!(result["verified"], false);
        assert_eq!(result["enforcement"], "enforced");
    }

    #[test]
    fn auth_check_verified_with_claims() {
        let gate = MethodGate::new(EnforcementMode::Enforced);
        let caller = CallerContext {
            bearer_token: Some("verified-tok".to_owned()),
            verified_claims: Some(TokenClaims {
                subject: "songbird-test".to_owned(),
                scopes: vec!["discovery.*".to_owned(), "mesh.connect".to_owned()],
                expires_at: None,
            }),
            peer: None,
            origin: ConnectionOrigin::Unix,
        };
        let result = handle_auth_check(&caller, &gate);
        assert_eq!(result["authenticated"], true);
        assert_eq!(result["verified"], true);
        assert_eq!(result["subject"], "songbird-test");
        assert_eq!(result["scopes"][0], "discovery.*");
        assert_eq!(result["scopes"][1], "mesh.connect");
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
            verified_claims: None,
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

    #[test]
    fn auth_mode_accessible_via_tcp_context() {
        let gate = MethodGate::new(EnforcementMode::Permissive);
        let tcp_addr: std::net::SocketAddr = "192.168.1.100:5000".parse().unwrap();
        let caller = CallerContext::from_tcp(tcp_addr);
        assert_eq!(caller.origin, ConnectionOrigin::Remote);
        let result = dispatch_auth_method("auth.mode", &gate, &caller);
        assert!(result.is_some(), "auth.mode must be reachable over TCP");
        let value = result.unwrap();
        assert_eq!(value["mode"], "permissive");
    }

    // ─── scope_permits_method tests ──────────────────────────────────────

    #[test]
    fn scope_wildcard_permits_all() {
        let scopes = vec!["*".to_owned()];
        assert!(scope_permits_method(&scopes, "discovery.peers"));
        assert!(scope_permits_method(&scopes, "mesh.connect"));
        assert!(scope_permits_method(&scopes, "anything"));
    }

    #[test]
    fn scope_domain_wildcard_permits_domain() {
        let scopes = vec!["discovery.*".to_owned()];
        assert!(scope_permits_method(&scopes, "discovery.peers"));
        assert!(scope_permits_method(&scopes, "discovery.register"));
        assert!(!scope_permits_method(&scopes, "mesh.connect"));
        assert!(!scope_permits_method(&scopes, "discover.peers"));
    }

    #[test]
    fn scope_exact_match() {
        let scopes = vec!["mesh.connect".to_owned()];
        assert!(scope_permits_method(&scopes, "mesh.connect"));
        assert!(!scope_permits_method(&scopes, "mesh.disconnect"));
        assert!(!scope_permits_method(&scopes, "mesh.connect.sub"));
    }

    #[test]
    fn scope_multiple_patterns() {
        let scopes = vec!["discovery.*".to_owned(), "mesh.connect".to_owned()];
        assert!(scope_permits_method(&scopes, "discovery.peers"));
        assert!(scope_permits_method(&scopes, "mesh.connect"));
        assert!(!scope_permits_method(&scopes, "mesh.disconnect"));
    }

    #[test]
    fn scope_empty_denies_all() {
        let scopes: Vec<String> = vec![];
        assert!(!scope_permits_method(&scopes, "anything"));
    }

    #[test]
    fn scope_domain_wildcard_boundary() {
        let scopes = vec!["ipc.*".to_owned()];
        assert!(scope_permits_method(&scopes, "ipc.register"));
        assert!(!scope_permits_method(&scopes, "ipcx.register"));
        assert!(!scope_permits_method(&scopes, "ipc"));
    }

    // ─── extract_bearer_token tests ──────────────────────────────────────

    #[test]
    fn extract_token_from_params() {
        let mut params = serde_json::json!({
            "_bearer_token": "ionic-tok-123",
            "capability": "compute"
        });
        let token = extract_bearer_token(&mut params);
        assert_eq!(token, Some("ionic-tok-123".to_owned()));
        assert!(params.get("_bearer_token").is_none(), "token should be removed");
        assert_eq!(params["capability"], "compute");
    }

    #[test]
    fn extract_token_missing() {
        let mut params = serde_json::json!({ "capability": "compute" });
        let token = extract_bearer_token(&mut params);
        assert_eq!(token, None);
    }

    #[test]
    fn extract_token_empty_string() {
        let mut params = serde_json::json!({ "_bearer_token": "" });
        let token = extract_bearer_token(&mut params);
        assert_eq!(token, None);
    }

    #[test]
    fn extract_token_non_object_params() {
        let mut params = serde_json::json!([1, 2, 3]);
        let token = extract_bearer_token(&mut params);
        assert_eq!(token, None);
    }

    // ─── verified claims in gate check ───────────────────────────────────

    #[test]
    fn verified_claims_with_matching_scope_passes_enforced() {
        let gate = MethodGate::new(EnforcementMode::Enforced);
        let caller = CallerContext {
            bearer_token: Some("tok".to_owned()),
            verified_claims: Some(TokenClaims {
                subject: "test-primal".to_owned(),
                scopes: vec!["discovery.*".to_owned()],
                expires_at: None,
            }),
            peer: None,
            origin: ConnectionOrigin::Loopback,
        };
        assert!(gate.check("discovery.peers", &caller).is_ok());
    }

    #[test]
    fn verified_claims_wrong_scope_rejected_enforced() {
        let gate = MethodGate::new(EnforcementMode::Enforced);
        let caller = CallerContext {
            bearer_token: Some("tok".to_owned()),
            verified_claims: Some(TokenClaims {
                subject: "test-primal".to_owned(),
                scopes: vec!["mesh.*".to_owned()],
                expires_at: None,
            }),
            peer: None,
            origin: ConnectionOrigin::Loopback,
        };
        let err = gate.check("discovery.peers", &caller).unwrap_err();
        assert_eq!(err.code, error_codes::PERMISSION_DENIED);
        assert!(err.message.contains("scope"));
    }

    #[test]
    fn verified_claims_wrong_scope_allowed_permissive() {
        let gate = MethodGate::new(EnforcementMode::Permissive);
        let caller = CallerContext {
            bearer_token: Some("tok".to_owned()),
            verified_claims: Some(TokenClaims {
                subject: "test-primal".to_owned(),
                scopes: vec!["mesh.*".to_owned()],
                expires_at: None,
            }),
            peer: None,
            origin: ConnectionOrigin::Loopback,
        };
        assert!(gate.check("discovery.peers", &caller).is_ok());
    }
}
