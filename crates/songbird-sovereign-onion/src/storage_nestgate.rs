// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Onion persistence via NestGate JSON-RPC `storage.*` (SB-03).
//!
//! Uses the same newline-delimited JSON-RPC 2.0 framing as `songbird-universal-ipc`'s Tower Atomic client,
//! implemented with a direct [`tokio::net::UnixStream`] so this crate stays independent of that crate
//! (`songbird-universal-ipc` depends on sovereign-onion, so it cannot be a dependency here).

use crate::error::{OnionError, Result};
use crate::keys::OnionIdentity;
use crate::storage::{OnionStorageBackend, PeerInfo};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::debug;

const IDENTITY_KEY: &str = "songbird-onion/identity/primary";

fn peer_key(addr: &str) -> String {
    format!("songbird-onion/peer/{addr}")
}

/// NestGate-backed onion storage (sync trait over async JSON-RPC).
pub struct NestGateOnionStorage {
    socket_path: PathBuf,
}

impl NestGateOnionStorage {
    /// Create a backend targeting the given Unix socket.
    #[must_use]
    pub fn new(socket_path: PathBuf) -> Self {
        Self {
            socket_path,
        }
    }

    /// Socket path used for IPC.
    #[must_use]
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    fn run_ipc<F, T>(fut: F) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>>,
    {
        match tokio::runtime::Handle::try_current() {
            Ok(h) => h.block_on(fut),
            Err(_) => tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| OnionError::Other(e.to_string()))?
                .block_on(fut),
        }
    }

    fn rpc(&self, method: &str, params: Value) -> Result<Value> {
        let path = self.socket_path.clone();
        let method = method.to_string();
        Self::run_ipc(async move {
            debug!(method = %method, path = %path.display(), "NestGate onion JSON-RPC");
            let mut stream = UnixStream::connect(&path)
                .await
                .map_err(|e| OnionError::ConnectionError(format!("{}: {e}", path.display())))?;
            let id = 1u64;
            let request = json!({
                "jsonrpc": "2.0",
                "method": method,
                "params": params,
                "id": id,
            });
            let mut payload =
                serde_json::to_vec(&request).map_err(|e| OnionError::Serialization(e))?;
            payload.push(b'\n');
            stream
                .write_all(&payload)
                .await
                .map_err(|e| OnionError::ConnectionError(e.to_string()))?;
            let mut line = String::new();
            BufReader::new(&mut stream)
                .read_line(&mut line)
                .await
                .map_err(|e| OnionError::ConnectionError(e.to_string()))?;
            let v: Value =
                serde_json::from_str(line.trim()).map_err(|e| OnionError::Serialization(e))?;
            if let Some(err) = v.get("error") {
                let msg = err.get("message").and_then(|m| m.as_str()).unwrap_or("JSON-RPC error");
                return Err(OnionError::RpcError(msg.to_string()));
            }
            v.get("result")
                .cloned()
                .ok_or_else(|| OnionError::RpcError("missing JSON-RPC result".to_string()))
        })
    }

    fn storage_put_str(&self, key: &str, value: &str) -> Result<()> {
        debug!(key, "NestGate onion storage.put");
        self.rpc("storage.put", json!({ "key": key, "value": value }))?;
        Ok(())
    }

    fn storage_get_str(&self, key: &str) -> Result<Option<String>> {
        debug!(key, "NestGate onion storage.get");
        let v = self.rpc("storage.get", json!({ "key": key }))?;
        parse_get_value_string(&v).map_err(OnionError::Other)
    }

    fn storage_delete(&self, key: &str) -> Result<()> {
        self.rpc("storage.delete", json!({ "key": key }))?;
        Ok(())
    }

    fn storage_list_keys(&self, prefix: &str) -> Result<Vec<String>> {
        let v = self.rpc("storage.list", json!({ "prefix": prefix }))?;
        parse_list_keys(&v).map_err(OnionError::Other)
    }

    fn storage_flush(&self) -> Result<()> {
        self.rpc("storage.flush", json!({}))?;
        Ok(())
    }
}

#[allow(clippy::unnecessary_wraps)]
fn parse_get_value_string(result: &Value) -> std::result::Result<Option<String>, String> {
    if result.is_null() {
        return Ok(None);
    }
    if let Some(s) = result.as_str() {
        if s.is_empty() {
            return Ok(None);
        }
        return Ok(Some(s.to_string()));
    }
    if let Some(inner) = result.get("value") {
        if inner.is_null() {
            return Ok(None);
        }
        if let Some(s) = inner.as_str() {
            return Ok(Some(s.to_string()));
        }
        return Ok(Some(inner.to_string()));
    }
    Ok(Some(result.to_string()))
}

fn parse_list_keys(result: &Value) -> std::result::Result<Vec<String>, String> {
    if let Some(arr) = result.as_array() {
        return arr
            .iter()
            .map(|v| {
                v.as_str()
                    .map(std::string::ToString::to_string)
                    .ok_or_else(|| "list: expected string keys".to_string())
            })
            .collect();
    }
    if let Some(keys) = result.get("keys").and_then(|v| v.as_array()) {
        return keys
            .iter()
            .map(|v| {
                v.as_str()
                    .map(std::string::ToString::to_string)
                    .ok_or_else(|| "list.keys: expected strings".to_string())
            })
            .collect();
    }
    Err("storage.list: unexpected result shape".to_string())
}

impl OnionStorageBackend for NestGateOnionStorage {
    fn load_identity(&self) -> Result<Option<OnionIdentity>> {
        let Some(s) = self.storage_get_str(IDENTITY_KEY)? else {
            return Ok(None);
        };
        let bytes: Vec<u8> = serde_json::from_str(&s)?;
        Ok(Some(OnionIdentity::from_stored_bytes(&bytes)?))
    }

    fn store_identity(&self, identity: &OnionIdentity) -> Result<()> {
        let bytes = identity.to_stored_bytes()?;
        let json = serde_json::to_string(&bytes)?;
        self.storage_put_str(IDENTITY_KEY, &json)
    }

    fn store_peer(&self, peer: &PeerInfo) -> Result<()> {
        let json = serde_json::to_string(peer)?;
        self.storage_put_str(&peer_key(&peer.onion_address), &json)
    }

    fn get_peer(&self, onion_address: &str) -> Result<Option<PeerInfo>> {
        let Some(s) = self.storage_get_str(&peer_key(onion_address))? else {
            return Ok(None);
        };
        let p: PeerInfo = serde_json::from_str(&s)?;
        Ok(Some(p))
    }

    fn list_peers(&self) -> Result<Vec<PeerInfo>> {
        let keys = self.storage_list_keys("songbird-onion/peer/")?;
        let mut out = Vec::new();
        for k in keys {
            let Some(addr) = k.strip_prefix("songbird-onion/peer/") else {
                continue;
            };
            if let Some(p) = self.get_peer(addr)? {
                out.push(p);
            }
        }
        Ok(out)
    }

    fn update_peer_last_seen(&self, onion_address: &str, timestamp: u64) -> Result<()> {
        if let Some(mut peer) = self.get_peer(onion_address)? {
            peer.last_seen = timestamp;
            self.store_peer(&peer)?;
        }
        Ok(())
    }

    fn remove_peer(&self, onion_address: &str) -> Result<()> {
        self.storage_delete(&peer_key(onion_address))
    }

    fn flush(&self) -> Result<()> {
        self.storage_flush()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use serde_json::json;

    #[test]
    fn parse_get_wire_object() {
        let v = json!({ "value": "[1,2,3]" });
        let s = parse_get_value_string(&v).unwrap();
        assert_eq!(s.as_deref(), Some("[1,2,3]"));
    }

    #[test]
    fn parse_list_wire_keys() {
        let v = json!({ "keys": ["songbird-onion/peer/a.onion"] });
        let k = parse_list_keys(&v).unwrap();
        assert_eq!(k.len(), 1);
    }

    #[test]
    fn peer_key_namespacing() {
        assert_eq!(peer_key("x.onion"), "songbird-onion/peer/x.onion");
    }
}
