// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use crate::error::IpcResult;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

use super::types::HttpResponse;

/// Trait for HTTP client capability
///
/// This abstraction allows for:
/// - Multiple implementations (Pure Rust, mocked, etc.)
/// - Easy testing via dependency injection
/// - Future evolution (HTTP/2, HTTP/3, etc.)
#[async_trait]
pub trait HttpClientCapability: Send + Sync {
    /// Execute HTTP request
    async fn request(
        &self,
        method: &str,
        url: &str,
        headers: &HashMap<String, String>,
        body: Option<&[u8]>,
    ) -> IpcResult<HttpResponse>;
}

/// Trait for discovering crypto capabilities at runtime
///
/// This enables:
/// - No hardcoded `BearDog` endpoints
/// - Capability-based discovery
/// - Multiple discovery backends (env, IPC, mDNS)
#[async_trait]
pub trait CryptoCapabilityDiscovery: Send + Sync {
    /// Discover crypto provider by capability
    async fn discover(&self, capability: &str) -> IpcResult<String>;
}

/// Factory for creating HTTP clients
///
/// Benefits:
/// - Dependency injection
/// - Easy mocking in tests
/// - Centralized client configuration
#[async_trait]
pub trait HttpClientFactory: Send + Sync {
    /// Create HTTP client with automatic crypto provider discovery
    async fn create_client(&self) -> IpcResult<Arc<dyn HttpClientCapability>>;
}
