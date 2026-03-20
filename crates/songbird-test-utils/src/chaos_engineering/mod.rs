// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Chaos experiment definitions, configs, and a small runner for resilience tests.

pub mod config;
/// Orchestrates starting/stopping [`ChaosExperiment`] runs from tests.
pub mod manager;

// Re-export canonical types
pub use config::*;
pub use manager::ChaosEngineeringManager;
