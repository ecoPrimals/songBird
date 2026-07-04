// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Onion identity and peer persistence (IPC storage provider production path; in-memory fallback).

use crate::error::Result;
use crate::keys::OnionIdentity;
use crate::storage_ipc::IpcOnionStorage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Sync persistence backend for onion service data.
///
/// Production path: [`IpcOnionStorage`] delegates
/// to the `storage.*` capability provider via JSON-RPC at runtime.
/// Fallback: [`InMemoryOnionStorage`] when no provider is available.
pub trait OnionStorageBackend: Send + Sync {
    /// Load an existing identity from persistent storage.
    fn load_identity(&self) -> Result<Option<OnionIdentity>>;

    /// Store identity to persistent storage.
    fn store_identity(&self, identity: &OnionIdentity) -> Result<()>;

    /// Store peer info.
    fn store_peer(&self, peer: &PeerInfo) -> Result<()>;

    /// Get peer info by onion address.
    fn get_peer(&self, onion_address: &str) -> Result<Option<PeerInfo>>;

    /// List all known peers.
    fn list_peers(&self) -> Result<Vec<PeerInfo>>;

    /// Update peer's last seen timestamp.
    fn update_peer_last_seen(&self, onion_address: &str, timestamp: u64) -> Result<()>;

    /// Remove peer by onion address.
    fn remove_peer(&self, onion_address: &str) -> Result<()>;

    /// Flush to durable storage.
    fn flush(&self) -> Result<()>;
}

/// Peer information stored in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Peer's .onion address
    pub onion_address: String,
    /// Last seen timestamp (Unix seconds)
    pub last_seen: u64,
    /// Optional actual IP:port (if known)
    pub actual_addr: Option<String>,
}

/// In-memory onion storage (fallback when no storage provider is available).
#[derive(Debug, Clone)]
pub struct InMemoryOnionStorage {
    identity: Arc<RwLock<Option<Vec<u8>>>>,
    peers: Arc<RwLock<HashMap<String, PeerInfo>>>,
}

impl InMemoryOnionStorage {
    /// Create empty in-memory storage.
    #[must_use]
    pub fn new() -> Self {
        Self {
            identity: Arc::new(RwLock::new(None)),
            peers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Load or generate onion identity (STANDALONE mode - testing only)
    ///
    /// ⚠️ **TRUE PRIMAL NOTE**: This method uses direct crypto and should ONLY be
    /// used for testing! Production code should use `load_or_generate_identity_via_security_provider()`.
    ///
    /// # Errors
    ///
    /// Returns an error if database access, deserialization, or persistence fails.
    #[cfg(feature = "standalone")]
    pub fn load_or_generate_identity(&self) -> Result<OnionIdentity> {
        const IDENTITY_KEY: &[u8] = b"identity/key";
        let _ = IDENTITY_KEY;

        let mut slot = self
            .identity
            .write()
            .map_err(|e| crate::error::OnionError::Other(format!("identity lock poisoned: {e}")))?;

        if let Some(ref bytes) = *slot {
            return OnionIdentity::from_stored_bytes(bytes);
        }

        let identity = OnionIdentity::generate();
        let bytes = identity.to_stored_bytes()?;
        *slot = Some(bytes);

        tracing::info!(
            onion_address = %identity.onion_address(),
            "Generated new onion identity (standalone, in-memory)"
        );

        Ok(identity)
    }

    /// Load existing identity if the in-memory slot was populated from serialized bytes.
    fn load_identity_bytes(&self) -> Result<Option<Vec<u8>>> {
        let g = self
            .identity
            .read()
            .map_err(|e| crate::error::OnionError::Other(format!("identity lock poisoned: {e}")))?;
        Ok(g.clone())
    }

    fn store_identity_bytes(&self, bytes: Vec<u8>) -> Result<()> {
        let mut g = self
            .identity
            .write()
            .map_err(|e| crate::error::OnionError::Other(format!("identity lock poisoned: {e}")))?;
        *g = Some(bytes);
        Ok(())
    }
}

impl Default for InMemoryOnionStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl OnionStorageBackend for InMemoryOnionStorage {
    fn load_identity(&self) -> Result<Option<OnionIdentity>> {
        if let Some(bytes) = self.load_identity_bytes()? {
            Ok(Some(OnionIdentity::from_stored_bytes(&bytes)?))
        } else {
            Ok(None)
        }
    }

    fn store_identity(&self, identity: &OnionIdentity) -> Result<()> {
        self.store_identity_bytes(identity.to_stored_bytes()?)
    }

    fn store_peer(&self, peer: &PeerInfo) -> Result<()> {
        let mut m = self
            .peers
            .write()
            .map_err(|e| crate::error::OnionError::Other(format!("peers lock poisoned: {e}")))?;
        m.insert(peer.onion_address.clone(), peer.clone());
        Ok(())
    }

    fn get_peer(&self, onion_address: &str) -> Result<Option<PeerInfo>> {
        let m = self
            .peers
            .read()
            .map_err(|e| crate::error::OnionError::Other(format!("peers lock poisoned: {e}")))?;
        Ok(m.get(onion_address).cloned())
    }

    fn list_peers(&self) -> Result<Vec<PeerInfo>> {
        let m = self
            .peers
            .read()
            .map_err(|e| crate::error::OnionError::Other(format!("peers lock poisoned: {e}")))?;
        Ok(m.values().cloned().collect())
    }

    fn update_peer_last_seen(&self, onion_address: &str, timestamp: u64) -> Result<()> {
        if let Some(mut peer) = self.get_peer(onion_address)? {
            peer.last_seen = timestamp;
            self.store_peer(&peer)?;
        }
        Ok(())
    }

    fn remove_peer(&self, onion_address: &str) -> Result<()> {
        let mut m = self
            .peers
            .write()
            .map_err(|e| crate::error::OnionError::Other(format!("peers lock poisoned: {e}")))?;
        m.remove(onion_address);
        Ok(())
    }

    fn flush(&self) -> Result<()> {
        Ok(())
    }
}

/// Concrete onion storage backend (enum dispatch over [`InMemoryOnionStorage`] and [`IpcOnionStorage`]).
#[derive(Debug)]
pub enum OnionStorage {
    /// In-memory fallback when no `storage.*` provider is available.
    InMemory(InMemoryOnionStorage),
    /// IPC JSON-RPC `storage.*` capability (production).
    Ipc(IpcOnionStorage),
}

impl OnionStorageBackend for OnionStorage {
    fn load_identity(&self) -> Result<Option<OnionIdentity>> {
        match self {
            Self::InMemory(s) => s.load_identity(),
            Self::Ipc(s) => s.load_identity(),
        }
    }

    fn store_identity(&self, identity: &OnionIdentity) -> Result<()> {
        match self {
            Self::InMemory(s) => s.store_identity(identity),
            Self::Ipc(s) => s.store_identity(identity),
        }
    }

    fn store_peer(&self, peer: &PeerInfo) -> Result<()> {
        match self {
            Self::InMemory(s) => s.store_peer(peer),
            Self::Ipc(s) => s.store_peer(peer),
        }
    }

    fn get_peer(&self, onion_address: &str) -> Result<Option<PeerInfo>> {
        match self {
            Self::InMemory(s) => s.get_peer(onion_address),
            Self::Ipc(s) => s.get_peer(onion_address),
        }
    }

    fn list_peers(&self) -> Result<Vec<PeerInfo>> {
        match self {
            Self::InMemory(s) => s.list_peers(),
            Self::Ipc(s) => s.list_peers(),
        }
    }

    fn update_peer_last_seen(&self, onion_address: &str, timestamp: u64) -> Result<()> {
        match self {
            Self::InMemory(s) => s.update_peer_last_seen(onion_address, timestamp),
            Self::Ipc(s) => s.update_peer_last_seen(onion_address, timestamp),
        }
    }

    fn remove_peer(&self, onion_address: &str) -> Result<()> {
        match self {
            Self::InMemory(s) => s.remove_peer(onion_address),
            Self::Ipc(s) => s.remove_peer(onion_address),
        }
    }

    fn flush(&self) -> Result<()> {
        match self {
            Self::InMemory(s) => s.flush(),
            Self::Ipc(s) => s.flush(),
        }
    }
}

#[cfg(all(test, feature = "standalone"))]
mod standalone_tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn test_inmemory_identity_persistence() {
        let storage = InMemoryOnionStorage::new();

        let identity1 = storage.load_or_generate_identity().unwrap();
        let identity2 = storage.load_or_generate_identity().unwrap();

        assert_eq!(identity1.onion_address(), identity2.onion_address());
        assert_eq!(identity1.created_at(), identity2.created_at());
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;

    fn test_backend(storage: &OnionStorage) {
        let peer = PeerInfo {
            onion_address: String::from("test.onion"),
            last_seen: 1_234_567_890,
            actual_addr: Some(String::from("192.168.1.100:9735")),
        };

        storage.store_peer(&peer).unwrap();

        let retrieved = storage.get_peer("test.onion").unwrap().unwrap();
        assert_eq!(retrieved.onion_address, peer.onion_address);
        assert_eq!(retrieved.last_seen, peer.last_seen);
        assert_eq!(retrieved.actual_addr, peer.actual_addr);

        storage.update_peer_last_seen("test.onion", 9_999_999_999).unwrap();
        let updated = storage.get_peer("test.onion").unwrap().unwrap();
        assert_eq!(updated.last_seen, 9_999_999_999);

        let peers = storage.list_peers().unwrap();
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0].onion_address, "test.onion");

        storage.remove_peer("test.onion").unwrap();
        assert!(storage.get_peer("test.onion").unwrap().is_none());
    }

    #[test]
    fn test_inmemory_peer_operations() {
        test_backend(&OnionStorage::InMemory(InMemoryOnionStorage::new()));
    }

    #[test]
    fn load_identity_returns_none_when_empty() {
        let storage = InMemoryOnionStorage::new();
        assert!(storage.load_identity().unwrap().is_none(), "fresh storage has no identity");
    }

    #[test]
    fn peer_info_roundtrips_through_serde() {
        let p = PeerInfo {
            onion_address: String::from("peer.onion"),
            last_seen: 100,
            actual_addr: Some(String::from("127.0.0.1:1")),
        };
        let json = serde_json::to_string(&p).expect("serialize");
        let q: PeerInfo = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(q.onion_address, p.onion_address, "address");
        assert_eq!(q.last_seen, p.last_seen, "last_seen");
        assert_eq!(q.actual_addr, p.actual_addr, "actual_addr");
    }

    #[test]
    fn update_peer_last_seen_is_noop_when_peer_missing() {
        let storage = InMemoryOnionStorage::new();
        storage.update_peer_last_seen("ghost.onion", 99).expect("update should not error");
        assert!(
            storage.get_peer("ghost.onion").unwrap().is_none(),
            "missing peer should stay absent"
        );
    }

    #[test]
    fn store_identity_and_load_roundtrip() {
        let storage = InMemoryOnionStorage::new();
        let j = serde_json::json!({
            "secret_key_bytes": vec![7u8; 32],
            "public_key_bytes": vec![8u8; 32],
            "onion_address": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.onion",
            "created_at": 11u64
        });
        let bytes = serde_json::to_vec(&j).expect("fixture");
        let id = crate::OnionIdentity::from_stored_bytes(&bytes).expect("identity");
        storage.store_identity(&id).expect("store");
        let loaded = storage.load_identity().expect("load").expect("some");
        assert_eq!(loaded.onion_address(), id.onion_address(), "onion address");
    }

    #[test]
    fn test_inmemory_multiple_peers() {
        let storage = InMemoryOnionStorage::new();

        for i in 0..5 {
            let peer = PeerInfo {
                onion_address: format!("peer{i}.onion"),
                last_seen: 1_234_567_890 + i,
                actual_addr: None,
            };
            storage.store_peer(&peer).unwrap();
        }

        let peers = storage.list_peers().unwrap();
        assert_eq!(peers.len(), 5);

        let mut addresses: Vec<String> = peers.iter().map(|p| p.onion_address.clone()).collect();
        addresses.sort();
        addresses.dedup();
        assert_eq!(addresses.len(), 5);
    }

    #[test]
    #[allow(clippy::unwrap_used, reason = "test assertion")]
    fn store_peer_overwrites_same_onion_address() {
        let storage = InMemoryOnionStorage::new();
        let a = PeerInfo {
            onion_address: String::from("dup.onion"),
            last_seen: 1,
            actual_addr: Some(String::from("10.0.0.1:1")),
        };
        let b = PeerInfo {
            onion_address: String::from("dup.onion"),
            last_seen: 2,
            actual_addr: None,
        };
        storage.store_peer(&a).unwrap();
        storage.store_peer(&b).unwrap();
        let got = storage.get_peer("dup.onion").unwrap().expect("peer");
        assert_eq!(got.last_seen, 2);
        assert_eq!(got.actual_addr, None);
    }

    #[test]
    #[allow(clippy::unwrap_used, reason = "test assertion")]
    fn remove_peer_missing_is_noop() {
        let storage = InMemoryOnionStorage::new();
        storage.remove_peer("nope.onion").unwrap();
        assert!(storage.list_peers().unwrap().is_empty());
    }

    #[test]
    #[allow(clippy::unwrap_used, reason = "test assertion")]
    fn flush_on_in_memory_is_ok() {
        let storage = InMemoryOnionStorage::new();
        storage.flush().unwrap();
    }
}
