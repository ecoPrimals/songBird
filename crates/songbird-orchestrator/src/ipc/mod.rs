//! Inter-Primal Communication (IPC) Module
//!
//! This module provides Unix socket-based IPC for Songbird to communicate with
//! other primals (security provider, ToadStool, Gorilla, etc.) using JSON-RPC 2.0.
//!
//! ## Architecture
//!
//! ```text
//! ┌───────────────────────────────────────────────────────────────┐
//! │                      Songbird                                 │
//! │                                                               │
//! │  ┌──────────────┐         ┌─────────────────┐               │
//! │  │   Unix       │         │    Primal       │               │
//! │  │   Socket     │◄────────│    Registry     │               │
//! │  │   Server     │         │ (Capabilities)  │               │
//! │  └──────┬───────┘         └─────────────────┘               │
//! │         │                                                     │
//! └─────────┼─────────────────────────────────────────────────────┘
//!           │ /tmp/songbird-{family}.sock
//!           │
//!     ┌─────┴──────┬──────────┬──────────────┐
//!     │            │          │              │
//! ┌───┴────┐  ┌───┴────┐ ┌───┴────┐   ┌────┴────┐
//! │security provider │  │ToadStol│ │Gorilla │   │ Future  │
//! │        │  │        │ │        │   │ Primals │
//! │security│  │storage │ │compute │   │   ...   │
//! └────────┘  └────────┘ └────────┘   └─────────┘
//! ```
//!
//! ## Key Features
//!
//! - **Unix Socket IPC**: Low-latency, secure local communication
//! - **JSON-RPC 2.0**: Language-agnostic protocol
//! - **Capability Registry**: Dynamic primal discovery by capability
//! - **Zero Hardcoding**: No compile-time knowledge of specific primals
//!
//! ## Usage
//!
//! ```rust,no_run
//! use songbird_orchestrator::ipc::{UnixSocketIpcServer, PrimalRegistry};
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Start IPC server
//! let server = UnixSocketIpcServer::new("/tmp/songbird.sock").await?;
//! let registry = server.registry();
//!
//! // Server accepts connections and handles primal registration
//! tokio::spawn(async move {
//!     server.start().await.unwrap();
//! });
//!
//! // Query registry for capabilities
//! let security_provider = registry.read().await.get_provider("security").await?;
//! # Ok(())
//! # }
//! ```

pub mod primal_registry;
pub mod unix_socket;

pub use primal_registry::{PrimalInfo, PrimalRegistry, RegistryStats};
pub use unix_socket::{
    JsonRpcError, JsonRpcRequest, JsonRpcResponse, UnixSocketIpcServer,
};

