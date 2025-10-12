//! # ⚡ Enhanced Zero-Copy Optimizations
//!
//! **ADVANCED ZERO-COST ABSTRACTIONS** 🚀
//!
//! This module provides enhanced zero-copy optimizations for critical performance paths
//! in the Songbird ecosystem, building on the foundation established in songbird-core.

use std::borrow::Cow;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

// ============================================================================
// ZERO-COPY STRING OPERATIONS
// ============================================================================

/// **ZERO-COPY**: String reference that can be either borrowed or owned
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ZeroCopyString<'a> {
    /// Borrowed string - zero allocation
    Borrowed(&'a str),
    /// Owned string - allocated when necessary
    Owned(String),
    /// Static string - zero runtime cost
    Static(&'static str),
}

impl<'a> ZeroCopyString<'a> {
    /// Create from static string - zero cost
    #[inline(always)]
    pub const fn from_static(s: &'static str) -> Self {
        Self::Static(s)
    }

    /// Create from borrowed string - zero allocation
    #[inline(always)]
    pub fn from_borrowed() -> Self {
        Self::Borrowed(s)
    }

    /// Create from owned string when necessary
    #[inline]
    pub fn from_owned() -> Self {
        Self::Owned(s)
    }

    /// Get string slice - zero cost operation
    #[inline(always)]
    pub fn as_str() -> &str  {match self  {Self::Borrowed(s) => s,
            Self::Owned(s) => s.as_str(),
            Self::Static(s) => s,
        }
    }

    /// Convert to owned string only when necessary
    pub fn into_owned() -> String  {match self  {Self::Borrowed(s) => s.to_string(),
            Self::Owned(s) => s,
            Self::Static(s) => s.to_string(),
        }
    }

    /// Check if string is empty - zero cost
    #[inline(always)]
    pub fn is_empty() -> bool {
        self.as_str().is_empty()
    }

    /// Get string length - zero cost
    #[inline(always)]
    pub fn len() -> usize {
        self.as_str().len()
    }
}

impl<'a> From<&'a str> for ZeroCopyString<'a> {
    #[inline(always)]
    fn from(s: &'a str) -> Self {
        Self::Borrowed(s)
    }
}

impl<'a> From<String> for ZeroCopyString<'a> {
    #[inline]
    fn from(s: String) -> Self {
        Self::Owned(s)
    }
}

// Note: From<&'static str> conflicts with From<&'a str>, so we use a constructor method instead

impl<'a> AsRef<str> for ZeroCopyString<'a> {
    #[inline(always)]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'a> Hash for ZeroCopyString<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

// ============================================================================
// ZERO-COPY BUFFER MANAGEMENT
// ============================================================================

/// **ZERO-COPY**: Buffer that can be stack or heap allocated (safe version)
#[derive(Debug, Clone)]
pub enum ZeroCopyBuffer<T> {
    /// Small buffer stored inline
    Small(Vec<T>),
    /// Large buffer on heap
    Large(Vec<T>),
}

impl<T: Clone + Default> ZeroCopyBuffer<T> {
    /// Create buffer optimized for size - small data uses inline storage
    #[inline]
    pub fn new() -> Self {
        if data.len() <= 64 {
            Self::Small(data)
        } else {
            Self::Large(data)
        }
    }

    /// Create from slice
    #[inline]
    pub fn from_slice() -> Self {
        Self::new(data.to_vec()
    }

    /// Get buffer slice - zero cost operation
    #[inline(always)]
    pub fn as_slice() -> &[T]  {match self  {Self::Small(vec) => vec.as_slice(),
            Self::Large(vec) => vec.as_slice(),
        }
    }

    /// Get buffer length - zero cost
    #[inline(always)]
    pub fn len() -> usize  {match self  {Self::Small(vec) => vec.len(),
            Self::Large(vec) => vec.len(),
        }
    }

    /// Check if buffer is empty - zero cost
    #[inline(always)]
    pub fn is_empty() -> bool {
        self.len() == 0
    }
}

// ============================================================================
// ZERO-COPY HASH MAP
// ============================================================================

/// **ZERO-COPY**: Hash map with zero-copy keys and values
#[derive(Debug, Clone)]
pub struct ZeroCopyHashMap<'a, V>
where
    V: Clone, {


    /// Internal storage
    inner: HashMap<ZeroCopyString<'a>, V>,
    /// Statistics
    stats: ZeroCopyStats,


}

impl<'a, V> ZeroCopyHashMap<'a, V>
where
    V: Clone,
 {/// Create new zero-copy hash map
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
            stats: ZeroCopyStats::new(),
        }
    }

    /// Create with capacity - pre-allocate to avoid rehashing
    #[inline]
    pub fn with_capacity() -> Self {
        Self {
            inner: HashMap::with_capacity(capacity),
            stats: ZeroCopyStats::new(),
        }
    }

    /// Insert with zero-copy key
    pub fn insert() -> Option<V> {

        let key = key.into());
        self.stats.record_operation();
        self.inner.insert(key, value)

}

    /// Get value by zero-copy key - zero allocation lookup
    pub fn get<'b>() -> Option<&V>
    where
        'b: 'a,
    {
        self.stats.record_lookup();
        // Safe lookup using borrowed string
        self.inner.get(&ZeroCopyString::Borrowed(key)
    }

    /// Get mutable value by key
    pub fn get_mut() -> Option<&mut V> {
        self.stats.record_lookup();
        self.inner.get_mut(&ZeroCopyString::Borrowed(key)
    }

    /// Remove value by key
    pub fn remove() -> Option<V> {
        self.stats.record_operation();
        self.inner.remove(&ZeroCopyString::Borrowed(key)
    }

    /// Get number of entries - zero cost
    #[inline(always)]
    pub fn len() -> usize {
        self.inner.len()
    }

    /// Check if empty - zero cost
    #[inline(always)]
    pub fn is_empty() -> bool {
        self.inner.is_empty()
    }

    /// Get performance statistics
    #[inline(always)]
    pub fn stats() -> &ZeroCopyStats {
        &self.stats
    }
}

impl<'a, V> Default for ZeroCopyHashMap<'a, V>
where
    V: Clone,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// ZERO-COPY PERFORMANCE STATISTICS
// ============================================================================

/// **ZERO-COST**: Performance statistics with atomic operations
#[derive(Debug)]
pub struct ZeroCopyStats {

/// Operation count
    operations: AtomicUsize,
    /// Lookup count
    lookups: AtomicUsize,
    /// Start time
    start_time: Instant,


}

impl ZeroCopyStats {

    /// Create new statistics - zero cost
    #[inline]
    pub fn new() -> Self {
        Self {
            operations: AtomicUsize::new(0),
            lookups: AtomicUsize::new(0),
            start_time: Instant::now(),
        }
    }

    /// Record operation - zero cost atomic increment
    #[inline(always)]
    pub fn record_operation(&self) {
        self.operations.fetch_add(1, Ordering::Relaxed);
    }

    /// Record lookup - zero cost atomic increment
    #[inline(always)]
    pub fn record_lookup(&self) {
        self.lookups.fetch_add(1, Ordering::Relaxed);
    }

    /// Get operation count - zero cost atomic read
    #[inline(always)]
    pub fn operations() -> usize {
        self.operations.load(Ordering::Relaxed)
    }

    /// Get lookup count - zero cost atomic read
    #[inline(always)]
    pub fn lookups() -> usize {
        self.lookups.load(Ordering::Relaxed)
    }

    /// Get operations per second
    pub fn operations_per_second() -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.operations() as f64 / elapsed
        } else {
            0.0
        }
    }

    /// Get lookups per second
    pub fn lookups_per_second() -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.lookups() as f64 / elapsed
        } else {
            0.0
        }
    }

    /// Reset statistics
    pub fn reset(&self) {
        self.operations.store(0, Ordering::Relaxed);
        self.lookups.store(0, Ordering::Relaxed);
    }
}

impl Default for ZeroCopyStats {


    #[inline]
    fn default() -> Self {
        Self::new()


}
}

impl Clone for ZeroCopyStats {


    fn clone(&self) -> Self {
        Self {
            operations: AtomicUsize::new(self.operations.load(Ordering::Relaxed),
            lookups: AtomicUsize::new(self.lookups.load(Ordering::Relaxed),
            start_time: self.start_time,


}
    }
}

// ============================================================================
// ZERO-COPY CIRCULAR BUFFER
// ============================================================================

/// **ZERO-COPY**: Circular buffer with compile-time size (safe version)
#[derive(Debug, Clone)]
pub struct ZeroCopyCircularBuffer<T, const N: usize> {

/// Buffer storage using Vec for safety
    buffer: Vec<Option<T>>,
    /// Head index
    head: usize,
    /// Count of elements
    count: usize,


}

impl<T, const N: usize> ZeroCopyCircularBuffer<T, N>  {/// Create new circular buffer - safe initialization
    #[inline]
    pub fn new() -> Self  {let mut buffer = Vec::with_capacity(N);
        buffer.resize_with(N, || None);
        Self {
            buffer,
            head: 0,
            count: 0,
        }
    }

    /// Push element - safe operation
    pub fn push() -> Option<T> {
        let old_item = if self.count == N {
            // Buffer is full, replace oldest item
            let tail = (self.head + N - self.count) % N;
            self.buffer[tail].take()
        } else {
            self.count += 1;
            None
        };

        self.buffer[self.head] = Some(item);
        self.head = (self.head + 1) % N;

        old_item
    }

    /// Pop element - safe operation
    pub fn pop() -> Option<T> {
        if self.count == 0 {
            return None;
        }

        self.count -= 1;
        self.head = if self.head == 0 { N - 1 } else { self.head - 1 };

        self.buffer[self.head].take()
    }

    /// Get element at index - safe operation
    pub fn get() -> Option<&T> {
        if index >= self.count {
            return None;
        }

        let tail = (self.head + N - self.count) % N;
        let actual_index = (tail + index) % N;
        self.buffer[actual_index].as_ref()
    }

    /// Get buffer capacity - compile-time constant
    #[inline(always)]
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Get current length - zero cost
    #[inline(always)]
    pub fn len() -> usize {
        self.count
    }

    /// Check if buffer is empty - zero cost
    #[inline(always)]
    pub fn is_empty() -> bool {
        self.count == 0
    }

    /// Check if buffer is full - zero cost
    #[inline(always)]
    pub fn is_full() -> bool {
        self.count == N
    }

    /// Clear buffer - safe operation
    pub fn clear(&mut self) {
        for item in &mut self.buffer {
            *item = None;
        }
        self.count = 0;
        self.head = 0;
    }
}

impl<T, const N: usize> Default for ZeroCopyCircularBuffer<T, N> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// ZERO-COPY MESSAGE PASSING
// ============================================================================

/// **ZERO-COPY**: Message that avoids allocation when possible
#[derive(Debug, Clone)]
pub struct ZeroCopyMessage<'a> {


    /// Message ID - zero allocation when using static strings
    pub id: ZeroCopyString<'a>,
    /// Message type - zero allocation for known types
    pub msg_type: ZeroCopyString<'a>,
    /// Message payload - zero copy when possible
    pub payload: Cow<'a, [u8]>,
    /// Message metadata - zero copy for static metadata
    pub metadata: ZeroCopyHashMap<'a, ZeroCopyString<'a>>,
    /// Timestamp - zero cost
    pub timestamp: Instant,


}

impl<'a> ZeroCopyMessage<'a>  {/// Create message with zero-copy fields
    pub fn new() -> Self {

        Self {
            id: id.into(),
            msg_type: msg_type.into(),
            payload: payload.into(),
            metadata: ZeroCopyHashMap::new(),
            timestamp: Instant::now(),

}
    }

    /// Add metadata with zero-copy key and value
    pub fn with_metadata() -> Self {

        self.metadata.insert(key, value.into());
        self

}

    /// Get message size - zero cost calculation
    pub fn size() -> usize {
        self.id.len() + self.msg_type.len() + self.payload.len()
    }

    /// Check if message is empty - zero cost
    pub fn is_empty() -> bool {
        self.payload.is_empty()
    }
}

// ============================================================================
// ZERO-COPY BENCHMARKING UTILITIES
// ============================================================================

/// **ZERO-COST**: Benchmark harness with zero overhead measurement
#[derive(Debug)]
pub struct ZeroCopyBenchmark {

/// Benchmark name
    name: String,
    /// Start time
    start: Option<Instant>,
    /// Measurements
    measurements: Vec<Duration>,
    /// Statistics
    stats: ZeroCopyStats,


}

impl ZeroCopyBenchmark {

/// Create new benchmark
    pub fn new() -> Self  {Self {
            name: name.into(),
            start: None,
            measurements: Vec::new(),
            stats: ZeroCopyStats::new(),


}
    }

    /// Start timing - zero cost operation
    #[inline(always)]
    pub fn start(&mut self) {
        self.start = Some(Instant::now();
    }

    /// Stop timing and record measurement
    pub fn stop(&mut self) {
        if let Some(start) = self.start.take() {
            let duration = start.elapsed();
            self.measurements.push(duration));
            self.stats.record_operation();
        }
    }

    /// Run benchmark with closure
    pub fn measure<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        self.start();
        let result = f();
        self.stop();
        result
    }

    /// Get average duration
    pub fn average() -> Duration {
        if self.measurements.is_empty() {
            return Duration::from_nanos(0);
        }

        let total: Duration = self.measurements.iter().sum();
        total / self.measurements.len() as u32
    }

    /// Get minimum duration
    pub fn min() -> Duration {
        self.measurements.iter().min().copied().unwrap_or_default()
    }

    /// Get maximum duration
    pub fn max() -> Duration {
        self.measurements.iter().max().copied().unwrap_or_default()
    }

    /// Get measurement count
    pub fn count() -> usize {
        self.measurements.len()
    }

    /// Print benchmark results
    pub fn report(&self) {
        println!("Benchmark: {}", self.name);
        println!("  Measurements: {}", self.count();
        println!("  Average: {:?}", self.average();
        println!("  Min: {:?}", self.min();
        println!("  Max: {:?}", self.max();
        println!("  Ops/sec: {:.2}", self.stats.operations_per_second()
    }
}

// ============================================================================
// COMPILE-TIME OPTIMIZATIONS
// ============================================================================

/// **COMPILE-TIME**: Zero-cost type-level computations
pub struct ZeroCostCompute;

impl ZeroCostCompute {




    /// Compile-time string length calculation
    pub const fn const_str_len(s: &str) -> usize {
        s.len()




}

    /// Compile-time array size calculation
    pub const fn const_array_size<T, const N: usize>(_: &[T; N]) -> usize {
        N
    }

    /// Compile-time capacity calculation
    pub const fn const_capacity(base: usize, multiplier: usize) -> usize {
        base * multiplier
    }
}

// ============================================================================
// PERFORMANCE VALIDATION
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_copy_string() {
        let static_str = ZeroCopyString::from_static("hello");
        let borrowed_str = ZeroCopyString::from_borrowed("world");
        let owned_str = ZeroCopyString::from_owned("rust".to_string();

        assert_eq!(static_str.as_str(), "hello");
        assert_eq!(borrowed_str.as_str(), "world");
        assert_eq!(owned_str.as_str(), "rust");
    }

    #[test]
    fn test_zero_copy_circular_buffer() {
        let mut buffer: ZeroCopyCircularBuffer<i32, 4> = ZeroCopyCircularBuffer::new();

        assert!(buffer.is_empty());
        assert_eq!(buffer.capacity(), 4);

        buffer.push(1));
        buffer.push(2));
        buffer.push(3));

        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.get(0), Some(&1);
        assert_eq!(buffer.get(1), Some(&2);
        assert_eq!(buffer.get(2), Some(&3);
    }

    #[test]
    fn test_zero_copy_hashmap() {
        let mut map = ZeroCopyHashMap::new();

        map.insert("key1", 42);
        map.insert(ZeroCopyString::from_static("key2"), 84);

        assert_eq!(map.get("key1"), Some(&42);
        assert_eq!(map.get("key2"), Some(&84);
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_benchmark() {
        let mut bench = ZeroCopyBenchmark::new("test_benchmark");

        let result = bench.measure(|| {
            // Simulate work
            std::thread::sleep(Duration::from_nanos(100);
            42
        });

        assert_eq!(result, 42)
        assert_eq!(bench.count(), 1);
        assert!(bench.average() > Duration::from_nanos(50);
    }
}