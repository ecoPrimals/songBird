// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::super::types::Task;
use super::CapabilityRouter;
use super::RoutingDecision;
use songbird_config::capability_endpoints::CapabilityType;
use songbird_network_federation::state::NodeStatus;
use songbird_types::{SongbirdError, SongbirdResult};
use tracing::{debug, info, warn};

impl CapabilityRouter {
    pub(super) async fn route_lightweight_task(
        &self,
        _task: &Task,
    ) -> SongbirdResult<RoutingDecision> {
        debug!("Routing lightweight task");

        if self.has_local_capacity().await {
            debug!("Executing lightweight task locally");
            return Ok(RoutingDecision::ExecuteLocally);
        }

        debug!("Routing lightweight task to peer");
        self.route_to_peer_songbird().await
    }

    pub(super) async fn route_moderate_task(&self, task: &Task) -> SongbirdResult<RoutingDecision> {
        debug!("Routing moderate task");

        match self.route_to_peer_songbird().await {
            Ok(decision) => {
                debug!("Routing moderate task to peer Songbird");
                Ok(decision)
            }
            Err(e) => {
                warn!("No peer Songbirds available: {}, falling back to capability", e);
                self.route_to_specialized_capability(task).await
            }
        }
    }

    pub(super) async fn route_heavy_task(&self, task: &Task) -> SongbirdResult<RoutingDecision> {
        debug!("Routing heavy task to specialized capability");
        self.route_to_specialized_capability(task).await
    }

    pub(super) async fn route_to_specialized_capability(
        &self,
        task: &Task,
    ) -> SongbirdResult<RoutingDecision> {
        let capability_type = Self::determine_capability_type(task);
        debug!("Task requires capability: {:?}", capability_type);

        let capability_type_str = format!("{capability_type:?}");

        if let Some(registry) = &self.capability_registry {
            let capability_name = Self::capability_type_to_name(&capability_type);

            match registry.find_providers_with_capability(&capability_name).await {
                Ok(providers) if !providers.is_empty() => {
                    let provider = &providers[0];
                    let execution_endpoint = format!(
                        "{}{}",
                        provider.registration.endpoint, provider.registration.workload_endpoint
                    );

                    info!(
                        "Routing to external provider '{}' ({}) at: {}",
                        provider.registration.provider_name,
                        provider.registration.provider_id,
                        execution_endpoint
                    );

                    return Ok(RoutingDecision::RouteToExternalProvider {
                        provider_id: provider.registration.provider_id.clone(),
                        execution_endpoint,
                        capability_name,
                    });
                }
                Ok(_) => {
                    debug!("No external providers found for capability: {}", capability_name);
                }
                Err(e) => {
                    warn!("Error querying capability registry: {}", e);
                }
            }
        }

        let endpoint =
            self.capability_resolver.get_endpoint(capability_type.clone()).await.map_err(|e| {
                SongbirdError::service(
                    capability_type_str.clone(),
                    format!("No capability provider found: {e}"),
                )
            })?;

        info!("Routing to {} capability at: {}", capability_type_str, endpoint);

        Ok(RoutingDecision::RouteToCapability {
            capability_type,
            provider_endpoint: endpoint,
        })
    }

    pub(super) fn capability_type_to_name(cap_type: &CapabilityType) -> String {
        match cap_type {
            CapabilityType::Compute => String::from("compute_heavy"),
            CapabilityType::Security => String::from("security"),
            CapabilityType::Ai => String::from("ai_inference"),
            CapabilityType::Storage => String::from("storage"),
            CapabilityType::Orchestration => String::from("orchestration"),
            CapabilityType::Observability => String::from("observability"),
            CapabilityType::Networking => String::from("networking"),
            CapabilityType::Custom(name) => name.clone(),
        }
    }

    pub(super) async fn route_to_peer_songbird(&self) -> SongbirdResult<RoutingDecision> {
        let nodes = self.federation_state.nodes.read().await;

        for (node_id, registration) in nodes.iter() {
            if registration.status == NodeStatus::Active {
                debug!("Found available peer: {} at {}", node_id, registration.node_address);
                return Ok(RoutingDecision::RouteToSongbird {
                    node_id: node_id.clone(),
                    endpoint: registration.node_address.clone(),
                });
            }
        }

        Err(SongbirdError::service("federation", "No available peer Songbirds found in federation"))
    }

    pub(super) fn determine_capability_type(task: &Task) -> CapabilityType {
        if task.resource_requirements.as_ref().is_some_and(|r| r.gpu_required) {
            return CapabilityType::Compute;
        }

        match task.task_type.as_ref() {
            "ml_training" | "gpu_compute" | "batch_processing" | "video_processing" => {
                CapabilityType::Compute
            }

            "encrypt" | "decrypt" | "sign" | "verify" | "auth" => CapabilityType::Security,

            "inference" | "ai_query" | "model_serve" => CapabilityType::Ai,

            "store" | "retrieve" | "backup" => CapabilityType::Storage,

            _ => CapabilityType::Compute,
        }
    }

    #[expect(
        clippy::unused_async,
        reason = "async signature required by Axum, trait objects, or future I/O"
    )]
    pub(super) async fn has_local_capacity(&self) -> bool {
        true
    }
}
