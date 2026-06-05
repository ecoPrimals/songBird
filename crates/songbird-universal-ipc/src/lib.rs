// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::clone_on_ref_ptr,
    reason = "Arc::clone() is idiomatic for shared ownership in IPC service contexts"
)]
#![allow(
    clippy::expect_used,
    reason = "IPC initialization invariants use expect() for startup-critical paths"
)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions"))]
#![warn(missing_docs)]

//! # 🌍 Songbird Universal IPC
//!
//! **Platform-agnostic IPC for ecoPrimals** - Works on ALL platforms!
//!
//! ## Overview
//!
//! This crate provides a universal IPC abstraction that works consistently across
//! all platforms, eliminating the need for `#[cfg(unix)]` and `#[cfg(windows)]`
//! in application code.
//!
//! **Supported Platforms**:
//! - ✅ Unix (Linux, macOS, BSD) - Unix domain sockets
//! - ✅ Windows - Named pipes (when implemented)
//! - ✅ Fallback - TCP localhost
//!
//! ## Architecture
//!
//! ```text
//! Application Layer:
//!   - Other primals use virtual paths by capability (e.g. security provider, capability crypto.delegate; formerly virtual id `beardog`)
//!
//! Universal IPC Layer (this crate):
//!   - Translates virtual paths to native endpoints
//!   - Platform abstraction via `PlatformIpcImpl` enum dispatch
//!
//! Platform Layer:
//!   - Unix: e.g. `/tmp/primal-security.sock` (legacy deployments: `primal-beardog.sock`)
//!   - Windows: e.g. `\\.\pipe\primal-security` (legacy: `primal-beardog`)
//!   - Fallback: 127.0.0.1:{port}
//! ```
//!
//! ## Quick Start
//!
//! ### Register and Listen (Server)
//!
//! ```rust,no_run
//! use songbird_universal_ipc::ipc;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!
//! // Initialize universal IPC
//! ipc::init()?;
//!
//! // Register this primal
//! let endpoint = ipc::register("myprimal", vec!["capability1".to_string()]).await?;
//!
//! // Listen for connections
//! let mut listener = ipc::listen(endpoint).await?;
//!
//! // Accept connections
//! while let Ok(mut stream) = listener.accept().await {
//!     // Handle connection...
//!     tokio::spawn(async move {
//!         // Use stream...
//!     });
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ### Connect (Client)
//!
//! ```rust,no_run
//! use songbird_universal_ipc::ipc;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!
//! // Connect to a capability provider (e.g. security provider / crypto.delegate; legacy path `/primal/beardog`)
//! let mut stream = ipc::connect("/primal/security-provider").await?;
//!
//! // Use stream (works the same on ALL platforms!)
//! use tokio::io::{AsyncReadExt, AsyncWriteExt};
//! stream.write_all(b"hello").await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Features
//!
//! - **Platform-Agnostic**: Same API on all platforms
//! - **Type-Safe**: Rust type system ensures correctness
//! - **Async**: Built on Tokio for high performance
//! - **Zero Overhead**: Minimal abstraction layer
//! - **Discoverable**: Capability-based service discovery
//!
//! ## Integration with Tower Atomic
//!
//! This crate is designed to work seamlessly with Tower Atomic (JSON-RPC over IPC):
//!
//! ```rust,no_run
//! # use songbird_universal_ipc::ipc;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Get universal stream (security provider example; legacy: `/primal/beardog`)
//! let stream = ipc::connect("/primal/security-provider").await?;
//!
//! // Use with Tower Atomic (JSON-RPC)
//! // ... Tower Atomic code here ...
//! # Ok(())
//! # }
//! ```
#![recursion_limit = "256"]
#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::items_after_statements,
    clippy::unused_async,
    clippy::unused_self,
    clippy::needless_continue,
    clippy::match_same_arms,
    clippy::cast_possible_wrap,
    reason = "universal IPC: broad surface; doc and style exceptions during consolidation"
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
        clippy::await_holding_invalid_type,
        reason = "test harnesses: intentional leniency for assertion ergonomics and legacy test patterns"
    )
)]

// Public modules
/// Capability registry types and provider trait for IPC-facing discovery.
#[allow(missing_docs, reason = "internal module; public items documented incrementally")]
pub mod capability;
/// Virtual (`/primal/...`) and native (socket, pipe, TCP) endpoint representations.
#[allow(missing_docs, reason = "internal module; public items documented incrementally")]
pub mod endpoint;
/// Universal IPC error type and result alias.
#[allow(missing_docs, reason = "internal module; public items documented incrementally")]
pub mod error;
/// JSON-RPC method handlers (HTTP, discovery, STUN, mesh, etc.).
#[allow(missing_docs, reason = "internal module; public items documented incrementally")]
pub mod handlers;
/// Primal self-description and introspection helpers for IPC consumers.
#[allow(missing_docs, reason = "internal module; public items documented incrementally")]
pub mod introspection;
/// Platform-specific IPC implementation and async stream abstractions.
#[allow(missing_docs, reason = "internal module; public items documented incrementally")]
pub mod platform;
/// In-memory registry of registered primals and service metadata.
#[allow(missing_docs, reason = "internal module; public items documented incrementally")]
pub mod registry;
/// Songbird IPC JSON-RPC broker and `IpcServiceHandler` entrypoints.
#[allow(missing_docs, reason = "internal module; public items documented incrementally")]
pub mod service;
/// Wire-protocol DTOs for JSON-RPC requests and responses.
#[allow(missing_docs, reason = "internal module; public items documented incrementally")]
pub mod service_types;
/// JSON-RPC over universal IPC (Tower Atomic integration).
#[allow(missing_docs, reason = "internal module; public items documented incrementally")]
pub mod tower_atomic;

// Public API
/// User-facing universal IPC API: `init`, `register`, `listen`, `connect`.
#[allow(missing_docs, reason = "internal module; public items documented incrementally")]
pub mod ipc;

// Re-exports
/// Capability registry and provider trait for IPC-facing discovery.
pub use capability::{CapabilityRegistry, Provider};
/// Resolved native socket paths and virtual primal paths (`/primal/...`).
pub use endpoint::{NativeEndpoint, VirtualEndpoint};
/// Universal IPC error type and result alias.
pub use error::{IpcError, IpcResult};
/// Async stream abstraction and platform IPC trait for listeners and connectors.
pub use platform::{AsyncStream, AsyncStreamImpl, PlatformIpcImpl};
/// In-memory service registry and metadata for registered primals.
pub use registry::{ServiceMetadata, ServiceRegistry};
/// Phase 2 transport-qualified endpoint for `ipc.resolve` consumers.
pub use service_types::TransportEndpoint;
