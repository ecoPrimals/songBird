// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![warn(missing_docs)]

//! # 🌐 Songbird Network Federation
//!
//! **CONSOLIDATED NETWORK & FEDERATION CRATE** ✅
//!
//! This crate provides unified networking and federation capabilities for Songbird,
//! consolidating the previously fragmented network and federation functionality.
//!
//! ## 🎯 **Domain Consolidation Benefits**
//!
//! - ✅ **Unified Network Stack**: Single source for all networking functionality
//! - ✅ **Federation Integration**: Seamless network-federation coordination
//! - ✅ **Modern Rust**: Latest networking patterns with async/await
//! - ✅ **Zero Technical Debt**: Clean implementation with no legacy baggage
//! - ✅ **Gaming Protocols**: Specialized gaming network protocol support
#![forbid(unsafe_code)]
#![deny(unsafe_code)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions, reason = "federation crate uses nested module layout")]
#![allow(clippy::pub_use, reason = "consolidated crate re-exports federation/network entry points")]
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::upper_case_acronyms,
    clippy::trivially_copy_pass_by_ref,
    clippy::cast_possible_truncation,
    clippy::unused_async,
    clippy::unused_self,
    clippy::if_same_then_else,
    clippy::struct_field_names,
    clippy::struct_excessive_bools,
    clippy::items_after_statements,
    clippy::match_same_arms,
    reason = "network federation: large API; doc and style exceptions during consolidation"
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
#[allow(missing_docs, reason = "federation submodule; primary docs on crate root and re-exports")]
pub mod beardog; // ✨ NEW: BearDog integration traits (lineage, birdSong, relay)
#[allow(missing_docs, reason = "federation submodule; primary docs on crate root and re-exports")]
pub mod birdsong_payload; // ✨ NEW: BirdSong payload structures
#[allow(missing_docs, reason = "federation submodule; primary docs on crate root and re-exports")]
pub mod btsp; // ✨ NEW: BearDog Secure Tunnel Protocol interface
#[allow(missing_docs, reason = "federation submodule; primary docs on crate root and re-exports")]
pub mod discovery_mode; // ✨ NEW: Discovery mode (plaintext vs birdSong)
#[allow(missing_docs, reason = "federation submodule; primary docs on crate root and re-exports")]
pub mod federation;
#[allow(missing_docs, reason = "federation submodule; primary docs on crate root and re-exports")]
pub mod integration;
#[allow(missing_docs, reason = "federation submodule; primary docs on crate root and re-exports")]
pub mod multi_federation; // ✨ NEW: Multi-federation support with context-aware boundaries
#[allow(missing_docs, reason = "federation submodule; primary docs on crate root and re-exports")]
pub mod network;
#[allow(missing_docs, reason = "federation submodule; primary docs on crate root and re-exports")]
pub mod protocol_capability; // ✨ NEW: Protocol capability advertisement
#[allow(missing_docs, reason = "federation submodule; primary docs on crate root and re-exports")]
pub mod rendezvous; // ✨ NEW: Rendezvous client for internet discovery
#[allow(missing_docs, reason = "federation submodule; primary docs on crate root and re-exports")]
pub mod service_registry;
#[allow(missing_docs, reason = "federation submodule; primary docs on crate root and re-exports")]
pub mod state;
// pub mod tls;  // ✅ DEPRECATED: Using songbird-tls instead (100% Pure Rust via BearDog!)
#[allow(missing_docs, reason = "federation submodule; primary docs on crate root and re-exports")]
pub mod zero_copy_registry; // ✨ NEW: Zero-copy evolved registry

// Re-export core types for convenience
/// `BearDog` Secure Tunnel Protocol configuration and provider traits.
pub use btsp::{BtspConfig, BtspProvider, LocalBtspProvider};
/// Federation coordinator, node metadata, and configuration for multi-node setups.
pub use federation::{FederationConfig, FederationCoordinator, NodeInfo};
/// Bridge between network stack and federation-aware coordination.
pub use integration::NetworkFederationBridge;
/// Network manager traits and configuration for binding and transport selection.
pub use network::{NetworkConfig, NetworkManager, NetworkProvider};
/// Protocol advertisement and capability negotiation for tower federation.
pub use protocol_capability::{
    Protocol, ProtocolCapability, ProtocolCapabilityManager, TowerCapabilities,
};

// Legacy compatibility removed - use canonical APIs directly
