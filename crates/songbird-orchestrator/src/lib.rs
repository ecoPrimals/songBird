// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![warn(missing_docs)]

//! Songbird Orchestrator Application
//!
//! This is the main orchestrator application that coordinates all Songbird services
//! and provides the primary interface for system management.
//!
//! ## Features
//!
//! - **Service Orchestration**: Coordinate multiple services and primals
//! - **CLI Interface**: Command-line interface for system management
//! - **Web Interface**: Web-based management dashboard
//! - **Health Monitoring**: System-wide health monitoring and alerting
//! - **Integration Management**: Manage integrations with external systems
//! - **Server Management**: HTTP server for API and web interface
//!
//! ## Architecture
//!
//! The orchestrator application consists of:
//!
//! 1. **App Core**: Main application logic and coordination
//! 2. **CLI Interface**: Command-line interface for operations
//! 3. **Integration Manager**: Manages external integrations
//! 4. **Server Manager**: HTTP server for API and web interface
//!
//! ## Usage
//!
//! ```rust,no_run
//! use songbird_orchestrator::SongbirdOrchestrator;
//! use songbird_types::config::CanonicalSongbirdConfig;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Load configuration from environment variables
//!     let config = CanonicalSongbirdConfig::from_env()?;
//!     let mut orchestrator = SongbirdOrchestrator::new(config).await?;
//!
//!     // Start the orchestrator
//!     orchestrator.start().await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Service Coordination
//!
//! The orchestrator coordinates:
//!
//! - Songbird core services
//! - Universal primals
//! - External integrations
//! - Health monitoring
//!
//! ## `UniBin` API
//!
//! For `UniBin` integration, this crate exposes public entry points:
//!
//! - `run_orchestrator()` - Main server mode entry point
//! - CLI types and functions (re-exported from main.rs)
//! - Performance optimization
//!
//! ## Management Interface
//!
//! Multiple management interfaces:
//!
//! - Command-line interface
//! - Web dashboard
//! - REST API
//! - Health check endpoints
//! - Metrics endpoints
// Lint policy for this crate lives in Cargo.toml `[lints]` so integration tests and bins
// receive the same allows as the library (inner attributes here do not apply to them).
#![forbid(unsafe_code)]

/// Access control and graduated information disclosure.
#[allow(missing_docs, reason = "internal module; doc coverage expanding incrementally")]
pub mod access_control;
/// Core application wiring and the main [`SongbirdOrchestrator`] entrypoint.
#[allow(missing_docs, reason = "internal module; doc coverage expanding incrementally")]
pub mod app;
/// JWT authentication delegated to BearDog over IPC.
#[allow(missing_docs, reason = "internal module; doc coverage expanding incrementally")]
pub mod auth;
/// UniBin public API: `run_server`, `run_doctor`, `run_config`, and related CLI types.
#[allow(missing_docs, reason = "internal module; doc coverage expanding incrementally")]
pub mod bin_interface;
/// BTSP Unix socket client for BearDog secure tunnels.
#[allow(missing_docs, reason = "internal module; doc coverage expanding incrementally")]
pub mod btsp_client;
/// Neural API capability registration for the local runtime.
#[allow(missing_docs, reason = "internal module; doc coverage expanding incrementally")]
pub mod capability_registration;
/// Orchestrator CLI parsing and command dispatch.
#[allow(missing_docs, reason = "internal module; doc coverage expanding incrementally")]
pub mod cli;
/// Server, doctor, and config command implementations.
#[allow(missing_docs, reason = "internal module; doc coverage expanding incrementally")]
pub mod commands;
/// Progressive trust and federated peer connection types.
#[allow(missing_docs, reason = "internal module; doc coverage expanding incrementally")]
pub mod connections;
/// User consent management for sensitive operations.
#[allow(missing_docs, reason = "internal module; doc coverage expanding incrementally")]
pub mod consent_management;
/// Consolidated orchestrator core (routing, execution, benchmarks, adapters).
#[allow(missing_docs, reason = "internal module; doc coverage expanding incrementally")]
pub mod core;
/// Pure Rust TLS and crypto delegation to BearDog.
#[allow(missing_docs, reason = "internal module; doc coverage expanding incrementally")]
pub mod crypto;
/// Environment-backed self-configuration for the orchestrator process.
#[allow(missing_docs, reason = "internal module; doc coverage expanding incrementally")]
pub mod env_config;
/// Error recovery, circuit breaking, and resilience helpers.
#[allow(missing_docs, reason = "internal module; doc coverage expanding incrementally")]
pub mod error_recovery;
/// Graph validation for collaborative intelligence workflows.
#[allow(missing_docs, reason = "internal module; doc coverage expanding incrementally")]
pub mod graph;
/// HTTP gateway for the universal pure Rust service surface.
#[allow(missing_docs, reason = "internal module; doc coverage expanding incrementally")]
pub mod http_gateway;
/// External system and service integration glue.
#[allow(missing_docs, reason = "internal module; doc coverage expanding incrementally")]
pub mod integration;
/// Inter-primal communication, Unix IPC, and primal registry.
#[allow(missing_docs, reason = "internal module; doc coverage expanding incrementally")]
pub mod ipc;
/// Network binding, listeners, and endpoint management.
#[allow(missing_docs, reason = "internal module; doc coverage expanding incrementally")]
pub mod network;
/// Stable node identity and cryptographic node identifiers.
#[allow(missing_docs, reason = "internal module; doc coverage expanding incrementally")]
pub mod node_identity;
/// Metrics, tracing hooks, and basic observability.
#[allow(missing_docs, reason = "internal module; doc coverage expanding incrementally")]
pub mod observability;
/// Legacy MVP orchestrator integration and health types.
#[allow(missing_docs, reason = "internal module; doc coverage expanding incrementally")]
pub mod orchestrator;
/// Capability-agnostic primal and provider endpoint discovery.
#[allow(missing_docs, reason = "internal module; doc coverage expanding incrementally")]
pub mod primal_discovery;
/// Privileged capability handling (e.g. network caps) without interactive sudo.
#[allow(missing_docs, reason = "internal module; doc coverage expanding incrementally")]
pub mod privilege;
/// Process lifecycle, spawning, and multi-instance coordination.
#[allow(missing_docs, reason = "internal module; doc coverage expanding incrementally")]
pub mod process_manager;
/// Node registration and genetic lineage with biomeOS.
#[allow(missing_docs, reason = "internal module; doc coverage expanding incrementally")]
pub mod registration;
/// Modern resilience patterns (circuit breaker, bulkhead, etc.).
#[allow(missing_docs, reason = "internal module; doc coverage expanding incrementally")]
pub mod resilience;
/// Resource limits, fairness, and orchestrator-wide resource policy.
#[allow(missing_docs, reason = "internal module; doc coverage expanding incrementally")]
pub mod resource_management;
/// Multi-protocol RPC (JSON-RPC, tarpc, etc.).
#[allow(missing_docs, reason = "internal module; doc coverage expanding incrementally")]
pub mod rpc;
/// HTTP client for security-capability and trust RPC against BearDog.
#[allow(missing_docs, reason = "internal module; doc coverage expanding incrementally")]
pub mod security_client;
/// Backward-compatible module alias for [`security_client`] (existing imports use `security_capability_client`).
pub use security_client as security_capability_client; // Backward compatibility alias
/// Runtime introspection: what this primal exposes without hardcoded names.
#[allow(missing_docs, reason = "internal module; doc coverage expanding incrementally")]
pub mod self_knowledge;
/// HTTP/API server stack for orchestrator endpoints.
#[allow(missing_docs, reason = "internal module; doc coverage expanding incrementally")]
pub mod server;
/// Inter-primal service registration and port authority.
#[allow(missing_docs, reason = "internal module; doc coverage expanding incrementally")]
pub mod service_registry;
/// Task lifecycle and scheduling hooks for orchestrated work.
#[allow(missing_docs, reason = "internal module; doc coverage expanding incrementally")]
pub mod task_lifecycle;
/// Trust escalation, evaluation, and progressive trust policy.
#[allow(missing_docs, reason = "internal module; doc coverage expanding incrementally")]
pub mod trust;
/// Universal primal adapter and capability-based provider discovery.
#[allow(missing_docs, reason = "internal module; doc coverage expanding incrementally")]
pub mod universal_adapter;

// Re-export main orchestrator
/// Primary orchestrator application type: loads config, wires subsystems, runs the server loop.
pub use app::SongbirdOrchestrator;

// Re-export security capability client (provider-agnostic!)
/// HTTP client for security-capability RPC against BearDog (trust evaluation and related calls).
pub use security_capability_client::{
    SecurityCapabilityClient, TrustEvaluationRequest, TrustEvaluationResponse,
};

// Re-export universal adapter (capability-based discovery!)
/// Capability-based discovery of external providers and adapter query API.
pub use universal_adapter::{CapabilityQuery, DiscoveredProvider, UniversalAdapter};

// Re-export self-knowledge (what we know about ourselves!)
// self_knowledge exports removed - module provides functions, not types

// Re-export MVP orchestrator integration
/// Legacy MVP orchestrator types retained for integration tests and gradual migration.
pub use orchestrator::{HealthStatus, OrchestratorConfig, SongbirdOrchestrator as MvpOrchestrator};

// Re-export all functionality from crates (consolidated from songbird-lib)
/// Re-export of [`songbird_config`] for orchestrator consumers.
pub use songbird_config as config;
/// Re-export of [`songbird_discovery`] for orchestrator consumers.
pub use songbird_discovery as discovery;
/// Re-export of [`songbird_registry`] for orchestrator consumers.
pub use songbird_registry as registry;

// Re-export commonly used types
/// Canonical Songbird configuration loaded from environment or files.
pub use songbird_types::config::CanonicalSongbirdConfig;
/// Top-level `SongbirdError` and `SongbirdResult` aliases used across the orchestrator API.
pub use songbird_types::{SongbirdError, SongbirdResult};

// Re-export key types that are commonly used
/// Discriminant for which primal role this process represents.
pub use songbird_universal::PrimalType;

// Re-export UniBin public API for easy access
/// UniBin CLI entrypoints: `run_server`, `run_doctor`, `run_config`, and argument structs.
pub use bin_interface::{
    ConfigCommands, DoctorArgs, ServerArgs, run_config, run_doctor, run_server,
};

// Re-export connection types (progressive trust)
/// Progressive trust connection types used by federation and peer flows.
pub use connections::{
    Connection, FederatedConnection, FullTrustConnection, LimitedConnection, PeerConnection,
};

// Re-export capability registration (Neural API integration)
/// Register Neural API capabilities with the local runtime when available.
pub use capability_registration::{
    check_neural_api_available, register_capabilities, unregister_capabilities,
};
