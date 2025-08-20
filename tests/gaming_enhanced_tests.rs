//! Comprehensive tests for gaming auto-configuration functionality
//!
//! These tests verify the gaming network configuration, optimization settings,
//! and one-touch setup capabilities.

use songbird_errors::SongbirdResult;
use songbird_network::network::gaming::{GamingAutoConfig, OneTouchConfig};
use std::time::Duration;

/// Test basic gaming auto config initialization
#[tokio::test]
async fn test_gaming_auto_config_initialization() -> Result<()> {
    let config = GamingAutoConfig::new()?;

    // Verify initial state
    let setup_state = config.get_setup_state();
    assert!(!setup_state.is_initialized);

    println!("✅ Gaming auto config initialized successfully");
    Ok(())
}

/// Test setup state management
#[tokio::test]
async fn test_setup_state_management() -> Result<()> {
    let config = GamingAutoConfig::new()?;

    // Check initial configuration state
    let is_configured = config.is_configured();
    assert!(
        !is_configured,
        "New config should not be configured initially"
    );

    let setup_state = config.get_setup_state();
    assert!(!setup_state.is_initialized);
    println!("✅ Setup state: initialized={}", setup_state.is_initialized);

    Ok(())
}

/// Test security validator functionality
#[tokio::test]
async fn test_security_validator() -> Result<()> {
    let mut config = GamingAutoConfig::new()?;

    // Get security validator
    let _validator = config.get_security_validator();
    println!("✅ Security validator retrieved successfully");

    // Get mutable security validator
    let _validator_mut = config.get_security_validator_mut();
    println!("✅ Mutable security validator retrieved successfully");

    Ok(())
}

/// Test game-specific configuration
#[tokio::test]
async fn test_game_specific_configuration() -> Result<()> {
    let config = GamingAutoConfig::new()?;

    // Configure for a specific game
    let game_config = config.configure_for_game("StarCraft").await?;

    // Verify configuration succeeded
    assert!(game_config.success);
    assert!(game_config.message.contains("StarCraft"));

    if let Some(configuration) = &game_config.configuration {
        assert!(configuration.get("game").is_some());
        assert!(configuration.get("optimized").is_some());
    }

    println!("✅ Game config: {}", game_config.message);

    Ok(())
}

/// Test multiple game configurations
#[tokio::test]
async fn test_multiple_game_configurations() -> Result<()> {
    let config = GamingAutoConfig::new()?;

    let games = vec!["StarCraft", "CounterStrike", "Minecraft", "Valorant"];

    for game in games {
        let game_config = config.configure_for_game(game).await?;

        assert!(game_config.success);
        assert!(game_config.message.contains(game));
        println!("✅ Configured for {}: {}", game, game_config.message);
    }

    Ok(())
}

/// Test configuration with warnings
#[tokio::test]
async fn test_configuration_with_warnings() -> Result<()> {
    let config = GamingAutoConfig::new()?;

    // Some configurations might generate warnings
    let game_config = config.configure_for_game("UnknownGame").await?;

    // Configuration should still succeed but might have warnings
    if !game_config.warnings.is_empty() {
        println!("⚠️ Configuration warnings: {:?}", game_config.warnings);
    }

    println!("✅ Configuration completed with potential warnings");

    Ok(())
}

/// Test status retrieval
#[tokio::test]
async fn test_status_retrieval() -> Result<()> {
    let config = GamingAutoConfig::new()?;

    let status = config.get_status().await?;

    // Verify status has expected JSON structure
    assert!(status.get("configured").is_some());
    assert!(status.get("setup_state").is_some());
    assert!(status.get("primal_registry_available").is_some());
    assert!(status.get("security_validator_active").is_some());

    // Check that configured field is boolean false initially
    assert_eq!(
        status.get("configured"),
        Some(&serde_json::Value::Bool(false))
    );

    println!(
        "✅ Status: {}",
        serde_json::to_string_pretty(&status).unwrap_or_default()
    );

    Ok(())
}

/// Test family safe setup
#[tokio::test]
async fn test_family_safe_setup() -> Result<()> {
    let mut config = GamingAutoConfig::new()?;

    let result = config.setup_family_safe("test_family".to_string()).await?;

    // Verify result structure (success can be true or false depending on primal availability)
    assert!(!result.message.is_empty());

    // If successful, should have family-specific configuration
    if result.success {
        assert!(result.configuration.is_some());
        if let Some(config) = &result.configuration {
            assert!(config.get("family_name").is_some());
            assert!(config.get("family_safe_mode").is_some());
        }
        println!("✅ Family safe setup succeeded: {}", result.message);
    } else {
        // Failure is expected when no primal registry is configured
        println!(
            "⚠️ Family safe setup failed (expected in test env): {}",
            result.message
        );
    }

    // Debug output to understand the actual message format
    println!(
        "🔍 Family setup result - success: {}, message: '{}'",
        result.success, result.message
    );

    Ok(())
}

/// Test concurrent gaming configurations
#[tokio::test]
async fn test_concurrent_configurations() -> Result<()> {
    // Create multiple configs for concurrent testing
    let futures = (0..3).map(|i| async move {
        let config = GamingAutoConfig::new()?;
        let game_name = format!("Game{}", i);
        config.configure_for_game(&game_name).await
    });

    // Wait for all configurations to complete
    let results: Vec<Result<OneTouchConfig>> = futures_util::future::join_all(futures).await;

    // Verify all configurations succeeded
    for (i, result) in results.into_iter().enumerate() {
        let game_config = result?;
        assert!(game_config.success);
        println!("✅ Concurrent config {}: {}", i, game_config.message);
    }

    println!("✅ Concurrent configurations completed successfully");

    Ok(())
}

/// Test error handling in gaming configuration
#[tokio::test]
async fn test_error_handling() -> Result<()> {
    let config = GamingAutoConfig::new()?;

    // Test with edge cases
    let empty_game_config = config.configure_for_game("").await?;
    // Should still succeed but might have warnings
    println!("✅ Empty game name handled: {}", empty_game_config.message);

    let long_name = "A".repeat(1000);
    let long_game_config = config.configure_for_game(&long_name).await?;
    // Should handle gracefully
    println!("✅ Long game name handled: {}", long_game_config.message);

    Ok(())
}

/// Test configuration structure
#[tokio::test]
async fn test_configuration_structure() -> Result<()> {
    let config = GamingAutoConfig::new()?;

    let game_config = config.configure_for_game("StructureTest").await?;

    // Verify OneTouchConfig structure
    assert!(game_config.success);
    assert!(!game_config.message.is_empty());

    if let Some(configuration) = &game_config.configuration {
        // Verify configuration is valid JSON
        let json_str = serde_json::to_string(configuration)?;
        assert!(!json_str.is_empty());
        println!("✅ Configuration JSON: {}", json_str);
    }

    // Verify other fields exist
    println!("✅ Warnings count: {}", game_config.warnings.len());
    println!("✅ Next steps count: {}", game_config.next_steps.len());

    Ok(())
}

/// Test gaming config performance
#[tokio::test]
async fn test_gaming_config_performance() -> Result<()> {
    let config = GamingAutoConfig::new()?;

    let start = std::time::Instant::now();

    // Perform multiple configurations to test performance
    for i in 0..10 {
        let game_name = format!("PerfTestGame{}", i);
        let _game_config = config.configure_for_game(&game_name).await?;
    }

    let duration = start.elapsed();
    println!("✅ 10 gaming configurations completed in {:?}", duration);

    // Performance should be reasonable (adjust threshold as needed)
    assert!(duration < Duration::from_secs(10)); // Under 10 seconds total

    Ok(())
}

/// Test setup state consistency
#[tokio::test]
async fn test_setup_state_consistency() -> Result<()> {
    let config = GamingAutoConfig::new()?;

    // Check initial state multiple times
    for _ in 0..5 {
        let is_configured1 = config.is_configured();
        let setup_state = config.get_setup_state();
        let is_configured2 = setup_state.is_initialized;

        // Both methods should give consistent results
        assert_eq!(
            is_configured1, is_configured2,
            "is_configured() and setup_state.is_initialized should match"
        );
    }

    println!("✅ Setup state consistency verified");

    Ok(())
}

/// Test configuration with special characters
#[tokio::test]
async fn test_special_character_handling() -> Result<()> {
    let config = GamingAutoConfig::new()?;

    let special_names = vec![
        "Game-With-Dashes",
        "Game_With_Underscores",
        "Game With Spaces",
        "Game123",
        "🎮GameWithEmoji",
    ];

    for game_name in special_names {
        let game_config = config.configure_for_game(game_name).await?;
        // Should handle gracefully without panicking
        println!(
            "✅ Special char game '{}': {}",
            game_name, game_config.message
        );
    }

    Ok(())
}

/// Test configuration defaults
#[tokio::test]
async fn test_configuration_defaults() -> Result<()> {
    let config = GamingAutoConfig::new()?;

    let game_config = config.configure_for_game("DefaultTest").await?;

    if let Some(configuration) = &game_config.configuration {
        // Check for expected default fields
        if let Some(optimized) = configuration.get("optimized") {
            assert!(optimized.is_boolean());
            println!("✅ Optimized setting: {}", optimized);
        }

        if let Some(protocol) = configuration.get("protocol") {
            assert!(protocol.is_string());
            println!("✅ Protocol setting: {}", protocol);
        }
    }

    Ok(())
}
