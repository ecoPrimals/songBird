//! Songbird Federation - Distributed Orchestration Network
//!
//! This crate provides federation capabilities for the Songbird Universal Orchestrator,
//! enabling distributed orchestration across multiple nodes and networks.
//!
//! ## Features
//!
//! - **Distributed Orchestration**: Coordinate services across multiple nodes
//! - **Federation Management**: Manage federation topology and membership
//! - **Encrypted Communication**: Secure communication between federation nodes
//! - **Route Optimization**: Intelligent routing based on network conditions
//! - **Health Monitoring**: Continuous health checks across the federation
//! - **MCP Protocol**: Model Context Protocol for AI service coordination
//!
//! ## Architecture
//!
//! The federation system consists of:
//!
//! 1. **Federation Manager**: Coordinates federation membership and topology
//! 2. **Discovery Engine**: Discovers and monitors federation nodes
//! 3. **Route Optimizer**: Optimizes routing between federation nodes
//! 4. **Security Manager**: Handles authentication and encryption
//! 5. **Deployment Manager**: Manages distributed deployments
//! 6. **MCP Handler**: Implements Model Context Protocol for AI services
//!
//! ## Usage
//!
//! ```rust
//! use songbird_federation::FederationManager;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let federation = FederationManager::new().await?;
//!     
//!     // Join federation
//!     federation.join_federation("node-1").await?;
//!     
//!     // Start federation services
//!     federation.start().await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Federation Management
//!
//! Key federation management features:
//!
//! - Node discovery and registration
//! - Topology management
//! - Health monitoring
//! - Load balancing across nodes
//! - Failure detection and recovery
//!
//! ## Security
//!
//! Security features include:
//!
//! - End-to-end encryption
//! - Node authentication
//! - Certificate management
//! - Access control
//! - Audit logging
//!
//! ## MCP Protocol
//!
//! Model Context Protocol support:
//!
//! - AI service coordination
//! - Context sharing
//! - Model management
//! - Resource allocation
//! - Performance optimization

#![allow(dead_code)]

pub mod config;
pub mod deployment;
pub mod discovery;
pub mod manager;
pub mod mcp_handler;
pub mod messages;
pub mod routing;
pub mod security;
pub mod types;

// Re-export important types for convenience
#[allow(ambiguous_glob_reexports)]
pub use config::*;
pub use deployment::DeploymentManager;
pub use discovery::DiscoveryEngine;
pub use manager::FederationManager;
pub use mcp_handler::McpFederation;
pub use messages::*;
pub use routing::RouteOptimizer;
pub use security::SecurityManager;
#[allow(ambiguous_glob_reexports)]
pub use types::*;
