# 🌐 **SONGBIRD INTERNET CONNECTION WIZARD SPECIFICATION**

**Project:** Songbird Orchestrator  
**Component:** Secure Remote Connectivity System  
**Document Version:** 1.1  
**Date:** December 2024  
**Status:** Design Specification  

---

## 🎯 **EXECUTIVE SUMMARY**

The Songbird Internet Connection Wizard enables **secure, encrypted connectivity** between Songbird nodes across different networks (e.g., your home cluster connecting to family members' machines). It maintains our **security-by-default** philosophy while providing **enterprise-grade VPN-like security** for distributed home HPC clusters.

### **Core Design Principles**

1. **🔒 Security-First**: All internet connections use end-to-end encryption
2. **🏠 Family-Friendly**: Optimized for connecting home networks across family
3. **🛡️ Zero-Trust Internet**: Every remote connection must be authenticated
4. **🎯 Songbird-Only**: Only Songbird traffic allowed through internet tunnels
5. **📋 Automatic Setup**: One-click secure connectivity between trusted nodes
6. **⚙️ Configuration-Driven**: Dynamically discovers ports from Songbird configuration

---

## 🏗️ **SYSTEM ARCHITECTURE**

### **Distributed HPC Network Topology**

```
┌─────────────────────────────────────────────────────────────┐
│                    Your Home Network                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │ Gaming Rig  │  │ Workstation │  │   Songbird Node     │  │
│  │   (Node 1)  │  │   (Node 2)  │  │   (Coordinator)     │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼ Secure Tunnel
                    ┌─────────────────────┐
                    │      Internet       │
                    │   (Encrypted Only)  │
                    └─────────────────────┘
                              │
                              ▼ Secure Tunnel
┌─────────────────────────────────────────────────────────────┐
│                 Family Member's Network                     │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   Laptop    │  │  Old PC     │  │   Songbird Node     │  │
│  │   (Node 3)  │  │  (Node 4)   │  │   (Remote)          │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### **Security Architecture**

```
┌─────────────────────────────────────────────────────────────┐
│                Internet Connection Wizard                   │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   Tunnel    │  │    Key      │  │   Authentication    │  │
│  │  Manager    │  │ Management  │  │     Manager         │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │  Network    │  │  Security   │  │   Configuration     │  │
│  │  Detection  │  │ Validator   │  │    Discovery        │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                 Secure Connectivity Options                 │
├─────────────────────────────────────────────────────────────┤
│  WireGuard    │  OpenVPN     │  Tailscale  │  ZeroTier      │
│  (Primary)    │  (Fallback)  │  (Easy)     │  (Alternative) │
└─────────────────────────────────────────────────────────────┘
```

---

## 🔒 **SECURITY-BY-DEFAULT FOR INTERNET**

### **Default Internet Security Posture**

```rust
// Default internet connection configuration - MAXIMUM SECURITY
pub struct DefaultInternetConfig {
    // Encryption is MANDATORY for all internet traffic
    encryption: EncryptionConfig {
        protocol: EncryptionProtocol::WireGuard,  // Modern, fast, secure
        key_rotation: Duration::hours(24),        // Daily key rotation
        perfect_forward_secrecy: true,            // Keys can't decrypt past traffic
        cipher_suite: "ChaCha20Poly1305".to_string(), // Modern encryption
    },
    
    // Authentication REQUIRED for all connections
    authentication: AuthConfig {
        mutual_auth: true,                        // Both sides verify each other
        certificate_pinning: true,                // Prevent MITM attacks
        node_whitelist: Vec::new(),              // Only explicitly trusted nodes
        auto_accept: false,                       // Manual approval required
    },
    
    // Network restrictions - DISCOVERED FROM SONGBIRD CONFIG
    network_policy: NetworkPolicy {
        songbird_only: true,                      // Only Songbird traffic allowed
        port_discovery: PortDiscoveryMode::Dynamic, // Get ports from config
        traffic_analysis_protection: true,       // Hide traffic patterns
        kill_switch: true,                        // Block all if tunnel fails
    },
    
    // Family-specific settings
    family_mode: FamilyModeConfig {
        simplified_setup: true,                   // Easy for non-technical family
        auto_discovery: false,                    // Manual node addition only
        family_certificate: None,                 // Generated during setup
        trust_on_first_use: false,               // Explicit verification required
    },
}
```

### **Configuration Discovery System**

```rust
pub struct SongbirdConfigurationDiscovery;

impl SongbirdConfigurationDiscovery {
    /// Discover active Songbird ports from running configuration
    pub async fn discover_active_ports(&self) -> Result<SongbirdPortConfiguration> {
        let config = self.load_songbird_configuration().await?;
        
        SongbirdPortConfiguration {
            orchestrator_port: config.orchestrator.port,
            federation_port: config.federation.port,
            metrics_port: config.monitoring.prometheus_port,
            discovery_port: config.discovery.multicast_port,
            additional_service_ports: self.discover_service_ports(&config).await?,
        }
    }
    
    /// Discover ports used by registered services
    async fn discover_service_ports(&self, config: &OrchestratorConfig) -> Result<Vec<u16>> {
        let mut service_ports = Vec::new();
        
        // Query service registry for active services
        for service in self.get_active_services().await? {
            for endpoint in &service.endpoints {
                if let Some(port) = self.extract_port_from_endpoint(&endpoint.path) {
                    service_ports.push(port);
                }
            }
        }
        
        // Remove duplicates and sort
        service_ports.sort_unstable();
        service_ports.dedup();
        
        Ok(service_ports)
    }
    
    /// Load current Songbird configuration
    async fn load_songbird_configuration(&self) -> Result<OrchestratorConfig> {
        // Try multiple configuration sources
        if let Ok(config) = self.load_from_running_instance().await {
            return Ok(config);
        }
        
        if let Ok(config) = self.load_from_config_file().await {
            return Ok(config);
        }
        
        if let Ok(config) = self.load_from_environment().await {
            return Ok(config);
        }
        
        // Fallback to defaults if no configuration found
        Ok(OrchestratorConfig::default())
    }
    
    /// Get configuration from running Songbird instance
    async fn load_from_running_instance(&self) -> Result<OrchestratorConfig> {
        // Connect to running Songbird instance via API
        let client = SongbirdApiClient::new("http://localhost:8080")?; // Try default first
        
        // If default fails, scan for running instances
        if client.health_check().await.is_err() {
            let discovered_ports = self.scan_for_songbird_instances().await?;
            for port in discovered_ports {
                let client = SongbirdApiClient::new(&format!("http://localhost:{}", port))?;
                if let Ok(config) = client.get_configuration().await {
                    return Ok(config);
                }
            }
        } else {
            return client.get_configuration().await;
        }
        
        Err(SongbirdError::ConfigurationNotFound)
    }
    
    /// Scan for running Songbird instances on common ports
    async fn scan_for_songbird_instances(&self) -> Result<Vec<u16>> {
        let mut found_instances = Vec::new();
        let common_ports = vec![8080, 8081, 8082, 9090, 3000]; // Common defaults
        
        for port in common_ports {
            if self.test_songbird_api_endpoint(port).await {
                found_instances.push(port);
            }
        }
        
        Ok(found_instances)
    }
}

#[derive(Debug, Clone)]
pub struct SongbirdPortConfiguration {
    pub orchestrator_port: u16,
    pub federation_port: u16,
    pub metrics_port: u16,
    pub discovery_port: u16,
    pub additional_service_ports: Vec<u16>,
}

impl SongbirdPortConfiguration {
    /// Get all ports that need to be allowed through firewall
    pub fn get_all_required_ports(&self) -> Vec<u16> {
        let mut ports = vec![
            self.orchestrator_port,
            self.federation_port,
            self.metrics_port,
            self.discovery_port,
        ];
        
        ports.extend(&self.additional_service_ports);
        ports.sort_unstable();
        ports.dedup();
        
        ports
    }
    
    /// Check if a port is used by Songbird
    pub fn is_songbird_port(&self, port: u16) -> bool {
        self.get_all_required_ports().contains(&port)
    }
}
```

### **Internet Security Validation**

1. **🛡️ Mandatory Encryption**
   - All internet traffic MUST be encrypted
   - No plaintext Songbird communication over internet
   - Automatic tunnel failure detection and blocking

2. **🔐 Strong Authentication**
   - Mutual authentication between all nodes
   - Certificate pinning to prevent MITM attacks
   - Regular key rotation for forward secrecy

3. **🎯 Traffic Isolation**
   - Only Songbird traffic allowed through tunnels (discovered ports only)
   - All other traffic blocked by default
   - Network-level isolation from other applications

---

## 🏠 **FAMILY NETWORK OPTIMIZATION**

### **Family HPC Scenarios**

```yaml
Family HPC Use Cases:
  Scenario 1: "Extended Family Cluster"
    - Your home: Gaming rig + workstation
    - Parents' house: Old desktop + laptop
    - Sibling's place: Gaming laptop
    
  Scenario 2: "Student Support Network"  
    - Home base: Main HPC cluster
    - College dorm: Student's laptop
    - Summer internship: Temporary machine
    
  Scenario 3: "Vacation Computing"
    - Home cluster: Always-on base
    - Vacation rental: Laptop joins temporarily
    - Mobile hotspot: Emergency compute access

Security Considerations:
  - Different network administrators (family members)
  - Varying technical expertise levels
  - Potential for misconfiguration
  - Need for easy troubleshooting
```

### **Family-Friendly Features**

```rust
pub struct FamilyNetworkProfile {
    // Simplified setup for non-technical users
    setup_complexity: SetupComplexity::Beginner,
    
    // Pre-configured security templates
    security_templates: vec![
        SecurityTemplate::ParentsHouse {
            description: "High security, easy maintenance",
            auto_updates: true,
            remote_troubleshooting: true,
            simplified_interface: true,
        },
        SecurityTemplate::CollegeDorm {
            description: "Medium security, portable setup",
            quick_connect: true,
            bandwidth_optimization: true,
            study_mode_scheduling: true,
        },
        SecurityTemplate::TempLocation {
            description: "High security, temporary access",
            time_limited_access: Some(Duration::days(7)),
            auto_disconnect: true,
            minimal_footprint: true,
        },
    ],
    
    // Family-specific networking
    family_network_features: FamilyNetworkFeatures {
        contact_sharing: true,        // Share family member contact info
        status_notifications: true,   // "Dad's computer joined the cluster"
        usage_reports: true,          // "This week's compute contribution"
        parental_controls: false,     // Not needed for HPC
    },
}
```

---

## 🌐 **CONNECTIVITY OPTIONS**

### **Tunnel Technology Selection**

| Technology | Security | Speed | Ease of Setup | Family-Friendly | Recommended |
|------------|----------|-------|---------------|-----------------|-------------|
| **WireGuard** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | **Primary** |
| **Tailscale** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | **Easy Mode** |
| **OpenVPN** | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ | ⭐⭐ | **Fallback** |
| **ZeroTier** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | **Alternative** |

### **WireGuard Integration (Primary)**

```rust
pub struct WireGuardTunnel {
    // Modern cryptography
    encryption: WireGuardEncryption {
        public_key: PublicKey,
        private_key: PrivateKey,        // Never leaves local machine
        preshared_key: Option<PresharedKey>, // Additional security layer
        cipher: ChaCha20Poly1305,
    },
    
    // Network configuration
    network_config: WireGuardNetworkConfig {
        interface_name: "songbird0".to_string(),
        local_ip: IpAddr,               // Virtual IP in tunnel
        allowed_ips: Vec<IpAddr>,       // Which IPs can communicate
        endpoint: SocketAddr,           // Public IP:port of peer
        persistent_keepalive: Some(25), // Keep NAT mappings alive
    },
    
    // Songbird-specific optimizations
    songbird_optimizations: SongbirdOptimizations {
        mtu_optimization: true,         // Optimize for Songbird packet sizes
        congestion_control: true,       // Handle varying family internet speeds
        automatic_reconnect: true,      // Handle temporary disconnections
        bandwidth_adaptation: true,     // Adapt to available bandwidth
    },
}
```

### **Tailscale Integration (Easy Mode)**

```rust
pub struct TailscaleIntegration {
    // Simplified setup - perfect for family members
    setup_mode: TailscaleSetupMode::FamilyFriendly,
    
    // Automatic configuration
    auto_config: TailscaleAutoConfig {
        magic_dns: true,                // songbird-gaming-rig.family-net
        auto_updates: true,             // Keep security patches current
        subnet_routing: false,          // Only Songbird traffic
        exit_nodes: false,              // No internet routing
    },
    
    // Family network management
    family_management: FamilyManagement {
        shared_tailnet: "songbird-family-hpc".to_string(),
        admin_approval: true,           // You approve new devices
        device_naming: DeviceNaming::Descriptive, // "kevin-gaming-rig"
        usage_monitoring: true,         // Track compute contributions
    },
}
```

---

## 🧙‍♂️ **INTERNET CONNECTION WIZARD**

### **Interactive Setup Flow**

```
🌐 Songbird Internet Connection Wizard
======================================

Step 1: Configuration Discovery
├── Scan for running Songbird instances
├── Load current Songbird configuration
├── Discover active ports and services
└── Validate configuration completeness

Step 2: Connection Type Selection
├── 🏠 Family Network Extension (Recommended)
├── 🎓 Student/Mobile Computing
├── 🏢 Remote Office Integration
└── ⚙️  Custom Internet Setup

Step 3: Family Network Discovery
├── Scan for existing family Songbird nodes
├── Import family member contact information
├── Detect network topology and constraints
└── Recommend optimal connection strategy

Step 4: Security Configuration
├── Generate family certificate authority
├── Create node-specific certificates
├── Configure encryption and authentication
└── Set up automatic key rotation

Step 5: Tunnel Setup
├── Choose tunnel technology (WireGuard/Tailscale/etc.)
├── Configure network addressing
├── Set up firewall rules for discovered ports
└── Test connectivity and performance

Step 6: Family Onboarding
├── Generate setup instructions for family members
├── Create easy-install packages with discovered config
├── Set up monitoring and notifications
└── Verify secure connectivity
```

### **Family Network Wizard Flow**

```rust
pub struct FamilyNetworkWizardConfig {
    // Configuration discovery
    songbird_config: SongbirdPortConfiguration,
    
    // Family topology
    family_members: Vec<FamilyMember>,
    network_topology: NetworkTopology,
    
    // Connection preferences
    connection_preferences: ConnectionPreferences {
        prioritize_security: true,      // Security over convenience
        prioritize_ease_of_use: true,   // But still easy for family
        bandwidth_consideration: BandwidthProfile::Mixed, // Some fast, some slow
        technical_expertise: TechnicalLevel::Mixed,       // You're tech, others aren't
    },
    
    // Security settings
    family_security: FamilySecurityConfig {
        mutual_authentication: true,
        certificate_authority: CertificateAuthority::SelfSigned,
        key_rotation_interval: Duration::days(30),
        emergency_access: true,         // You can always access family nodes
        parental_oversight: false,      // This is compute, not internet filtering
    },
    
    // Practical considerations
    practical_config: PracticalConfig {
        different_isps: true,           // Family has different internet providers
        varying_speeds: true,           // Different connection speeds
        dynamic_ips: true,              // Most family doesn't have static IPs
        firewall_complexity: FirewallComplexity::Varied, // Different router configs
    },
}

#[derive(Debug, Clone)]
pub struct FamilyMember {
    name: String,                       // "Dad", "Mom", "Sister"
    location: String,                   // "Parents House", "College Dorm"
    technical_level: TechnicalLevel,    // Beginner, Intermediate, Advanced
    available_hardware: Vec<HardwareSpec>,
    network_constraints: NetworkConstraints,
    contact_info: ContactInfo,
}
```

---

## 🔧 **SECURE TUNNEL MANAGEMENT**

### **Automatic Tunnel Configuration**

```rust
impl TunnelManager {
    pub async fn setup_family_network(&self, config: &FamilyNetworkConfig) -> Result<FamilyTunnel> {
        // Step 1: Discover Songbird configuration
        let songbird_ports = SongbirdConfigurationDiscovery::new()
            .discover_active_ports().await?;
        
        // Step 2: Generate family certificate authority
        let family_ca = self.generate_family_ca(&config.family_name).await?;
        
        // Step 3: Create tunnel mesh between all family nodes
        let tunnel_mesh = self.create_tunnel_mesh(&config.family_members).await?;
        
        // Step 4: Configure each tunnel with discovered Songbird ports
        for tunnel in &tunnel_mesh.tunnels {
            self.configure_songbird_tunnel(tunnel, &songbird_ports).await?;
        }
        
        // Step 5: Set up monitoring and health checks
        self.setup_tunnel_monitoring(&tunnel_mesh).await?;
        
        // Step 6: Generate family member setup packages with discovered config
        let setup_packages = self.generate_setup_packages(&config, &songbird_ports).await?;
        
        Ok(FamilyTunnel {
            ca: family_ca,
            mesh: tunnel_mesh,
            setup_packages,
            monitoring: TunnelMonitoring::new(),
            port_configuration: songbird_ports,
        })
    }
    
    async fn configure_songbird_tunnel(&self, tunnel: &Tunnel, ports: &SongbirdPortConfiguration) -> Result<()> {
        // Configure tunnel specifically for discovered Songbird ports
        let allowed_ports = ports.get_all_required_ports();
        tunnel.set_allowed_ports(&allowed_ports).await?;
        tunnel.set_traffic_filtering(TrafficFilter::SongbirdOnly).await?;
        tunnel.set_encryption_level(EncryptionLevel::Maximum).await?;
        tunnel.enable_perfect_forward_secrecy().await?;
        
        // Family-specific optimizations
        tunnel.set_keepalive_interval(Duration::seconds(25)).await?; // Keep NAT alive
        tunnel.enable_automatic_reconnect().await?;
        tunnel.set_bandwidth_adaptation(true).await?;
        
        Ok(())
    }
}
```

### **Family Member Onboarding**

```rust
pub struct FamilyOnboarding;

impl FamilyOnboarding {
    pub async fn generate_setup_package(&self, member: &FamilyMember, ports: &SongbirdPortConfiguration) -> Result<SetupPackage> {
        let package = match member.technical_level {
            TechnicalLevel::Beginner => {
                SetupPackage::OneClickInstaller {
                    executable: self.create_one_click_installer(member, ports).await?,
                    instructions: self.generate_simple_instructions(member).await?,
                    support_contact: "Call Kevin if this doesn't work".to_string(),
                }
            },
            TechnicalLevel::Intermediate => {
                SetupPackage::GuidedSetup {
                    config_files: self.generate_config_files(member, ports).await?,
                    step_by_step_guide: self.generate_detailed_guide(member).await?,
                    troubleshooting_guide: self.generate_troubleshooting_guide().await?,
                }
            },
            TechnicalLevel::Advanced => {
                SetupPackage::ManualConfiguration {
                    certificates: self.generate_certificates(member).await?,
                    config_templates: self.generate_config_templates(member, ports).await?,
                    api_documentation: self.generate_api_docs().await?,
                }
            },
        };
        
        Ok(package)
    }
    
    async fn create_one_click_installer(&self, member: &FamilyMember, ports: &SongbirdPortConfiguration) -> Result<OneClickInstaller> {
        // Create a simple executable that:
        // 1. Installs WireGuard/Tailscale
        // 2. Configures Songbird tunnel with discovered ports
        // 3. Sets up firewall rules for discovered ports only
        // 4. Tests connectivity
        // 5. Sends "I'm connected!" message to coordinator
        
        OneClickInstaller {
            platform: member.platform.clone(),
            installer_type: InstallerType::SelfExtracting,
            embedded_certificates: true,
            automatic_firewall_config: true,
            success_notification: true,
            port_configuration: ports.clone(),
        }
    }
}
```

---

## 🛡️ **SECURITY VALIDATION FOR INTERNET**

### **Internet-Specific Security Checks**

```rust
pub struct InternetSecurityValidator;

impl InternetSecurityValidator {
    pub fn validate_internet_configuration(&self, config: &InternetConfig, ports: &SongbirdPortConfiguration) -> InternetSecurityReport {
        let mut critical_issues = Vec::new();
        let mut warnings = Vec::new();
        
        // Critical: All internet traffic must be encrypted
        if !config.encryption.enabled {
            critical_issues.push(SecurityIssue::Critical {
                issue: "Internet traffic not encrypted".to_string(),
                risk: "All Songbird communication visible to ISPs and attackers".to_string(),
                fix: "Enable WireGuard or equivalent encryption".to_string(),
            });
        }
        
        // Critical: No plaintext authentication over internet
        if config.authentication.plaintext_allowed {
            critical_issues.push(SecurityIssue::Critical {
                issue: "Plaintext authentication over internet".to_string(),
                risk: "Credentials can be intercepted".to_string(),
                fix: "Use certificate-based authentication only".to_string(),
            });
        }
        
        // Critical: Only discovered Songbird ports should be allowed
        for rule in &config.firewall_rules {
            if let Some(port) = rule.port {
                if !ports.is_songbird_port(port) {
                    critical_issues.push(SecurityIssue::Critical {
                        issue: format!("Non-Songbird port {} allowed through tunnel", port),
                        risk: "Potential security breach - unknown service exposed".to_string(),
                        fix: "Remove rule or verify port is actually used by Songbird".to_string(),
                    });
                }
            }
        }
        
        // Warning: Weak encryption
        if config.encryption.cipher_suite.contains("DES") || 
           config.encryption.cipher_suite.contains("RC4") {
            warnings.push(SecurityWarning {
                issue: "Weak encryption cipher detected".to_string(),
                recommendation: "Use ChaCha20Poly1305 or AES-256-GCM".to_string(),
            });
        }
        
        // Warning: No key rotation
        if config.key_management.rotation_interval > Duration::days(90) {
            warnings.push(SecurityWarning {
                issue: "Infrequent key rotation".to_string(),
                recommendation: "Rotate keys at least monthly for internet connections".to_string(),
            });
        }
        
        InternetSecurityReport {
            overall_security_level: self.calculate_internet_security_level(config),
            critical_issues,
            warnings,
            compliance_status: self.check_internet_compliance(config),
            port_validation: self.validate_port_configuration(config, ports),
        }
    }
    
    fn validate_port_configuration(&self, config: &InternetConfig, ports: &SongbirdPortConfiguration) -> PortValidationResult {
        let mut issues = Vec::new();
        let discovered_ports = ports.get_all_required_ports();
        
        // Check if all discovered ports are properly configured
        for port in &discovered_ports {
            if !self.is_port_configured_in_tunnel(config, *port) {
                issues.push(format!("Discovered Songbird port {} not configured in tunnel", port));
            }
        }
        
        // Check for any configured ports that aren't discovered
        for rule in &config.firewall_rules {
            if let Some(port) = rule.port {
                if !discovered_ports.contains(&port) {
                    issues.push(format!("Configured port {} not found in Songbird configuration", port));
                }
            }
        }
        
        PortValidationResult {
            passed: issues.is_empty(),
            issues,
            discovered_ports,
            configured_ports: self.extract_configured_ports(config),
        }
    }
}
```

---

## 📋 **FAMILY NETWORK TEMPLATES**

### **Parents House Template (Dynamic Ports)**

```toml
# Songbird Family Network Configuration - Parents House
# Generated by Songbird Internet Connection Wizard
# Security Level: Maximum (Internet Connection)
# Ports: Dynamically discovered from Songbird configuration

[family_network]
name = "songbird-family-hpc"
location = "parents-house"
coordinator = false  # This is a remote node
main_coordinator = "kevin-home-cluster"

[tunnel]
technology = "wireguard"
interface = "songbird-family"
local_ip = "10.100.1.10/24"
endpoint = "your-home-ip:51820"

[encryption]
protocol = "wireguard"
cipher = "ChaCha20Poly1305"
key_rotation_hours = 24
perfect_forward_secrecy = true

[authentication]
method = "certificate"
ca_file = "/etc/songbird/family-ca.crt"
cert_file = "/etc/songbird/parents-house.crt"
key_file = "/etc/songbird/parents-house.key"
mutual_auth = true

[firewall]
# Only allow discovered Songbird ports through tunnel
# These ports are dynamically discovered from Songbird configuration
allow_ports = "${DISCOVERED_SONGBIRD_PORTS}"  # Will be replaced with actual discovered ports
allow_protocols = ["tcp", "udp"]
source_restriction = "tunnel_only"
internet_isolation = true

[port_discovery]
enabled = true
discovery_method = "songbird_api"
fallback_ports = []  # No fallback - only use discovered ports
refresh_interval = "1h"  # Re-discover ports hourly

[family_features]
simplified_interface = true
auto_updates = true
remote_troubleshooting = true
status_notifications = true

# Emergency contact
[emergency]
contact = "kevin@example.com"
phone = "+1-555-0123"
auto_support = true
```

### **College Dorm Template (Dynamic Ports)**

```toml
# Songbird Family Network Configuration - College Dorm
# Optimized for: Temporary connection, bandwidth efficiency
# Ports: Dynamically discovered from Songbird configuration

[family_network]
name = "songbird-family-hpc"
location = "college-dorm"
connection_type = "temporary"
bandwidth_limited = true

[tunnel]
technology = "tailscale"  # Easier for college network
auto_connect = true
bandwidth_optimization = true
mobile_friendly = true

[port_discovery]
enabled = true
discovery_method = "songbird_api"
discovery_endpoint = "https://home-cluster.family-hpc.net/api/config"
fallback_discovery = ["multicast", "dns_sd"]

[scheduling]
# Don't interfere with classes/sleep
active_hours = "18:00-02:00"  # 6 PM to 2 AM
weekend_extended = true
exam_period_disable = true

[bandwidth]
max_usage_mbps = 10  # Don't saturate dorm internet
adaptive_quality = true
pause_for_video_calls = true

[security]
# Still maximum security despite convenience features
encryption = "maximum"
authentication = "certificate"
vpn_kill_switch = true
# Ports are dynamically discovered and configured
dynamic_port_configuration = true
```

---

## 🧪 **INTERNET CONNECTIVITY TESTING**

### **Family Network Testing Suite**

```rust
pub struct FamilyNetworkTester {
    family_nodes: Vec<FamilyNode>,
    test_scenarios: Vec<TestScenario>,
    port_configuration: SongbirdPortConfiguration,
}

impl FamilyNetworkTester {
    pub async fn run_family_connectivity_tests(&self) -> FamilyConnectivityReport {
        let mut results = Vec::new();
        
        // Test 1: Basic tunnel connectivity
        results.push(self.test_tunnel_connectivity().await);
        
        // Test 2: Songbird service discovery across internet
        results.push(self.test_cross_internet_discovery().await);
        
        // Test 3: Encrypted communication validation
        results.push(self.test_encryption_validation().await);
        
        // Test 4: Authentication and authorization
        results.push(self.test_family_authentication().await);
        
        // Test 5: Performance over various internet connections
        results.push(self.test_performance_across_isps().await);
        
        // Test 6: Failover and reconnection
        results.push(self.test_connection_resilience().await);
        
        // Test 7: Family member accessibility
        results.push(self.test_family_member_experience().await);
        
        // Test 8: Port configuration validation
        results.push(self.test_port_configuration_accuracy().await);
        
        FamilyConnectivityReport {
            overall_status: self.calculate_family_network_health(&results),
            individual_tests: results,
            family_member_feedback: self.collect_family_feedback().await,
            recommendations: self.generate_family_recommendations(&results),
            port_configuration_status: self.validate_port_configuration().await,
        }
    }
    
    async fn test_port_configuration_accuracy(&self) -> TestResult {
        // Test that only discovered Songbird ports are accessible
        // and that all required ports are properly configured
        let mut issues = Vec::new();
        
        for port in self.port_configuration.get_all_required_ports() {
            if !self.test_port_accessibility(port).await {
                issues.push(format!("Required Songbird port {} not accessible", port));
            }
        }
        
        // Test that no unexpected ports are open
        let open_ports = self.scan_open_ports().await;
        for port in open_ports {
            if !self.port_configuration.is_songbird_port(port) {
                issues.push(format!("Unexpected port {} is open", port));
            }
        }
        
        TestResult {
            test_name: "Port Configuration Accuracy".to_string(),
            status: if issues.is_empty() { TestStatus::Passed } else { TestStatus::Failed },
            details: if issues.is_empty() {
                "All discovered Songbird ports properly configured".to_string()
            } else {
                format!("Port configuration issues: {}", issues.join(", "))
            },
        }
    }
}
```

---

## 📊 **FAMILY NETWORK MONITORING**

### **Family-Friendly Monitoring**

```rust
pub struct FamilyNetworkMonitor {
    family_topology: FamilyTopology,
    monitoring_config: FamilyMonitoringConfig,
    port_configuration: SongbirdPortConfiguration,
}

impl FamilyNetworkMonitor {
    pub async fn start_family_monitoring(&self) -> Result<()> {
        // Monitor tunnel health across family networks
        tokio::spawn(self.monitor_family_tunnel_health());
        
        // Monitor family member connectivity and experience
        tokio::spawn(self.monitor_family_member_experience());
        
        // Monitor security across all family connections
        tokio::spawn(self.monitor_family_network_security());
        
        // Monitor port configuration drift
        tokio::spawn(self.monitor_port_configuration_changes());
        
        // Generate family-friendly status reports
        tokio::spawn(self.generate_family_status_reports());
        
        Ok(())
    }
    
    async fn monitor_port_configuration_changes(&self) {
        // Periodically re-discover Songbird configuration
        // and update tunnel configuration if ports change
        loop {
            tokio::time::sleep(Duration::hours(1)).await;
            
            if let Ok(new_config) = SongbirdConfigurationDiscovery::new()
                .discover_active_ports().await {
                
                if new_config != self.port_configuration {
                    tracing::info!("Songbird port configuration changed, updating tunnels");
                    self.update_tunnel_configuration(&new_config).await;
                    self.notify_family_of_configuration_change(&new_config).await;
                }
            }
        }
    }
    
    async fn send_family_status_update(&self, metrics: &FamilyExperienceMetrics) {
        let discovered_ports = self.port_configuration.get_all_required_ports();
        
        let report = format!(
            "🏠 Weekly Family HPC Cluster Report\n\
             =====================================\n\
             \n\
             📊 This Week's Compute Contributions:\n\
             • Parents House: 45.2 compute hours\n\
             • Sister's Laptop: 12.8 compute hours\n\
             • College Dorm: 8.3 compute hours\n\
             \n\
             🔒 Security Status: All connections secure ✅\n\
             🌐 Network Health: Excellent ✅\n\
             🛠️  Support Requests: 0 this week ✅\n\
             ⚙️  Active Songbird Ports: {} ✅\n\
             \n\
             💡 Next Week: Planning to add Grandpa's new computer!\n\
             \n\
             Questions? Just call Kevin! 📞",
             discovered_ports.len()
        );
        
        // Send via email, SMS, or family chat app
        self.send_family_notification(&report).await;
    }
}
```

---

## 🚀 **IMPLEMENTATION ROADMAP**

### **Phase 1: Configuration Discovery & Core Connectivity (Week 1-2)**
- [ ] Songbird configuration discovery system
- [ ] Dynamic port detection and validation
- [ ] WireGuard tunnel management with discovered ports
- [ ] Certificate authority for family networks

### **Phase 2: Family-Friendly Features (Week 3-4)**
- [ ] Tailscale integration for easy setup
- [ ] One-click installer generation with discovered config
- [ ] Family member onboarding wizard
- [ ] Configuration templates with dynamic port substitution

### **Phase 3: Security & Monitoring (Week 5-6)**
- [ ] Internet-specific security validation
- [ ] Port configuration drift monitoring
- [ ] Family network monitoring
- [ ] Automatic security updates

### **Phase 4: Advanced Family Features (Week 7-8)**
- [ ] Mobile device support
- [ ] Bandwidth optimization
- [ ] Family usage reporting
- [ ] Integration with family communication tools

---

## 🎯 **SUCCESS CRITERIA**

### **Security Metrics**
- ✅ **100% encrypted** internet communication
- ✅ **Zero plaintext** Songbird traffic over internet
- ✅ **Mutual authentication** for all family connections
- ✅ **Perfect forward secrecy** for all tunnels
- ✅ **Dynamic port discovery** - no hardcoded ports

### **Family Experience Metrics**
- ✅ **<5 minute setup** for non-technical family members
- ✅ **>99% uptime** for family network connections
- ✅ **<1 support request per month** per family member
- ✅ **Zero security incidents** across family network
- ✅ **Automatic port updates** when Songbird configuration changes

### **Technical Metrics**
- ✅ **<100ms added latency** for family connections
- ✅ **>95% tunnel reliability** across different ISPs
- ✅ **Automatic reconnection** within 30 seconds
- ✅ **Works behind NAT** and restrictive firewalls
- ✅ **100% configuration-driven** port management

---

## 📚 **RELATED DOCUMENTATION**

- [Firewall Configuration Wizard](./FIREWALL_WIZARD_SPECIFICATION.md)
- [Security-by-Default Analysis](./DEFAULT_SECURITY_ANALYSIS.md)
- [Family Network Setup Guide](../user/FAMILY_NETWORK_SETUP.md)
- [Troubleshooting Family Connections](../user/FAMILY_TROUBLESHOOTING.md)

---

## 📝 **CONCLUSION**

The Songbird Internet Connection Wizard extends our **security-by-default** philosophy to **distributed family HPC networks** while maintaining **configuration-driven port management**. By **dynamically discovering** Songbird's active ports and services, the wizard ensures that only legitimate Songbird traffic is allowed through secure tunnels, eliminating the security risks of hardcoded port configurations.

The system's **mandatory encryption**, **mutual authentication**, **traffic isolation**, and **dynamic port discovery** ensure that family HPC clusters maintain the same security standards as local deployments, while the **simplified setup** and **automatic management** make it accessible to family members with varying technical expertise.

**Key Benefits:**
- 🔒 **Enterprise-grade security** for family networks
- 🏠 **Family-friendly setup** for non-technical users
- 🌐 **Secure internet connectivity** without compromising local security
- 🎯 **Songbird-specific optimization** for HPC workloads
- 📊 **Family-oriented monitoring** and reporting
- ⚙️ **Configuration-driven port management** - no hardcoded values 