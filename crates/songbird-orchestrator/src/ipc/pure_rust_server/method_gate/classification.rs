// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Method classification and scope utilities.
//!
//! Classifies JSON-RPC methods as public or protected, and provides
//! scope-matching logic for token-based authorization.

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
