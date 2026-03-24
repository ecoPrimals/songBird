// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! # 🎮 Songbird Gaming CLI
//!
//! **MODERN GAMING-FOCUSED COMMAND-LINE INTERFACE** ✅
//!
//! Command-line interface for the Songbird gaming orchestrator platform, providing
//! streamlined tools for gaming session management, network optimization, and gaming federation.
//!
//! ## 🎯 **Gaming-First Features**
//!
//! - **🎮 Gaming Sessions**: Create, join, and manage gaming sessions
//! - **🌐 Network Gaming**: Gaming-specific network optimization and protocol support
//! - **🤝 Gaming Federation**: Multi-region gaming federation and matchmaking
//! - **📊 Gaming Metrics**: Real-time gaming performance and latency monitoring
//! - **🔧 Gaming Config**: Dynamic gaming configuration and protocol management
//! - **🛡️ Gaming Security**: Gaming-specific security and anti-cheat integration
//! - **🔍 Gaming Discovery**: Automatic gaming service discovery and scanning
//!
//! ## 🏗️ **Modern Architecture**
//!
//! The CLI is organized into focused gaming command modules:
//!
//! - `gaming`: Core gaming session and protocol commands
//! - `network`: Gaming network optimization and diagnostics
//! - `federation`: Gaming federation and matchmaking
//! - `security`: Gaming security and authentication
//! - `config`: Gaming configuration management
//!
//! ## 🚀 **Usage Examples**
//!
//! ### Gaming Session Commands
//! ```bash
//! # Quick start a gaming session
//! songbird gaming host --name "My Game" --protocol udp"
//!
//! # Join an existing gaming session
//! songbird gaming join GAME-CODE-123
//!
//! # Scan for nearby gaming sessions
//! songbird gaming scan --protocol all
//! ```
//!
//! ### Network Gaming Commands
//! ```bash
//! # Optimize network for gaming
//! songbird network optimize --game-mode
//!
//! # Test gaming network latency
//! songbird network test --gaming-protocols
//!
//! # Configure gaming port forwarding
//! songbird network ports --gaming --auto-configure
//! ```
//!
//! ### Gaming Federation Commands
//! ```bash
//! # Initialize gaming federation
//! songbird federation init --gaming
//!
//! # Join gaming federation
//! songbird federation join --gaming-endpoint https://gaming.example.com
//!
//! # Create gaming lobby
//! songbird federation lobby --create --name "Epic Battle""
//! ```
//!
//! ## 🔧 **Integration**
//!
//! Integrates seamlessly with the consolidated Songbird architecture:
//! - Uses `songbird-network-federation` for gaming protocols
//! - Uses `songbird-types` for unified error handling
//! - Uses `songbird-orchestrator` for session coordination
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    test,
    allow(
        deprecated,
        dead_code,
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
        unused_imports,
        unused_variables,
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
    )
)]
// unsafe_code is already forbidden at crate level
#![warn(clippy::all)]
#![warn(rust_2018_idioms)]
// CLI tool: Allow pedantic lints for user-facing code (lower test coverage ~5%, style flexibility)
#![expect(clippy::pedantic, reason = "intentional pattern; clippy false positive for this API")]
#![expect(clippy::nursery, reason = "intentional pattern; clippy false positive for this API")] // Allow nursery lints in CLI
// CLI tool: Allow specific patterns common in user-facing tools
#![expect(
    clippy::cast_precision_loss,
    reason = "intentional pattern; clippy false positive for this API"
)]
#![expect(clippy::unused_async, reason = "unused bindings/imports in this compilation unit")] // Many CLI commands prepared for future async operations
#![expect(
    clippy::struct_excessive_bools,
    reason = "intentional pattern; clippy false positive for this API"
)]

// Core CLI modules
/// Command tree, parsers, and shared CLI infrastructure.
pub mod cli;
/// Error wrappers surfaced to operators from CLI commands.
pub mod errors;

// Re-export main CLI types
pub use cli::{Cli, CliArgs, OutputFormat};
pub use errors::{CliError, SongbirdResult};
