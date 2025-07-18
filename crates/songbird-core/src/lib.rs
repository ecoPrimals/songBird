//! Songbird Core - Universal Orchestration Engine
//!
//! This crate provides the core orchestration functionality for the Songbird Universal
//! Orchestrator platform. It includes load balancing, service registry, scaling,
//! benchmarking, and zero-touch deployment capabilities.
//!
//! ## Features
//!
//! - **Load Balancing**: Advanced load balancing algorithms and health-aware routing
//! - **Service Registry**: Dynamic service discovery and registration
//! - **Auto-scaling**: Intelligent scaling based on performance metrics
//! - **Benchmarking**: Built-in performance testing and optimization
//! - **Robustness**: Circuit breakers, retry logic, and fault tolerance
//! - **Zero-touch Deployment**: Automated deployment and configuration
//!
//! ## Architecture
//!
//! The core system is built around several key components:
//!
//! 1. **Load Balancer**: Distributes traffic across service instances
//! 2. **Service Registry**: Manages service discovery and health monitoring
//! 3. **Orchestrator**: Coordinates service interactions and scaling
//! 4. **Benchmarking Engine**: Provides performance testing capabilities
//! 5. **Robustness Layer**: Implements fault tolerance patterns
//!
//! ## Usage
//!
//! ```rust
//! use songbird_core::orchestrator::Orchestrator;
//! use songbird_core::load_balancer::LoadBalancer;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let orchestrator = Orchestrator::new().await?;
//!     let load_balancer = LoadBalancer::new().await?;
//!     
//!     // Start orchestration
//!     orchestrator.start().await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Load Balancing
//!
//! The load balancer supports multiple algorithms:
//!
//! - Round Robin
//! - Least Connections
//! - Weighted Round Robin
//! - Health-aware routing
//! - Geographic routing
//!
//! ## Scaling
//!
//! Auto-scaling features include:
//!
//! - CPU and memory-based scaling
//! - Custom metric scaling
//! - Predictive scaling
//! - Cost-aware scaling
//!
//! ## Zero-touch Deployment
//!
//! Automated deployment capabilities:
//!
//! - Environment detection
//! - Configuration management
//! - Service discovery
//! - Health monitoring
//! - Rollback capabilities

#![allow(dead_code)]

// pub mod api; // Temporarily commented out due to missing axum dependency
pub mod benchmarks;
pub mod biome;
pub mod biomeos_integration;
pub mod load_balancer;
pub mod orchestrator;
pub mod performance_optimizer;
pub mod production_benchmarks;
pub mod registry;
pub mod robustness;
pub mod scalability;
pub mod structural_improvements;
pub mod substrate;
pub mod traits;
pub mod zero_touch;
