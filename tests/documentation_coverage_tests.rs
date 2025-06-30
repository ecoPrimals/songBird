use songbird_gaming_bridge::SongbirdOrchestrator;
use songbird_gaming_bridge::config::NetworkConfig;
use std::collections::HashMap;
//! Documentation Coverage Tests

use regex::Regex;
use std::fs;
use std::path::Path;

#[test]
fn test_public_function_documentation() {
    let function_regex = Regex::new(r"pub\s+(?:async\s+)?fn\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap_or_default();
    let mut undocumented = Vec::new();
    let mut total = 0;
    
    scan_documentation_coverage(Path::new("src"), &function_regex, &mut undocumented, &mut total);
    
    let coverage = if total > 0 { ((total - undocumented.len()) * 100) / total } else { 100 };
    
    println!("📚 Public Function Documentation: {}% ({}/{} documented)", coverage, total - undocumented.len(), total);
    
    if coverage < 70 {
        println!("📝 Undocumented functions:");
        for item in &undocumented[..std::cmp::min(5, undocumented.len())] {
            println!("  {}", item);
        }
        if undocumented.len() > 5 {
            println!("  ... and {} more", undocumented.len() - 5);
        }
        println!("💡 Add /// documentation comments above public functions");
    }
}

#[test]
fn test_public_struct_documentation() {
    let struct_regex = Regex::new(r"pub\s+struct\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap_or_default();
    let mut undocumented = Vec::new();
    let mut total = 0;
    
    scan_documentation_coverage(Path::new("src"), &struct_regex, &mut undocumented, &mut total);
    
    let coverage = if total > 0 { ((total - undocumented.len()) * 100) / total } else { 100 };
    
    println!("📚 Public Struct Documentation: {}% ({}/{} documented)", coverage, total - undocumented.len(), total);
    
    if coverage < 80 {
        println!("📝 Undocumented structs found - consider adding documentation");
    } else {
        println!("✅ Struct documentation coverage is good");
    }
}

#[test]
fn test_module_documentation() {
    let mut undocumented_modules = Vec::new();
    
    scan_module_documentation(Path::new("src"), &mut undocumented_modules);
    
    println!("📁 Module Documentation: {} modules without //! comments", undocumented_modules.len());
    
    if undocumented_modules.len() > 20 {
        println!("📝 Consider adding module documentation (//!) for better code navigation");
    } else {
        println!("✅ Module documentation coverage is reasonable");
    }
}

fn scan_documentation_coverage(dir: &Path, regex: &Regex, undocumented: &mut Vec<String>, total: &mut usize) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_dir() {
                scan_documentation_coverage(&path, regex, undocumented, total);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                let path_str = path.to_string_lossy();
                
                if path_str.contains("test") { continue; }
                
                if let Ok(content) = fs::read_to_string(&path) {
                    let lines: Vec<&str> = content.lines().collect();
                    
                    for (line_num, line) in lines.iter().enumerate() {
                        if let Some(captures) = regex.captures(line) {
                            if let Some(item_name) = captures.get(1) {
                                *total += 1;
                                
                                let has_doc = check_documentation_above(&lines, line_num);
                                
                                if !has_doc {
                                    undocumented.push(format!("{}:{}: {}", path.display(), line_num + 1, item_name.as_str()));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn check_documentation_above(lines: &[&str], line_num: usize) -> bool {
    if line_num == 0 { return false; }
    
    let start_line = line_num.saturating_sub(5);
    
    for i in (start_line..line_num).rev() {
        let line = lines[i].trim();
        if line.starts_with("///") || line.starts_with("#[doc") {
            return true;
        }
        if !line.is_empty() && !line.starts_with("//") && !line.starts_with("#[") {
            break;
        }
    }
    false
}

fn scan_module_documentation(dir: &Path, undocumented_modules: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_dir() {
                scan_module_documentation(&path, undocumented_modules);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                let path_str = path.to_string_lossy();
                
                if path_str.contains("test") { continue; }
                
                if let Ok(content) = fs::read_to_string(&path) {
                    let has_module_doc = content.lines()
                        .take(10)
                        .any(|line| line.trim().starts_with("//!"));
                    
                    if !has_module_doc {
                        undocumented_modules.push(path_str.to_string());
                    }
                }
            }
        }
    }
}
