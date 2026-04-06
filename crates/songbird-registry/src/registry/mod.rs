// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Registry module
//!
//! Core registry functionality and traits.

pub mod core;
pub mod query;
pub mod traits;

// Re-export public items
pub use core::Registry;
pub use query::Query;
pub use traits::{Composable, PluginRegistry};
