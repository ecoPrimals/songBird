//! Primal Capability Registry
//!
//! This module provides capability-based discovery and routing for primals.
//! Unlike the plugin registry (which manages internal plugins), this registry
//! tracks **external primals** (security provider, ToadStool, Gorilla, etc.) and routes
//! requests based on the capabilities they provide.
//!
//! ## Architecture
//!
//! ```text
//! Capability Request: "Who provides 'security'?"
//!                     ↓
//!         ┌─────────────────────┐
//!         │  Primal Registry    │
//!         │  (Songbird)         │
//!         └──────────┬──────────┘
//!                    │
//!     ┌──────────────┼──────────────┐
//!     │              │              │
//! ┌───┴────┐    ┌───┴────┐    ┌───┴────┐
//! │security provider │    │ToadStol│    │Gorilla │
//! │        │    │        │    │        │
//! │security│    │storage │    │compute │
//! └────────┘    └────────┘    └────────┘
//! ```
//!
//! ## Key Principles
//!
//! 1. **Capability-Based** - Routes by what a primal *does*, not what it *is*
//! 2. **Zero Hardcoding** - No compile-time knowledge of specific primals
//! 3. **Dynamic Discovery** - Primals register at runtime
//! 4. **Linear Scaling** - n primals = n registrations (not n²!)
//!
//! ## Usage
//!
//! ```rust,no_run
//! use songbird_orchestrator::ipc::primal_registry::{PrimalRegistry, PrimalInfo};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let mut registry = PrimalRegistry::new();
//!
//! // Register a primal
//! registry.register(PrimalInfo {
//!     primal_id: "security provider-tower1".to_string(),
//!     capabilities: vec!["security".to_string(), "encryption".to_string()],
//!     endpoint: Some("http://localhost:9000".to_string()),
//!     metadata: Default::default(),
//! }).await?;
//!
//! // Find provider by capability
//! let security_provider = registry.get_provider("security").await?;
//! println!("Security provider: {:?}", security_provider);
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// Information about a registered primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalInfo {
    /// Unique primal identifier (e.g., "security provider-tower1", "toadstool-main")
    pub primal_id: String,
    
    /// Capabilities this primal provides (e.g., ["security", "encryption", "trust"])
    pub capabilities: Vec<String>,
    
    /// Endpoint for communication (e.g., Unix socket path, HTTP URL)
    pub endpoint: Option<String>,
    
    /// Additional metadata (version, health status, etc.)
    #[serde(default)]
    pub metadata: serde_json::Map<String, serde_json::Value>,
}

/// Capability-based registry for primals
///
/// Tracks which primals provide which capabilities and enables
/// capability-based routing (avoiding n² connection complexity).
pub struct PrimalRegistry {
    /// Map: primal_id -> PrimalInfo
    primals: HashMap<String, PrimalInfo>,
    
    /// Map: capability -> Vec<primal_id>
    capabilities: HashMap<String, Vec<String>>,
}

impl PrimalRegistry {
    /// Create a new empty primal registry
    pub fn new() -> Self {
        info!("📋 Creating new primal registry");
        Self {
            primals: HashMap::new(),
            capabilities: HashMap::new(),
        }
    }
    
    /// Register a primal with its capabilities
    ///
    /// # Arguments
    ///
    /// * `info` - Primal information including ID, capabilities, and endpoint
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use songbird_orchestrator::ipc::primal_registry::{PrimalRegistry, PrimalInfo};
    /// # async fn example() -> anyhow::Result<()> {
    /// let mut registry = PrimalRegistry::new();
    ///
    /// registry.register(PrimalInfo {
    ///     primal_id: "security provider-tower1".to_string(),
    ///     capabilities: vec!["security".to_string(), "encryption".to_string()],
    ///     endpoint: Some("/tmp/security provider.sock".to_string()),
    ///     metadata: Default::default(),
    /// }).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn register(&mut self, info: PrimalInfo) -> Result<()> {
        let primal_id = info.primal_id.clone();
        
        info!("📝 Registering primal: {}", primal_id);
        debug!("   Capabilities: {:?}", info.capabilities);
        if let Some(ref endpoint) = info.endpoint {
            debug!("   Endpoint: {}", endpoint);
        }
        
        // Register each capability
        for capability in &info.capabilities {
            self.capabilities
                .entry(capability.clone())
                .or_insert_with(Vec::new)
                .push(primal_id.clone());
            
            debug!("   Registered capability: {}", capability);
        }
        
        // Store primal info
        self.primals.insert(primal_id.clone(), info);
        
        info!("✅ Primal registered: {} (provides {} capabilities)", 
              primal_id, 
              self.primals.get(&primal_id).map(|p| p.capabilities.len()).unwrap_or(0));
        
        Ok(())
    }
    
    /// Unregister a primal
    ///
    /// Removes the primal and all its capability associations.
    ///
    /// # Arguments
    ///
    /// * `primal_id` - ID of the primal to unregister
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use songbird_orchestrator::ipc::primal_registry::PrimalRegistry;
    /// # async fn example() -> anyhow::Result<()> {
    /// let mut registry = PrimalRegistry::new();
    /// registry.unregister("security provider-tower1").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn unregister(&mut self, primal_id: &str) -> Result<()> {
        info!("📤 Unregistering primal: {}", primal_id);
        
        // Get primal info
        let info = self.primals.remove(primal_id)
            .context(format!("Primal not found: {}", primal_id))?;
        
        // Remove from capability mappings
        for capability in &info.capabilities {
            if let Some(providers) = self.capabilities.get_mut(capability) {
                providers.retain(|id| id != primal_id);
                
                // Remove capability entry if no more providers
                if providers.is_empty() {
                    self.capabilities.remove(capability);
                }
            }
        }
        
        info!("✅ Primal unregistered: {}", primal_id);
        
        Ok(())
    }
    
    /// Get the first provider for a capability
    ///
    /// Returns `None` if no primal provides this capability.
    ///
    /// # Arguments
    ///
    /// * `capability` - The capability to search for (e.g., "security", "storage")
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use songbird_orchestrator::ipc::primal_registry::PrimalRegistry;
    /// # async fn example() -> anyhow::Result<()> {
    /// let registry = PrimalRegistry::new();
    ///
    /// if let Some(provider) = registry.get_provider("security").await? {
    ///     println!("Security provider: {}", provider.primal_id);
    ///     println!("Endpoint: {:?}", provider.endpoint);
    /// } else {
    ///     println!("No security provider found");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_provider(&self, capability: &str) -> Result<Option<PrimalInfo>> {
        debug!("🔍 Looking for provider of capability: {}", capability);
        
        // Get first provider for this capability
        if let Some(providers) = self.capabilities.get(capability) {
            if let Some(primal_id) = providers.first() {
                if let Some(info) = self.primals.get(primal_id) {
                    debug!("✅ Found provider: {} (endpoint: {:?})", 
                           info.primal_id, info.endpoint);
                    return Ok(Some(info.clone()));
                }
            }
        }
        
        debug!("❌ No provider found for capability: {}", capability);
        Ok(None)
    }
    
    /// List all providers for a capability
    ///
    /// Returns an empty vector if no primal provides this capability.
    ///
    /// # Arguments
    ///
    /// * `capability` - The capability to search for
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use songbird_orchestrator::ipc::primal_registry::PrimalRegistry;
    /// # async fn example() -> anyhow::Result<()> {
    /// let registry = PrimalRegistry::new();
    ///
    /// let providers = registry.list_providers("storage").await?;
    /// for provider in providers {
    ///     println!("Storage provider: {} at {:?}", provider.primal_id, provider.endpoint);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_providers(&self, capability: &str) -> Result<Vec<PrimalInfo>> {
        debug!("📋 Listing all providers of capability: {}", capability);
        
        let mut providers = Vec::new();
        
        if let Some(primal_ids) = self.capabilities.get(capability) {
            for primal_id in primal_ids {
                if let Some(info) = self.primals.get(primal_id) {
                    providers.push(info.clone());
                }
            }
        }
        
        debug!("✅ Found {} providers for capability: {}", providers.len(), capability);
        
        Ok(providers)
    }
    
    /// List all registered primals
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use songbird_orchestrator::ipc::primal_registry::PrimalRegistry;
    /// # async fn example() -> anyhow::Result<()> {
    /// let registry = PrimalRegistry::new();
    ///
    /// let primals = registry.list_all().await?;
    /// for primal in primals {
    ///     println!("Primal: {} provides {:?}", primal.primal_id, primal.capabilities);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_all(&self) -> Result<Vec<PrimalInfo>> {
        debug!("📋 Listing all registered primals");
        
        let primals: Vec<PrimalInfo> = self.primals.values().cloned().collect();
        
        debug!("✅ Total registered primals: {}", primals.len());
        
        Ok(primals)
    }
    
    /// Get information about a specific primal
    ///
    /// # Arguments
    ///
    /// * `primal_id` - ID of the primal to query
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use songbird_orchestrator::ipc::primal_registry::PrimalRegistry;
    /// # async fn example() -> anyhow::Result<()> {
    /// let registry = PrimalRegistry::new();
    ///
    /// if let Some(info) = registry.get_primal("security provider-tower1").await? {
    ///     println!("Primal: {}", info.primal_id);
    ///     println!("Capabilities: {:?}", info.capabilities);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_primal(&self, primal_id: &str) -> Result<Option<PrimalInfo>> {
        Ok(self.primals.get(primal_id).cloned())
    }
    
    /// Get all capabilities currently available
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use songbird_orchestrator::ipc::primal_registry::PrimalRegistry;
    /// # async fn example() -> anyhow::Result<()> {
    /// let registry = PrimalRegistry::new();
    ///
    /// let capabilities = registry.available_capabilities().await?;
    /// println!("Available capabilities: {:?}", capabilities);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn available_capabilities(&self) -> Result<Vec<String>> {
        Ok(self.capabilities.keys().cloned().collect())
    }
    
    /// Get statistics about the registry
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use songbird_orchestrator::ipc::primal_registry::PrimalRegistry;
    /// # async fn example() -> anyhow::Result<()> {
    /// let registry = PrimalRegistry::new();
    ///
    /// let stats = registry.stats().await?;
    /// println!("Total primals: {}", stats.total_primals);
    /// println!("Total capabilities: {}", stats.total_capabilities);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn stats(&self) -> Result<RegistryStats> {
        Ok(RegistryStats {
            total_primals: self.primals.len(),
            total_capabilities: self.capabilities.len(),
            primals_per_capability: self.capabilities
                .iter()
                .map(|(cap, providers)| (cap.clone(), providers.len()))
                .collect(),
        })
    }
}

/// Registry statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStats {
    /// Total number of registered primals
    pub total_primals: usize,
    
    /// Total number of available capabilities
    pub total_capabilities: usize,
    
    /// Number of primals providing each capability
    pub primals_per_capability: HashMap<String, usize>,
}

impl Default for PrimalRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_primal(id: &str, capabilities: Vec<&str>) -> PrimalInfo {
        PrimalInfo {
            primal_id: id.to_string(),
            capabilities: capabilities.iter().map(|s| s.to_string()).collect(),
            endpoint: Some(format!("/tmp/{}.sock", id)),
            metadata: Default::default(),
        }
    }
    
    #[tokio::test]
    async fn test_register_primal() {
        let mut registry = PrimalRegistry::new();
        
        let primal = create_test_primal("security provider", vec!["security", "encryption"]);
        registry.register(primal).await.unwrap();
        
        let all = registry.list_all().await.unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].primal_id, "security provider");
    }
    
    #[tokio::test]
    async fn test_get_provider() {
        let mut registry = PrimalRegistry::new();
        
        let primal = create_test_primal("security provider", vec!["security", "encryption"]);
        registry.register(primal).await.unwrap();
        
        let provider = registry.get_provider("security").await.unwrap();
        assert!(provider.is_some());
        assert_eq!(provider.unwrap().primal_id, "security provider");
        
        let no_provider = registry.get_provider("storage").await.unwrap();
        assert!(no_provider.is_none());
    }
    
    #[tokio::test]
    async fn test_list_providers() {
        let mut registry = PrimalRegistry::new();
        
        registry.register(create_test_primal("security provider1", vec!["security"])).await.unwrap();
        registry.register(create_test_primal("security provider2", vec!["security"])).await.unwrap();
        registry.register(create_test_primal("toadstool", vec!["storage"])).await.unwrap();
        
        let security_providers = registry.list_providers("security").await.unwrap();
        assert_eq!(security_providers.len(), 2);
        
        let storage_providers = registry.list_providers("storage").await.unwrap();
        assert_eq!(storage_providers.len(), 1);
    }
    
    #[tokio::test]
    async fn test_unregister() {
        let mut registry = PrimalRegistry::new();
        
        registry.register(create_test_primal("security provider", vec!["security"])).await.unwrap();
        assert_eq!(registry.list_all().await.unwrap().len(), 1);
        
        registry.unregister("security provider").await.unwrap();
        assert_eq!(registry.list_all().await.unwrap().len(), 0);
        
        let provider = registry.get_provider("security").await.unwrap();
        assert!(provider.is_none());
    }
    
    #[tokio::test]
    async fn test_available_capabilities() {
        let mut registry = PrimalRegistry::new();
        
        registry.register(create_test_primal("security provider", vec!["security", "encryption"])).await.unwrap();
        registry.register(create_test_primal("toadstool", vec!["storage"])).await.unwrap();
        
        let capabilities = registry.available_capabilities().await.unwrap();
        assert_eq!(capabilities.len(), 3);
        assert!(capabilities.contains(&"security".to_string()));
        assert!(capabilities.contains(&"encryption".to_string()));
        assert!(capabilities.contains(&"storage".to_string()));
    }
    
    #[tokio::test]
    async fn test_stats() {
        let mut registry = PrimalRegistry::new();
        
        registry.register(create_test_primal("security provider", vec!["security"])).await.unwrap();
        registry.register(create_test_primal("toadstool", vec!["storage"])).await.unwrap();
        
        let stats = registry.stats().await.unwrap();
        assert_eq!(stats.total_primals, 2);
        assert_eq!(stats.total_capabilities, 2);
        assert_eq!(*stats.primals_per_capability.get("security").unwrap(), 1);
        assert_eq!(*stats.primals_per_capability.get("storage").unwrap(), 1);
    }
}

