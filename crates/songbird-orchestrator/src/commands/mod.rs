// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Command handlers for Songbird orchestrator
//!
//! This module contains the command implementations for CLI commands.
//! Each command is isolated in its own module for testability and maintainability.
//!
//! ## Architecture
//!
//! ```text
//! main.rs (CLI definitions + dispatch)
//!     ↓
//! commands/mod.rs (re-exports)
//!     ↓
//! commands/{server,doctor,config}.rs (command implementations)
//! ```

pub mod config;
pub mod doctor;
pub mod server;

// Re-export command handlers for convenience
pub use config::{ConfigAction, run_config};
pub use doctor::run_doctor;
pub use server::run_server;
