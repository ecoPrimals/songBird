//! Sled-based persistent storage for onion identity and peer info

use crate::error::Result;
use crate::keys::OnionIdentity;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

/// Persistent storage for onion service
#[derive(Clone)]
pub struct OnionStorage {
    db: Arc<sled::Db>,
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

impl OnionStorage {
    /// Open or create storage at specified path
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
        Ok(Self { db: Arc::new(db) })
    }

    /// Create in-memory storage (for testing)
    #[cfg(test)]
    pub fn memory() -> Result<Self> {
        let config = sled::Config::new().temporary(true);
        let db = config.open()?;
        Ok(Self { db: Arc::new(db) })
    }

    /// Load or generate onion identity via BearDog (TRUE PRIMAL)
    ///
    /// If identity exists in storage, loads it via BearDog. Otherwise generates
    /// new identity via BearDog and stores it.
    pub async fn load_or_generate_identity_via_beardog(
        &self,
        client: &crate::beardog_crypto::BeardogCryptoClient,
    ) -> Result<OnionIdentity> {
        const IDENTITY_KEY: &[u8] = b"identity/key";

        if let Some(bytes) = self.db.get(IDENTITY_KEY)? {
            // Load existing identity via BearDog
            OnionIdentity::from_stored_via_beardog(client, &bytes).await
        } else {
            // Generate new identity
            let identity = OnionIdentity::generate_via_beardog(client).await?;
            let bytes = identity.to_stored_bytes();
            self.db.insert(IDENTITY_KEY, bytes)?;
            self.db.flush()?;

            tracing::info!(
                onion_address = %identity.onion_address(),
                "Generated new onion identity via BearDog"
            );

            Ok(identity)
        }
    }

    /// Load existing identity from storage (production safe)
    ///
    /// Returns None if no identity exists yet.
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
    /// used for testing! Production code should use `load_or_generate_identity_via_beardog()`.
    #[cfg(feature = "standalone")]
    pub fn load_or_generate_identity(&self) -> Result<OnionIdentity> {
        const IDENTITY_KEY: &[u8] = b"identity/key";

        if let Some(bytes) = self.db.get(IDENTITY_KEY)? {
            // Load existing identity
            OnionIdentity::from_stored_bytes(&bytes)
        } else {
            // Generate new identity
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
    pub fn store_peer(&self, peer: &PeerInfo) -> Result<()> {
        let key = format!("peers/{}", peer.onion_address);
        let bytes = serde_json::to_vec(peer)?;
        self.db.insert(key.as_bytes(), bytes)?;
        Ok(())
    }

    /// Get peer info by onion address
    pub fn get_peer(&self, onion_address: &str) -> Result<Option<PeerInfo>> {
        let key = format!("peers/{}", onion_address);
        if let Some(bytes) = self.db.get(key.as_bytes())? {
            let peer: PeerInfo = serde_json::from_slice(&bytes)?;
            Ok(Some(peer))
        } else {
            Ok(None)
        }
    }

    /// List all known peers
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
    pub fn update_peer_last_seen(&self, onion_address: &str, timestamp: u64) -> Result<()> {
        if let Some(mut peer) = self.get_peer(onion_address)? {
            peer.last_seen = timestamp;
            self.store_peer(&peer)?;
        }
        Ok(())
    }

    /// Remove peer
    pub fn remove_peer(&self, onion_address: &str) -> Result<()> {
        let key = format!("peers/{}", onion_address);
        self.db.remove(key.as_bytes())?;
        Ok(())
    }

    /// Clear all data (dangerous!)
    #[cfg(test)]
    pub fn clear_all(&self) -> Result<()> {
        self.db.clear()?;
        Ok(())
    }

    /// Flush to disk
    pub fn flush(&self) -> Result<()> {
        self.db.flush()?;
        Ok(())
    }
}

#[cfg(all(test, feature = "standalone"))]
mod standalone_tests {
    use super::*;

    #[test]
    fn test_storage_identity_persistence() {
        let storage = OnionStorage::memory().unwrap();

        // First load: generates new identity
        let identity1 = storage.load_or_generate_identity().unwrap();

        // Second load: should return same identity
        let identity2 = storage.load_or_generate_identity().unwrap();

        assert_eq!(identity1.onion_address(), identity2.onion_address());
        assert_eq!(identity1.created_at(), identity2.created_at());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_peer_operations() {
        let storage = OnionStorage::memory().unwrap();

        let peer = PeerInfo {
            onion_address: "test.onion".to_string(),
            last_seen: 1234567890,
            actual_addr: Some("192.168.1.100:9735".to_string()),
        };

        // Store peer
        storage.store_peer(&peer).unwrap();

        // Get peer
        let retrieved = storage.get_peer("test.onion").unwrap().unwrap();
        assert_eq!(retrieved.onion_address, peer.onion_address);
        assert_eq!(retrieved.last_seen, peer.last_seen);
        assert_eq!(retrieved.actual_addr, peer.actual_addr);

        // Update last seen
        storage
            .update_peer_last_seen("test.onion", 9999999999)
            .unwrap();
        let updated = storage.get_peer("test.onion").unwrap().unwrap();
        assert_eq!(updated.last_seen, 9999999999);

        // List peers
        let peers = storage.list_peers().unwrap();
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0].onion_address, "test.onion");

        // Remove peer
        storage.remove_peer("test.onion").unwrap();
        assert!(storage.get_peer("test.onion").unwrap().is_none());
    }

    #[test]
    fn test_storage_multiple_peers() {
        let storage = OnionStorage::memory().unwrap();

        // Add multiple peers
        for i in 0..5 {
            let peer = PeerInfo {
                onion_address: format!("peer{}.onion", i),
                last_seen: 1234567890 + i,
                actual_addr: None,
            };
            storage.store_peer(&peer).unwrap();
        }

        // List all
        let peers = storage.list_peers().unwrap();
        assert_eq!(peers.len(), 5);

        // Check they're all unique
        let addresses: Vec<String> = peers.iter().map(|p| p.onion_address.clone()).collect();
        let mut sorted = addresses.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), 5); // No duplicates
    }
}
