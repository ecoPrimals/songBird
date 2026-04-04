// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

/// CLI testing helpers and utilities
///
/// Provides utilities for testing CLI components and formatting
use colored::{Color, Colorize};

/// CLI output utilities for testing and debugging
pub mod output {
    use super::{Color, Colorize};

    /// Print informational message (centralized)
    pub fn print_info(msg: &str) {
        println!("{}", msg.blue());
    }

    /// Print success message (centralized)
    pub fn print_success(msg: &str) {
        println!("{}", msg.green());
    }

    /// Print error message (centralized)
    pub fn print_error(msg: &str) {
        eprintln!("{}", msg.red());
    }

    /// Print warning message (centralized)
    pub fn print_warning(msg: &str) {
        println!("{}", msg.yellow());
    }

    /// Print debug message (only in debug builds)
    pub fn print_debug(msg: &str) {
        #[cfg(debug_assertions)]
        println!("{}", format!("DEBUG: {msg}").dimmed());
        #[cfg(not(debug_assertions))]
        let _ = msg; // Prevent unused variable warning in release builds
    }

    /// Print with custom color
    pub fn print_colored(msg: &str, color: Color) {
        println!("{}", msg.color(color));
    }
}

/// CLI testing utilities
pub mod testing {
    use std::sync::{Arc, Mutex};

    /// Capture CLI output for testing
    pub struct OutputCapture {
        captured_output: Arc<Mutex<Vec<String>>>,
    }

    impl OutputCapture {
        /// Create new output capture
        #[must_use]
        pub fn new() -> Self {
            Self {
                captured_output: Arc::new(Mutex::new(Vec::new())),
            }
        }

        /// Capture a message
        pub fn capture(&self, msg: &str) {
            self.captured_output
                .lock()
                .unwrap_or_else(|poisoned| {
                    tracing::warn!("Lock poisoned, recovering gracefully ");
                    poisoned.into_inner()
                })
                .push(msg.to_string());
        }

        /// Get all captured messages
        #[must_use]
        pub fn get_captured(&self) -> Vec<String> {
            self.captured_output
                .lock()
                .unwrap_or_else(|poisoned| {
                    tracing::warn!("Lock poisoned, recovering gracefully ");
                    poisoned.into_inner()
                })
                .clone()
        }

        /// Clear captured messages
        pub fn clear(&self) {
            self.captured_output
                .lock()
                .unwrap_or_else(|poisoned| {
                    tracing::warn!("Lock poisoned, recovering gracefully ");
                    poisoned.into_inner()
                })
                .clear();
        }
    }

    impl Default for OutputCapture {
        fn default() -> Self {
            Self::new()
        }
    }
}

/// Progress display utilities for long-running operations
pub mod progress {
    use std::io::Write;

    /// Simple progress indicator
    pub struct ProgressIndicator {
        current: usize,
        total: usize,
        prefix: String,
    }

    impl ProgressIndicator {
        /// Create new progress indicator
        #[must_use]
        pub fn new(total: usize, prefix: &str) -> Self {
            Self {
                current: 0,
                total,
                prefix: prefix.to_string(),
            }
        }

        /// Update progress
        pub fn update(&mut self, current: usize) {
            self.current = current;
            let percentage = if self.total > 0 {
                (self.current * 100) / self.total
            } else {
                0
            };

            print!("\r{} {}/{} ({}%) ", self.prefix, self.current, self.total, percentage);
            if let Err(e) = std::io::stdout().flush() {
                tracing::error!("Failed to flush stdout: {:?}", e);
            }
        }

        /// Complete progress
        pub fn complete(&self) {
            println!(" {} Complete! {}/{} ", self.prefix, self.total, self.total);
        }
    }
}
