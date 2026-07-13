// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![forbid(unsafe_code)]

use super::super::IpcServiceHandler;
use serde_json::Value;
use songbird_types::json_rpc_method::{
    BtspMethod, CapabilitiesMethod, HealthMethod, IdentityMethod, JsonRpcMethod, LifecycleMethod,
    PrimalMethod, RpcMethod,
};

pub(super) async fn dispatch_introspection(
    handler: &IpcServiceHandler,
    method: JsonRpcMethod,
    params: Value,
) -> Result<Value, String> {
    match method {
        JsonRpcMethod::Primal(PrimalMethod::Info) => Ok(crate::introspection::primal_info()),
        JsonRpcMethod::Primal(PrimalMethod::Capabilities) => {
            Ok(crate::introspection::primal_capabilities())
        }
        JsonRpcMethod::Primal(PrimalMethod::Announce) => {
            Ok(crate::introspection::primal_announce())
        }
        JsonRpcMethod::Rpc(RpcMethod::Methods) => Ok(crate::introspection::rpc_methods()),
        JsonRpcMethod::Rpc(RpcMethod::Discover) => {
            Ok(crate::introspection::rpc_discover_standard())
        }
        JsonRpcMethod::DiscoverCapabilities => Ok(crate::introspection::discover_capabilities()),

        JsonRpcMethod::Health(HealthMethod::Liveness) => {
            Ok(crate::introspection::health_liveness())
        }
        JsonRpcMethod::Health(HealthMethod::Readiness) => {
            let status = crate::introspection::SubsystemStatus {
                ipc: true,
                ..Default::default()
            };
            Ok(crate::introspection::health_readiness(&status))
        }
        JsonRpcMethod::Health(HealthMethod::Check) => handler.handle_health().await,
        JsonRpcMethod::Capabilities(CapabilitiesMethod::List) => {
            let extra = handler.runtime_capabilities();
            Ok(crate::introspection::capabilities_list_with_runtime(&extra))
        }
        JsonRpcMethod::Capabilities(CapabilitiesMethod::Methods) => {
            Ok(crate::introspection::capabilities_methods())
        }
        JsonRpcMethod::Identity => handler.handle_identity().await,
        JsonRpcMethod::IdentityGet(IdentityMethod::Get) => Ok(crate::introspection::identity_get()),

        JsonRpcMethod::Btsp(BtspMethod::Capabilities) => {
            Ok(crate::introspection::btsp_capabilities())
        }
        JsonRpcMethod::Btsp(BtspMethod::Negotiate) => {
            Err(String::from("btsp.negotiate is handled at the transport layer"))
        }

        JsonRpcMethod::Lifecycle(LifecycleMethod::Composition) => {
            handler.handle_lifecycle_composition(params).await
        }
        JsonRpcMethod::Lifecycle(LifecycleMethod::ValidateConsumed) => {
            handler.handle_validate_consumed(params).await
        }

        other => Err(format!("Unknown method: {other}")),
    }
}
