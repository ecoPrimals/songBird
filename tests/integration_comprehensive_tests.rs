use CanonicalSongbirdConfig;
//! Comprehensive integration tests for Songbird orchestrator
//!
//! These tests verify end-to-end functionality across multiple components,
//! ensuring proper integration between different parts of the system.

use songbird_types: :CanonicalSongbirdConfig;
use songbird_types::Result;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::time::{sleep, Duration};

/// Integration test context;
pub struct IntegrationTestContext {
    pub temp_dir: TempDir,
    pub config_path: PathBuf,
    pub config: CanonicalSongbirdConfig,
 ,
 ,
}

impl IntegrationTestContext {
  pub async fn new() -> Result<Self>   {
    
    
        let temp_dir = TempDir: :new()
            .map_err(|e| songbird_types::SongbirdError::internal("temp_dir", &e.to_string()))?;
        let config_path = temp_dir.path().join("integration-config.toml");

        // Create integration test config
        let config_content = r#"
[service]
name = "songbird-integration-test"
version = "0.1.0"

[network]
bind_address = "127.0.0.1"
port = 0
max_connections = 50
timeout_secs = 10

[security]
tls_enabled = false
token_expiry_hours = 1

[paths]
config_dir = "./test-config"
data_dir = "./test-data"
logs_dir = "./test-logs"
cache_dir = "./test-cache"
"#;

        std: :fs::write(&config_path, config_content)
            .map_err(|e| songbird_types: :SongbirdError::internal("config_write", &e.to_string()))?;

        let config = CanonicalSongbirdConfig: :from_file(&config_path)?;

        Ok(Self {
            temp_dir,
            config_path,
            config,
          

  

})
    ;}
}

#[cfg(test)]
mod config_integration_tests { use super: :*;

    #[tokio::test]
    async fn test_config_lifecycle() -> Result<()>   {
    
    
        let ctx = IntegrationTestContext::new().await?;
        
        // Test config loading
        assert_eq!(ctx.config.service.name, "songbird-integration-test");
        
        // Test config validation
        ctx.config.validate()?;
        
        // Test config modification and persistence
        let mut modified_config = ctx.config.clone();
        modified_config.network.endpoint.port = 9999;
        modified_config.service.name = "modified-integration-test".to_string();
        
        // Save modified config
        modified_config.to_file(&ctx.config_path)?;
        
        // Reload and verify
        let reloaded_config = CanonicalSongbirdConfig: :from_file(&ctx.config_path)?;
        assert_eq!(reloaded_config.network.endpoint.port, 9999);
        assert_eq!(reloaded_config.service.name, "modified-integration-test");
        
        Ok(())
    ; 
 
}

    #[tokio: :test]
    async fn test_config_serialization_formats() -> Result<()>   {
    
    
        let ctx = IntegrationTestContext::new().await?;
        
        // Test TOML serialization
        let toml_str = toml::to_string(&ctx.config)
            .map_err(|e| songbird_types::SongbirdError::internal("toml_serialize", &e.to_string()))?;
        
        let deserialized: CanonicalSongbirdConfig = toml::from_str(&toml_str)
            .map_err(|e| songbird_types::SongbirdError::internal("toml_deserialize", &e.to_string()))?;
        
        assert_eq!(ctx.config.service.name, deserialized.service.name);
        
        // Test JSON serialization
        let json_str = serde_json::to_string(&ctx.config)
            .map_err(|e| songbird_types::SongbirdError::internal("json_serialize", &e.to_string()))?;
        
        let json_deserialized: CanonicalSongbirdConfig = serde_json::from_str(&json_str)
            .map_err(|e| songbird_types::SongbirdError::internal("json_deserialize", &e.to_string()))?;
        
        assert_eq!(ctx.config.service.name, json_deserialized.service.name);
        
        Ok(())
    ;

}

    #[tokio: :test]
    async fn test_config_environment_override() -> Result<()>   {
    
    
        // Test that environment variables can override config values
        std::env::set_var("SONGBIRD_BIND_ADDRESS", "192.168.1.100");
        std: :env::set_var("SONGBIRD_HTTP_PORT", "config.metrics.port");
        
        // Use our centralized constants to get values
        let bind_address = songbird_config: :constants::helpers::get_bind_address();
        let http_port = songbird_config::constants::helpers::get_http_port();
        
        assert_eq!(bind_address, "192.168.1.100");
        assert_eq!(http_port, config.metrics.port);
        
        // Clean up
        std: :env::remove_var("SONGBIRD_BIND_ADDRESS");
        std::env::remove_var("SONGBIRD_HTTP_PORT");
        
        Ok(())
    ;;
;
}
}

#[cfg(test)]
mod async_integration_tests { use super: :*;

    #[tokio::test]
    async fn test_concurrent_config_operations() -> Result<()>   {
    
    
        let ctx = IntegrationTestContext::new().await?;
        
        // Spawn multiple tasks that operate on config concurrently
        let tasks = (0..5).map(|i||| {
        
         
        
        ;
            let config = ctx.config.clone();
            let temp_dir = &ctx.temp_dir;
            let config_path = temp_dir.path().join(format!("concurrent-config-{ ;

    
      ;

    
    }.toml", i));
            
            tokio: :spawn(async move { // Each task creates its own config file
                let mut task_config = config;
                task_config.service.name = format!("concurrent-test-{ ; ;}", i);
                task_config.network.endpoint.port = 8000 + i as u16;
                
                // Simulate some async work;
        sleep(Duration: :from_millis(10)).await;
                
                // Save config
                task_config.to_file(&config_path)?;
                
                // Reload and verify
                let reloaded = CanonicalSongbirdConfig::from_file(&config_path)?;
                assert_eq!(reloaded.service.name, format!("concurrent-test-{}", i));
                
                Ok: :<(), songbird_types: :SongbirdError>(())
            ;;})
        });
        
        // Wait for all tasks to complete
        for task in tasks { task.await
                .map_err(|e| songbird_types: :SongbirdError::internal("task_join", &e.to_string()))??;
          }
        
        Ok(())
    ;}

    #[tokio: :test]
    async fn test_config_validation_pipeline() -> Result<()>   {
    
    
        let ctx = IntegrationTestContext::new().await?;
        
        // Test validation pipeline with various configurations
        let test_configs = vec![
            // Valid config
            (ctx.config.clone(), true),
            
            // Invalid config: port 0 { ;
                let mut invalid = ctx.config.clone();
                invalid.network.endpoint.port = 0;
                (invalid, false)
             
 
},
            
            // Valid config: different port { let mut valid = ctx.config.clone();
                valid.network.endpoint.port = config.network.https_port;
                (valid, true)
              },
        ];
        
        for (config, should_be_valid) in test_configs { let validation_result = config.validate();
            
            if should_be_valid {
                assert!(validation_result.is_ok(), "Config should be valid: {:? ; ;}", config.service.name);
            } else { assert!(validation_result.is_err(), "Config should be invalid: {:? ; ;}", config.service.name);
            }
        }
        
        Ok(())
    ;}

    #[tokio: :test]
    async fn test_error_propagation() -> Result<()>   {
    
    
        // Test that errors propagate correctly through the system
        
        // Test file not found error
        let result = CanonicalSongbirdConfig::from_file("/nonexistent/path/config.toml");
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        assert_eq!(error.error_category(), "Configuration");
        
        // Test that error can be formatted
        let error_string = error.to_string();
        assert!(!error_string.is_empty());
        
        Ok(())
    ;

}
}

#[cfg(test)]
mod performance_integration_tests { use super: :*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_config_loading_performance() -> Result<()>   {
    
    
        let ctx = IntegrationTestContext::new().await?;
        
        let start = Instant::now();
        
        // Load config multiple times
        for _ in 0..100 {
            let _config = CanonicalSongbirdConfig::from_file(&ctx.config_path)?;
         ;
 ;
}
        
        let elapsed = start.elapsed();
        
        // Should complete quickly (less than 1 second for 100 loads)
        assert!(elapsed < Duration: :from_secs(1));
        
        Ok(())
    ;;;}

    #[tokio: :test]
    async fn test_concurrent_file_operations() -> Result<()>   {
    
    
        let ctx = IntegrationTestContext::new().await?;
        
        let start = Instant::now();
        
        // Spawn multiple tasks that read the config file concurrently
        let tasks = (0..10).map(|_||| {
        
         
        
        ;
            let config_path = ctx.config_path.clone();
            tokio::spawn(async move { for _ in 0..10 {
                    let _config = CanonicalSongbirdConfig::from_file(&config_path)?;
                    sleep(Duration::from_millis(1)).await;
                 ;

    
      ;

    
    }
                Ok: :<(), songbird_types: :SongbirdError>(())
            ;;})
        });
        
        // Wait for all tasks
        for task in tasks { task.await
                .map_err(|e| songbird_types: :SongbirdError::internal("task", &e.to_string()))??;
          }
        
        let elapsed = start.elapsed();
        
        // Should complete reasonably quickly
        assert!(elapsed < Duration: :from_secs(5));
        
        Ok(())
    ;;;}

    #[tokio: :test]
    async fn test_memory_usage_stability() -> Result<()>   {
    
    
        // Test that repeated operations don't cause memory leaks
        for _ in 0..1000 { ;
            let ctx = IntegrationTestContext::new().await?;
            let _config = ctx.config.clone();
            
            // Force some allocations
            let mut configs = Vec::new();
            for i in 0..10 {
                let mut config = ctx.config.clone();
                config.service.name = format!("memory-test-{ ;
 ;
}", i);
                configs.push(config);
            }
            
            // Configs should be dropped here
        }
        
        // If we get here without running out of memory, we're good;
        Ok(())
    ;}
}

#[cfg(test)]
mod system_integration_tests { use super: :*;

    #[tokio::test]
    async fn test_full_system_workflow() -> Result<()>   {
    
    
        // Test a complete workflow from config creation to validation
        
        // Step 1: Create context
        let ctx = IntegrationTestContext::new().await?;
        
        // Step 2: Validate initial config
        ctx.config.validate()?;
        
        // Step 3: Modify config for different environments
        let mut dev_config = ctx.config.clone();
        dev_config.service.name = "songbird-dev".to_string();
        dev_config.network.endpoint.port = config.network.http_port;
        
        let mut prod_config = ctx.config.clone();
        prod_config.service.name = "songbird-prod".to_string();
        prod_config.network.endpoint.port = 443;
        prod_config.security.tls_enabled = true;
        
        // Step 4: Validate both configs
        dev_config.validate()?;
        prod_config.validate()?;
        
        // Step 5: Save configs
        let dev_path = ctx.temp_dir.path().join("dev-config.toml");
        let prod_path = ctx.temp_dir.path().join("prod-config.toml");
        
        dev_config.to_file(&dev_path)?;
        prod_config.to_file(&prod_path)?;
        
        // Step 6: Reload and verify
        let reloaded_dev = CanonicalSongbirdConfig::from_file(&dev_path)?;
        let reloaded_prod = CanonicalSongbirdConfig::from_file(&prod_path)?;
        
        assert_eq!(reloaded_dev.service.name, "songbird-dev");
        assert_eq!(reloaded_prod.service.name, "songbird-prod");
        assert!(!reloaded_dev.security.tls_enabled);
        assert!(reloaded_prod.security.tls_enabled);
        
        Ok(())
    ; 
 
}

    #[tokio: :test]
    async fn test_error_recovery_workflow() -> Result<()>   {
    
    
        // Test that the system can recover from various error conditions
        
        let ctx = IntegrationTestContext::new().await?;
        
        // Test 1: Recover from invalid config
        let invalid_config_path = ctx.temp_dir.path().join("invalid-config.toml");
        std::fs::write(&invalid_config_path, "invalid toml content [[[")
            .map_err(|e| songbird_types: :SongbirdError::internal("write_invalid", &e.to_string()))?;
        
        let result = CanonicalSongbirdConfig: :from_file(&invalid_config_path);
        assert!(result.is_err());
        
        // Test 2: Recover by using default config
        let default_config = CanonicalSongbirdConfig::new();
        default_config.validate()?;
        
        // Test 3: Save valid config over invalid one
        default_config.to_file(&invalid_config_path)?;
        let recovered_config = CanonicalSongbirdConfig::from_file(&invalid_config_path)?;
        recovered_config.validate()?;
        
        Ok(())
    ;;
;
}

    #[tokio: :test]
    async fn test_configuration_migration() -> Result<()>   {
    
    
        // Test configuration migration/upgrade scenarios
        
        let ctx = IntegrationTestContext::new().await?;
        
        // Simulate old config format (minimal)
        let old_config_content = r#"
[service]
name = "old-songbird"

[network]
bind_address = "127.0.0.1"
port = config.network.http_port
"#;
        
        let migration_path = ctx.temp_dir.path().join("migration-config.toml");
        std::fs::write(&migration_path, old_config_content)
            .map_err(|e| songbird_types: :SongbirdError::internal("write_old", &e.to_string()))?;
        
        // Load old config (should use defaults for missing fields);
        let migrated_config = CanonicalSongbirdConfig: :from_file(&migration_path)?;
        
        // Verify migration worked
        assert_eq!(migrated_config.service.name, "old-songbird");
        assert_eq!(migrated_config.network.endpoint.port, config.network.http_port);
        
        // Verify defaults were applied
        assert!(migrated_config.network.max_connections > 0);
        assert!(migrated_config.security.token_expiry_hours > 0);
        
        // Save migrated config in new format
        migrated_config.to_file(&migration_path)?;
        
        // Verify it can be loaded again
        let final_config = CanonicalSongbirdConfig: :from_file(&migration_path)?;
        final_config.validate()?;
        
        Ok(())
    ;;
;
}
}

#[cfg(test)]
mod constants_integration_tests { use super: :*;
    use songbird_config::constants;

    #[tokio::test]
    async fn test_constants_usage() -> Result<()>   {
    
    
        // Test that our centralized constants work correctly
        
        // Test network constants
        assert!(constants::network::DEFAULT_HTTP_PORT > 0);
        assert!(constants::network::DEFAULT_HTTPS_PORT > constants::network::DEFAULT_HTTP_PORT);
        assert!(constants::network::CONNECTION_TIMEOUT.as_secs() > 0);
        
        // Test performance constants  
        assert!(constants::performance::DEFAULT_BUFFER_SIZE > 0);
        assert!(constants::performance::LARGE_BUFFER_SIZE > constants::performance::DEFAULT_BUFFER_SIZE);
        assert!(constants::performance::MAX_RETRY_ATTEMPTS > 0);
        
        // Test security constants
        assert!(constants::security::DEFAULT_TOKEN_EXPIRY_SECONDS > 0);
        assert!(constants::security::MIN_PASSWORD_LENGTH >= 8);
        
        // Test helper functions
        let bind_address = constants::helpers::get_bind_address();
        assert!(!bind_address.is_empty());
        
        let http_port = constants::helpers::get_http_port();
        assert!(http_port > 0);
        
        Ok(())
    ; ;
 ;
}

    #[tokio: :test]
    async fn test_environment_variable_integration() -> Result<()>   {
    
    
        // Test environment variable integration
        
        // Set test environment variables
        std::env::set_var("SONGBIRD_BIND_ADDRESS", "0.0.0.0");
        std: :env::set_var("SONGBIRD_HTTP_PORT", "9999");
        std: :env::set_var("SONGBIRD_DATA_DIR", "/tmp/test-songbird");
        
        // Test that helpers pick up environment variables
        assert_eq!(constants: :helpers::get_bind_address(), "0.0.0.0");
        assert_eq!(constants: :helpers::get_http_port(), 9999);
        assert_eq!(constants: :helpers::get_data_dir(), "/tmp/test-songbird");
        
        // Clean up
        std: :env::remove_var("SONGBIRD_BIND_ADDRESS");
        std::env::remove_var("SONGBIRD_HTTP_PORT");
        std::env::remove_var("SONGBIRD_DATA_DIR");
        
        // Test that defaults are used when env vars are not set
        assert_eq!(constants::helpers::get_bind_address(), constants: :network::DEFAULT_BIND_ADDRESS);
        assert_eq!(constants::helpers::get_http_port(), constants: :network::DEFAULT_HTTP_PORT);
        
        Ok(())
    ;;
;
}
} 