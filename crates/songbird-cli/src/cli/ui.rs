// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! CLI UI utilities for beautiful terminal output

use crate::errors::{CliError, SongbirdResult};
use colored::{ColoredString, Colorize};
use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};
use indicatif::{ProgressBar, ProgressStyle};
use std::fmt::Display;
use std::time::Duration;
use terminal_size::{Height, Width, terminal_size};

/// Create a styled progress bar with enhanced formatting
#[must_use]
pub fn progress_bar(len: u64) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} {msg}",
            )
            .unwrap_or_else(|_| ProgressStyle::default_bar().progress_chars("█▇▆▅▄▃▂▁  ")),
    );
    pb
}

/// Create a modern spinner for indefinite progress
#[must_use]
pub fn spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner().tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")),
    );
    pb.set_message(message.to_string());
    pb
}

/// Show a confirmation prompt with enhanced styling
pub fn confirm(message: &str, default: bool) -> SongbirdResult<bool> {
    let theme = ColorfulTheme::default();
    Confirm::with_theme(&theme)
        .with_prompt(message)
        .default(default)
        .interact()
        .map_err(|_| CliError::UserCancelled.into())
}

/// Show a selection prompt with enhanced options
pub fn select<T: Display>(
    message: &str,
    items: &[T],
    default: Option<usize>,
) -> SongbirdResult<usize> {
    let theme = ColorfulTheme::default();
    let mut select = Select::with_theme(&theme).with_prompt(message).items(items);

    if let Some(default_idx) = default {
        select = select.default(default_idx);
    }
    select.interact().map_err(|_| CliError::UserCancelled.into())
}

/// Show a text input prompt with validation
pub fn input_text(message: &str, default: Option<&str>) -> SongbirdResult<String> {
    let theme = ColorfulTheme::default();
    let mut input = Input::with_theme(&theme).with_prompt(message);
    if let Some(default_value) = default {
        input = input.default(default_value.to_string());
    }
    input.interact_text().map_err(|_| CliError::UserCancelled.into())
}

/// Show a password input prompt with security considerations
pub fn input_password(message: &str) -> SongbirdResult<String> {
    let theme = ColorfulTheme::default();
    dialoguer::Password::with_theme(&theme)
        .with_prompt(message)
        .interact()
        .map_err(|_| CliError::UserCancelled.into())
}

/// Print colored success message with icon
#[must_use]
pub fn success(message: &str) -> String {
    format!("✅ {}", message.green().bold())
}

/// Print colored info message with icon
#[must_use]
pub fn info(message: &str) -> String {
    format!("ℹ️  {}", message.blue())
}

/// Print colored warning message with icon
#[must_use]
pub fn warn(message: &str) -> String {
    format!("⚠️  {}", message.yellow().bold())
}

/// Print colored error message with icon
#[must_use]
pub fn error(message: &str) -> String {
    format!("❌ {}", message.red().bold())
}

/// Print debugging message with icon
#[must_use]
pub fn debug(message: &str) -> String {
    format!("🔍 {}", message.magenta().dimmed())
}

/// Print progress message with icon
#[must_use]
pub fn progress(message: &str) -> String {
    format!("⏳ {}", message.cyan())
}

/// Print success message to stdout
pub fn print_success(message: &str) {
    println!("{}", success(message));
}

/// Print info message to stdout
pub fn print_info(message: &str) {
    println!("{}", info(message));
}

/// Print warning message to stdout
pub fn print_warning(message: &str) {
    println!("{}", warn(message));
}

/// Print error message to stderr
pub fn print_error(message: &str) {
    eprintln!("{}", error(message));
}

/// Print debug message to stdout
pub fn print_debug(message: &str) {
    println!("{}", debug(message));
}

/// Print a formatted header with consistent styling
pub fn header(title: &str) {
    println!("\n{}", title.bright_blue().bold());
    println!("{}", "━".repeat(title.len()).bright_blue());
}

/// Print a formatted subheader
pub fn subheader(title: &str) {
    println!("\n{}", title.bright_cyan().bold());
    println!("{}", "─".repeat(title.len()).bright_cyan());
}

/// Print a section separator
pub fn separator() {
    println!("{}", "─".repeat(50).dimmed());
}

/// Print a prominent banner
pub fn banner(title: &str, subtitle: Option<&str>) {
    let width = title.len().max(subtitle.map_or(0, str::len));
    let border = "═".repeat(width + 4);

    println!("\n{}", border.bright_blue().bold());
    println!("  {}", title.bright_blue().bold());
    if let Some(sub) = subtitle {
        println!("  {}", sub.bright_cyan());
    }
    println!("{}", border.bright_blue().bold());
}

/// Format bytes in human-readable format
#[must_use]
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "display-only human-readable size; integer part only"
        )]
        #[expect(
            clippy::cast_sign_loss,
            reason = "display-only; value is non-negative after unit scaling"
        )]
        {
            format!("{} {}", size as u64, UNITS[unit_index])
        }
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Format duration in human-readable format
#[must_use]
pub fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let days = total_seconds / 86400;
    let hours = (total_seconds % 86400) / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if days > 0 {
        format!("{days}d {hours}h {minutes}m")
    } else if hours > 0 {
        format!("{hours}h {minutes}m")
    } else if minutes > 0 {
        format!("{minutes}m {seconds}s")
    } else {
        format!("{seconds}s")
    }
}

/// Format percentage with styling
#[must_use]
pub fn format_percentage(value: f64) -> String {
    let percentage = value * 100.0;
    let color = if percentage >= 90.0 {
        "green"
    } else if percentage >= 70.0 {
        "yellow"
    } else {
        "red"
    };
    format!("{percentage:.1}%").color(color).to_string()
}

/// Format health status with color coding
#[must_use]
pub fn format_health_status(status: &str) -> String {
    match status.to_lowercase().as_str() {
        "healthy" | "ok" | "running" => format!("🟢 {}", status.green().bold()),
        "warning" | "degraded" => format!("🟡 {}", status.yellow().bold()),
        "error" | "failed" | "stopped" => format!("🔴 {}", status.red().bold()),
        _ => format!("⚪ {}", status.dimmed()),
    }
}

/// CLI UI helper functions for beautiful output
#[must_use]
pub fn title(message: &str) -> ColoredString {
    message.bold().bright_blue()
}

/// Get terminal width safely
#[must_use]
pub fn terminal_width() -> usize {
    terminal_size().map_or(80, |(Width(w), _)| usize::from(w))
}

/// Get terminal height safely
#[must_use]
pub fn terminal_height() -> usize {
    terminal_size().map_or(24, |(_, Height(h))| usize::from(h))
}

/// Create a table-like output with enhanced formatting
pub struct Table {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    widths: Vec<usize>,
}

impl Table {
    /// Create a new table
    #[must_use]
    pub fn new() -> Self {
        Self {
            headers: Vec::new(),
            rows: Vec::new(),
            widths: Vec::new(),
        }
    }

    /// Add headers to the table
    #[must_use]
    pub fn headers(mut self, headers: Vec<String>) -> Self {
        self.widths = headers.iter().map(std::string::String::len).collect();
        self.headers = headers;
        self
    }

    /// Add a row to the table
    #[must_use]
    pub fn row(mut self, row: Vec<String>) -> Self {
        // Update column widths
        for (i, cell) in row.iter().enumerate() {
            if i < self.widths.len() {
                self.widths[i] = self.widths[i].max(cell.len());
            } else {
                self.widths.push(cell.len());
            }
        }
        self.rows.push(row);
        self
    }

    /// Print the table with enhanced formatting
    pub fn print(self) {
        if self.headers.is_empty() {
            return;
        }

        // Print headers
        let header_line = self
            .headers
            .iter()
            .enumerate()
            .map(|(i, h)| format!("{:width$}", h, width = self.widths[i]))
            .collect::<Vec<_>>()
            .join(" │ ");

        println!("{}", header_line.bright_blue().bold());

        // Print separator
        let separator_line =
            self.widths.iter().map(|&w| "─".repeat(w)).collect::<Vec<_>>().join("─┼─");
        println!("{}", separator_line.bright_blue());

        // Print rows
        for row in self.rows {
            let row_line = row
                .iter()
                .enumerate()
                .map(|(i, cell)| {
                    let cell_len = cell.len();
                    let width = self.widths.get(i).unwrap_or(&cell_len);
                    format!("{cell:width$}")
                })
                .collect::<Vec<_>>()
                .join(" │ ");
            println!("{row_line}");
        }
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

/// Show a loading animation for async operations
pub async fn with_loading<F, T>(message: &str, future: F) -> T
where
    F: std::future::Future<Output = T>,
{
    let pb = spinner(message);
    // Use configurable refresh interval (with minimum for UI responsiveness)
    let ui_refresh_interval = songbird_config::canonical::constants::health::DEFAULT_CHECK_TIMEOUT
        .min(Duration::from_millis(500)) // UI refresh shouldn't be too slow
        .max(Duration::from_millis(50)); // UI refresh shouldn't be too fast
    pb.enable_steady_tick(ui_refresh_interval);
    let result = future.await;
    pb.finish_with_message("Done");
    result
}

/// Show an error with suggestions for recovery
pub fn error_with_suggestions(message: &str, suggestions: &[&str]) {
    print_error(message);
    if !suggestions.is_empty() {
        println!("\n💡 Suggestions:");
        for suggestion in suggestions {
            println!("  • {}", suggestion.bright_yellow());
        }
    }
}

/// Show a warning with additional context
pub fn warning_with_context(message: &str, context: &str) {
    print_warning(message);
    println!("   {}", context.dimmed());
}

/// Show system information in a formatted way
pub fn system_info(info: &[(&str, &str)]) {
    println!("\n{}", "System Information".bright_blue().bold());
    println!("{}", "═".repeat(20).bright_blue());

    for (key, value) in info {
        println!("{:>15}: {}", key.bright_cyan(), value.bright_white());
    }
}

/// Show configuration summary
pub fn config_summary(config: &[(&str, &str)]) {
    println!("\n{}", "Configuration Summary".bright_blue().bold());
    println!("{}", "═".repeat(25).bright_blue());

    for (key, value) in config {
        println!("{:>20}: {}", key.bright_cyan(), value.bright_white());
    }
}

/// Clear screen for watch mode
pub fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
}

/// Show a step in a process
pub fn step(step_num: usize, total: usize, message: &str) {
    println!("{} {}", format!("[{step_num}/{total}]").bright_blue().bold(), message);
}

/// Show completion message with next steps
pub fn completion_message(message: &str, next_steps: &[&str]) {
    println!("\n{}", "🎉 Success!".bright_green().bold());
    println!("{}", message.bright_green());

    if !next_steps.is_empty() {
        println!("\n{}", "Next Steps:".bright_blue().bold());
        for (i, step) in next_steps.iter().enumerate() {
            println!("  {}. {}", i + 1, step.bright_white());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn success_info_warn_error_debug_progress_contain_message_text() {
        let msg = "network-timeout";
        assert!(success(msg).contains(msg));
        assert!(info(msg).contains(msg));
        assert!(warn(msg).contains(msg));
        assert!(error(msg).contains(msg));
        assert!(debug(msg).contains(msg));
        assert!(progress(msg).contains(msg));
    }

    #[test]
    fn format_bytes_scales_units() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1023), "1023 B");
        assert!(format_bytes(1024).contains("KB"));
        assert!(format_bytes(1024 * 1024).contains("MB"));
    }

    #[test]
    fn format_duration_human_readable() {
        assert_eq!(format_duration(Duration::from_secs(45)), "45s");
        assert_eq!(format_duration(Duration::from_secs(125)), "2m 5s");
        assert_eq!(format_duration(Duration::from_secs(3725)), "1h 2m");
        assert_eq!(format_duration(Duration::from_secs(90061)), "1d 1h 1m");
    }

    #[test]
    fn format_percentage_uses_threshold_colors() {
        let s_high = format_percentage(0.95);
        let s_mid = format_percentage(0.80);
        let s_low = format_percentage(0.50);
        assert!(s_high.contains("95.0%"));
        assert!(s_mid.contains("80.0%"));
        assert!(s_low.contains("50.0%"));
    }

    #[test]
    fn format_health_status_branches() {
        assert!(format_health_status("Healthy").contains("Healthy"));
        assert!(format_health_status("WARNING").contains("WARNING"));
        assert!(format_health_status("Failed").contains("Failed"));
        assert!(format_health_status("unknown").contains("unknown"));
    }

    #[test]
    fn title_and_terminal_dimensions_are_sensible() {
        let t = title("Songbird");
        assert!(t.to_string().contains("Songbird"));
        assert!(terminal_width() > 0);
        assert!(terminal_height() > 0);
    }

    #[test]
    fn table_builder_chains() {
        let _table =
            Table::new().headers(vec!["a".into(), "bb".into()]).row(vec!["ccc".into(), "d".into()]);
    }

    #[tokio::test]
    async fn with_loading_completes_future() {
        let out = with_loading("test-op", async { 7_u32 }).await;
        assert_eq!(out, 7);
    }

    #[test]
    fn error_with_suggestions_and_warning_with_context_run() {
        error_with_suggestions("main failure", &["retry", "check logs"]);
        warning_with_context("degraded", "queue depth high");
    }

    #[test]
    fn system_info_and_config_summary_run() {
        system_info(&[("os", "linux"), ("arch", "x86_64")]);
        config_summary(&[("region", "local")]);
    }

    #[test]
    fn step_and_completion_message_run() {
        step(2, 5, "deploy");
        completion_message("All set.", &["Run status", "Open dashboard"]);
    }
}
