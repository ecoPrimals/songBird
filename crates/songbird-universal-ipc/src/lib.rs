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
//!   - BearDog, Squirrel, etc. use virtual paths: "/primal/beardog"
//!
//! Universal IPC Layer (this crate):
//!   - Translates virtual paths to native endpoints
//!   - Platform abstraction via PlatformIPC trait
//!
//! Platform Layer:
//!   - Unix: /tmp/primal-beardog.sock
//!   - Windows: \\.\pipe\primal-beardog
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
//! // Connect to a primal
//! let mut stream = ipc::connect("/primal/beardog").await?;
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
//! // Get universal stream
//! let stream = ipc::connect("/primal/beardog").await?;
//!
//! // Use with Tower Atomic (JSON-RPC)
//! // ... Tower Atomic code here ...
//! # Ok(())
//! # }
//! ```
#![recursion_limit = "256"]
#![forbid(unsafe_code)]
#![deny(unsafe_code)]
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
    clippy::cast_possible_wrap
)]

// Public modules
pub mod capability; // ✨ NEW: Capability-based discovery
pub mod endpoint;
pub mod error;
pub mod handlers; // ✨ NEW: IPC method handlers (HTTP, etc.)
pub mod introspection; // Smart refactor: primal self-description (Feb 8, 2026)
pub mod platform;
pub mod registry;
pub mod service; // ✨ NEW: IPC Service (JSON-RPC broker)
pub mod tower_atomic; // ✨ NEW: JSON-RPC over Universal IPC

// Public API
pub mod ipc;

// Re-exports
pub use capability::{CapabilityRegistry, Provider};
pub use endpoint::{NativeEndpoint, VirtualEndpoint};
pub use error::{IpcError, IpcResult};
pub use platform::{AsyncStream, PlatformIPC};
pub use registry::{ServiceMetadata, ServiceRegistry};
