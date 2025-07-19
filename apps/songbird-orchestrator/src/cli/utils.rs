//! CLI utility functions
//!
//! This module provides utility functions for CLI output formatting and common operations.

use colored::*;

/// Print informational message
pub fn print_info(msg: &str) {
    println!("{}", msg.blue());
}

/// Print success message  
pub fn print_success(msg: &str) {
    println!("{}", msg.green());
}

/// Print error message
pub fn print_error(msg: &str) {
    eprintln!("{}", msg.red());
}

/// Print warning message
pub fn print_warning(msg: &str) {
    println!("{}", msg.yellow());
}
