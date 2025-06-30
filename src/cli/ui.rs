//! CLI UI utilities for beautiful terminal output

use crate::cli::CliError;
use colored::{ColoredString, Colorize};
use console::Term;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::fmt::Display;
use std::time::Duration;

/// Create a styled progress bar
pub fn progress_bar(len: u64) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} {msg}",
            )
            .unwrap_or_else(|_| ProgressStyle::default_bar())
            .progress_chars("#>-"),
    );
    pb
}

/// Create a spinner for indefinite progress
pub fn spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner()),
    );
    pb.set_message(message.to_string());
    pb
}

/// Show a confirmation prompt
pub fn confirm(message: &str, default: bool) -> Result<bool, CliError> {
    let theme = ColorfulTheme::default();
    Confirm::with_theme(&theme)
        .with_prompt(message)
        .default(default)
        .interact()
        .map_err(|_| CliError::UserCancelled)
}

/// Show a selection prompt
pub fn select<T: Display>(
    message: &str,
    items: &[T],
    default: Option<usize>,
) -> Result<usize, CliError> {
    let theme = ColorfulTheme::default();
    let mut select = Select::with_theme(&theme).with_prompt(message).items(items);

    if let Some(default_idx) = default {
        select = select.default(default_idx);
    }
    select.interact().map_err(|_| CliError::UserCancelled)
}

/// Show a text input prompt
pub fn input_text(message: &str, default: Option<&str>) -> Result<String, CliError> {
    let theme = ColorfulTheme::default();
    let mut input = Input::with_theme(&theme).with_prompt(message);
    if let Some(default_value) = default {
        input = input.default(default_value.to_string());
    }
    input.interact_text().map_err(|_| CliError::UserCancelled)
}

/// Show a password input prompt
pub fn input_password(message: &str) -> Result<String, CliError> {
    let theme = ColorfulTheme::default();
    dialoguer::Password::with_theme(&theme)
        .with_prompt(message)
        .interact()
        .map_err(|_| CliError::UserCancelled)
}

/// Print colored success message
pub fn success(message: &str) -> String {
    message.green().bold().to_string()
}

/// Print colored info message
pub fn info(message: &str) -> String {
    message.blue().to_string()
}

/// Print colored warning message
pub fn warn(message: &str) -> String {
    message.yellow().to_string()
}

/// Print colored error message
pub fn error(message: &str) -> String {
    message.red().bold().to_string()
}

/// Print colored success message to stdout
pub fn print_success(message: &str) {
    println!("{}", success(message));
}

/// Print colored info message to stdout
pub fn print_info(message: &str) {
    println!("{}", info(message));
}

/// Print colored warning message to stdout
pub fn print_warning(message: &str) {
    println!("{}", warn(message));
}

/// Print colored error message to stdout
pub fn print_error(message: &str) {
    eprintln!("{}", error(message));
}

/// Print progress bar or spinner
pub fn progress(message: &str) -> String {
    format!("⏳ {}", message.cyan())
}

/// Print a header with formatting
pub fn header(title: &str) {
    println!("\n{}", title.bright_blue().bold());
    println!("{}", "=".repeat(title.len()).bright_blue());
}

/// Print a subheader
pub fn subheader(title: &str) {
    println!("\n{}", title.bright_cyan().bold());
    println!("{}", "-".repeat(title.len()).bright_cyan());
}

/// Format bytes in human-readable format
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Format duration in human-readable format
pub fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let days = total_seconds / 86400;
    let hours = (total_seconds % 86400) / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

/// Format percentage
pub fn format_percentage(value: f64) -> String {
    format!("{:.1}%", value * 100.0)
}

/// CLI UI helper functions for beautiful output
pub fn title(message: &str) -> ColoredString {
    message.bold().bright_blue()
}

/// Get terminal width safely
pub fn terminal_width() -> usize {
    term_size::dimensions().map(|(w, _)| w).unwrap_or(80) // Safe fallback to 80 columns
}

/// Get terminal height safely
pub fn terminal_height() -> usize {
    term_size::dimensions().map(|(_, h)| h).unwrap_or(24) // Safe fallback to 24 rows
}

/// Create a table-like output
pub struct Table {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    widths: Vec<usize>,
}

impl Table {
    pub fn new(headers: Vec<String>) -> Self {
        let widths = headers.iter().map(|h| h.len()).collect();
        Self {
            headers,
            rows: Vec::new(),
            widths,
        }
    }

    pub fn add_row(&mut self, row: Vec<String>) {
        // Update column widths
        for (i, cell) in row.iter().enumerate() {
            if i < self.widths.len() {
                self.widths[i] = self.widths[i].max(cell.len());
            }
        }
        self.rows.push(row);
    }

    pub fn print(&self) {
        // Print headers
        self.print_separator();
        print!("│");
        for (i, header) in self.headers.iter().enumerate() {
            print!(
                " {:width$} │",
                header.bright_blue().bold(),
                width = self.widths[i]
            );
        }
        println!();

        // Print rows
        for row in &self.rows {
            print!("│");
            for (i, cell) in row.iter().enumerate() {
                let width = if i < self.widths.len() {
                    self.widths[i]
                } else {
                    0
                };
                print!(" {:width$} │", cell, width = width);
            }
            println!();
        }
    }

    fn print_separator(&self) {
        print!("├");
        for width in &self.widths {
            print!("{}", "─".repeat(width + 2));
            print!("┼");
        }
        // Replace last ┼ with ┤
        print!("\x08┤");
    }
}

/// Spinner with custom messages
pub struct Spinner {
    pb: ProgressBar,
}

impl Spinner {
    pub fn new(message: &str) -> Self {
        let pb = spinner(message);
        pb.enable_steady_tick(Duration::from_millis(120));
        Self { pb }
    }

    pub fn set_message(&self, message: &str) {
        self.pb.set_message(message.to_string());
    }

    pub fn finish_with_message(&self, message: &str) {
        self.pb.finish_with_message(message.to_string());
    }

    pub fn finish(&self) {
        self.pb.finish_and_clear();
    }
}

/// Multi-progress manager for concurrent operations
pub struct MultiProgressManager {
    multi: MultiProgress,
}

impl Default for MultiProgressManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiProgressManager {
    pub fn new() -> Self {
        Self {
            multi: MultiProgress::new(),
        }
    }

    pub fn add_progress_bar(&self, len: u64) -> ProgressBar {
        let pb = progress_bar(len);
        self.multi.add(pb)
    }

    pub fn add_spinner(&self, message: &str) -> ProgressBar {
        spinner(message)
    }
}

/// Clear the terminal
pub fn clear_screen() {
    let term = Term::stdout();
    let _ = term.clear_screen();
}

/// Move cursor to specific position
pub fn move_cursor(row: u16, col: u16) {
    let term = Term::stdout();
    let _ = term.move_cursor_to(col as usize, row as usize);
}

/// Get terminal size
pub fn terminal_size() -> (u16, u16) {
    term_size::dimensions()
        .map(|(w, h)| (w as u16, h as u16))
        .unwrap_or((80, 24))
}
