//! Encrypted Snapshots with Universal Security Integration
//!
//! This module provides encrypted snapshot functionality using the universal
//! capability adapter system for security primal discovery and routing.

use songbird_universal::capabilities::UniversalCapabilityAdapter;
use songbird_config::config::constants::get_primal_endpoint;
use std::collections::HashMap;
use tracing::{debug, info, warn, error};

/// Encrypted snapshots manager with universal security capabilities
pub struct EncryptedSnapshotManager {
    /// Universal capability adapter for security primal discovery
    capability_adapter: UniversalCapabilityAdapter,
    
    /// Active security clients for encryption operations
    security_clients: HashMap<String, Box<dyn SecurityClient>>,
    
    /// Last security capability refresh
    last_security_refresh: Option<chrono::DateTime<chrono::Utc>>,
}

/// Universal security client trait
pub trait SecurityClient: Send + Sync {
    async fn encrypt(&self, data: &[u8], context: &EncryptionContext) -> Result<Vec<u8>, SnapshotError>;
    async fn decrypt(&self, data: &[u8], context: &EncryptionContext) -> Result<Vec<u8>, SnapshotError>;
    async fn generate_key(&self, key_spec: &KeySpec) -> Result<String, SnapshotError>;
    async fn health_check(&self) -> Result<bool, SnapshotError>;
    fn endpoint(&self) -> &str;
}

/// Generic HTTP security client for universal integration
pub struct HttpSecurityClient {
    endpoint: String,
    client: reqwest::Client,
    primal_type: String,
}

impl HttpSecurityClient {
    pub fn new(endpoint: String, primal_type: String) -> Self {
        Self {
            endpoint,
            client: reqwest::Client::new(),
            primal_type,
        }
    }
}

impl SecurityClient for HttpSecurityClient {
    async fn encrypt(&self, data: &[u8], context: &EncryptionContext) -> Result<Vec<u8>, SnapshotError> {
        debug!("🔐 Encrypting data using {} security primal", self.primal_type);
        
        let request = serde_json::json!({
            "action": "encrypt",
            "data": base64::encode(data),
            "context": context,
        });
        
        let response = self.client
            .post(&format!("{}/api/crypto/encrypt", self.endpoint))
            .json(&request)
            .send()
            .await
            .map_err(|e| SnapshotError::SecurityError(e.to_string()))?;
            
        if response.status().is_success() {
            let result: serde_json::Value = response.json().await
                .map_err(|e| SnapshotError::ParseError(e.to_string()))?;
                
            let encrypted_b64 = result.get("encrypted_data")
                .and_then(|v| v.as_str())
                .ok_or_else(|| SnapshotError::ParseError("Missing encrypted_data field".to_string()))?;
                
            let encrypted = base64::decode(encrypted_b64)
                .map_err(|e| SnapshotError::ParseError(e.to_string()))?;
                
            debug!("✅ Data encrypted successfully using {}", self.primal_type);
            Ok(encrypted)
        } else {
            let error_msg = format!("Encryption failed with status: {}", response.status());
            error!("❌ {} encryption failed: {}", self.primal_type, error_msg);
            Err(SnapshotError::SecurityError(error_msg))
        }
    }
    
    async fn decrypt(&self, data: &[u8], context: &EncryptionContext) -> Result<Vec<u8>, SnapshotError> {
        debug!("🔓 Decrypting data using {} security primal", self.primal_type);
        
        let request = serde_json::json!({
            "action": "decrypt",
            "encrypted_data": base64::encode(data),
            "context": context,
        });
        
        let response = self.client
            .post(&format!("{}/api/crypto/decrypt", self.endpoint))
            .json(&request)
            .send()
            .await
            .map_err(|e| SnapshotError::SecurityError(e.to_string()))?;
            
        if response.status().is_success() {
            let result: serde_json::Value = response.json().await
                .map_err(|e| SnapshotError::ParseError(e.to_string()))?;
                
            let decrypted_b64 = result.get("decrypted_data")
                .and_then(|v| v.as_str())
                .ok_or_else(|| SnapshotError::ParseError("Missing decrypted_data field".to_string()))?;
                
            let decrypted = base64::decode(decrypted_b64)
                .map_err(|e| SnapshotError::ParseError(e.to_string()))?;
                
            debug!("✅ Data decrypted successfully using {}", self.primal_type);
            Ok(decrypted)
        } else {
            let error_msg = format!("Decryption failed with status: {}", response.status());
            error!("❌ {} decryption failed: {}", self.primal_type, error_msg);
            Err(SnapshotError::SecurityError(error_msg))
        }
    }
    
    async fn generate_key(&self, key_spec: &KeySpec) -> Result<String, SnapshotError> {
        debug!("🔑 Generating key using {} security primal", self.primal_type);
        
        let request = serde_json::json!({
            "action": "generate_key",
            "key_spec": key_spec,
        });
        
        let response = self.client
            .post(&format!("{}/api/crypto/generate_key", self.endpoint))
            .json(&request)
            .send()
            .await
            .map_err(|e| SnapshotError::SecurityError(e.to_string()))?;
            
        if response.status().is_success() {
            let result: serde_json::Value = response.json().await
                .map_err(|e| SnapshotError::ParseError(e.to_string()))?;
                
            let key_handle = result.get("key_handle")
                .and_then(|v| v.as_str())
                .ok_or_else(|| SnapshotError::ParseError("Missing key_handle field".to_string()))?;
                
            debug!("✅ Key generated successfully using {}", self.primal_type);
            Ok(key_handle.to_string())
        } else {
            let error_msg = format!("Key generation failed with status: {}", response.status());
            error!("❌ {} key generation failed: {}", self.primal_type, error_msg);
            Err(SnapshotError::SecurityError(error_msg))
        }
    }
    
    async fn health_check(&self) -> Result<bool, SnapshotError> {
        match self.client.get(&format!("{}/health", self.endpoint)).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
    
    fn endpoint(&self) -> &str {
        &self.endpoint
    }
}

impl EncryptedSnapshotManager {
    /// Create new encrypted snapshot manager with universal security discovery
    pub async fn new() -> Result<Self, SnapshotError> {
        info!("🚀 Initializing Encrypted Snapshot Manager with universal security system");
        
        let discovery_config = songbird_universal::capabilities::DiscoveryConfig::default();
        let capability_adapter = UniversalCapabilityAdapter::new(discovery_config);
        
        let mut manager = Self {
            capability_adapter,
            security_clients: HashMap::new(),
            last_security_refresh: None,
        };
        
        // Discover security primals (replaces hardcoded beardog_user)
        manager.refresh_security_capabilities().await?;
        
        info!("✅ Encrypted Snapshot Manager initialized with {} security endpoints", 
              manager.security_clients.len());
        
        Ok(manager)
    }
    
    /// Refresh security capabilities discovery
    async fn refresh_security_capabilities(&mut self) -> Result<(), SnapshotError> {
        info!("🔍 Discovering security capability primals...");
        
        // Find all primals with security capabilities
        let security_primals = self.capability_adapter.find_capability_providers("security").await;
        
        self.security_clients.clear();
        
        for primal_name in security_primals {
            let endpoint = get_primal_endpoint(&primal_name);
            debug!("Found security primal: {} at {}", primal_name, endpoint);
            
            // Create client for this primal
            let client = HttpSecurityClient::new(endpoint.clone(), primal_name.clone());
            
            // Test connectivity
            if client.health_check().await.unwrap_or(false) {
                self.security_clients.insert(primal_name.clone(), Box::new(client));
                info!("✅ Connected to security primal: {}", primal_name);
            } else {
                warn!("⚠️ Could not connect to security primal: {}", primal_name);
            }
        }
        
        // Fallback: Try traditional beardog if no security primals found
        if self.security_clients.is_empty() {
            info!("🔄 No security primals discovered, trying beardog fallback...");
            
            let beardog_endpoint = get_primal_endpoint("beardog");
            let beardog_client = HttpSecurityClient::new(beardog_endpoint.clone(), "beardog".to_string());
            
            if beardog_client.health_check().await.unwrap_or(false) {
                self.security_clients.insert("beardog".to_string(), Box::new(beardog_client));
                info!("✅ Connected to beardog security fallback");
            }
        }
        
        self.last_security_refresh = Some(chrono::Utc::now());
        
        if self.security_clients.is_empty() {
            warn!("⚠️ No security capabilities available - encryption operations will fail");
            return Err(SnapshotError::SecurityError("No security capabilities available".to_string()));
        }
        
        Ok(())
    }
    
    /// Get the best available security client
    async fn get_security_client(&self) -> Option<&Box<dyn SecurityClient>> {
        // Return the first healthy client
        for (primal_name, client) in &self.security_clients {
            if client.health_check().await.unwrap_or(false) {
                debug!("Using security client: {}", primal_name);
                return Some(client);
            }
        }
        None
    }
    
    /// Encrypt data using universal security capabilities (replaces beardog_user.encrypt)
    pub async fn encrypt(&self, data: &[u8], context: &EncryptionContext) -> Result<Vec<u8>, SnapshotError> {
        debug!("🔐 Encrypting data using universal security capabilities");
        
        if let Some(security_client) = self.get_security_client().await {
            return security_client.encrypt(data, context).await;
        }
        
        Err(SnapshotError::SecurityError("No security capabilities available for encryption".to_string()))
    }
    
    /// Generate key using universal security capabilities (replaces beardog_user.generate_key)
    pub async fn generate_key(&self, key_spec: &KeySpec) -> Result<String, SnapshotError> {
        debug!("🔑 Generating key using universal security capabilities");
        
        if let Some(security_client) = self.get_security_client().await {
            return security_client.generate_key(key_spec).await;
        }
        
        Err(SnapshotError::SecurityError("No security capabilities available for key generation".to_string()))
    }
}

/// Error types for snapshot operations
#[derive(Debug)]
pub enum SnapshotError {
    SecurityError(String),
    ParseError(String),
    NetworkError(String),
    CapabilityNotFound(String),
}

impl std::fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SnapshotError::SecurityError(msg) => write!(f, "Security error: {}", msg),
            SnapshotError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            SnapshotError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            SnapshotError::CapabilityNotFound(cap) => write!(f, "Capability not found: {}", cap),
        }
    }
}

impl std::error::Error for SnapshotError {}

// Placeholder types for compilation
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EncryptionContext {
    pub algorithm: String,
    pub key_id: Option<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct KeySpec {
    pub algorithm: String,
    pub key_size: u32,
    pub purpose: String,
}
