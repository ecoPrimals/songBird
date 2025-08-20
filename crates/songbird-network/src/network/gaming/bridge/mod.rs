pub mod manager;
pub mod packet_forwarder;
pub mod socket_pool;
/// Gaming Bridge Module
///
/// Provides intelligent modularization of gaming bridge functionality
///
/// ## Architecture
/// - `types`: Core data structures and configuration types
/// - `socket_pool`: Socket allocation and network resource management
/// - `packet_forwarder`: Packet forwarding and processing logic
/// - `manager`: High-level bridge management and orchestration
///
/// This module demonstrates smart refactoring by splitting a 933-line monolithic file
/// into focused, single-responsibility modules for better maintainability.
// Submodules organized by responsibility
pub mod types;

// Re-exports for backward compatibility and convenience
pub use manager::*;
pub use packet_forwarder::*;
pub use socket_pool::*;
pub use types::*;
