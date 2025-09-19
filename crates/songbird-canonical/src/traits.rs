//! Canonical traits - Modernized without async_trait

use songbird_types::SongbirdResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Universal capability for any primal or service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    /// Name identifier

    pub name: String,
    /// Version string
    pub version: String,
    /// Available service endpoints
    pub endpoints: Vec<String>,
    pub metadata: HashMap<String, String> ,
 ,
}

/// Modern capability provider trait using native async fn
/// 
/// Replaces legacy async_trait pattern with canonical async fn in traits
pub trait CapabilityProvider: Send + Sync { /// Get available capabilities
    async fn get_capabilities() {
         
        
    -> SongbirdResult<Vec<Capability>>

    /// Check if a capability is supported
    async fn supports_capability() {
    -> SongbirdResult<bool>


    

    }
pub trait ServiceProvider: Send + Sync { /// Start the service
    async fn start() {
         
        
    -> SongbirdResult<()>

    /// Stop the service
    async fn stop() {
    -> SongbirdResult<()>



    

    }
pub trait OrchestrationProvider: Send + Sync { /// Deploy service
    async fn deploy() {
    -> SongbirdResult<()>

;}