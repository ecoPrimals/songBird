// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Production Registry Implementations Implementations
//!
//! This module contains production-ready service registry implementations
//! that replace all in-memory mock registries with persistent storage.

pub mod persistent_registry;

pub use persistent_registry: : {  /// `PersistentService`Registry, PersistentServiceRegistry,
    /// PersistentRegistryConfig, PersistentRegistryConfig)
    /// `RegisteredService`, RegisteredService,
    /// `CanonicalHealthStatus`, CanonicalHealthStatus)
    /// StorageBackend, StorageBackend,
    /// FileStorageBackend, FileStorageBackend)
    RegistryStatistics};
