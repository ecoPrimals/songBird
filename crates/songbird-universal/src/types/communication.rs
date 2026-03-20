// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Inter-service communication primitives
//!
//! This module provides types for requests, responses, events,
//! and protocol characteristics in the universal adapter system.

use super::capability::SecurityLevel;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Security context for operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SecurityContext {
    /// Optional user identifier
    pub user_id: Option<String>,
    /// Session identifier
    pub session_id: String,
    /// List of granted permissions
    pub permissions: Vec<String>,
    /// Security level of this context
    pub security_level: SecurityLevel,
}

/// Universal request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalRequest {
    /// Unique identifier for this request
    pub request_id: String,
    /// Source system or service making the request
    pub source: String,
    /// Target system or service for the request
    pub target: String,
    /// Action to be performed
    pub action: String,
    /// Request parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Optional security context for authorization
    pub security_context: Option<SecurityContext>,
}

/// Response status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ResponseStatus {
    /// Request completed successfully
    #[default]
    Success,
    /// Request failed due to an error
    Error,
    /// Request is still being processed
    Pending,
    /// Request was partially completed
    PartialSuccess,
}

/// Universal response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalResponse {
    /// Request identifier this response corresponds to
    pub request_id: String,
    /// Status of the response
    pub status: ResponseStatus,
    /// Optional response data
    pub data: Option<serde_json::Value>,
    /// Optional error message
    pub error: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Universal event for system-wide communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalEvent {
    /// Unique identifier for this event
    pub event_id: String,
    /// Type of event
    pub event_type: String,
    /// Source system or service that generated the event
    pub source: String,
    /// Timestamp when the event occurred
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Event payload data
    pub payload: serde_json::Value,
}

/// Protocol characteristics for capability negotiation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolCharacteristics {
    /// Protocol name (e.g., "http", "grpc", "websocket")
    pub name: String,
    /// Protocol version
    pub version: String,
    /// Supported serialization formats
    pub serialization_formats: Vec<String>,
    /// Maximum message size in bytes
    pub max_message_size: Option<u64>,
    /// Whether streaming is supported
    pub streaming_supported: bool,
}
