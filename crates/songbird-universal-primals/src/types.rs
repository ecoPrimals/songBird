//! Type definitions for universal primal communication
//! 
//! This module defines the core data structures used for communication between
//! Songbird orchestrator and primal services.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Universal request structure for primal services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalRequest {
    /// Unique identifier for this request
    pub id: Uuid,
    /// Type of request being made
    pub request_type: PrimalRequestType,
    /// Request payload data
    pub payload: HashMap<String, serde_json::Value>,
    /// Timestamp when request was created
    pub timestamp: DateTime<Utc>,
    /// User context making the request
    pub context: Option<String>,
    /// Priority level for request processing
    pub priority: Option<u8>,
    /// Security classification of the request
    pub security_level: Option<String>,
}

/// Universal response structure from primal services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalResponse {
    /// Request ID this response corresponds to
    pub request_id: Uuid,
    /// Type of response being returned
    pub response_type: PrimalResponseType,
    /// Response payload data
    pub payload: HashMap<String, serde_json::Value>,
    /// Timestamp when response was created
    pub timestamp: DateTime<Utc>,
    /// Whether the request was successful
    pub success: bool,
    /// Error message if request failed
    pub error_message: Option<String>,
    /// Additional metadata about the response
    pub metadata: Option<HashMap<String, String>>,
}

/// Types of requests that can be made to primals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrimalRequestType {
    /// Authentication request
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
    Custom(String),
}

/// Types of responses that can be returned from primals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrimalResponseType {
    /// Authentication response
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
    Custom(String),
}

impl PrimalRequestType {
    /// Convert request type to string representation
    pub fn as_str(&self) -> &str {
        match self {
            PrimalRequestType::Authenticate => "authenticate",
            PrimalRequestType::Authentication => "authenticate",
            PrimalRequestType::Encrypt => "encrypt",
            PrimalRequestType::Encryption => "encrypt",
            PrimalRequestType::Decrypt => "decrypt",
            PrimalRequestType::Decryption => "decrypt",
            PrimalRequestType::Authorize => "authorize",
            PrimalRequestType::Authorization => "authorize",
            PrimalRequestType::AuditLog => "audit_log",
            PrimalRequestType::ThreatDetection => "threat_detection",
            PrimalRequestType::HealthCheck => "health_check",
            PrimalRequestType::Store => "store",
            PrimalRequestType::StorageWrite => "storage_write",
            PrimalRequestType::Retrieve => "retrieve",
            PrimalRequestType::StorageRead => "storage_read",
            PrimalRequestType::StorageDelete => "storage_delete",
            PrimalRequestType::StorageList => "storage_list",
            PrimalRequestType::BackupCreate => "backup_create",
            PrimalRequestType::BackupRestore => "backup_restore",
            PrimalRequestType::Compute => "compute",
            PrimalRequestType::Infer => "infer",
            PrimalRequestType::Custom(s) => s,
        }
    }
}

impl PrimalResponseType {
    /// Convert response type to string representation
    pub fn as_str(&self) -> &str {
        match self {
            PrimalResponseType::Authentication => "authentication",
            PrimalResponseType::Encryption => "encryption",
            PrimalResponseType::Decryption => "decryption",
            PrimalResponseType::Authorization => "authorization",
            PrimalResponseType::Audit => "audit",
            PrimalResponseType::ThreatDetection => "threat_detection",
            PrimalResponseType::HealthCheck => "health_check",
            PrimalResponseType::Storage => "storage",
            PrimalResponseType::StorageSuccess => "storage_success",
            PrimalResponseType::StorageData => "storage_data",
            PrimalResponseType::BackupSuccess => "backup_success",
            PrimalResponseType::Retrieval => "retrieval",
            PrimalResponseType::Compute => "compute",
            PrimalResponseType::Inference => "inference",
            PrimalResponseType::Custom(s) => s,
        }
    }
}

impl PrimalRequest {
    /// Create a new primal request
    pub fn new(request_type: PrimalRequestType, payload: HashMap<String, serde_json::Value>) -> Self {
        Self {
            id: Uuid::new_v4(),
            request_type,
            payload,
            timestamp: Utc::now(),
            context: None,
            priority: None,
            security_level: None,
        }
    }

    /// Create a new primal request with context
    pub fn with_context(
        request_type: PrimalRequestType,
        payload: HashMap<String, serde_json::Value>,
        context: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            request_type,
            payload,
            timestamp: Utc::now(),
            context: Some(context),
            priority: None,
            security_level: None,
        }
    }
}

impl PrimalResponse {
    /// Create a new successful response
    pub fn success(
        request_id: Uuid,
        response_type: PrimalResponseType,
        payload: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            request_id,
            response_type,
            payload,
            timestamp: Utc::now(),
            success: true,
            error_message: None,
            metadata: None,
        }
    }

    /// Create a new error response
    pub fn error(
        request_id: Uuid,
        response_type: PrimalResponseType,
        error_message: String,
    ) -> Self {
        Self {
            request_id,
            response_type,
            payload: HashMap::new(),
            timestamp: Utc::now(),
            success: false,
            error_message: Some(error_message),
            metadata: None,
        }
    }
} 