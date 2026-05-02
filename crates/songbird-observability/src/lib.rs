// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! # Songbird Observability
//!
//! Comprehensive observability and monitoring platform providing real-time insights,
//! health monitoring, and performance analytics for distributed systems.
//!
//! ## Features
//!
//! - **Real-time Monitoring**: Live system metrics and performance data
//! - **Health Checks**: Comprehensive health monitoring and alerting
//! - **Performance Analytics**: Deep performance analysis and optimization
//! - **Distributed Tracing**: Request tracing across service boundaries
//! - **Metrics Collection**: Prometheus-compatible metrics collection
//! - **Dashboard System**: Advanced web-based monitoring dashboards
//! - **Alerting**: Intelligent alerting with multiple notification channels
//! - **Log Aggregation**: Centralized log collection and analysis

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
//!
//! ## Architecture
//!
//! The observability crate is organized into focused modules:
//!
//! - `health`: Health monitoring and status reporting
//! - `observability`: Core observability infrastructure and metrics
//!
//! ## Usage
//!
//! ```rust,ignore
//! // Example usage (simplified for documentation)
//! use songbird_observability::health::HealthStatus;
//!
//! // Create and check health status
//! let status = HealthStatus::Healthy;
//! assert!(matches!(status, HealthStatus::Healthy));
//! ```
//!
//! ## Health Monitoring
//!
//! Comprehensive health monitoring capabilities:
//!
//! - **Service Health**: Individual service health status
//! - **System Health**: Overall system health and resource usage
//! - **Dependency Health**: External dependency monitoring
//! - **Custom Health Checks**: Application-specific health validations
//! - **Health Aggregation**: Hierarchical health status reporting
//!
//! ## Metrics and Analytics
//!
//! - **Performance Metrics**: Latency, throughput, and error rates
//! - **Resource Metrics**: CPU, memory, disk, and network usage
//! - **Business Metrics**: Application-specific business indicators
//! - **SLA Monitoring**: Service level agreement compliance tracking
//! - **Trend Analysis**: Historical performance trend analysis
//!
//! ## Dashboard Features
//!
//! - **Real-time Dashboards**: Live system visualization
//! - **Custom Dashboards**: User-configurable monitoring views
//! - **Alert Management**: Visual alert management and acknowledgment
//! - **Performance Insights**: AI-powered performance recommendations
//! - **Capacity Planning**: Resource usage forecasting and planning
//!
//! ## Error Handling
//!
//! All observability operations return `Result<T, SongbirdError>` with detailed
//! error information and recovery suggestions for monitoring system issues.

#![cfg_attr(
    test,
    allow(
        deprecated,
        dead_code,
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::await_holding_lock,
        clippy::float_cmp,
        clippy::absurd_extreme_comparisons,
        clippy::nonminimal_bool,
        clippy::needless_collect,
        clippy::used_underscore_binding,
        clippy::overly_complex_bool_expr,
        clippy::assertions_on_constants,
        clippy::unreadable_literal,
        clippy::empty_line_after_doc_comments,
        clippy::field_reassign_with_default,
        clippy::unnecessary_wraps,
        clippy::no_effect_underscore_binding,
        clippy::return_self_not_must_use,
        clippy::duplicated_attributes,
        clippy::needless_pass_by_value,
        clippy::must_use_candidate,
        clippy::missing_panics_doc,
        clippy::missing_errors_doc,
        clippy::doc_markdown,
        clippy::wildcard_imports,
        clippy::enum_glob_use,
        unused_imports,
        unused_variables,
        clippy::unused_self,
        clippy::unnecessary_cast,
        clippy::items_after_test_module,
        clippy::clone_on_ref_ptr,
        clippy::default_trait_access,
        clippy::needless_range_loop,
        clippy::similar_names,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::unnecessary_unwrap,
        clippy::ignore_without_reason,
        clippy::case_sensitive_file_extension_comparisons,
        reason = "test code: relaxed lints for assertions, mock construction, and test ergonomics"
    )
)]

/// Production analytics engine and related types.
pub mod analytics;
/// Aggregated health-check types and the [`health::HealthMonitor`] trait for custom probes.
pub mod health;
/// Core metrics, dashboards, and the [`observability::ObservabilityManager`] facade.
pub mod observability;
pub use observability::*;
