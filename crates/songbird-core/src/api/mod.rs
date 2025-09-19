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

// Import modules with conflict resolution
pub use crate::api::core::{CoreApiConfig, CoreApiHandler, CoreApiRequest, CoreApiResponse};

pub use ai_first_response::ResourceUsage as AiResourceUsage;

pub use universal_service_registration::types::{
    AlertThresholds as UniversalAlertThresholds,
    CircuitBreakerConfig as UniversalCircuitBreakerConfig,
};

pub use ai_enhanced_service_mesh::*;

// Re-export WorkloadCircuitBreakerConfig from robustness module instead of AI classification
pub use crate::robustness::CircuitBreakerConfig as WorkloadCircuitBreakerConfig;

pub use real_time_ai_streaming::{
    types as streaming_types, AlertThresholds as StreamingAlertThresholds,
};
