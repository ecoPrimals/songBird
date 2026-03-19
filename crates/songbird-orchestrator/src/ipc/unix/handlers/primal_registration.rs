// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2025 ecoPrimals
//! Primal Registration Handlers
//!
//! Handlers for primal registration, unregistration, and capability queries.
//! These methods enable primals to advertise their capabilities and discover other primals.

use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::ipc::jsonrpc::JsonRpcError;
use crate::ipc::primal_registry::PrimalRegistry;

/// Handle primal.register - Register a primal with capabilities
pub async fn handle_primal_register(
    registry: Arc<RwLock<PrimalRegistry>>,
    params: Option<Value>,
) -> Result<Value, JsonRpcError> {
    #[derive(Deserialize)]
    struct RegisterParams {
        primal_id: String,
        capabilities: Vec<String>,
        endpoint: Option<String>,
        metadata: Option<serde_json::Map<String, Value>>,
    }
    
    let params: RegisterParams = match params {
        Some(p) => serde_json::from_value(p)
            .map_err(|e| JsonRpcError::invalid_params(e.to_string()))?,
        None => return Err(JsonRpcError::invalid_params("Missing params")),
    };
    
    let info = crate::ipc::primal_registry::PrimalInfo {
        primal_id: params.primal_id.clone(),
        capabilities: params.capabilities,
        endpoint: params.endpoint,
        metadata: params.metadata.unwrap_or_default(),
    };
    
    let mut reg = registry.write().await;
    reg.register(info).await
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?;
    
    info!("✅ Registered primal: {}", params.primal_id);
    
    Ok(serde_json::json!({
        "success": true,
        "primal_id": params.primal_id
    }))
}

/// Handle primal.unregister - Unregister a primal
pub async fn handle_primal_unregister(
    registry: Arc<RwLock<PrimalRegistry>>,
    params: Option<Value>,
) -> Result<Value, JsonRpcError> {
    #[derive(Deserialize)]
    struct UnregisterParams {
        primal_id: String,
    }
    
    let params: UnregisterParams = match params {
        Some(p) => serde_json::from_value(p)
            .map_err(|e| JsonRpcError::invalid_params(e.to_string()))?,
        None => return Err(JsonRpcError::invalid_params("Missing params")),
    };
    
    let mut reg = registry.write().await;
    reg.unregister(&params.primal_id).await
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?;
    
    info!("✅ Unregistered primal: {}", params.primal_id);
    
    Ok(serde_json::json!({
        "success": true,
        "primal_id": params.primal_id
    }))
}

/// Handle primal.get_provider - Get a provider for a specific capability
pub async fn handle_get_provider(
    registry: Arc<RwLock<PrimalRegistry>>,
    params: Option<Value>,
) -> Result<Value, JsonRpcError> {
    #[derive(Deserialize)]
    struct GetProviderParams {
        capability: String,
    }
    
    let params: GetProviderParams = match params {
        Some(p) => serde_json::from_value(p)
            .map_err(|e| JsonRpcError::invalid_params(e.to_string()))?,
        None => return Err(JsonRpcError::invalid_params("Missing params")),
    };
    
    let reg = registry.read().await;
    let provider = reg.get_provider(&params.capability).await
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?;
    
    match provider {
        Some(info) => {
            debug!("🎯 Found provider for '{}': {}", params.capability, info.primal_id);
            Ok(serde_json::to_value(&info)
                .map_err(|e| JsonRpcError::internal_error(e.to_string()))?)
        }
        None => {
            debug!("🔍 No provider found for capability: {}", params.capability);
            Ok(Value::Null)
        }
    }
}

/// Handle primal.list_providers - List all providers for a capability
pub async fn handle_list_providers(
    registry: Arc<RwLock<PrimalRegistry>>,
    params: Option<Value>,
) -> Result<Value, JsonRpcError> {
    #[derive(Deserialize)]
    struct ListProvidersParams {
        capability: String,
    }
    
    let params: ListProvidersParams = match params {
        Some(p) => serde_json::from_value(p)
            .map_err(|e| JsonRpcError::invalid_params(e.to_string()))?,
        None => return Err(JsonRpcError::invalid_params("Missing params")),
    };
    
    let reg = registry.read().await;
    let providers = reg.list_providers(&params.capability).await
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?;
    
    debug!("📋 Found {} providers for '{}'", providers.len(), params.capability);
    
    Ok(serde_json::to_value(&providers)
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?)
}

/// Handle primal.list_all - List all registered primals
pub async fn handle_list_all_primals(
    registry: Arc<RwLock<PrimalRegistry>>,
) -> Result<Value, JsonRpcError> {
    let reg = registry.read().await;
    let primals = reg.list_all().await
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?;
    
    debug!("📋 Listing {} registered primals", primals.len());
    
    Ok(serde_json::to_value(&primals)
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?)
}
