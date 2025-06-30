use songbird_gaming_bridge::SongbirdOrchestrator;
use songbird_gaming_bridge::config::NetworkConfig;
use std::collections::HashMap;
//! Hardcoding Detection Tests

use regex::Regex;
use std::fs;
use std::path::Path;

#[test]
fn test_ip_address_hardcoding() {
    let ip_regex = Regex::new(r"\b(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\b").unwrap_or_default();
    let mut violations = Vec::new();
    
    scan_src_files(&ip_regex, &mut violations, &["tests/", "examples/"]);
    
    println!("🔍 IP Address Hardcoding Scan: {} potential issues found", violations.len());
    
    for violation in &violations[..std::cmp::min(3, violations.len())] {
        println!("  {}", violation);
    }
    
    if violations.len() > 10 {
        panic!("Too many hardcoded IP addresses found: {}", violations.len());
    }
}

#[test]
fn test_port_hardcoding() {
    let port_regex = Regex::new(r":\s*(\d{4,5})\b").unwrap_or_default();
    let mut violations = Vec::new();
    
    scan_src_files(&port_regex, &mut violations, &["tests/", "examples/", "config/"]);
    
    println!("🔍 Port Hardcoding Scan: {} potential issues found", violations.len());
    
    if violations.len() > 15 {
        println!("⚠️  Many hardcoded ports found - consider using configuration");
    }
}

#[test]
fn test_magic_numbers() {
    let magic_regex = Regex::new(r"\b(?:1024|2048|4096|8192|65536|1048576)\b").unwrap_or_default();
    let mut violations = Vec::new();
    
    scan_src_files(&magic_regex, &mut violations, &["tests/", "examples/", "config/constants.rs"]);
    
    println!("🔍 Magic Numbers Scan: {} potential issues found", violations.len());
    
    if violations.len() > 20 {
        println!("⚠️  Consider moving magic numbers to named constants");
    }
}

#[test]
fn test_error_handling_patterns() {
    let unwrap_regex = Regex::new(r"\.unwrap\(\)").unwrap_or_default();
    let mut violations = Vec::new();
    
    scan_src_files(&unwrap_regex, &mut violations, &["tests/", "examples/"]);
    
    println!("🔍 Error Handling Scan: {} unwrap() calls found", violations.len());
    
    if violations.len() > 30 {
        println!("⚠️  Consider using proper error handling instead of unwrap()");
        for violation in &violations[..std::cmp::min(5, violations.len())] {
            println!("  {}", violation);
        }
    } else {
        println!("✅ Error handling looks reasonable");
    }
}

fn scan_src_files(regex: &Regex, violations: &mut Vec<String>, allowed_paths: &[&str]) {
    scan_directory(Path::new("src"), regex, violations, allowed_paths);
}

fn scan_directory(dir: &Path, regex: &Regex, violations: &mut Vec<String>, allowed_paths: &[&str]) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_dir() {
                scan_directory(&path, regex, violations, allowed_paths);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                let path_str = path.to_string_lossy();
                
                let is_allowed = allowed_paths.iter().any(|allowed| path_str.contains(allowed));
                
                if !is_allowed {
                    if let Ok(content) = fs::read_to_string(&path) {
                        for (line_num, line) in content.lines().enumerate() {
                            if regex.is_match(line) {
                                violations.push(format!("{}:{}: {}", path.display(), line_num + 1, line.trim()));
                            }
                        }
                    }
                }
            }
        }
    }
}
