// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! # 🔧 Unified Universal Adapter
//!
//! **SINGLE SOURCE OF TRUTH FOR UNIVERSAL ADAPTATION** ✅
//!
//! This module consolidates all fragmented `UniversalCapabilityAdapter` implementations
//! into a single, unified adapter that can handle any capability type.

mod adapter;
mod error;
mod types;

pub use adapter::UnifiedUniversalAdapter;
pub use error::UniversalAdapterError;
pub use types::{CapabilityRegistry, RegistryStats, ServiceConnection, UnifiedAdapterConfig};
