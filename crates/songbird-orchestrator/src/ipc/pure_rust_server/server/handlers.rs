// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! JSON-RPC request routing and response construction for the pure Rust IPC server.

use super::super::coordination_handlers;
use super::super::protocol::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use super::UnixSocketServer;

impl UnixSocketServer {
    /// Handle a JSON-RPC 2.0 request and route to appropriate API handler
    pub(crate) async fn handle_jsonrpc_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let id = request.id.clone().unwrap_or(serde_json::Value::Null);

        let result = match request.method.as_str() {
            "discover_by_family" => self.handlers.discover_by_family_json(request.params).await,
            "create_genetic_tunnel" => {
                self.handlers.create_genetic_tunnel_json(request.params).await
            }
            "announce_capabilities" => {
                self.handlers.announce_capabilities_json(request.params).await
            }

            "register_service" => self.handlers.register_service_json(request.params).await,
            "discover_by_capability" => {
                self.handlers.discover_by_capability_json(request.params).await
            }
            "get_service_health" => self.handlers.get_service_health_json(request.params).await,
            "health_check" => self.handlers.health_check_json().await,

            "graph.validate" => self.handlers.validate_graph_json(request.params).await,
            "graph.check_availability" => {
                self.handlers.check_availability_json(request.params).await
            }
            "graph.suggest_alternatives" => {
                self.handlers.suggest_alternatives_json(request.params).await
            }
            "coordination.validate_pattern" => {
                self.handlers.validate_coordination_pattern_json(request.params).await
            }

            "http.request" => {
                self.handlers
                    .http_request(request.params.unwrap_or_else(|| serde_json::json!({})))
                    .await
            }
            "http.get" => {
                self.handlers
                    .http_get(request.params.unwrap_or_else(|| serde_json::json!({})))
                    .await
            }
            "http.post" => {
                self.handlers
                    .http_post(request.params.unwrap_or_else(|| serde_json::json!({})))
                    .await
            }
            "http.put" => {
                self.handlers
                    .http_put(request.params.unwrap_or_else(|| serde_json::json!({})))
                    .await
            }
            "http.delete" => {
                self.handlers
                    .http_delete(request.params.unwrap_or_else(|| serde_json::json!({})))
                    .await
            }

            "discover_capabilities" => coordination_handlers::handle_discover_capabilities().await,
            "health" => coordination_handlers::handle_health().await,

            _ => Err(JsonRpcError::method_not_found(&request.method)),
        };

        match result {
            Ok(value) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(value),
                error: None,
                id,
            },
            Err(error) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(error),
                id,
            },
        }
    }
}
