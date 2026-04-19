// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(async_fn_in_trait, reason = "discovery trait uses AFIT; callers are internal")]

//! Provider abstraction layer for discovery services (local types and [`DiscoveryProvider`] trait).
//!
//! Concrete backends and enum dispatch live in [`super::adapters`].

use std::any::Any;
use std::collections::HashMap;
use std::pin::Pin;

use futures_util::Stream;
use serde::{Deserialize, Serialize};

use crate::abstraction::capabilities::DiscoveryCapability;
use crate::traits::discovery::ServiceHealthStatus;
use crate::traits::{ServiceEvent, ServiceInfo, ServiceQuery};
use songbird_types::SongbirdResult;

/// Configuration passed when creating or initializing a discovery provider instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Instance identifier.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Provider-specific parameters (JSON values).
    pub parameters: HashMap<String, serde_json::Value>,
    /// Environment variables or string parameters.
    pub environment: HashMap<String, String>,
    /// Optional timeout in milliseconds.
    pub timeout_ms: Option<u64>,
    /// Optional retry policy blob.
    pub retry_config: Option<serde_json::Value>,
}

/// Runtime metadata cached for routing and capability matching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetadata {
    /// Unique provider id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Version string.
    pub version: String,
    /// Declared discovery capabilities.
    pub capabilities: Vec<DiscoveryCapability>,
    /// Arbitrary string metadata for operators.
    pub metadata: HashMap<String, String>,
    /// Last known health flag.
    pub healthy: bool,
    /// Relative load estimate (lower is preferred).
    pub load_score: f64,
}

/// Metrics snapshot for a single service instance (abstraction-layer shape).
#[derive(Debug, Clone)]
pub struct ServiceMetrics {
    /// Service id.
    pub service_id: String,
    /// Total requests observed.
    pub request_count: u64,
    /// Failed requests.
    pub error_count: u64,
    /// Average latency in milliseconds.
    pub average_response_time_ms: f64,
    /// CPU utilization (0–100).
    pub cpu_usage_percent: f64,
    /// Resident memory usage in bytes.
    pub memory_usage_bytes: u64,
    /// Custom numeric metrics by name.
    pub custom_metrics: HashMap<String, f64>,
}

/// Hints for clients performing load balancing across instances.
#[derive(Debug, Clone)]
pub struct LoadBalancingHints {
    /// Logical service name.
    pub service_name: String,
    /// Preferred instance ids.
    pub preferred_instances: Vec<String>,
    /// Relative weights by instance id.
    pub weights: HashMap<String, f64>,
    /// Health scores by instance id.
    pub health_scores: HashMap<String, f64>,
    /// Optional locality hints.
    pub locality_preferences: Vec<String>,
}

/// Discovery provider abstraction used by the registry and adapters (enum-backed, no trait objects).
pub trait DiscoveryProvider: Send + Sync {
    /// Provider metadata for routing and capability queries.
    fn metadata(&self) -> &ProviderMetadata;

    /// Initialize with configuration.
    async fn initialize(&mut self, config: ProviderConfig) -> SongbirdResult<()>;

    /// Graceful shutdown.
    async fn shutdown(&mut self) -> SongbirdResult<()>;

    /// Lightweight liveness probe.
    async fn health_check(&self) -> SongbirdResult<bool>;

    /// Register a service instance.
    async fn register(&self, service: ServiceInfo) -> SongbirdResult<()>;

    /// Remove a service instance.
    async fn unregister(&self, service_id: &str) -> SongbirdResult<()>;

    /// Discover instances matching `query`.
    async fn discover(&self, query: ServiceQuery) -> SongbirdResult<Vec<ServiceInfo>>;

    /// Watch for changes (may be an empty stream if unsupported).
    async fn watch(
        &self,
        query: ServiceQuery,
    ) -> SongbirdResult<Pin<Box<dyn Stream<Item = ServiceEvent> + Send>>>;

    /// Update reported health for an instance.
    async fn update_health(
        &self,
        service_id: &str,
        health: ServiceHealthStatus,
    ) -> SongbirdResult<()>;

    /// Merge metadata for an instance.
    async fn update_metadata(
        &self,
        service_id: &str,
        metadata: HashMap<String, String>,
    ) -> SongbirdResult<()>;

    /// List all known instances.
    async fn list_all(&self) -> SongbirdResult<Vec<ServiceInfo>>;

    /// Whether an instance id exists.
    async fn exists(&self, service_id: &str) -> SongbirdResult<bool>;

    /// Metrics for observability.
    async fn get_service_metrics(&self, service_id: &str) -> SongbirdResult<ServiceMetrics>;

    /// Load-balancing hints for a logical service name.
    async fn get_load_balancing_hints(
        &self,
        service_name: &str,
    ) -> SongbirdResult<LoadBalancingHints>;

    /// Type-erased downcast hook for tests and extensions.
    fn as_any(&self) -> &dyn Any;
}
