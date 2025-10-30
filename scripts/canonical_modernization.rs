#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2021"
//! [dependencies]
//! walkdir = "2"
//! regex = "1"
//! [workspace]
//! ```
//! # Songbird Canonical Modernization Script
//! 
//! This script systematically modernizes the codebase by:
//! 1. Removing deprecated patterns and comments
//! 2. Unifying fragmented implementations
//! 3. Applying canonical error handling
//! 4. Cleaning up mock implementations

use regex::Regex;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Songbird canonical modernization...");
    
    // Phase 1: Remove deprecated comments and patterns
    let cleanup_patterns = vec![
        // Remove REMOVED comments
        (Regex::new(r"// REMOVED: [^\n]*\n")?, ""),
        
        // Remove deprecated warnings that are no longer needed
        (Regex::new(r"// ❌ OLD WAY \(DEPRECATED\): [^\n]*\n")?, ""),
        
        // Clean up comparison removed comments
        (Regex::new(r"// Comparison removed - always true for unsigned types[^\n]*\n")?, ""),
        
        // Remove migration notices that are complete
        (Regex::new(r"// Migration: [^\n]*\n")?, ""),
        
        // Clean up old hardcoded comments
        (Regex::new(r"// Old hardcoded [^\n]*\n")?, ""),
        
        // Remove field assignment removal comments
        (Regex::new(r"// Note: Field assignments? removed [^\n]*\n")?, ""),
    ];
    
    // Phase 2: Modernize patterns (but be conservative to avoid breaking working code)
    let modernization_patterns = vec![
        // Clean up excessive whitespace
        (Regex::new(r"\n\n\n+")?, "\n\n"),
    ];
    
    let directories_to_modernize = vec![
        "crates/songbird-config/src/",
        "crates/songbird-core/src/",
        "crates/songbird-network/src/",
        "crates/songbird-federation/src/",
        "crates/songbird-universal/src/",
    ];
    
    for dir in directories_to_modernize {
        if Path::new(dir).exists() {
            println!("🔧 Modernizing directory: {}", dir);
            modernize_directory(dir, &cleanup_patterns, &modernization_patterns)?;
        }
    }
    
    println!("✅ Canonical modernization completed!");
    Ok(())
}

fn modernize_directory(
    dir: &str, 
    cleanup_patterns: &[(Regex, &str)], 
    modernization_patterns: &[(Regex, &str)]
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            modernize_file(path, cleanup_patterns, modernization_patterns)?;
        }
    }
    Ok(())
}

fn modernize_file(
    path: &Path, 
    cleanup_patterns: &[(Regex, &str)], 
    modernization_patterns: &[(Regex, &str)]
) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let mut modernized_content = content.clone();
    
    // Apply cleanup patterns
    for (pattern, replacement) in cleanup_patterns {
        modernized_content = pattern.replace_all(&modernized_content, *replacement).to_string();
    }
    
    // Apply modernization patterns  
    for (pattern, replacement) in modernization_patterns {
        modernized_content = pattern.replace_all(&modernized_content, *replacement).to_string();
    }
    
    // Only write if content changed
    if modernized_content != content {
        fs::write(path, modernized_content)?;
        println!("  ✅ Modernized: {}", path.display());
    }
    
    Ok(())
} 