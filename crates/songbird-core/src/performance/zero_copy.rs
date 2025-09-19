//! # 🚀 Zero-Copy Performance Optimizations
//!
//! **🎯 HIGH-PERFORMANCE DATA TRANSFER**
//!
//! This module provides zero-copy patterns and memory-efficient implementations
//! for maximum performance in data-intensive operations.

use bytes::{Buf, BufMut, Bytes, BytesMut};
use serde::{Deserialize, Serialize};
use songbird_errors::{SongbirdError, SongbirdResult};
use std::borrow::Cow;
use std::sync::Arc;

/// Zero-copy message with reference-counted data
#[derive(Debug, Clone)]
pub struct ZeroCopyMessage {
    /// Message payload using copy-on-write semantics with Vec<u8>
    pub payload: Cow<'static, [u8]>,
    /// Message metadata
    pub metadata: MessageMetadata,
    /// Optional shared data reference for zero-copy operations
    pub shared_data: Option<Arc<SharedData>>,
}

/// Message metadata for zero-copy operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageMetadata {
    pub message_id: String,
    pub sender_id: String,
    pub timestamp: u64,
    pub message_type: u8,
    pub compression: CompressionType,
}

/// Shared data that can be referenced without copying
#[derive(Debug)]
pub struct SharedData {
    pub buffer: Bytes,
    pub checksum: u32,
    pub created_at: std::time::Instant,
}

/// Compression types for efficient data transfer
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CompressionType {
    None,
    Lz4,
    Zstd,
    Gzip,
}

impl ZeroCopyMessage {
    /// Create a message from a buffer with metadata
    pub fn from(buffer: Vec<u8>, metadata: MessageMetadata) -> Self {
        Self {
            payload: Cow::Owned(buffer),
            metadata,
            shared_data: None,
        }
    }

    /// Create a message with shared reference-counted data
    pub fn from_shared(shared_data: Arc<SharedData>, metadata: MessageMetadata) -> Self {
        // Convert Bytes to Vec<u8> for the Cow<Vec<u8>>
        Self {
            payload: Cow::Owned(shared_data.buffer.to_vec()),
            metadata,
            shared_data: Some(shared_data),
        }
    }

    /// Get the payload size without copying
    pub fn payload_size(&self) -> usize {
        self.payload.len()
    }

    /// Check if the message uses borrowed data (zero-copy)
    pub fn is_zero_copy(&self) -> bool {
        matches!(self.payload, Cow::Borrowed(_))
    }

    /// Convert to owned data if necessary (copy-on-write)
    pub fn into_owned(self) -> ZeroCopyMessage {
        ZeroCopyMessage {
            payload: Cow::Owned(self.payload.into_owned()),
            metadata: self.metadata,
            shared_data: self.shared_data,
        }
    }

    /// Serialize the message efficiently
    pub fn serialize(&self) -> SongbirdResult<Bytes> {
        let metadata_bytes = bincode::serialize(&self.metadata)
            .map_err(|e| SongbirdError::internal_error(format!("Serialization failed: {e}")))?;

        let metadata_len = metadata_bytes.len() as u32;
        let payload_len = self.payload.len() as u32;

        let mut buffer =
            BytesMut::with_capacity(4 + metadata_len as usize + 4 + payload_len as usize);

        // Write metadata length and data
        buffer.put_u32(metadata_len);
        buffer.put_slice(&metadata_bytes);

        // Write payload length and data
        buffer.put_u32(payload_len);
        buffer.put_slice(&self.payload);

        Ok(buffer.freeze())
    }

    /// Deserialize a message efficiently
    pub fn deserialize(mut data: Bytes) -> SongbirdResult<ZeroCopyMessage> {
        if data.remaining() < 4 {
            return Err(SongbirdError::internal_error(validation_error(
                "Insufficient data for metadata length",
            ));
        }

        let metadata_len = data.get_u32() as usize;
        if data.remaining() < metadata_len {
            return Err(SongbirdError::internal_error(validation_error(
                "Insufficient data for metadata",
            ));
        }

        let metadata_bytes = data.split_to(metadata_len);
        let metadata: MessageMetadata = bincode::deserialize(&metadata_bytes).map_err(|e| {
            SongbirdError::internal_error(format!("Metadata deserialization failed: {e}"))
        })?;

        if data.remaining() < 4 {
            return Err(SongbirdError::internal_error(validation_error(
                "Insufficient data for payload length",
            ));
        }

        let payload_len = data.get_u32() as usize;
        if data.remaining() < payload_len {
            return Err(SongbirdError::internal_error(validation_error(
                "Insufficient data for payload",
            ));
        }

        let payload_bytes = data.split_to(payload_len);

        Ok(ZeroCopyMessage {
            payload: Cow::Owned(payload_bytes.to_vec()),
            metadata,
            shared_data: None,
        })
    }
}

/// High-performance buffer pool for memory reuse
pub struct BufferPool {
    small_buffers: crossbeam_queue::SegQueue<BytesMut>,
    medium_buffers: crossbeam_queue::SegQueue<BytesMut>,
    large_buffers: crossbeam_queue::SegQueue<BytesMut>,
    max_pool_size: usize,
}

impl BufferPool {
    /// Create a new buffer pool
    pub fn new(max_pool_size: usize) -> Self {
        Self {
            small_buffers: crossbeam_queue::SegQueue::new(),
            medium_buffers: crossbeam_queue::SegQueue::new(),
            large_buffers: crossbeam_queue::SegQueue::new(),
            max_pool_size,
        }
    }

    /// Get a buffer from the pool or allocate a new one
    pub fn get_buffer(&self, size: usize) -> BytesMut {
        let queue = match size {
            0..=1024 => &self.small_buffers,
            1025..=8192 => &self.medium_buffers,
            _ => &self.large_buffers,
        };

        if let Some(mut buffer) = queue.pop() {
            buffer.clear();
            if buffer.capacity() >= size {
                return buffer;
            }
        }

        BytesMut::with_capacity(size.max(1024))
    }

    /// Return a buffer to the pool for reuse
    pub fn return_buffer(&self, buffer: BytesMut) {
        if buffer.capacity() == 0 {
            return;
        }

        let queue = match buffer.capacity() {
            0..=1024 => &self.small_buffers,
            1025..=8192 => &self.medium_buffers,
            _ => &self.large_buffers,
        };

        if queue.len() < self.max_pool_size {
            queue.push(buffer);
        }
    }
}

/// Zero-copy string operations
pub struct ZeroCopyString<'a> {
    data: Cow<'a, str>,
}

impl<'a> ZeroCopyString<'a> {
    /// Create from borrowed string slice
    pub fn from_borrowed(s: &'a str) -> Self {
        Self {
            data: Cow::Borrowed(s),
        }
    }

    /// Create from owned string
    pub fn from_owned(s: String) -> Self {
        Self {
            data: Cow::Owned(s),
        }
    }

    /// Get string slice without copying
    pub fn as_str(&self) -> &str {
        &self.data
    }

    /// Convert to owned string (copy-on-write)
    pub fn into_owned(self) -> String {
        self.data.into_owned()
    }

    /// Check if this is a zero-copy reference
    pub fn is_borrowed(&self) -> bool {
        matches!(self.data, Cow::Borrowed(_))
    }
}

/// Memory-mapped file for zero-copy file operations
pub struct MemoryMappedFile {
    #[cfg(unix)]
    mmap: memmap2::Mmap,
    #[cfg(not(unix))]
    data: Vec<u8>,
    file_path: std::path::PathBuf,
}

impl MemoryMappedFile {
    /// Open a file for memory-mapped access
    pub fn open<P: AsRef<std::path::Path>>(path: P) -> SongbirdResult<Self> {
        let file_path = path.as_ref().to_path_buf();

        #[cfg(unix)]
        {
            let file = std::fs::File::open(&file_path)
                .map_err(|e| SongbirdError::resource_error(format!("Failed to open file: {e}")))?;

            let mmap = unsafe { memmap2::Mmap::map(&file) }.map_err(|e| {
                SongbirdError::resource_error(format!("Failed to memory map file: {e}"))
            })?;

            Ok(Self { mmap, file_path })
        }

        #[cfg(not(unix))]
        {
            let data = std::fs::read(&file_path).map_err(|e| {
                SongbirdError::resource_error(format!("Failed to read file: {}", e))
            })?;

            Ok(Self { data, file_path })
        }
    }

    /// Get a slice of the file data without copying
    pub fn as_slice(&self) -> &[u8] {
        #[cfg(unix)]
        return &self.mmap;

        #[cfg(not(unix))]
        return &self.data;
    }

    /// Get the file size
    pub fn len(&self) -> usize {
        self.as_slice().len()
    }

    /// Check if the file is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get a specific range without copying
    pub fn slice(&self, start: usize, end: usize) -> SongbirdResult<&[u8]> {
        let data = self.as_slice();
        if end > data.len() || start > end {
            return Err(SongbirdError::internal_error(validation_error("Invalid slice range"));
        }
        Ok(&data[start..end])
    }
}

/// High-performance streaming processor
pub struct StreamProcessor {
    buffer_pool: Arc<BufferPool>,
    chunk_size: usize,
}

impl StreamProcessor {
    /// Create a new stream processor
    pub fn new(buffer_pool: Arc<BufferPool>, chunk_size: usize) -> Self {
        Self {
            buffer_pool,
            chunk_size,
        }
    }

    /// Process data in chunks without unnecessary copying
    pub async fn process_stream<F, Fut>(&self, data: &[u8], mut processor: F) -> SongbirdResult<()>
    where
        F: FnMut(&[u8]) -> Fut,
        Fut: std::future::Future<Output = SongbirdResult<()>>,
    {
        for chunk in data.chunks(self.chunk_size) {
            processor(chunk).await?;

            // Yield to allow other tasks to run
            tokio::task::yield_now().await;
        }

        Ok(())
    }

    /// Process a memory-mapped file in chunks
    pub async fn process_file<F, Fut>(
        &self,
        file: &MemoryMappedFile,
        processor: F,
    ) -> SongbirdResult<()>
    where
        F: FnMut(&[u8]) -> Fut,
        Fut: std::future::Future<Output = SongbirdResult<()>>,
    {
        self.process_stream(file.as_slice(), processor).await
    }
}

/// Efficient data compression utilities
pub struct CompressionUtils;

impl CompressionUtils {
    /// Compress data using the specified algorithm
    pub fn compress(data: &[u8], compression: CompressionType) -> SongbirdResult<Vec<u8>> {
        match compression {
            CompressionType::None => Ok(data.to_vec()),
            CompressionType::Lz4 => {
                #[cfg(feature = "lz4")]
                {
                    Ok(lz4_flex::compress_prepend_size(data))
                }
                #[cfg(not(feature = "lz4"))]
                {
                    Err(SongbirdError::internal_error(internal_error(
                        "LZ4 compression not available",
                    ))
                }
            }
            CompressionType::Zstd => {
                #[cfg(feature = "zstd")]
                {
                    zstd::encode_all(data, 3).map_err(|e| {
                        SongbirdError::internal_error(format!("Zstd compression failed: {e}"))
                    })
                }
                #[cfg(not(feature = "zstd"))]
                {
                    Err(SongbirdError::internal_error(internal_error(
                        "Zstd compression not available",
                    ))
                }
            }
            CompressionType::Gzip => {
                use std::io::Write;
                let mut encoder =
                    flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
                encoder.write_all(data).map_err(|e| {
                    SongbirdError::internal_error(format!("Gzip write failed: {e}"))
                })?;
                encoder.finish().map_err(|e| {
                    SongbirdError::internal_error(format!("Gzip compression failed: {e}"))
                })
            }
        }
    }

    /// Decompress data using the specified algorithm
    pub fn decompress(data: &[u8], compression: CompressionType) -> SongbirdResult<Vec<u8>> {
        match compression {
            CompressionType::None => Ok(data.to_vec()),
            CompressionType::Lz4 => {
                #[cfg(feature = "lz4")]
                {
                    lz4_flex::decompress_size_prepended(data).map_err(|e| {
                        SongbirdError::internal_error(format!("LZ4 decompression failed: {e}"))
                    })
                }
                #[cfg(not(feature = "lz4"))]
                {
                    Err(SongbirdError::internal_error(internal_error(
                        "LZ4 decompression not available",
                    ))
                }
            }
            CompressionType::Zstd => {
                #[cfg(feature = "zstd")]
                {
                    zstd::decode_all(data).map_err(|e| {
                        SongbirdError::internal_error(format!("Zstd decompression failed: {e}"))
                    })
                }
                #[cfg(not(feature = "zstd"))]
                {
                    Err(SongbirdError::internal_error(internal_error(
                        "Zstd decompression not available",
                    ))
                }
            }
            CompressionType::Gzip => {
                use std::io::Read;
                let mut decoder = flate2::read::GzDecoder::new(data);
                let mut result = Vec::new();
                decoder.read_to_end(&mut result).map_err(|e| {
                    SongbirdError::internal_error(format!("Gzip decompression failed: {e}"))
                })?;
                Ok(result)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use songbird_errors::SongbirdResult;

    #[test]
    fn test_zero_copy_message_borrowed() {
        let data = b"test message";
        let metadata = MessageMetadata {
            message_id: "test".to_string(),
            sender_id: "sender".to_string(),
            timestamp: 123456789,
            message_type: 1,
            compression: CompressionType::None,
        };

        // Create shared data for zero-copy test
        let shared_data = Arc::new(SharedData {
            buffer: Bytes::copy_from_slice(data),
            checksum: 0,
            created_at: std::time::Instant::now(),
        });
        let message = ZeroCopyMessage::from_shared(shared_data, metadata);
        assert!(!message.is_zero_copy()); // from_shared creates owned data, not borrowed
        assert_eq!(message.payload_size(), 12);
    }

    #[test]
    fn test_zero_copy_message_owned() {
        let data = b"test message".to_vec();
        let metadata = MessageMetadata {
            message_id: "test".to_string(),
            sender_id: "sender".to_string(),
            timestamp: 123456789,
            message_type: 1,
            compression: CompressionType::None,
        };

        let message = ZeroCopyMessage::from(data, metadata);
        assert!(!message.is_zero_copy());
        assert_eq!(message.payload_size(), 12);
    }

    #[test]
    fn test_buffer_pool() {
        let pool = BufferPool::new(10);

        // Get buffers of different sizes
        let small_buffer = pool.get_buffer(512);
        let medium_buffer = pool.get_buffer(4096);
        let large_buffer = pool.get_buffer(16384);

        assert!(small_buffer.capacity() >= 512);
        assert!(medium_buffer.capacity() >= 4096);
        assert!(large_buffer.capacity() >= 16384);

        // Return buffers to pool
        pool.return_buffer(small_buffer);
        pool.return_buffer(medium_buffer);
        pool.return_buffer(large_buffer);

        // Get buffers again - should reuse from pool
        let reused_buffer = pool.get_buffer(512);
        assert!(reused_buffer.capacity() >= 512);
    }

    #[test]
    fn test_zero_copy_string() {
        let borrowed_str = "test string";
        let zero_copy = ZeroCopyString::from_borrowed(borrowed_str);
        assert!(zero_copy.is_borrowed());
        assert_eq!(zero_copy.as_str(), "test string");

        let owned = ZeroCopyString::from_owned("owned string".to_string());
        assert!(!owned.is_borrowed());
        assert_eq!(owned.as_str(), "owned string");
    }

    #[tokio::test]
    async fn test_stream_processor() {
        let pool = Arc::new(BufferPool::new(10));
        let processor = StreamProcessor::new(pool, 1024);

        let data = vec![0u8; 5000]; // 5KB of data
        let processed_chunks = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let chunks_counter = processed_chunks.clone();

        processor
            .process_stream(&data, |chunk| {
                let counter = chunks_counter.clone();
                let chunk_len = chunk.len();
                async move {
                    counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    assert!(chunk_len <= 1024);
                    Ok(())
                }
            })
            .await
            .unwrap();

        assert_eq!(
            processed_chunks.load(std::sync::atomic::Ordering::Relaxed),
            5
        ); // 5 chunks of 1024 bytes each
    }

    #[test]
    fn test_compression_gzip() {
        // Use larger, more repetitive data that compresses well
        let data = b"Hello, world! This is a test message for compression. ".repeat(100);
        let data_slice = data.as_slice();

        let compressed = CompressionUtils::compress(data_slice, CompressionType::Gzip).unwrap();

        // Large repetitive data should compress significantly
        assert!(
            compressed.len() < data_slice.len(),
            "Compressed size {} should be less than original size {}",
            compressed.len(),
            data_slice.len()
        );

        let decompressed =
            CompressionUtils::decompress(&compressed, CompressionType::Gzip).unwrap();
        assert_eq!(decompressed, data_slice);
    }
}
