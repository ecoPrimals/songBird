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
//! ## UniBin API
//!
//! For UniBin integration, this crate exposes public entry points:
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
#![forbid(unsafe_code)]
#![allow(
    dead_code,
    unused_variables,
    // Documentation lints - evolving toward full coverage
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::doc_link_with_quotes,
    // Structural/style lints
    clippy::used_underscore_binding,
    clippy::no_effect_underscore_binding,
    clippy::items_after_statements,
    clippy::struct_excessive_bools,
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::similar_names,
    clippy::unreadable_literal,
    clippy::needless_pass_by_value,
    clippy::trivially_copy_pass_by_ref,
    clippy::unnecessary_wraps,
    clippy::manual_let_else,
    clippy::match_same_arms,
    clippy::match_wildcard_for_single_variants,
    clippy::needless_continue,
    clippy::should_implement_trait,
    clippy::missing_fields_in_debug,
    // Async patterns - many are stubs or trait-conforming
    clippy::unused_async,
    clippy::unused_self,
    clippy::future_not_send,
    // Cast lints - each site verified for correctness
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    // Builder/fluent API patterns
    clippy::return_self_not_must_use,
    clippy::double_must_use,
    // Clone clarity - Arc::clone() preferred but not enforced here
    clippy::clone_on_ref_ptr,
)]

pub mod access_control; // Access control & graduated information disclosure (Q1 2025)
pub mod app;
pub mod auth; // JWT authentication via BearDog delegation (Pure Rust!) (Jan 17, 2026)
pub mod bin_interface; // ✅ UniBin public API (Jan 19, 2026)
pub mod btsp_client; // BTSP Unix socket client for BearDog tunnels (Jan 16, 2026)
pub mod capability_registration; // Neural API capability registration (TRUE PRIMAL) (Jan 25, 2026)
pub mod cli;
pub mod connections; // Progressive trust connection management (Jan 2026)
pub mod consent_management;
pub mod core; // Consolidated core functionality
pub mod crypto; // Pure Rust TLS via BearDog crypto delegation (Jan 18, 2026) - Path to 100% ecoBin!
pub mod env_config; // Environment configuration - self-knowledge (TRUE PRIMAL) (Jan 21, 2026)
pub mod error_recovery; // Error recovery & resilience (Week 3 - Dec 18, 2025)
pub mod graph; // Graph validation for Collaborative Intelligence (Jan 11, 2026)
pub mod http_gateway; // HTTP gateway for universal pure Rust ecosystem (Jan 16, 2026)
pub mod integration;
pub mod ipc; // Inter-Primal Communication (Unix socket IPC + primal registry) (Jan 4, 2026)
pub mod network; // Network binding & endpoint management (Dec 20, 2025) - Zero-config intelligent binding
pub mod node_identity;
pub mod observability; // Basic observability (Week 4 - Dec 18, 2025)
pub mod orchestrator; // MVP Integration (Week 1-5 - Dec 18, 2025)
pub mod primal_discovery; // Agnostic primal discovery (TRUE PRIMAL) (Jan 21, 2026)
pub mod privilege; // Secure privilege management (Dec 20, 2025) - CAP_NET_ADMIN, no sudo prompts
pub mod process_manager; // Process lifecycle & multi-instance support (Jan 4, 2026) - Enables fractal scaling
pub mod registration; // Node registration with genetic lineage (Jan 1, 2026) - biomeOS integration
pub mod resilience; // Modern resilience patterns (Circuit Breaker, etc.) (Feb 3, 2026)
pub mod resource_management; // Resource management & fairness (Week 2 - Dec 18, 2025)
pub mod rpc; // Multi-protocol RPC (JSON-RPC, tarpc)
pub mod security_client; // Security capability client (refactored v4.9.0) - ✅ Pure Rust HTTP (Jan 21, 2026)
pub use security_client as security_capability_client; // Backward compatibility alias
pub mod self_knowledge; // Self-knowledge about this primal (zero hardcoding!) (Jan 1, 2026)
pub mod server;
pub mod service_registry; // Universal Port Authority (Dec 20, 2025) - Inter-primal service registration
pub mod task_lifecycle; // Task lifecycle management (Week 1 - Dec 18, 2025) // Consent management (Week 5 - Dec 18, 2025)
pub mod trust; // Trust escalation system (Dec 19, 2025) - Zero-trust progressive escalation
pub mod universal_adapter; // Universal Primal Adapter (capability-based discovery) (Jan 1, 2026)

// Re-export main orchestrator
pub use app::SongbirdOrchestrator;

// Re-export security capability client (provider-agnostic!)
pub use security_capability_client::{
    SecurityCapabilityClient, TrustEvaluationRequest, TrustEvaluationResponse,
};

// Re-export universal adapter (capability-based discovery!)
pub use universal_adapter::{CapabilityQuery, DiscoveredProvider, UniversalAdapter};

// Re-export self-knowledge (what we know about ourselves!)
// self_knowledge exports removed - module provides functions, not types

// Re-export MVP orchestrator integration
pub use orchestrator::{HealthStatus, OrchestratorConfig, SongbirdOrchestrator as MvpOrchestrator};

// Re-export all functionality from crates (consolidated from songbird-lib)
pub use songbird_config as config;
pub use songbird_discovery as discovery;
pub use songbird_registry as registry;

// Re-export commonly used types
pub use songbird_types::config::CanonicalSongbirdConfig;
pub use songbird_types::{SongbirdError, SongbirdResult};

// Re-export key types that are commonly used
pub use songbird_universal::PrimalType;

// Re-export UniBin public API for easy access
pub use bin_interface::{
    run_config, run_doctor, run_server, ConfigCommands, DoctorArgs, ServerArgs,
};

// Re-export connection types (progressive trust)
pub use connections::{
    Connection, FederatedConnection, FullTrustConnection, LimitedConnection, PeerConnection,
};

// Re-export capability registration (Neural API integration)
pub use capability_registration::{
    check_neural_api_available, register_capabilities, unregister_capabilities,
};
