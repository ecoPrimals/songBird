// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Songbird Registry
//!
//! Plugin registry and management system for Songbird.
//!
//! # Features
//! - Plugin registration and discovery
//! - Health monitoring
//! - Auto-scaling
//! - Event streaming
//!
//! # Example
//! ```no_run
//! use songbird_registry::{Registry, Plugin, PluginRegistry};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut registry = Registry::new();
//!     
//!     let plugin = Plugin::new("my-plugin", "My Plugin", "1.0.0");
//!     registry.register(plugin).await?;
//!     
//!     let plugins = registry.list().await;
//!     println!("Registered {} plugins", plugins.len());
//!     
//!     Ok(())
//! }
//! ```

#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(
    clippy::missing_errors_doc,
    clippy::return_self_not_must_use,
    clippy::if_same_then_else,
    clippy::trivially_copy_pass_by_ref,
    clippy::cast_precision_loss,
    clippy::unused_async
)]
#![cfg_attr(
    test,
    allow(
        deprecated,
        dead_code,
        unused_imports,
        unused_variables,
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::await_holding_lock,
        clippy::float_cmp,
        clippy::absurd_extreme_comparisons,
        clippy::nonminimal_bool,
        clippy::needless_collect,
        clippy::used_underscore_binding,
        clippy::overly_complex_bool_expr,
        clippy::assertions_on_constants,
        clippy::unreadable_literal,
        clippy::empty_line_after_doc_comments,
        clippy::field_reassign_with_default,
        clippy::unnecessary_wraps,
        clippy::no_effect_underscore_binding,
        clippy::return_self_not_must_use,
        clippy::duplicated_attributes,
        clippy::needless_pass_by_value,
        clippy::must_use_candidate,
        clippy::missing_panics_doc,
        clippy::missing_errors_doc,
        clippy::doc_markdown,
        clippy::wildcard_imports,
        clippy::enum_glob_use,
        clippy::unused_self,
        clippy::unnecessary_cast,
        clippy::items_after_test_module,
        clippy::clone_on_ref_ptr,
        clippy::default_trait_access,
        clippy::needless_range_loop,
        clippy::similar_names,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::unnecessary_unwrap,
        clippy::ignore_without_reason,
        clippy::case_sensitive_file_extension_comparisons,
        clippy::needless_update,
    )
)]

// Core modules
pub mod registry;
pub mod types;

// Health and scaling modules (new clean implementations)
pub mod health_new;
pub mod scaling_new;

// Re-export commonly used types
pub use types::{
    Capability, CapabilityType, EventType, HealthCheckConfig, HealthCheckType, HealthStatus,
    Plugin, PluginId, PluginMetadata, RegistryEvent,
};

pub use registry::{Composable, PluginRegistry, Query, Registry};

// Note: Old modules (health, scaling, plugin, service) are temporarily disabled
// during the rebuild. They will be replaced with modern implementations.

// Note: Service module functionality integrated into main registry module
// pub mod service;
// pub use service::{ServiceInstance, ServiceMetrics};

// Production modules remain for reference
// pub mod production;
// pub mod persistence;
