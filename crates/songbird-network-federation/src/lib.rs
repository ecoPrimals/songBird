// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

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
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::pub_use)] // Re-exports are acceptable for consolidated crates
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
    clippy::match_same_arms
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
pub mod beardog; // ✨ NEW: BearDog integration traits (lineage, birdSong, relay)
pub mod birdsong_payload; // ✨ NEW: BirdSong payload structures
pub mod btsp; // ✨ NEW: BearDog Secure Tunnel Protocol interface
pub mod discovery_mode; // ✨ NEW: Discovery mode (plaintext vs birdSong)
pub mod federation;
pub mod integration;
pub mod multi_federation; // ✨ NEW: Multi-federation support with context-aware boundaries
pub mod network;
pub mod protocol_capability; // ✨ NEW: Protocol capability advertisement
pub mod rendezvous; // ✨ NEW: Rendezvous client for internet discovery
pub mod service_registry;
pub mod state;
// pub mod tls;  // ✅ DEPRECATED: Using songbird-tls instead (100% Pure Rust via BearDog!)
pub mod zero_copy_registry; // ✨ NEW: Zero-copy evolved registry

// Re-export core types for convenience
pub use btsp::{BtspConfig, BtspProvider, LocalBtspProvider};
pub use federation::{FederationConfig, FederationCoordinator, NodeInfo};
pub use integration::NetworkFederationBridge;
pub use network::{NetworkConfig, NetworkManager, NetworkProvider};
pub use protocol_capability::{
    Protocol, ProtocolCapability, ProtocolCapabilityManager, TowerCapabilities,
};

// Legacy compatibility removed - use canonical APIs directly
