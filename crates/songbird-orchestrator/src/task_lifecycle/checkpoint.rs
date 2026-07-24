// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![cfg_attr(
    test,
    expect(clippy::float_cmp, reason = "test: exact float comparison is intentional")
)]
//! Task checkpointing
//!
//! Enable long-running tasks to save state and resume after failures.
//! No unsafe code, modern async patterns.

use super::TaskId;
#[cfg(feature = "task-checkpoint-gzip")]
use anyhow::Context;
use anyhow::Result;
use base64::{Engine, engine::general_purpose::STANDARD};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use songbird_crypto_provider::CryptoProvider;
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
    Gzip, // Pure Rust via flate2 (migrated from Zstd on Jan 17, 2026)
    Zlib, // Pure Rust via flate2 (alternative compression)
}

impl Checkpoint {
    /// Create a new checkpoint
    #[must_use]
    pub fn new(task_id: TaskId, progress: f32, state: Vec<u8>) -> Self {
        let size_bytes = state.len() as u64;
        let checksum = Self::calculate_checksum_local(&state);

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
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn new_compressed(task_id: TaskId, progress: f32, state: Vec<u8>) -> Result<Self> {
        #[cfg(not(feature = "task-checkpoint-gzip"))]
        {
            anyhow::bail!(
                "checkpoint gzip compression requires the `task-checkpoint-gzip` crate feature"
            );
        }
        #[cfg(feature = "task-checkpoint-gzip")]
        {
            let compressed = Self::compress_state(&state)?;
            let size_bytes = compressed.len() as u64;
            let checksum = Self::calculate_checksum_local(&compressed);

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
    }

    /// Verify checkpoint integrity
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn verify(&self) -> Result<()> {
        let calculated_checksum = Self::calculate_checksum_local(&self.state);
        if calculated_checksum != self.metadata.checksum.as_ref() {
            anyhow::bail!("Checkpoint integrity check failed: checksum mismatch");
        }
        Ok(())
    }

    /// Get decompressed state
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn get_state(&self) -> Result<Vec<u8>> {
        match self.metadata.compression {
            Some(CompressionAlgorithm::Gzip) => Self::decompress_gzip(&self.state),
            Some(CompressionAlgorithm::Zlib) => Self::decompress_zlib(&self.state),
            Some(CompressionAlgorithm::None) | None => Ok(self.state.clone()),
        }
    }

    #[cfg(feature = "local-crypto-fallback")]
    fn calculate_checksum_local(data: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    #[cfg(not(feature = "local-crypto-fallback"))]
    fn calculate_checksum_local(_data: &[u8]) -> String {
        tracing::error!(
            "Checkpoint checksum requires bearDog CryptoProvider or local-crypto-fallback feature"
        );
        String::from("0".repeat(64))
    }

    /// Compress state using gzip (pure Rust via flate2)
    #[cfg(feature = "task-checkpoint-gzip")]
    fn compress_state(data: &[u8]) -> Result<Vec<u8>> {
        use flate2::Compression;
        use flate2::write::GzEncoder;
        use std::io::Write;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
        encoder.write_all(data).context("Failed to write data to gzip encoder")?;
        encoder.finish().context("Failed to compress checkpoint state with gzip")
    }

    /// Decompress state using gzip (pure Rust via flate2)
    #[cfg(feature = "task-checkpoint-gzip")]
    fn decompress_gzip(data: &[u8]) -> Result<Vec<u8>> {
        use flate2::read::GzDecoder;
        use std::io::Read;

        let mut decoder = GzDecoder::new(data);
        let mut result = Vec::new();
        decoder
            .read_to_end(&mut result)
            .context("Failed to decompress checkpoint state with gzip")?;
        Ok(result)
    }

    #[cfg(not(feature = "task-checkpoint-gzip"))]
    fn decompress_gzip(_data: &[u8]) -> Result<Vec<u8>> {
        anyhow::bail!("reading gzip checkpoints requires the `task-checkpoint-gzip` crate feature");
    }

    /// Decompress state using zlib (pure Rust via flate2)
    #[cfg(feature = "task-checkpoint-gzip")]
    fn decompress_zlib(data: &[u8]) -> Result<Vec<u8>> {
        use flate2::read::ZlibDecoder;
        use std::io::Read;

        let mut decoder = ZlibDecoder::new(data);
        let mut result = Vec::new();
        decoder
            .read_to_end(&mut result)
            .context("Failed to decompress checkpoint state with zlib")?;
        Ok(result)
    }

    #[cfg(not(feature = "task-checkpoint-gzip"))]
    fn decompress_zlib(_data: &[u8]) -> Result<Vec<u8>> {
        anyhow::bail!("reading zlib checkpoints requires the `task-checkpoint-gzip` crate feature");
    }
}

async fn sha256_bytes_via_provider(data: &[u8], provider: &CryptoProvider) -> Result<Vec<u8>> {
    let data_b64 = STANDARD.encode(data);
    let result = provider
        .call("crypto.sha256", json!({ "data": data_b64 }))
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let hash_b64 = result
        .get("hash")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing hash in SHA-256 response"))?;
    STANDARD.decode(hash_b64).map_err(|e| anyhow::anyhow!("Invalid hash: {e}"))
}

pub async fn calculate_checksum(data: &[u8], crypto: Option<&CryptoProvider>) -> Result<String> {
    if let Some(provider) = crypto {
        let bytes = sha256_bytes_via_provider(data, provider).await?;
        if bytes.len() != 32 {
            anyhow::bail!("Unexpected SHA-256 digest length");
        }
        let mut hex = String::with_capacity(64);
        for b in &bytes {
            use std::fmt::Write;
            let _ = write!(hex, "{b:02x}");
        }
        return Ok(hex);
    }

    #[cfg(feature = "local-crypto-fallback")]
    {
        tracing::debug!("Checkpoint checksum: using local SHA-256 fallback (bearDog unavailable)");
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(data);
        Ok(format!("{:x}", hasher.finalize()))
    }

    #[cfg(not(feature = "local-crypto-fallback"))]
    {
        anyhow::bail!(
            "Checkpoint checksum requires bearDog CryptoProvider (local-crypto-fallback disabled)"
        )
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
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
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
        let mut corrupted = checkpoint;
        corrupted.state[0] = 99;

        // Should fail verification
        assert!(corrupted.verify().is_err());
    }

    #[cfg(feature = "task-checkpoint-gzip")]
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
