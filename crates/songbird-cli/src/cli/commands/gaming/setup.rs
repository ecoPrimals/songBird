//! Gaming setup functionality - one-touch, zero-touch, family-safe setup

use crate::cli::CliResult;
use colored::Colorize;

/// Execute one-touch setup
pub async fn execute_one_touch(
    name: String,
    family_safe: bool,
    _parental_controls: bool,
    _guests: bool,
) -> CliResult<()> {
    println!(
        "{}",
        format!("🚀 One-touch setup for: {name}").bright_green()
    );
    if family_safe {
        println!("👨‍👩‍👧‍👦 Family-safe mode enabled");
    }
    Ok(())
}

/// Execute zero-touch setup
pub async fn execute_zero_touch(
    _endpoint: Option<String>,
    _token: Option<String>,
) -> CliResult<()> {
    println!("{}", "⚡ Zero-touch setup with BearDog".bright_green());
    Ok(())
}

/// Execute family-safe setup
pub async fn execute_family_safe(family_name: String) -> CliResult<()> {
    println!(
        "{}",
        format!("👨‍👩‍👧‍👦 Family-safe setup for: {family_name}").bright_green()
    );
    Ok(())
}

/// Execute quick start
pub async fn execute_quick_start(
    _auto_detect: bool,
    _game: Option<String>,
    _family_safe: bool,
    name: Option<String>,
) -> CliResult<()> {
    let session_name = name.unwrap_or_else(|| "Quick Gaming Session".to_string());
    println!(
        "{}",
        format!("⚡ Quick start: {session_name}").bright_green()
    );
    Ok(())
}
