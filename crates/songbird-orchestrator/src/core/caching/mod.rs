//! Caching subsystem for Songbird Universal Orchestrator Orchestrator
//!
//! This module provides comprehensive caching capabilities including: //! - Advanced caching with LRU, LFU, FIFO, and TTL-based eviction
//! - Integration with string interning for memory efficiency
//! - Real-time metrics and monitoring
//! - Zero-copy optimizations where possible

pub mod advanced_cache;

pub use advanced_cache::*;
