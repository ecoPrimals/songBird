// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Discovery Backends
//!
//! EVOLVED: Modular discovery backends for different environments

pub mod container;
pub mod environment;
pub mod network;

// Re-export for convenience
pub use container::discover_from_containers;
pub use environment::discover_from_environment;
pub use network::discover_from_network;
// pub use network::discover_from_network_scan; // Removed - function doesn't exist
