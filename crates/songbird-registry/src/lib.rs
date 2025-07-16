//! Dynamic Service Registry for Songbird Universal Orchestrator
//!
//! This crate provides a comprehensive service registry system with health monitoring,
//! plugin management, and dynamic service discovery capabilities.
//!
//! ## Features
//!
//! - **Dynamic Service Registration**: Register and discover services dynamically
//! - **Health Monitoring**: Continuous health checks and monitoring
//! - **Plugin Management**: Dynamic plugin loading and management
//! - **Service Metadata**: Rich metadata support for services
//! - **Performance Metrics**: Built-in performance tracking
//! - **Event System**: Comprehensive event handling for service lifecycle
//!
//! ## Architecture
//!
//! The registry uses a multi-layered architecture:
//!
//! 1. **Service Layer**: Core service registration and discovery
//! 2. **Health Layer**: Health monitoring and alerting
//! 3. **Plugin Layer**: Dynamic plugin loading and management
//! 4. **Event Layer**: Event handling and notification
//!
//! ## Usage
//!
//! ```rust
//! use songbird_registry::service::ServiceRegistry;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let registry = ServiceRegistry::new().await?;
//!     
//!     // Register a service
//!     registry.register_service("my-service", "http://localhost:8080").await?;
//!     
//!     // Discover services
//!     let services = registry.discover_services().await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Service Management
//!
//! The registry provides comprehensive service management:
//!
//! - Service registration and deregistration
//! - Health check management
//! - Service metadata management
//! - Performance monitoring
//! - Event handling
//!
//! ## Health Monitoring
//!
//! Built-in health monitoring provides:
//!
//! - Continuous health checks
//! - Health status tracking
//! - Alert thresholds
//! - Health event notifications
//!
//! ## Plugin System
//!
//! The dynamic plugin system supports:
//!
//! - Runtime plugin loading
//! - Plugin dependency management
//! - Plugin hooks and events
//! - Plugin metadata and requirements
//!
//! ## Error Handling
//!
//! All registry operations return `Result<T, SongbirdError>` with detailed
//! error information including service registration failures, health check
//! errors, and plugin loading issues.

#![allow(dead_code)]

pub mod health;
pub mod plugin;
pub mod scaling;
pub mod service;

pub use health::*;
pub use plugin::*;
// Use specific imports to avoid ambiguous re-exports
pub use scaling::{
    AutoScalingEngine, AutoScalingPolicy, ScalingDecision, ScalingDirection, ScalingState,
    ScalingStrategy, ScalingThreshold,
};
pub use service::{
    DynUniversalService, ServiceEntry, ServiceEvent, ServiceHandle, ServiceHealthStatus,
    ServiceLifecycleState, ServiceMetrics, ServiceRegistry,
};

// Re-export traits from discovery for convenience
pub use songbird_discovery::traits::PluginRegistry;
