//! BearDog Security Primal Integration
//!
//! Provides HTTP client adapter for BearDog security primal with multi-instance support

use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, error, info};

use crate::errors::{PrimalError, PrimalResult};
use crate::traits::{
    DynamicPortInfo, PortType, PrimalCapability, PrimalContext, PrimalDependency, PrimalEndpoints,
    PrimalHealth, PrimalProvider, PrimalType, SecurityLevel,
};
use crate::types::{PrimalRequest, PrimalRequestType, PrimalResponse, PrimalResponseType};

/// BearDog Security Primal
///
/// Enhanced to support multi-instance deployment with dynamic port management
pub struct BearDogPrimal {
    /// Instance identifier
    instance_id: String,
    /// User/device context
    context: PrimalContext,
    /// HTTP client for API communication
    client: BearDogClient,
    /// Dynamic port information
    port_info: Option<DynamicPortInfo>,
}

impl BearDogPrimal {
    /// Create a new BearDog primal instance
    pub fn new() -> Self {
        let default_config = BearDogConfig::default();
        let context = PrimalContext::default();
        let instance_id = format!("beardog-{}-{}", context.user_id, context.device_id);

        Self {
            instance_id,
            context,
            client: BearDogClient::new(default_config),
            port_info: None,
        }
    }

    /// Create a new BearDog primal instance with specific context
    pub fn with_context(context: PrimalContext) -> Self {
        let instance_id = format!("beardog-{}-{}", context.user_id, context.device_id);
        let config = BearDogConfig::for_context(&context);

        Self {
            instance_id,
            context,
            client: BearDogClient::new(config),
            port_info: None,
        }
    }

    /// Create a new BearDog primal instance with dynamic port
    pub fn with_dynamic_port(context: PrimalContext, port_info: DynamicPortInfo) -> Self {
        let instance_id = format!("beardog-{}-{}", context.user_id, context.device_id);
        let mut config = BearDogConfig::for_context(&context);

        // Update endpoint to use dynamic port
        config.endpoint = match port_info.port_type {
            PortType::Http => format!("http://localhost:{}", port_info.assigned_port),
            PortType::Https => format!("https://localhost:{}", port_info.assigned_port),
            _ => format!("http://localhost:{}", port_info.assigned_port),
        };

        Self {
            instance_id,
            context,
            client: BearDogClient::new(config),
            port_info: Some(port_info),
        }
    }

    /// Create from environment variables for specific context
    pub fn from_env_with_context(context: PrimalContext) -> PrimalResult<Self> {
        let instance_id = format!("beardog-{}-{}", context.user_id, context.device_id);
        let config = BearDogConfig::from_env_for_context(&context)?;

        Ok(Self {
            instance_id,
            context,
            client: BearDogClient::new(config),
            port_info: None,
        })
    }

    /// Create from environment variables (legacy support)
    pub fn from_env() -> PrimalResult<Self> {
        let context = PrimalContext::default();
        Self::from_env_with_context(context)
    }
}

#[async_trait]
impl PrimalProvider for BearDogPrimal {
    fn primal_id(&self) -> &str {
        "beardog"
    }

    fn instance_id(&self) -> &str {
        &self.instance_id
    }

    fn context(&self) -> &PrimalContext {
        &self.context
    }

    fn primal_type(&self) -> PrimalType {
        PrimalType::Security
    }

    fn capabilities(&self) -> Vec<PrimalCapability> {
        vec![
            PrimalCapability::Authentication {
                methods: vec!["basic".to_string(), "token".to_string(), "mfa".to_string()],
            },
            PrimalCapability::Encryption {
                algorithms: vec![
                    "AES256".to_string(),
                    "RSA2048".to_string(),
                    "ChaCha20".to_string(),
                ],
            },
            PrimalCapability::KeyManagement { hsm_support: true },
            PrimalCapability::ThreatDetection { ml_enabled: true },
            PrimalCapability::AuditLogging {
                compliance: vec!["SOC2".to_string(), "GDPR".to_string(), "HIPAA".to_string()],
            },
            PrimalCapability::Authorization { rbac_support: true },
        ]
    }

    fn dependencies(&self) -> Vec<PrimalDependency> {
        vec![
            // BearDog is typically self-contained for security
        ]
    }

    async fn health_check(&self) -> PrimalHealth {
        match self.client.health_check().await {
            Ok(true) => PrimalHealth::Healthy,
            Ok(false) => PrimalHealth::Degraded {
                issues: vec!["Service responding but degraded".to_string()],
            },
            Err(e) => PrimalHealth::Unhealthy {
                reason: format!("Health check failed: {e}"),
            },
        }
    }

    fn endpoints(&self) -> PrimalEndpoints {
        let config = &self.client.config;

        PrimalEndpoints {
            primary: config.endpoint.clone(),
            health: format!("{}/health", config.endpoint),
            metrics: Some(config.monitoring_endpoint.clone()),
            admin: Some(format!("{}/admin", config.endpoint)),
            websocket: None,
            custom: HashMap::new(),
        }
    }

    async fn handle_primal_request(&self, request: PrimalRequest) -> PrimalResult<PrimalResponse> {
        debug!(
            "BearDog instance {} handling request: {:?}",
            self.instance_id, request.request_type
        );

        match request.request_type {
            PrimalRequestType::Authentication => self.handle_authentication(&request).await,
            PrimalRequestType::Encryption => self.handle_encryption(&request).await,
            PrimalRequestType::Decryption => self.handle_decryption(&request).await,
            PrimalRequestType::AuditLog => self.handle_audit_log(&request).await,
            PrimalRequestType::Authorization => self.handle_authorization(&request).await,
            PrimalRequestType::ThreatDetection => self.handle_threat_detection(&request).await,
            _ => Err(PrimalError::InvalidRequest(format!(
                "Unsupported request type: {}",
                request.request_type.as_str()
            ))),
        }
    }

    /// Initialize the primal with configuration
    async fn initialize(&mut self, config: serde_json::Value) -> PrimalResult<()> {
        debug!("Initializing BearDog primal with config: {:?}", config);

        // Initialize HTTP client and perform health check
        match self.client.health_check().await {
            Ok(_) => {
                info!("BearDog primal initialized successfully");
                Ok(())
            }
            Err(e) => {
                error!("Failed to initialize BearDog primal: {}", e);
                Err(PrimalError::Configuration(format!(
                    "Initialization failed: {e}"
                )))
            }
        }
    }

    async fn shutdown(&mut self) -> PrimalResult<()> {
        info!("Shutting down BearDog instance: {}", self.instance_id);

        // Graceful shutdown - close connections, etc.
        // The HTTP client will be dropped automatically

        Ok(())
    }

    fn can_serve_context(&self, context: &PrimalContext) -> bool {
        // Check if this instance can serve the given context
        self.context.user_id == context.user_id
            && self.context.device_id == context.device_id
            && self.context.security_level >= context.security_level
    }

    fn dynamic_port_info(&self) -> Option<DynamicPortInfo> {
        self.port_info.clone()
    }
}

impl BearDogPrimal {
    /// Handle authentication request
    async fn handle_authentication(&self, request: &PrimalRequest) -> PrimalResult<PrimalResponse> {
        let payload = HashMap::new();

        Ok(PrimalResponse {
            request_id: request.id,
            response_type: PrimalResponseType::Authentication,
            payload,
            timestamp: chrono::Utc::now(),
            success: true,
            error_message: None,
            metadata: Some(HashMap::new()),
        })
    }

    /// Handle authorization request
    async fn handle_authorization(&self, request: &PrimalRequest) -> PrimalResult<PrimalResponse> {
        let payload = HashMap::new();

        Ok(PrimalResponse {
            request_id: request.id,
            response_type: PrimalResponseType::Authorization,
            payload,
            timestamp: chrono::Utc::now(),
            success: true,
            error_message: None,
            metadata: Some(HashMap::new()),
        })
    }

    /// Handle encryption request
    async fn handle_encryption(&self, request: &PrimalRequest) -> PrimalResult<PrimalResponse> {
        let payload = HashMap::new();

        Ok(PrimalResponse {
            request_id: request.id,
            response_type: PrimalResponseType::Encryption,
            payload,
            timestamp: chrono::Utc::now(),
            success: true,
            error_message: None,
            metadata: Some(HashMap::new()),
        })
    }

    /// Handle decryption request
    async fn handle_decryption(&self, request: &PrimalRequest) -> PrimalResult<PrimalResponse> {
        let payload = HashMap::new();

        Ok(PrimalResponse {
            request_id: request.id,
            response_type: PrimalResponseType::Decryption,
            payload,
            timestamp: chrono::Utc::now(),
            success: true,
            error_message: None,
            metadata: Some(HashMap::new()),
        })
    }

    /// Handle audit logging request
    async fn handle_audit_log(&self, request: &PrimalRequest) -> PrimalResult<PrimalResponse> {
        let payload = HashMap::new();

        Ok(PrimalResponse {
            request_id: request.id,
            response_type: PrimalResponseType::Audit,
            payload,
            timestamp: chrono::Utc::now(),
            success: true,
            error_message: None,
            metadata: Some(HashMap::new()),
        })
    }

    /// Handle threat detection request
    async fn handle_threat_detection(
        &self,
        request: &PrimalRequest,
    ) -> PrimalResult<PrimalResponse> {
        let payload = HashMap::new();

        Ok(PrimalResponse {
            request_id: request.id,
            response_type: PrimalResponseType::ThreatDetection,
            payload,
            timestamp: chrono::Utc::now(),
            success: true,
            error_message: None,
            metadata: Some(HashMap::new()),
        })
    }
}

/// BearDog HTTP client for API communication
pub struct BearDogClient {
    config: BearDogConfig,
    http_client: reqwest::Client,
}

impl BearDogClient {
    /// Create a new BearDog client with the given configuration
    pub fn new(config: BearDogConfig) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            http_client,
        }
    }

    /// Health check endpoint
    pub async fn health_check(&self) -> PrimalResult<bool> {
        let url = format!("{}/health", self.config.endpoint);

        match self.http_client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Err(e) => {
                error!("Health check failed: {}", e);
                Err(PrimalError::Network(e.to_string()))
            }
        }
    }

    /// Authenticate user
    pub async fn authenticate(&self, username: &str, password: &str) -> PrimalResult<AuthResponse> {
        let url = format!("{}/api/auth/login", self.config.endpoint);

        let payload = serde_json::json!({
            "username": username,
            "password": password
        });

        let response = self.http_client.post(&url).json(&payload).send().await?;

        if response.status().is_success() {
            let auth_response: AuthResponse = response.json().await?;
            Ok(auth_response)
        } else {
            Err(PrimalError::Authentication(format!(
                "Authentication failed: {}",
                response.status()
            )))
        }
    }

    /// Encrypt data
    pub async fn encrypt(&self, data: &str) -> PrimalResult<String> {
        let url = format!("{}/api/crypto/encrypt", self.config.endpoint);

        let payload = serde_json::json!({
            "data": data,
            "algorithm": "AES256"
        });

        let response = self.http_client.post(&url).json(&payload).send().await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            Ok(result["encrypted_data"].as_str().unwrap_or("").to_string())
        } else {
            Err(PrimalError::Encryption(format!(
                "Encryption failed: {}",
                response.status()
            )))
        }
    }

    /// Decrypt data
    pub async fn decrypt(&self, encrypted_data: &str) -> PrimalResult<String> {
        let url = format!("{}/api/crypto/decrypt", self.config.endpoint);

        let payload = serde_json::json!({
            "encrypted_data": encrypted_data
        });

        let response = self.http_client.post(&url).json(&payload).send().await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            Ok(result["decrypted_data"].as_str().unwrap_or("").to_string())
        } else {
            Err(PrimalError::Encryption(format!(
                "Decryption failed: {}",
                response.status()
            )))
        }
    }

    /// Log audit event
    pub async fn audit_log(&self, event: &str) -> PrimalResult<bool> {
        let url = format!("{}/api/audit/log", self.config.endpoint);

        let payload = serde_json::json!({
            "event": event,
            "timestamp": chrono::Utc::now(),
            "user_id": "system"
        });

        let response = self.http_client.post(&url).json(&payload).send().await?;

        Ok(response.status().is_success())
    }

    /// Authorize user action
    pub async fn authorize(&self, user: &str, resource: &str, action: &str) -> PrimalResult<bool> {
        let url = format!("{}/api/authz/check", self.config.endpoint);

        let payload = serde_json::json!({
            "user": user,
            "resource": resource,
            "action": action
        });

        let response = self.http_client.post(&url).json(&payload).send().await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            Ok(result["authorized"].as_bool().unwrap_or(false))
        } else {
            Ok(false)
        }
    }

    /// Detect threats
    pub async fn detect_threat(&self, event_data: &str) -> PrimalResult<bool> {
        let url = format!("{}/api/threat/detect", self.config.endpoint);

        let payload = serde_json::json!({
            "event_data": event_data,
            "timestamp": chrono::Utc::now()
        });

        let response = self.http_client.post(&url).json(&payload).send().await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            Ok(result["threat_detected"].as_bool().unwrap_or(false))
        } else {
            Ok(false)
        }
    }
}

/// BearDog configuration
#[derive(Debug, Clone)]
pub struct BearDogConfig {
    /// The primary endpoint URL for the BearDog service
    pub endpoint: String,
    /// The monitoring endpoint URL for health checks and metrics
    pub monitoring_endpoint: String,
    /// Optional API key for authentication
    pub api_key: Option<String>,
    /// Request timeout in seconds
    pub timeout_secs: u64,
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Security level for this configuration
    pub security_level: SecurityLevel,
}

impl BearDogConfig {
    /// Create configuration for specific context
    pub fn for_context(context: &PrimalContext) -> Self {
        let base_port = Self::get_base_port_for_security_level(&context.security_level);

        Self {
            endpoint: format!("https://localhost:{base_port}"),
            monitoring_endpoint: format!("http://localhost:{}", base_port + 1000),
            api_key: None,
            timeout_secs: 30,
            max_retries: 3,
            security_level: context.security_level.clone(),
        }
    }

    /// Create configuration from environment variables for specific context
    pub fn from_env_for_context(context: &PrimalContext) -> PrimalResult<Self> {
        let endpoint = std::env::var("BEARDOG_ENDPOINT").unwrap_or_else(|_| {
            let base_port = Self::get_base_port_for_security_level(&context.security_level);
            format!("https://localhost:{base_port}")
        });

        let monitoring_endpoint =
            std::env::var("BEARDOG_MONITORING_ENDPOINT").unwrap_or_else(|_| {
                let base_port = Self::get_base_port_for_security_level(&context.security_level);
                format!("http://localhost:{}", base_port + 1000)
            });

        let api_key = std::env::var("BEARDOG_API_KEY").ok();

        let timeout_secs = std::env::var("BEARDOG_TIMEOUT_SECS")
            .unwrap_or("30".to_string())
            .parse()
            .unwrap_or(30);

        let max_retries = std::env::var("BEARDOG_MAX_RETRIES")
            .unwrap_or("3".to_string())
            .parse()
            .unwrap_or(3);

        Ok(Self {
            endpoint,
            monitoring_endpoint,
            api_key,
            timeout_secs,
            max_retries,
            security_level: context.security_level.clone(),
        })
    }

    /// Get base port for security level
    fn get_base_port_for_security_level(security_level: &SecurityLevel) -> u16 {
        match security_level {
            SecurityLevel::Basic => 8400,
            SecurityLevel::Standard => 8443,
            SecurityLevel::High => 8500,
            SecurityLevel::Maximum => 8600,
        }
    }
}

impl Default for BearDogConfig {
    fn default() -> Self {
        Self {
            endpoint: "https://localhost:8443".to_string(),
            monitoring_endpoint: "http://localhost:9090".to_string(),
            api_key: None,
            timeout_secs: 30,
            max_retries: 3,
            security_level: SecurityLevel::Standard,
        }
    }
}

/// Authentication response from BearDog
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthResponse {
    /// Whether authentication was successful
    pub success: bool,
    /// The authenticated user ID if successful
    pub user_id: Option<String>,
    /// The authentication token if successful
    pub token: Option<String>,
    /// Token expiration time if successful
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

// Add default implementations for PrimalContext and other traits
impl Default for PrimalContext {
    fn default() -> Self {
        use crate::traits::NetworkLocation;

        Self {
            user_id: "default-user".to_string(),
            device_id: "default-device".to_string(),
            session_id: uuid::Uuid::new_v4().to_string(),
            network_location: NetworkLocation {
                ip_address: "127.0.0.1".to_string(),
                subnet: None,
                network_id: None,
                geo_location: None,
            },
            security_level: SecurityLevel::Standard,
            metadata: HashMap::new(),
        }
    }
}

impl PartialOrd for SecurityLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SecurityLevel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_level = match self {
            SecurityLevel::Basic => 0,
            SecurityLevel::Standard => 1,
            SecurityLevel::High => 2,
            SecurityLevel::Maximum => 3,
        };

        let other_level = match other {
            SecurityLevel::Basic => 0,
            SecurityLevel::Standard => 1,
            SecurityLevel::High => 2,
            SecurityLevel::Maximum => 3,
        };

        self_level.cmp(&other_level)
    }
}

impl Default for BearDogPrimal {
    fn default() -> Self {
        Self::new()
    }
}
