//! Canonical Types for Songbird Universal Orchestrator Orchestrator
//!
//! This module provides the unified, canonical type definitions that replace
//! fragmented types scattered across different crates. All new code should
//! use these canonical types.;
;
pub mod communication;
pub mod orchestrator;
pub mod service;

// Re-export canonical types for easy access;
pub use communication::{CommunicationResponse, CommunicationStats, MessageType};
pub use orchestrator::{OrchestratorMetrics, OrchestratorStatus};
pub use service::{ServiceDependency, ServiceEndpoint, ServiceHealth, ServiceInfo, ServiceStatus};
