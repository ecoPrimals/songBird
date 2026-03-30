// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Log viewing and following for Songbird CLI.

use super::LogLevel;
use crate::errors::SongbirdResult;
use colored::Colorize;
use std::time::Duration;

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
    let service_name = service.unwrap_or("all");

    println!(
        "{}",
        format!(
            "📋 SONGBIRD LOGS - {}",
            service_name.to_uppercase()
        )
        .bright_blue()
        .bold()
    );
    println!(
        "{}",
        format!("Filter: {level:?} | Lines: {lines} | Follow: {follow}").dimmed()
    );
    println!();

    if follow {
        println!(
            "{}",
            "Following logs (press Ctrl+C to stop)...".bright_yellow()
        );
        follow_logs(service, level).await
    } else {
        show_recent_logs(service, lines, level).await
    }
}

async fn show_recent_logs(
    service: Option<&str>,
    lines: usize,
    level: LogLevel,
) -> SongbirdResult<()> {
    let service_name = service.unwrap_or("all");
    match read_recent_logs(service_name, lines, &level).await {
        Ok(entries) => {
            for log_entry in entries {
                println!("{log_entry}");
            }
        }
        Err(_) => {
            println!(
                "{}",
                "No log files discovered; showing synthetic preview.".dimmed()
            );
            let sample_logs = generate_sample_logs(service, lines);
            for log_entry in sample_logs {
                println!("{log_entry}");
            }
        }
    }
    Ok(())
}

/// Read recent logs from discovered log files.
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
    Err(crate::errors::CliError::Command {
        command: "logs".to_string(),
        message: "No log files found".to_string(),
    })
}

/// Read last N lines from a log file, filtering by level.
async fn read_last_lines(
    log_path: &std::path::Path,
    lines: usize,
    level: &LogLevel,
) -> SongbirdResult<Vec<String>> {
    let content = tokio::fs::read_to_string(log_path)
        .await
        .map_err(crate::errors::CliError::Io)?;
    let all_lines: Vec<&str> = content.lines().collect();
    let recent_lines = all_lines.iter().rev().take(lines).rev();
    let filtered_logs: Vec<String> = recent_lines
        .filter_map(|line| {
            if should_show_log(line, level) {
                Some((*line).to_string())
            } else {
                None
            }
        })
        .collect();
    Ok(filtered_logs)
}

/// Generate sample logs for simulation mode
fn generate_sample_logs(service: Option<&str>, lines: usize) -> Vec<String> {
    let mut logs = Vec::new();
    let service_name = service.unwrap_or("orchestrator");
    for i in 0..lines {
        let timestamp =
            chrono::Utc::now() - chrono::Duration::seconds((lines - i) as i64 * 10);
        let formatted_timestamp = timestamp.format("%Y-%m-%d %H:%M:%S%.3f");

        let (level_str, color) = match i % 4 {
            0 => ("INFO", "bright_blue"),
            1 => ("DEBUG", "bright_magenta"),
            2 => ("WARN", "bright_yellow"),
            _ => ("ERROR", "bright_red"),
        };
        let message = match i % 6 {
            0 => "Service started successfully",
            1 => "Health check completed",
            2 => "Processing incoming request",
            3 => "Configuration reloaded",
            4 => "Metrics updated",
            _ => "Connection pool refreshed",
        };

        let log_entry = format!(
            "{} [{}] {} {message}",
            formatted_timestamp.to_string().dimmed(),
            level_str.color(color),
            service_name.bright_cyan(),
        );
        logs.push(log_entry);
    }
    logs
}

/// Follow logs in real-time.
///
/// Attempts real log tailing first; falls back to synthetic preview when no log
/// files are discoverable yet (services not started, first boot, etc.).
async fn follow_logs(service: Option<&str>, level: LogLevel) -> SongbirdResult<()> {
    let service_name = service.unwrap_or("orchestrator");
    match follow_real_logs(service_name, &level).await {
        Ok(()) => Ok(()),
        Err(_) => {
            println!(
                "{}",
                "No log files discovered; showing synthetic preview (Ctrl+C to stop)."
                    .bright_yellow()
            );
            follow_synthetic(service).await
        }
    }
}

/// Follow real logs from the system by tailing the first discoverable log file.
async fn follow_real_logs(service_name: &str, level: &LogLevel) -> SongbirdResult<()> {
    let log_paths = get_log_paths(service_name);
    for log_path in log_paths {
        if tokio::fs::metadata(&log_path).await.is_ok() {
            println!("{}", format!("Tailing: {}", log_path.display()).dimmed());
            return tail_log_file(&log_path, level.clone()).await;
        }
    }
    Err(crate::errors::CliError::Command {
        command: "logs".to_string(),
        message: "No log files found".to_string(),
    })
}

/// Synthetic log stream used when no real log files are available.
async fn follow_synthetic(service: Option<&str>) -> SongbirdResult<()> {
    let service_name = service.unwrap_or("orchestrator");
    loop {
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f");
        println!(
            "{} [{}] {} Awaiting log infrastructure...",
            timestamp.to_string().dimmed(),
            "INFO".bright_blue(),
            service_name.bright_cyan(),
        );
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

/// Get potential log file paths for the service.
///
/// Discovery order: XDG state dir -> XDG config dir -> platform temp dir -> cwd
fn get_log_paths(service_name: &str) -> Vec<std::path::PathBuf> {
    let mut paths = Vec::new();

    if let Some(state_dir) = dirs::state_dir() {
        paths.push(
            state_dir
                .join("songbird")
                .join("logs")
                .join(format!("{service_name}.log")),
        );
        paths.push(state_dir.join("songbird").join("songbird.log"));
    }

    if let Some(config_dir) = dirs::config_dir() {
        paths.push(
            config_dir
                .join("songbird")
                .join("logs")
                .join(format!("{service_name}.log")),
        );
        paths.push(config_dir.join("songbird").join("songbird.log"));
    }

    let temp = std::env::temp_dir();
    paths.push(temp.join("songbird.log"));

    paths.push(std::path::PathBuf::from("songbird.log"));
    paths.push(std::path::PathBuf::from(format!("{service_name}.log")));

    paths
}

/// Tail a log file and filter by level.
async fn tail_log_file(log_path: &std::path::Path, level: LogLevel) -> SongbirdResult<()> {
    use tokio::io::{AsyncBufReadExt, BufReader};
    let file = tokio::fs::File::open(log_path)
        .await
        .map_err(crate::errors::CliError::Io)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await.map_err(crate::errors::CliError::Io)? {
        if should_show_log(&line, &level) {
            println!("{line}");
        }
    }
    Ok(())
}

/// Check if log entry should be shown based on level filter.
fn should_show_log(log_entry: &str, filter_level: &LogLevel) -> bool {
    let entry_level = if log_entry.contains("[ERROR]") {
        LogLevel::Error
    } else if log_entry.contains("[WARN") {
        LogLevel::Warn
    } else if log_entry.contains("[INFO") {
        LogLevel::Info
    } else if log_entry.contains("[DEBUG]") {
        LogLevel::Debug
    } else {
        LogLevel::Trace
    };

    match filter_level {
        LogLevel::Error => matches!(entry_level, LogLevel::Error),
        LogLevel::Warn => matches!(entry_level, LogLevel::Error | LogLevel::Warn),
        LogLevel::Info => {
            matches!(entry_level, LogLevel::Error | LogLevel::Warn | LogLevel::Info)
        }
        LogLevel::Debug => matches!(
            entry_level,
            LogLevel::Error | LogLevel::Warn | LogLevel::Info | LogLevel::Debug
        ),
        LogLevel::Trace => true,
    }
}

/// Format log level for display.
fn format_log_level(level: &LogLevel) -> &str {
    match level {
        LogLevel::Error => "ERROR",
        LogLevel::Warn => "WARN",
        LogLevel::Info => "INFO",
        LogLevel::Debug => "DEBUG",
        LogLevel::Trace => "TRACE",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_show_log_error_filter() {
        assert!(should_show_log("[ERROR] test", &LogLevel::Error));
        assert!(!should_show_log("[INFO] test", &LogLevel::Error));
    }

    #[test]
    fn should_show_log_info_filter() {
        assert!(should_show_log("[ERROR] test", &LogLevel::Info));
        assert!(should_show_log("[WARN] test", &LogLevel::Info));
        assert!(should_show_log("[INFO] test", &LogLevel::Info));
        assert!(!should_show_log("[DEBUG] test", &LogLevel::Info));
    }

    #[test]
    fn should_show_log_trace_shows_all() {
        assert!(should_show_log("[ERROR] test", &LogLevel::Trace));
        assert!(should_show_log("[DEBUG] test", &LogLevel::Trace));
        assert!(should_show_log("unknown format", &LogLevel::Trace));
    }

    #[test]
    fn format_log_level_all_variants() {
        assert_eq!(format_log_level(&LogLevel::Error), "ERROR");
        assert_eq!(format_log_level(&LogLevel::Warn), "WARN");
        assert_eq!(format_log_level(&LogLevel::Info), "INFO");
        assert_eq!(format_log_level(&LogLevel::Debug), "DEBUG");
        assert_eq!(format_log_level(&LogLevel::Trace), "TRACE");
    }

    #[test]
    fn get_log_paths_returns_multiple() {
        let paths = get_log_paths("orchestrator");
        assert!(!paths.is_empty());
        let last = paths.last().unwrap();
        assert_eq!(last, &std::path::PathBuf::from("orchestrator.log"));
    }

    #[test]
    fn get_log_paths_no_hardcoded_system_paths() {
        let paths = get_log_paths("test");
        for path in &paths {
            let s = path.to_string_lossy();
            assert!(
                !s.starts_with("/var/log"),
                "should not hardcode /var/log: {s}"
            );
        }
    }

    #[test]
    fn generate_sample_logs_correct_count() {
        let logs = generate_sample_logs(None, 5);
        assert_eq!(logs.len(), 5);
    }

    #[test]
    fn generate_sample_logs_with_service() {
        let logs = generate_sample_logs(Some("stun"), 3);
        assert_eq!(logs.len(), 3);
    }
}
