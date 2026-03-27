// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Universal capability adapters for metrics: discover primals by capability and
//! build [`super::ComputeMetrics`] snapshots from that state.

use super::{ComputeMetrics, ComputeMetricsCounters};
use songbird_config::capability_endpoints;
use songbird_universal::capabilities::{DiscoveryConfig, UniversalCapabilityAdapter};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::atomic::Ordering;
use tracing::{info, warn};

fn default_orchestrator_url() -> &'static str {
    static URL: LazyLock<String> = LazyLock::new(|| {
        use songbird_types::constants::{DEFAULT_ORCHESTRATOR_PORT, LOCALHOST};
        songbird_process_env::var("SONGBIRD_ORCHESTRATOR_URL")
            .unwrap_or_else(|_| format!("http://{LOCALHOST}:{DEFAULT_ORCHESTRATOR_PORT}"))
    });
    URL.as_str()
}

/// Errors surfaced while resolving endpoints for metrics collection.
#[derive(Debug, Clone)]
pub enum MetricsError {
    /// Capability discovery failed with a reason.
    DiscoveryFailed(String),
    /// No endpoints were found for the requested capability.
    NoEndpointsFound(String),
    /// Network or I/O failure talking to a primal.
    NetworkError(String),
}

impl std::fmt::Display for MetricsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DiscoveryFailed(msg) => write!(f, "Discovery failed: {msg}"),
            Self::NoEndpointsFound(cap) => write!(f, "No endpoints found for capability: {cap}"),
            Self::NetworkError(msg) => write!(f, "Network error: {msg}"),
        }
    }
}

impl std::error::Error for MetricsError {}

/// Capability-based metrics adapter: discovers primals without hardcoding names.
#[derive(Debug, Clone)]
pub struct UniversalMetricsAdapter {
    capability_adapter: UniversalCapabilityAdapter,
    pub compute_endpoints: Vec<String>,
    pub security_endpoints: Vec<String>,
    pub storage_endpoints: Vec<String>,
    pub ai_endpoints: Vec<String>,
    pub custom_endpoints: HashMap<String, Vec<String>>,
    pub last_updated: Option<chrono::DateTime<chrono::Utc>>,
    /// Shared counters updated when snapshots are collected.
    pub metrics_counters: Arc<ComputeMetricsCounters>,
}

impl Default for UniversalMetricsAdapter {
    fn default() -> Self {
        Self {
            capability_adapter: UniversalCapabilityAdapter::new(DiscoveryConfig::default()),
            compute_endpoints: Vec::new(),
            security_endpoints: Vec::new(),
            storage_endpoints: Vec::new(),
            ai_endpoints: Vec::new(),
            custom_endpoints: HashMap::new(),
            last_updated: None,
            metrics_counters: Arc::new(ComputeMetricsCounters::new()),
        }
    }
}

impl UniversalMetricsAdapter {
    /// Creates an adapter with default discovery configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Total discovered HTTP endpoints across built-in capability buckets.
    #[must_use]
    pub fn total_endpoints(&self) -> usize {
        self.compute_endpoints.len()
            + self.security_endpoints.len()
            + self.storage_endpoints.len()
            + self.ai_endpoints.len()
            + self.custom_endpoints.values().map(std::vec::Vec::len).sum::<usize>()
    }

    /// Build a [`ComputeMetrics`] snapshot from current discovery state and counters.
    #[must_use]
    pub fn snapshot_compute_metrics(&self) -> ComputeMetrics {
        let total = self.total_endpoints();
        let collections = self.metrics_counters.collections_total.load(Ordering::Relaxed);
        let queued_hint = self.metrics_counters.queued_jobs_hint.load(Ordering::Relaxed);
        let zc_ops = self.metrics_counters.zero_copy_ops_observed.load(Ordering::Relaxed);

        // Heuristic load from discovered footprint (bounded, deterministic).
        let footprint = (total as f64) * 3.5;
        let cpu_usage_percent = footprint.clamp(5.0_f64, 95.0_f64);
        let cpu_fraction = (cpu_usage_percent / 100.0).clamp(0.0, 1.0);

        let mut m = ComputeMetrics::default();
        m.cpu_usage_percent = cpu_usage_percent;
        m.cpu_usage = cpu_fraction;
        m.load_average = (total as f64 * 0.12).min(16.0);
        m.memory_usage = (cpu_fraction * 0.85).min(1.0);
        m.active_containers = total.min(u32::MAX as usize) as u32;
        m.queued_jobs = (queued_hint as u32).saturating_add((collections % 64) as u32);
        m.zero_copy_operations_per_sec = zc_ops.max(1);
        m.metric_name = format!("songbird.compute.endpoints={total}");
        m.timestamp = chrono::Utc::now();
        m
    }

    /// Discover and refresh endpoint lists from the capability registry.
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn discover_and_update_endpoints(&mut self) -> Result<(), MetricsError> {
        info!("Discovering primals for metrics collection");

        self.compute_endpoints = self.discover_primals_with_capability("compute").await;
        info!("Found {} compute primals", self.compute_endpoints.len());

        self.security_endpoints = self.discover_primals_with_capability("security").await;
        info!("Found {} security primals", self.security_endpoints.len());

        self.storage_endpoints = self.discover_primals_with_capability("storage").await;
        info!("Found {} storage primals", self.storage_endpoints.len());

        self.ai_endpoints = self.discover_primals_with_capability("ai").await;
        info!("Found {} AI primals", self.ai_endpoints.len());

        let hint = self.compute_endpoints.len().saturating_add(self.ai_endpoints.len()) as u64;
        self.metrics_counters.queued_jobs_hint.store(hint, Ordering::Relaxed);

        self.last_updated = Some(chrono::Utc::now());
        Ok(())
    }

    async fn discover_primals_with_capability(&self, capability: &str) -> Vec<String> {
        let mut providers = self.capability_adapter.find_capability_providers(capability).await;

        if providers.is_empty() {
            for alt in ["ml", "intelligence", "model"] {
                let extra = self.capability_adapter.find_capability_providers(alt).await;
                providers.extend(extra);
            }
        }

        let mut endpoints = Vec::new();
        for _ in &providers {
            let endpoint = capability_endpoints::get_capability_endpoint(capability)
                .await
                .unwrap_or_else(|_| default_orchestrator_url().to_string());
            endpoints.push(endpoint);
        }

        if endpoints.is_empty() {
            endpoints = self.discover_capability_fallback(capability).await;
        }

        endpoints
    }

    #[expect(
        clippy::unused_async,
        reason = "async signature required by Axum, trait objects, or future I/O"
    )]
    async fn discover_capability_fallback(&self, capability: &str) -> Vec<String> {
        let mut endpoints = Vec::new();

        match capability {
            "compute" => {
                if let Ok(endpoint) = songbird_process_env::var("COMPUTE_ENDPOINT") {
                    endpoints.push(endpoint);
                } else if let Ok(endpoint) = songbird_process_env::var("compute_endpoint") {
                    endpoints.push(endpoint);
                }
            }
            "security" => {
                if let Ok(endpoint) = songbird_process_env::var("SECURITY_ENDPOINT") {
                    endpoints.push(endpoint);
                } else if let Ok(endpoint) = songbird_process_env::var("security_endpoint") {
                    endpoints.push(endpoint);
                }
            }
            "storage" => {
                if let Ok(endpoint) = songbird_process_env::var("STORAGE_ENDPOINT") {
                    endpoints.push(endpoint);
                } else if let Ok(endpoint) = songbird_process_env::var("storage_endpoint") {
                    endpoints.push(endpoint);
                }
            }
            "ai" => {
                if let Ok(endpoint) = songbird_process_env::var("AI_ENDPOINT") {
                    endpoints.push(endpoint);
                } else if let Ok(endpoint) = songbird_process_env::var("ai_endpoint") {
                    endpoints.push(endpoint);
                }
            }
            _ => {
                let env_var = format!("{}_ENDPOINT", capability.to_uppercase());
                if let Ok(endpoint) = songbird_process_env::var(&env_var) {
                    endpoints.push(endpoint);
                }
            }
        }

        if endpoints.is_empty() {
            warn!("No env fallback endpoints for capability {capability}");
        }

        endpoints
    }

    /// Borrow the endpoint list for a well-known or custom capability key.
    #[must_use]
    pub fn get_endpoints_for_capability(&self, capability: &str) -> &[String] {
        match capability {
            "compute" => &self.compute_endpoints,
            "security" => &self.security_endpoints,
            "storage" => &self.storage_endpoints,
            "ai" => &self.ai_endpoints,
            _ => self.custom_endpoints.get(capability).map_or(&[] as &[String], Vec::as_slice),
        }
    }

    /// Returns true if at least one endpoint exists for the capability.
    #[must_use]
    pub fn has_capability(&self, capability: &str) -> bool {
        !self.get_endpoints_for_capability(capability).is_empty()
    }

    /// First endpoint for the capability, if any.
    #[must_use]
    pub fn get_primary_endpoint_for_capability(&self, capability: &str) -> Option<&String> {
        self.get_endpoints_for_capability(capability).first()
    }
}
