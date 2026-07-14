// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Onion persistence via IPC JSON-RPC `storage.*` capability (SB-03).
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
use tracing::debug;

#[cfg(unix)]
use tokio::net::UnixStream as IpcStream;
#[cfg(windows)]
use tokio::net::TcpStream as IpcStream;

const IDENTITY_KEY: &str = "songbird-onion/identity/primary";

fn peer_key(addr: &str) -> String {
    format!("songbird-onion/peer/{addr}")
}

/// IPC-backed onion storage (sync trait over async JSON-RPC `storage.*` capability).
///
/// Connects to whichever primal provides the `storage.*` capability domain via a
/// Unix socket discovered at runtime. No primal identity assumed.
#[derive(Debug)]
pub struct IpcOnionStorage {
    socket_path: PathBuf,
}

impl IpcOnionStorage {
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

    /// Connect to the IPC endpoint. Unix uses the socket path directly;
    /// Windows interprets it as a TCP port file (same convention as biomeOS sidecar).
    #[cfg(unix)]
    async fn connect_ipc(path: &Path) -> Result<IpcStream> {
        IpcStream::connect(path)
            .await
            .map_err(|e| OnionError::ConnectionError(format!("{}: {e}", path.display())))
    }

    /// Connect to the IPC endpoint via TCP localhost on Windows.
    /// The socket_path is treated as a file containing the TCP port number,
    /// or as a fallback the path basename is parsed as `<name>-<port>`.
    #[cfg(windows)]
    async fn connect_ipc(path: &Path) -> Result<IpcStream> {
        let port: u16 = std::fs::read_to_string(path)
            .ok()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(songbird_types::defaults::ports::DEFAULT_HTTP_PORT);
        let addr = format!("127.0.0.1:{port}");
        IpcStream::connect(&addr)
            .await
            .map_err(|e| OnionError::ConnectionError(format!("{addr}: {e}")))
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
            debug!(method = %method, path = %path.display(), "onion storage capability RPC");
            let mut stream = Self::connect_ipc(&path).await?;
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
                .ok_or_else(|| OnionError::RpcError(String::from("missing JSON-RPC result")))
        })
    }

    fn storage_put_str(&self, key: &str, value: &str) -> Result<()> {
        debug!(key, "onion storage.put");
        self.rpc("storage.put", json!({ "key": key, "value": value }))?;
        Ok(())
    }

    fn storage_get_str(&self, key: &str) -> Result<Option<String>> {
        debug!(key, "onion storage.get");
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

#[expect(
    clippy::unnecessary_wraps,
    reason = "Result matches rpc() error mapping; Ok-wrapping kept for uniform call sites"
)]
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
                    .ok_or_else(|| String::from("list: expected string keys"))
            })
            .collect();
    }
    if let Some(keys) = result.get("keys").and_then(|v| v.as_array()) {
        return keys
            .iter()
            .map(|v| {
                v.as_str()
                    .map(std::string::ToString::to_string)
                    .ok_or_else(|| String::from("list.keys: expected strings"))
            })
            .collect();
    }
    Err(String::from("storage.list: unexpected result shape"))
}

impl OnionStorageBackend for IpcOnionStorage {
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
