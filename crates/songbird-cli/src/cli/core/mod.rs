// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

pub mod cli;
pub mod constants;
// NOTE: errors module removed - use crate::errors::CliError instead
pub mod types;

// ============================================================================
// EXPLICIT EXPORTS - Replacing wildcard exports for API clarity
// ============================================================================

// Constants re-exported from songbird_config::canonical::constants

// CLI types (`ColorMode`, `ConfigAction`, etc.): import from `crate::cli::core::types` at use sites.
