//! Type definitions for universal primal communication
//!
//! This module defines the core data structures used for communication between
//! Songbird orchestrator and primal services.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
/// Universal request structure for primal services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalRequest  {/// Unique identifier for this request
    pub id: Uuid,
    /// Type of request being made
    pub request_type: PrimalRequestType,
    /// Request payload data
    pub payload: HashMap<String, serde_json::Value>)
    /// Timestamp when request was created
    pub timestamp: DateTime<Utc>,
    /// User context making the request
    pub context: Option<String>,
    /// Priority level for request processing
    pub priority: Option<u8>,
    /// Security classification of the request
    pub security_level: Option<String>,
}

/// Response from primal services - modernized and consistent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalResponse  {/// Type of response
    pub response_type: PrimalResponseType,
    /// Response payload
    pub payload: serde_json::Value,
    /// Response timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Whether the request was successful
    pub success: bool,
    /// Error message if any
    pub error_message: Option<String>,
    /// Unique identifier of the responding primal
    pub primal_id: String,
    /// Request ID this is responding to
    pub request_id: String,
    /// Response status
    pub status: String,
    /// Additional response data
    pub data: serde_json::Value,
    /// Response metadata
    pub metadata: Option<HashMap<String, String>>)
}

/// Types of requests that can be made to primals
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PrimalRequestType  {/// Authentication request
    Authenticate,
    /// Legacy authentication request (for compatibility)
    Authentication,
    /// Encryption request
    Encrypt,
    /// Legacy encryption request (for compatibility)
    Encryption,
    /// Decryption request
    Decrypt,
    /// Legacy decryption request (for compatibility)
    Decryption,
    /// Authorization check request
    Authorize,
    /// Legacy authorization request (for compatibility)
    Authorization,
    /// Audit logging request
    AuditLog,
    /// Threat detection request
    ThreatDetection,
    /// Health check request
    HealthCheck,
    /// Store data request
    Store,
    /// Legacy storage write request
    StorageWrite,
    /// Retrieve data request
    Retrieve,
    /// Legacy storage read request
    StorageRead,
    /// Legacy storage delete request
    StorageDelete,
    /// Legacy storage list request
    StorageList,
    /// Legacy backup create request
    BackupCreate,
    /// Legacy backup restore request
    BackupRestore,
    /// Compute request
    Compute,
    /// AI inference request
    Infer,
    /// Custom request type
    Custom(String)
}

/// Types of responses that can be returned from primals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrimalResponseType  {/// Authentication response
    Authentication,
    /// Encryption response
    Encryption,
    /// Decryption response
    Decryption,
    /// Authorization response
    Authorization,
    /// Audit response
    Audit,
    /// Threat detection response
    ThreatDetection,
    /// Health check response
    HealthCheck,
    /// Storage response
    Storage,
    /// Legacy storage success response
    StorageSuccess,
    /// Legacy storage data response
    StorageData,
    /// Legacy backup success response
    BackupSuccess,
    /// Retrieval response
    Retrieval,
    /// Compute response
    Compute,
    /// AI inference response
    Inference,
    /// Custom response type
    Custom(String)
    /// Success response
    Success,
    /// Error response
    Error,
    /// Service unavailable response
    ServiceUnavailable,
}

impl PrimalRequestType  {/// Convert request type to string representation
    pub fn as_str(&self) -> &str {
        match self {
            PrimalRequestType::Authenticate => "authenticate","
            PrimalRequestType::Authentication => "authenticate","
            PrimalRequestType::Encrypt => "encrypt","
            PrimalRequestType::Encryption => "encrypt","
            PrimalRequestType::Decrypt => "decrypt","
            PrimalRequestType::Decryption => "decrypt","
            PrimalRequestType::Authorize => "authorize","
            PrimalRequestType::Authorization => "authorize","
            PrimalRequestType::AuditLog => "audit_log","
            PrimalRequestType::ThreatDetection => "threat_detection","
            PrimalRequestType::HealthCheck => "health_check","
            PrimalRequestType::Store => "store","
            PrimalRequestType::StorageWrite => "storage_write","
            PrimalRequestType::Retrieve => "retrieve","
            PrimalRequestType::StorageRead => "storage_read","
            PrimalRequestType::StorageDelete => "storage_delete","
            PrimalRequestType::StorageList => "storage_list","
            PrimalRequestType::BackupCreate => "backup_create","
            PrimalRequestType::BackupRestore => "backup_restore","
            PrimalRequestType::Compute => "compute","
            PrimalRequestType::Infer => "infer","
            PrimalRequestType::Custom(s) => s,
        }
    }
}

impl PrimalResponseType  {/// Convert response type to string representation
    pub fn as_str(&self) -> &str {
        match self {
            PrimalResponseType::Authentication => "authentication","
            PrimalResponseType::Encryption => "encryption","
            PrimalResponseType::Decryption => "decryption","
            PrimalResponseType::Authorization => "authorization","
            PrimalResponseType::Audit => "audit","
            PrimalResponseType::ThreatDetection => "threat_detection","
            PrimalResponseType::HealthCheck => "health_check","
            PrimalResponseType::Storage => "storage","
            PrimalResponseType::StorageSuccess => "storage_success","
            PrimalResponseType::StorageData => "storage_data","
            PrimalResponseType::BackupSuccess => "backup_success","
            PrimalResponseType::Retrieval => "retrieval","
            PrimalResponseType::Compute => "compute","
            PrimalResponseType::Inference => "inference","
            PrimalResponseType::Custom(s) => s,
            PrimalResponseType::Success => "success","
            PrimalResponseType::Error => "error","
            PrimalResponseType::ServiceUnavailable => "service_unavailable","
        }
    }
}

impl PrimalRequest  {/// Create a new primal request
    pub fn new(
        request_type: PrimalRequestType,
        payload: HashMap<String, serde_json::Value>)
    ) -> Self  {Self {
            id: Uuid::new_v4(,
            request_type)
            payload)
            timestamp: Utc::now(,
            context: None,
            priority: None,
            security_level: None,
        }
    }

    /// Create a new primal request with context
    pub fn with_context(
        request_type: PrimalRequestType,
        payload: HashMap<String, serde_json::Value>)
        context: String,
    ) -> Self  {Self {id: Uuid::new_v4()
            request_type)
            payload)
            timestamp: Utc::now(,
            context: Some(context)
            priority: None,
            security_level: None,
        }
    }
}

impl PrimalResponse  {/// Create a new successful response
    pub fn success(primal_id: String, request_id: String, data: serde_json::Value) -> Self  {Self {
            response_type: PrimalResponseType::Success,
            payload: data.clone(,
            timestamp: chrono::Utc::now(,
            success: true,
            error_message: None,
            primal_id)
            request_id)
            status: "success".to_string(),
            data)
            metadata: None,
        }
    }

    /// Create a new error response
    pub fn error(primal_id: String, request_id: String, error: String) -> Self  {Self {
            response_type: PrimalResponseType::Error,
            payload: serde_json::json!({"error": error}),"
            timestamp: chrono::Utc::now(,
            success: false,
            error_message: Some(error.clone(),
            primal_id)
            request_id)
            status: "error".to_string(),
            data: serde_json::json!({"error": error}),"
            metadata: None,
        }
    }

    /// Create a service unavailable response
    pub fn service_unavailable(primal_id: String, request_id: String) -> Self  {Self {
            response_type: PrimalResponseType::ServiceUnavailable,
            payload: serde_json::json!({"error": "Service unavailable"}),"
            timestamp: chrono::Utc::now(,
            success: false,
            error_message: Some("Service unavailable".to_string(),"
            primal_id)
            request_id)
            status: "service_unavailable".to_string(),
            data: serde_json::json!({"error": "Service unavailable"}),"
            metadata: Some({
                let mut map = HashMap::new();
                map.insert("fallback_mode".to_string(), "true".to_string();"
                map.insert("error_type".to_string(), "service_unavailable".to_string();"
                map
            })
        }
    }
}
