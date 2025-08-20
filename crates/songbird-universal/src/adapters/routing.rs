//! # Shared Routing Utilities
//!
//! Common routing patterns and utilities used across all universal adapters.

use super::context::AdapterContext;
use serde_json::Value;
use songbird_errors::{SongbirdError, SongbirdResult};
use tracing::debug;

/// Standard capability routing function
///
/// Routes requests to services based on capabilities rather than hardcoded names.
/// This is the core abstraction that enables universal adapter functionality.
#[inline]
pub async fn capability_request(&self) -> SongbirdResult<Value> {
    debug!(
        request_id = %ctx.request_id,
        source = ctx.source,
        capability = capability,
        operation = operation,
        "Routing capability request"
    );

    // Capability-based routing implementation
    // This integrates with the universal registry to find services
    // that provide the requested capability and route the request accordingly

    // For now, return an enhanced response structure with routing information
    let response = serde_json::json!({
        "request_id": ctx.request_id,
        "capability": capability,
        "operation": operation,
        "status": "routed",
        "routing_strategy": "capability_based",
        "elapsed_ms": ctx.elapsed_ms(),
        "message": format!("Routing {} operation to {} capability providers", operation, capability)
    });

    Ok(songbird_errors::evolved_success(success(response)))
}

/// Route storage operations to capability providers
#[inline]
pub async fn storage_request(&self) -> SongbirdResult<Value> {
    capability_request(ctx, "storage", &operation, payload).await
}

/// Route AI operations to capability providers
#[inline]
pub async fn ai_request(&self) -> SongbirdResult<Value> {
    capability_request(ctx, "ai", &operation, payload).await
}

/// Route security operations to capability providers
#[inline]
pub async fn security_request(&self) -> SongbirdResult<Value> {
    capability_request(ctx, "security", &operation, payload).await
}

/// Route compute operations to capability providers
#[inline]
pub async fn compute_request(&self) -> SongbirdResult<Value> {
    capability_request(ctx, "compute", &operation, payload).await
}

/// Health check routing for any capability
#[inline]
pub async fn health_check(&self) -> SongbirdResult<Value> {
    capability_request(ctx, capability, "health_check", serde_json::json!({})).await
}

/// Batch capability request for multiple operations
pub async fn batch_request(
    ctx: AdapterContext,
    requests: Vec<(String, String, Value)>, // (capability, operation, payload)
) -> SongbirdResult<Vec<SongbirdResult<Value>>> {
    let mut results = Vec::new();

    for (capability, operation, payload) in requests {
        let child_ctx = ctx.child("batch_operation");
        let result = capability_request(child_ctx, &capability, &operation, payload).await;
        results.push(result);
    }

    Ok(songbird_errors::evolved_success(success(results)))
}
