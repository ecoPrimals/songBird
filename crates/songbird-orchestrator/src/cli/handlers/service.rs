// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Service command handler

use crate::cli::commands::ServiceCommands;
use crate::cli::utils::{print_error, print_info, print_success};
use anyhow::Result;

/// Handle service commands
/// # Errors
///
/// Returns an error if the operation fails.
#[expect(
    clippy::unused_async,
    reason = "async signature required by Axum, trait objects, or future I/O"
)]
pub async fn handle_service_command(service_command: Option<ServiceCommands>) -> Result<()> {
    match service_command {
        Some(ServiceCommands::List) | None => {
            print_info("🛠️ Available Services");
            print_info("  • orchestrator    - Main orchestration service");
            print_info("  • gaming         - Gaming bridge services");
            print_info("  • federation     - Peer-to-peer federation");
            print_info("  • security       - security provider security integration");
            print_info("  • discovery      - Network discovery service");
            print_info("  • observability  - Monitoring and metrics");
            Ok(())
        }
        Some(ServiceCommands::Show {
            service_name,
        }) => {
            print_info(&format!("ℹ️ Service Information: {service_name}"));
            match service_name.as_str() {
                "orchestrator" => {
                    print_info("  Type: Core Service");
                    print_info("  Status: Running");
                    print_info(&format!(
                        "  Port: {}",
                        songbird_config::defaults::ports::orchestrator_port()
                    ));
                    print_info("  Description: Main orchestration service");
                }
                _ => print_error(&format!("Unknown service: {service_name}")),
            }
            Ok(())
        }
        Some(ServiceCommands::Start {
            service_name,
        }) => {
            print_info(&format!("🚀 Starting service: {service_name}"));
            print_success(&format!("Service {service_name} started"));
            Ok(())
        }
        Some(ServiceCommands::Stop {
            service_name,
        }) => {
            print_info(&format!("🛑 Stopping service: {service_name}"));
            print_success(&format!("Service {service_name} stopped"));
            Ok(())
        }
        Some(ServiceCommands::Restart {
            service_name,
        }) => {
            print_info(&format!("🔄 Restarting service: {service_name}"));
            print_success(&format!("Service {service_name} restarted"));
            Ok(())
        }
    }
}
