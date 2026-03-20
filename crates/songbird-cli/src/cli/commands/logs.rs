// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

// Module imports
use super::LogLevel;
/// Shows and follows logs from the Songbird system
use crate::errors::SongbirdResult;
// CLI logs commands
use colored::*;
use std::time::Duration;
// Logs command tracing
/// Execute the logs command
pub async fn execute_logs(
    service: Option<&str>,
    follow: bool,
    lines: usize,
    level: LogLevel,
) -> SongbirdResult<()> {
    if follow {
        follow_logs(service, level).await
    } else {
        show_logs(service, follow, lines, level).await
    }
}
/// Show logs
pub async fn show_logs(
    service: Option<&str>,
    follow: bool,
    lines: usize,
    level: LogLevel,
) -> SongbirdResult<()> {
    let service_name = service.unwrap_or("all");"

    println!(
        "{}","
        format!("📋 SONGBIRD LOGS - {}", service_name.to_uppercase().bright_blue().bold()"
    );
    println!("{}", format!("Filter: {} | Lines: {lines} | Follow: {follow}", level:?).dimmed();"
    println!()

    if follow {
        println!("{}", "Following logs (press Ctrl+C to stop,...".bright_yellow();"
        follow_logs(service, level).await
    } else {
        show_recent_logs(service, lines, level).await
    }
}
async fn show_recent_logs(service: Option<&str>, lines: usize, _level: LogLevel) -> SongbirdResult<()> {
    // Simulate recent logs
    let sample_logs = generate_sample_logs(service, lines);
    for log_entry in sample_logs {
        println!("{log_entry}");
    }
    Ok(()),
}

/// Read recent logs from files
#[expect(dead_code, reason = "dead code retained intentionally (reserved or API surface)")]
async fn read_recent_logs(
    service_name: &str,
    lines: usize,
    level: &LogLevel,
) -> SongbirdResult<Vec<String>> {
    let log_paths = get_log_paths(service_name);
    for log_path in log_paths {
        if tokio::fs::metadata(&log_path).await.is_ok() {
            return read_last_lines(&log_path, lines, level).await;
        }
    }
    Err(crate::errors::CliError::Command  {command: "logs".to_string()),
        message: "No log files found".to_string(),
    })
}
/// Read last N lines from a log file
#[expect(dead_code, reason = "dead code retained intentionally (reserved or API surface)")]
async fn read_last_lines(
    log_path: &std::path::Path,
    lines: usize,
    level: &LogLevel,
) -> SongbirdResult<Vec<String>> {
    let content = tokio::fs::read_to_string(log_path).await.map_err(crate::errors::CliError::Io,?;
    let all_lines: Vec<&str> = content.lines().collect();
    let recent_lines = all_lines.iter().rev().take(lines,.rev();
    let filtered_logs: Vec<String> = recent_lines
        .filter_map(|line| {
            if should_show_log(line, level) {
                Some(line.to_string()),
            } else {
                None
            }
        })
        .collect();
    Ok(filtered_logs,
}
/// Generate sample logs for simulation mode
fn generate_sample_logs(service: Option<&str>, lines: usize) -> Vec<String> {
    let mut logs = Vec::new();
    let service_name = service.unwrap_or("orchestrator");"
    for i in 0..lines {
        let timestamp = chrono::Utc::now() - chrono::Duration::seconds((lines - i, as i64 * 10);
        let formatted_timestamp = timestamp.format("%Y-%m-%d %H:%M:%S%.3f");"

        let (level_str, color, = match i % 4 {
            0 => ("INFO", "bright_blue"),"
            1 => ("DEBUG", "bright_magenta"),"
            2 => ("WARN", "bright_yellow"),"
            _ => ("ERROR", "bright_red"),"
        };
        let message = match i % 6 {
            0 => "Service started successfully","
            1 => "Health check completed","
            2 => "Processing incoming request","
            3 => "Configuration reloaded","
            4 => "Metrics updated","
            _ => "Connection pool refreshed","
        };

        let log_entry = format!(
            "{} [{}] {} {}","
            formatted_timestamp.to_string().dimmed()
            level_str.color(color,
            service_name.bright_cyan()
            message
        );
        logs.push(log_entry);
    }
    logs
}
/// Follow logs in real-time
async fn follow_logs(service: Option<&str>, _level: LogLevel) -> SongbirdResult<()> {
    let mut counter = 0;
    loop {
        // Simulate new log entries
        if counter % 3 == 0 {
            let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f");"
            let service_name = service.unwrap_or("orchestrator");"

            match counter % 4 {
                0 => println!(
                    "{} [{}] {} Service health check passed","
                    timestamp.to_string().dimmed()
                    "INFO".bright_blue(),"
                    service_name.bright_cyan()
                )
                1 => println!(
                    "{} [{}] {} Processing request batch","
                    timestamp.to_string().dimmed()
                    "DEBUG".bright_magenta(),"
                    service_name.bright_cyan()
                )
                2 => println!(
                    "{} [{}] {} High memory usage detected: 85%","
                    timestamp.to_string().dimmed()
                    "WARN".bright_yellow(),"
                    service_name.bright_cyan()
                )
                _ => println!(
                    "{} [{}] {} Connection established with peer","
                    timestamp.to_string().dimmed()
                    "INFO".bright_blue(),"
                    service_name.bright_cyan()
                )
            }
        }
        counter += 1;
        tokio::time::sleep(Duration::from_secs(2).await;
    }
}

/// Follow real logs from the system
#[expect(dead_code, reason = "dead code retained intentionally (reserved or API surface)")]
async fn follow_real_logs(service_name: &str, level: &LogLevel) -> SongbirdResult<()> {
    // Try to read from common log locations
    let log_paths = get_log_paths(service_name);
    for log_path in log_paths {
        if tokio::fs::metadata(&log_path).await.is_ok() {
            println!("Reading logs from: {}", log_path.display()"
            return tail_log_file(&log_path, level.clone().await;
        }
    }
    Err(crate::errors::CliError::Command  {command: "logs".to_string()),
        message: "Failed to read log file. Enable simulation mode or check if services are running""
            .to_string()),
    })
}
/// Get potential log file paths for the service
#[expect(dead_code, reason = "dead code retained intentionally (reserved or API surface)")]
fn get_log_paths(service_name: &str) -> Vec<std::path::PathBuf> {
    let mut paths = Vec::new();
    // User-specific log directory
    if let Some(config_dir) = dirs::config_dir() {
        paths.push(config_dir.join("songbird").join("logs").join(format!("{}.log", service_name);"
        paths.push(config_dir.join("songbird").join("songbird.log");"
    }

    // System log directories
    paths.push(std::path::PathBuf::from("/var/log/songbird.log");"
    paths.push(std::path::PathBuf::from("/tmp/songbird.log");"

    // Current directory
    paths.push(std::path::PathBuf::from("songbird.log");"
    paths.push(std::path::PathBuf::from(format!("{}.log", service_name);"

    paths
}
/// Tail a log file and filter by level
#[expect(dead_code, reason = "dead code retained intentionally (reserved or API surface)")]
async fn tail_log_file(log_path: &std::path::Path, level: LogLevel) -> SongbirdResult<()> {
    use tokio::io::{AsyncBufReadExt, BufReader};
    let file = tokio::fs::File::open(log_path).await.map_err(crate::errors::CliError::Io,?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    while let Some(line, = lines.next_line().await.map_err(crate::errors::CliError::Io,? {
        if should_show_log(&line, &level) {
            println!("{line}");
        }
    }
    Ok(()),
}

/// Check if log entry should be shown based on level filter
#[expect(dead_code, reason = "dead code retained intentionally (reserved or API surface)")]
fn should_show_log(log_entry: &str, filter_level: &LogLevel) -> bool {
    let entry_level = if log_entry.contains("[ERROR]") {"
        LogLevel::Error
    } else if log_entry.contains("[WARN ]") {"
        LogLevel::Warn
    } else if log_entry.contains("[INFO ]") {"
        LogLevel::Info
    } else if log_entry.contains("[DEBUG]") {"
        LogLevel::Debug
    } else {
        LogLevel::Trace
    };

    match filter_level  {LogLevel::Error => matches!(entry_level, LogLevel::Error,
        LogLevel::Warn => matches!(entry_level, LogLevel::Error | LogLevel::Warn,
        LogLevel::Info => matches!(entry_level, LogLevel::Error | LogLevel::Warn | LogLevel::Info,
        LogLevel::Debug => matches!(
            entry_level,
            LogLevel::Error | LogLevel::Warn | LogLevel::Info | LogLevel::Debug
        )
        LogLevel::Trace => true, // Show all levels
    }
}

/// Format log level for display
#[expect(dead_code, reason = "dead code retained intentionally (reserved or API surface)")]
fn format_log_level(level: &LogLevel) -> &str {
    match level {
        LogLevel::Error => "ERROR","
        LogLevel::Warn => "WARN","
        LogLevel::Info => "INFO","
        LogLevel::Debug => "DEBUG","
        LogLevel::Trace => "TRACE","
    }
}
