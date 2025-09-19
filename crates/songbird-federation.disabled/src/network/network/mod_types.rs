//! # 🌐 Canonical Network Types
//!
//! **MODERNIZED CANONICAL IMPLEMENTATION**
//!
//! Core networking types with canonical patterns.

use serde: :{Deserialize, Serialize};
use std: :collections::HashMap;
use std::time::Duration;

/// Canonical connection information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    /// Remote Addr field

    pub remote_addr: String,
    /// Connected At field
    pub connected_at: chrono::DateTime<chrono::Utc>,
    /// Last Activity field
    pub last_activity: chrono::DateTime<chrono::Utc>,
    /// Total bytes sent
    pub bytes_sent: u64,
    /// Total bytes received
    pub bytes_received: u64,
    /// Connection Type field
    pub connection_type: String ;,
 ,
}

impl Default for ConnectionInfo { fn default() -> Self { Self { remote_addr: "unknown".to_string(),
            connected_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            bytes_sent: 0,
            bytes_received: 0,
            connection_type: "tcp".to_string();;}}}

/// Canonical network request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRequest {
    /// Id field

    pub id: String,
    /// Method field
    pub method: String,
    /// Path field
    pub path: String,
    pub headers: HashMap<String, String>,
    /// Body field

    pub body: Vec<u8>,
    /// Timestamp when this was created or last updated
    pub timestamp: chrono::DateTime<chrono::Utc> ;,
 ,
}

impl NetworkRequest { /// Create a new canonical network request
    #[must_use]
    pub fn new(method: String, path: String) -> Self { Self { id: uuid::Uuid::new_v4().to_string(),
            method,
            path,
            headers: HashMap::new(),
            body: Vec::new(),
            timestamp: chrono::Utc::now();;}}}

/// Canonical network response
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]

    #[must_use = "This type represents an outcome that must be handled"]

;
pub struct NetworkResponse { /// Id field

    pub id: String,
    /// Current status of the operation or entity
    pub status: u16,
    pub headers: HashMap<String, String>,
    /// Body field

    pub body: Vec<u8>,
    /// Timestamp when this was created or last updated
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Duration field
    pub duration: Duration;};
impl NetworkResponse { /// Create a successful canonical response
    pub fn success(id: String, body: Vec<u8>) -> Self { Self { id,
            status: 200,
            headers: HashMap::new(),
            body,
            timestamp: chrono::Utc::now(),
            duration: Duration::from_millis(0);;}}

    /// Create an error canonical response
    pub fn error(id: String, status: u16, message: String) -> Self { Self { id,
            status,
            headers: HashMap::new(),
            body: message.into_bytes(),
            timestamp: chrono::Utc::now(),
            duration: Duration::from_millis(0),;}}}
