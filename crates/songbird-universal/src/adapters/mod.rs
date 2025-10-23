//! Primal Adapters for Songbird Universal Orchestrator
//!
//! This module provides capability-based adapters for ingesting metrics and
//! coordinating with ecosystem primals without hardcoded names.
//!
//! ## Design Principles
//!
//! 1. **Capability-Based**: Adapters work with capabilities, not primal names
//! 2. **Graceful Degradation**: Handle failures without cascading
//! 3. **Observability**: All operations are traced and measurable
//! 4. **Zero Hardcoding**: No hardcoded IPs, ports, or primal names
//!
//! ## Available Adapters
//!
//! - `ToadStoolMetricsAdapter` - Compute metrics ingestion
//! - `BearDogSecurityAdapter` - Security and auth coordination (planned)
//! - `NestGateStorageAdapter` - Storage metrics and coordination (planned)
//! - `SquirrelAIAdapter` - AI/MCP integration (planned)

pub mod beardog;
pub mod nestgate;
pub mod squirrel;
pub mod toadstool;

pub use beardog::BearDogSecurityAdapter;
pub use nestgate::NestGateStorageAdapter;
pub use squirrel::SquirrelAIAdapter;
pub use toadstool::ToadStoolMetricsAdapter;
