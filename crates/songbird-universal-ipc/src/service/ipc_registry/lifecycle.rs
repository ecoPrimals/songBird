// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::super::{
    CompositionPrimalInfo, CompositionState, IpcServiceHandler, ValidateConsumedResult,
};
use crate::introspection::CONSUMED_CAPABILITIES;
use serde_json::Value;
use tracing::debug;

impl IpcServiceHandler {
    /// Handle `lifecycle.composition` — returns current composition state for dashboards.
    pub(in crate::service) async fn handle_lifecycle_composition(
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
    pub(in crate::service) async fn handle_validate_consumed(
        &self,
        _params: Value,
    ) -> Result<Value, String> {
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
