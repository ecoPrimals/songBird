// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

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
    pub fn new(data: Vec<T>) -> Self {
        if data.len() <= 64 {
            Self::Small(data)
        } else {
            Self::Large(data)
        }
    }

    /// Create from slice
    #[inline]
    pub fn from_slice(data: &[T]) -> Self {
        Self::new(data.to_vec())
    }

    /// Get buffer slice - zero cost operation
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        match self {
            Self::Small(vec) | Self::Large(vec) => vec.as_slice(),
        }
    }

    /// Get buffer length - zero cost
    #[inline]
    pub fn len(&self) -> usize {
        match self {
            Self::Small(vec) | Self::Large(vec) => vec.len(),
        }
    }

    /// Check if buffer is empty - zero cost
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Byte-specialized buffer alias for APIs that expect raw bytes.
pub type ZeroCopyBytes = ZeroCopyBuffer<u8>;

/// **ZERO-COPY**: Circular buffer with compile-time size (safe version)
#[derive(Debug, Clone)]
pub struct ZeroCopyCircularBuffer<T, const N: usize> {
    buffer: Vec<Option<T>>,
    head: usize,
    count: usize,
}

impl<T, const N: usize> ZeroCopyCircularBuffer<T, N> {
    /// Create new circular buffer - safe initialization
    #[inline]
    pub fn new() -> Self {
        let mut buffer = Vec::with_capacity(N);
        buffer.resize_with(N, || None);
        Self {
            buffer,
            head: 0,
            count: 0,
        }
    }

    /// Push element - safe operation
    pub fn push(&mut self, item: T) -> Option<T> {
        let old_item = if self.count == N {
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
    pub fn pop(&mut self) -> Option<T> {
        if self.count == 0 {
            return None;
        }

        self.count -= 1;
        self.head = if self.head == 0 {
            N - 1
        } else {
            self.head - 1
        };

        self.buffer[self.head].take()
    }

    /// Get element at index - safe operation
    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.count {
            return None;
        }

        let tail = (self.head + N - self.count) % N;
        let actual_index = (tail + index) % N;
        self.buffer[actual_index].as_ref()
    }

    /// Get buffer capacity - compile-time constant
    #[inline]
    pub const fn capacity() -> usize {
        N
    }

    /// Get current length - zero cost
    #[inline]
    pub fn len(&self) -> usize {
        self.count
    }

    /// Check if buffer is empty - zero cost
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Check if buffer is full - zero cost
    #[inline]
    pub fn is_full(&self) -> bool {
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
