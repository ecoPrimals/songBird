// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! # 🎯 Canonical Constants - Environment-Aware Defaults
//!
//! **ZERO HARDCODING SYSTEM**
//!
//! This module provides environment-aware defaults that eliminate hardcoded values
//! while maintaining secure defaults for development and production.
//!
//! **Status**: Phase 4 consolidation complete - merged from config/constants.rs
//! **Philosophy**: All values configurable via environment, calculated defaults for production
//!
//! ## Submodules (by responsibility)
//!
//! - `env_helpers` — shared env parsing utilities
//! - `ports_env` — bind addresses, port ranges, discovery/dashboard ports
//! - `timeouts_resources` — timeouts, default intervals, worker/buffer/batch limits
//! - `logging_cors_env` — log level, environment checks, CORS
//! - `protocol_identity` — protocol port maps, external address, node id
//! - `nested_defaults` — nested `network`/`health`/`resources`/`services` namespaces

#![allow(
    missing_docs,
    reason = "constants are self-describing; top-level module doc explains policy"
)]

/// Platform-aware directory resolution (logs, cache, data, config, temp).
pub mod directories;
/// Primal endpoint discovery and capability-based filtering.
pub mod primal_discovery;

pub use directories::*;
pub use primal_discovery::*;

mod env_helpers;
mod logging_cors_env;
mod nested_defaults;
mod ports_env;
mod protocol_identity;
mod timeouts_resources;

pub use env_helpers::read_process_env;
pub(crate) use env_helpers::{
    env_get_bool_with, env_or_default_with, env_parse_with, env_port_with,
};
pub use logging_cors_env::*;
pub use nested_defaults::*;
pub use ports_env::*;
pub use protocol_identity::*;
pub use timeouts_resources::*;

#[cfg(test)]
#[allow(clippy::expect_used, reason = "test assertions")]
mod tests;
