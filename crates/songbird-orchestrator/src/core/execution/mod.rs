// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Remote execution orchestration module
//!
//! This module provides capabilities to execute commands on remote federated towers,
//! manage distributed job execution, and coordinate multi-tower operations.

pub mod broadcast;
pub mod client;
pub mod manager;

pub use broadcast::{BroadcastExecutor, BroadcastOptions};
pub use client::ExecutionClient;
pub use manager::ExecutionManager;
