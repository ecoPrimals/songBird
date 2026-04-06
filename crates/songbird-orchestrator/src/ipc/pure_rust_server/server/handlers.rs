// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! JSON-RPC request routing and response construction for the pure Rust IPC server.

use songbird_types::json_rpc_method::{HealthMethod, HttpMethod, IpcMethod};
use songbird_types::{JsonRpcMethod, normalize_json_rpc_method_name};

use super::super::coordination_handlers;
use super::super::protocol::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use super::UnixSocketServer;

impl UnixSocketServer {
    /// Handle a JSON-RPC 2.0 request and route to appropriate API handler
    pub(crate) async fn handle_jsonrpc_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let id = request.id.clone().unwrap_or(serde_json::Value::Null);

        if request.jsonrpc != "2.0" {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError::invalid_request(
                    r#"Invalid Request: jsonrpc must be "2.0""#,
                )),
                id,
            };
        }

        let normalized = normalize_json_rpc_method_name(&request.method);

        let result = match JsonRpcMethod::parse_ipc(&request.method) {
            Ok(JsonRpcMethod::Health(HealthMethod::Liveness)) => {
                Ok(songbird_universal_ipc::introspection::health_liveness())
            }
            Ok(JsonRpcMethod::Health(HealthMethod::Readiness)) => {
                Ok(songbird_universal_ipc::introspection::health_readiness())
            }
            Ok(JsonRpcMethod::Health(HealthMethod::Check)) => {
                Ok(songbird_universal_ipc::introspection::health_check())
            }
            Ok(JsonRpcMethod::Ipc(IpcMethod::Register)) => {
                self.handlers.register_service_json(request.params).await
            }
            Ok(JsonRpcMethod::DiscoverCapabilities) => {
                coordination_handlers::handle_discover_capabilities().await
            }
            Ok(JsonRpcMethod::Http(HttpMethod::Request)) => {
                self.handlers
                    .http_request(request.params.unwrap_or_else(|| serde_json::json!({})))
                    .await
            }
            Ok(JsonRpcMethod::Http(HttpMethod::Get)) => {
                self.handlers
                    .http_get(request.params.unwrap_or_else(|| serde_json::json!({})))
                    .await
            }
            Ok(JsonRpcMethod::Http(HttpMethod::Post)) => {
                self.handlers
                    .http_post(request.params.unwrap_or_else(|| serde_json::json!({})))
                    .await
            }
            Ok(_) => Err(JsonRpcError::method_not_found(normalized)),
            Err(_) => match normalized {
                "discover_by_family" => self.handlers.discover_by_family_json(request.params).await,
                "create_genetic_tunnel" => {
                    self.handlers.create_genetic_tunnel_json(request.params).await
                }
                "announce_capabilities" => {
                    self.handlers.announce_capabilities_json(request.params).await
                }
                "discover_by_capability" => {
                    self.handlers.discover_by_capability_json(request.params).await
                }
                "get_service_health" => self.handlers.get_service_health_json(request.params).await,
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
                _ => Err(JsonRpcError::method_not_found(&request.method)),
            },
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
