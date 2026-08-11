// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Shared BTSP tunnel RPC transport layer.
//!
//! Provides the common JSON-RPC over BTSP tunnel implementation used by all
//! three BTSP connection types (`FullTrustBtsp`, `LimitedBtsp`, `FederatedBtsp`).
//!
//! ## Status
//!
//! **Blocked on security provider v0.16.0+** — the `BtspClient` does not yet
//! expose `send_data_over_tunnel()`. When that API ships, this module becomes
//! the single integration point for all BTSP bidirectional RPC.

use anyhow::Result;
use serde_json::Value;
use songbird_types::SongbirdError;
use tracing::debug;

use crate::btsp_client::BtspClient;

/// Send a JSON-RPC request over an established BTSP tunnel and return the response.
///
/// ## Protocol
///
/// 1. Serializes a JSON-RPC 2.0 request (`method`, `params`, auto-generated `id`)
/// 2. Sends encrypted bytes through the tunnel via `BtspClient::send_data_over_tunnel`
/// 3. Receives the JSON-RPC response bytes from the remote peer
/// 4. Parses and returns the `result` field, or propagates the `error`
///
/// ## Current State
///
/// Returns `SongbirdError::not_implemented` until security provider v0.16.0+
/// provides `btsp.tunnel_send` / `btsp.tunnel_exchange` RPC method.
pub async fn send_btsp_jsonrpc(
    _btsp_client: &BtspClient,
    tunnel_id: &str,
    operation: &str,
    request: Value,
) -> Result<Value> {
    let rpc_id = uuid::Uuid::new_v4().to_string();
    let _rpc_request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": operation,
        "params": request,
        "id": rpc_id,
    });

    debug!("📡 BTSP tunnel RPC {tunnel_id}: {operation}");

    // Phase 2 implementation (when security provider v0.16.0+ ships):
    //
    // let request_bytes = serde_json::to_vec(&rpc_request)?;
    // let response_bytes = btsp_client
    //     .send_data_over_tunnel(tunnel_id, &request_bytes, BTSP_RPC_TIMEOUT)
    //     .await
    //     .context("BTSP tunnel RPC send failed")?;
    // let response: Value = serde_json::from_slice(&response_bytes)?;
    // if let Some(error) = response.get("error") {
    //     return Err(anyhow::anyhow!("Remote JSON-RPC error: {error}"));
    // }
    // response.get("result")
    //     .cloned()
    //     .ok_or_else(|| anyhow::anyhow!("Missing result in BTSP RPC response"))

    Err(SongbirdError::not_implemented_with_detail(
        "btsp_bidirectional_rpc",
        "Requires security provider v0.16.0+ with BtspClient::send_data_over_tunnel(); \
         the server-side BTSP frame handler is ready — only the client tunnel transport is pending.",
    )
    .into())
}
