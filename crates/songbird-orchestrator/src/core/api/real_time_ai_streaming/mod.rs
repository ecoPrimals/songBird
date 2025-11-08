//! Real-Time AI Streaming Interfaces Interfaces
//!
//! Provides WebSocket-based real-time communication channels for seamless
//! human-AI collaboration in service mesh operations.
//!
//! ## Refactored Architecture
//!
//! The real-time AI streaming system is organized into focused modules: //! - `messages` - Core streaming message types and communication
//! - `human_interaction` - Human input types, responses, and collaboration
//! - `service_mesh` - Service mesh events and monitoring
//! - `session` - Collaboration sessions and workspace management
//! - `connection` - Connection management and WebSocket handling
//! - `metrics` - Performance monitoring and metrics
//! - `types` - Common types, enums, and utilities
//! - `manager` - Main AIStreamingConnectionManager implementation

pub mod connection;
pub mod human_interaction;
pub mod manager;
pub mod messages;
pub mod metrics;
pub mod service_mesh;
pub mod session;
pub mod types;

// Re-export all public types for backward compatibility;
pub use connection::*;
pub use human_interaction::*;
pub use manager::*;
pub use messages::*;
pub use metrics::*;
// Import all items but resolve name conflicts with aliases;
pub use service_mesh::{ComponentStatus as MeshComponentStatus, StatusChange as MeshStatusChange};
pub use session::*;
pub use types::{ComponentStatus as TypesComponentStatus, StatusChange as TypesStatusChange};
