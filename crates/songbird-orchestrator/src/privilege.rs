//! Secure Privilege Management
//!
//! Handles privilege elevation securely without exposing sudo to users.
//! Uses Linux capabilities and polkit for granular permission management.

use anyhow::{anyhow, Result};
use std::process::Command;
use tracing::{debug, info, warn};

/// Privilege Manager
///
/// Manages system privileges securely for network operations.
/// Aims to minimize or eliminate need for sudo prompts.
pub struct PrivilegeManager {
    /// Whether we have network admin capabilities
    has_net_admin: bool,
    
    /// Whether we're running with elevated privileges
    is_elevated: bool,
}

impl PrivilegeManager {
    /// Create a new privilege manager and detect current capabilities
    pub fn new() -> Self {
        let has_net_admin = Self::check_net_admin_capability();
        let is_elevated = Self::check_elevated();
        
        debug!("🔐 Privilege detection: net_admin={}, elevated={}", has_net_admin, is_elevated);
        
        Self {
            has_net_admin,
            is_elevated,
        }
    }
    
    /// Check if we have CAP_NET_ADMIN capability
    fn check_net_admin_capability() -> bool {
        // Check via getcap on our own binary
        // Note: This requires the binary to have capabilities set
        if let Ok(output) = Command::new("getcap")
            .arg("/proc/self/exe")
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return stdout.contains("cap_net_admin");
        }
        
        false
    }
    
    /// Check if running with elevated privileges
    fn check_elevated() -> bool {
        // Check effective UID via environment variable (safe alternative to libc)
        std::env::var("EUID")
            .ok()
            .and_then(|euid| euid.parse::<u32>().ok())
            .map(|euid| euid == 0)
            .unwrap_or(false)
    }
    
    /// Configure firewall rules for Songbird ports
    ///
    /// This is the **secure** way to handle firewall configuration:
    /// 1. Use systemd service with capabilities (no sudo needed)
    /// 2. Or use polkit policy for granular permission
    /// 3. Fallback: Provide user instructions (no auto-sudo!)
    pub fn configure_firewall(&self, ports: &[u16]) -> Result<()> {
        if self.has_net_admin {
            info!("🔐 Configuring firewall with CAP_NET_ADMIN capability");
            return self.configure_firewall_with_capability(ports);
        }
        
        if self.is_elevated {
            info!("🔐 Configuring firewall with elevated privileges");
            return self.configure_firewall_with_privilege(ports);
        }
        
        // Can't auto-configure - provide instructions
        warn!("⚠️  No firewall management capability detected");
        self.provide_firewall_instructions(ports);
        
        Ok(())
    }
    
    /// Configure firewall using CAP_NET_ADMIN capability
    fn configure_firewall_with_capability(&self, ports: &[u16]) -> Result<()> {
        for port in ports {
            // TCP
            let status = Command::new("iptables")
                .args(["-I", "INPUT", "-p", "tcp", "--dport", &port.to_string(), "-j", "ACCEPT"])
                .status()?;
            
            if !status.success() {
                return Err(anyhow!("Failed to add iptables rule for TCP port {}", port));
            }
            
            // UDP
            let status = Command::new("iptables")
                .args(["-I", "INPUT", "-p", "udp", "--dport", &port.to_string(), "-j", "ACCEPT"])
                .status()?;
            
            if !status.success() {
                return Err(anyhow!("Failed to add iptables rule for UDP port {}", port));
            }
            
            info!("✅ Firewall configured for port {}", port);
        }
        
        Ok(())
    }
    
    /// Configure firewall with elevated privileges
    fn configure_firewall_with_privilege(&self, ports: &[u16]) -> Result<()> {
        // Same as capability method, but we're running as root
        self.configure_firewall_with_capability(ports)
    }
    
    /// Provide user instructions for manual firewall configuration
    fn provide_firewall_instructions(&self, ports: &[u16]) {
        info!("");
        info!("╔═══════════════════════════════════════════════════════════╗");
        info!("║  🔐 FIREWALL CONFIGURATION REQUIRED                        ║");
        info!("╚═══════════════════════════════════════════════════════════╝");
        info!("");
        info!("Songbird needs firewall rules for the following ports:");
        for port in ports {
            info!("  • Port {}: TCP (HTTPS) and UDP (Discovery)", port);
        }
        info!("");
        info!("Option 1: Set Binary Capabilities (Recommended)");
        info!("  sudo setcap cap_net_admin+ep target/release/songbird-orchestrator");
        info!("  # Then restart Songbird (no sudo needed!)");
        info!("");
        info!("Option 2: Manual iptables Rules");
        for port in ports {
            info!("  sudo iptables -I INPUT -p tcp --dport {} -j ACCEPT", port);
            info!("  sudo iptables -I INPUT -p udp --dport {} -j ACCEPT", port);
        }
        info!("  sudo iptables-save | sudo tee /etc/iptables/rules.v4");
        info!("");
        info!("Option 3: Systemd Service (Most Secure)");
        info!("  # Create /etc/systemd/system/songbird.service with:");
        info!("  [Service]");
        info!("  AmbientCapabilities=CAP_NET_BIND_SERVICE CAP_NET_ADMIN");
        info!("  # Then: sudo systemctl daemon-reload && sudo systemctl start songbird");
        info!("");
        info!("╚═══════════════════════════════════════════════════════════╝");
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
        println!("Privilege manager created: has_net_admin={}, is_elevated={}", 
                 manager.has_net_admin, manager.is_elevated);
    }
    
    #[test]
    fn test_firewall_instructions() {
        let manager = PrivilegeManager::new();
        manager.provide_firewall_instructions(&[8080, 2300]);
        // Should print instructions without panicking
    }
}

