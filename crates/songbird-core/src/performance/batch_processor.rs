//! Async batch processor for pipeline optimization

use songbird_errors::SongbirdError;
use songbird_errors::SongbirdResult;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use tokio::time::interval;

/// Async batch processor for efficient pipeline optimization
pub struct AsyncBatchProcessor<T, R> {
    /// Input channel for individual items
    sender: mpsc::UnboundedSender<BatchItem<T>>,
    /// Batch processing configuration
    batch_size: usize,
    batch_timeout: Duration,
    /// Processing statistics
    stats: Arc<Mutex<BatchStats>>,
    /// Phantom marker for unused generic parameter
    _phantom: PhantomData<R>,
}

/// Item in the batch with response channel
struct BatchItem<T> {
    item: T,
    response_tx: tokio::sync::oneshot::Sender<Result<(), BatchError>>,
}

/// Batch processing statistics
#[derive(Debug, Clone, Default)]
pub struct BatchStats {
    pub items_processed: u64,
    pub batches_processed: u64,
    pub avg_batch_size: f64,
    pub avg_processing_time: Duration,
    pub errors: u64,
}

/// Batch processing errors
#[derive(Debug, Clone)]
pub enum BatchError {
    ProcessingFailed(String),
    Timeout,
    Cancelled,
}

impl From<BatchError> for SongbirdError {
    fn from(error: BatchError) -> Self {
        match error {
            BatchError::ProcessingFailed(msg) => SongbirdError::Service {
                service: "BatchProcessor".to_string(),
                message: format!("Batch processing failed: {msg}"),
                suggested_alternatives: Vec::new(),
                recovery_actions: vec!["Check batch processing configuration and retry".to_string()],
            },
            BatchError::Timeout => SongbirdError::Service {
                service: "BatchProcessor".to_string(),
                message: "Batch processing timed out".to_string(),
                suggested_alternatives: Vec::new(),
                recovery_actions: vec!["Increase timeout or reduce batch size".to_string()],
            },
            BatchError::Cancelled => SongbirdError::Service {
                service: "BatchProcessor".to_string(),
                message: "Batch processing was cancelled".to_string(),
                suggested_alternatives: Vec::new(),
                recovery_actions: vec!["Retry the operation if needed".to_string()],
            },
        }
    }
}

impl<T, R> AsyncBatchProcessor<T, R>
where
    T: Send + 'static,
    R: Send + 'static,
{
    /// Create new async batch processor
    pub fn new<F>(batch_size: usize, batch_timeout: Duration, processor: F) -> Self
    where
        F: Fn(Vec<T>) -> SongbirdResult<Vec<R>> + Send + Sync + 'static,
    {
        let (sender, receiver) = mpsc::unbounded_channel();
        let stats = Arc::new(Mutex::new(BatchStats::default()));

        let processor_stats = stats.clone();
        let processor = Arc::new(processor);

        // Start batch processing task
        tokio::spawn(async move {
            Self::batch_processing_loop(
                receiver,
                batch_size,
                batch_timeout,
                processor,
                processor_stats,
            )
            .await;
        });

        Self {
            sender,
            batch_size,
            batch_timeout,
            stats,
            _phantom: PhantomData,
        }
    }

    /// Submit item for batch processing
    pub async fn submit(&self, item: T) -> SongbirdResult<()> {
        let (response_tx, response_rx) = tokio::sync::oneshot::channel();

        let batch_item = BatchItem { item, response_tx };

        if self.sender.send(batch_item).is_err() {
            return Err(BatchError::ProcessingFailed("Processor is shut down".to_string()).into());
        }

        // Wait for processing result
        match response_rx.await {
            Ok(result) => result.map_err(|e| e.into()),
            Err(_) => Err(BatchError::Cancelled.into()),
        }
    }

    /// Get processing statistics
    pub async fn get_stats(&self) -> BatchStats {
        self.stats.lock().await.clone()
    }

    /// Main batch processing loop
    async fn batch_processing_loop<F>(
        mut receiver: mpsc::UnboundedReceiver<BatchItem<T>>,
        batch_size: usize,
        batch_timeout: Duration,
        processor: Arc<F>,
        stats: Arc<Mutex<BatchStats>>,
    ) where
        F: Fn(Vec<T>) -> SongbirdResult<Vec<R>> + Send + Sync,
    {
        let mut current_batch = Vec::with_capacity(batch_size);
        let mut response_channels = Vec::with_capacity(batch_size);
        let mut batch_timer = interval(batch_timeout);

        loop {
            tokio::select! {
                // New item received
                item = receiver.recv() => {
                    match item {
                        Some(batch_item) => {
                            current_batch.push(batch_item.item);
                            response_channels.push(batch_item.response_tx);

                            // Process batch if it's full
                            if current_batch.len() >= batch_size {
                                Self::process_batch(
                                    &mut current_batch,
                                    &mut response_channels,
                                    processor.as_ref(),
                                    &stats,
                                ).await;
                            }
                        }
                        None => {
                            // Channel closed, process remaining items and exit
                            if !current_batch.is_empty() {
                                Self::process_batch(
                                    &mut current_batch,
                                    &mut response_channels,
                                    processor.as_ref(),
                                    &stats,
                                ).await;
                            }
                            break;
                        }
                    }
                }

                // Batch timeout reached
                _ = batch_timer.tick() => {
                    if !current_batch.is_empty() {
                        Self::process_batch(
                            &mut current_batch,
                            &mut response_channels,
                            processor.as_ref(),
                            &stats,
                        ).await;
                    }
                }
            }
        }
    }

    /// Process a batch of items
    async fn process_batch<F>(
        batch: &mut Vec<T>,
        response_channels: &mut Vec<tokio::sync::oneshot::Sender<Result<(), BatchError>>>,
        processor: &F,
        stats: &Arc<Mutex<BatchStats>>,
    ) where
        F: Fn(Vec<T>) -> SongbirdResult<Vec<R>> + Send + Sync,
    {
        let start_time = std::time::Instant::now();
        let batch_size = batch.len();

        // Take ownership of the batch
        let items = std::mem::take(batch);
        let channels = std::mem::take(response_channels);

        // Process the batch
        let result = processor(items);
        let processing_time = start_time.elapsed();

        // Send results back to all waiting clients
        match result {
            Ok(_results) => {
                // Success - notify all clients
                for tx in channels {
                    let _ = tx.send(Ok(()));
                }
            }
            Err(error) => {
                // Error - notify all clients of the failure
                for tx in channels {
                    let _ = tx.send(Err(BatchError::ProcessingFailed(error.to_string())));
                }

                // Update error stats
                let mut stats_guard = stats.lock().await;
                stats_guard.errors += 1;
            }
        }

        // Update processing statistics
        let mut stats_guard = stats.lock().await;
        stats_guard.items_processed += batch_size as u64;
        stats_guard.batches_processed += 1;

        // Update average batch size
        let total_items = stats_guard.items_processed;
        let total_batches = stats_guard.batches_processed;
        stats_guard.avg_batch_size = total_items as f64 / total_batches as f64;

        // Update average processing time
        let current_avg_nanos = stats_guard.avg_processing_time.as_nanos() as f64;
        let new_avg_nanos = (current_avg_nanos * (total_batches - 1) as f64
            + processing_time.as_nanos() as f64)
            / total_batches as f64;
        stats_guard.avg_processing_time = Duration::from_nanos(new_avg_nanos as u64);
    }

    /// Shutdown the processor
    pub fn shutdown(&self) {
        // Closing the sender will cause the processing loop to exit
        // This is automatically handled when AsyncBatchProcessor is dropped
    }
}

impl<T, R> Drop for AsyncBatchProcessor<T, R> {
    fn drop(&mut self) {
        // The sender is dropped here, which will close the channel
        // and cause the processing loop to handle remaining items and exit
    }
}

impl BatchStats {
    /// Calculate processing efficiency (items per second)
    pub fn items_per_second(&self) -> f64 {
        if self.avg_processing_time.as_secs_f64() == 0.0 {
            0.0
        } else {
            self.avg_batch_size / self.avg_processing_time.as_secs_f64()
        }
    }

    /// Calculate error rate
    pub fn error_rate(&self) -> f64 {
        if self.batches_processed == 0 {
            0.0
        } else {
            self.errors as f64 / self.batches_processed as f64
        }
    }

    /// Check if performance is healthy
    pub fn is_healthy(&self) -> bool {
        self.error_rate() < 0.05 && self.avg_processing_time < Duration::from_millis(100)
    }
}

impl std::fmt::Display for BatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BatchError::ProcessingFailed(msg) => write!(f, "Processing failed: {msg}"),
            BatchError::Timeout => write!(f, "Processing timed out"),
            BatchError::Cancelled => write!(f, "Processing was cancelled"),
        }
    }
}

impl std::error::Error for BatchError {}

/// Specialized batch processor for strings
pub type StringBatchProcessor = AsyncBatchProcessor<String, String>;

/// Specialized batch processor for bytes
pub type ByteBatchProcessor = AsyncBatchProcessor<Vec<u8>, Vec<u8>>;

/// High-level batch processing utilities
pub struct BatchProcessorBuilder<T, R> {
    batch_size: usize,
    batch_timeout: Duration,
    /// Phantom markers for unused generic parameters
    _phantom: PhantomData<(T, R)>,
}

impl<T, R> Default for BatchProcessorBuilder<T, R> {
    fn default() -> Self {
        Self {
            batch_size: 100,
            batch_timeout: Duration::from_millis(50),
            _phantom: PhantomData,
        }
    }
}

impl<T, R> BatchProcessorBuilder<T, R>
where
    T: Send + 'static,
    R: Send + 'static,
{
    /// Create new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set batch size
    pub fn batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    /// Set batch timeout
    pub fn batch_timeout(mut self, timeout: Duration) -> Self {
        self.batch_timeout = timeout;
        self
    }

    /// Build the batch processor
    pub fn build<F>(self, processor: F) -> AsyncBatchProcessor<T, R>
    where
        F: Fn(Vec<T>) -> SongbirdResult<Vec<R>> + Send + Sync + 'static,
    {
        AsyncBatchProcessor::new(self.batch_size, self.batch_timeout, processor)
    }
}
