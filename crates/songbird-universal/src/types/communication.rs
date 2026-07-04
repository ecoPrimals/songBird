// SPDX-License-Identifier: AGPL-3.0-or-later
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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]
    #![allow(clippy::expect_used, reason = "test assertions")]

    use super::{
        ProtocolCharacteristics, ResponseStatus, SecurityContext, UniversalEvent, UniversalRequest,
        UniversalResponse,
    };
    use crate::types::capability::SecurityLevel;
    use songbird_test_utils::canonical_test_framework::TestContext;
    use std::collections::HashMap;

    #[test]
    fn security_context_roundtrip() {
        let ctx = TestContext::new("comm_sec");
        let s = SecurityContext {
            user_id: Some(String::from("u")),
            session_id: String::from("sess"),
            permissions: vec![String::from("read")],
            security_level: SecurityLevel::High,
        };
        let j = serde_json::to_string(&s).unwrap();
        let back: SecurityContext = serde_json::from_str(&j).unwrap();
        assert_eq!(s, back);
        assert!(!ctx.is_timeout());
    }

    #[test]
    fn universal_request_roundtrip() {
        let mut params = HashMap::new();
        params.insert(String::from("k"), serde_json::json!(1));
        let r = UniversalRequest {
            request_id: String::from("r1"),
            source: String::from("a"),
            target: String::from("b"),
            action: String::from("act"),
            parameters: params,
            security_context: None,
        };
        let j = serde_json::to_string(&r).unwrap();
        let back: UniversalRequest = serde_json::from_str(&j).unwrap();
        assert_eq!(r.request_id, back.request_id);
        assert_eq!(r.action, back.action);
    }

    #[test]
    fn response_status_default_is_success() {
        assert_eq!(ResponseStatus::default(), ResponseStatus::Success);
    }

    #[test]
    fn response_status_serde_roundtrip() {
        for st in [
            ResponseStatus::Success,
            ResponseStatus::Error,
            ResponseStatus::Pending,
            ResponseStatus::PartialSuccess,
        ] {
            let j = serde_json::to_string(&st).unwrap();
            let back: ResponseStatus = serde_json::from_str(&j).unwrap();
            assert_eq!(st, back);
        }
    }

    #[test]
    fn universal_response_roundtrip() {
        let r = UniversalResponse {
            request_id: String::from("id"),
            status: ResponseStatus::Pending,
            data: Some(serde_json::json!({"x": 1})),
            error: None,
            metadata: HashMap::from([(String::from("m"), String::from("v"))]),
        };
        let j = serde_json::to_string(&r).unwrap();
        let back: UniversalResponse = serde_json::from_str(&j).unwrap();
        assert_eq!(r.request_id, back.request_id);
        assert_eq!(r.status, back.status);
    }

    #[test]
    fn universal_event_roundtrip() {
        let e = UniversalEvent {
            event_id: String::from("e1"),
            event_type: String::from("t"),
            source: String::from("s"),
            timestamp: chrono::Utc::now(),
            payload: serde_json::json!({}),
        };
        let j = serde_json::to_string(&e).unwrap();
        let back: UniversalEvent = serde_json::from_str(&j).unwrap();
        assert_eq!(e.event_id, back.event_id);
    }

    #[test]
    fn protocol_characteristics_roundtrip() {
        let p = ProtocolCharacteristics {
            name: String::from("http"),
            version: String::from("1.1"),
            serialization_formats: vec![String::from("json")],
            max_message_size: Some(1024),
            streaming_supported: false,
        };
        let j = serde_json::to_string(&p).unwrap();
        let back: ProtocolCharacteristics = serde_json::from_str(&j).unwrap();
        assert_eq!(p.name, back.name);
        assert_eq!(p.streaming_supported, back.streaming_supported);
    }
}
