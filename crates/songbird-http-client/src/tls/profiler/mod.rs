// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Server Profiling System
//!
//! Learns from successful and failed TLS connections to optimize future handshakes.
//! Profiles are persisted and shared across connections for continuous improvement.

mod profiler_impl;
mod types;

#[cfg(test)]
mod tests;

pub use profiler_impl::ServerProfiler;
pub use types::{GlobalStats, ServerProfile};
