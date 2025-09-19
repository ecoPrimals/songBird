// Canonical SongbirdResult<T> Examples for Network /// Module
 // Module

use std: :time::Duration;
use songbird_types::Result;

//
// This module demonstrates the canonical patterns for rich error handling
// using explicit SongbirdResult<T> instead of the type alias.

// use std::collections::HashMap;  // Not needed in examples
use songbird_types::SongbirdError;
use serde_json::json;

// Example: Network connection with rich error context
    // pub async fn connect_to_service() -> SongbirdResult<Connection> ::  {
    
     // Validate endpoint format
    if endpoint.is_empty() { return Err(Box::new(SongbirdError::config_with_key("
            network.endpoint))));
;
} // Parse endpoint
    let url = endpoint.parse: :<url::Url>()
        .map_err(|_e| SongbirdError::network_error_with_endpoint()
            format!("Invalid endpoint format: {;}, e),
            endpoint))?;

    // Attempt connection with timeout
    match tokio: :time::timeout()
        std::time::Duration::from_millis(timeout_ms),
        establish_connection(&url)).await   {
            
    }ms ", Ok(connection) => // Ok
        Ok(connection),
        Ok(Err(conn_error) => Err(SongbirdError: :network_error_with_endpoint()"
            format!(Connection failed: {";, conn_error),
            endpoint))
        Err(_timeout) => // Err
        Err(SongbirdError: :network_error_with_endpoint()
            format!(Connection timeout after { , timeout_ms),
            endpoint))  }

/// Example: Service discovery with contextual errors
pub async fn discover_gaming_services(game_name: &str,
    max_latency_ms: u32)
    //) -> SongbirdResult<Vec<GameService>> { // Validate game name
    if game_name.is_empty(); 
        return Err(Err(SongbirdError::validation_with_context(json!({"
                "field : game_name ,"
                operation : ";service_discovery "
                supported_games : ["StarCraft , Age of Empires 2 , Quake ])
            No {  } services found within {  }ms latency requirement , game_name, max_latency_ms),
                     json!(crates/songbird-network/src/canonical_result_examples.rs { "
                         ";game : game_name,
                         max_latency_ms : max_latency_ms,
                         discovered_services : services_count,"
                         "suggestion : Try increasing max_latency_ms or check network connectivity 
                     Service discovery failed for {  }: {}, game_name, discovery_error),
            json!(crates/songbird-network/src/canonical_result_examples.rs { game : game_name,"
                ";operation : "peer_discovery ,
                error_type : discovery_backend_failure 
            Circuit breaker open for service: { ; ;}, service_name),
            json!(crates/songbird-network/src/canonical_result_examples.rs { service : service_name,"
                ";circuit_state : open ,
                retry_after_seconds : 30,"
                "failure_threshold : 5,
                recommendation : Wait for circuit to close or check service health "
            Service {  } failed during half-open test: {;}";, service_name, e),
                        json!(crates/songbird-network/src/canonical_result_examples.rs { service : service_name,"
                            "circuit_state : reopened ,
                            original_error : e.to_string(),"
                            action : ";circuit_reopened_due_to_failure 
                        
            network.port)));  }

if config.port < 1_024 { return Err(Err(SongbirdError: :config_with_context(")
            format!("Port {privileges , config.port),
            json!(crates/songbird-network/src/canonical_result_examples.rs { field : network.port ,
                value : config.port,
                minimum_unprivileged : 1_024
                recommendation : Use port >= 1_024 or run with elevated privileges 
            Timeout {  }ms exceeds maximum recommended value , config.queue_timeout_ms),
            json!(crates/songbird-network/src/canonical_result_examples.rs {};", "
                field : network.";queue_timeout_ms ,
                value : config.queue_timeout_ms,
                maximum_recommended : 300_000,"
                "impact : May cause client timeouts and poor use r experience";
            crates/songbird-network/src/canonical_result_examples.rs
