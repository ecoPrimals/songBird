// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Dynamic JSON-RPC-style dispatch (`call_method`) for protocol-agnostic adapters.

use serde_json::Value;
use tracing::debug;

use crate::tarpc_types::ServiceRegistration;
use songbird_types::{SongbirdError, SongbirdResult};

use super::TarpcClient;

impl TarpcClient {
    /// Dynamic JSON dispatch for adapter integration (`discover`, `health`, …).
    pub async fn call_method(&self, method: &str, params: Option<Value>) -> SongbirdResult<Value> {
        debug!("Calling method: {} with params: {:?}", method, params);

        match method {
            "discover" => {
                let capability = params
                    .as_ref()
                    .and_then(|v| v.get("capability"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| SongbirdError::rpc("Missing capability parameter"))?
                    .to_string();

                let result = self.discover(&capability).await?;
                serde_json::to_value(result)
                    .map_err(|e| SongbirdError::serialization(format!("Failed to serialize: {e}")))
            }
            "discover_all" => {
                let result = self.discover_all().await?;
                serde_json::to_value(result)
                    .map_err(|e| SongbirdError::serialization(format!("Failed to serialize: {e}")))
            }
            "register" => {
                let registration: ServiceRegistration = serde_json::from_value(
                    params.ok_or_else(|| SongbirdError::rpc("Missing registration parameter"))?,
                )
                .map_err(|e| SongbirdError::serialization(format!("Invalid registration: {e}")))?;

                let result = self.register(registration).await?;
                serde_json::to_value(result)
                    .map_err(|e| SongbirdError::serialization(format!("Failed to serialize: {e}")))
            }
            "unregister" => {
                let service_id = params
                    .as_ref()
                    .and_then(|v| v.get("service_id"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| SongbirdError::rpc("Missing service_id parameter"))?
                    .to_string();

                let result = self.unregister(&service_id).await?;
                serde_json::to_value(result)
                    .map_err(|e| SongbirdError::serialization(format!("Failed to serialize: {e}")))
            }
            "health" => {
                let result = self.health().await?;
                serde_json::to_value(result)
                    .map_err(|e| SongbirdError::serialization(format!("Failed to serialize: {e}")))
            }
            "version" => {
                let result = self.version().await?;
                serde_json::to_value(result)
                    .map_err(|e| SongbirdError::serialization(format!("Failed to serialize: {e}")))
            }
            "protocols" => {
                let result = self.protocols().await?;
                serde_json::to_value(result)
                    .map_err(|e| SongbirdError::serialization(format!("Failed to serialize: {e}")))
            }
            _ => Err(SongbirdError::rpc(format!("Unknown method: {method}"))),
        }
    }
}
