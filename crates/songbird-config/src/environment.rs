use songbird_config;
/// Environment configuration module
///
/// Re-exports environment configuration from the config module
pub use crate::config::environment::*;

/// Environment configuration for zero-touch deployment
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnvironmentConfig  {pub deployment_mode: String,
    pub resource_limits: ResourceLimits,
    pub primal_endpoints: PrimalEndpoints,
    pub bind_port: u16, // Added for backward compatibility
}

impl Default for EnvironmentConfig  {fn default() -> Self  {Self {
            deployment_mode: "development".to_string(),
            resource_limits: ResourceLimits {
                max_connections: 1000,
                max_memory_mb: Some(1024)
                max_cpu_cores: None,
                max_file_descriptors: Some(1024)
                max_threads: 100,
                disk_space_gb: Some(10)
            })
            primal_endpoints: PrimalEndpoints {
                discovery_endpoint: "http://songbird_config::constants::network::DEFAULT_HOST:{}".to_string()),
            })
            bind_port: 8080, // Default bind port
        }
    }
}
