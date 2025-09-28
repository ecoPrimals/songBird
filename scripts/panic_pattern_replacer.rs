#!/usr/bin/env rust-script

//! # Panic Pattern Replacer
//! 
//! This script systematically replaces panic-prone patterns with proper error handling
//! throughout the Songbird codebase.

use std::fs;
use std::path::Path;
use regex::Regex;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Starting panic pattern replacement...");
    
    let patterns = vec![
        // .unwrap() patterns
        PanicPattern {
            pattern: Regex::new(r"\.unwrap\(\)")?,
            replacement: ".map_err(|e| SongbirdError::internal(format!(\"Operation failed: {:?}\", e)))?",
            description: "Replace .unwrap() with proper error handling",
        },
        
        // .expect() patterns with generic messages
        PanicPattern {
            pattern: Regex::new(r"\.expect\(\"([^\"]+)\"\)")?,
            replacement: ".map_err(|e| SongbirdError::internal(format!(\"$1: {:?}\", e)))?",
            description: "Replace .expect() with proper error handling",
        },
        
        // panic! patterns
        PanicPattern {
            pattern: Regex::new(r"panic!\(\"([^\"]+)\"\)")?,
            replacement: "return Err(SongbirdError::internal(\"$1\"))",
            description: "Replace panic! with proper error return",
        },
        
        // unwrap_or_else patterns that might panic
        PanicPattern {
            pattern: Regex::new(r"\.unwrap_or_else\(\|\| panic!\(\"([^\"]+)\"\)\)")?,
            replacement: ".map_err(|e| SongbirdError::internal(format!(\"$1: {:?}\", e)))?",
            description: "Replace unwrap_or_else panic with proper error handling",
        },
    ];
    
    let target_dirs = vec![
        "crates/songbird-types/src",
        "crates/songbird-network/src", 
        "crates/songbird-universal/src",
        "crates/songbird-core/src",
        "crates/songbird-config/src",
    ];
    
    for dir in target_dirs {
        if Path::new(dir).exists() {
            process_directory(dir, &patterns)?;
        } else {
            println!("⚠️  Directory not found: {}", dir);
        }
    }
    
    println!("✅ Panic pattern replacement complete!");
    Ok(())
}

struct PanicPattern {
    pattern: Regex,
    replacement: &'static str,
    description: &'static str,
}

fn process_directory(dir: &str, patterns: &[PanicPattern]) -> Result<(), Box<dyn std::error::Error>> {
    println!("📁 Processing directory: {}", dir);
    
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            // Recursively process subdirectories
            if let Some(path_str) = path.to_str() {
                process_directory(path_str, patterns)?;
            }
        } else if path.extension().map_or(false, |ext| ext == "rs") {
            // Process Rust files
            if let Some(path_str) = path.to_str() {
                process_file(path_str, patterns)?;
            }
        }
    }
    
    Ok(())
}

fn process_file(file_path: &str, patterns: &[PanicPattern]) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path)?;
    let mut modified_content = content.clone();
    let mut changes_made = false;
    
    for pattern in patterns {
        if pattern.pattern.is_match(&modified_content) {
            println!("🔧 Applying pattern '{}' to: {}", pattern.description, file_path);
            modified_content = pattern.pattern.replace_all(&modified_content, pattern.replacement).to_string();
            changes_made = true;
        }
    }
    
    if changes_made {
        // Add necessary imports if not present
        if !modified_content.contains("use songbird_errors::SongbirdError;") && 
           !modified_content.contains("use songbird_types::SongbirdError;") {
            // Find the appropriate place to add the import
            if let Some(use_pos) = modified_content.find("use ") {
                let import = "use songbird_errors::SongbirdError;\n";
                modified_content.insert_str(use_pos, import);
            }
        }
        
        fs::write(file_path, modified_content)?;
        println!("✅ Updated: {}", file_path);
    }
    
    Ok(())
}

// Test patterns (for validation)
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_unwrap_replacement() {
        let pattern = Regex::new(r"\.unwrap\(\)").unwrap();
        let input = "let result = some_operation().unwrap();";
        let expected = "let result = some_operation().map_err(|e| SongbirdError::internal(format!(\"Operation failed: {:?}\", e)))?;";
        let output = pattern.replace(input, ".map_err(|e| SongbirdError::internal(format!(\"Operation failed: {:?}\", e)))?");
        assert_eq!(output, expected);
    }
    
    #[test]
    fn test_expect_replacement() {
        let pattern = Regex::new(r"\.expect\(\"([^\"]+)\"\)").unwrap();
        let input = "let result = some_operation().expect(\"Should not fail\");";
        let expected = "let result = some_operation().map_err(|e| SongbirdError::internal(format!(\"Should not fail: {:?}\", e)))?;";
        let output = pattern.replace(input, ".map_err(|e| SongbirdError::internal(format!(\"$1: {:?}\", e)))?");
        assert_eq!(output, expected);
    }
    
    #[test]
    fn test_panic_replacement() {
        let pattern = Regex::new(r"panic!\(\"([^\"]+)\"\)").unwrap();
        let input = "panic!(\"This should not happen\");";
        let expected = "return Err(SongbirdError::internal(\"This should not happen\"));";
        let output = pattern.replace(input, "return Err(SongbirdError::internal(\"$1\"))");
        assert_eq!(output, expected);
    }
} 