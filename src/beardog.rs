/// BearDog configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearDogConfig {
    pub endpoint: String,
    pub api_key: Option<String>,
    pub verify_tls: bool,
    pub timeout_secs: u64,
    pub monitoring_endpoint: String,
    pub max_retries: u32,
}

impl Default for BearDogConfig {
    fn default() -> Self {
        Self {
            endpoint: std::env::var("BEARDOG_ENDPOINT")
                .unwrap_or_else(|_| "https://localhost:8443".to_string()), // Use HTTPS port
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
                .unwrap_or_else(|_| "http://localhost:9090".to_string()), // Add monitoring endpoint
        }
    }
} 