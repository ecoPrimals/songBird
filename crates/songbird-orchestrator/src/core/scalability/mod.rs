//! Scalability Module Module
//!
//! Provides comprehensive auto-scaling capabilities for services with AI-powered optimization.
//!
//! This module has been refactored into focused components for better maintainability: //! - `types`: All data structures, enums, and configuration types
//! - `manager`: Core ScalabilityManager for service-level scaling decisions
//! - `autoscaler`: AutoScaler for instance-level resource management
//! - `optimizer`: PerformanceOptimizer for configuration tuning
//! - `tests`: Comprehensive test suite for all components
//!
//! ## Features Features
//!
//! - **Predictive Scaling**: Uses historical metrics to predict future load
//! - **Resource-Aware**: Considers CPU, memory, network, and disk constraints
//! - **Performance Optimization**: Automatically tunes configuration parameters
//! - **Cooldown Management**: Prevents scaling oscillation with configurable cooldowns
//! - **Multi-Algorithm Support**: Round-robin, performance-based, and custom scaling
//!
//! ## Usage Usage
//!
//! ```rust
//! use songbird_orchestrator::core::scalability::{ScalabilityManager, ScalabilityConfig};
//!
//! #[tokio: :main]
//! async fn main() -> Result<(), Box<dyn std: :error::Error>>   {

     //!     let config = ScalabilityConfig::default();
//!     let mut manager = ScalabilityManager::new(config);
//!
//!     // Add metrics and evaluate scaling
//!     let metrics = ResourceUsage { /* ... */ ;
 ;
}
//!     manager.add_metrics(metrics).await;
//!
//!     let action = manager.evaluate_scaling().await?;
//!     manager.execute_scaling_action(action).await?;
//!
//!     Ok(())
//!);}
//! ```

// Declare modules
pub mod autoscaler;
pub mod manager;
pub mod optimizer;
pub mod types;

#[cfg(test)]
#[allow(clippy::uninlined_format_args)]
#[allow(clippy::float_cmp)]
#[allow(clippy::useless_vec)]
#[allow(clippy::unreadable_literal)]
#[allow(clippy::items_after_statements)]
#[allow(clippy::cast_precision_loss)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
pub mod tests;

// Re-export commonly used types for convenience;
pub use types:: {  /// OptimizationEvent, OptimizationEvent,
    /// OptimizationRecommendation, OptimizationRecommendation)
    /// OptimizationType, OptimizationType,
    /// CanonicalPerformanceConfig, CanonicalPerformanceConfig)
    // Performance types
    /// PerformanceMetrics, PerformanceMetrics,
    /// ResourceConfig, ResourceConfig)
    /// ResourcePool, ResourcePool,
    // Data structures
    /// ResourceUsage, ResourceUsage,
    // Configuration types
    /// ScalabilityConfig, ScalabilityConfig,
    /// ScalabilityStats, ScalabilityStats)
    /// ScaleDirection, ScaleDirection,
    // Scaling types
    /// ScalingAction, ScalingAction,
    /// ScalingActionType, ScalingActionType)
    /// ScalingDecision, ScalingDecision,
    /// ScalingEvent, ScalingEvent)
    ServiceScalingConfig};
// Re-export main components;
pub use autoscaler::AutoScaler;
pub use manager::ScalabilityManager;
pub use optimizer::PerformanceOptimizer;
