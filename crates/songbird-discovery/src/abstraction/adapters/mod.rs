// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! # Legacy Backend Adapters
//!
//! This module provides adapters that wrap the existing hardcoded Consul and Kubernetes
//! backends to work with the new agnostic provider system. This enables gradual migration
//! without breaking existing functionality.

pub mod consul_adapter;
pub mod kubernetes_adapter;
pub mod static_adapter;

// Re-export adapters
pub use consul_adapter::{ConsulProviderAdapter, ConsulProviderFactory};
pub use kubernetes_adapter::{KubernetesProviderAdapter, KubernetesProviderFactory};
pub use static_adapter::{StaticProviderAdapter, StaticProviderFactory};
