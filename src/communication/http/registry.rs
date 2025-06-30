// Module imports
//! HTTP Service Registry
//!
//! Service discovery and registration for HTTP communication

use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;
use crate::errors::Result;
/// Service registry trait for HTTP communication
#[async_trait]
pub trait ServiceRegistry: Send + Sync {
    async fn get_service_endpoint(&self, service_id: &str) -> Result<Option<String>>;
    async fn get_service_info(&self, service_id: &str) -> Result<Option<crate::traits::service::ServiceInfo>>;
    async fn get_all_endpoints(&self) -> Vec<(String, String)>;
}
/// Simple HTTP service registry implementation
pub struct HttpServiceRegistry {
    /// Map of service_id -> endpoint URL
    service_endpoints: Arc<DashMap<String, String>>,
    /// Map of service_id -> ServiceInfo
    service_info: Arc<DashMap<String, crate::traits::service::ServiceInfo>>,
impl HttpServiceRegistry {
    pub fn new() -> Self {
        Self {
            service_endpoints: Arc::new(DashMap::new()),
            service_info: Arc::new(DashMap::new()),
        }
    }
    /// Register a service endpoint
    pub fn register_service_endpoint(&self, service_id: String, endpoint: String) {
        tracing::debug!(
            service_id = %service_id,
            endpoint = %endpoint,
            "Registering service endpoint"
        );
        self.service_endpoints.insert(service_id, endpoint);
    /// Register service info
    pub fn register_service_info(&self, service_info: crate::traits::service::ServiceInfo) {
        let service_id = service_info.id.clone();
            service_type = %service_info.service_type,
            "Registering service info"
        self.service_info.insert(service_id, service_info);
    /// Unregister a service
    pub fn unregister_service(&self, service_id: &str) {
        tracing::debug!(service_id = service_id, "Unregistering service");
        self.service_endpoints.remove(service_id);
        self.service_info.remove(service_id);
    /// Get all registered service endpoints
    pub fn get_all_endpoints(&self) -> Vec<(String, String)> {
        self.service_endpoints
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
impl ServiceRegistry for HttpServiceRegistry {
    async fn get_service_endpoint(&self, service_id: &str) -> Result<Option<String>> {
        Ok(self.service_endpoints.get(service_id).map(|e| e.value().clone()))
    async fn get_service_info(&self, service_id: &str) -> Result<Option<crate::traits::service::ServiceInfo>> {
        Ok(self.service_info.get(service_id).map(|info| info.value().clone()))
    async fn get_all_endpoints(&self) -> Vec<(String, String)> {
} 
