use serde::{Deserialize, Serialize};
/// Federation Snapshots Module
///
/// This module provides snapshot functionality for federation state management,
/// allowing for backup and restoration of federation configurations and node states.
use songbird_config::unified::UnifiedFederationConfig;
use songbird_errors::{SongbirdError, SongbirdResult, config_error};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use tracing::{debug, info, warn};

/// Federation snapshot containing complete state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationSnapshot {
    /// Timestamp when the snapshot was created
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Federation configuration at snapshot time
    pub config: FederationConfig,
    /// All nodes in the federation
    pub nodes: HashMap<String, FederationNode>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Federation snapshot manager for handling backup and restore operations
#[derive(Debug)]
pub struct SnapshotManager {
    snapshot_dir: std::path::PathBuf,
}

impl SnapshotManager {
    /// Create a new snapshot manager
    pub fn new<P: AsRef<Path>>(snapshot_dir: P) -> Self {
        Self {
            snapshot_dir: snapshot_dir.as_ref().to_path_buf(),
        }
    }

    /// Create a snapshot of the current federation state
    pub async fn create_snapshot(&self) -> SongbirdResult<()> {let snapshot = FederationSnapshot {
            timestamp: chrono::Utc::now(),
            config: config.clone(),
            nodes: nodes.clone(),
            metadata: HashMap::new(),
        };

        info!("Created federation snapshot at {}", snapshot.timestamp);
        Ok(snapshot)
    }

    /// Save a snapshot to disk
    pub async fn save_snapshot(&self) -> SongbirdResult<()> {// Ensure snapshot directory exists
        fs::create_dir_all(&self.snapshot_dir).await.map_err(|e| {
            SongbirdError::io_error(format!("Failed to create snapshot directory: {}", error))
        })?;

        let filename = format!(
            "federation_snapshot_{}.json",
            snapshot.timestamp.format("%Y%m%d_%H%M%S")
        );
        let path = self.snapshot_dir.join(filename);

        let json = serde_json::to_string_pretty(snapshot)
            .map_err(|e| SongbirdError::communication(format!("Serialization failed: {}", error)))?;

        fs::write(&path, json).await.map_err(|e| {
            SongbirdError::io_error(format!("Failed to write snapshot file: {}", error))
        })?;

        info!("Saved federation snapshot to {:?}", path);
        Ok(path)
    }

    /// Load a snapshot from disk
    pub async fn load_snapshot<P: AsRef<Path>>(&self, path: P) -> songbird_errors::Result<FederationSnapshot> {
        let content = fs::read_to_string(path.as_ref())
            .await
            .map_err(|e| SongbirdError::io_error(format!("Failed to read snapshot file: {}", error)))?;

        let snapshot: FederationSnapshot = serde_json::from_str(&content).map_err(|e| {
            SongbirdError::communication(format!("Failed to deserialize snapshot: {}", error))
        })?;

        debug!("Loaded federation snapshot from {:?}", path.as_ref());
        Ok(snapshot)
    }

    /// List all available snapshots
    pub async fn list_snapshots(&self) -> SongbirdResult<()> {if !self.snapshot_dir.exists() {
            return Ok(Vec::new());
        }

        let mut entries = fs::read_dir(&self.snapshot_dir).await.map_err(|e| {
            SongbirdError::io_error(format!("Failed to read snapshot directory: {}", error))
        })?;

        let mut snapshots = Vec::new();
        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            SongbirdError::io_error(format!("Failed to read directory entry: {}", error))
        })? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json")
                && path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .map_or(false, |s| s.starts_with("federation_snapshot_"))
            {
                snapshots.push(path);
            }
        }

        snapshots.sort();
        Ok(snapshots)
    }

    /// Delete old snapshots, keeping only the most recent N snapshots
    pub async fn cleanup_old_snapshots(&self) -> SongbirdResult<()> {let mut snapshots = self.list_snapshots().await?;

        if snapshots.len() <= keep_count {
            return Ok(0);
        }

        // Sort by modification time, newest first
        snapshots.sort_by_key(|path| {
            std::fs::metadata(path)
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        });
        snapshots.reverse();

        let to_delete = snapshots.split_off(keep_count);
        let deleted_count = to_delete.len();

        for path in to_delete {
            if let Err(e) = fs::remove_file(&path).await {
                warn!("Failed to delete snapshot {:?}: {}", path, e);
            } else {
                debug!("Deleted old snapshot {:?}", path);
            }
        }

        info!(
            "Cleaned up {} old snapshots, keeping {}",
            deleted_count, keep_count
        );
        Ok(deleted_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_snapshot_creation() {
        let config = FederationConfig::default();
        let nodes = HashMap::new();

        let temp_dir = TempDir::new()
            .map_err(|e| SongbirdError::internal_error(format!("Failed to create temp directory: {e}")))?;
        
        let path = temp_dir.path().join("test_snapshot.json");
        
        manager
            .save_snapshot(&snapshot, &path)
            .await
            .map_err(|e| SongbirdError::internal_error(format!("Failed to create snapshot: {e}")))?;
        assert_eq!(snapshot.nodes.len(), 0);
        assert!(snapshot.timestamp <= chrono::Utc::now());
    }

    #[tokio::test]
    async fn test_snapshot_save_and_load() -> SongbirdResult<()> {
        let config = FederationConfig::default();
        let nodes = HashMap::new();

        let temp_dir = TempDir::new()
            .map_err(|e| SongbirdError::internal_error(format!("Failed to create temp directory: {e}")))?;
        let manager = SnapshotManager::new(temp_dir.path());

        let original_snapshot = manager
            .create_snapshot(&config, &nodes)
            .await?;
        let path = manager
            .save_snapshot(&original_snapshot)
            .await?;

        let loaded_snapshot = manager.load_snapshot(&path).await?;
        assert_eq!(
            original_snapshot.nodes.len(),
            loaded_snapshot.nodes.len()
        );
    }
}
