//! Network optimization modules for Songbird Universal Orchestrator
//!
//! This module provides advanced network optimizations including:
//! - Zero-copy operations for minimal memory allocation
//! - Buffer pooling and reuse strategies
//! - High-performance I/O patterns
//! - Integration with performance monitoring

pub mod zero_copy_network;

pub use zero_copy_network::*;
