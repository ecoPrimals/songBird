//! # 🌳 Songbird Primal Coordination - Universal Signal and Coordinator
//!
//! **MISSION**: Songbird is the nervous system, not the organs
//!
//! This crate implements capability-based primal coordination with ZERO hardcoded primal names.
//! Each primal discovers itself and others through capabilities, not hardcoded connections.
//!
//! ## Core Principles
//!
//! 1. **Zero Primal Name Hardcoding**: Never reference BearDog, Toadstool, NestGate, Squirrel
//! 2. **Capability-Based Discovery**: Request "security", "compute", "storage", "ai"
//! 3. **Self-Knowledge Only**: Each primal knows itself, discovers others
//! 4. **N to 1 Coordination**: N primals connect through Songbird, not 2^N connections
//! 5. **Agnostic Abstraction**: Like gaming system evolution
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Songbird Coordinator                     │
//! │  (Universal Signal - Knows capabilities, not primals)       │
//! └───┬─────────┬─────────┬─────────┬─────────┬───────────────┘
//!     │         │         │         │         │
//!     ▼         ▼         ▼         ▼         ▼
//! ┌───────┐ ┌───────┐ ┌───────┐ ┌───────┐ ┌───────┐
//! │Primal │ │Primal │ │Primal │ │Primal │ │Primal │
//! │   A   │ │   B   │ │   C   │ │   D   │ │   E   │
//! │(Sec)  │ │(Comp) │ │(Stor) │ │(AI)   │ │(?)    │
//! └───────┘ └───────┘ └───────┘ └───────┘ └───────┘
//! ```
//!
//! ## Example Usage
//!
//! ```rust
//! use songbird_primal_coordination::*;
//!
//! # async fn example() -> Result<()> {
//! // Create coordinator with zero knowledge
//! let mut coordinator = PrimalCoordinator::new();
//!
//! // Primals register themselves (not hardcoded)
//! // coordinator.register_primal(some_bridge).await?;
//!
//! // Request capability, get provider
//! let security_conn = coordinator.request_capability("security").await?;
//!
//! // Coordinate operation (e.g., genesis ceremony)
//! // let identity = coordinator.coordinate_genesis(node_id).await?;
//! # Ok(())
//! # }
//! ```

pub mod bridge;
pub mod coordinator;
pub mod error;
pub mod types;

// Re-exports
pub use bridge::{PrimalBridge, PrimalConnection};
pub use coordinator::PrimalCoordinator;
pub use error::{PrimalCoordinationError, Result};
pub use types::*;
