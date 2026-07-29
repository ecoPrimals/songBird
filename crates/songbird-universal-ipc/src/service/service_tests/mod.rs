// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, reason = "test assertions")]
#![allow(clippy::expect_used, reason = "test assertions")]
#![allow(clippy::unchecked_time_subtraction, reason = "test time arithmetic")]

mod capability_resolve;
mod introspection_federation;
mod ipc_core;
mod peer_capability_routing;
mod transport_endpoint_wire;
