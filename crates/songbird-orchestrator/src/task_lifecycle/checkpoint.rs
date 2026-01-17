//! Task checkpointing
//!
//! Enable long-running tasks to save state and resume after failures.
//! No unsafe code, modern async patterns.

use super::TaskId;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Checkpoint for resuming tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Checkpoint ID (UUID)
    pub id: Arc<str>,

    /// Task this checkpoint belongs to
    pub task_id: TaskId,

    /// Timestamp when checkpoint was created
    pub created_at: DateTime<Utc>,

    /// Progress at checkpoint (0.0 - 1.0)
    pub progress: f32,

    /// Serialized task state (opaque bytes)
    pub state: Vec<u8>,

    /// Checkpoint metadata
    pub metadata: CheckpointMetadata,
}

/// Checkpoint metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointMetadata {
    /// Size in bytes
    pub size_bytes: u64,

    /// Compression algorithm used (if any)
    pub compression: Option<CompressionAlgorithm>,

    /// Checksum for integrity verification
    pub checksum: Arc<str>,
}

/// Compression algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionAlgorithm {
    None,
    Gzip,  // Pure Rust via flate2 (migrated from Zstd on Jan 17, 2026)
}

impl Checkpoint {
    /// Create a new checkpoint
    pub fn new(task_id: TaskId, progress: f32, state: Vec<u8>) -> Self {
        let size_bytes = state.len() as u64;
        let checksum = Self::calculate_checksum(&state);

        Self {
            id: Arc::from(uuid::Uuid::now_v7().to_string()),
            task_id,
            created_at: Utc::now(),
            progress,
            state,
            metadata: CheckpointMetadata {
                size_bytes,
                compression: Some(CompressionAlgorithm::None),
                checksum: Arc::from(checksum),
            },
        }
    }

    /// Create a checkpoint with compression
    pub fn new_compressed(task_id: TaskId, progress: f32, state: Vec<u8>) -> Result<Self> {
        let compressed = Self::compress_state(&state)?;
        let size_bytes = compressed.len() as u64;
        let checksum = Self::calculate_checksum(&compressed);

        Ok(Self {
            id: Arc::from(uuid::Uuid::now_v7().to_string()),
            task_id,
            created_at: Utc::now(),
            progress,
            state: compressed,
            metadata: CheckpointMetadata {
                size_bytes,
                compression: Some(CompressionAlgorithm::Gzip),
                checksum: Arc::from(checksum),
            },
        })
    }

    /// Verify checkpoint integrity
    pub fn verify(&self) -> Result<()> {
        let calculated_checksum = Self::calculate_checksum(&self.state);
        if calculated_checksum != self.metadata.checksum.as_ref() {
            anyhow::bail!("Checkpoint integrity check failed: checksum mismatch");
        }
        Ok(())
    }

    /// Get decompressed state
    pub fn get_state(&self) -> Result<Vec<u8>> {
        match self.metadata.compression {
            Some(CompressionAlgorithm::Gzip) => Self::decompress_state(&self.state),
            Some(CompressionAlgorithm::None) | None => Ok(self.state.clone()),
        }
    }

    /// Calculate SHA-256 checksum
    fn calculate_checksum(data: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    /// Compress state using gzip (pure Rust via flate2)
    fn compress_state(data: &[u8]) -> Result<Vec<u8>> {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;
        
        let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
        encoder.write_all(data)
            .context("Failed to write data to gzip encoder")?;
        encoder.finish()
            .context("Failed to compress checkpoint state with gzip")
    }

    /// Decompress state using gzip (pure Rust via flate2)
    fn decompress_state(data: &[u8]) -> Result<Vec<u8>> {
        use flate2::read::GzDecoder;
        use std::io::Read;
        
        let mut decoder = GzDecoder::new(data);
        let mut result = Vec::new();
        decoder.read_to_end(&mut result)
            .context("Failed to decompress checkpoint state with gzip")?;
        Ok(result)
    }
}

/// Checkpoint manager configuration
#[derive(Debug, Clone)]
pub struct CheckpointConfig {
    /// Enable compression for checkpoints
    pub enable_compression: bool,

    /// Compression threshold (bytes) - compress if larger
    pub compression_threshold: u64,

    /// Maximum checkpoint age before cleanup (seconds)
    pub max_age_seconds: u64,

    /// Maximum checkpoints per task
    pub max_checkpoints_per_task: usize,
}

impl Default for CheckpointConfig {
    fn default() -> Self {
        Self {
            enable_compression: true,
            compression_threshold: 1024 * 1024, // 1MB
            max_age_seconds: 7 * 24 * 3600,     // 7 days
            max_checkpoints_per_task: 10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkpoint_creation() {
        let task_id = TaskId::new();
        let state = vec![1, 2, 3, 4, 5];

        let checkpoint = Checkpoint::new(task_id, 0.5, state.clone());

        assert_eq!(checkpoint.task_id, task_id);
        assert_eq!(checkpoint.progress, 0.5);
        assert_eq!(checkpoint.state, state);
        assert_eq!(checkpoint.metadata.size_bytes, 5);
    }

    #[test]
    fn test_checkpoint_integrity() {
        let task_id = TaskId::new();
        let state = vec![1, 2, 3, 4, 5];

        let checkpoint = Checkpoint::new(task_id, 0.5, state);

        // Should verify successfully
        assert!(checkpoint.verify().is_ok());

        // Corrupt the checkpoint
        let mut corrupted = checkpoint.clone();
        corrupted.state[0] = 99;

        // Should fail verification
        assert!(corrupted.verify().is_err());
    }

    #[test]
    fn test_checkpoint_compression() {
        let task_id = TaskId::new();
        let state = vec![1u8; 5000]; // 5000 bytes of repetitive data

        let checkpoint = Checkpoint::new_compressed(task_id, 0.5, state.clone()).unwrap();

        // Compressed should be smaller than original (repetitive data compresses well)
        assert!(checkpoint.metadata.size_bytes < 5000);
        assert_eq!(checkpoint.metadata.compression, Some(CompressionAlgorithm::Gzip));

        // Should decompress correctly
        let decompressed = checkpoint.get_state().unwrap();
        assert_eq!(decompressed.len(), state.len());
        assert_eq!(decompressed, state);
    }

    #[test]
    fn test_checkpoint_checksum() {
        let task_id = TaskId::new();
        let state = vec![1, 2, 3, 4, 5];

        let checkpoint1 = Checkpoint::new(task_id, 0.5, state.clone());
        let checkpoint2 = Checkpoint::new(task_id, 0.5, state);

        // Same data should produce same checksum
        assert_eq!(checkpoint1.metadata.checksum, checkpoint2.metadata.checksum);
    }
}
