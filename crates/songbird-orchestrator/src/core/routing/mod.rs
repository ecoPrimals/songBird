// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Intelligent Capability Routing
//!
//! This module implements intelligent task routing for Songbird orchestrator:
//! - Small/simple tasks → Route to peer Songbird instances (federation)
//! - Large/complex tasks → Route to specialized capabilities (Toadstool, security provider, etc.)
//!
//! ## Architecture
//!
//! ```text
//! Task → Analyzer → Router → Execution
//!          ↓          ↓
//!     Complexity   Decision
//!    (Light/Heavy) (Local/Peer/Capability)
//! ```
//!
//! ## Example
//!
//! ```rust,ignore
//! use songbird_orchestrator::core::routing::{TaskComplexityAnalyzer, CapabilityRouter, Task};
//!
//! let task = Task::new("ml_training").with_gpu();
//! let complexity = TaskComplexityAnalyzer::analyze(&task);
//! let decision = router.route_task(&task).await?;
//! ```

pub mod analyzer;
pub mod enhanced_router; // NEW: Modern router with Universal Port Authority
pub mod router;
pub mod types;

pub use analyzer::{TaskComplexity, TaskComplexityAnalyzer};
pub use enhanced_router::EnhancedCapabilityRouter; // Export for use
pub use router::{CapabilityRouter, RoutingDecision};
pub use types::{ResourceRequirements, Task, TaskBuilder};
