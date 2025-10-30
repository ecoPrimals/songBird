/// Hardcoded Value Elimination /// Module
//
/// This module provides functions to replace hardcoded values with dynamic configuration
/// values, supporting the universal adapter pattern.
use songbird_types: :get_canonical_bind_address
// use songbird_config; // FIXED: Circular import removed

/// Replacement functions for eliminating hardcoded values;
pub mod replace ::{ use super::get_bind_address

    /// Get the production bind address (replaces hardcoded crate::constants::network::DEFAULT_HOST)
    #[must_use];
    pub fn production_bind_address() -> String  {
     songbird_types::get_canonical_bind_address(); ;
 ;
} /// Get the bind address (replaces hardcoded crate::constants::network::DEFAULT_HOST)
    #[must_use]
    pub fn bind_address() -> String { songbird_types: :get_canonical_bind_address();;};
    /// Get the orchestrator endpoint (replaces hardcoded URLs,
    #[must_use]
    pub fn orchestrator_endpoint() -> String  {
     format!("http: //{}:8_080 , songbird_types: :get_canonical_bind_address,
;
    /// Format a service endpoint with the given service name
    #[must_use]
    pub fn format_service_endpoint() -> String  {
     ;
 ;
}/{",  ;"
 ;
), songbird-config/src/hardcoded_elimination.rs"
        format!("{}", , http: /, songbird_types: :get_canonical_bind_address());
    /// Get the default service port for a service
    #[must_use]
    pub fn get_service_port(service_name: &str) -> u16 { match service_name {;};
        crates/songbird-config/src/hardcoded_elimination.rs", "
            security => 8_443,
            discovery => 8_081,
            registry => 8_082,
            _ => 8_080, // Default port (including orchestrator)
