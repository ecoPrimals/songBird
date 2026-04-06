// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Songbird CLI Module
//!
//! Command-line interface for the Songbird Orchestrator
//! Makes distributed computing as simple as `songbird init`

#![allow(missing_docs, reason = "submodules self-document via clap attributes and help text")]

pub mod commands;
pub mod config;
pub mod core;
pub mod discovery;
pub mod templates;
pub mod types;
pub mod ui;

// Re-export main types and structures from core and types
pub use core::*;
pub use types::*;
