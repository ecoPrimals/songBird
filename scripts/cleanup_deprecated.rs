#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2021"
//! [dependencies]
//! walkdir = "2"
//! regex = "1"
//! [workspace]
//! ```
//! # Songbird Deprecated Code Cleanup Script
//! 
//! This script systematically removes deprecated code patterns and cleans up
//! the codebase for the final modernization phase.

use regex::Regex;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧹 Starting Songbird deprecated code cleanup...");
    
    let patterns_to_remove = vec![
        // Remove deprecated struct definitions that are already replaced
        r#"#\[deprecated\(note = "Use.*instead"\)\]\s*pub struct \w+Config \{[^}]*\}"#,
        
        // Remove deprecated function definitions
        r#"#\[deprecated\(note = "Use.*instead"\)\]\s*pub fn \w+\([^}]*\} "#,
        
        // Remove REMOVED comments for already migrated code
        r#"// REMOVED: \w+Config deprecated struct - use.*instead"#,
        
        // Remove deprecated type aliases
        r#"#\[deprecated\(note = "Use.*instead"\)\]\s*pub type \w+ = .*;"#,
    ];
    
    let directories_to_clean = vec![
        "crates/songbird-config/src/config/",
        "crates/songbird-core/src/",
        "crates/songbird-network/src/",
        "crates/songbird-security/src/",
    ];
    
    for dir in directories_to_clean {
        if Path::new(dir).exists() {
            println!("🔍 Cleaning directory: {}", dir);
            clean_directory(dir, &patterns_to_remove)?;
        }
    }
    
    println!("✅ Deprecated code cleanup completed!");
    Ok(())
}

fn clean_directory(dir: &str, patterns: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            clean_file(path, patterns)?;
        }
    }
    Ok(())
}

fn clean_file(path: &Path, patterns: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let mut cleaned_content = content.clone();
    let mut changes_made = false;
    
    for pattern in patterns {
        let regex = Regex::new(pattern)?;
        if regex.is_match(&cleaned_content) {
            cleaned_content = regex.replace_all(&cleaned_content, "").to_string();
            changes_made = true;
        }
    }
    
    if changes_made {
        println!("  🧹 Cleaned: {}", path.display());
        fs::write(path, cleaned_content)?;
    }
    
    Ok(())
} 