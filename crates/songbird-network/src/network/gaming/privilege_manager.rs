/// Privilege Manager - Secure Root Access for Packet Capture
///
/// This module provides secure, agnostic methods to obtain necessary privileges
/// for network packet capture across different platforms and deployment scenarios.
use songbird_errors::{Result, SongbirdError};
use std::env;
use std::process::Command;
use tokio::process::Command as AsyncCommand;
use tracing::{debug, info, warn};

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
    /// Create a new privilege manager with automatic detection
    pub async fn new(config: PrivilegeConfig) -> Result<Self> {
        let mut manager = Self {
            current_method: PrivilegeMethod::Unprivileged,
            fallback_methods: Vec::new(),
        };

        manager.detect_best_method(&config).await?;
        Ok(manager)
    }

    /// Detect the best privilege escalation method for the current environment
    async fn detect_best_method(&mut self, config: &PrivilegeConfig) -> Result<()> {
        info!("🔍 Detecting privilege escalation methods...");

        // Check if already running as root
        if self.is_running_as_root() {
            info!("✅ Already running with root privileges");
            self.current_method = PrivilegeMethod::AlreadyRoot;
            return Ok(());
        }

        let mut available_methods = Vec::new();

        // Check for Linux capabilities (preferred)
        if config.prefer_capabilities && self.check_capabilities_support().await {
            available_methods.push(PrivilegeMethod::Capabilities);
        }

        // Check for sudo
        if config.allow_sudo && self.check_sudo_available().await {
            available_methods.push(PrivilegeMethod::Sudo);
        }

        // Check for PolicyKit (pkexec)
        if self.check_pkexec_available().await {
            available_methods.push(PrivilegeMethod::PolicyKit);
        }

        // Check for systemd service approach
        if config.allow_systemd && self.check_systemd_available().await {
            available_methods.push(PrivilegeMethod::SystemdService);
        }

        // Check for Docker capabilities
        if self.check_docker_capabilities().await {
            available_methods.push(PrivilegeMethod::DockerCapabilities);
        }

        // Check for setuid binary (less preferred due to security)
        if config.allow_setuid && self.check_setuid_available().await {
            available_methods.push(PrivilegeMethod::SetuidBinary);
        }

        if available_methods.is_empty() {
            if config.fallback_to_unprivileged {
                warn!("⚠️  No privilege escalation methods available, falling back to unprivileged mode");
                self.current_method = PrivilegeMethod::Unprivileged;
                return Ok(());
            } else {
                return Err(SongbirdError::Network {
                    service: Some("Gaming Privilege Manager".to_string()),
                    message: "No suitable privilege escalation method found".to_string(),
                    details: None,
                    endpoint: None,
                    suggestion: Some("Check network connectivity and configuration".to_string()),
                });
            }
        }

        self.current_method = available_methods[0].clone();
        self.fallback_methods = available_methods[1..].to_vec();

        info!("✅ Selected privilege method: {:?}", self.current_method);
        if !self.fallback_methods.is_empty() {
            debug!("📋 Fallback methods: {:?}", self.fallback_methods);
        }

        Ok(())
    }

    /// Execute a command with elevated privileges
    pub async fn execute_privileged_command(
        &self,
        _command: &str,
        _args: &[&str],
    ) -> Result<std::process::Output> {
        match &self.current_method {
            PrivilegeMethod::AlreadyRoot => self.execute_direct_command(_command, _args).await,
            PrivilegeMethod::Sudo => self.execute_sudo_command(_command, _args).await,
            PrivilegeMethod::PolicyKit => self.execute_pkexec_command(_command, _args).await,
            PrivilegeMethod::Capabilities => self.execute_with_capabilities(_command, _args).await,
            PrivilegeMethod::SetuidBinary => self.execute_setuid_command(_command, _args).await,
            PrivilegeMethod::SystemdService => self.execute_systemd_service(_command, _args).await,
            PrivilegeMethod::DockerCapabilities => {
                self.execute_direct_command(_command, _args).await // Already have caps in container
            }
            PrivilegeMethod::Unprivileged => {
                warn!("⚠️  Executing command without privileges - functionality may be limited");
                self.execute_direct_command(_command, _args).await
            }
        }
    }

    /// Check if we need elevated privileges for packet capture
    pub fn requires_privileges(&self) -> bool {
        !matches!(
            self.current_method,
            PrivilegeMethod::AlreadyRoot | PrivilegeMethod::DockerCapabilities
        )
    }

    /// Get user-friendly instructions for manual privilege setup
    pub fn get_setup_instructions(&self) -> Vec<String> {
        match &self.current_method {
            PrivilegeMethod::AlreadyRoot => {
                vec!["✅ Already running with sufficient privileges".to_string()]
            }
            PrivilegeMethod::Sudo => {
                vec![
                    "🔧 To enable packet capture with sudo:".to_string(),
                    "   sudo ./songbird gaming scan".to_string(),
                    "   or".to_string(),
                    "   sudo setcap cap_net_raw+ep $(which songbird)".to_string(),
                ]
            }
            PrivilegeMethod::PolicyKit => {
                vec![
                    "🔧 PolicyKit will prompt for authentication".to_string(),
                    "   pkexec ./songbird gaming scan".to_string(),
                ]
            }
            PrivilegeMethod::Capabilities => {
                vec![
                    "🔧 To set up Linux capabilities:".to_string(),
                    "   sudo setcap cap_net_raw+ep $(which songbird)".to_string(),
                    "   sudo setcap cap_net_admin+ep $(which songbird)".to_string(),
                ]
            }
            PrivilegeMethod::SetuidBinary => {
                vec![
                    "⚠️  Setuid approach (use with caution):".to_string(),
                    "   sudo chown root:root $(which songbird)".to_string(),
                    "   sudo chmod u+s $(which songbird)".to_string(),
                ]
            }
            PrivilegeMethod::SystemdService => {
                vec![
                    "🔧 To set up as systemd service:".to_string(),
                    "   sudo systemctl enable songbird-gaming".to_string(),
                    "   sudo systemctl start songbird-gaming".to_string(),
                ]
            }
            PrivilegeMethod::DockerCapabilities => {
                vec![
                    "🔧 Docker container needs NET_ADMIN capability:".to_string(),
                    "   docker run --cap-add=NET_ADMIN songbird".to_string(),
                ]
            }
            PrivilegeMethod::Unprivileged => {
                vec![
                    "⚠️  Running in unprivileged mode".to_string(),
                    "   Some features may not work".to_string(),
                    "   Consider running with sudo or setting capabilities".to_string(),
                ]
            }
        }
    }

    /// Detect available privilege escalation methods
    pub async fn detect_available_methods(&self) -> Result<Vec<PrivilegeMethod>> {
        info!("🔍 Detecting available privilege methods...");

        let mut available_methods = vec![self.current_method.clone()];
        available_methods.extend(self.fallback_methods.clone());

        Ok(available_methods)
    }

    /// Initialize privileges for packet capture
    pub async fn initialize_privileges(&self) -> Result<()> {
        info!("🔧 Initializing privileges for packet capture...");

        match &self.current_method {
            PrivilegeMethod::AlreadyRoot => {
                info!("✅ Already running with root privileges");
                Ok(())
            }
            PrivilegeMethod::Unprivileged => {
                warn!("⚠️  Running in unprivileged mode - some features may be limited");
                Ok(())
            }
            _ => {
                info!("✅ Privilege method available: {:?}", self.current_method);
                Ok(())
            }
        }
    }

    // Private implementation methods

    fn is_running_as_root(&self) -> bool {
        #[cfg(unix)]
        {
            unsafe { libc::geteuid() == 0 }
        }
        #[cfg(not(unix))]
        {
            // On Windows, check if running as administrator
            false // Simplified for now
        }
    }

    async fn check_capabilities_support(&self) -> bool {
        // Check if we're on Linux and libcap is available
        #[cfg(target_os = "linux")]
        {
            Command::new("which")
                .arg("setcap")
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false)
        }
        #[cfg(not(target_os = "linux"))]
        {
            false
        }
    }

    async fn check_sudo_available(&self) -> bool {
        Command::new("which")
            .arg("sudo")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    async fn check_pkexec_available(&self) -> bool {
        Command::new("which")
            .arg("pkexec")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    async fn check_systemd_available(&self) -> bool {
        Command::new("which")
            .arg("systemctl")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    async fn check_docker_capabilities(&self) -> bool {
        // Check if we're in a Docker container with NET_ADMIN capability
        if let Ok(contents) = std::fs::read_to_string("/proc/1/cgroup") {
            if contents.contains("docker") || contents.contains("containerd") {
                // Check for NET_ADMIN capability
                if let Ok(caps) = std::fs::read_to_string("/proc/self/status") {
                    return caps.contains("CapEff") && caps.contains("2000"); // NET_ADMIN bit
                }
            }
        }
        false
    }

    async fn check_setuid_available(&self) -> bool {
        // Check if current binary can be made setuid
        if let Ok(exe_path) = env::current_exe() {
            if let Ok(metadata) = std::fs::metadata(&exe_path) {
                // Check if we can potentially set setuid bit
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    return metadata.permissions().mode() & 0o111 != 0; // Executable
                }
                #[cfg(not(unix))]
                {
                    return true; // Simplified for non-Unix
                }
            }
        }
        false
    }

    async fn execute_direct_command(
        &self,
        _command: &str,
        _args: &[&str],
    ) -> Result<std::process::Output> {
        let output = AsyncCommand::new(_command)
            .args(_args)
            .output()
            .await
            .map_err(|e| SongbirdError::Network {
                service: Some("Gaming Privilege Manager".to_string()),
                message: format!("Command execution failed: {}", e),
                details: None,
                endpoint: None,
                suggestion: Some("Check network connectivity and configuration".to_string()),
            })?;

        Ok(output)
    }

    async fn execute_sudo_command(
        &self,
        _command: &str,
        _args: &[&str],
    ) -> Result<std::process::Output> {
        let mut sudo_args = vec![_command];
        sudo_args.extend(_args);

        let output = AsyncCommand::new("sudo")
            .args(&sudo_args)
            .output()
            .await
            .map_err(|e| SongbirdError::Network {
                service: Some("Gaming Privilege Manager".to_string()),
                message: format!("Sudo execution failed: {}", e),
                details: None,
                endpoint: None,
                suggestion: Some("Check network connectivity and configuration".to_string()),
            })?;

        Ok(output)
    }

    async fn execute_pkexec_command(
        &self,
        _command: &str,
        _args: &[&str],
    ) -> Result<std::process::Output> {
        let mut pkexec_args = vec![_command];
        pkexec_args.extend(_args);

        let output = AsyncCommand::new("pkexec")
            .args(&pkexec_args)
            .output()
            .await
            .map_err(|e| SongbirdError::Network {
                service: Some("Gaming Privilege Manager".to_string()),
                message: format!("Pkexec execution failed: {}", e),
                details: None,
                endpoint: None,
                suggestion: Some("Check network connectivity and configuration".to_string()),
            })?;

        Ok(output)
    }

    async fn execute_with_capabilities(
        &self,
        _command: &str,
        _args: &[&str],
    ) -> Result<std::process::Output> {
        // For capabilities, we assume the binary already has the required caps set
        // This is the most secure approach as it doesn't require runtime privilege escalation
        self.execute_direct_command(_command, _args).await
    }

    async fn execute_setuid_command(
        &self,
        _command: &str,
        _args: &[&str],
    ) -> Result<std::process::Output> {
        // For setuid, the binary should already have the setuid bit set
        self.execute_direct_command(_command, _args).await
    }

    async fn execute_systemd_service(
        &self,
        _command: &str,
        _args: &[&str],
    ) -> Result<std::process::Output> {
        // Execute via systemd service (implementation would depend on service setup)
        let service_args = vec!["start", "songbird-gaming-command"];

        let output = AsyncCommand::new("systemctl")
            .args(&service_args)
            .output()
            .await
            .map_err(|e| SongbirdError::Network {
                service: Some("Gaming Privilege Manager".to_string()),
                message: format!("Systemd service execution failed: {}", e),
                details: None,
                endpoint: None,
                suggestion: Some("Check network connectivity and configuration".to_string()),
            })?;

        Ok(output)
    }
}

/// Utility function to create a privilege manager with safe defaults
pub async fn create_safe_privilege_manager() -> Result<PrivilegeManager> {
    let config = PrivilegeConfig {
        prefer_capabilities: true,
        allow_sudo: true,
        allow_setuid: false, // Disabled for security
        allow_systemd: true,
        fallback_to_unprivileged: true,
        custom_sudo_command: None,
    };

    PrivilegeManager::new(config).await
}

/// Check if packet capture is possible with current privileges
pub async fn can_capture_packets() -> bool {
    // Try to open a raw socket to test permissions
    #[cfg(unix)]
    {
        match std::net::UdpSocket::bind("0.0.0.0:0") {
            Ok(_) => {
                // Try to create a raw socket
                unsafe {
                    let sockfd = libc::socket(libc::AF_PACKET, libc::SOCK_RAW, 0);
                    if sockfd >= 0 {
                        libc::close(sockfd);
                        return true;
                    }
                }
                false
            }
            Err(_) => false,
        }
    }
    #[cfg(not(unix))]
    {
        // On non-Unix systems, assume we need admin privileges
        false
    }
}
