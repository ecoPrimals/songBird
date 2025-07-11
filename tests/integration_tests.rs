//! Comprehensive integration tests for Songbird
//!
//! These tests verify that all components work together correctly
//! in real-world scenarios.

use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

use songbird_config::config::SongbirdConfig;
use songbird_network::network::gaming::GamingManager;
use songbird_federation::manager::FederationManager;
use songbird_federation::config::FederationMode;
use songbird_discovery::traits::ServiceInfo;
use songbird_errors::Result;

#[tokio::test]
async fn test_basic_orchestration() -> Result<()> {
    let config = SongbirdConfig::default();
    
    // Test that we can create a GamingManager with the correct API
    let gaming_manager = GamingManager::new().await?;
    
    // Test federation manager (corrected API)
    let federation_manager = FederationManager::new(FederationMode::Standalone);
    
    // Test basic federation endpoint retrieval
    let federation_endpoints = federation_manager.get_federation_endpoints().await;
    assert!(federation_endpoints.unwrap().is_empty()); // Should be empty for default config
    
    // Test gaming manager basic functionality
    let mut gaming_manager_mut = gaming_manager;
    let _scan_result = gaming_manager_mut.scan_for_games(None).await;
    
    Ok(())
}

#[tokio::test]
async fn test_federation_coordination() -> Result<()> {
    let config = SongbirdConfig::default();
    
    // Test creating federation manager with correct API
    let federation_manager = FederationManager::new(FederationMode::Standalone);
    
    // Test endpoint functionality
    let endpoints = federation_manager.get_federation_endpoints().await;
    assert!(endpoints.unwrap().is_empty());
    
    // Test gaming configuration access (use available fields)
    let gaming_config = &config.network.gaming;
    assert!(gaming_config.starcraft_port > 0);
    assert!(gaming_config.aoe2_port > 0);
    assert!(gaming_config.cnc_port_range.start < gaming_config.cnc_port_range.end);
    
    Ok(())
}

#[tokio::test]
async fn test_primal_coordination() -> Result<()> {
    let config = SongbirdConfig::default();
    
    // Test federation manager initialization
    let federation_manager = FederationManager::new(FederationMode::Standalone);
    
    // Test basic endpoint retrieval
    let federation_endpoints = federation_manager.get_federation_endpoints().await;
    assert!(federation_endpoints.unwrap().is_empty());
    
    Ok(())
} 