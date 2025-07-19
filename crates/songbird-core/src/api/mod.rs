//! REST API Layer for Songbird Orchestrator
//!
//! Provides HTTP endpoints for service management, monitoring, and system information

/// AI-optimized API module
pub mod ai_optimized;

/// BYOB API module
pub mod byob;

/// Core API functionality
pub mod core;

/// AI First Response API module
pub mod ai_first_response;

/// Universal Service Registration API module
pub mod universal_service_registration;

/// AI-Enhanced Service Mesh API module
pub mod ai_enhanced_service_mesh;

/// AI Workload Classification API module
pub mod ai_workload_classification;

/// Real-Time AI Streaming API module
pub mod real_time_ai_streaming;

// Re-export main types and functions from core
pub use core::*;

// Re-export AI-first response types
pub use ai_first_response::*;

// Re-export universal service registration types
pub use universal_service_registration::*;

// Re-export AI-enhanced service mesh types
pub use ai_enhanced_service_mesh::*;

// Re-export AI workload classification types
pub use ai_workload_classification::*;

// Re-export real-time AI streaming types
pub use real_time_ai_streaming::*;
