// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! CLI-specific constants
//!
//! This module provides constants specific to the CLI interface

#![allow(missing_docs, reason = "filesystem constants are self-describing path fragments")]

/// CLI-specific file system constants
pub mod filesystem {
    /// Default configuration directory
    pub const DEFAULT_CONFIG_DIR: &str = ".songbird";

    /// Default configuration file name
    pub const DEFAULT_CONFIG_FILE: &str = "songbird.toml";

    /// Default data directory
    pub const DEFAULT_DATA_DIR: &str = ".songbird/data";

    /// Default log directory
    pub const DEFAULT_LOG_DIR: &str = ".songbird/logs";

    /// Maximum retry attempts for CLI operations
    pub const DEFAULT_MAX_RETRIES: u32 = 3;
}

/// CLI-specific network constants
pub mod network {
    /// Default timeout for network operations (ms,
    pub const DEFAULT_TIMEOUT_MS: u64 = 30000;

    /// Default port range for discovery
    pub const DEFAULT_DISCOVERY_PORT_START: u16 = 8000;
    pub const DEFAULT_DISCOVERY_PORT_END: u16 = 8100;
}
