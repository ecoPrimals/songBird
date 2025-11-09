//! Remote execution orchestration module
//! 
//! This module provides capabilities to execute commands on remote federated towers,
//! manage distributed job execution, and coordinate multi-tower operations.

pub mod client;
pub mod broadcast;
pub mod manager;

pub use client::ExecutionClient;
pub use broadcast::{BroadcastExecutor, BroadcastOptions};
pub use manager::ExecutionManager;

