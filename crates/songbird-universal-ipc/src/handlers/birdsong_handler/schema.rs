// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use super::BirdSongHandler;
use serde_json::{Value, json};
use tracing::debug;

impl BirdSongHandler {
    /// Handle `birdsong.schema`
    ///
    /// Returns the beacon request schema: field names, types, required/optional.
    /// Clients can use this to generate beacon requests programmatically
    /// without hardcoding field lists.
    pub async fn handle_schema(&self, _params: Value) -> Result<Value, String> {
        debug!("🌲 RPC: birdsong.schema");

        Ok(json!({
            "method": "birdsong.generate_encrypted_beacon",
            "description": "Generate a family-encrypted discovery beacon for Dark Forest broadcast",
            "fields": [
                {
                    "name": "node_id",
                    "type": "string",
                    "required": true,
                    "description": "Unique node identifier for this primal instance"
                },
                {
                    "name": "capabilities",
                    "type": "array<string>",
                    "required": false,
                    "default": "[]",
                    "description": "Capability tokens this node advertises (e.g. network.discovery, crypto.delegate)"
                },
                {
                    "name": "onion_endpoint",
                    "type": "string | null",
                    "required": false,
                    "default": "null",
                    "description": "Sovereign Onion endpoint (e.g. abc123...xyz.onion:3492). Dark Forest: only visible to family members"
                },
                {
                    "name": "endpoint_hints",
                    "type": "object | null",
                    "required": false,
                    "default": "null",
                    "description": "Additional endpoint hints (LAN IP, port, relay addresses, etc.)"
                }
            ],
            "related_methods": [
                "birdsong.decrypt_beacon",
                "birdsong.verify_lineage",
                "birdsong.get_lineage",
                "birdsong.advertise"
            ],
            "version": env!("CARGO_PKG_VERSION")
        }))
    }
}
