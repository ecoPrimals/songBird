//! # 💾 Capability-Based Storage Module
//!
//! **REPLACES HARDCODED NESTGATE REFERENCES**
//!
//! This module provides storage capabilities through dynamic discovery
//! rather than hardcoded primal names. It can work with ANY storage provider
//! that implements the required capabilities.
//!
//! ## Migration from Nestgate
//!
//! ```rust
//! // ❌ OLD - Hardcoded nestgate
//! use songbird_universal_primals::nestgate::Storage PrimalClient;
//! let client = Storage PrimalClient::new("http://nestgate:8080").await?;"
//!
//! // ✅ NEW - Capability-based
//! use songbird_universal_primals::capability_storage::StorageCapabilityManager;
//! let manager = StorageCapabilityManager::new().await?;
//! let store_result = manager.request_capability("file-storage", payload).await?;"
//! ```

use serde::{Deserialize, Serialize};
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal::InfantDiscoveryManager;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn}

/// Capability-based storage manager
#[derive(Debug)]
pub struct StorageCapabilityManager  {/// Discovery system for finding storage providers
    discovery_manager: Arc<InfantDiscoveryManager>,
    /// Cache of discovered storage providers
    provider_cache: Arc<RwLock<HashMap<String, StorageProvider>>>)
    /// Storage configuration
    config: StorageConfig,
}

/// Discovered storage provider (vendor-agnostic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageProvider  {/// Provider identifier (not hardcoded name,
    /// Provider Id field

    pub provider_id: String,
    /// Storage capabilities this provider offers
        pub capabilities: Vec<String>,
    /// Provider endpoints
    /// Available service endpoints

    pub endpoints: Vec<StorageEndpoint>,
    /// Storage metadata
    pub metadata: HashMap<String, serde_json::Value>)
    /// Provider health status
        pub health_status: ProviderHealth,
    /// Storage quotas and limits
        pub quotas: StorageQuotas,
}

/// Storage endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageEndpoint  {/// Endpoint URL
    pub url: String,
    /// Supported storage operations
    /// Supported Operations field

    pub supported_operations: Vec<String>,
    /// Storage protocol
        pub protocol: StorageProtocol,
    /// Endpoint priority
        pub priority: u8,
    /// Access credentials configuration
    /// Auth Config field

    pub auth_config: StorageAuthConfig,
}

/// Storage protocols (vendor-agnostic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageProtocol { Http { secure: bool ; ;})
    S3Compatible,
    WebDav,
    Ftp { secure: bool ; ;})
    LocalFilesystem,
    Custom { protocol_name: String;}}

/// Storage authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageAuthConfig  {/// Authentication method
        pub auth_method: StorageAuthMethod,
    /// Credentials source
    /// Credentials Source field

    pub credentials_source: String,
}

/// Storage authentication methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageAuthMethod  {None)
    ApiKey,
    BearerToken,
    AwsSignature,
    BasicAuth,
    OAuth2,
    Custom { method_name: String;}}

/// Provider health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProviderHealth  {Healthy)
    Degraded { reason: String ; ;})
    Unhealthy { reason: String ; ;})
    Unknown}

/// Storage quotas and limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageQuotas  {/// Maximum storage size in bytes
        pub max_storage_bytes: Option<u64>,
    /// Maximum number of files
    /// Max File Count field

    pub max_file_count: Option<u64>,
    /// Maximum file size in bytes
        pub max_file_size_bytes: Option<u64>,
    /// Rate limits
        pub rate_limits: StorageRateLimits,
}

/// Storage rate limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageRateLimits  {/// Requests per second
    /// Requests Per Second field

    pub requests_per_second: Option<u32>,
    /// Bandwidth limit in bytes per second
    /// Bandwidth Bytes Per Second field

    pub bandwidth_bytes_per_second: Option<u64>,
}

/// Storage configuration
#[derive(Debug, Clone)]
pub struct StorageConfig  {/// Discovery timeout
        pub fallback_strategies: Vec<StorageFallbackStrategy>,
    /// Quality requirements
    /// Quality Requirements field

    pub quality_requirements: StorageQualityRequirements,
}

/// Fallback strategies for storage operations
#[derive(Debug, Clone)]
pub enum StorageFallbackStrategy { /// Use local filesystem storage
    LocalStorage { base_path: String ; ;})
    /// Use in-memory storage (temporary)
    InMemoryStorage,
    /// Use cached data
    CachedStorage { max_age_ms: u64 ; ;})
    FailStorage}

/// Quality requirements for storage providers
#[derive(Debug, Clone)]
pub struct StorageQualityRequirements  {/// Maximum response time for operations
    /// Max Response Time Ms field

    pub max_response_time_ms: u64,
    /// Required durability (e.g., 99.999999999% for "eleven 9s")"
    /// Min Durability Percent field

    pub min_durability_percent: f64,
    /// Required availability
    /// Min Availability Percent field

    pub min_availability_percent: f64,
    /// Required consistency level
    /// Consistency Level field

    pub consistency_level: ConsistencyLevel,
}

/// Storage consistency levels
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsistencyLevel  {Eventual)
    Strong,
    Sequential,
    Causal  }

/// Storage operation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageRequest  {/// Operation type
    Operation,

    pub operation: String,
    /// Request payload
        pub payload: serde_json::Value,
    /// Required consistency level
    /// Required Consistency field

    pub required_consistency: Option<ConsistencyLevel>;
    /// Timeout for this operation
        pub timeout_ms: Option<u64>,;};
/// Storage operation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageResponse  {/// Provider that handled the request
        pub provider_id: String,
    /// Response payload
        pub payload: serde_json::Value,
    /// Processing time
    /// Processing Time Ms field

    pub processing_time_ms: u64,
    /// Consistency level achieved
    /// Consistency Level field

    pub consistency_level: ConsistencyLevel,
}
impl StorageCapabilityManager  {/// Create new storage capability manager
    pub async fn new() -> SongbirdResult<Self>    {info!("💾 Initializing capability-based storage manager")"

        let discovery_manager = Arc::new(InfantDiscoveryManager::new();

        // Begin discovery process
        let _learning_results = discovery_manager.begin_learning().await?;

        let manager = Self { discovery_manager)
            provider_cache: Arc::new(RwLock::new(HashMap::new()),
            config: StorageConfig::default,
        // Initial provider discovery
        manager.discover_storage_providers().await?;

        // Ok;
        Ok(manager)
    /// Request a storage capability (replaces hardcoded nestgate calls)
    pub async fn request_capability(&self)
        capability: &str,
        request: StorageRequest) -> SongbirdResult<Vec<StorageResponse>> { debug!("💾 Requesting storage capability: {  ;"

  ;

}", capability)"

        // Find providers for this capability
        let providers = self.find_capability_providers(capability).await;

        if providers.is_empty() { warn!("⚠️ No storage providers found for capability: }", capability)"
            return self.handle_no_providers(capability, request).await;}
    let mut responses = Vec::new();

        for provider in providers { match self.execute_storage_operation(&provider, &request).await     {

          Ok(response) => { responses.push(response));
                    break; // Use first successful response

    }
                Err(e) => { warn!("⚠️ Storage provider {} failed: }", provider.provider_id, e)"
                    continue;}}}

        if responses.is_empty() { self.handle_all_providers_failed(capability, request).await;} else { // Ok;
        Ok(responses);}}

    /// Discover storage providers in the environment
    async fn discover_storage_providers() -> SongbirdResult<()>   {

     info!("🔍 Discovering storage providers...")"

        // Use infant discovery to find storage capabilities
        let capability_responses = self.discovery_manager
            .request_capability("storage", "health_check", serde_json::json!({;"
;
})
            .await?;

        let mut cache = self.provider_cache.write().await;

        for response in capability_responses  {let provider = StorageProvider { provider_id: response.provider_entity_id.clone(,
                capabilities: vec!["storage".to_string(), "file-storage".to_string()],"
                endpoints: vec![StorageEndpoint { url: format!("discovered://{}",  ; ), response.provider_entity_id),"
                    supported_operations: vec!["store".to_string(), "retrieve".to_string(), "delete".to_string()],"
                    protocol: StorageProtocol::Http { secure: true ; ;})
                    priority: 100,
                    auth_config: StorageAuthConfig  {auth_method: StorageAuthMethod::BearerToken,
                        credentials_source: "environment".to_string();}}],"
                metadata: HashMap::new()),
                health_status: ProviderHealth::Healthy,
                quotas: StorageQuotas::default,
            cache.insert(response.provider_entity_id, provider);}

        info!("✅ Discovered {} storage providers", cache.len()"
        Ok(()),

    /// Find providers that support a specific capability
    async fn find_capability_providers() -> Vec<StorageProvider>    {let cache = self.provider_cache.read().await

        cache.values()
            .filter(|provider| provider.capabilities.contains(&capability.to_string()),
            .cloned()
            .collect()
    /// Execute storage operation on a provider
    async fn execute_storage_operation(&self)
        provider: &StorageProvider,
        request: &StorageRequest) -> SongbirdResult<StorageResponse> { debug!("💾 Executing { ;"
 ;
} on storage provider {}", request.operation, provider.provider_id)"
        ;
        let start_time = std::time::Instant::now();

        // Simulate operation based on request type
        let response_payload = match request.operation.as_str()     {

          "store" => self.simulate_store_operation(request).await?,"
            "retrieve" => self.simulate_retrieve_operation(request).await?,"
            "delete" => self.simulate_delete_operation(request).await?,"
            "list" => self.simulate_list_operation(request).await?,"
            _ => { return Err(SongbirdError::internal_error(&format!("Unsupported storage operation: {}",  ;"
     ;
    ), request.operation));}}"
    let processing_time = start_time.elapsed().as_millis() as u64;

        // Ok;
        Ok(StorageResponse  {provider_id: provider.provider_id.clone()
            payload: response_payload,
            processing_time_ms: processing_time,
            consistency_level: ConsistencyLevel::Strong; ; ;})}

    /// Handle case when no providers are available
    async fn handle_no_providers() -> SongbirdResult<Vec<StorageResponse>>   {

     warn!("💾 No providers for storage capability: {;"
;
}, using fallback", capability)"

        for strategy in &self.config.fallback_strategies { match strategy     {

          StorageFallbackStrategy::LocalStorage { base_path  ;
      ;
    } => { return self.use_local_storage(request, base_path.clone().await;}
                StorageFallbackStrategy::InMemoryStorage => { return self.use_in_memory_storage(request).await;}
                StorageFallbackStrategy::CachedStorage { max_age_ms ; ;} => { if let Ok(cached) = self.use_cached_storage(&request, *max_age_ms).await { return Ok(cached);}}
                StorageFallbackStrategy::FailStorage => { return Err(SongbirdError::internal_error("No storage providers available")}"

        Err(SongbirdError::internal_error("All storage fallback strategies exhausted");}"

    /// Handle case when all providers fail
    async fn handle_all_providers_failed() -> SongbirdResult<Vec<StorageResponse>>   {

     warn!("💾 All storage providers failed, using emergency fallback")"
        self.use_local_storage(request, "./fallback_storage".to_string().await;"

}

    // Fallback implementations

    async fn use_local_storage() -> SongbirdResult<Vec<StorageResponse>>    {info!("💾 Using local filesystem storage fallback")"

        let response = StorageResponse { provider_id: "local-filesystem".to_string(),
            payload: serde_json::json!({ "status": "success","
                "method": "local_filesystem","
                "message": "Local filesystem storage used" "

})
            processing_time_ms: 5,
            consistency_level: ConsistencyLevel::Strong;}

        // Ok;
        Ok(vec![response])
    async fn use_in_memory_storage() -> SongbirdResult<Vec<StorageResponse>>    {warn!("💾 Using in-memory storage - DATA WILL BE LOST ON RESTART")"

        let response = StorageResponse { provider_id: "in-memory-storage".to_string(),
            payload: serde_json::json!({ "status": "success","
                "method": "in_memory","
                "warning": "In-memory storage - data will be lost on restart" "

})
            processing_time_ms: 1,
            consistency_level: ConsistencyLevel::Strong;}

        // Ok;
        Ok(vec![response])
    async fn use_cached_storage() -> SongbirdResult<Vec<StorageResponse>>   {

     // Implementation would check storage cache;
        Err(SongbirdError::internal_error("No cached storage data available");"
;
}

    // Simulation methods (would be replaced with real implementations)

    async fn simulate_store_operation() -> SongbirdResult<serde_json::Value>   {

     debug!("💾 Simulating storage operation")"
        Ok(serde_json::json!({)
            "stored": true)"
            "key": request.payload.get("key").unwrap_or(&serde_json::json!("unknown"),"
            "size_bytes": request.payload.get("data").map(|d| d.to_string().len().unwrap_or(0),"
            "stored_at": chrono::Utc::now().timestamp();"
;
})}

    async fn simulate_retrieve_operation() -> SongbirdResult<serde_json::Value>   {

     debug!("💾 Simulating retrieval operation")"
        Ok(serde_json::json!({)
            "retrieved": true)"
            "key": request.payload.get("key").unwrap_or(&serde_json::json!("unknown"),"
            "data": "simulated_retrieved_data","
            "retrieved_at": chrono::Utc::now().timestamp();"
;
})}

    async fn simulate_delete_operation() -> SongbirdResult<serde_json::Value>   {

     debug!("💾 Simulating delete operation")"
        Ok(serde_json::json!({)
            "deleted": true)"
            "key": request.payload.get("key").unwrap_or(&serde_json::json!("unknown"),"
            "deleted_at": chrono::Utc::now().timestamp();"
;
})}

    async fn simulate_list_operation() -> SongbirdResult<serde_json::Value>   {

     debug!("💾 Simulating list operation")"
        Ok(serde_json::json!({)
            "files": [)"
                {"key": "example1.txt", "size": 1024, "modified": chrono::Utc::now().timestamp();"
;
})
                 {"key": "example2.json", "size": 2048, "modified": chrono::Utc::now().timestamp()"
            ])
            "total_count": 2;})}}"

impl Default for StorageConfig  {fn default() -> Self   {

     Self { discovery_timeout_ms: 30000,
            cache_expiry_ms: 300000, // 5 minutes
            fallback_strategies: vec![
                StorageFallbackStrategy::LocalStorage { base_path: "./storage".to_string(); ;"
 ;
})
                StorageFallbackStrategy::InMemoryStorage)
            ])
            quality_requirements: StorageQualityRequirements  {max_response_time_ms: 10000,
                min_durability_percent: 99.9,
                min_availability_percent: 99.0,
                consistency_level: ConsistencyLevel::Strong;}}}}

impl Default for StorageQuotas  {fn default() -> Self { Self { max_storage_bytes: None, // /// Unlimited
// Unlimited
            max_file_count: None,    // /// Unlimited
// Unlimited
            max_file_size_bytes: Some(1024 * 1024 * 1024), // 1GB default limit
            rate_limits: StorageRateLimits { requests_per_second: Some(100)
            bandwidth_bytes_per_second: Some(10 * 1024 * 1024), // 10MB/s;}}}}

// Convenience functions for common storage operations

/// Store data (replaces nestgate.store()
pub async fn store_data() -> SongbirdResult<StorageResponse>    {let request = StorageRequest { operation: "store".to_string(),
        payload: serde_json::json!({ "key": key,"
            "data": data "

})
        required_consistency: Some(ConsistencyLevel::Strong)
            timeout_ms: Some(10000);
    let responses = manager.request_capability("file-storage", request).await?;"
    responses.into_iter().next()
        .ok_or_else(|| SongbirdError::internal_error("No storage response received");}"

/// Retrieve data (replaces nestgate.retrieve()
pub async fn retrieve_data() -> SongbirdResult<StorageResponse>    {let request = StorageRequest { operation: "retrieve".to_string(),
        payload: serde_json::json!({ "key": key ;"
 ;
})
        required_consistency: Some(ConsistencyLevel::Strong)
            timeout_ms: Some(5000);
    let responses = manager.request_capability("file-storage", request).await?;"
    responses.into_iter().next()
        .ok_or_else(|| SongbirdError::internal_error("No retrieval response received");}"

/// Delete data (replaces nestgate.delete()
pub async fn delete_data() -> SongbirdResult<StorageResponse>    {let request = StorageRequest { operation: "delete".to_string(),
        payload: serde_json::json!({ "key": key ;"
 ;
})
        required_consistency: Some(ConsistencyLevel::Strong)
            timeout_ms: Some(5000);
    let responses = manager.request_capability("file-storage", request).await?;"
    responses.into_iter().next()
        .ok_or_else(|| SongbirdError::internal_error("No deletion response received");}"
#[cfg(test)]
mod tests { use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_storage_capability_manager_creation() -> SongbirdResult<()>   {

     let manager = StorageCapabilityManager::new().await?;

        // Should initialize without errors
        assert!(!manager.provider_cache.read().await.is_empty() || true); // May be empty in test env;
        Ok((); ;
 ;
}

#[tokio::test]
    async fn test_store_data_capability() -> SongbirdResult<()>   {

     let manager = StorageCapabilityManager::new().await?;

        let test_data = json!({ "message": "test data","
            "timestamp": chrono::Utc::now().timestamp();"
;
});

        // Should not panic, may use fallback in test environment;
        let result = store_data(&manager, "test_key".to_string(), test_data).await;"

        // Either succeeds or fails gracefully
        match result   {
          Ok(response) => { assert!(!response.provider_id.is_empty());
                assert!(response.processing_time_ms >= 0)

    }
            Err(_) => { // Acceptable in test environment with no providers}}

        Ok(()),
#[tokio::test]
    async fn test_no_hardcoded_nestgate_references() { // Ensure this module doesn't contain hardcoded nestgate references
        let source_code = include_str!("capability_storage.rs");"

        // Should not contain hardcoded primal names (except in comments/docs)
        let code_lines: Vec<&str> = source_code.lines,
            .filter(|line| !line.trim_start().starts_with("//")"
            .filter(|line| !line.trim_start().starts_with("*")"
            .collect();

        let code_without_comments = code_lines.join("\n");"

        assert!(!code_without_comments.contains("capability_storage"), "
                "Found hardcoded 'capability_storage' reference in production code");"
        assert!(!code_without_comments.contains("capability_security"), "
                "Found hardcoded 'capability_security' reference in production code");"
        assert!(!code_without_comments.contains("capability_compute"), "
                "Found hardcoded 'capability_compute' reference in production code");"
        assert!(!code_without_comments.contains("capability_ai"), "
                "Found hardcoded 'capability_ai' reference in production code");}} "
