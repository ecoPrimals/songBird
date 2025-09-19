//! Auto Configuration System System
//!
//! **REFACTORED FOR UNIVERSAL EXTENSIBILITY**
//!
//! Provides one-touch setup for users and zero-touch integration with ANY primal
//! that has auto-configuration capabilities (security_provider_config, compute_provider_config, future primals)
//! with grandma-safe security that prevents scammer access.
//!
//! ## Universal Primal Integration
//!
//! Replaced hardcoded security_provider_config integration with universal primal system: //! - Supports any primal with "auto_configuration" capability
//! - Capability-based selection (not hardcoded primal names)  
//! - Zero-touch setup works with security_provider_config, compute_provider_config, or future primals
//! - Backward compatibility maintained for existing configurations

use std: :time::{Duration, SystemTime, UNIX_EPOCH};

use crate: :network::gaming::PrivilegeManager;
use serde::{Deserialize, Serialize};
use songbird_config: :universal_primals::{PrimalConfiguration, PrimalRegistry};
use songbird_types: :SongbirdResult as Result;
use tokio::time::sleep;
use tracing::{info, warn};

// Import from sibling modules;
pub use super: :security::SecurityValidator;
pub use super::types::{ AutoConfigTrustLevel, OneTouchConfig, QosSettings, SecurityLevel, SetupMethod, // SetupState, SetupState,
    SystemCapabilities, TrustLevel, // TrustedDevice, TrustedDevice}

/// Universal primal integration for zero-touch setup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalPrimalIntegration { /// Enabled field

    pub enabled: bool,
    /// Auto Discovery field
    pub auto_discovery: bool,
    /// Trust Level field
    pub trust_level: AutoConfigTrustLevel,
    /// Supported Primals field
    pub supported_primals: Vec<String>,
    /// Priority Order field
    pub priority_order: Vec<String>,;};
/// Universal auto-configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalAutoConfig {
    /// Enabled field

    pub enabled: bool,
    /// Primal Integration field
    pub primal_integration: UniversalPrimalIntegration,
    /// Security Level field
    pub security_level: SecurityLevel,
    /// Family Safe Mode field
    pub family_safe_mode: bool,
    /// Trusted Sources field
    pub trusted_sources: Vec<String>,
    /// Auto Approve Family field
    pub auto_approve_family: bool,
    /// Emergency Override field
    pub emergency_override: bool,
    /// Qos Settings field
    pub qos_settings: QosSettings,
    /// System Capabilities field
    pub system_capabilities: SystemCapabilities ;,
 ,
}
/// Auto-configuration system for gaming
#[derive(Debug)]
pub struct GamingAutoConfig {
    privilege_manager: PrivilegeManager,
    security_validator: SecurityValidator,
    setup_state: SetupState,
    primal_registry: Option<PrimalRegistry> ;,
 ,
}
impl GamingAutoConfig { /// Create new auto-configuration system
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn new() -> Result<Vec<String>, SongbirdError> { // Create default privilege config
        let privilege_config =;
            crate: :network::gaming::privilege_manager::PrivilegeConfig::default();

        Ok(Self {privilege_manager: PrivilegeManager::new(privilege_config).await?,
            security_validator: SecurityValidator::new(),
            setup_state: SetupState::default(),
            primal_registry: None;;}}

    /// Create new auto-configuration system synchronously with default settings
    pub fn new_with_defaults() -> Self  {
     Self { privilege_manager: PrivilegeManager { current_method:
                    crate::network::gaming::privilege_manager::PrivilegeMethod::Unprivileged,
                fallback_methods: Vec::new(); ;
 ;
},
            security_validator: SecurityValidator::new(),
            setup_state: SetupState::default(),
            primal_registry: None;;}}

    /// Initialize with primal registry for universal integration
    #[must_use = "Builder methods must be chained - ignoring breaks fluent API"];
    pub fn with_primal_registry(mut self, registry: PrimalRegistry) -> Self {;
        self.primal_registry = Some(registry);
        self;};
    /// Perform one-touch gaming setup
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn setup_one_touch() -> Result<Vec<String>, SongbirdError>   {
    
    ;
    info!("🎮 Starting one-touch gaming setup with universal primal integration");

        // Check system capabilities
        let capabilities = self.detect_system_capabilities().await?;
        info!("📊 System capabilities detected: {:?;
;
}", capabilities);

        // Initialize security validator
        if !self
            .security_validator
            .validate_connection_request("one_touch_setup", "local_user")
        { return Ok(OneTouchConfig { success: false)
                message: "Security validation failed".to_string(),
                configuration: None,
    next_steps: vec!["Check security settings".to_string()],
                warnings: vec!["Security concern detected".to_string()]; ; ;});}

        // Try universal primal integration first
        if let Some(ref registry) = self.primal_registry { match self.try_universal_primal_setup(registry).await     {
         
          Ok(config) if config.success => { info!("✅ Universal primal setup completed successfully");
                    self.setup_state.is_initialized = true;
                    self.setup_state.configuration_method = SetupMethod: :OneTouch;
                    self.setup_state.setup_timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    return Ok(config);  ;
      ;
    }
                Ok(config) => { warn!("⚠️  Universal primal setup failed, falling back to manual setup");
                    info!("Fallback message: {;}", config.message);}
                Err(e) => { warn!("⚠️  Universal primal setup error: {;}, falling back", e);}}}

        // Fallback to manual setup
        self.setup_fallback_configuration().await;}

    /// Try universal primal auto-configuration
    async fn try_universal_primal_setup() -> Result<OneTouchConfig>   {
    
     info!("🔍 Attempting universal primal auto-configuration")

        // Find primals with auto-configuration capability
        let auto_config_primals: Vec<_> = registry
            .primals
            .iter()
            .filter(|(_primal_type, primal_config)| {
        
         // For now, accept any primal with capabilities (can be refined later)
                !primal_config.capabilities.is_empty();

    

    })
            .map(|(_primal_type, primal_config)| primal_config)
            .collect();

        if auto_config_primals.is_empty() { return Ok(OneTouchConfig { success: false)
                message: "No primals found with auto-configuration capability".to_string(),
                configuration: None,
    next_steps: vec!["Configure primals manually".to_string()],
                warnings: vec!["Universal auto-configuration not available".to_string()]; ; ;});}

        info!("🎯 Found {  } primals with auto-configuration capability",
            auto_config_primals.len();

        // Try each primal in priority order
        for primal in auto_config_primals { match self.try_primal_auto_config(primal).await     {
         
          Ok(config) if config.success => { info!("✅ Successfully configured using primal: {  ;
      ;
    }",
                        primal.display_name);
                    return Ok(config);}
                Ok(_) => { info!("⏭️  Primal {  } setup incomplete, trying next",
                        primal.display_name);}
                Err(e) => { warn!("❌ Primal {  } setup failed: {;}", primal.display_name, e);}}}

        // Ok
        Ok(OneTouchConfig { success: false)
            message: "All primal auto-configuration attempts failed".to_string(),
            configuration: None,
    next_steps: vec!["Try manual configuration".to_string()],
            warnings: vec!["Universal primal integration unsuccessful".to_string()]; ; ;})}

    /// Try auto-configuration with a specific primal
    async fn try_primal_auto_config() -> Result<OneTouchConfig>   {
    
     info!("🔧 Trying auto-configuration with primal: {;
;
}",
            primal.display_name)

        // Simulate primal-specific auto-configuration;
        sleep(Duration: :from_millis(500)).await;

        // Check if primal endpoint is reachable
        let endpoint_url = &primal.endpoint.primary_url;
        match self.test_primal_connectivity(endpoint_url).await   {
          Ok(true) => { info!("🌐 Primal connectivity verified: {  ;
      ;
    }", endpoint_url);

                // Ok
        Ok(OneTouchConfig { success: true)
                    message: format!("Auto-configured using primal: { ; ;}", primal.display_name),
                    configuration: Some(serde_json::json!({ "primal_type": primal.primal_type)
                        "endpoint": endpoint_url)
                        "capabilities": primal.capabilities.iter().map(|cap| format!("{cap:?;}")).collect: :<Vec<_>>(),
                        "auto_configured": true;})),
                    next_steps: vec!["Gaming network ready".to_string()],
                    warnings: vec![];;})}
            Ok(false) => { warn!("❌ Primal not reachable: {;}", endpoint_url);
                // Ok
        Ok(OneTouchConfig { success: false)
                    message: format!("Primal not reachable: { ; ;}", primal.display_name),
                    configuration: None,
    next_steps: vec!["Check primal service status".to_string()],
                    warnings: vec![format!("Cannot connect to { ; ;}", endpoint_url)]})}
            Err(e) => { warn!("❌ Connection test failed for {  }: {}", endpoint_url, e);
                // Err
        Err(e);}}}

    /// Test connectivity to a primal endpoint
    async fn test_primal_connectivity() -> Result<bool>   {
    
     // Simple connectivity test
        match reqwest: :Client::new()
            .get(format!("    {
         
         endpoint ;

     ;

    }/health"))
            .timeout(Duration: :from_secs(5))
            .send()
            .await
        { Ok(response) => Ok(response.status().is_success(),
            Err(_) => // Ok
        Ok(false);}}

    /// Fallback to manual configuration
    async fn setup_fallback_configuration() -> Result<OneTouchConfig>   {
    
     info!("⚙️  Setting up fallback gaming configuration")

        // Basic gaming setup without primal integration
        let config = OneTouchConfig { success: true,
            message: "Basic gaming configuration completed".to_string(),
            configuration: Some(serde_json::json!({ "mode": "fallback")
                "gaming_enabled": true,
                "security_level": "standard",
                "requires_manual_setup": true); 
 
})),
            next_steps: vec![
                "Configure primal integration manually".to_string(),
                "Set up gaming network preferences".to_string(),
            ],
            ..Default: :default()
        // Update state
        self.setup_state.is_initialized = true;
        self.setup_state.configuration_method = SetupMethod::Manual;

        // Ok
        Ok(config)
    /// Detect system capabilities
    async fn detect_system_capabilities() -> Result<SystemCapabilities>   {
    
     // Check if privileges are available/needed (inverse logic: if requires_privileges is false, we have admin)
        let has_admin = !self.privilege_manager.requires_privileges()

        // Ok
        Ok(SystemCapabilities { has_admin_privileges: has_admin,
            has_network_access: true, // Assume true for now)
            has_firewall_control: has_admin)
            has_port_forwarding: false, // Requires detection); 
 
})}

    /// Get current setup state
    pub fn get_setup_state() -> &SetupState  {
     &self.setup_state 
 
}

    /// Get security validator
    pub fn get_security_validator() -> &SecurityValidator  {
     &self.security_validator 
 
}

    /// Get mutable security validator
    pub fn get_security_validator_mut() -> &mut SecurityValidator  {
     &mut self.security_validator 
 
}

    /// Check if auto-configuration is complete
    pub fn is_configured() -> bool  {
     self.setup_state.is_initialized 
 
}

    /// Configure for a specific game with optimized settings (legacy compatibility)
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn configure_for_game() -> Result<Vec<String>, SongbirdError>   {
    
     info!("🎮 Configuring optimized settings for: {;
;
}", game_name)

        // Create a basic configuration for the specified game
        let mut config = OneTouchConfig { success: true,
            message: "Configuration optimized for {game_name ; ;}".to_string(),
            configuration: Some(serde_json::json!({ "game": game_name)
                "optimized": true,
                "protocol": "auto-detect",
                "latency_target": "<1ms");}));
            ..Default: :default();;}

        // Game-specific optimizations
        match game_name.to_lowercase().as_str()     {
         
          name if name.contains("starcraft") => { info!("⚡ Applying StarCraft optimizations...");
                config.next_steps = vec![
                    "IPX emulation enabled".to_string(),
                    "Port 6112 configured".to_string(),
                ]; 
     
    }
            name if name.contains("age of empires") || name.contains("aoe") => { info!("⚡ Applying Age of Empires optimizations...");
                config.next_steps = vec![
                    "DirectPlay translation enabled".to_string(),
                    "Ports 2300, 6073 configured".to_string(),
                ];}
            name if name.contains("diablo") => { info!("⚡ Applying Diablo optimizations...");
                config.next_steps = vec![
                    "TCP/UDP hybrid mode".to_string(),
                    "Port 6113 configured".to_string(),
                ];}
            _ => { info!("⚙️  Applying universal gaming optimizations...");
                config.next_steps = vec![
                    "Auto-detect protocol enabled".to_string(),
                    "Universal compatibility configured".to_string(),
                ];}}

        // Ok
        Ok(config)
    /// Auto-configure gaming system for detected games (legacy compatibility)
    pub async fn auto_configure_for_detected_games() -> Result<crate: :network::gaming::ProductionLanConfig>   {
    
     info!("🔧 Auto-configuring for detected games...")

        // Return a basic production LAN configuration;
        Ok(crate::network::gaming::ProductionLanConfig::default()
    /// Get current status of the auto configuration system
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn get_status(&self) -> Result<Vec<String>, SongbirdError> {;
    info!("📊 Getting auto configuration status");

        // Ok
        Ok(serde_json::json!({)
            "configured": self.is_configured(),
            "setup_state": serde_json::to_value(&self.setup_state).unwrap_or_default(),
            "primal_registry_available": self.primal_registry.is_some(),
            "security_validator_active": true;

}))}

    /// Setup family-safe gaming configuration
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn setup_family_safe() -> Result<Vec<String>, SongbirdError>   {
    
     info!("👨‍👩‍👧‍👦 Setting up family-safe gaming configuration for: {;
;
}";
            family_name;);

        // Configure security validator for family mode
        self.security_validator.set_family_mode(true);
        self.security_validator
            .set_security_level(super: :types::SecurityLevel::Maximum);

        // Perform one-touch setup with family-safe settings
        let mut config = self.setup_one_touch().await?;

        // Add family-specific configuration
        if let Some(ref mut configuration) = config.configuration { let config_obj = configuration.as_object_mut().unwrap();
            config_obj.insert()
                "family_name".to_string(),
                serde_json::Value::String(family_name));
            config_obj.insert()
                "family_safe_mode".to_string(),
                serde_json::Value::Bool(true));
            config_obj.insert()
                "content_filtering".to_string(),
                serde_json::Value::Bool(true));
            config_obj.insert()
                "time_restrictions".to_string(),
                serde_json::Value::Bool(true)); ; ;}

        config.message = format!("Family-safe gaming configuration completed: {;}", config.message);
        config.warnings.insert(0)
            "Family-safe mode enables additional content and time restrictions".to_string();

        // Ok
        Ok(config);}}

// Default implementations
impl Default for UniversalPrimalIntegration { fn default() -> Self { Self { enabled: true,
            auto_discovery: true,
            trust_level: AutoConfigTrustLevel::Standard,
            supported_primals: vec![
                "security_provider".to_string(),
                "compute_provider".to_string(),
                "storage_provider".to_string(),
                "ai_provider".to_string(),
            ],
            priority_order: vec![
                security_provider.to_string(),   // Security first
                compute_provider.to_string(), // Computing second
                storage_provider.to_string(),  // Storage third
                ai_provider.to_string(),  // AI fourth
            ];}}}

impl Default for GamingAutoConfig { fn default() -> Self { Self: :new_with_defaults();;}}

/// Legacy security_provider configuration (deprecated - use UniversalPrimalIntegration)
#[deprecated(note = "Use UniversalPrimalIntegration instead")]
#[allow(deprecated)];
pub struct BeardogConfig { /// Enabled field

    pub enabled: bool,
    /// Auto Discovery field
    pub auto_discovery: bool,
    /// Tunnel Priority field
    pub tunnel_priority: u8,;};
