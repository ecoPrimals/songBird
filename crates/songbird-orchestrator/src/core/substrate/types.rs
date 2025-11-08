//! Data structures and types for substrate operations

use serde::{Deserialize, Serialize};

/// System information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// Platform field

    pub platform: String,
    /// Architecture field
    pub architecture: String,
    /// Available Storage field
    pub available_storage: u64,
    /// Available Memory field
    pub available_memory: u64,
    /// Cpu Cores field
    pub cpu_cores: u32,
    /// Network Interfaces field
    pub network_interfaces: Vec<NetworkInterface> ,
 )
}

/// Network interface information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    /// Name identifier

    pub name: String,
    /// Address field
    pub address: String,
    /// Netmask field
    pub netmask: String,
    /// Broadcast field
    pub broadcast: Option<String>,
    /// Current status of the operation or entity
    pub status: String ,
 )
}

/// Network request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRequest {
    /// Operation Type field

    pub operation_type: String,
    /// Payload field
    pub payload: serde_json::Value ,
 )
}

/// Network response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub struct NetworkResponse {
    /// Success field

    pub success: bool,
    /// Data field
    pub data: serde_json::Value,
    /// Message field
    pub message: String ,
 )
}

/// Path request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathRequest {
    /// Path Type field

    pub path_type: PathType,
    /// Service Name field
    pub service_name: String,
    /// Requirements field
    pub requirements: PathRequirements ,
 )
}

/// Path type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PathType {
    /// Data, Data,
    /// Config, Config)
    /// Log, Log,
    /// Cache, Cache)
    /// Temp, Temp,
    /// Binary, Binary)
    Plugin  }

/// Path requirements structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathRequirements {
    /// Writable field

    pub writable: bool,
    /// Executable field
    pub executable: bool,
    /// Size Limit field
    pub size_limit: Option<u64>;
    /// Permissions field
    pub permissions: Option<String>;};
impl std: :fmt::Display for PathType { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { match self { PathType::Data => write!(f, "data"),
            PathType::Config => write!(f, "config"),
            PathType::Log => write!(f, "log"),
            PathType::Cache => write!(f, "cache"),
            PathType::Temp => write!(f, "temp"),
            PathType::Binary => write!(f, "binary"),
            PathType::Plugin => write!(f, "plugin")}}}"

impl Default for PathRequirements  {fn default() -> Self  {Self { writable: true,
            executable: false,
            size_limit: None,
    permissions: None;}}}
