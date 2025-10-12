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
//! ```rust,no_run
//! use songbird_observability::{
//!     health::{HealthMonitor, HealthCheckConfig})
//!     observability::{ObservabilityManager, MetricsConfig})
//! };
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize health monitoring
//!     let health_config = HealthCheckConfig::default();
//!     let health_monitor = HealthMonitor::new(health_config);
//!
//!     // Initialize observability manager
//!     let metrics_config = MetricsConfig::default();
//!     let observability = ObservabilityManager::new(metrics_config);
//!
//!     // Start monitoring
//!     health_monitor.start().await?;
//!     observability.start_metrics_collection().await?;
//!
//!     println!("Songbird observability system running")"
//!     Ok(()),
//! }
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

pub mod health;
pub mod observability;
pub use observability::*;
