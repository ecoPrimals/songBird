// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

mod common;
mod federation_health;
mod integrations;
mod services;

pub use federation_health::{
    handle_beacon_exchange, handle_federation_join, handle_federation_peers, handle_health,
    handle_health_standard, handle_identity, handle_protocol_capabilities, handle_version,
};
pub use integrations::{
    handle_compute_job_status, handle_compute_route, handle_consent_check, handle_consent_grant,
    handle_deployment_create, handle_deployment_hot_swap, handle_deployment_list,
    handle_deployment_restart, handle_deployment_status, handle_protocol_negotiate_semantic,
    handle_task_create, handle_task_list,
};
pub use services::{
    handle_registry_discover, handle_registry_register, handle_service_get,
    handle_service_register, handle_services_list,
};
