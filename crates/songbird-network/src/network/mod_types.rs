//! # 🌐 Canonical Network Types
//!
//! **MODERNIZED CANONICAL IMPLEMENTATION**
//!
//! Core networking types with canonical patterns.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Canonical connection information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub remote_addr: String,
    pub connected_at: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub connection_type: String,
}

impl Default for ConnectionInfo {
    fn default() -> Self {
        Self {
            remote_addr: "unknown".to_string(),
            connected_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            bytes_sent: 0,
            bytes_received: 0,
            connection_type: "tcp".to_string(),
        }
    }
}

/// Canonical network request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRequest {
    pub id: String,
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl NetworkRequest {
    /// Create a new canonical network request
    pub fn new(method: String, path: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            method,
            path,
            headers: HashMap::new(),
            body: Vec::new(),
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Canonical network response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkResponse {
    pub id: String,
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub duration: Duration,
}

impl NetworkResponse {
    /// Create a successful canonical response
    pub fn success(id: String, body: Vec<u8>) -> Self {
        Self {
            id,
            status: 200,
            headers: HashMap::new(),
            body,
            timestamp: chrono::Utc::now(),
            duration: Duration::from_millis(0),
        }
    }

    /// Create an error canonical response
    pub fn error(id: String, status: u16, message: String) -> Self {
        Self {
            id,
            status,
            headers: HashMap::new(),
            body: message.into_bytes(),
            timestamp: chrono::Utc::now(),
            duration: Duration::from_millis(0),
        }
    }
}
