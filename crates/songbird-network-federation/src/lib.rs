// SPDX-License-Identifier: AGPL-3.0-or-later
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
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![expect(clippy::module_name_repetitions, reason = "federation crate uses nested module layout")]
#![expect(
    clippy::pub_use,
    reason = "consolidated crate re-exports federation/network entry points"
)]
#![allow(
    clippy::missing_errors_doc,
    reason = "network federation: large API; # Errors docs added incrementally"
)]
#![expect(
    clippy::unused_async,
    clippy::struct_excessive_bools,
    clippy::items_after_statements,
    clippy::match_same_arms,
    reason = "network federation: large API; doc and style exceptions during consolidation"
)]
#![cfg_attr(
    test,
    allow(
        deprecated,
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
/// Security-provider integration traits (lineage, BirdSong, relay hooks); discovered by capability.
#[allow(missing_docs, reason = "internal module; public items documented incrementally")]
pub mod security;

/// BirdSong wire payloads and federation message types.
#[allow(missing_docs, reason = "internal module; public items documented incrementally")]
pub mod birdsong_payload;
/// security provider Secure Tunnel Protocol configuration and provider traits.
#[allow(missing_docs, reason = "internal module; public items documented incrementally")]
pub mod btsp;
/// Internal cryptography helpers shared by federation modules.
#[allow(missing_docs, reason = "internal module; public items documented incrementally")]
mod crypto_helpers;
/// Discovery mode selection (plaintext vs BirdSong-encrypted).
#[allow(missing_docs, reason = "internal module; public items documented incrementally")]
pub mod discovery_mode;
/// Federation coordinator, node metadata, and multi-node configuration.
#[allow(missing_docs, reason = "internal module; public items documented incrementally")]
pub mod federation;
/// Federation configuration types and node info structures.
pub mod federation_config;
/// Bridge between the network stack and federation-aware coordination.
#[allow(missing_docs, reason = "internal module; public items documented incrementally")]
pub mod integration;
/// Multi-federation contexts and boundary-aware routing.
#[allow(missing_docs, reason = "internal module; public items documented incrementally")]
pub mod multi_federation;
/// Network managers, providers, bind configuration, and transports.
#[allow(missing_docs, reason = "internal module; public items documented incrementally")]
pub mod network;
/// Protocol capability advertisement and tower/federation negotiation.
#[allow(missing_docs, reason = "internal module; public items documented incrementally")]
pub mod protocol_capability;
/// Rendezvous client for internet-scale peer discovery.
#[allow(missing_docs, reason = "internal module; public items documented incrementally")]
pub mod rendezvous;
/// Federated service registry and lookup helpers.
#[allow(missing_docs, reason = "internal module; public items documented incrementally")]
pub mod service_registry;
/// Federation and network shared runtime state.
#[allow(missing_docs, reason = "internal module; public items documented incrementally")]
pub mod state;
/// Zero-copy friendly registry for hot federation paths.
#[allow(missing_docs, reason = "internal module; public items documented incrementally")]
pub mod zero_copy_registry;
pub mod transport_impl;

// Re-export core types for convenience
/// `security provider` Secure Tunnel Protocol configuration and provider traits.
pub use btsp::{BtspConfig, BtspProvider, BtspProviderImpl, LocalBtspProvider};
/// Federation coordinator, node metadata, and configuration for multi-node setups.
pub use federation::FederationCoordinator;
pub use federation_config::{FederationConfig, NodeInfo};
/// Bridge between network stack and federation-aware coordination.
pub use integration::NetworkFederationBridge;
/// Network manager traits and configuration for binding and transport selection.
pub use network::{NetworkConfig, NetworkManager, NetworkProvider};
/// Protocol advertisement and capability negotiation for tower federation.
pub use protocol_capability::{
    Protocol, ProtocolCapability, ProtocolCapabilityManager, TowerCapabilities,
};

/// Transport adapter for network federation lifecycle.
pub use transport_impl::FederationTransport;

// Legacy compatibility removed - use canonical APIs directly
