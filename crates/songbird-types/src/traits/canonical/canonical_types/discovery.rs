// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Discovery criteria, queries, and registry events.

use super::classification::ServiceType;
use super::health::HealthStatus;
use super::service::ServiceInfo;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Filter services returned by discovery queries.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiscoveryCriteria {
    /// Match on service name substring or exact id.
    pub name: Option<String>,
    /// Restrict to a service class.
    pub service_type: Option<ServiceType>,
    /// Require a minimum semantic version.
    pub version: Option<String>,
    /// Match if any tag overlaps.
    pub tags: Vec<String>,
    /// Require all listed capability names.
    pub capabilities: Vec<String>,
    /// Structured filters for advanced matchers.
    pub metadata: HashMap<String, serde_json::Value>,
    /// Restrict to instances at a given health level.
    pub health_status: Option<HealthStatus>,
    /// Cap the number of results.
    pub limit: Option<usize>,
}

/// Configure long-poll or watch-style discovery calls.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiscoveryQuery {
    /// Filters to apply.
    pub criteria: DiscoveryCriteria,
    /// Wait for subsequent changes instead of a one-shot list.
    pub watch_changes: bool,
    /// Include full metadata blobs in results.
    pub include_metadata: bool,
}

/// Notify subscribers when registry membership or health changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceEvent {
    /// A new service instance appeared.
    Registered(ServiceInfo),
    /// An existing instance changed materially.
    Updated(ServiceInfo),
    /// A service left the registry.
    Unregistered {
        /// Id of the removed service.
        service_id: String,
    },
    /// Health status changed for a known instance.
    HealthChanged {
        /// Affected service id.
        service_id: String,
        /// New health value.
        health: HealthStatus,
    },
}
