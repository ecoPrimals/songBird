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

#[allow(missing_docs, reason = "orchestrator submodule; primary docs on crate root and re-exports")]
pub mod access_control; // Access control & graduated information disclosure (Q1 2025)
#[allow(missing_docs, reason = "orchestrator submodule; primary docs on crate root and re-exports")]
pub mod app;
#[allow(missing_docs, reason = "orchestrator submodule; primary docs on crate root and re-exports")]
pub mod auth; // JWT authentication via BearDog delegation (Pure Rust!) (Jan 17, 2026)
#[allow(missing_docs, reason = "orchestrator submodule; primary docs on crate root and re-exports")]
pub mod bin_interface; // ✅ UniBin public API (Jan 19, 2026)
#[allow(missing_docs, reason = "orchestrator submodule; primary docs on crate root and re-exports")]
pub mod btsp_client; // BTSP Unix socket client for BearDog tunnels (Jan 16, 2026)
#[allow(missing_docs, reason = "orchestrator submodule; primary docs on crate root and re-exports")]
pub mod capability_registration; // Neural API capability registration (TRUE PRIMAL) (Jan 25, 2026)
#[allow(missing_docs, reason = "orchestrator submodule; primary docs on crate root and re-exports")]
pub mod cli;
#[allow(missing_docs, reason = "orchestrator submodule; primary docs on crate root and re-exports")]
pub mod commands; // Server, doctor, config command implementations (extracted from main.rs)
#[allow(missing_docs, reason = "orchestrator submodule; primary docs on crate root and re-exports")]
pub mod connections; // Progressive trust connection management (Jan 2026)
#[allow(missing_docs, reason = "orchestrator submodule; primary docs on crate root and re-exports")]
pub mod consent_management;
#[allow(missing_docs, reason = "orchestrator submodule; primary docs on crate root and re-exports")]
pub mod core; // Consolidated core functionality
#[allow(missing_docs, reason = "orchestrator submodule; primary docs on crate root and re-exports")]
pub mod crypto; // Pure Rust TLS via BearDog crypto delegation (Jan 18, 2026) - Path to 100% ecoBin!
#[allow(missing_docs, reason = "orchestrator submodule; primary docs on crate root and re-exports")]
pub mod env_config; // Environment configuration - self-knowledge (TRUE PRIMAL) (Jan 21, 2026)
#[allow(missing_docs, reason = "orchestrator submodule; primary docs on crate root and re-exports")]
pub mod error_recovery; // Error recovery & resilience (Week 3 - Dec 18, 2025)
#[allow(missing_docs, reason = "orchestrator submodule; primary docs on crate root and re-exports")]
pub mod graph; // Graph validation for Collaborative Intelligence (Jan 11, 2026)
#[allow(missing_docs, reason = "orchestrator submodule; primary docs on crate root and re-exports")]
pub mod http_gateway; // HTTP gateway for universal pure Rust ecosystem (Jan 16, 2026)
#[allow(missing_docs, reason = "orchestrator submodule; primary docs on crate root and re-exports")]
pub mod integration;
#[allow(missing_docs, reason = "orchestrator submodule; primary docs on crate root and re-exports")]
pub mod ipc; // Inter-Primal Communication (Unix socket IPC + primal registry) (Jan 4, 2026)
#[allow(missing_docs, reason = "orchestrator submodule; primary docs on crate root and re-exports")]
pub mod network; // Network binding & endpoint management (Dec 20, 2025) - Zero-config intelligent binding
#[allow(missing_docs, reason = "orchestrator submodule; primary docs on crate root and re-exports")]
pub mod node_identity;
#[allow(missing_docs, reason = "orchestrator submodule; primary docs on crate root and re-exports")]
pub mod observability; // Basic observability (Week 4 - Dec 18, 2025)
#[allow(missing_docs, reason = "orchestrator submodule; primary docs on crate root and re-exports")]
pub mod orchestrator; // MVP Integration (Week 1-5 - Dec 18, 2025)
#[allow(missing_docs, reason = "orchestrator submodule; primary docs on crate root and re-exports")]
pub mod primal_discovery; // Agnostic primal discovery (TRUE PRIMAL) (Jan 21, 2026)
#[allow(missing_docs, reason = "orchestrator submodule; primary docs on crate root and re-exports")]
pub mod privilege; // Secure privilege management (Dec 20, 2025) - CAP_NET_ADMIN, no sudo prompts
#[allow(missing_docs, reason = "orchestrator submodule; primary docs on crate root and re-exports")]
pub mod process_manager; // Process lifecycle & multi-instance support (Jan 4, 2026) - Enables fractal scaling
#[allow(missing_docs, reason = "orchestrator submodule; primary docs on crate root and re-exports")]
pub mod registration; // Node registration with genetic lineage (Jan 1, 2026) - biomeOS integration
#[allow(missing_docs, reason = "orchestrator submodule; primary docs on crate root and re-exports")]
pub mod resilience; // Modern resilience patterns (Circuit Breaker, etc.) (Feb 3, 2026)
#[allow(missing_docs, reason = "orchestrator submodule; primary docs on crate root and re-exports")]
pub mod resource_management; // Resource management & fairness (Week 2 - Dec 18, 2025)
#[allow(missing_docs, reason = "orchestrator submodule; primary docs on crate root and re-exports")]
pub mod rpc; // Multi-protocol RPC (JSON-RPC, tarpc)
#[allow(missing_docs, reason = "orchestrator submodule; primary docs on crate root and re-exports")]
pub mod security_client; // Security capability client (refactored v4.9.0) - ✅ Pure Rust HTTP (Jan 21, 2026)
/// Backward-compatible module alias for [`security_client`] (existing imports use `security_capability_client`).
pub use security_client as security_capability_client; // Backward compatibility alias
#[allow(missing_docs, reason = "orchestrator submodule; primary docs on crate root and re-exports")]
pub mod self_knowledge; // Self-knowledge about this primal (zero hardcoding!) (Jan 1, 2026)
#[allow(missing_docs, reason = "orchestrator submodule; primary docs on crate root and re-exports")]
pub mod server;
#[allow(missing_docs, reason = "orchestrator submodule; primary docs on crate root and re-exports")]
pub mod service_registry; // Universal Port Authority (Dec 20, 2025) - Inter-primal service registration
#[allow(missing_docs, reason = "orchestrator submodule; primary docs on crate root and re-exports")]
pub mod task_lifecycle; // Task lifecycle management (Week 1 - Dec 18, 2025) // Consent management (Week 5 - Dec 18, 2025)
#[allow(missing_docs, reason = "orchestrator submodule; primary docs on crate root and re-exports")]
pub mod trust; // Trust escalation system (Dec 19, 2025) - Zero-trust progressive escalation
#[allow(missing_docs, reason = "orchestrator submodule; primary docs on crate root and re-exports")]
pub mod universal_adapter; // Universal Primal Adapter (capability-based discovery) (Jan 1, 2026)

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
