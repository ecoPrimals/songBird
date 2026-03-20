// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! JSON-RPC bodies for Neural API capability registration.

use serde_json::json;

/// JSON-RPC body for `capability.register` (pure; covered by unit tests).
pub(super) fn capability_registration_params(
    primal_id: &str,
    songbird_socket: &str,
    family_id: &str,
    version: &str,
) -> serde_json::Value {
    json!({
        "jsonrpc": "2.0",
        "method": "capability.register",
        "params": {
            "primal_id": primal_id,
            "capability": "secure_http",
            "socket_path": songbird_socket,
            "operations": [
                "http.get",
                "http.post",
                "http.put",
                "http.delete",
                "http.patch",
                "http.request"
            ],
            "metadata": {
                "tls_version": "1.3",
                "pure_rust": true,
                "supports_http2": true,
                "tower_atomic": true,
                "ecobin_compliant": true,
                "provider": "songbird",
                "family_id": family_id,
                "version": version
            }
        },
        "id": 1
    })
}

/// JSON-RPC body for `capability.unregister` (pure; covered by unit tests).
pub(super) fn capability_unregister_params(primal_id: &str) -> serde_json::Value {
    json!({
        "jsonrpc": "2.0",
        "method": "capability.unregister",
        "params": {
            "primal_id": primal_id,
            "capability": "secure_http"
        },
        "id": 2
    })
}
