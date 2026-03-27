// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Secure Privilege Management - User Collaboration
//!
//! Philosophy (Dec 20, 2025):
//!   "Work WITH users on permissions, not around them"
//!
//! Instead of trying to silently circumvent permission issues (like `SO_REUSEPORT`),
//! we collaborate with users:
//!   - Detect what's needed
//!   - Explain clearly
//!   - Offer to help configure
//!   - Guide through process
//!   - Verify it worked
//!
//! This builds trust and sovereignty rather than creating confusion.

use anyhow::{Result, anyhow};
use std::io::{self, Write};
use std::process::Command;
use tracing::{debug, error, info};

/// Privilege Manager
///
/// Manages system privileges securely through user collaboration.
/// Detects needs, explains clearly, offers help, verifies success.
pub struct PrivilegeManager {
    /// Whether we have network admin capabilities
    has_net_admin: bool,

    /// Whether we're running with elevated privileges
    is_elevated: bool,

    /// Whether to run in interactive mode (ask user for help)
    interactive: bool,
}

impl PrivilegeManager {
    /// Create a new privilege manager and detect current capabilities
    #[must_use]
    pub fn new() -> Self {
        Self::with_interactive(true)
    }

    /// Create a privilege manager with specific interactivity setting
    pub fn with_interactive(interactive: bool) -> Self {
        let has_net_admin = Self::check_net_admin_capability();
        let is_elevated = Self::check_elevated();

        debug!(
            "🔐 Privilege detection: net_admin={}, elevated={}, interactive={}",
            has_net_admin, is_elevated, interactive
        );

        Self {
            has_net_admin,
            is_elevated,
            interactive,
        }
    }

    /// Check if we have `CAP_NET_ADMIN` capability
    fn check_net_admin_capability() -> bool {
        // Check via getcap on our own binary
        // Note: This requires the binary to have capabilities set
        if let Ok(output) = Command::new("getcap").arg("/proc/self/exe").output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return stdout.contains("cap_net_admin");
        }

        false
    }

    /// Check if running with elevated privileges
    fn check_elevated() -> bool {
        // Check effective UID via environment variable (safe alternative to libc)
        songbird_process_env::var("EUID")
            .ok()
            .and_then(|euid| euid.parse::<u32>().ok())
            .is_some_and(|euid| euid == 0)
    }

    /// Configure firewall rules for Songbird ports
    ///
    /// This is the **collaborative** way to handle firewall configuration:
    /// 1. Detect what's needed
    /// 2. Check if we can do it automatically
    /// 3. If not, explain clearly to user
    /// 4. Offer to help configure
    /// 5. Verify it worked
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn configure_firewall(&self, ports: &[u16]) -> Result<()> {
        info!("🔐 Checking firewall configuration...");

        // Check if firewall rules already exist
        if self.check_firewall_rules(ports) {
            info!("✅ Firewall rules already configured");
            return Ok(());
        }

        // Try automatic configuration if we have permissions
        if self.has_net_admin || self.is_elevated {
            info!("🔐 Configuring firewall automatically...");
            return self.configure_firewall_auto(ports);
        }

        // Interactive mode: collaborate with user
        if self.interactive {
            return self.collaborate_on_firewall(ports);
        }

        // Non-interactive: just provide instructions
        self.provide_firewall_instructions(ports);
        Ok(())
    }

    /// Check if firewall rules exist for the given ports
    fn check_firewall_rules(&self, ports: &[u16]) -> bool {
        for port in ports {
            let tcp_check = Command::new("iptables")
                .args(["-C", "INPUT", "-p", "tcp", "--dport", &port.to_string(), "-j", "ACCEPT"])
                .output();

            let udp_check = Command::new("iptables")
                .args(["-C", "INPUT", "-p", "udp", "--dport", &port.to_string(), "-j", "ACCEPT"])
                .output();

            match (tcp_check, udp_check) {
                (Ok(tcp), Ok(udp)) if tcp.status.success() && udp.status.success() => {}
                _ => return false,
            }
        }

        true
    }

    /// Configure firewall automatically (with permissions)
    fn configure_firewall_auto(&self, ports: &[u16]) -> Result<()> {
        for port in ports {
            // TCP
            let status = Command::new("iptables")
                .args(["-I", "INPUT", "-p", "tcp", "--dport", &port.to_string(), "-j", "ACCEPT"])
                .status()?;

            if !status.success() {
                return Err(anyhow!("Failed to add iptables rule for TCP port {port}"));
            }

            // UDP
            let status = Command::new("iptables")
                .args(["-I", "INPUT", "-p", "udp", "--dport", &port.to_string(), "-j", "ACCEPT"])
                .status()?;

            if !status.success() {
                return Err(anyhow!("Failed to add iptables rule for UDP port {port}"));
            }

            info!("✅ Firewall configured for port {}", port);
        }

        Ok(())
    }

    /// Collaborate with user on firewall configuration
    fn collaborate_on_firewall(&self, ports: &[u16]) -> Result<()> {
        info!("");
        info!("╔═══════════════════════════════════════════════════════════════════╗");
        info!("║                                                                   ║");
        info!("║  🔧 NETWORK CONFIGURATION NEEDED                                  ║");
        info!("║                                                                   ║");
        info!("╚═══════════════════════════════════════════════════════════════════╝");
        info!("");
        info!("Songbird needs to accept connections on these ports:");
        for port in ports {
            info!("  • Port {}: TCP (HTTPS) and UDP (Discovery)", port);
        }
        info!("");
        info!("I can help you configure this. The commands I'll run:");
        info!("");
        for port in ports {
            info!("  sudo iptables -I INPUT -p tcp --dport {} -j ACCEPT", port);
            info!("  sudo iptables -I INPUT -p udp --dport {} -j ACCEPT", port);
        }
        info!("");

        // Ask user permission
        print!("Would you like me to run these commands for you? (y/n): ");
        io::stdout().flush()?;

        let mut response = String::new();
        io::stdin().read_line(&mut response)?;

        if response.trim().to_lowercase() == "y" {
            info!("Running configuration commands...");

            for port in ports {
                // TCP
                let status = Command::new("sudo")
                    .args([
                        "iptables",
                        "-I",
                        "INPUT",
                        "-p",
                        "tcp",
                        "--dport",
                        &port.to_string(),
                        "-j",
                        "ACCEPT",
                    ])
                    .status()?;

                if !status.success() {
                    error!("❌ Failed to configure TCP port {}", port);
                    return Err(anyhow!("Firewall configuration failed"));
                }

                // UDP
                let status = Command::new("sudo")
                    .args([
                        "iptables",
                        "-I",
                        "INPUT",
                        "-p",
                        "udp",
                        "--dport",
                        &port.to_string(),
                        "-j",
                        "ACCEPT",
                    ])
                    .status()?;

                if !status.success() {
                    error!("❌ Failed to configure UDP port {}", port);
                    return Err(anyhow!("Firewall configuration failed"));
                }

                info!("✅ Port {} configured", port);
            }

            info!("");
            info!("╔═══════════════════════════════════════════════════════════════════╗");
            info!("║  ✅ FIREWALL CONFIGURATION COMPLETE                               ║");
            info!("╚═══════════════════════════════════════════════════════════════════╝");
            info!("");

            // Offer to persist
            print!("Make these rules persistent across reboots? (y/n): ");
            io::stdout().flush()?;

            response.clear();
            io::stdin().read_line(&mut response)?;

            if response.trim().to_lowercase() == "y" {
                let status = Command::new("sudo")
                    .args(["iptables-save"])
                    .stdout(std::process::Stdio::piped())
                    .spawn()?
                    .wait_with_output()?;

                if status.status.success() {
                    std::fs::write("/tmp/iptables-rules.v4", status.stdout)?;
                    let copy_status = Command::new("sudo")
                        .args(["cp", "/tmp/iptables-rules.v4", "/etc/iptables/rules.v4"])
                        .status()?;

                    if copy_status.success() {
                        info!("✅ Rules saved to /etc/iptables/rules.v4");
                    }
                }
            }
        } else {
            info!("No problem! You can run these commands manually later.");
            self.provide_firewall_instructions(ports);
        }

        Ok(())
    }

    /// Provide user instructions for manual firewall configuration
    fn provide_firewall_instructions(&self, ports: &[u16]) {
        info!("");
        info!("╔═══════════════════════════════════════════════════════════════════╗");
        info!("║  📋 FIREWALL CONFIGURATION INSTRUCTIONS                           ║");
        info!("╚═══════════════════════════════════════════════════════════════════╝");
        info!("");
        info!("Songbird needs firewall rules for the following ports:");
        for port in ports {
            info!("  • Port {}: TCP (HTTPS) and UDP (Discovery)", port);
        }
        info!("");
        info!("╔═══════════════════════════════════════════════════════════════════╗");
        info!("║  Option 1: Quick Fix (Temporary)                                 ║");
        info!("╚═══════════════════════════════════════════════════════════════════╝");
        for port in ports {
            info!("  sudo iptables -I INPUT -p tcp --dport {} -j ACCEPT", port);
            info!("  sudo iptables -I INPUT -p udp --dport {} -j ACCEPT", port);
        }
        info!("");
        info!("╔═══════════════════════════════════════════════════════════════════╗");
        info!("║  Option 2: Persistent Rules (Recommended)                        ║");
        info!("╚═══════════════════════════════════════════════════════════════════╝");
        for port in ports {
            info!("  sudo iptables -I INPUT -p tcp --dport {} -j ACCEPT", port);
            info!("  sudo iptables -I INPUT -p udp --dport {} -j ACCEPT", port);
        }
        info!("  sudo iptables-save | sudo tee /etc/iptables/rules.v4");
        info!("");
        info!("╔═══════════════════════════════════════════════════════════════════╗");
        info!("║  Option 3: Binary Capabilities (No sudo needed!)                 ║");
        info!("╚═══════════════════════════════════════════════════════════════════╝");
        info!("  sudo setcap cap_net_admin+ep target/release/songbird-orchestrator");
        info!("  # Then restart Songbird - it will configure itself!");
        info!("");
        info!("╔═══════════════════════════════════════════════════════════════════╗");
        info!("║  Option 4: Systemd Service (Most Secure & Automatic)             ║");
        info!("╚═══════════════════════════════════════════════════════════════════╝");
        info!("  Create /etc/systemd/system/songbird.service:");
        info!("");
        info!("  [Service]");
        info!("  ExecStart=/path/to/songbird-orchestrator");
        info!("  AmbientCapabilities=CAP_NET_BIND_SERVICE CAP_NET_ADMIN");
        info!("  User=your-username");
        info!("");
        info!("  sudo systemctl daemon-reload");
        info!("  sudo systemctl start songbird");
        info!("");
        info!("╚═══════════════════════════════════════════════════════════════════╝");
        info!("");
    }
}

impl Default for PrivilegeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_privilege_manager_creation() {
        let manager = PrivilegeManager::new();
        // Should not panic
        println!(
            "Privilege manager created: has_net_admin={}, is_elevated={}",
            manager.has_net_admin, manager.is_elevated
        );
    }

    #[test]
    fn test_non_interactive_mode() {
        let manager = PrivilegeManager::with_interactive(false);
        // Should provide instructions without asking for input
        manager.provide_firewall_instructions(&[8080, 2300]);
    }

    #[test]
    fn test_firewall_rule_check() {
        let manager = PrivilegeManager::new();
        // Should not panic even if iptables not available
        let _has_rules = manager.check_firewall_rules(&[8080]);
    }
}
