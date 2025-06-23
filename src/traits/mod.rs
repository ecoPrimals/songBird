//! Core traits for the Songbird Orchestrator
//!
//! This module defines the fundamental traits that enable universal service orchestration
//! across different project types and deployment environments.

pub mod communication;
pub mod config;
pub mod discovery;
pub mod health;
pub mod load_balancer;
pub mod service;
pub mod observability;
pub mod resource_management;
pub mod hooks;
pub mod feature_flags;
pub mod validation;

// Re-export all trait types
pub use communication::CommunicationLayer;
pub use config::ConfigProvider;
pub use discovery::{ServiceDiscovery, ServiceEvent, ServiceQuery};
pub use health::{HealthCheck, HealthMonitor, HealthStatus};
pub use load_balancer::LoadBalancer;
pub use service::*;

// Re-export new trait modules
pub use observability::*;
pub use resource_management::*;
pub use hooks::*;
pub use feature_flags::*;
pub use validation::*;

// Re-export from the main health module
pub use crate::health::HealthState;
