// SPDX-License-Identifier: AGPL-3.0-or-later
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
#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(
    clippy::missing_errors_doc,
    clippy::return_self_not_must_use,
    clippy::if_same_then_else,
    clippy::trivially_copy_pass_by_ref,
    clippy::cast_precision_loss,
    clippy::unused_async,
    reason = "registry crate: plugin API surface; doc and style exceptions"
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
        reason = "test code: relaxed lints for assertions, mock construction, and test ergonomics"
    )
)]

// Core modules
/// Federation-aware registry integration hooks.
pub mod federation;
/// Disk-backed and in-memory persistence for registry state.
pub mod persistence;
/// Dynamic plugin composition and runtime discovery.
pub mod plugin;
/// Core registry implementation and query API.
pub mod registry;
/// Shared registry value types (plugins, events, health).
pub mod types;

// Health and scaling modules (new clean implementations)
/// Health probe adapters for registered workloads.
pub mod health_new;
/// Autoscaling signals tied to registry entries.
pub mod scaling_new;

// Re-export commonly used types
pub use types::{
    Capability, CapabilityType, EventType, HealthCheckConfig, HealthCheckType, HealthStatus,
    Plugin, PluginId, PluginMetadata, RegistryEvent,
};

pub use registry::{Composable, PluginRegistry, Query, Registry};

// Legacy modules (health, scaling, service, production) removed — replaced by
// health_new and scaling_new above.

#[cfg(test)]
mod lib_smoke_tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use crate::federation::FederationState;
    use crate::{Plugin, PluginRegistry, Registry};

    #[tokio::test]
    async fn registry_default_register_and_list_roundtrip() {
        let mut reg = Registry::default();
        let p = Plugin::new("lib-smoke-plugin", "Lib Smoke", "0.0.1");
        let id = reg.register(p).await.unwrap();
        assert_eq!(id.as_str(), "lib-smoke-plugin");
        let plugins = reg.list().await;
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].name, "Lib Smoke");
    }

    #[test]
    fn federation_state_default_join_and_peer_roundtrip() {
        let mut s = FederationState::default();
        assert!(!s.is_joined());
        s.join();
        assert!(s.register_peer("peer-a"));
        assert!(s.has_peer("peer-a"));
        s.leave();
        assert!(!s.is_joined());
        assert_eq!(s.peer_count(), 0);
    }

    #[test]
    fn plugin_id_display_and_serde_roundtrip() {
        use crate::PluginId;
        let id = PluginId::new("pid-1");
        assert_eq!(id.to_string(), "pid-1");
        let json = serde_json::to_string(&id).unwrap();
        let back: PluginId = serde_json::from_str(&json).unwrap();
        assert_eq!(back.as_str(), "pid-1");
    }
}
