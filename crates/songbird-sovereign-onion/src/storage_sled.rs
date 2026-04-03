// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Sled-backed persistent storage for onion identity and peer info.

use crate::error::Result;
use crate::keys::OnionIdentity;
use crate::storage::{OnionStorageBackend, PeerInfo};
use std::path::Path;
use std::sync::Arc;

/// Persistent storage for onion service (sled).
#[derive(Clone)]
pub struct OnionStorage {
    db: Arc<sled::Db>,
}

impl OnionStorage {
    /// Open or create storage at specified path
    ///
    /// # Errors
    ///
    /// Returns error if sled fails to open database.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use songbird_sovereign_onion::OnionStorage;
    ///
    /// let storage = OnionStorage::open("./data/onion").unwrap();
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let db = sled::open(path)?;
        Ok(Self {
            db: Arc::new(db),
        })
    }

    /// Create in-memory storage (for testing)
    ///
    /// # Errors
    ///
    /// Returns error if sled fails to create temporary database.
    #[cfg(test)]
    pub fn memory() -> Result<Self> {
        let config = sled::Config::new().temporary(true);
        let db = config.open()?;
        Ok(Self {
            db: Arc::new(db),
        })
    }

    /// Load or generate onion identity via `security provider` (TRUE PRIMAL)
    ///
    /// If identity exists in storage, loads it via `security provider`. Otherwise generates
    /// new identity via `security provider` and stores it.
    ///
    /// # Errors
    ///
    /// Returns error if storage, load, or generation fails.
    pub async fn load_or_generate_identity_via_security_provider(
        &self,
        client: &crate::security_crypto::SecurityCryptoClient,
    ) -> Result<OnionIdentity> {
        const IDENTITY_KEY: &[u8] = b"identity/key";

        if let Some(bytes) = self.db.get(IDENTITY_KEY)? {
            OnionIdentity::from_stored_via_security_provider(client, &bytes).await
        } else {
            let identity = OnionIdentity::generate_via_security_provider(client).await?;
            let bytes = identity.to_stored_bytes();
            self.db.insert(IDENTITY_KEY, bytes)?;
            self.db.flush()?;

            tracing::info!(
                onion_address = %identity.onion_address(),
                "Generated new onion identity via security provider"
            );

            Ok(identity)
        }
    }

    /// Deprecated alias for [`Self::load_or_generate_identity_via_security_provider`].
    #[deprecated(note = "use load_or_generate_identity_via_security_provider")]
    pub async fn load_or_generate_identity_via_beardog(
        &self,
        client: &crate::security_crypto::SecurityCryptoClient,
    ) -> Result<OnionIdentity> {
        self.load_or_generate_identity_via_security_provider(client).await
    }

    /// Load existing identity from storage (production safe)
    ///
    /// Returns None if no identity exists yet.
    ///
    /// # Errors
    ///
    /// Returns error if database read or deserialization fails.
    pub fn load_identity(&self) -> Result<Option<OnionIdentity>> {
        const IDENTITY_KEY: &[u8] = b"identity/key";

        if let Some(bytes) = self.db.get(IDENTITY_KEY)? {
            let identity = OnionIdentity::from_stored_bytes(&bytes)?;
            Ok(Some(identity))
        } else {
            Ok(None)
        }
    }

    /// Store identity to persistent storage (production safe)
    ///
    /// # Errors
    ///
    /// Returns error if serialization or database write fails.
    pub fn store_identity(&self, identity: &OnionIdentity) -> Result<()> {
        const IDENTITY_KEY: &[u8] = b"identity/key";
        let bytes = identity.to_stored_bytes();
        self.db.insert(IDENTITY_KEY, bytes)?;
        self.db.flush()?;
        Ok(())
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

        if let Some(bytes) = self.db.get(IDENTITY_KEY)? {
            OnionIdentity::from_stored_bytes(&bytes)
        } else {
            let identity = OnionIdentity::generate();
            let bytes = identity.to_stored_bytes();
            self.db.insert(IDENTITY_KEY, bytes)?;
            self.db.flush()?;

            tracing::info!(
                onion_address = %identity.onion_address(),
                "Generated new onion identity (standalone)"
            );

            Ok(identity)
        }
    }

    /// Store peer info
    ///
    /// # Errors
    ///
    /// Returns error if serialization or database write fails.
    pub fn store_peer(&self, peer: &PeerInfo) -> Result<()> {
        let key = format!("peers/{}", peer.onion_address);
        let bytes = serde_json::to_vec(peer)?;
        self.db.insert(key.as_bytes(), bytes)?;
        Ok(())
    }

    /// Get peer info by onion address
    ///
    /// # Errors
    ///
    /// Returns error if database read or deserialization fails.
    pub fn get_peer(&self, onion_address: &str) -> Result<Option<PeerInfo>> {
        let key = format!("peers/{onion_address}");
        if let Some(bytes) = self.db.get(key.as_bytes())? {
            let peer: PeerInfo = serde_json::from_slice(&bytes)?;
            Ok(Some(peer))
        } else {
            Ok(None)
        }
    }

    /// List all known peers
    ///
    /// # Errors
    ///
    /// Returns error if database iteration or deserialization fails.
    pub fn list_peers(&self) -> Result<Vec<PeerInfo>> {
        let prefix = b"peers/";
        let mut peers = Vec::new();

        for result in self.db.scan_prefix(prefix) {
            let (_key, value) = result?;
            let peer: PeerInfo = serde_json::from_slice(&value)?;
            peers.push(peer);
        }

        Ok(peers)
    }

    /// Update peer's last seen timestamp
    ///
    /// # Errors
    ///
    /// Returns error if `get_peer` or `store_peer` fails.
    pub fn update_peer_last_seen(&self, onion_address: &str, timestamp: u64) -> Result<()> {
        if let Some(mut peer) = self.get_peer(onion_address)? {
            peer.last_seen = timestamp;
            self.store_peer(&peer)?;
        }
        Ok(())
    }

    /// Remove peer
    ///
    /// # Errors
    ///
    /// Returns error if database remove fails.
    pub fn remove_peer(&self, onion_address: &str) -> Result<()> {
        let key = format!("peers/{onion_address}");
        self.db.remove(key.as_bytes())?;
        Ok(())
    }

    /// Clear all data (test only)
    ///
    /// # Errors
    ///
    /// Returns error if database clear fails.
    #[cfg(test)]
    pub fn clear_all(&self) -> Result<()> {
        self.db.clear()?;
        Ok(())
    }

    /// Flush to disk
    ///
    /// # Errors
    ///
    /// Returns error if database flush fails.
    pub fn flush(&self) -> Result<()> {
        self.db.flush()?;
        Ok(())
    }
}

impl OnionStorageBackend for OnionStorage {
    fn load_identity(&self) -> Result<Option<OnionIdentity>> {
        Self::load_identity(self)
    }

    fn store_identity(&self, identity: &OnionIdentity) -> Result<()> {
        Self::store_identity(self, identity)
    }

    fn store_peer(&self, peer: &PeerInfo) -> Result<()> {
        Self::store_peer(self, peer)
    }

    fn get_peer(&self, onion_address: &str) -> Result<Option<PeerInfo>> {
        Self::get_peer(self, onion_address)
    }

    fn list_peers(&self) -> Result<Vec<PeerInfo>> {
        Self::list_peers(self)
    }

    fn update_peer_last_seen(&self, onion_address: &str, timestamp: u64) -> Result<()> {
        Self::update_peer_last_seen(self, onion_address, timestamp)
    }

    fn remove_peer(&self, onion_address: &str) -> Result<()> {
        Self::remove_peer(self, onion_address)
    }

    fn flush(&self) -> Result<()> {
        Self::flush(self)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use crate::storage::PeerInfo;
    use serde_json::json;

    #[test]
    fn memory_storage_opens_and_load_identity_is_none() {
        let s = OnionStorage::memory().expect("memory db");
        assert!(s.load_identity().expect("query").is_none(), "empty sled store has no identity");
    }

    #[test]
    fn store_and_load_identity_roundtrip() {
        let s = OnionStorage::memory().expect("memory db");
        let j = json!({
            "secret_key_bytes": vec![5u8; 32],
            "public_key_bytes": vec![6u8; 32],
            "onion_address": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.onion",
            "created_at": 99u64
        });
        let bytes = serde_json::to_vec(&j).expect("fixture");
        let id = crate::OnionIdentity::from_stored_bytes(&bytes).expect("identity");
        s.store_identity(&id).expect("store");
        let loaded = s.load_identity().expect("load").expect("some");
        assert_eq!(loaded.onion_address(), id.onion_address(), "onion address");
    }

    #[test]
    fn peer_crud_and_flush() {
        let s = OnionStorage::memory().expect("memory db");
        let p = PeerInfo {
            onion_address: "peer.onion".to_string(),
            last_seen: 1,
            actual_addr: None,
        };
        s.store_peer(&p).expect("store peer");
        s.flush().expect("flush");
        let got = s.get_peer("peer.onion").expect("get").expect("peer");
        assert_eq!(got.onion_address, p.onion_address, "roundtrip");
        assert_eq!(s.list_peers().expect("list").len(), 1, "one peer");
        s.remove_peer("peer.onion").expect("remove");
        assert!(s.get_peer("peer.onion").expect("get").is_none());
    }

    #[test]
    fn clear_all_empties_database() {
        let s = OnionStorage::memory().expect("memory db");
        let j = json!({
            "secret_key_bytes": vec![1u8; 32],
            "public_key_bytes": vec![2u8; 32],
            "onion_address": "cccccccccccccccccccccccccccccccccccccccccccccccccccccc.onion",
            "created_at": 0u64
        });
        let bytes = serde_json::to_vec(&j).expect("fixture");
        let id = crate::OnionIdentity::from_stored_bytes(&bytes).expect("identity");
        s.store_identity(&id).expect("store");
        s.clear_all().expect("clear");
        assert!(s.load_identity().expect("load").is_none(), "cleared");
    }
}
