// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Universal Container Orchestration Adapter
//!
//! Provides vendor-agnostic container orchestration discovery that can work with:
//! - Any Kubernetes-compatible system (K8s, K3s, `OpenShift`, etc.)
//! - Any Docker-compatible system (Docker, Podman, containerd, etc.)
//! - Any container runtime environment
//! - Any orchestration API that provides service information
//!
//! ## Native Async Traits
//! This module uses native async trait methods (Rust 1.75+) for zero-cost abstractions.

#![allow(
    async_fn_in_trait,
    clippy::unused_async,
    clippy::struct_field_names,
    clippy::missing_errors_doc,
    clippy::used_underscore_binding,
    clippy::unused_self,
    reason = "async discovery traits: native async traits and adapter ergonomics"
)]

mod adapter;
mod discovery;
mod docker;
mod environment;
mod kubernetes;
mod trait_impl;
mod types;

#[cfg(test)]
mod tests;

pub use types::{
    ApiEndpoint, AuthenticationMethod, ContainerInfo, NamespaceConfig,
    UniversalContainerOrchestration,
};
