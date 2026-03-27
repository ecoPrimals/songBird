// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! `UniBin` Chaos Tests
//!
//! Unpredictable scenario testing including:
//! - Random input generation
//! - Concurrent operations
//! - Resource contention
//! - Rapid state changes
//! - Stress conditions
//!
//! **Evolution**: Removed all #[serial] - Commands are process-isolated!
//! Modern, idiomatic, async Rust with zero global state.

// `cargo_bin!` only applies to binaries in *this* package; the `songbird` executable is built
// by the workspace root crate. The deprecated `cargo_bin` function resolves the path at runtime.
#![allow(deprecated)]

use assert_cmd::Command;
use rand::Rng;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::time::{Duration, sleep};

// ====================
// TEST HELPERS
// ====================

// ✅ NO clear_chaos_env() needed!
// `cargo_bin!` resolves the test binary path for isolated child processes.
// Each process has its own environment - no global state pollution!

// ====================
// CHAOS TESTS
// ====================

#[tokio::test]
// ✅ NO #[serial]! Commands are process-isolated!
async fn test_chaos_rapid_fire_commands() -> Result<(), Box<dyn std::error::Error>> {
    // Fire 100 commands as fast as possible
    for _ in 0..100 {
        let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("songbird"));
        cmd.arg("--version").assert().success();
    }

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Commands are process-isolated!
async fn test_chaos_random_subcommands() -> Result<(), Box<dyn std::error::Error>> {
    let subcommands = ["--version", "--help"];
    let mut rng = rand::thread_rng();

    // Execute random subcommands
    for _ in 0..50 {
        let cmd_choice = subcommands[rng.gen_range(0..subcommands.len())];

        let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("songbird"));
        cmd.arg(cmd_choice).assert().success();
    }

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Commands are process-isolated!
async fn test_chaos_concurrent_version_checks() -> Result<(), Box<dyn std::error::Error>> {
    // Spawn 20 concurrent version checks
    let mut handles = vec![];

    for _ in 0..20 {
        let handle = tokio::spawn(async {
            let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("songbird"));
            cmd.arg("--version").assert().success();
        });
        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        handle.await?;
    }

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Commands are process-isolated!
async fn test_chaos_concurrent_help_requests() -> Result<(), Box<dyn std::error::Error>> {
    // Spam help requests concurrently
    let mut handles = vec![];

    for _ in 0..15 {
        let handle = tokio::spawn(async {
            let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("songbird"));
            cmd.arg("--help").assert().success();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await?;
    }

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Commands are process-isolated!
async fn test_chaos_mixed_concurrent_commands() -> Result<(), Box<dyn std::error::Error>> {
    let mut handles = vec![];
    let mut rng = rand::thread_rng();

    // Mix of different commands running concurrently
    for _ in 0..30 {
        let cmd_type = rng.gen_range(0..3);

        let handle = tokio::spawn(async move {
            match cmd_type {
                0 => {
                    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("songbird"));
                    cmd.arg("--version").assert().success();
                }
                1 => {
                    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("songbird"));
                    cmd.arg("--help").assert().success();
                }
                _ => {
                    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("songbird"));
                    cmd.arg("doctor").assert().success();
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await?;
    }

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Commands are process-isolated!
async fn test_chaos_rapid_doctor_checks() -> Result<(), Box<dyn std::error::Error>> {
    // Rapid fire doctor checks
    for _ in 0..30 {
        let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("songbird"));
        cmd.arg("doctor").assert().success();
    }

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Commands are process-isolated!
async fn test_chaos_interleaved_commands() -> Result<(), Box<dyn std::error::Error>> {
    // Interleave different command types
    for i in 0..20 {
        match i % 3 {
            0 => {
                Command::new(assert_cmd::cargo::cargo_bin("songbird"))
                    .arg("--version")
                    .assert()
                    .success();
            }
            1 => {
                Command::new(assert_cmd::cargo::cargo_bin("songbird"))
                    .arg("doctor")
                    .assert()
                    .success();
            }
            _ => {
                Command::new(assert_cmd::cargo::cargo_bin("songbird"))
                    .arg("--help")
                    .assert()
                    .success();
            }
        }
    }

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Commands are process-isolated! (sleeps are for chaos timing)
async fn test_chaos_random_delays() -> Result<(), Box<dyn std::error::Error>> {
    let mut rng = rand::thread_rng();

    // Execute commands with random delays
    for _ in 0..10 {
        let delay_ms = rng.gen_range(1..50);
        sleep(Duration::from_millis(delay_ms)).await;

        Command::new(assert_cmd::cargo::cargo_bin("songbird")).arg("--version").assert().success();
    }

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Commands are process-isolated! (sleeps are for chaos timing)
async fn test_chaos_burst_pattern() -> Result<(), Box<dyn std::error::Error>> {
    // Burst pattern: rapid commands, then pause, repeat
    for _ in 0..5 {
        // Burst
        for _ in 0..10 {
            Command::new(assert_cmd::cargo::cargo_bin("songbird"))
                .arg("--version")
                .assert()
                .success();
        }

        // Pause
        sleep(Duration::from_millis(50)).await;
    }

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Commands are process-isolated!
async fn test_chaos_concurrent_doctor_formats() -> Result<(), Box<dyn std::error::Error>> {
    let formats = vec!["text", "json", "yaml"];
    let mut handles = vec![];

    // Run doctor with different formats concurrently
    for format in formats {
        let format_owned = format.to_string();
        let handle = tokio::spawn(async move {
            let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("songbird"));
            cmd.arg("doctor").arg("--format").arg(&format_owned).assert().success();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await?;
    }

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! songbird_process_env::set_var only affects child processes via Command!
async fn test_chaos_random_env_vars() -> Result<(), Box<dyn std::error::Error>> {
    let mut rng = rand::thread_rng();

    // Set random environment variables and run commands
    for _ in 0..10 {
        let random_port = rng.gen_range(8000..9000);
        songbird_process_env::set_var("SONGBIRD_PORT", random_port.to_string());

        Command::new(assert_cmd::cargo::cargo_bin("songbird")).arg("--version").assert().success();

        songbird_process_env::remove_var("SONGBIRD_PORT");
    }

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Commands are process-isolated!
async fn test_chaos_stress_help_system() -> Result<(), Box<dyn std::error::Error>> {
    let subcommands = vec!["server", "doctor", "config"];

    // Stress the help system by requesting help for all subcommands rapidly
    for _ in 0..20 {
        for subcmd in &subcommands {
            Command::new(assert_cmd::cargo::cargo_bin("songbird"))
                .arg(subcmd)
                .arg("--help")
                .assert()
                .success();
        }
    }

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Commands are process-isolated! (Arc<AtomicUsize> is thread-safe)
async fn test_chaos_concurrent_with_counter() -> Result<(), Box<dyn std::error::Error>> {
    let counter = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    // Track successful concurrent executions
    for _ in 0..25 {
        let counter_clone = Arc::clone(&counter);
        let handle = tokio::spawn(async move {
            let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("songbird"));
            cmd.arg("--version").assert().success();
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await?;
    }

    // Verify all succeeded
    assert_eq!(counter.load(Ordering::SeqCst), 25);

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Commands are process-isolated!
async fn test_chaos_alternating_success_patterns() -> Result<(), Box<dyn std::error::Error>> {
    // Alternate between different successful commands
    for i in 0..40 {
        if i % 2 == 0 {
            Command::new(assert_cmd::cargo::cargo_bin("songbird"))
                .arg("--version")
                .assert()
                .success();
        } else {
            Command::new(assert_cmd::cargo::cargo_bin("songbird")).arg("doctor").assert().success();
        }
    }

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Commands are process-isolated! (sleeps are for chaos timing)
async fn test_chaos_wave_pattern() -> Result<(), Box<dyn std::error::Error>> {
    // Wave pattern: increasing then decreasing load
    let wave = vec![1, 3, 5, 7, 5, 3, 1];

    for &count in &wave {
        let mut handles = vec![];

        for _ in 0..count {
            let handle = tokio::spawn(async {
                let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("songbird"));
                cmd.arg("--version").assert().success();
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await?;
        }

        sleep(Duration::from_millis(10)).await;
    }

    Ok(())
}
