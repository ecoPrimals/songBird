//! Gaming session management

use super::utils::generate_session_code;
use crate::cli::CliResult;
use colored::Colorize;
use songbird_errors::Result;

/// Host a gaming session
pub async fn host_gaming_session(
    _auto: bool,
    name: Option<String>,
    _encrypt: bool,
    _private: bool,
) -> Result<()> {
    let session_name = name.unwrap_or_else(|| "Songbird Gaming Session".to_string());
    println!(
        "{}",
        format!("🎮 Hosting gaming session: {}", session_name).bright_green()
    );

    let session_code = generate_session_code();
    println!("🔗 Join Code: {}", session_code.bright_yellow());
    Ok(())
}

/// Join a gaming session
pub async fn join_gaming_session(code: Option<String>) -> Result<()> {
    println!("🎮 Joining gaming session...");
    if let Some(code) = code {
        println!("🔗 Session code: {}", code);
    }
    Ok(())
}

/// Show gaming status
pub async fn show_gaming_status() -> Result<()> {
    println!("{}", "🎮 Gaming Status".bright_green());
    Ok(())
}

// Legacy execute functions returning CliResult
pub async fn execute_host(_auto: bool) -> CliResult<()> {
    host_gaming_session(false, None, false, false)
        .await
        .map_err(|e| crate::cli::CliError::ExecutionError(e.to_string()))
}

pub async fn execute_join(code: String) -> CliResult<()> {
    join_gaming_session(Some(code))
        .await
        .map_err(|e| crate::cli::CliError::ExecutionError(e.to_string()))
}

pub async fn execute_status() -> CliResult<()> {
    show_gaming_status()
        .await
        .map_err(|e| crate::cli::CliError::ExecutionError(e.to_string()))
}

pub async fn execute_browse() -> CliResult<()> {
    println!("🔍 Browsing sessions...");
    Ok(())
}

pub async fn execute_diagnostics() -> CliResult<()> {
    println!("🔧 Running diagnostics...");
    Ok(())
}

pub async fn execute_configure() -> CliResult<()> {
    println!("⚙️  Configuration...");
    Ok(())
}
