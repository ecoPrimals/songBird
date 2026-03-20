// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

// Module imports
//! Configuration Provider Trait
//!
//! Defines the interface for pluggable configuration backends
//! supporting file-based, environment, Consul, and other configuration sources.
//!
//! **CONSOLIDATED**: Now uses canonical definition from songbird-discovery
//! (November 10, 2025 - Trait Unification)

pub use songbird_discovery::traits::config::{ConfigProvider, ConfigProviderInfo, ConfigMetadata, ConfigFormat};
