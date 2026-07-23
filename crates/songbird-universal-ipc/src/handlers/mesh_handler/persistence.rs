// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Mesh Peer Persistence
//!
//! Persists mesh peer state to `<data_dir>/peers.toml` so that mesh
//! connectivity survives process restarts without requiring manual
//! `mesh.init` or `SONGBIRD_PEERS` on every boot.
//!
//! Format: TOML array of `[[peers]]` entries with `node_id` and `address`.

use std::net::SocketAddr;
use std::path::PathBuf;

#[cfg(test)]
use tracing::debug;
use tracing::{info, warn};

/// A single persisted peer entry.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct PersistedPeer {
    node_id: String,
    address: String,
    /// LAN address for same-subnet peers (priority 0 `EndpointType::Local`).
    /// When present, songBird will prefer this path over the WG overlay.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    lan_addr: Option<String>,
}

/// Top-level structure for the peers.toml file.
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
struct PeersFile {
    /// Our node ID at time of last mesh.init
    #[serde(default)]
    node_id: String,
    /// Persisted peer list
    #[serde(default)]
    peers: Vec<PersistedPeer>,
}

/// Resolve the path to the peers persistence file.
fn peers_file_path() -> PathBuf {
    songbird_types::defaults::paths::data_dir().join("peers.toml")
}

/// Persist the current mesh peers to disk.
///
/// Merges with any existing peers (deduplicates by `node_id`, preferring
/// the newer address if it differs).
pub(crate) fn save_peers(node_id: &str, peers: &[(String, SocketAddr)]) {
    let path = peers_file_path();

    let mut file = load_peers_file().unwrap_or_default();
    file.node_id = node_id.to_string();

    for (peer_id, addr) in peers {
        let addr_str = addr.to_string();
        if let Some(existing) = file.peers.iter_mut().find(|p| p.node_id == *peer_id) {
            existing.address = addr_str;
        } else {
            file.peers.push(PersistedPeer {
                node_id: peer_id.clone(),
                address: addr_str,
                lan_addr: None,
            });
        }
    }

    if let Some(parent) = path.parent()
        && let Err(e) = std::fs::create_dir_all(parent)
    {
        warn!("Cannot create mesh persistence directory {}: {e}", parent.display());
        return;
    }

    match toml::to_string_pretty(&file) {
        Ok(content) => {
            if let Err(e) = std::fs::write(&path, content) {
                warn!("Failed to persist mesh peers to {}: {e}", path.display());
            } else {
                info!("Persisted {} mesh peer(s) to {}", file.peers.len(), path.display());
            }
        }
        Err(e) => warn!("Failed to serialize mesh peers: {e}"),
    }
}

/// A loaded peer with optional LAN address for same-subnet discovery.
#[derive(Debug, Clone)]
pub struct LoadedPeer {
    /// Node identifier (gate name).
    pub node_id: String,
    /// Primary address (typically WG overlay or WAN).
    pub address: SocketAddr,
    /// LAN address for priority-0 local routing (same physical subnet).
    pub lan_addr: Option<SocketAddr>,
}

/// Load persisted peers from disk.
///
/// Returns `(node_id, peers)` if the file exists and is valid.
/// Returns `None` if the file doesn't exist or is empty/invalid.
pub fn load_persisted_peers() -> Option<(String, Vec<(String, SocketAddr)>)> {
    let loaded = load_persisted_peers_full()?;
    let peers = loaded.1.iter().map(|p| (p.node_id.clone(), p.address)).collect();
    Some((loaded.0, peers))
}

/// Load persisted peers with full metadata (including LAN addresses).
pub fn load_persisted_peers_full() -> Option<(String, Vec<LoadedPeer>)> {
    let file = load_peers_file()?;

    if file.node_id.is_empty() || file.peers.is_empty() {
        return None;
    }

    let peers: Vec<LoadedPeer> = file
        .peers
        .iter()
        .filter_map(|p| {
            let addr: SocketAddr = p.address.parse().ok()?;
            let lan = p.lan_addr.as_ref().and_then(|la| la.parse().ok());
            Some(LoadedPeer {
                node_id: p.node_id.clone(),
                address: addr,
                lan_addr: lan,
            })
        })
        .collect();

    if peers.is_empty() {
        return None;
    }

    let lan_count = peers.iter().filter(|p| p.lan_addr.is_some()).count();
    info!(
        "Loaded {} persisted mesh peer(s) for node '{}' ({} with LAN)",
        peers.len(), file.node_id, lan_count
    );

    Some((file.node_id, peers))
}

/// Remove a peer from the persisted store (e.g., after permanent disconnect).
#[cfg(test)]
fn remove_persisted_peer(node_id: &str) {
    let path = peers_file_path();
    let Some(mut file) = load_peers_file() else {
        return;
    };

    let before = file.peers.len();
    file.peers.retain(|p| p.node_id != node_id);

    if file.peers.len() < before
        && let Ok(content) = toml::to_string_pretty(&file)
    {
        let _ = std::fs::write(&path, content);
        debug!("Removed peer '{node_id}' from persisted store");
    }
}

/// Persist a newly enrolled peer (from `mesh.enroll` BTSP-verified enrollment).
///
/// If an address is provided, it is stored alongside the node. If not, the node
/// is recorded without an address (will be discoverable but not directly routable
/// until it connects).
pub(crate) fn save_enrolled_peer(
    node_id: &str,
    _public_key: &str,
    address: &str,
    lan_addr: &str,
) {
    let path = peers_file_path();
    let mut file = load_peers_file().unwrap_or_default();

    let addr_str = if address.is_empty() {
        String::from("0.0.0.0:0")
    } else {
        address.to_string()
    };
    let lan = if lan_addr.is_empty() {
        None
    } else {
        Some(lan_addr.to_string())
    };

    if let Some(existing) = file.peers.iter_mut().find(|p| p.node_id == node_id) {
        existing.address.clone_from(&addr_str);
        if lan.is_some() {
            existing.lan_addr = lan;
        }
    } else {
        file.peers.push(PersistedPeer {
            node_id: node_id.to_string(),
            address: addr_str,
            lan_addr: lan,
        });
    }

    if let Some(parent) = path.parent()
        && let Err(e) = std::fs::create_dir_all(parent)
    {
        warn!("Cannot create mesh persistence directory {}: {e}", parent.display());
        return;
    }

    match toml::to_string_pretty(&file) {
        Ok(content) => {
            if let Err(e) = std::fs::write(&path, content) {
                warn!("Failed to persist enrolled peer to {}: {e}", path.display());
            } else {
                info!("Enrolled peer '{node_id}' persisted to {}", path.display());
            }
        }
        Err(e) => warn!("Failed to serialize enrolled peer: {e}"),
    }
}

fn load_peers_file() -> Option<PeersFile> {
    let path = peers_file_path();
    let content = std::fs::read_to_string(&path).ok()?;
    toml::from_str(&content).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;
    use std::sync::Mutex;
    use tempfile::TempDir;

    static DATA_DIR_LOCK: Mutex<()> = Mutex::new(());

    fn with_temp_data_dir(f: impl FnOnce()) {
        let _lock = DATA_DIR_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let dir = TempDir::new().unwrap();
        let _env =
            songbird_process_env::ScopedEnv::new("SONGBIRD_DATA_DIR", dir.path().to_str().unwrap());
        f();
    }

    #[test]
    fn save_and_load_roundtrip() {
        with_temp_data_dir(|| {
            let peers = vec![
                ("east-gate".to_string(), "192.168.1.144:7700".parse::<SocketAddr>().unwrap()),
                ("strand-gate".to_string(), "192.168.1.173:7700".parse::<SocketAddr>().unwrap()),
            ];

            save_peers("my-node", &peers);
            let (node_id, loaded) = load_persisted_peers().expect("should load");

            assert_eq!(node_id, "my-node");
            assert_eq!(loaded.len(), 2);
            assert_eq!(loaded[0].0, "east-gate");
            assert_eq!(loaded[0].1, "192.168.1.144:7700".parse::<SocketAddr>().unwrap());
        });
    }

    #[test]
    fn save_merges_duplicates() {
        with_temp_data_dir(|| {
            let peers1 =
                vec![("peer-a".to_string(), "10.0.0.1:7700".parse::<SocketAddr>().unwrap())];
            save_peers("node-1", &peers1);

            let peers2 = vec![
                ("peer-a".to_string(), "10.0.0.2:7700".parse::<SocketAddr>().unwrap()),
                ("peer-b".to_string(), "10.0.0.3:7700".parse::<SocketAddr>().unwrap()),
            ];
            save_peers("node-1", &peers2);

            let (_, loaded) = load_persisted_peers().expect("should load");
            assert_eq!(loaded.len(), 2);
            assert_eq!(loaded[0].1, "10.0.0.2:7700".parse::<SocketAddr>().unwrap());
        });
    }

    #[test]
    fn load_returns_none_when_no_file() {
        with_temp_data_dir(|| {
            assert!(load_persisted_peers().is_none());
        });
    }

    #[test]
    fn remove_peer_works() {
        with_temp_data_dir(|| {
            let peers = vec![
                ("peer-x".to_string(), "1.2.3.4:7700".parse::<SocketAddr>().unwrap()),
                ("peer-y".to_string(), "5.6.7.8:7700".parse::<SocketAddr>().unwrap()),
            ];
            save_peers("node-z", &peers);
            remove_persisted_peer("peer-x");

            let (_, loaded) = load_persisted_peers().expect("should load");
            assert_eq!(loaded.len(), 1);
            assert_eq!(loaded[0].0, "peer-y");
        });
    }
}
