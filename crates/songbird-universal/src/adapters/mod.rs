//! Capability-Based Adapters for Universal Orchestration
//!
//! **SOVEREIGNTY PRINCIPLE**: These adapters work with capabilities, NOT primal names.
//! Each primal only knows itself. Songbird discovers capabilities dynamically.
//!
//! ## Design Philosophy
//!
//! Like in ecology, each organism exists independently:
//! - Songbird doesn't "know" specific primals exist (`BearDog`, `NestGate`, etc.)
//! - Songbird only knows "something provides security capability"
//! - Discovery is dynamic through `ZeroKnowledgeBootstrap`
//! - No 2^n hardcoded connections - only universal adapter for network effects
//!
//! ## Capability-Based Adapters
//!
//! - `ComputeAdapter` - Any compute capability provider
//! - `SecurityAdapter` - Any security capability provider
//! - `StorageAdapter` - Any storage capability provider  
//! - `AIAdapter` - Any AI capability provider
//!
//! ## Example Implementations
//!
//! See `examples/integration/ecosystem-primals/` for how specific primals
//! in our ecosystem happen to implement these capabilities. But the production
//! code here doesn't know about them!

pub mod ai;
pub mod compute;
pub mod security;
pub mod storage;

pub use ai::AIAdapter;
pub use compute::ComputeAdapter;
pub use security::SecurityAdapter;
pub use storage::StorageAdapter;
