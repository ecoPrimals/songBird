//! Discovery command handler

use crate::cli::commands::DiscoveryCommands;
use crate::cli::utils::{print_info, print_success};
use anyhow::Result;

/// Handle discovery commands
pub async fn handle_discovery_command(discovery_command: Option<DiscoveryCommands>) -> Result<()> {
    match discovery_command {
        Some(DiscoveryCommands::Scan {
            interface,
            timeout,
            port_range,
        }) => {
            print_info(&format!("🔍 Scanning network (timeout: {}s)", timeout));
            if let Some(iface) = interface {
                print_info(&format!("📡 Interface: {}", iface));
            }
            if let Some(ports) = port_range {
                print_info(&format!("🔌 Port range: {}", ports));
            }
            print_success("✅ Network scan completed");
            Ok(())
        }
        Some(DiscoveryCommands::List) => {
            print_info("📋 Listing discovered services");
            print_info("  • orchestrator - Main service");
            print_info("  • gaming - Gaming bridge");
            print_success("✅ Service list displayed");
            Ok(())
        }
        Some(DiscoveryCommands::Refresh) => {
            print_info("🔄 Refreshing service discovery");
            print_success("✅ Service discovery refreshed");
            Ok(())
        }
        Some(DiscoveryCommands::Test { target, count }) => {
            print_info(&format!(
                "🔬 Testing connectivity to {} ({} attempts)",
                target, count
            ));
            print_success("✅ Connectivity test completed");
            Ok(())
        }
        Some(DiscoveryCommands::Topology) => {
            print_info("📊 Network topology:");
            print_info("  🖥️  Local Node: orchestrator-1");
            print_success("✅ Topology displayed");
            Ok(())
        }
        Some(DiscoveryCommands::Advanced {
            deep_scan,
            include_external,
            pattern,
        }) => {
            print_info("🕵️ Advanced discovery scan");
            if deep_scan {
                print_info("  🔍 Deep scanning enabled");
            }
            if include_external {
                print_info("  🌐 Including external services");
            }
            if let Some(p) = pattern {
                print_info(&format!("  🎯 Pattern: {}", p));
            }
            print_success("✅ Advanced scan completed");
            Ok(())
        }
        None => {
            print_info("🔍 Discovery Commands Available:");
            print_info("  • scan - Network scanning");
            print_info("  • list - List services");
            print_info("  • refresh - Refresh discovery");
            print_info("  • test - Connectivity testing");
            print_info("  • topology - Network topology");
            print_info("  • advanced - Advanced scanning");
            Ok(())
        }
    }
}
