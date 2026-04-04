// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use std::borrow::Cow;
use std::time::{Duration, Instant};

use super::map::{ZeroCopyHashMap, ZeroCopyStats};
use super::string::ZeroCopyString;

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

impl<'a> ZeroCopyMessage<'a> {
    /// Create message with zero-copy fields
    pub fn new(
        id: impl Into<ZeroCopyString<'a>>,
        msg_type: impl Into<ZeroCopyString<'a>>,
        payload: impl Into<Cow<'a, [u8]>>,
    ) -> Self {
        Self {
            id: id.into(),
            msg_type: msg_type.into(),
            payload: payload.into(),
            metadata: ZeroCopyHashMap::new(),
            timestamp: Instant::now(),
        }
    }

    /// Add metadata with zero-copy key and value
    pub fn with_metadata(
        mut self,
        key: impl Into<ZeroCopyString<'a>>,
        value: impl Into<ZeroCopyString<'a>>,
    ) -> Self {
        self.metadata.insert(key, value.into());
        self
    }

    /// Get message size - zero cost calculation
    pub fn size(&self) -> usize {
        self.id.len() + self.msg_type.len() + self.payload.len()
    }

    /// Check if message is empty - zero cost
    pub fn is_empty(&self) -> bool {
        self.payload.is_empty()
    }
}

/// **ZERO-COST**: Benchmark harness with zero overhead measurement
#[derive(Debug)]
pub struct ZeroCopyBenchmark {
    name: String,
    start: Option<Instant>,
    measurements: Vec<Duration>,
    stats: ZeroCopyStats,
}

impl ZeroCopyBenchmark {
    /// Create new benchmark
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start: None,
            measurements: Vec::new(),
            stats: ZeroCopyStats::new(),
        }
    }

    /// Start timing - zero cost operation
    #[inline]
    pub fn start(&mut self) {
        self.start = Some(Instant::now());
    }

    /// Stop timing and record measurement
    pub fn stop(&mut self) {
        if let Some(start) = self.start.take() {
            let duration = start.elapsed();
            self.measurements.push(duration);
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
    #[allow(
        clippy::cast_possible_truncation,
        reason = "measurement count won't exceed u32::MAX in practice"
    )]
    pub fn average(&self) -> Duration {
        if self.measurements.is_empty() {
            return Duration::from_nanos(0);
        }

        let total: Duration = self.measurements.iter().sum();
        total / self.measurements.len() as u32
    }

    /// Get minimum duration
    pub fn min(&self) -> Duration {
        self.measurements.iter().min().copied().unwrap_or_default()
    }

    /// Get maximum duration
    pub fn max(&self) -> Duration {
        self.measurements.iter().max().copied().unwrap_or_default()
    }

    /// Get measurement count
    pub fn count(&self) -> usize {
        self.measurements.len()
    }

    /// Log benchmark results via `tracing::info!`
    pub fn report(&self) {
        tracing::info!(
            benchmark = %self.name,
            measurements = self.count(),
            average = ?self.average(),
            min = ?self.min(),
            max = ?self.max(),
            ops_per_sec = format_args!("{:.2}", self.stats.operations_per_second()),
            "benchmark report"
        );
    }
}

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
