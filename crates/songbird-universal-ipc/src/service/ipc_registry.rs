// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::{
    CompositionPrimalInfo, CompositionState, DiscoverParams, DiscoverResult, IpcServiceHandler,
    ListResult, ProviderInfo, RegisterParams, RegisterResult, ResolveParams, ResolveResult,
    ServiceInfo, TransportEndpoint, ValidateConsumedResult,
};
use crate::endpoint::NativeEndpoint;
use crate::introspection::CONSUMED_CAPABILITIES;
use serde_json::Value;
use songbird_types::defaults::timeouts::DEFAULT_SOCKET_IO_TIMEOUT;
use std::sync::Arc;
use tracing::debug;

/// Build a deterministic canonical JSON payload for signing.
///
/// Field ordering is alphabetical by short key to ensure identical bytes
/// regardless of capability insertion order.
fn build_canonical_payload(
    primal_id: &str,
    capabilities: &[String],
    endpoint: &str,
    registered_at: &str,
) -> String {
    let mut sorted_caps = capabilities.to_vec();
    sorted_caps.sort();
    serde_json::json!({
        "c": sorted_caps,
        "e": endpoint,
        "p": primal_id,
        "t": registered_at,
    })
    .to_string()
}

impl IpcServiceHandler {
    /// Handle `ipc.register` method
    pub(super) async fn handle_register(&self, params: Value) -> Result<Value, String> {
        let params: RegisterParams =
            serde_json::from_value(params).map_err(|e| format!("Invalid params: {e}"))?;

        tracing::info!("Registering primal: {} at {}", params.primal_id, params.endpoint);

        let registered_at = chrono::Utc::now().to_rfc3339();

        // Build canonical payload before endpoint parsing moves the string
        let canonical = build_canonical_payload(
            &params.primal_id,
            &params.capabilities,
            &params.endpoint,
            &registered_at,
        );

        // Parse native endpoint
        let native_endpoint = if params.endpoint.starts_with('/') {
            NativeEndpoint::UnixSocket(params.endpoint.into())
        } else if let Some(stripped) = params.endpoint.strip_prefix("unix://") {
            NativeEndpoint::UnixSocket(stripped.into())
        } else if let Some(stripped) = params.endpoint.strip_prefix("tcp://") {
            let port = Self::parse_tcp_port(stripped)?;
            NativeEndpoint::TcpLocal(port)
        } else if let Some(port) = Self::parse_local_tcp_endpoint(&params.endpoint) {
            NativeEndpoint::TcpLocal(port)
        } else {
            return Err(format!(
                "Invalid endpoint format: '{}'. Expected unix socket path, unix://<path>, tcp://<host>:<port>, or <localhost>:<port>",
                params.endpoint
            ));
        };

        // Verify registrant identity: probe the endpoint with identity.get and confirm
        // the primal_id matches. Gracefully degrades if endpoint is unreachable (trust-on-first-use
        // for primals still starting). Hard-rejects identity mismatch (spoofed names).
        self.verify_registrant_identity(&native_endpoint, &params.primal_id).await?;

        // Sign via BearDog if crypto provider is available
        let (signature, signed_payload) = self.sign_payload(&canonical).await;

        // Build structured transport before register() consumes native_endpoint
        let transport = match &native_endpoint {
            NativeEndpoint::UnixSocket(path) => Some(TransportEndpoint::Uds {
                path: path.to_string_lossy().to_string(),
            }),
            NativeEndpoint::AbstractSocket(name) => Some(TransportEndpoint::Uds {
                path: format!("@{name}"),
            }),
            NativeEndpoint::TcpLocal(port) => Some(TransportEndpoint::Tcp {
                host: songbird_types::constants::LOCALHOST.to_string(),
                port: *port,
            }),
            NativeEndpoint::NamedPipe(name) => Some(TransportEndpoint::NamedPipe {
                name: name.clone(),
            }),
            _ => None,
        };

        // Register in registry (`register` takes `&self` and uses its own inner lock)
        let native_socket = native_endpoint.socket_path();
        let has_capabilities = !params.capabilities.is_empty();
        let virtual_endpoint = self
            .registry
            .read()
            .await
            .register(
                &params.primal_id,
                native_endpoint,
                params.capabilities,
                signature.clone(),
                signed_payload.clone(),
            )
            .await
            .map_err(|e| format!("Registration failed: {e}"))?;

        // Phase 1 (shadow mode): spawn virtual relay listener alongside native endpoint
        if let Some(ref socket_path) = native_socket
            && let Err(e) = self.virtual_relay.start_relay(&params.primal_id, socket_path).await
        {
            tracing::warn!(
                primal = %params.primal_id,
                error = %e,
                "Virtual relay start failed (non-blocking)"
            );
        }

        // Propagate capabilities to mesh peers (push model for cross-gate discovery)
        if has_capabilities {
            let all_capabilities = self.collect_all_local_capabilities().await;
            let mesh = Arc::clone(&self.mesh_handler);
            let primal_id_for_gossip = params.primal_id.clone();
            let caps_for_gossip = all_capabilities.clone();
            tokio::spawn(async move {
                mesh.announce_capabilities_to_peers(all_capabilities).await;
            });

            // Phase 3 seam: inject into local swarmVine for epidemic gossip propagation.
            // Resolve gate identity from environment (same as swarmVine itself does).
            let gate_id = std::env::var("GATE_ID")
                .or_else(|_| std::env::var("HOSTNAME"))
                .unwrap_or_default();
            if !gate_id.is_empty() {
                tokio::spawn(async move {
                    inject_to_swarmvine(&gate_id, &primal_id_for_gossip, &caps_for_gossip).await;
                });
            }
        }

        let result = RegisterResult {
            virtual_endpoint: virtual_endpoint.path,
            registered_at,
            transport,
            signature,
            signed_payload,
        };

        serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
    }

    /// Sign a canonical payload via the crypto provider (`BearDog` Ed25519 delegation).
    ///
    /// Returns `(Some(signature), Some(payload))` on success, `(None, None)` if
    /// no crypto provider is configured or signing fails (standalone mode).
    async fn sign_payload(&self, canonical_payload: &str) -> (Option<String>, Option<String>) {
        use base64::Engine as _;

        let Some(ref provider) = self.crypto_provider else {
            return (None, None);
        };

        let data_b64 =
            base64::engine::general_purpose::STANDARD.encode(canonical_payload.as_bytes());

        match provider.call("crypto.sign.ed25519", serde_json::json!({ "data": data_b64 })).await {
            Ok(result) => result.get("signature").and_then(|v| v.as_str()).map_or_else(
                || {
                    tracing::warn!("crypto.sign.ed25519 returned no signature field");
                    (None, None)
                },
                |s| {
                    debug!("Signed registration payload ({} bytes)", s.len());
                    (Some(s.to_string()), Some(canonical_payload.to_string()))
                },
            ),
            Err(e) => {
                tracing::warn!("Failed to sign registration (standalone fallback): {e}");
                (None, None)
            }
        }
    }

    /// Collect all capabilities from all locally registered primals.
    ///
    /// Used to build the aggregate capability set announced to mesh peers.
    async fn collect_all_local_capabilities(&self) -> Vec<String> {
        let registry = self.registry.read().await;
        let metadata = registry.get_all_metadata().await;
        let mut all_caps: Vec<String> = metadata.into_iter().flat_map(|m| m.capabilities).collect();
        all_caps.sort();
        all_caps.dedup();
        all_caps
    }

    /// Verify a registering primal's identity by probing its endpoint with `identity.get`.
    ///
    /// Returns `Ok(())` if the probe confirms the primal's identity matches `expected_name`,
    /// or if the endpoint is unreachable (graceful degradation for primals still starting).
    /// Returns `Err(reason)` only if the primal responds but claims a DIFFERENT identity.
    async fn verify_registrant_identity(
        &self,
        endpoint: &NativeEndpoint,
        expected_name: &str,
    ) -> Result<(), String> {
        let Some(response) = self.probe_identity(endpoint).await else {
            tracing::warn!(
                "ipc.register identity probe: no response from {expected_name} \
                 (allowing trust-on-first-use)"
            );
            return Ok(());
        };

        if response.get("error").is_some() {
            debug!("ipc.register identity probe: identity.get returned error (non-fatal)");
            return Ok(());
        }

        let Some(claimed_name) =
            response.get("result").and_then(|r| r.get("primal")).and_then(|p| p.as_str())
        else {
            return Ok(());
        };

        if !claimed_name.eq_ignore_ascii_case(expected_name) {
            return Err(format!(
                "Identity mismatch: registering as '{expected_name}' but endpoint \
                 claims to be '{claimed_name}'"
            ));
        }
        debug!("ipc.register identity verified: {expected_name}");
        Ok(())
    }

    /// Send `identity.get` to an endpoint and return the parsed JSON response.
    ///
    /// Returns `None` on connection failure, timeout, or unparseable response
    /// (all treated as trust-on-first-use by the caller).
    async fn probe_identity(&self, endpoint: &NativeEndpoint) -> Option<Value> {
        let req_bytes =
            b"{\"jsonrpc\":\"2.0\",\"method\":\"identity.get\",\"params\":{},\"id\":1}\n";

        match endpoint {
            NativeEndpoint::UnixSocket(path) => {
                #[cfg(unix)]
                {
                    let stream = tokio::time::timeout(
                        DEFAULT_SOCKET_IO_TIMEOUT,
                        tokio::net::UnixStream::connect(path),
                    )
                    .await
                    .ok()?
                    .ok()?;

                    Self::send_and_read(stream, req_bytes).await
                }
                #[cfg(not(unix))]
                {
                    let _ = path;
                    None
                }
            }
            NativeEndpoint::TcpLocal(port) => {
                let addr = format!("{}:{port}", songbird_types::constants::LOCALHOST);
                let stream = tokio::time::timeout(
                    DEFAULT_SOCKET_IO_TIMEOUT,
                    tokio::net::TcpStream::connect(&addr),
                )
                .await
                .ok()?
                .ok()?;

                Self::send_and_read(stream, req_bytes).await
            }
            _ => None,
        }
    }

    /// Write a request and read a single NDJSON response line from a stream.
    async fn send_and_read<S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin>(
        stream: S,
        request: &[u8],
    ) -> Option<Value> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

        let (reader, mut writer) = tokio::io::split(stream);
        let mut reader = BufReader::new(reader);
        writer.write_all(request).await.ok()?;
        writer.flush().await.ok()?;

        let mut line = String::new();
        let n = tokio::time::timeout(DEFAULT_SOCKET_IO_TIMEOUT, reader.read_line(&mut line))
            .await
            .ok()?
            .ok()?;
        if n == 0 {
            return None;
        }
        serde_json::from_str(line.trim()).ok()
    }

    /// Handle `ipc.resolve` method — resolves by `capability` or `primal_id`/`name`.
    ///
    /// **Capability-first** is the standard pattern: callers should resolve by what
    /// a primal *does* (e.g. `"security"`), not what it *is* (e.g. `"beardog"`).
    ///
    /// Precedence: `capability` > `primal_id`/`name`.
    ///
    /// Graceful fallback: if `capability` lookup fails, the same string is tried
    /// as a primal name. This accommodates callers who conflate capability tokens
    /// with primal names (e.g. `resolve({"capability": "beardog"})` will succeed
    /// if `BearDog` registered with `primal_id` = `"beardog"`).
    ///
    /// `ipc.resolve_by_name` is a normalization alias that routes here.
    pub(super) async fn handle_resolve(&self, params: Value) -> Result<Value, String> {
        let params: ResolveParams =
            serde_json::from_value(params).map_err(|e| format!("Invalid params: {e}"))?;

        let registry = self.registry.read().await;

        let (resolved_name, entry) = if let Some(ref capability) = params.capability {
            debug!("Resolving by capability: {capability}");
            if let Some(found) = registry.resolve_by_capability(capability).await {
                found
            } else if let Some(entry) = registry.get_service(capability).await {
                tracing::warn!(
                    capability = %capability,
                    "Name-based fallback in ipc.resolve is deprecated; resolve by capability, not primal name"
                );
                debug!(
                    "Capability '{capability}' not found, but matched as primal name (capability-first fallback)"
                );
                (capability.clone(), entry)
            } else {
                drop(registry);
                // Mesh fallback: check if a remote peer advertises this capability
                if let Some((peer_id, peer_caps)) =
                    self.mesh_handler.find_peer_with_capability(capability).await
                {
                    debug!(
                        capability = %capability,
                        peer = %peer_id,
                        "Resolved capability via mesh peer (topology-aware routing)"
                    );
                    let native_endpoint = format!("mesh://{peer_id}");
                    let result = ResolveResult {
                        socket: None,
                        virtual_endpoint: String::new(),
                        native_endpoint,
                        endpoint: TransportEndpoint::MeshRelay {
                            peer_id,
                            capability: capability.clone(),
                        },
                        capabilities: peer_caps,
                        relay: true,
                        relay_socket: None,
                        signature: None,
                        signed_payload: None,
                    };
                    return serde_json::to_value(result)
                        .map_err(|e| format!("Serialization error: {e}"));
                }
                return Err(format!("No provider found for capability: {capability}"));
            }
        } else if let Some(ref primal_id) = params.primal_id {
            debug!("Resolving by primal_id: {primal_id}");
            let entry = registry
                .get_service(primal_id)
                .await
                .ok_or_else(|| format!("Primal not found: {primal_id}"))?;
            (primal_id.clone(), entry)
        } else {
            return Err(
                "ipc.resolve requires either `primal_id`/`name` or `capability` parameter".into()
            );
        };

        let native_transport = transport_endpoint_from_native(&entry.native_endpoint);
        let native_socket = entry.native_endpoint.socket_path();
        let native_display = entry.native_endpoint.display();
        let resolved_capability = params.capability.clone().unwrap_or_default();
        let capabilities = entry.capabilities;
        let signature = entry.signature;
        let signed_payload = entry.signed_payload;
        let virtual_path = entry.virtual_endpoint.path;
        drop(registry);

        // Determine relay availability and whether to use it
        let relay_path = self.virtual_relay.get_relay_path(&resolved_name).await;
        let use_relay = params.prefer_virtual && !params.native && relay_path.is_some();

        let socket = if use_relay {
            relay_path.as_ref().map(|p| p.display().to_string())
        } else {
            native_socket
        };

        // Build transport-qualified endpoint (Phase 2)
        let endpoint = if use_relay {
            TransportEndpoint::MeshRelay {
                peer_id: resolved_name.clone(),
                capability: resolved_capability,
            }
        } else {
            native_transport
        };

        let result = ResolveResult {
            socket,
            virtual_endpoint: virtual_path,
            native_endpoint: native_display,
            endpoint,
            capabilities,
            relay: use_relay,
            relay_socket: relay_path.map(|p| p.display().to_string()),
            signature,
            signed_payload,
        };

        debug!("Resolved to: {resolved_name} (relay={})", result.relay);

        serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
    }

    /// Handle `ipc.discover` method
    pub(super) async fn handle_discover(&self, params: Value) -> Result<Value, String> {
        let params: DiscoverParams =
            serde_json::from_value(params).map_err(|e| format!("Invalid params: {e}"))?;

        debug!("Discovering capability: {}", params.capability);

        // Discover from registry (returns virtual paths)
        let registry = self.registry.read().await;
        let virtual_paths = registry.find_by_capability(&params.capability).await;

        // Get full service entries for each path
        let mut provider_infos = Vec::new();
        for virtual_path in virtual_paths {
            // Extract service name from virtual path
            if let Some(name) = virtual_path.strip_prefix("/primal/")
                && let Some(entry) = registry.get_service(name).await
            {
                provider_infos.push(ProviderInfo {
                    primal_id: name.to_string(),
                    socket: entry.native_endpoint.socket_path(),
                    virtual_endpoint: entry.virtual_endpoint.path,
                    native_endpoint: entry.native_endpoint.display(),
                    capabilities: entry.capabilities,
                    signature: entry.signature,
                    signed_payload: entry.signed_payload,
                });
            }
        }

        let result = DiscoverResult {
            providers: provider_infos,
        };

        serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
    }

    /// Handle `ipc.list` method
    pub(super) async fn handle_list(&self, _params: Value) -> Result<Value, String> {
        debug!("Listing all services");

        let registry = self.registry.read().await;
        let service_names = registry.list_services().await;

        let mut service_infos = Vec::new();
        for name in service_names {
            if let Some(entry) = registry.get_service(&name).await {
                service_infos.push(ServiceInfo {
                    primal_id: name,
                    virtual_endpoint: entry.virtual_endpoint.path,
                    capabilities: entry.capabilities,
                });
            }
        }

        let result = ListResult {
            services: service_infos,
        };

        serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
    }

    /// Handle `ipc.relay_stats` — return virtual relay performance metrics.
    ///
    /// ## Response
    /// ```json
    /// { "active_relays": 3, "total_requests": 1024, "avg_overhead_us": 342 }
    /// ```
    pub(super) async fn handle_relay_stats(&self, _params: Value) -> Result<Value, String> {
        let relays = self.virtual_relay.list_relays().await;
        let metrics = self.virtual_relay.metrics();

        Ok(serde_json::json!({
            "active_relays": relays.len(),
            "relays": relays.iter().map(|(name, path)| {
                serde_json::json!({"primal": name, "socket": path.display().to_string()})
            }).collect::<Vec<_>>(),
            "total_requests": metrics.requests.load(std::sync::atomic::Ordering::Relaxed),
            "avg_overhead_us": metrics.avg_overhead_us(),
            "total_overhead_us": metrics.overhead_us.load(std::sync::atomic::Ordering::Relaxed),
        }))
    }

    /// Handle `ipc.watch` — poll for registry changes since a given revision.
    ///
    /// Enables consuming primals (e.g. toadStool) to detect when new providers
    /// register capabilities they depend on. Returns events since `since_revision`,
    /// optionally filtered by capability names.
    ///
    /// ## Params
    /// ```json
    /// { "since_revision": 0, "capabilities": ["shader", "compile"] }
    /// ```
    ///
    /// ## Response
    /// ```json
    /// {
    ///   "revision": 5,
    ///   "events": [
    ///     { "revision": 3, "kind": "registered", "primal": "coralReef",
    ///       "capabilities": ["shader", "compile", "visualization"],
    ///       "endpoint": "unix:///run/user/1000/biomeos/coralreef-nucleus01.sock" }
    ///   ]
    /// }
    /// ```
    pub(super) async fn handle_watch(&self, params: Value) -> Result<Value, String> {
        #[derive(serde::Deserialize)]
        struct WatchParams {
            #[serde(default)]
            since_revision: u64,
            #[serde(default)]
            capabilities: Option<Vec<String>>,
        }

        let params: WatchParams =
            serde_json::from_value(params).map_err(|e| format!("Invalid params: {e}"))?;

        let registry = self.registry.read().await;
        let (current_rev, events) =
            registry.events_since(params.since_revision, params.capabilities.as_deref()).await;

        serde_json::to_value(serde_json::json!({
            "revision": current_rev,
            "events": events,
        }))
        .map_err(|e| format!("Serialization error: {e}"))
    }

    /// Handle `lifecycle.composition` — returns current composition state for dashboards.
    pub(super) async fn handle_lifecycle_composition(
        &self,
        _params: Value,
    ) -> Result<Value, String> {
        debug!("Returning composition state");

        let registry = self.registry.read().await;
        let service_names = registry.list_services().await;

        let mut primals = Vec::new();
        let mut total_capabilities = 0usize;

        for name in service_names {
            if let Some(entry) = registry.get_service(&name).await {
                total_capabilities += entry.capabilities.len();
                primals.push(CompositionPrimalInfo {
                    primal_id: name,
                    capabilities: entry.capabilities,
                    virtual_endpoint: entry.virtual_endpoint.path,
                    status: "up",
                });
            }
        }

        let result = CompositionState {
            primals,
            total_capabilities,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
    }

    /// Handle `lifecycle.validate_consumed` — checks that all consumed capabilities
    /// are satisfiable by currently registered providers in the composition.
    pub(super) async fn handle_validate_consumed(&self, _params: Value) -> Result<Value, String> {
        debug!("Validating consumed capabilities");

        let registry = self.registry.read().await;
        let mut satisfied = Vec::new();
        let mut unsatisfied = Vec::new();

        for &cap in CONSUMED_CAPABILITIES {
            let providers = registry.find_by_capability(cap).await;
            if providers.is_empty() {
                unsatisfied.push(cap.to_string());
            } else {
                satisfied.push(cap.to_string());
            }
        }

        let result = ValidateConsumedResult {
            valid: unsatisfied.is_empty(),
            satisfied,
            unsatisfied,
        };

        serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
    }
}

/// Convert a `NativeEndpoint` to the Phase 2 `TransportEndpoint` wire type.
pub(super) fn transport_endpoint_from_native(ep: &NativeEndpoint) -> TransportEndpoint {
    match ep {
        NativeEndpoint::UnixSocket(path) => TransportEndpoint::Uds {
            path: path.display().to_string(),
        },
        NativeEndpoint::AbstractSocket(name) => TransportEndpoint::Uds {
            path: format!("@{name}"),
        },
        NativeEndpoint::TcpLocal(port) => TransportEndpoint::Tcp {
            host: songbird_types::constants::LOCALHOST.to_string(),
            port: *port,
        },
        NativeEndpoint::NamedPipe(name) => TransportEndpoint::NamedPipe {
            name: name.clone(),
        },
        NativeEndpoint::XPC(service) => TransportEndpoint::Uds {
            path: service.clone(),
        },
        NativeEndpoint::InProcess(id) => TransportEndpoint::Tcp {
            host: songbird_types::constants::LOCALHOST.to_string(),
            port: *id,
        },
        NativeEndpoint::SharedMemory(region) => TransportEndpoint::Uds {
            path: region.clone(),
        },
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn canonical_payload_is_deterministic_regardless_of_cap_order() {
        let a = build_canonical_payload(
            "nestgate",
            &["storage".into(), "crypto".into(), "auth".into()],
            "/tmp/nestgate.sock",
            "2026-04-28T12:00:00Z",
        );
        let b = build_canonical_payload(
            "nestgate",
            &["auth".into(), "crypto".into(), "storage".into()],
            "/tmp/nestgate.sock",
            "2026-04-28T12:00:00Z",
        );
        assert_eq!(a, b, "canonical payload must be order-independent");
    }

    #[test]
    fn canonical_payload_contains_all_fields() {
        let payload = build_canonical_payload(
            "beardog",
            &["crypto".into(), "security".into()],
            "/run/user/1000/biomeos/beardog.sock",
            "2026-04-28T14:30:00Z",
        );
        let parsed: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(parsed["p"], "beardog");
        assert_eq!(parsed["e"], "/run/user/1000/biomeos/beardog.sock");
        assert_eq!(parsed["t"], "2026-04-28T14:30:00Z");
        let caps = parsed["c"].as_array().unwrap();
        assert_eq!(caps[0], "crypto");
        assert_eq!(caps[1], "security");
    }

    #[test]
    fn canonical_payload_empty_capabilities() {
        let payload =
            build_canonical_payload("minimal", &[], "tcp://127.0.0.1:9000", "2026-01-01T00:00:00Z");
        let parsed: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert!(parsed["c"].as_array().unwrap().is_empty());
    }

    #[test]
    fn turn_config_from_env_fails_gracefully_when_not_set() {
        // In CI and dev, SONGBIRD_TURN_SERVER is not set — from_env returns Err
        if songbird_process_env::var("SONGBIRD_TURN_SERVER").is_err() {
            let peer_addr: std::net::SocketAddr = "192.168.1.100:8080".parse().unwrap();
            let result = songbird_turn_client::TurnSessionConfig::from_env(peer_addr);
            assert!(result.is_err(), "Should fail when TURN env vars are absent");
        }
    }

    #[test]
    fn transport_endpoint_from_unix_socket() {
        let ep = NativeEndpoint::UnixSocket("/run/membrane/beardog.sock".into());
        let te = transport_endpoint_from_native(&ep);
        assert_eq!(
            te,
            TransportEndpoint::Uds {
                path: String::from("/run/membrane/beardog.sock")
            }
        );
    }

    #[test]
    fn transport_endpoint_from_abstract_socket() {
        let ep = NativeEndpoint::AbstractSocket("biomeos_security".into());
        let te = transport_endpoint_from_native(&ep);
        assert_eq!(
            te,
            TransportEndpoint::Uds {
                path: String::from("@biomeos_security")
            }
        );
    }

    #[test]
    fn transport_endpoint_from_tcp_local() {
        let ep = NativeEndpoint::TcpLocal(7700);
        let te = transport_endpoint_from_native(&ep);
        assert_eq!(
            te,
            TransportEndpoint::Tcp {
                host: String::from("127.0.0.1"),
                port: 7700
            }
        );
    }

    #[test]
    fn transport_endpoint_from_in_process() {
        let ep = NativeEndpoint::InProcess(42);
        let te = transport_endpoint_from_native(&ep);
        assert_eq!(
            te,
            TransportEndpoint::Tcp {
                host: String::from("127.0.0.1"),
                port: 42
            }
        );
    }
}

/// Inject a capability advertisement into the local swarmVine gossip engine.
///
/// Discovers the swarmVine UDS socket via standard biomeOS resolution, then
/// sends a `gossip.inject` JSON-RPC call with the capability advertisement.
/// Fire-and-forget: if swarmVine is unavailable, the announcement still
/// propagates via the existing songBird `mesh.capabilities_announce` path.
#[cfg(unix)]
async fn inject_to_swarmvine(node_id: &str, primal_id: &str, capabilities: &[String]) {
    let socket = discover_swarmvine_socket();
    let Some(socket_path) = socket else {
        debug!("swarmVine socket not found — skipping gossip inject (not deployed yet)");
        return;
    };

    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "gossip.inject",
        "params": {
            "topic": "tower",
            "key": format!("capability.advertise:{node_id}:{primal_id}"),
            "payload": {
                "capabilities": capabilities,
                "primal": primal_id,
                "gate": node_id,
            }
        },
        "id": null,
    });

    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    match tokio::net::UnixStream::connect(&socket_path).await {
        Ok(stream) => {
            let mut reader = BufReader::new(stream);
            let writer = reader.get_mut();
            // riboCipher clear signal + JSON-RPC
            if writer.write_all(&[0xEC, 0x01]).await.is_err() {
                return;
            }
            let msg = format!("{payload}\n");
            if writer.write_all(msg.as_bytes()).await.is_err() {
                return;
            }
            let mut response = String::new();
            let _ = reader.read_line(&mut response).await;
            debug!(response = %response.trim(), "swarmVine gossip.inject response");
        }
        Err(e) => {
            debug!(error = %e, "swarmVine not reachable — gossip inject skipped");
        }
    }
}

#[cfg(not(unix))]
async fn inject_to_swarmvine(_node_id: &str, _primal_id: &str, _capabilities: &[String]) {
    debug!("swarmVine gossip inject not available on this platform (UDS-only)");
}

/// Discover swarmVine socket via standard biomeOS resolution.
#[cfg(unix)]
fn discover_swarmvine_socket() -> Option<std::path::PathBuf> {
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        let p = std::path::PathBuf::from(format!("{runtime_dir}/biomeos/swarmvine.sock"));
        if p.exists() {
            return Some(p);
        }
    }
    let p = std::path::PathBuf::from("/tmp/biomeos/swarmvine.sock");
    if p.exists() {
        return Some(p);
    }
    None
}
