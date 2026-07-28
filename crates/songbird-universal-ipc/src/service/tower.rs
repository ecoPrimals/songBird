// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! `tower.*` — Tower Atomic stack health facade for biomeOS signal graphs.
//!
//! Aggregates health from the Tower Atomic stack (songBird + crypto provider +
//! mesh connectivity) into a single response that biomeOS can consume for
//! `tower.health` signal graph validation.

use super::IpcServiceHandler;
use serde_json::{Value, json};

impl IpcServiceHandler {
    /// Handle `tower.health` — aggregate Tower Atomic stack health.
    ///
    /// Returns a unified view of:
    /// - Process liveness (always true if responding)
    /// - Crypto provider availability
    /// - Mesh initialization state
    /// - Peer connectivity summary
    /// - Drawbridge proxy readiness
    #[allow(clippy::unused_async)]
    pub(super) async fn handle_tower_health(&self) -> Result<Value, String> {
        let mesh_initialized = self.mesh_handler.is_initialized();
        let peer_count = self.mesh_handler.peer_count();
        let capabilities = self.capability_router.list_capabilities();

        let crypto_socket = songbird_crypto_provider::socket_discovery::discover_security_socket();
        let crypto_available = std::path::Path::new(&crypto_socket).exists();

        let status = if mesh_initialized && crypto_available {
            "healthy"
        } else if mesh_initialized || crypto_available {
            "degraded"
        } else {
            "initializing"
        };

        Ok(json!({
            "status": status,
            "primal": "songbird",
            "version": env!("CARGO_PKG_VERSION"),
            "tower_atomic": {
                "process": "alive",
                "crypto_provider": if crypto_available { "available" } else { "unavailable" },
                "mesh": if mesh_initialized { "active" } else { "awaiting_init" },
                "peers": peer_count,
                "capabilities_registered": capabilities.len(),
                "drawbridge": "ready",
            },
        }))
    }

    /// Handle `tower.mesh_status` — enriched mesh status for Tower validation.
    #[allow(clippy::unused_async)]
    pub(super) async fn handle_tower_mesh_status(&self) -> Result<Value, String> {
        let mesh_initialized = self.mesh_handler.is_initialized();
        let peer_count = self.mesh_handler.peer_count();
        let node_id = self.mesh_handler.node_id();

        Ok(json!({
            "initialized": mesh_initialized,
            "node_id": node_id,
            "peers": peer_count,
            "tower_transport": {
                "ipc": "songbird.sock",
                "federation_port": 7700,
                "drawbridge_port": 7780,
            },
        }))
    }

    /// Handle `acme.challenge_ready` — register an HTTP-01 challenge token.
    ///
    /// bearDog calls this when it needs songBird's drawbridge to serve an
    /// ACME challenge response. Once registered, GET requests to
    /// `/.well-known/acme-challenge/{token}` will return the authorization.
    #[allow(clippy::unused_async)]
    pub(super) async fn handle_acme_challenge_ready(&self, params: Value) -> Result<Value, String> {
        let token = params
            .get("token")
            .and_then(Value::as_str)
            .ok_or_else(|| "missing 'token' parameter".to_string())?;

        let authorization = params
            .get("authorization")
            .and_then(Value::as_str)
            .ok_or_else(|| "missing 'authorization' parameter".to_string())?;

        super::drawbridge::register_acme_challenge(token, authorization);

        Ok(json!({ "registered": true, "token": token }))
    }

    /// Handle `acme.challenge_cleanup` — remove a completed challenge token.
    #[allow(clippy::unused_async)]
    pub(super) async fn handle_acme_challenge_cleanup(
        &self,
        params: Value,
    ) -> Result<Value, String> {
        let token = params
            .get("token")
            .and_then(Value::as_str)
            .ok_or_else(|| "missing 'token' parameter".to_string())?;

        super::drawbridge::remove_acme_challenge(token);

        Ok(json!({ "removed": true, "token": token }))
    }
}
