// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Core type definitions for the Songbird Registry
//!
//! This module contains all the fundamental types used throughout the registry system.
//! Types are designed for clarity, safety, and zero-copy operations where possible.

pub mod capability;
pub mod event;
pub mod health;
pub mod plugin;

// Re-export commonly used types
pub use capability::{Capability, CapabilityType};
pub use event::{EventType, RegistryEvent};
pub use health::{HealthCheckConfig, HealthCheckType, HealthStatus};
pub use plugin::{Plugin, PluginId, PluginMetadata};
