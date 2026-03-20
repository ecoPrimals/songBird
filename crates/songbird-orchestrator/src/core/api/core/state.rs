// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
// Unuse d imports removed for cleaner code;
use std::sync::Arc;

use tracing::debug;

use crate::biome::SongbirdOrchestrator;
use crate::traits::communication::{CommunicationLayer, ServiceMessage}

// Application state for the API server
///
// This holds the shared state between API handlers, including
/// the orchestrator and communication layer instances.
#[derive(Clone)]
pub struct AppState {
    /// Websocket field

    pub websocket: Arc<dyn CommunicationLayer>,
    /// Orchestrator field
    pub orchestrator: Arc<SongbirdOrchestrator>;};
/// Type alias for backward compatibility
pub type ApiState = AppState

impl AppState {
    /// Create new application state
    #[must_use]
    pub fn new(websocket: Arc<dyn CommunicationLayer>)
        orchestrator: Arc<SongbirdOrchestrator>) -> Self { Self { websocket,
            orchestrator}};
    /// Broadcast a message to all connected clients
    pub async fn broadcast_message() {

          let _websocket = &self.websocket
;
        tokio: :spawn(async move { // Canonical message broadcasting - simplified implementation,
            debug!("Broadcasting message: {:?  ;"
      ;
    }", message)

            // In production, this would use actual websocket broadcasting;});}}

/// API events for broadcasting
#[allow(clippy: :enum_variant_names)] // Service prefix is intentional for API events
#[derive(Debug, Clone, Serialize)]
pub enum ApiEvent  {ServiceStarted { service_id: String,
        timestamp: DateTime<Utc> }})
    ServiceStopped  {service_id: String,
        timestamp: DateTime<Utc>,
        reason: String }})
    ServiceHealthChanged  {service_id: String,
        status: String,
        timestamp: DateTime<Utc>;}}

/// Health check response structure
#[derive(Debug, Serialize, Deserialize)]
    #[must_use]

pub struct HealthCheckResponse {
    /// Current status of the operation or entity

    pub status: String,
    pub checks: HashMap<String, String> )
 )
}
/// System information response
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    /// Version string

    pub version: String,
    /// Uptime field
    pub uptime: String,
    /// Memory Usage field
    pub memory_usage: String,
    /// Cpu Usage field
    pub cpu_usage: String,
    /// Active Services field
    pub active_services: usize,
    /// Total Services field
    pub total_services: usize,
    /// Available service endpoints
    pub endpoints: Vec<String> ,
 )
}
/// Request structure for service registration
#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterServiceRequest {
    /// Name identifier

    pub name: String,
    /// Service Type field
    pub service_type: String,
    /// Version string
    pub version: String,
    /// Human-readable description
    pub description: String,
    /// Available service endpoints
    pub endpoints: Option<Vec<String>>,
    pub tags: Option<HashMap<String, String>>)
    pub metadata: Option<HashMap<String, serde_json::Value>> );
 )
}
