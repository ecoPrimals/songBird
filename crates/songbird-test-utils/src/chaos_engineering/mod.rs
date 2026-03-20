// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

// Chaos Engineering Module
//
// Canonical chaos engineering system following modernization patterns.
// Refactored from large monolithic file into focused, maintainable modules.

pub mod config;
pub mod manager;

// Re-export canonical types
pub use config::*;
pub use manager::ChaosEngineeringManager;
