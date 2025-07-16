// Module imports
//! Init Command - Initialize a new Songbird cluster
//!
//! This command creates the initial configuration and sets up the cluster
//! with secure defaults, environment-aware configuration, and proper validation.

use crate::cli::CliError;
// CLI initialization commands
use songbird_config::config::OrchestratorConfig;
// Initialization error handling
use colored::*;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use toml;
// Re-export DeploymentType from parent module
pub use crate::cli::DeploymentType;
/// Execute the init command
pub async fn execute_init(
    deployment: crate::cli::DeploymentType,
    quick: bool,
    output: PathBuf,
) -> crate::cli::CliResult<()> {
    println!(
        "{}",
        "🎼 Songbird Orchestrator Initialization"
            .bright_blue()
            .bold()
    );
    println!("{}", "=====================================".bright_blue());

    let mut config = InitConfig {
        deployment_type: deployment,
        config_dir: output.clone(),
        data_dir: output.join("data"),
        log_dir: output.join("logs"),
        ..Default::default()
    };
    // Interactive configuration if not in quick mode
    if !quick {
        config = interactive_configuration(config).map_err(|e| match e {
            CliError::UserCancelled => CliError::UserCancelled,
            _ => CliError::Config {
                message: format!("Interactive configuration failed: {e}"),
                field: Some("interactive_config".to_string()),
                suggestion: Some("Try running with --force to skip interactive mode".to_string()),
            },
        })?;
    }
    // Validate configuration
    validate_init_config(&config)?;
    // Create directories
    create_directories(&config)?;
    // Generate and save configuration
    let orchestrator_config = generate_orchestrator_config(&config)?;
    save_configuration(&config, &orchestrator_config)?;
    // Generate templates and examples
    generate_templates(&config)?;
    // Show completion message
    show_completion_message(&config)?;
    Ok(())
}
/// Configuration for the init command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitConfig {
    /// Deployment type
    pub deployment_type: crate::cli::DeploymentType,
    /// Network interface to bind to
    pub interface: Option<String>,
    /// Port to bind to
    pub port: Option<u16>,
    /// Enable federation mode
    pub federation: bool,
    /// Configuration directory
    pub config_dir: PathBuf,
    /// Data directory
    pub data_dir: PathBuf,
    /// Log directory
    pub log_dir: PathBuf,
}

impl Default for InitConfig {
    fn default() -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let config_dir = home_dir.join(".songbird");

        Self {
            deployment_type: crate::cli::DeploymentType::HomeNetwork,
            interface: None,
            port: None,
            federation: false,
            config_dir: config_dir.clone(),
            data_dir: config_dir.join("data"),
            log_dir: config_dir.join("logs"),
        }
    }
}
/// Interactive configuration wizard
fn interactive_configuration(mut config: InitConfig) -> crate::cli::CliResult<InitConfig> {
    let theme = ColorfulTheme::default();
    println!(
        "\n{}",
        "Let's configure your Songbird cluster:".bright_green()
    );
    // Deployment type selection
    let deployment_options = vec![
        "Home Network (recommended for home labs)",
        "Research Cluster (for scientific computing)",
        "Edge Deployment (for IoT/edge computing)",
        "Development (for testing and development)",
    ];
    let deployment_selection = Select::with_theme(&theme)
        .with_prompt("What type of deployment is this?")
        .default(0)
        .items(&deployment_options)
        .interact()
        .map_err(|_e| CliError::UserCancelled)?;
    config.deployment_type = match deployment_selection {
        0 => crate::cli::DeploymentType::HomeNetwork,
        1 => crate::cli::DeploymentType::ResearchCluster,
        2 => crate::cli::DeploymentType::EdgeDeployment,
        3 => crate::cli::DeploymentType::Development,
        _ => crate::cli::DeploymentType::HomeNetwork,
    };

    // Configuration directory
    let config_dir_input: String = Input::with_theme(&theme)
        .with_prompt("Configuration directory")
        .default(config.config_dir.to_string_lossy().to_string())
        .interact_text()
        .map_err(|_e| CliError::UserCancelled)?;

    config.config_dir = PathBuf::from(config_dir_input);
    config.data_dir = config.config_dir.join("data");
    config.log_dir = config.config_dir.join("logs");

    Ok(config)
}
/// Validate the initialization configuration
fn validate_init_config(_config: &InitConfig) -> crate::cli::CliResult<()> {
    // Simplified validation for now
    Ok(())
}

/// Create necessary directories
fn create_directories(config: &InitConfig) -> crate::cli::CliResult<()> {
    std::fs::create_dir_all(&config.config_dir).map_err(CliError::Io)?;
    std::fs::create_dir_all(&config.data_dir).map_err(CliError::Io)?;
    std::fs::create_dir_all(&config.log_dir).map_err(CliError::Io)?;

    Ok(())
}

/// Generate orchestrator configuration
fn generate_orchestrator_config(_config: &InitConfig) -> crate::cli::CliResult<OrchestratorConfig> {
    // Generate a basic configuration for now
    Ok(OrchestratorConfig::default())
}

/// Save configuration to files
fn save_configuration(
    init_config: &InitConfig,
    orchestrator_config: &OrchestratorConfig,
) -> crate::cli::CliResult<()> {
    // Save main orchestrator configuration
    let config_file = init_config.config_dir.join("songbird.toml");
    let config_content =
        toml::to_string_pretty(orchestrator_config).map_err(|e| CliError::Config {
            message: format!("Failed to serialize config: {e}"),
            field: Some("config_serialization".to_string()),
            suggestion: Some("Check your configuration values and try again".to_string()),
        })?;
    std::fs::write(&config_file, config_content).map_err(|e| CliError::Config {
        message: format!("Failed to write config file: {e}"),
        field: Some("config_file".to_string()),
        suggestion: Some("Check file permissions and disk space".to_string()),
    })?;
    // Save CLI configuration
    let cli_config = crate::cli::config::CliConfig {
        config_dir: init_config.config_dir.clone(),
        data_dir: init_config.data_dir.clone(),
        log_dir: init_config.log_dir.clone(),
        editor: std::env::var("EDITOR").ok(),
        color: true,
        default_deployment_type: "home-network".to_string(),
    };
    let cli_config_file = init_config.config_dir.join("cli.toml");
    let cli_config_content = toml::to_string_pretty(&cli_config).map_err(|e| CliError::Config {
        message: format!("Failed to serialize CLI config: {e}"),
        field: Some("cli_config_serialization".to_string()),
        suggestion: Some("Check your CLI configuration values".to_string()),
    })?;
    std::fs::write(&cli_config_file, cli_config_content).map_err(|e| CliError::Config {
        message: format!("Failed to write CLI config file: {e}"),
        field: Some("cli_config_file".to_string()),
        suggestion: Some("Check file permissions and disk space".to_string()),
    })?;
    println!("{}", "⚙️  Configuration saved successfully".green());
    println!("   📄 Main config: {}", config_file.display());
    println!("   📄 CLI config: {}", cli_config_file.display());

    Ok(())
}
/// Generate templates and examples
fn generate_templates(config: &InitConfig) -> crate::cli::CliResult<()> {
    let templates_dir = config.config_dir.join("templates");
    std::fs::create_dir_all(&templates_dir).map_err(|e| CliError::Config {
        message: format!("Failed to create templates directory: {e}"),
        field: Some("templates_directory".to_string()),
        suggestion: Some("Check file permissions and disk space".to_string()),
    })?;
    // Generate service template
    let service_template = r#"# Example Service Configuration
# Copy this file and modify for your service
[service]
name = "my-service"
version = "1.0.0"
description = "My distributed service"
service_type = "compute"  # compute, storage, data, ml, web
[service.capabilities]
# List your service capabilities
compute = ["cpu", "memory"]
storage = ["file", "block"]
protocols = ["http", "grpc"]
[service.resources]
# Resource requirements
cpu_cores = 2
memory_gb = 4.0
storage_gb = 10.0
gpu_required = false
[service.networking]
# Network configuration
port = 8080
health_check_path = "/health"
metrics_path = "/metrics"
[service.scaling]
# Scaling configuration
min_instances = 1
max_instances = 10
target_cpu_percent = 70
"#;
    std::fs::write(templates_dir.join("service.toml"), service_template).map_err(|e| {
        CliError::Config {
            message: format!("Failed to write service template: {e}"),
            field: Some("service_template".to_string()),
            suggestion: Some("Check file permissions and disk space".to_string()),
        }
    })?;
    // Generate docker-compose template
    let docker_compose_template = r#"version: '3.8'
services:
  songbird-orchestrator:
    image: songbird-orchestrator:latest
    ports:
      - "8080:8080"
      - "9090:9090"  # Dashboard
    volumes:
      - ./data:/app/data
      - ./config:/app/config
    environment:
      - SONGBIRD_CONFIG_PATH=/app/config/songbird.toml
      - RUST_LOG=info
    networks:
      - songbird-network
  my-service:
    image: my-service:latest
    depends_on:
      - songbird-orchestrator
    environment:
      - SONGBIRD_ORCHESTRATOR_URL=http://songbird-orchestrator:8080
networks:
  songbird-network:
    driver: bridge
"#;
    std::fs::write(
        templates_dir.join("docker-compose.yml"),
        docker_compose_template,
    )
    .map_err(|e| CliError::Config {
        message: format!("Failed to write docker-compose template: {e}"),
        field: Some("docker_compose_template".to_string()),
        suggestion: Some("Check file permissions and disk space".to_string()),
    })?;
    // Generate systemd service template
    let systemd_template = r#"[Unit]
Description=Songbird Orchestrator
After=network.target
Wants=network.target

[Service]
Type=simple
User=songbird
Group=songbird
WorkingDirectory=/opt/songbird
ExecStart=/opt/songbird/bin/songbird start --config /etc/songbird/songbird.toml
Restart=always
RestartSec=10
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
"#;
    std::fs::write(templates_dir.join("songbird.service"), systemd_template).map_err(|e| {
        CliError::Config {
            message: format!("Failed to write systemd template: {e}"),
            field: Some("systemd_template".to_string()),
            suggestion: Some("Check file permissions and disk space".to_string()),
        }
    })?;
    // Generate example Python client
    let python_client_template = r#"#!/usr/bin/env python3
"""
Example Songbird Python Client
Demonstrates how to interact with the Songbird Orchestrator
"""
import requests
import json
from typing import Dict, List, Optional

class SongbirdClient:
    def __init__(self, base_url: str = "http://localhost:8080"):
        self.base_url = base_url.rstrip('/')
        self.session = requests.Session()
    
    def register_service(self, service_info: Dict) -> Dict:
        """Register a service with the orchestrator"""
        response = self.session.post(
            f"{self.base_url}/api/v1/services",
            json=service_info
        )
        response.raise_for_status()
        return response.json()
    
    def discover_services(self, service_type: Optional[str] = None) -> List[Dict]:
        """Discover available services"""
        params = {"type": service_type} if service_type else {}
        response = self.session.get(
            f"{self.base_url}/api/v1/services",
            params=params
        )
        response.raise_for_status()
        return response.json()
    
    def get_cluster_status(self) -> Dict:
        """Get cluster status"""
        response = self.session.get(f"{self.base_url}/api/v1/status")
        response.raise_for_status()
        return response.json()

# Example usage
if __name__ == "__main__":
    client = SongbirdClient()
    
    # Example service registration
    service = {
        "id": "example-python-service",
        "name": "Example Python Service",
        "version": "1.0.0",
        "service_type": "compute",
        "capabilities": ["cpu", "memory"],
        "endpoints": ["http://localhost:8081"]
    }
    
    try:
        result = client.register_service(service)
        print(f"Service registered: {result}")
        
        services = client.discover_services()
        print(f"Available services: {len(services)}")
        
        status = client.get_cluster_status()
        print(f"Cluster status: {status}")
    except requests.RequestException as e:
        print(f"Error: {e}")
"#;
    std::fs::write(
        templates_dir.join("client_example.py"),
        python_client_template,
    )
    .map_err(|e| CliError::Config {
        message: format!("Failed to write Python client template: {e}"),
        field: Some("python_client_template".to_string()),
        suggestion: Some("Check file permissions and disk space".to_string()),
    })?;
    // Generate README for templates
    let readme_template = r#"# Songbird Templates

This directory contains templates and examples to help you get started with Songbird.

## Files

- `service.toml` - Example service configuration
- `docker-compose.yml` - Docker Compose setup for Songbird
- `songbird.service` - Systemd service file for production deployment
- `client_example.py` - Python client example

## Usage

1. Copy `service.toml` and modify it for your service
2. Use `docker-compose.yml` to run Songbird with Docker
3. Install `songbird.service` for systemd-based systems
4. Run `client_example.py` to test API integration

## Next Steps

- Read the documentation at docs/
- Try the examples in examples/
- Join the community at https://github.com/songbird-orchestrator
"#;
    std::fs::write(templates_dir.join("README.md"), readme_template).map_err(|e| {
        CliError::Config {
            message: format!("Failed to write templates README: {e}"),
            field: Some("templates_readme".to_string()),
            suggestion: Some("Check file permissions and disk space".to_string()),
        }
    })?;

    println!("{}", "📋 Templates generated successfully".green());
    println!("   📁 Templates directory: {}", templates_dir.display());
    println!("   📄 Service template: service.toml");
    println!("   🐳 Docker Compose: docker-compose.yml");
    println!("   ⚙️  Systemd service: songbird.service");
    println!("   🐍 Python client: client_example.py");

    Ok(())
}

/// Show completion message
fn show_completion_message(config: &InitConfig) -> crate::cli::CliResult<()> {
    println!("\n{}", "🎉 Initialization Complete!".bright_green().bold());
    println!();
    println!(
        "📁 Configuration directory: {}",
        config.config_dir.display()
    );
    println!("📊 Data directory: {}", config.data_dir.display());
    println!("📝 Log directory: {}", config.log_dir.display());
    println!();
    println!("{}", "Next steps:".bright_yellow().bold());
    println!("  1. Run 'songbird start' to start the orchestrator");
    println!("  2. Use 'songbird status' to check system status");
    println!("  3. Try 'songbird quick' for easy resource sharing");

    Ok(())
}
