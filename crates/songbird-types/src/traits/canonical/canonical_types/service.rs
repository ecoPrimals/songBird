// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Service requests, responses, metrics, and endpoints.

use super::classification::ServiceType;
use super::health::HealthStatus;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

/// Carry one inbound service invocation across provider boundaries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRequest {
    /// Correlation id for logs and tracing.
    pub id: String,
    /// HTTP-style method or RPC verb.
    pub method: String,
    /// Resource path or RPC route.
    pub path: String,
    /// Transport headers (including auth).
    pub headers: HashMap<String, String>,
    /// JSON payload body.
    pub body: serde_json::Value,
    /// When the request was accepted.
    pub timestamp: SystemTime,
}

/// Return status, headers, and body for a service invocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceResponse {
    /// Matches the request correlation id.
    pub id: String,
    /// HTTP-style status code.
    pub status_code: u16,
    /// Outbound headers.
    pub headers: HashMap<String, String>,
    /// JSON response body.
    pub body: serde_json::Value,
    /// When the response was produced.
    pub timestamp: SystemTime,
}

/// Aggregate runtime stats for a single service instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetrics {
    /// Total handled requests since start.
    pub request_count: u64,
    /// Total failed requests since start.
    pub error_count: u64,
    /// Rolling average latency in milliseconds.
    pub average_response_time_ms: f64,
    /// Seconds the instance has been running.
    pub uptime_seconds: u64,
    /// Resident memory usage in megabytes.
    pub memory_usage_mb: f64,
    /// Recent CPU utilization percentage.
    pub cpu_usage_percent: f64,
}

/// Identify and locate a registered service for discovery clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// Unique service id in the registry.
    pub id: String,
    /// Human-readable display name.
    pub name: String,
    /// Classify the service role.
    pub service_type: ServiceType,
    /// Deployed version string.
    pub version: String,
    /// Reachable network endpoints.
    pub endpoints: Vec<Endpoint>,
    /// Latest health signal.
    pub health: HealthStatus,
    /// Arbitrary key/value metadata for UIs and routing.
    pub metadata: HashMap<String, String>,
    /// Optional grouping or cost tags.
    pub tags: Vec<String>,
    /// Capability names this instance implements.
    pub capabilities: Vec<String>,
    /// When this record was last refreshed.
    pub last_updated: SystemTime,
}

/// Describe how to dial a single network endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoint {
    /// Scheme or transport label (for example `https`).
    pub protocol: String,
    /// Hostname or IP.
    pub host: String,
    /// TCP or UDP port.
    pub port: u16,
    /// Optional HTTP path or RPC subpath.
    pub path: Option<String>,
    /// Extra routing hints (TLS SNI, region, etc.).
    pub metadata: HashMap<String, String>,
}
