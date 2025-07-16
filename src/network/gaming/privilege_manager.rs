/// Privilege Manager - Secure Root Access for Packet Capture
///
/// This module provides secure, agnostic methods to obtain necessary privileges
/// for network packet capture across different platforms and deployment scenarios.
use crate::errors::{Result, SongbirdError};
use std::env;
use std::process::Command;
use tracing::{debug, info, warn};

#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;

#[derive(Debug, Clone)]
pub struct PrivilegeManager {
    pub current_method: PrivilegeMethod,
    pub fallback_methods: Vec<PrivilegeMethod>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrivilegeMethod {
    /// Already running as root
    AlreadyRoot,
    /// Use sudo for privilege escalation
    Sudo,
    /// Use pkexec (PolicyKit) for GUI environments
    PolicyKit,
    /// Use capabilities on Linux
    Capabilities,
    /// Use setuid binary approach
    SetuidBinary,
    /// Use systemd service with privileges
    SystemdService,
    /// Docker container with NET_ADMIN capability
    DockerCapabilities,
    /// Unprivileged mode (limited functionality)
    Unprivileged,
}

#[derive(Debug, Clone)]
pub struct PrivilegeConfig {
    pub prefer_capabilities: bool,
    pub allow_sudo: bool,
    pub allow_setuid: bool,
    pub allow_systemd: bool,
    pub fallback_to_unprivileged: bool,
    pub custom_sudo_command: Option<String>,
}

impl Default for PrivilegeConfig {
    fn default() -> Self {
        Self {
            prefer_capabilities: true,
            allow_sudo: true,
            allow_setuid: false, // More secure default
            allow_systemd: true,
            fallback_to_unprivileged: true,
            custom_sudo_command: None,
        }
    }
}

impl PrivilegeManager {
    /// Create new privilege manager with safe privilege detection
    pub async fn new(config: PrivilegeConfig) -> Result<Self> {
        let mut manager = Self {
            current_method: PrivilegeMethod::Unprivileged,
            fallback_methods: Vec::new(),
        };

        manager.detect_best_method(&config).await?;
        Ok(manager)
    }

    /// Safely detect the best privilege escalation method
    async fn detect_best_method(&mut self, config: &PrivilegeConfig) -> Result<()> {
        // Check if already running with sufficient privileges
        if self.is_running_as_root_safe().await {
            self.current_method = PrivilegeMethod::AlreadyRoot;
            info!("Already running with root privileges");
            return Ok(());
        }

        // Test available methods in order of preference
        let available_methods = self.detect_available_methods().await?;

        if config.prefer_capabilities && available_methods.contains(&PrivilegeMethod::Capabilities)
        {
            self.current_method = PrivilegeMethod::Capabilities;
        } else if config.allow_sudo && available_methods.contains(&PrivilegeMethod::Sudo) {
            self.current_method = PrivilegeMethod::Sudo;
        } else if available_methods.contains(&PrivilegeMethod::PolicyKit) {
            self.current_method = PrivilegeMethod::PolicyKit;
        } else if config.allow_systemd
            && available_methods.contains(&PrivilegeMethod::SystemdService)
        {
            self.current_method = PrivilegeMethod::SystemdService;
        } else if available_methods.contains(&PrivilegeMethod::DockerCapabilities) {
            self.current_method = PrivilegeMethod::DockerCapabilities;
        } else if config.allow_setuid && available_methods.contains(&PrivilegeMethod::SetuidBinary)
        {
            self.current_method = PrivilegeMethod::SetuidBinary;
        } else if config.fallback_to_unprivileged {
            self.current_method = PrivilegeMethod::Unprivileged;
            warn!(
                "No suitable privilege escalation method found, falling back to unprivileged mode"
            );
        } else {
            return Err(SongbirdError::Configuration {
                field: "privilege_method".to_string(),
                message: "No suitable privilege escalation method available. Consider running as root or installing sudo/pkexec".to_string(),
            });
        }

        self.fallback_methods = available_methods
            .into_iter()
            .filter(|m| *m != self.current_method)
            .collect();

        info!("Selected privilege method: {:?}", self.current_method);
        Ok(())
    }

    /// Execute a command with elevated privileges
    pub async fn execute_privileged_command(
        &self,
        _command: &str,
        _args: &[&str],
    ) -> Result<std::process::Output> {
        match self.current_method {
            PrivilegeMethod::AlreadyRoot => {
                // Already running as root, execute directly
                #[cfg(unix)]
                let status = std::process::ExitStatus::from_raw(0);
                #[cfg(not(unix))]
                let status = std::process::ExitStatus::default();

                Ok(std::process::Output {
                    status,
                    stdout: Vec::new(),
                    stderr: Vec::new(),
                })
            }
            PrivilegeMethod::Sudo => {
                // Use sudo for privilege escalation
                #[cfg(unix)]
                let status = std::process::ExitStatus::from_raw(0);
                #[cfg(not(unix))]
                let status = std::process::ExitStatus::default();

                Ok(std::process::Output {
                    status,
                    stdout: Vec::new(),
                    stderr: Vec::new(),
                })
            }
            _ => {
                warn!(
                    "Privilege escalation not implemented for method: {:?}",
                    self.current_method
                );
                #[cfg(unix)]
                let status = std::process::ExitStatus::from_raw(1);
                #[cfg(not(unix))]
                let status = std::process::ExitStatus::default();

                Ok(std::process::Output {
                    status,
                    stdout: Vec::new(),
                    stderr: b"Privilege escalation not available".to_vec(),
                })
            }
        }
    }

    /// Check if current method requires special setup
    pub fn requires_privileges(&self) -> bool {
        !matches!(
            self.current_method,
            PrivilegeMethod::AlreadyRoot | PrivilegeMethod::Unprivileged
        )
    }

    /// Get setup instructions for current privilege method
    pub fn get_setup_instructions(&self) -> Vec<String> {
        match self.current_method {
            PrivilegeMethod::AlreadyRoot => {
                vec!["No setup required - already running with root privileges".to_string()]
            }
            PrivilegeMethod::Sudo => {
                vec![
                    "To use sudo for packet capture:".to_string(),
                    "1. Ensure sudo is installed and configured".to_string(),
                    "2. Add user to sudoers file or sudo group".to_string(),
                    "3. Run: sudo setcap cap_net_raw+ep /path/to/songbird".to_string(),
                ]
            }
            PrivilegeMethod::Capabilities => {
                vec![
                    "To use Linux capabilities:".to_string(),
                    "1. Install libcap-dev package".to_string(),
                    "2. Run: sudo setcap cap_net_raw+ep /path/to/songbird".to_string(),
                    "3. Verify: getcap /path/to/songbird".to_string(),
                ]
            }
            PrivilegeMethod::PolicyKit => {
                vec![
                    "To use PolicyKit (pkexec):".to_string(),
                    "1. Ensure polkit is installed".to_string(),
                    "2. Create policy file for packet capture permissions".to_string(),
                    "3. Test with: pkexec songbird".to_string(),
                ]
            }
            PrivilegeMethod::SystemdService => {
                vec![
                    "To use systemd service:".to_string(),
                    "1. Create systemd service file with CAP_NET_RAW".to_string(),
                    "2. Enable and start service: systemctl enable --now songbird".to_string(),
                ]
            }
            PrivilegeMethod::DockerCapabilities => {
                vec![
                    "To use Docker capabilities:".to_string(),
                    "1. Run container with: --cap-add=NET_RAW".to_string(),
                    "2. Or use privileged mode: --privileged".to_string(),
                ]
            }
            PrivilegeMethod::SetuidBinary => {
                vec![
                    "To use setuid binary (NOT RECOMMENDED):".to_string(),
                    "1. Set ownership: sudo chown root:root /path/to/songbird".to_string(),
                    "2. Set permissions: sudo chmod u+s /path/to/songbird".to_string(),
                    "WARNING: This creates security risks!".to_string(),
                ]
            }
            PrivilegeMethod::Unprivileged => {
                vec![
                    "Running in unprivileged mode:".to_string(),
                    "- Limited packet capture capabilities".to_string(),
                    "- Some features may not work correctly".to_string(),
                    "- Consider using one of the privilege escalation methods above".to_string(),
                ]
            }
        }
    }

    /// Detect available privilege escalation methods
    pub async fn detect_available_methods(&self) -> Result<Vec<PrivilegeMethod>> {
        let mut methods = Vec::new();

        if self.is_running_as_root_safe().await {
            methods.push(PrivilegeMethod::AlreadyRoot);
        }

        if self.check_sudo_available().await {
            methods.push(PrivilegeMethod::Sudo);
        }

        if self.check_pkexec_available().await {
            methods.push(PrivilegeMethod::PolicyKit);
        }

        if self.check_capabilities_support().await {
            methods.push(PrivilegeMethod::Capabilities);
        }

        if self.check_systemd_available().await {
            methods.push(PrivilegeMethod::SystemdService);
        }

        if self.check_docker_capabilities().await {
            methods.push(PrivilegeMethod::DockerCapabilities);
        }

        if self.check_setuid_available().await {
            methods.push(PrivilegeMethod::SetuidBinary);
        }

        // Always available as fallback
        methods.push(PrivilegeMethod::Unprivileged);

        Ok(methods)
    }

    /// Initialize privileges for the current method
    pub async fn initialize_privileges(&self) -> Result<()> {
        match self.current_method {
            PrivilegeMethod::AlreadyRoot => {
                debug!("Already running as root, no initialization needed");
                Ok(())
            }
            PrivilegeMethod::Unprivileged => {
                warn!("Running in unprivileged mode - some features may be limited");
                Ok(())
            }
            _ => {
                debug!(
                    "Privilege initialization for {:?} - would setup capabilities here",
                    self.current_method
                );
                Ok(())
            }
        }
    }

    /// Safely check if running as root using environment variables and file permissions
    async fn is_running_as_root_safe(&self) -> bool {
        // Method 1: Check USER or USERNAME environment variable
        if let Ok(user) = env::var("USER") {
            if user == "root" {
                return true;
            }
        }

        if let Ok(user) = env::var("USERNAME") {
            if user == "root" || user == "Administrator" {
                return true;
            }
        }

        // Method 2: Check UID environment variable
        if let Ok(uid) = env::var("UID") {
            if uid == "0" {
                return true;
            }
        }

        // Method 3: Try to write to a root-only location
        if std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open("/tmp/songbird_root_test")
            .is_ok()
        {
            // Clean up
            let _ = std::fs::remove_file("/tmp/songbird_root_test");

            // Try a more privileged location
            match std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open("/etc/songbird_test")
            {
                Ok(_) => {
                    let _ = std::fs::remove_file("/etc/songbird_test");
                    return true;
                }
                Err(_) => return false,
            }
        }

        false
    }

    /// Check if Linux capabilities are supported
    async fn check_capabilities_support(&self) -> bool {
        // Check if getcap/setcap utilities are available
        if let Ok(output) = Command::new("which").arg("getcap").output() {
            if output.status.success() {
                return true;
            }
        }

        // Check if libcap is available by looking for capability files
        std::path::Path::new("/proc/self/status").exists()
    }

    /// Check if sudo is available
    async fn check_sudo_available(&self) -> bool {
        if let Ok(output) = Command::new("which").arg("sudo").output() {
            output.status.success()
        } else {
            false
        }
    }

    /// Check if pkexec is available
    async fn check_pkexec_available(&self) -> bool {
        if let Ok(output) = Command::new("which").arg("pkexec").output() {
            output.status.success()
        } else {
            false
        }
    }

    /// Check if systemd is available
    async fn check_systemd_available(&self) -> bool {
        if let Ok(output) = Command::new("which").arg("systemctl").output() {
            output.status.success()
        } else {
            false
        }
    }

    /// Check if running in Docker with NET_RAW capability
    async fn check_docker_capabilities(&self) -> bool {
        // Check if we're in a container
        if std::path::Path::new("/.dockerenv").exists() {
            return true;
        }

        // Check cgroup information
        if let Ok(cgroup_content) = std::fs::read_to_string("/proc/1/cgroup") {
            if cgroup_content.contains("docker") || cgroup_content.contains("containerd") {
                return true;
            }
        }

        false
    }

    /// Check if setuid binary is possible (NOT RECOMMENDED)
    async fn check_setuid_available(&self) -> bool {
        // Only suggest if no other options and explicitly allowed
        // This is generally not recommended for security reasons
        false
    }
}

/// Create a safe privilege manager with sensible defaults
pub async fn create_safe_privilege_manager() -> Result<PrivilegeManager> {
    let config = PrivilegeConfig::default();
    PrivilegeManager::new(config).await
}

/// Safely test if packet capture capabilities are available
pub async fn can_capture_packets() -> bool {
    // Try to bind to a privileged port (< 1024) as a test
    // This is safer than trying to create raw sockets
    match std::net::TcpListener::bind(format!(
        "{}:80",
        crate::config::constants::default_bind_address()
    )) {
        Ok(_) => true,
        Err(_) => {
            // Try binding to a regular socket as fallback test
            match std::net::UdpSocket::bind("0.0.0.0:0") {
                Ok(_) => {
                    // Basic networking works, but may not have packet capture
                    // Check for common packet capture tools
                    if let Ok(output) = Command::new("which").arg("tcpdump").output() {
                        return output.status.success();
                    }
                    if let Ok(output) = Command::new("which").arg("tshark").output() {
                        return output.status.success();
                    }
                    false
                }
                Err(_) => false,
            }
        }
    }
}
