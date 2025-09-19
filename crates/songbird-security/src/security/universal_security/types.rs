//! Universal Security Types Types
//!
//! Common types used across the universal security system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;
use uuid::Uuid;

/// Information about discovered security capabilities
#[derive(Debug, Clone)]
pub struct SecurityCapabilityInfo {
    /// Primal providing the capability
        pub primal_id: String,
    /// Instance ID for multi-instance support
    /// Instance Id field

    pub instance_id: String,
    /// Capabilities provided by this primal
        pub capabilities: Vec<String>,
    /// Endpoint URL for this primal
    /// Endpoint field

    pub endpoint: String,
    /// Last health check time
        pub security_level: SecurityLevel,
    /// Performance metrics
    pub performance_metrics: HashMap<String, f64> ,

}

/// Security levels for capability matching
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecurityLevel { /// Basic security (development/testing)
    /// Basic, Basic,
    /// Standard security (production)
    /// Standard, Standard,
    /// High security (sensitive data)
    /// High, High,
    Maximum  }

/// Security operation context
#[derive(Debug, Clone)]
pub struct SecurityContext {
    /// Request ID for tracing
        pub request_id: Uuid,
    /// User/service requesting the operation
        pub subject: String,
    /// Type of subject (user, service, system)
    /// Subject Type field

    pub subject_type: SubjectType,
    /// Operation being performed
    /// Operation field

    pub operation: String,
    /// Resource being accessed
        pub resource: Option<String>,
    /// Additional context metadata
    pub metadata: HashMap<String, String>,
    /// Required security level
        pub required_level: SecurityLevel ,

}

/// Subject types for security operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubjectType { /// Human user
    /// User, User,
    /// Service account
    /// Service, Service,
    /// System process
    /// System, System,
    Client  }

/// Security operation result
#[derive(Debug, Clone)]
pub struct SecurityResult<T> {
    /// Operation success status
    pub data: Option<T>,
    /// Error information (if failed)
    pub error: Option<String>,
    /// Security metadata
    pub security_metadata: HashMap<String, String>,
    /// Audit trail information
    /// Audit Info field

    pub audit_info: Option<AuditInfo>,
}

/// Audit information for security operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditInfo {
    /// Timestamp of the operation
    /// Timestamp when this was created or last updated

    pub timestamp: SystemTime,
    /// Subject that performed the operation
        pub subject: String,
    /// Operation that was performed
    /// Operation field

    pub operation: String,
    /// Resource that was accessed
        pub resource: Option<String>,
    /// Result of the operation
        pub result: AuditResult,
    /// Additional audit metadata
    pub metadata: HashMap<String, String> ,

}

/// Audit result types
#[derive(Debug, Clone, Serialize, Deserialize)]
    
pub enum AuditResult { /// Operation succeeded
    /// Success, Success,
    /// Operation failed
    /// Failure
        Failure(String),
    /// Operation was denied
    /// Denied
        Denied(String),
    /// Operation was partially successful
    /// Partial
        Partial(String),
}

impl<T> SecurityResult<T> {
    /// Create a successful result
    pub fn success(data: T) -> Self { 
        Self { 
            success: true,
            data: Some(data),
            error: None,
            security_metadata: HashMap::new(),
            audit_info: None,
        }
    }
    
    /// Create a failed result
    pub fn failure(error: String) -> Self { 
        Self { 
            success: false,
            data: None,
            error: Some(error),
            security_metadata: HashMap::new(),
            audit_info: None,
        }
    }
    
    /// Add audit information
    pub fn with_audit(mut self, audit: AuditInfo) -> Self {
        self.audit_info = Some(audit);
        self
    }
    /// Add security metadata
    
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.security_metadata.insert(key, value);
        self
    }
}

impl Default for SecurityLevel { 
    fn default() -> Self { 
        Self::Standard
    }
}

impl std::fmt::Display for SecurityLevel { 
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { 
        match self { 
            Self::Basic => write!(f, "basic"),
            Self::Standard => write!(f, "standard"),
            Self::High => write!(f, "high"),
            Self::Maximum => write!(f, "maximum"),
        }
    }
}
