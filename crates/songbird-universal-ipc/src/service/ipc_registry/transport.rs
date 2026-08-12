// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::super::TransportEndpoint;
use crate::endpoint::NativeEndpoint;

/// Convert a `NativeEndpoint` to the Phase 2 `TransportEndpoint` wire type.
pub(in crate::service) fn transport_endpoint_from_native(ep: &NativeEndpoint) -> TransportEndpoint {
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
