// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Auto-scaling module
//!
//! Provides automatic scaling based on metrics and policies.

pub mod engine;
pub mod policy;

// Re-export public items
pub use engine::ScalingEngine;
pub use policy::{ScalingAction, ScalingPolicy};
