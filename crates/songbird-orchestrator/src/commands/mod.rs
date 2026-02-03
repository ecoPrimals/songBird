//! Command handlers for Songbird orchestrator
//!
//! This module contains the command pattern implementation for CLI commands.
//! Each command is isolated in its own module for better testability and maintainability.
//!
//! ## Architecture
//!
//! ```text
//! bin_interface.rs (CLI definitions)
//!     ↓
//! commands/mod.rs (command routing)
//!     ↓
//! commands/{server,doctor,config}.rs (command implementations)
//! ```
//!
//! ## Deep Debt Principles
//!
//! - ✅ Smart refactoring: Architectural separation, not just file splitting
//! - ✅ Modern idiomatic Rust: async/await, type-safe, error handling
//! - ✅ Testability: Commands isolated, dependencies injected
//! - ✅ Maintainability: Each command in its own module

pub mod config;
pub mod doctor;
pub mod server;

// Re-export command handlers for convenience
pub use config::run_config;
pub use doctor::run_doctor;
pub use server::run_server;
