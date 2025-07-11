//! Songbird Universal Orchestrator
//!
//! Enterprise-grade universal orchestration platform that provides automatic discovery,
//! configuration, and management of distributed systems with zero-touch deployment.

#![deny(unsafe_code)]
#![warn(clippy::all)]
#![warn(rust_2018_idioms)]

// Re-export core functionality
pub use songbird_config as config;
pub use songbird_discovery as discovery;
pub use songbird_errors as errors;
pub use songbird_federation as federation;
pub use songbird_network as network;
pub use songbird_observability as observability;
pub use songbird_security as security;
pub use songbird_universal_primals as primals;

// Core modules
pub mod accessibility;
pub mod api;
pub mod basic_iot;
pub mod biome;
pub mod biomeos_integration;
pub mod cli;
pub mod communication;
pub mod firewall;
pub mod health;
pub mod http_server;
pub mod internet_connection;
pub mod load_balancer;
pub mod orchestrator;
pub mod proxy;
pub mod registry;
pub mod traits;
pub mod zero_touch;

// Additional modules
pub mod benchmarks;
pub mod robustness;

/// Universal Songbird result type
pub type Result<T> = std::result::Result<T, errors::SongbirdError>;

/// Initialize Songbird with universal primal integration
pub async fn initialize_songbird() -> Result<primals::UniversalPrimalRegistry> {
    use tracing::info;
    
    info!("🎵 Initializing Songbird Universal Orchestrator with Universal Primal Integration");
    
    // Initialize universal primals
    let mut registry = primals::UniversalPrimalRegistry::new();
    let discovered_primals = registry.auto_discover().await
        .map_err(|e| errors::SongbirdError::Configuration { 
            field: "universal_primals".to_string(),
            message: format!("Failed to auto-discover primals: {}", e) 
        })?;
    
    info!("✅ Discovered {} primals:", discovered_primals.len());
    for primal in &discovered_primals {
        info!("   - {} ({:?}) at {} - {:?}", 
              primal.id, 
              primal.primal_type, 
              primal.endpoint, 
              primal.health);
    }
    
    info!("🎵 Songbird Universal Orchestrator initialized successfully");
    Ok(registry)
}

/// Initialize Songbird with custom primal configuration
pub async fn initialize_songbird_with_config(
    config: &primals::config::UniversalPrimalConfig
) -> Result<primals::UniversalPrimalRegistry> {
    use tracing::info;
    
    info!("🎵 Initializing Songbird Universal Orchestrator with custom configuration");
    
    // Initialize universal primals with config
    let mut registry = primals::UniversalPrimalRegistry::new();
    registry.initialize_with_config(config).await
        .map_err(|e| errors::SongbirdError::Configuration { 
            field: "universal_primals_config".to_string(),
            message: format!("Failed to initialize primals with config: {}", e) 
        })?;
    
    info!("🎵 Songbird Universal Orchestrator initialized with custom config");
    Ok(registry)
}

/// Example usage of universal primal authentication
pub async fn authenticate_with_universal_primals(
    registry: &primals::UniversalPrimalRegistry,
    username: &str,
    password: &str,
) -> Result<String> {
    use tracing::info;
    
    // Find security primals (BearDog) - use the correct method name
    let security_primals = registry.get_instances_by_type(primals::PrimalType::Security).await;
    
    if security_primals.is_empty() {
        return Err(errors::SongbirdError::Authentication { 
            provider: "universal_primals".to_string(),
            message: "No security primals available for authentication".to_string() 
        });
    }
    
    // Create authentication request using the proper method
    let mut payload = std::collections::HashMap::new();
    payload.insert("username".to_string(), serde_json::Value::String(username.to_string()));
    payload.insert("password".to_string(), serde_json::Value::String(password.to_string()));
    
    let auth_request = primals::PrimalRequest::new(
        primals::PrimalRequestType::Authentication,
        payload
    );
    
    // Try first security primal
    let security_primal = &security_primals[0];
    match security_primal.handle_primal_request(auth_request).await {
        Ok(response) => {
            match response.response_type {
                primals::PrimalResponseType::Authentication => {
                    let token = response.payload.get("token")
                        .and_then(|v| v.as_str())
                        .unwrap_or("no-token")
                        .to_string();
                    info!("✅ Authentication successful via primal: {}", security_primal.primal_id());
                    Ok(token)
                }
                _ => {
                    let reason = response.payload.get("reason")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown reason");
                    info!("❌ Authentication failed: {}", reason);
                    Err(errors::SongbirdError::Authentication { 
                        provider: "universal_primals".to_string(),
                        message: format!("Authentication failed: {}", reason)
                    })
                }
            }
        }
        Err(e) => {
            Err(errors::SongbirdError::Authentication { 
                provider: "universal_primals".to_string(),
                message: format!("Security primal error: {}", e)
            })
        }
    }
}

/// Example usage of universal primal encryption
pub async fn encrypt_with_universal_primals(
    registry: &primals::UniversalPrimalRegistry,
    data: &[u8],
) -> Result<Vec<u8>> {
    use tracing::info;
    
    // Find encryption capability
    let encryption_capability = primals::PrimalCapability::Encryption { 
        algorithms: vec!["aes-256-gcm".to_string()] 
    };
    
    // Create a default context
    let encryption_context = primals::PrimalContext::default();
    let encryption_primals = registry.find_by_capability_for_context(&encryption_capability, &encryption_context).await;
    
    if encryption_primals.is_empty() {
        return Err(errors::SongbirdError::Security { 
            context: Some("encryption".to_string()),
            message: "No encryption primals available".to_string() 
        });
    }
    
    // Create encryption request using the proper method
    let mut payload = std::collections::HashMap::new();
    payload.insert("data".to_string(), serde_json::Value::String(
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, data)
    ));
    
    let encrypt_request = primals::PrimalRequest::new(
        primals::PrimalRequestType::Encryption,
        payload
    );
    
    // Try first encryption primal
    let encryption_primal = &encryption_primals[0];
    match encryption_primal.handle_primal_request(encrypt_request).await {
        Ok(response) => {
            match response.response_type {
                primals::PrimalResponseType::Encryption => {
                    let encrypted_data_b64 = response.payload.get("encrypted_data")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| errors::SongbirdError::Security { 
                            context: Some("encryption".to_string()),
                            message: "Missing encrypted_data in response".to_string() 
                        })?;
                    
                    use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
                    let encrypted_data = BASE64_STANDARD.decode(encrypted_data_b64)
                        .map_err(|e| errors::SongbirdError::Security { 
                            context: Some("encryption".to_string()),
                            message: format!("Failed to decode encrypted data: {}", e) 
                        })?;
                    
                    info!("✅ Encryption successful via primal: {}", encryption_primal.primal_id());
                    Ok(encrypted_data)
                }
                _ => {
                    Err(errors::SongbirdError::Security { 
                        context: Some("encryption".to_string()),
                        message: "Encryption failed".to_string()
                    })
                }
            }
        }
        Err(e) => {
            Err(errors::SongbirdError::Security { 
                context: Some("encryption".to_string()),
                message: format!("Encryption primal error: {}", e)
            })
        }
    }
}
