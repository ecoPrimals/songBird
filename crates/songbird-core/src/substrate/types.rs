//! Data structures and types for substrate operations

use serde::{Deserialize, Serialize};

/// System information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub platform: String,
    pub architecture: String,
    pub available_storage: u64,
    pub available_memory: u64,
    pub cpu_cores: u32,
    pub network_interfaces: Vec<NetworkInterface>,
}

/// Network interface information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub address: String,
    pub netmask: String,
    pub broadcast: Option<String>,
    pub status: String,
}

/// Network request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRequest {
    pub operation_type: String,
    pub payload: serde_json::Value,
}

/// Network response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkResponse {
    pub success: bool,
    pub data: serde_json::Value,
    pub message: String,
}

/// Path request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathRequest {
    pub path_type: PathType,
    pub service_name: String,
    pub requirements: PathRequirements,
}

/// Path type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PathType {
    Data,
    Config,
    Log,
    Cache,
    Temp,
    Binary,
    Plugin,
}

/// Path requirements structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathRequirements {
    pub writable: bool,
    pub executable: bool,
    pub size_limit: Option<u64>,
    pub permissions: Option<String>,
}

impl std::fmt::Display for PathType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathType::Data => write!(f, "data"),
            PathType::Config => write!(f, "config"),
            PathType::Log => write!(f, "log"),
            PathType::Cache => write!(f, "cache"),
            PathType::Temp => write!(f, "temp"),
            PathType::Binary => write!(f, "binary"),
            PathType::Plugin => write!(f, "plugin"),
        }
    }
}

impl Default for PathRequirements {
    fn default() -> Self {
        Self {
            writable: true,
            executable: false,
            size_limit: None,
            permissions: None,
        }
    }
}
