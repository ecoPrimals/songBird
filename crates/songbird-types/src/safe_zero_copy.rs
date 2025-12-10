//! Safe zero-copy patterns using modern Rust features
//!
//! Replaces unsafe zero-copy patterns with safe alternatives using Pin, MaybeUninit,
//! and careful lifetime management.

use std::pin::Pin;
use std::mem::MaybeUninit;
use std::marker::PhantomData;

/// Safe zero-copy buffer using Pin and MaybeUninit
///
/// Provides zero-copy access to uninitialized memory safely.
pub struct SafeZeroCopyBuffer<T> {
    data: Pin<Box<[MaybeUninit<T>]>>,
    initialized: usize,
    _marker: PhantomData<T>,
}

impl<T> SafeZeroCopyBuffer<T> {
    /// Create a new buffer with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        let mut vec = Vec::with_capacity(capacity);
        unsafe {
            vec.set_len(capacity);
        }
        let boxed = vec.into_boxed_slice();
        
        Self {
            data: Pin::new(boxed),
            initialized: 0,
            _marker: PhantomData,
        }
    }
    
    /// Get initialized portion as safe slice
    pub fn as_slice(&self) -> &[T] {
        // SAFETY: We track initialized count, only expose initialized portion
        unsafe {
            let ptr = self.data.as_ptr() as *const T;
            std::slice::from_raw_parts(ptr, self.initialized)
        }
    }
    
    /// Get mutable slice of initialized portion
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: We track initialized count and have exclusive access
        unsafe {
            let ptr = Pin::get_unchecked_mut(self.data.as_mut()).as_mut_ptr() as *mut T;
            std::slice::from_raw_parts_mut(ptr, self.initialized)
        }
    }
    
    /// Push initialized value (safe interface)
    pub fn push(&mut self, value: T) -> Result<(), T> {
        if self.initialized >= self.data.len() {
            return Err(value);
        }
        
        // SAFETY: We checked bounds and this index is uninitialized
        unsafe {
            let ptr = Pin::get_unchecked_mut(self.data.as_mut()).as_mut_ptr();
            ptr.add(self.initialized).write(MaybeUninit::new(value));
        }
        self.initialized += 1;
        Ok(())
    }
    
    /// Get number of initialized elements
    pub fn len(&self) -> usize {
        self.initialized
    }
    
    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.initialized == 0
    }
    
    /// Get total capacity
    pub fn capacity(&self) -> usize {
        self.data.len()
    }
}

impl<T> Drop for SafeZeroCopyBuffer<T> {
    fn drop(&mut self) {
        // SAFETY: We only drop initialized elements
        unsafe {
            let ptr = Pin::get_unchecked_mut(self.data.as_mut()).as_mut_ptr() as *mut T;
            std::ptr::drop_in_place(std::slice::from_raw_parts_mut(ptr, self.initialized));
        }
    }
}

/// Safe SIMD operations using std::simd (Rust 1.75+)
///
/// Replaces unsafe SIMD with safe portable_simd.
#[cfg(feature = "simd")]
pub mod safe_simd {
    use std::simd::{f32x8, SimdFloat, Simd};
    
    /// Vectorized addition (safe)
    pub fn add_slices(a: &[f32], b: &[f32]) -> Vec<f32> {
        assert_eq!(a.len(), b.len(), "Slices must have equal length");
        
        let mut result = Vec::with_capacity(a.len());
        let lanes = f32x8::LEN;
        
        // Process SIMD chunks
        let chunks = a.len() / lanes;
        for i in 0..chunks {
            let start = i * lanes;
            let va = f32x8::from_slice(&a[start..start + lanes]);
            let vb = f32x8::from_slice(&b[start..start + lanes]);
            let vr = va + vb; // Safe SIMD operation
            result.extend_from_slice(vr.as_array());
        }
        
        // Handle remainder
        let remainder_start = chunks * lanes;
        result.extend(
            a[remainder_start..].iter()
                .zip(&b[remainder_start..])
                .map(|(x, y)| x + y)
        );
        
        result
    }
    
    /// Vectorized dot product (safe)
    pub fn dot_product(a: &[f32], b: &[f32]) -> f32 {
        assert_eq!(a.len(), b.len(), "Slices must have equal length");
        
        let lanes = f32x8::LEN;
        let chunks = a.len() / lanes;
        let mut sum = f32x8::splat(0.0);
        
        // Accumulate SIMD chunks
        for i in 0..chunks {
            let start = i * lanes;
            let va = f32x8::from_slice(&a[start..start + lanes]);
            let vb = f32x8::from_slice(&b[start..start + lanes]);
            sum += va * vb;
        }
        
        // Sum SIMD lanes
        let mut result = sum.reduce_sum();
        
        // Add remainder
        let remainder_start = chunks * lanes;
        result += a[remainder_start..].iter()
            .zip(&b[remainder_start..])
            .map(|(x, y)| x * y)
            .sum::<f32>();
        
        result
    }
}

/// Safe atomic operations with type safety
///
/// Wraps Arc and atomic types with safe interfaces.
pub mod safe_atomics {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
    
    /// Thread-safe counter with safe operations
    #[derive(Clone)]
    pub struct SafeCounter {
        value: Arc<AtomicU64>,
    }
    
    impl SafeCounter {
        /// Create new counter
        pub fn new() -> Self {
            Self {
                value: Arc::new(AtomicU64::new(0)),
            }
        }
        
        /// Increment and return new value
        pub fn increment(&self) -> u64 {
            self.value.fetch_add(1, Ordering::Relaxed) + 1
        }
        
        /// Get current value
        pub fn get(&self) -> u64 {
            self.value.load(Ordering::Relaxed)
        }
        
        /// Reset to zero
        pub fn reset(&self) {
            self.value.store(0, Ordering::Relaxed);
        }
    }
    
    impl Default for SafeCounter {
        fn default() -> Self {
            Self::new()
        }
    }
    
    /// Thread-safe shared data with Arc<T>
    ///
    /// Alternative to raw atomic pointers.
    pub struct SharedData<T> {
        data: Arc<T>,
        version: Arc<AtomicUsize>,
    }
    
    impl<T> SharedData<T> {
        /// Create new shared data
        pub fn new(data: T) -> Self {
            Self {
                data: Arc::new(data),
                version: Arc::new(AtomicUsize::new(1)),
            }
        }
        
        /// Get reference to data
        pub fn get(&self) -> Arc<T> {
            Arc::clone(&self.data)
        }
        
        /// Get current version
        pub fn version(&self) -> usize {
            self.version.load(Ordering::Acquire)
        }
    }
    
    impl<T> Clone for SharedData<T> {
        fn clone(&self) -> Self {
            Self {
                data: Arc::clone(&self.data),
                version: Arc::clone(&self.version),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::safe_atomics::*;
    
    #[test]
    fn test_safe_zero_copy_buffer() {
        let mut buffer: SafeZeroCopyBuffer<u32> = SafeZeroCopyBuffer::with_capacity(10);
        
        buffer.push(1).unwrap();
        buffer.push(2).unwrap();
        buffer.push(3).unwrap();
        
        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn test_safe_zero_copy_buffer_capacity() {
        let buffer: SafeZeroCopyBuffer<u32> = SafeZeroCopyBuffer::with_capacity(5);
        
        assert_eq!(buffer.capacity(), 5);
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_safe_zero_copy_buffer_full() {
        let mut buffer: SafeZeroCopyBuffer<u32> = SafeZeroCopyBuffer::with_capacity(2);
        
        assert!(buffer.push(1).is_ok());
        assert!(buffer.push(2).is_ok());
        
        // Buffer full
        let result = buffer.push(3);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), 3);
    }

    #[test]
    fn test_safe_zero_copy_buffer_as_mut_slice() {
        let mut buffer: SafeZeroCopyBuffer<u32> = SafeZeroCopyBuffer::with_capacity(5);
        
        buffer.push(1).unwrap();
        buffer.push(2).unwrap();
        buffer.push(3).unwrap();
        
        let slice = buffer.as_mut_slice();
        slice[1] = 42;
        
        assert_eq!(buffer.as_slice(), &[1, 42, 3]);
    }

    #[test]
    fn test_safe_zero_copy_buffer_empty_slice() {
        let buffer: SafeZeroCopyBuffer<u32> = SafeZeroCopyBuffer::with_capacity(10);
        
        assert_eq!(buffer.as_slice(), &[]);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_safe_zero_copy_buffer_drop() {
        // Test that Drop works correctly (no memory leaks)
        let mut buffer: SafeZeroCopyBuffer<String> = SafeZeroCopyBuffer::with_capacity(3);
        
        buffer.push("hello".to_string()).unwrap();
        buffer.push("world".to_string()).unwrap();
        
        // Drop happens here - test just verifies no panic
    }

    #[test]
    fn test_safe_zero_copy_buffer_complex_type() {
        #[derive(Debug, PartialEq)]
        struct Complex {
            id: u32,
            name: String,
        }
        
        let mut buffer: SafeZeroCopyBuffer<Complex> = SafeZeroCopyBuffer::with_capacity(5);
        
        buffer.push(Complex { id: 1, name: "Alice".to_string() }).unwrap();
        buffer.push(Complex { id: 2, name: "Bob".to_string() }).unwrap();
        
        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer.as_slice()[0].name, "Alice");
        assert_eq!(buffer.as_slice()[1].name, "Bob");
    }
    
    #[test]
    fn test_safe_counter() {
        let counter = SafeCounter::new();
        
        assert_eq!(counter.increment(), 1);
        assert_eq!(counter.increment(), 2);
        assert_eq!(counter.get(), 2);
        
        counter.reset();
        assert_eq!(counter.get(), 0);
    }

    #[test]
    fn test_safe_counter_with_amount() {
        let counter = SafeCounter::new();
        
        assert_eq!(counter.add(5), 5);
        assert_eq!(counter.add(3), 8);
        assert_eq!(counter.get(), 8);
    }

    #[test]
    fn test_safe_counter_multiple_increments() {
        let counter = SafeCounter::new();
        
        for i in 1..=10 {
            assert_eq!(counter.increment(), i);
        }
        
        assert_eq!(counter.get(), 10);
    }

    #[test]
    fn test_safe_counter_reset_multiple_times() {
        let counter = SafeCounter::new();
        
        counter.increment();
        counter.increment();
        counter.reset();
        assert_eq!(counter.get(), 0);
        
        counter.increment();
        counter.reset();
        assert_eq!(counter.get(), 0);
    }
    
    #[test]
    fn test_shared_data() {
        let shared = SharedData::new(vec![1, 2, 3]);
        let data = shared.get();
        
        assert_eq!(*data, vec![1, 2, 3]);
        assert_eq!(shared.version(), 1);
    }

    #[test]
    fn test_shared_data_clone() {
        let shared1 = SharedData::new(42);
        let shared2 = shared1.clone();
        
        assert_eq!(*shared1.get(), 42);
        assert_eq!(*shared2.get(), 42);
        assert_eq!(shared1.version(), shared2.version());
    }

    #[test]
    fn test_shared_data_multiple_clones() {
        let original = SharedData::new("test".to_string());
        let clone1 = original.clone();
        let clone2 = original.clone();
        let clone3 = clone1.clone();
        
        assert_eq!(*original.get(), "test");
        assert_eq!(*clone1.get(), "test");
        assert_eq!(*clone2.get(), "test");
        assert_eq!(*clone3.get(), "test");
    }

    #[test]
    fn test_shared_data_arc_count() {
        let shared = SharedData::new(vec![1, 2, 3]);
        let arc1 = shared.get();
        let arc2 = shared.get();
        
        // Multiple Arc references to same data
        assert_eq!(*arc1, *arc2);
    }

    #[test]
    fn test_safe_atomics_counter_thread_safety() {
        // Test that SafeCounter is Send + Sync (compile-time test)
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<SafeCounter>();
    }

    #[test]
    fn test_shared_data_thread_safety() {
        // Test that SharedData is Send + Sync (compile-time test)
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<SharedData<Vec<u32>>>();
    }
    
    #[cfg(feature = "simd")]
    #[test]
    fn test_safe_simd_add() {
        use super::safe_simd::*;
        
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
        let b = vec![1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
        
        let result = add_slices(&a, &b);
        
        assert_eq!(result, vec![2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]);
    }
    
    #[cfg(feature = "simd")]
    #[test]
    fn test_safe_simd_dot() {
        use super::safe_simd::*;
        
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        
        let result = dot_product(&a, &b);
        
        // 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32
        assert_eq!(result, 32.0);
    }
}

