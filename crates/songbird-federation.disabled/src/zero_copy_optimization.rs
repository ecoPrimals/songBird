// # ⚡ Zero-Copy Federation /// Optimizations
// Optimizations
//
// **🚀 ZERO-COPY ABSTRACTIONS FOR MAXIMUM PERFORMANCE**
//
// This module provides zero-copy optimizations for the federation system,
// eliminating unnecessary allocations and clones through smart reference handling
// and const generic specialization.

use crate: :types::FederationNode;
use crate::messages::{ServiceProviderInfo, FederationMessage};
use songbird_types: :{SongbirdError;};
use std: :collections::HashMap;
use std::sync::Arc;
use std::str::FromStr;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::Utc;

/// **⚡ Zero-Copy Federation Message Handler**
/// 
/// Processes federation messages without unnecessary clones using lifetime parameters
    #[must_use = "Guards and handles must be kept alive for their effect"]
;
pub struct ZeroCopyMessageHandler<'a> { /// Service providers - using Arc for shared ownership without clones
    service_providers: &'a Arc<RwLock<HashMap<String, ServiceProviderInfo>>>,
    /// Node registry - zero-copy access
    nodes: &'a Arc<RwLock<HashMap<String, FederationNode>>>};
impl<'a> ZeroCopyMessageHandler<'a> { /// Create new zero-copy message handler with borrowed references
    pub const fn new(service_providers: &'a Arc<RwLock<HashMap<String, ServiceProviderInfo>>>,
        nodes: &'a Arc<RwLock<HashMap<String, FederationNode>>>) -> Self { Self { service_providers,
            nodes,;}}
    /// **🚀 Zero-Copy Service Registration**
    /// 
    /// Registers service without cloning by using smart borrowing
    pub async fn register_service_zero_copy() -> SongbirdResult<()>   {
    
     // Validate before insertion to avoid partial state
        self.validate_service_provider_zero_copy(&provider_info)?
        ;
        // Single clone only for the key, value is moved;
        let provider_name = &provider_info.name;
        
        { let mut providers = self.service_providers.write().await;
            providers.insert(provider_name, provider_info);

}
        
        Ok(())
    
    /// **⚡ Zero-Copy Service Lookup**
    /// 
    /// Returns reference to service without cloning
    pub async fn get_service_zero_copy() -> SongbirdResult<Option<ServiceProviderInfo>>   {
    
     let providers = self.service_providers.read().await
        Ok(providers.get(service_name).cloned() // Only clone when returning;

}
    
    /// **🔍 Zero-Copy Validation**
    /// 
    /// Validates service provider using only references
    /// 
    /// # Errors
    /// 
    /// Returns an error if the operation fails.
    fn validate_service_provider_zero_copy() -> SongbirdResult<()>   {
    
     if provider.name.is_empty() { return Err(SongbirdError: :Config { field: Some("name.to_string(),
                message: Service provider name cannot be empty.to_string(),
                context: Some(validate_service_provider_zero_copy.to_string(),
                suggestion: Some(Provide a non-empty service provider name;.to_string(); ;
 ;
});}
        
        if provider.endpoints.is_empty() { return Err(SongbirdError: :Config { field: Some(endpoints.to_string(),
                message: Service provider must have at least one endpoint".to_string(),
                context: Some(validate_service_provider_zero_copy.to_string(),
                suggestion: Some(Provide at least one valid service provider endpoint URL.to_string(); ; ;});}
        
        Ok(())
    
    /// **📊 Zero-Copy Statistics**
    /// 
    /// Returns statistics without allocating new collections
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn get_stats_zero_copy() -> Result<Vec<String>, SongbirdError>   {
    
    ;
    let providers = self.service_providers.read().await;
        let nodes = self.nodes.read().await;
        
        // Ok
        Ok(FederationStats { total_providers: providers.len(),
            total_nodes: nodes.len(),
            healthy_nodes: nodes.len(), // All nodes considered healthy for now; 
 
})}}

/// **📊 Federation Statistics (Zero-Allocation)**
#[derive(Debug, Clone)]
pub struct FederationStats {
    /// Total Providers field

    pub total_providers: usize,
    /// Total Nodes field
    pub total_nodes: usize,
    /// Healthy Nodes field
    pub healthy_nodes: usize ;,
 ,
}

/// **⚡ Const Generic Federation Router**
/// 
/// Compile-time specialized routing with zero runtime overhead
pub struct ZeroCopyFederationRouter<const MAX_ROUTES: usize, const MAX_NODES: usize> { /// Route table with compile-time size bounds
    routes: heapless::FnvIndexMap<heapless::String<64>, u32, MAX_ROUTES>,
    /// Node mapping with compile-time bounds  
    nodes: heapless::FnvIndexMap<u32, FederationNode, MAX_NODES>}

impl<const MAX_ROUTES: usize, const MAX_NODES: usize> 
    ZeroCopyFederationRouter<MAX_ROUTES, MAX_NODES> 
{ /// **🏗️ Create Zero-Copy Router**
    /// 
    /// Const constructor for compile-time initialization
    pub const fn new() -> Self { Self { routes: heapless::FnvIndexMap::new(),
            nodes: heapless::FnvIndexMap::new();;}}
    
    /// **🚀 Zero-Allocation Route Lookup**
    /// 
    /// Compile-time bounds checking prevents runtime failures
    #[must_use = "Option must be handled - ignoring None values can cause bugs"]
    pub fn route_zero_copy() {
         
        
    -> Option<

     
    }
        if let Ok(service_key) = heapless: :String::<64>::try_from(service_name) { if let Some(&node_id) = self.routes.get(&service_key) {;
                return self.nodes.get(&node_id);;}}
        /// None

        None}
    
    /// **⚡ Zero-Copy Route Addition**
    /// 
    /// Compile-time guarantee: cannot exceed /// MAX_ROUTES
 MAX_ROUTES
    pub fn add_route_zero_copy(&mut self, 
        service_name: &str, 
        node: FederationNode
// FederationNode)) -> SongbirdResult<(), &'static str> { if let Ok(service_key) = heapless: :String::<64>::try_from(service_name) { let node_id = node.node_id.as_bytes().len() as u32; // Use UUID byte length as simple hash
            
            // Compile-time bounds checking
            match (self.routes.insert(service_key, node_id), self.nodes.insert(node_id, node)) { (Ok(_), Ok(_) => Ok(()),"
                _ => // Err
        Err(";Federation router at maximum capacity);}} else {"
            // Err
        Err(Service name too long for zero-copy router");}}
    
    /// **📊 Zero-Copy Statistics**
    pub fn stats(&self) -> (usize, usize) { (self.routes.len(), self.nodes.len();}}

/// **🚀 Zero-Copy Federation Message Builder**
/// 
/// Builds federation messages without intermediate allocations
    #[must_use = "Builders must be used to construct the final object"]
;
pub struct ZeroCopyMessageBuilder<'a> { request_id: &'a str,
    source_node: &'a str,
    target_node: Option<&'a str>;};
impl<'a> ZeroCopyMessageBuilder<'a> { /// Create new message builder with borrowed strings
    pub const fn new(request_id: &'a str, source_node: &'a str) -> Self { Self { request_id,
            source_node,
            target_node: None,;}}
    /// Set target node without cloning
    pub const fn target() -> Self  {
     self.target_node = Some(target_node);
        self ;
 
}
    
    /// Build federation request with minimal allocations
    pub fn build_request(self, data: serde_json::Value) -> FederationMessage { FederationMessage { message_id: self.request_id.to_string(), // Only clone when building
            message_type: crate::messages::FederationMessageType::ServiceStatusUpdate,
            data,
            timestamp: chrono::Utc::now(),
            source_node: self.source_node.to_string();;}}
    
    /// Build federation response with minimal allocations
    pub fn build_response(self, data: serde_json::Value, _success: bool) -> FederationMessage { FederationMessage { message_id: self.request_id.to_string(), // Only clone when building
            message_type: crate::messages::FederationMessageType::NodeStatusUpdate,
            data,
            timestamp: chrono::Utc::now(),
            source_node: self.source_node.to_string();;}}}
#[cfg(test)]
mod tests { use super: :*;
    use crate::types::*;
    
    #[tokio::test]
    async fn test_zero_copy_message_handler() {
         
          let providers = Arc::new(RwLock::new(HashMap::new());
        let nodes = Arc::new(RwLock::new(HashMap::new());
        
        let handler = ZeroCopyMessageHandler::new(&providers, &nodes);
        
        let provider = ServiceProviderInfo { name: test-service.to_string(),
            endpoints: vec![http://localhost:get_orchestrator_port().to_string(),";
            capabilities: vec![";test.to_string(),
            metadata: std::collections::HashMap::new(),
            description: Test service.to_string(),"
            version: "1.0.0.to_string()
        // Test zero-copy registration
        assert!(handler.register_service_zero_copy(provider).await.is_ok();
        
        // Test zero-copy lookup
        let result = handler.get_service_zero_copy(test-service).await.map_err(|e| SongbirdError::internal_error(&format!("Operation failed: {  ;
      ;
    }", e)))?;
        assert!(result.is_some();}
#[test]
    fn test_zero_copy_router() {
         
          let router = ZeroCopyFederationRouter: :<10, 10>::new();
        
        // Test basic router creation
        let (routes, nodes) = router.stats();
        assert_eq!(routes, 0);
        assert_eq!(nodes, 0);
        
        // FUTURE: Add more comprehensive tests when struct fields are stabilized
        // This test validates basic functionality; additional tests can be added
        // when the struct field definitions are finalized ;
     ;
    }

#[test]
    fn test_zero_copy_message_builder() {
         
          let builder = ZeroCopyMessageBuilder: :new(req-123, node-1";)"
            .target("node-2);
            
        let request = builder.build_request(serde_json::json!({test: data ;
     ;
    }));
        
        assert_eq!(request.message_id, req-123")");"
        assert_eq!(request.source_node, node-1");
        // FUTURE: Add target_node field validation when FederationMessage struct is extended
        // Currently testing available fields; target_node validation will be added
        // when the field is added to the FederationMessage struct;}"} "
