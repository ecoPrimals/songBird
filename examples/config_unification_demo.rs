use songbird_types::config::consolidated_canonical::CanonicalSongbirdConfig;
//! Configuration Unification Demonstration
//!
//! This example shows how to use the new canonical configuration system
//! with unified constants and modern patterns.

use songbird_types::{
    NetworkConstants, TimeoutConstants, ResourceConstants, SystemConstants,
    SongbirdResult, SongbirdError
};
use songbird_config::CanonicalSongbirdConfig;
use std::time::Duration;

#[tokio::main]
async fn main() -> SongbirdResult<()> {
    println!("🎯 Songbird Canonical Configuration Demo");
    println!("=======================================");
    
    // ============================================================================
    // 1. CANONICAL CONSTANTS SYSTEM
    // ============================================================================
    
    println!("\n✅ NEW: Canonical Constants System");
    println!("----------------------------------");
    
    // Use canonical constants for configuration
    println!("📊 Canonical Constants Examples:");
    println!("  Default Port: {}", NetworkConstants::DEFAULT_PORT);
    println!("  Discovery Port: {}", NetworkConstants::DEFAULT_DISCOVERY_PORT);
    println!("  Request Timeout: {:?}", TimeoutConstants::DEFAULT_REQUEST_TIMEOUT);
    println!("  Health Check Interval: {:?}", TimeoutConstants::DEFAULT_HEALTH_CHECK_INTERVAL);
    println!("  Memory Limit: {} MB", ResourceConstants::DEFAULT_MEMORY_LIMIT / 1024 / 1024);
    println!("  CPU Limit: {}%", ResourceConstants::DEFAULT_CPU_LIMIT);
    println!("  Config Path: {}", SystemConstants::DEFAULT_CONFIG_PATH);
    println!("  Log Level: {}", SystemConstants::DEFAULT_LOG_LEVEL);
    
    // ============================================================================
    // 2. ENVIRONMENT-AWARE CONFIGURATION
    // ============================================================================
    
    println!("\n🚀 Environment-Aware Configuration");
    println!("----------------------------------");
    
    // Development environment
    let dev_bind_address = songbird_types::get_default_bind_address("development");
    println!("🔧 Development:");
    println!("  Bind Address: {}", dev_bind_address);
    println!("  Port: {}", NetworkConstants::DEFAULT_DEV_HTTP_PORT);
    
    // Production environment  
    let prod_bind_address = songbird_types::get_default_bind_address("production");
    println!("🏭 Production:");
    println!("  Bind Address: {}", prod_bind_address);
    println!("  Port: {}", NetworkConstants::DEFAULT_PORT);
    
    // Service-specific ports
    println!("\n🔌 Service-Specific Ports:");
    println!("  Orchestrator: {}", songbird_types::get_default_port_for_service("orchestrator"));
    println!("  Discovery: {}", songbird_types::get_default_port_for_service("discovery"));
    println!("  Health: {}", songbird_types::get_default_port_for_service("health"));
    println!("  Metrics: {}", songbird_types::get_default_port_for_service("metrics"));
    
    // ============================================================================
    // 3. UNIFIED CONFIGURATION WITH CANONICAL CONSTANTS
    // ============================================================================
    
    println!("\n⚙️ Unified Configuration");
    println!("------------------------");
    
    // Load unified configuration using canonical constants
    let mut config = CanonicalSongbirdConfig::default();
    
    // Override with canonical constants
    config.network.port = NetworkConstants::DEFAULT_PORT;
    config.network.bind_address = NetworkConstants::DEFAULT_BIND_ADDRESS.to_string();
    
    println!("📊 Configuration Access Examples:");
    println!("  Network Port: {}", config.network.port);
    println!("  Bind Address: {}", config.network.bind_address);
    
    // ============================================================================
    // 4. PORT VALIDATION UTILITIES
    // ============================================================================
    
    println!("\n🔍 Port Validation Utilities");
    println!("----------------------------");
    
    let test_ports = [80, 8080, 49152, 65535];
    
    for port in test_ports {
        println!("  Port {}: Reserved={}, Dynamic={}", 
            port,
            songbird_types::is_reserved_port(port),
            songbird_types::is_dynamic_port(port)
        );
    }
    
    println!("\n✅ Canonical configuration system working perfectly!");
    Ok(())
} 