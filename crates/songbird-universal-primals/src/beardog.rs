/// BearDog primal configuration
///
/// Configuration for connecting to and interacting with BearDog security services.
/// BearDog provides advanced security, authentication, and threat detection capabilities.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Configuration for BearDog primal integration
pub struct BearDogConfig {
    /// Primary endpoint URL for BearDog services
    pub endpoint: String,
    /// Optional API key for authentication
    pub api_key: Option<String>,
    /// Whether to verify TLS certificates
    pub verify_tls: bool,
    /// Request timeout in seconds
    pub timeout_secs: u64,
    /// Health monitoring endpoint URL
    pub monitoring_endpoint: String,
    /// Maximum number of retry attempts
    pub max_retries: u32,
}

impl Default for BearDogConfig {
    fn default() -> Self {
        let bind_address = std::env::var("SONGBIRD_BIND_ADDRESS").unwrap_or_else(|_| {
            songbird_config::constants::network::DEFAULT_BIND_ADDRESS.to_string()
        });

        Self {
            endpoint: std::env::var("BEARDOG_ENDPOINT")
                .unwrap_or_else(|_| format!("https://{bind_address}:8443")),
            api_key: std::env::var("BEARDOG_API_KEY").ok(),
            verify_tls: std::env::var("BEARDOG_VERIFY_TLS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            timeout_secs: std::env::var("BEARDOG_TIMEOUT_SECONDS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            max_retries: std::env::var("BEARDOG_MAX_RETRIES")
                .unwrap_or_else(|_| "3".to_string())
                .parse()
                .unwrap_or(3),
            monitoring_endpoint: std::env::var("BEARDOG_MONITORING_ENDPOINT")
                .unwrap_or_else(|_| format!("http://{bind_address}:9090")),
        }
    }
}
