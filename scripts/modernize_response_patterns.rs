#!/usr/bin/env rust-script

//! Modernize Response Patterns to Canonical Types
//!
//! This script fixes the inconsistent response patterns throughout the codebase
//! to use the canonical types properly.

use std::fs;
use std::path::Path;
use walkdir::WalkDir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Modernizing response patterns to canonical types...");
    
    let crates_dir = Path::new("crates");
    
    for entry in WalkDir::new(crates_dir) {
        let entry = entry?;
        if entry.file_type().is_file() && entry.path().extension() == Some("rs".as_ref()) {
            modernize_file(entry.path())?;
        }
    }
    
    println!("✅ Response pattern modernization complete!");
    Ok(())
}

fn modernize_file(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let mut updated_content = content.clone();
    
    // Pattern 1: Functions returning Result<(), SongbirdError> should just return Ok(())
    // not Ok(SongbirdResponse::success(()))
    let pattern1 = r"Ok\(SongbirdResponse::success\(\(\)\)\)";
    let replacement1 = "Ok(())";
    updated_content = regex::Regex::new(pattern1)?.replace_all(&updated_content, replacement1).to_string();
    
    // Pattern 2: Fix function signatures that should return Result<(), SongbirdError>
    // instead of SongbirdResult<()>
    let pattern2 = r"-> SongbirdResult<\(\)>";
    let replacement2 = "-> Result<()>";
    updated_content = regex::Regex::new(pattern2)?.replace_all(&updated_content, replacement2).to_string();
    
    // Pattern 3: Import the correct Result type
    if updated_content.contains("use songbird_errors::") && !updated_content.contains("use songbird_errors::Result") {
        let pattern3 = r"use songbird_errors::\{([^}]*)\}";
        if let Some(captures) = regex::Regex::new(pattern3)?.captures(&updated_content) {
            let imports = &captures[1];
            if !imports.contains("Result") {
                let new_imports = format!("{}, Result", imports);
                updated_content = updated_content.replace(&captures[0], &format!("use songbird_errors::{{{}}}", new_imports));
            }
        }
    }
    
    if content != updated_content {
        fs::write(path, updated_content)?;
        println!("📝 Updated: {}", path.display());
    }
    
    Ok(())
} 