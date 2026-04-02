// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use super::IpcServiceHandler;
use serde::Serialize;
use serde_json::Value;

impl IpcServiceHandler {
    pub(super) fn parse_tcp_port(addr: &str) -> Result<u16, String> {
        addr.split(':')
            .next_back()
            .and_then(|p| p.parse().ok())
            .ok_or_else(|| format!("Invalid TCP address: {addr}"))
    }

    pub(super) fn parse_local_tcp_endpoint(endpoint: &str) -> Option<u16> {
        let (host, port_str) = endpoint.rsplit_once(':')?;
        let port: u16 = port_str.parse().ok()?;
        let is_local = matches!(host, "127.0.0.1" | "0.0.0.0" | "localhost" | "::1" | "[::1]");
        is_local.then_some(port)
    }

    /// Serialize a handler result into a JSON-RPC response `Value`.
    pub(super) fn wrap_result<T: Serialize>(
        result: std::result::Result<T, impl std::fmt::Display>,
        context: &str,
    ) -> Result<Value, String> {
        let val = result.map_err(|e| format!("{context}: {e}"))?;
        serde_json::to_value(val).map_err(|e| format!("Serialization error: {e}"))
    }
}
