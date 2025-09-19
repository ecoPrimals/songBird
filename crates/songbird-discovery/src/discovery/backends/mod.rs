//! Universal capability-based discovery adapters
//!
//! This module provides vendor-agnostic discovery adapters that work with any
//! service discovery system through capability detection and universal interfaces.

pub mod container_orchestration;
pub mod service_discovery;
pub mod static_discovery;

// Re-export universal adapters
pub use container_orchestration::UniversalContainerOrchestration;
pub use service_discovery::UniversalServiceDiscovery;
pub use static_discovery::StaticServiceDiscovery;
