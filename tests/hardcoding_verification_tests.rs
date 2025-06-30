use songbird_gaming_bridge::SongbirdOrchestrator;
use songbird_gaming_bridge::config::NetworkConfig;
use std::collections::HashMap;
//! Hardcoding Verification Tests
//!
//! Ensures 100% compliance with no hardcoded values policy.
//! Uses regex patterns to identify and flag any disallowed hardcoded values.

use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

/// Comprehensive hardcoding verification test suite
#[cfg(test)]
mod hardcoding_tests {
    use super::*;
    use std::sync::Once;
    
    static INIT: Once = Once::new();
    
    fn init_test_logging() {
        INIT.call_once(|| {
            env_logger::init();
        });
    }

    /// **CRITICAL TEST:** Verify no hardcoded IP addresses
    #[tokio::test]
    async fn test_no_hardcoded_ip_addresses() {
        init_test_logging();
        
        let violations = scan_for_pattern(
            r"\b(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\b",
            "Hardcoded IP addresses",
            get_allowed_ip_patterns()
        ).await;
        
        assert_eq!(violations.len(), 0, 
            "Found {} hardcoded IP addresses:\n{}", 
            violations.len(), 
            format_violations(&violations)
        );
    }

    /// **CRITICAL TEST:** Verify no hardcoded ports
    #[tokio::test]
    async fn test_no_hardcoded_ports() {
        init_test_logging();
        
        let violations = scan_for_pattern(
            r":\s*\d{4,5}\b",
            "Hardcoded port numbers",
            get_allowed_port_patterns()
        ).await;
        
        assert_eq!(violations.len(), 0, 
            "Found {} hardcoded port numbers:\n{}", 
            violations.len(), 
            format_violations(&violations)
        );
    }

    /// **CRITICAL TEST:** Verify no hardcoded file paths
    #[tokio::test]
    async fn test_no_hardcoded_paths() {
        init_test_logging();
        
        let violations = scan_for_pattern(
            r#"(?:"/[^"]*"|'/[^']*'|/(?:tmp|var|etc|home|usr|opt)/\S+)"#,
            "Hardcoded file paths",
            get_allowed_path_patterns()
        ).await;
        
        assert_eq!(violations.len(), 0, 
            "Found {} hardcoded file paths:\n{}", 
            violations.len(), 
            format_violations(&violations)
        );
    }

    /// **CRITICAL TEST:** Verify no hardcoded URLs
    #[tokio::test]
    async fn test_no_hardcoded_urls() {
        init_test_logging();
        
        let violations = scan_for_pattern(
            r"https?://[^\s\"']+",
            "Hardcoded URLs",
            get_allowed_url_patterns()
        ).await;
        
        assert_eq!(violations.len(), 0, 
            "Found {} hardcoded URLs:\n{}", 
            violations.len(), 
            format_violations(&violations)
        );
    }

    /// **CRITICAL TEST:** Verify no hardcoded credentials
    #[tokio::test]
    async fn test_no_hardcoded_credentials() {
        init_test_logging();
        
        let violations = scan_for_pattern(
            r"(?i)(?:password|secret|key|token)\s*[=:]\s*[\"'][^\"']+[\"']",
            "Hardcoded credentials",
            HashSet::new() // No allowed hardcoded credentials
        ).await;
        
        assert_eq!(violations.len(), 0, 
            "CRITICAL SECURITY VIOLATION: Found {} hardcoded credentials:\n{}", 
            violations.len(), 
            format_violations(&violations)
        );
    }

    /// **COMPREHENSIVE TEST:** All hardcoding patterns combined
    #[tokio::test]
    async fn test_comprehensive_hardcoding_scan() {
        init_test_logging();
        
        let mut all_violations = Vec::new();
        
        // Scan for all patterns
        let patterns = vec![
            (r"\b(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\b", "IP addresses", get_allowed_ip_patterns()),
            (r":\s*\d{4,5}\b", "Port numbers", get_allowed_port_patterns()),
            (r#"(?:"/[^"]*"|'/[^']*'|/(?:tmp|var|etc|home|usr|opt)/\S+)"#, "File paths", get_allowed_path_patterns()),
            (r"https?://[^\s\"']+", "URLs", get_allowed_url_patterns()),
        ];
        
        for (pattern, description, allowed) in patterns {
            let violations = scan_for_pattern(pattern, description, allowed).await;
            all_violations.extend(violations);
        }
        
        // Generate comprehensive report
        if !all_violations.is_empty() {
            let report = generate_hardcoding_report(&all_violations);
            println!("{}", report);
            panic!("Found {} total hardcoding violations. See report above.", all_violations.len());
        }
    }
}

/// Core scanning functionality
#[derive(Debug, Clone)]
struct HardcodingViolation {
    file_path: String,
    line_number: usize,
    line_content: String,
    matched_text: String,
    violation_type: String,
}

/// Scan for specific regex pattern across codebase
async fn scan_for_pattern(
    pattern: &str,
    description: &str,
    allowed_patterns: HashSet<String>
) -> Vec<HardcodingViolation> {
    let mut violations = Vec::new();
    let regex = Regex::new(pattern).expect("Invalid regex pattern");
    
    // Scan source directories
    let directories = vec!["src", "tests", "examples"];
    
    for dir in directories {
        if Path::new(dir).exists() {
            violations.extend(scan_directory_for_pattern(&regex, dir, description, &allowed_patterns).await);
        }
    }
    
    violations
}

/// Recursively scan directory for violations
async fn scan_directory_for_pattern(
    regex: &Regex,
    dir_path: &str,
    description: &str,
    allowed_patterns: &HashSet<String>
) -> Vec<HardcodingViolation> {
    let mut violations = Vec::new();
    
    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_dir() {
                // Recursively scan subdirectories
                if let Some(subdir) = path.to_str() {
                    violations.extend(scan_directory_for_pattern(regex, subdir, description, allowed_patterns).await);
                }
            } else if let Some(ext) = path.extension() {
                if ext == "rs" {
                    // Scan Rust source files
                    violations.extend(scan_file_for_pattern(regex, &path, description, allowed_patterns));
                }
            }
        }
    }
    
    violations
}

/// Scan individual file for violations
fn scan_file_for_pattern(
    regex: &Regex,
    file_path: &Path,
    description: &str,
    allowed_patterns: &HashSet<String>
) -> Vec<HardcodingViolation> {
    let mut violations = Vec::new();
    
    if let Ok(content) = fs::read_to_string(file_path) {
        for (line_num, line) in content.lines().enumerate() {
            for capture in regex.find_iter(line) {
                let matched_text = capture.as_str();
                
                // Check if this match is in the allowed patterns
                if !is_allowed_pattern(matched_text, allowed_patterns) {
                    violations.push(HardcodingViolation {
                        file_path: file_path.to_string_lossy().to_string(),
                        line_number: line_num + 1,
                        line_content: line.trim().to_string(),
                        matched_text: matched_text.to_string(),
                        violation_type: description.to_string(),
                    });
                }
            }
        }
    }
    
    violations
}

/// Check if pattern is explicitly allowed
fn is_allowed_pattern(text: &str, allowed_patterns: &HashSet<String>) -> bool {
    allowed_patterns.iter().any(|pattern| {
        if let Ok(regex) = Regex::new(pattern) {
            regex.is_match(text)
        } else {
            text == pattern
        }
    })
}

/// **EXPLICITLY ALLOWED IP PATTERNS**
/// These are the ONLY IP addresses allowed to be hardcoded
fn get_allowed_ip_patterns() -> HashSet<String> {
    let mut allowed = HashSet::new();
    
    // Localhost patterns (always acceptable for examples/tests)
    allowed.insert(r"127\.0\.0\.1".to_string());
    allowed.insert(r"0\.0\.0\.0".to_string());
    
    // Test/example patterns
    allowed.insert(r"192\.168\.1\.1".to_string()); // Common router IP for examples
    allowed.insert(r"10\.0\.0\.1".to_string());    // Private network example
    
    // Documentation examples only
    allowed.insert(r"255\.255\.255\.255".to_string()); // Broadcast address for demos
    
    allowed
}

/// **EXPLICITLY ALLOWED PORT PATTERNS**
/// These are the ONLY ports allowed to be hardcoded
fn get_allowed_port_patterns() -> HashSet<String> {
    let mut allowed = HashSet::new();
    
    // Standard service ports (for examples/documentation only)
    allowed.insert(r":80\b".to_string());    // HTTP
    allowed.insert(r":443\b".to_string());   // HTTPS
    allowed.insert(r":8080\b".to_string());  // Common development port
    
    // STUN server standard ports (explicitly allowed)
    allowed.insert(r":19302\b".to_string()); // Google STUN
    allowed.insert(r":3478\b".to_string());  // Standard STUN
    
    allowed
}

/// **EXPLICITLY ALLOWED PATH PATTERNS**
/// These are the ONLY file paths allowed to be hardcoded
fn get_allowed_path_patterns() -> HashSet<String> {
    let mut allowed = HashSet::new();
    
    // Standard system paths that are acceptable in configuration
    allowed.insert(r"/dev/null".to_string());
    allowed.insert(r"/proc/.*".to_string());  // System proc files
    
    // Test fixtures only
    allowed.insert(r"/tmp/test.*".to_string());
    
    allowed
}

/// **EXPLICITLY ALLOWED URL PATTERNS**
/// These are the ONLY URLs allowed to be hardcoded
fn get_allowed_url_patterns() -> HashSet<String> {
    let mut allowed = HashSet::new();
    
    // STUN servers (explicitly required for NAT traversal)
    allowed.insert(r"stun\.l\.google\.com:19302".to_string());
    allowed.insert(r"stun1\.l\.google\.com:19302".to_string());
    
    // Documentation examples only
    allowed.insert(r"https://example\.com".to_string());
    allowed.insert(r"http://localhost.*".to_string());
    
    allowed
}

/// Format violations for clear display
fn format_violations(violations: &[HardcodingViolation]) -> String {
    let mut output = String::new();
    
    for violation in violations {
        output.push_str(&format!(
            "  📍 {}:{} - {} '{}'\n     Line: {}\n",
            violation.file_path,
            violation.line_number,
            violation.violation_type,
            violation.matched_text,
            violation.line_content
        ));
    }
    
    output
}

/// Generate comprehensive hardcoding report
fn generate_hardcoding_report(violations: &[HardcodingViolation]) -> String {
    let mut report = String::from("\n🚨 HARDCODING VIOLATIONS DETECTED 🚨\n");
    report.push_str("=====================================\n\n");
    
    // Group by violation type
    let mut by_type: HashMap<String, Vec<&HardcodingViolation>> = HashMap::new();
    for violation in violations {
        by_type.entry(violation.violation_type.clone()).or_default().push(violation);
    }
    
    for (violation_type, type_violations) in by_type {
        report.push_str(&format!("🔍 {} ({} violations):\n", violation_type, type_violations.len()));
        
        for violation in type_violations {
            report.push_str(&format!(
                "  📂 {}:{}\n",
                violation.file_path,
                violation.line_number
            ));
            report.push_str(&format!("     🎯 Matched: '{}'\n", violation.matched_text));
            report.push_str(&format!("     📄 Line: {}\n", violation.line_content));
            report.push_str("     💡 Solution: Replace with environment variable or configuration\n\n");
        }
    }
    
    report.push_str("🔧 REMEDIATION REQUIRED:\n");
    report.push_str("   1. Replace hardcoded values with environment variables\n");
    report.push_str("   2. Use configuration files for all settings\n");
    report.push_str("   3. Update allowed patterns if values are intentionally hardcoded\n\n");
    
    report
} 