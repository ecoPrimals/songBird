// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![forbid(unsafe_code)]

//! `content.*` method dispatch — CAS content location across local registry and federation mesh.

use super::super::IpcServiceHandler;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use songbird_types::json_rpc_method::{ContentMethod, JsonRpcMethod};
use songbird_types::{SongbirdError, SongbirdResult};
use tracing::debug;

/// Capability token for content-addressable storage providers (e.g. westGate ZFS CAS).
const CONTENT_STORAGE_CAPABILITY: &str = "content_storage";

pub(super) async fn dispatch_content(
    handler: &IpcServiceHandler,
    method: JsonRpcMethod,
    params: Value,
) -> Result<Value, String> {
    match method {
        JsonRpcMethod::Content(ContentMethod::Locate) => IpcServiceHandler::wrap_result(
            handler.handle_content_locate(params).await,
            "content.locate failed",
        ),
        JsonRpcMethod::Content(ContentMethod::Verify) => IpcServiceHandler::wrap_result(
            handler.handle_content_verify(params).await,
            "content.verify failed",
        ),
        JsonRpcMethod::Content(ContentMethod::Availability) => IpcServiceHandler::wrap_result(
            handler.handle_content_availability(params).await,
            "content.availability failed",
        ),
        JsonRpcMethod::Content(ContentMethod::Put) => IpcServiceHandler::wrap_result(
            handler.handle_content_put(params).await,
            "content.put failed",
        ),
        other => Err(format!("Unknown content method: {other}")),
    }
}

#[derive(Debug, Deserialize)]
struct ContentLocateParams {
    hash: String,
    #[serde(default = "default_algorithm")]
    algorithm: String,
    #[serde(default = "default_scope")]
    scope: String,
}

fn default_algorithm() -> String {
    String::from("blake3")
}

fn default_scope() -> String {
    String::from("all")
}

#[derive(Debug, Serialize)]
struct ContentLocation {
    gate: String,
    endpoint: String,
    verified: bool,
}

#[derive(Debug, Serialize)]
struct ContentLocateResult {
    locations: Vec<ContentLocation>,
    hash: String,
}

fn parse_locate_params(params: Value) -> SongbirdResult<ContentLocateParams> {
    serde_json::from_value(params)
        .map_err(|e| SongbirdError::validation(format!("Invalid params: {e}")))
}

fn validate_locate_params(params: &ContentLocateParams) -> SongbirdResult<()> {
    if params.hash.is_empty() {
        return Err(SongbirdError::validation("Missing or empty 'hash'"));
    }

    match params.algorithm.as_str() {
        "blake3" | "sha256" => {}
        other => {
            return Err(SongbirdError::validation(format!(
                "Invalid 'algorithm': {other} (expected 'blake3' or 'sha256')"
            )));
        }
    }

    match params.scope.as_str() {
        "local" | "mesh" | "all" => {}
        other => {
            return Err(SongbirdError::validation(format!(
                "Invalid 'scope': {other} (expected 'local', 'mesh', or 'all')"
            )));
        }
    }

    Ok(())
}

impl IpcServiceHandler {
    /// Handle `content.locate` — resolve endpoint(s) for a content hash.
    pub(super) async fn handle_content_locate(&self, params: Value) -> SongbirdResult<Value> {
        let params = parse_locate_params(params)?;
        validate_locate_params(&params)?;

        let mut locations = Vec::new();

        if params.scope == "local" || params.scope == "all" {
            locations.extend(self.locate_local_content_providers().await?);
        }

        if params.scope == "mesh" || params.scope == "all" {
            locations
                .extend(self.locate_mesh_content_providers(&params.hash, &params.algorithm).await?);
        }

        let result = ContentLocateResult {
            locations,
            hash: params.hash,
        };

        serde_json::to_value(result).map_err(|e| SongbirdError::serialization(e.to_string()))
    }

    /// Handle `content.verify` — stub pending CAS verification protocol.
    pub(super) async fn handle_content_verify(&self, _params: Value) -> SongbirdResult<Value> {
        Err(SongbirdError::not_implemented("content.verify"))
    }

    /// Handle `content.availability` — stub pending CAS availability probe.
    pub(super) async fn handle_content_availability(
        &self,
        _params: Value,
    ) -> SongbirdResult<Value> {
        Err(SongbirdError::not_implemented("content.availability"))
    }

    /// Handle `content.put` — route a CAS write to a local provider with `content_storage` capability.
    ///
    /// Accepts: `{ hash, content, algorithm?, scope? }`
    /// - `hash` (string): content-addressable hash of the content
    /// - `content` (string): base64-encoded content bytes
    /// - `algorithm` (string, default `"blake3"`): hash algorithm
    /// - `scope` (string, default `"local"`): `"local"` routes to local CAS providers only
    ///
    /// Delegates to `capability.call` with `content_storage` capability for routing.
    pub(super) async fn handle_content_put(&self, params: Value) -> SongbirdResult<Value> {
        let hash = params
            .get("hash")
            .and_then(Value::as_str)
            .ok_or_else(|| SongbirdError::validation("Missing or empty 'hash'"))?;

        if hash.is_empty() {
            return Err(SongbirdError::validation("Missing or empty 'hash'"));
        }

        let content = params
            .get("content")
            .ok_or_else(|| SongbirdError::validation("Missing 'content' field"))?;

        let algorithm = params
            .get("algorithm")
            .and_then(Value::as_str)
            .unwrap_or("blake3");

        match algorithm {
            "blake3" | "sha256" => {}
            other => {
                return Err(SongbirdError::validation(format!(
                    "Invalid 'algorithm': {other} (expected 'blake3' or 'sha256')"
                )));
            }
        }

        debug!(hash, algorithm, "content.put: routing via capability.call to content_storage provider");

        let cap_call_params = serde_json::json!({
            "capability": CONTENT_STORAGE_CAPABILITY,
            "operation": "put",
            "params": {
                "hash": hash,
                "content": content,
                "algorithm": algorithm,
            },
            "routing": "local"
        });

        self.handle_capability_call(cap_call_params)
            .await
            .map_err(|e| SongbirdError::service("content.put", format!("capability.call: {e}")))
    }

    async fn locate_local_content_providers(&self) -> SongbirdResult<Vec<ContentLocation>> {
        let registry = self.registry.read().await;
        let virtual_paths = registry.find_by_capability(CONTENT_STORAGE_CAPABILITY).await;

        let mut locations = Vec::new();
        for virtual_path in virtual_paths {
            let Some(name) = virtual_path.strip_prefix("/primal/") else {
                continue;
            };
            let Some(entry) = registry.get_service(name).await else {
                continue;
            };
            locations.push(ContentLocation {
                gate: name.to_string(),
                endpoint: entry.native_endpoint.display(),
                verified: false,
            });
        }

        Ok(locations)
    }

    async fn locate_mesh_content_providers(
        &self,
        hash: &str,
        algorithm: &str,
    ) -> SongbirdResult<Vec<ContentLocation>> {
        use songbird_types::defaults::ports::DEFAULT_HTTP_PORT;

        let mesh_guard = self.mesh_handler.mesh().await;
        let Some(mesh) = mesh_guard.as_ref() else {
            debug!("content.locate mesh: mesh not initialized");
            return Ok(Vec::new());
        };

        let reachable = mesh.get_reachable_nodes().await;
        if reachable.is_empty() {
            debug!("content.locate mesh: no reachable peers");
            return Ok(Vec::new());
        }

        let mut locations = Vec::new();

        for node_id in &reachable {
            let Some(path) = mesh.get_best_path(node_id).await else {
                continue;
            };
            let peer_sock = path.endpoint_type.socket_addr().unwrap_or_else(|| {
                let ip = path
                    .endpoint_type
                    .address()
                    .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST));
                std::net::SocketAddr::new(ip, DEFAULT_HTTP_PORT)
            });

            let tcp_endpoint = songbird_types::constants::jsonrpc_endpoint_url(&peer_sock);

            let request = serde_json::json!({
                "jsonrpc": "2.0",
                "method": "capability.call",
                "params": {
                    "capability": "content",
                    "operation": "exists",
                    "params": { "hash": hash, "algorithm": algorithm },
                    "routing": "local"
                },
                "id": 1
            });

            match self.http_post_jsonrpc(&tcp_endpoint, &request).await {
                Ok(response) => {
                    let exists = response
                        .get("result")
                        .and_then(|r| r.get("exists"))
                        .and_then(serde_json::Value::as_bool)
                        .unwrap_or(false);

                    if exists {
                        debug!(
                            peer = %node_id,
                            hash,
                            "content.locate: peer confirms content exists"
                        );
                        locations.push(ContentLocation {
                            gate: node_id.clone(),
                            endpoint: tcp_endpoint,
                            verified: true,
                        });
                    } else {
                        debug!(peer = %node_id, hash, "content.locate: peer does not have content");
                    }
                }
                Err(e) => {
                    debug!(
                        peer = %node_id,
                        error = %e,
                        "content.locate: failed to probe peer"
                    );
                }
            }
        }

        Ok(locations)
    }
}
