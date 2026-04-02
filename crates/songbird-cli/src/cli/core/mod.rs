// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

pub mod cli;
pub mod constants;
// NOTE: errors module removed - use crate::errors::CliError instead
pub mod types;

// ============================================================================
// EXPLICIT EXPORTS - Replacing wildcard exports for API clarity
// ============================================================================

// Core CLI functionality
// pub use cli::Cli; // Cli struct is defined in types.rs and re-exported via mod.rs

// Error handling - using songbird_errors instead
// pub use types::{CliError, SongbirdResult} ; // Re-enabled - now properly defined as type aliases
// Constants re-exported from songbird_config::canonical::constants

// CLI Types and enums
// pub use types::{ColorMode, ConfigAction, DeploymentType, OutputFormat};
