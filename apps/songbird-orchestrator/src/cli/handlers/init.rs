//! Initialize command handler

use crate::cli::utils::{print_info, print_success};
use anyhow::Result;

/// Initialize command handler
pub async fn handle_init_command(directory: Option<String>, non_interactive: bool) -> Result<()> {
    let target_dir = directory.unwrap_or_else(|| ".".to_string());
    print_info(&format!(
        "🚀 Initializing Songbird in directory: {}",
        target_dir
    ));

    std::fs::create_dir_all(&target_dir)?;

    if non_interactive {
        print_info("📋 Non-interactive mode: Creating default configuration");
        create_default_config(&target_dir).await?;
    } else {
        print_info("📋 Interactive mode: Setting up configuration");
        create_interactive_config(&target_dir).await?;
    }

    print_success("✅ Initialization complete");
    print_info("Next steps:");
    print_info("  1. Review the generated configuration files");
    print_info("  2. Run 'songbird-orchestrator status overview' to check system status");
    print_info("  3. Run 'songbird-orchestrator service list' to see available services");

    Ok(())
}

/// Create default configuration files
async fn create_default_config(target_dir: &str) -> Result<()> {
    let config_dir = format!("{}/.songbird", target_dir);
    std::fs::create_dir_all(&config_dir)?;

    let config_content = r#"# Songbird Universal Orchestrator Configuration
[environment]
data_dir = "./.songbird/data"
log_level = "info"

[network]
bind_port = 8080
discovery_port = 8001
"#;

    let config_path = format!("{}/songbird.toml", config_dir);
    std::fs::write(&config_path, config_content)?;
    print_success(&format!("Created configuration file: {}", config_path));

    Ok(())
}

/// Create configuration interactively
async fn create_interactive_config(target_dir: &str) -> Result<()> {
    print_info("🔧 Interactive configuration setup");
    print_info("Creating default configuration (interactive prompts not implemented yet)");
    create_default_config(target_dir).await?;
    Ok(())
}
