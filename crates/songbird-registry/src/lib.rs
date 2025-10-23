//! Songbird Registry
//!
//! Plugin registry and management system for Songbird.
//!
//! # Features
//! - Plugin registration and discovery
//! - Health monitoring
//! - Auto-scaling
//! - Event streaming
//!
//! # Example
//! ```no_run
//! use songbird_registry::{Registry, Plugin, PluginRegistry};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut registry = Registry::new();
//!     
//!     let plugin = Plugin::new("my-plugin", "My Plugin", "1.0.0");
//!     registry.register(plugin).await?;
//!     
//!     let plugins = registry.list().await;
//!     println!("Registered {} plugins", plugins.len());
//!     
//!     Ok(())
//! }
//! ```

#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]

// Core modules
pub mod registry;
pub mod types;

// Health and scaling modules (new clean implementations)
pub mod health_new;
pub mod scaling_new;

// Re-export commonly used types
pub use types::{
    Capability, CapabilityType, EventType, HealthCheckConfig, HealthCheckType, HealthStatus,
    Plugin, PluginId, PluginMetadata, RegistryEvent,
};

pub use registry::{Composable, PluginRegistry, Query, Registry};

// Note: Old modules (health, scaling, plugin, service) are temporarily disabled
// during the rebuild. They will be replaced with modern implementations.

// Note: Service module functionality integrated into main registry module
// pub mod service;
// pub use service::{ServiceInstance, ServiceMetrics};

// Production modules remain for reference
// pub mod production;
// pub mod persistence;
