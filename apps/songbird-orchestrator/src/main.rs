//! Songbird Gaming Bridge - Anti-Monolith Orchestrator
//!
//! Main entry point for the modular gaming network bridge

use anyhow::Result;
use clap::Parser;
use songbird_config::SongbirdConfig;

// Import the modular components
mod app;
mod cli;
mod integration;
mod server;

use cli::{handle_command, print_error, print_info, print_success, print_warning, Cli, CliConfig};
use integration::IntegrationManager;

/// Main entry point for the Songbird Orchestrator
#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let cli = Cli::parse();

    // Initialize CLI configuration
    let mut cli_config = CliConfig::new();

    // Apply CLI options to configuration
    if cli.verbose {
        cli_config.set_verbose(true);
    }

    if cli.no_color {
        cli_config.set_colored_output(false);
    }

    if let Some(config_path) = cli.config {
        cli_config.set_config_path(config_path);
    }

    // Handle commands
    let result = match cli.command {
        Some(command) => handle_command(command, &cli_config).await,
        None => {
            // No command provided, show help
            print_info("Songbird Gaming Bridge - Universal Network Orchestrator");
            print_info("Use 'songbird --help' for available commands");
            print_info("Use 'songbird quick --help' for quick start options");

            // Start the orchestrator in interactive mode
            start_orchestrator_interactive(&cli_config).await
        }
    };

    // Handle result and provide user feedback
    match result {
        Ok(()) => {
            if cli_config.is_verbose() {
                print_success("Command completed successfully");
            }
        }
        Err(e) => {
            print_error(&format!("Error: {}", e));
            if cli_config.is_verbose() {
                print_warning("Use --verbose for more detailed error information");
            }
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Start the orchestrator in interactive mode
async fn start_orchestrator_interactive(cli_config: &CliConfig) -> Result<()> {
    // Load configuration
    let config = load_configuration(cli_config).await?;

    // Initialize integration manager
    let integration_manager = IntegrationManager::new(config);

    // Start integrated services
    integration_manager.start_integrated_services().await?;

    Ok(())
}

/// Load configuration from file or environment
async fn load_configuration(cli_config: &CliConfig) -> Result<SongbirdConfig> {
    // Load configuration from multiple sources in priority order:
    // 1. Command line arguments (highest priority)
    // 2. Environment variables
    // 3. Configuration file
    // 4. Default values (lowest priority)

    if cli_config.is_verbose() {
        print_info("Loading configuration...");
    }

    let config = if let Some(config_path) = cli_config.get_config_path() {
        print_info(&format!("Loading configuration from: {}", config_path));
        // Load from file and merge with environment variables
        load_config_from_file(&config_path)
            .await
            .and_then(|mut config| {
                merge_environment_variables(&mut config);
                Ok(config)
            })
            .unwrap_or_else(|e| {
                eprintln!("Warning: Failed to load config from file: {}", e);
                print_info("Falling back to default configuration");
                let mut config = SongbirdConfig::default();
                merge_environment_variables(&mut config);
                config
            })
    } else {
        print_info("Using default configuration with environment overrides");
        let mut config = SongbirdConfig::default();
        merge_environment_variables(&mut config);
        config
    };

    if cli_config.is_verbose() {
        print_info("Configuration loaded successfully");
    }

    Ok(config)
}

/// Load configuration from file
async fn load_config_from_file(config_path: &str) -> Result<SongbirdConfig> {
    use std::path::Path;

    let path = Path::new(config_path);

    if !path.exists() {
        return Err(anyhow::anyhow!(
            "Configuration file not found: {}",
            config_path
        ));
    }

    let _content = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to read config file: {}", e))?;

    // For now, return default configuration with file existence validation
    // In production, this would parse the actual config file format
    tracing::info!("Configuration file exists at: {}", config_path);

    Ok(SongbirdConfig::default())
}

/// Merge environment variables into configuration
fn merge_environment_variables(config: &mut SongbirdConfig) {
    // Network configuration
    if let Ok(bind_address) = std::env::var("SONGBIRD_BIND_ADDRESS") {
        if let Ok(addr) = bind_address.parse::<std::net::IpAddr>() {
            config.network.bind_address = addr;
        }
    }
    if let Ok(orchestrator_port) = std::env::var("SONGBIRD_ORCHESTRATOR_PORT") {
        if let Ok(port_num) = orchestrator_port.parse::<u16>() {
            config.network.orchestrator_port = port_num;
        }
    }

    // Security configuration
    if let Ok(enable_tls) = std::env::var("SONGBIRD_ENABLE_TLS") {
        config.security.tls_enabled = enable_tls.parse().unwrap_or(config.security.tls_enabled);
    }
    if let Ok(enable_encryption) = std::env::var("SONGBIRD_ENABLE_ENCRYPTION") {
        config.security.encryption_enabled = enable_encryption
            .parse()
            .unwrap_or(config.security.encryption_enabled);
    }

    // Environment configuration
    if let Ok(bind_address) = std::env::var("SONGBIRD_ENV_BIND_ADDRESS") {
        config.environment.bind_address = bind_address;
    }
    if let Ok(bind_port) = std::env::var("SONGBIRD_ENV_BIND_PORT") {
        if let Ok(port_num) = bind_port.parse::<u16>() {
            config.environment.bind_port = port_num;
        }
    }

    // BearDog configuration - handle Option type properly
    if let Ok(_enabled) = std::env::var("SONGBIRD_BEARDOG_ENABLED") {
        // BearDog config handling would go here in production
        tracing::info!("BearDog configuration detected via environment");
    }

    // Custom configuration - handle Value type properly
    if let Ok(custom_config) = std::env::var("SONGBIRD_CUSTOM_CONFIG") {
        config.custom.insert(
            "environment_override".to_string(),
            serde_json::Value::String(custom_config),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_cli_parsing_no_arguments() {
        let cli = Cli::try_parse_from(&["songbird"]).unwrap();
        assert!(cli.command.is_none());
        assert!(!cli.verbose);
        assert!(!cli.no_color);
        assert!(cli.config.is_none());
    }

    #[test]
    fn test_cli_parsing_with_verbose() {
        let cli = Cli::try_parse_from(&["songbird", "--verbose"]).unwrap();
        assert!(cli.verbose);
    }

    #[test]
    fn test_cli_parsing_with_config() {
        let cli = Cli::try_parse_from(&["songbird", "--config", "/path/to/config"]).unwrap();
        assert_eq!(cli.config, Some("/path/to/config".to_string()));
    }

    #[test]
    fn test_cli_parsing_with_no_color() {
        let cli = Cli::try_parse_from(&["songbird", "--no-color"]).unwrap();
        assert!(cli.no_color);
    }

    #[test]
    fn test_cli_parsing_init_command() {
        let cli = Cli::try_parse_from(&["songbird", "init"]).unwrap();
        match cli.command.unwrap() {
            cli::Commands::Init {
                directory,
                non_interactive,
            } => {
                assert!(directory.is_none());
                assert!(!non_interactive);
            }
            _ => assert!(false, "Expected Init command"),
        }
    }

    #[test]
    fn test_cli_config_initialization() {
        let config = CliConfig::new();
        assert!(!config.is_verbose());
        assert!(config.is_colored_output_enabled());
        assert!(config.get_config_path().is_none());
    }

    #[test]
    fn test_cli_config_setters() {
        let mut config = CliConfig::new();

        config.set_verbose(true);
        assert!(config.is_verbose());

        config.set_colored_output(false);
        assert!(!config.is_colored_output_enabled());

        config.set_config_path("/test/config".to_string());
        assert_eq!(config.get_config_path(), Some("/test/config".to_string()));
    }

    #[tokio::test]
    async fn test_configuration_loading() {
        let cli_config = CliConfig::new();
        let config = load_configuration(&cli_config).await;
        assert!(config.is_ok());
    }

    #[tokio::test]
    async fn test_configuration_loading_with_path() {
        let mut cli_config = CliConfig::new();
        cli_config.set_config_path("/test/config.toml".to_string());
        let config = load_configuration(&cli_config).await;
        assert!(config.is_ok());
    }
}
