//! Songbird CLI Module
//!
//! Command-line interface for the Songbird Orchestrator
//! Makes distributed computing as simple as `songbird init`

pub mod commands;
pub mod config;
pub mod core;
pub mod discovery;
pub mod templates;
pub mod ui;

// Re-export main types and structures from core
pub use core::*;
