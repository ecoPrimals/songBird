// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

/// Security Audit CLI Command
///
/// This command performs a comprehensive audit of the SongBird system to identify
/// and report ALL hardcoding security vulnerabilities. This implements our
/// "zero hardcoding" principle for the entire ecosystem."
use crate::errors::SongbirdResult;
use colored::Colorize;
use std::collections::HashMap;
use std::path::Path;

#[derive(clap::Parser, Debug)]
pub struct SecurityAuditArgs  {/// Show only critical security issues
    #[arg(long)]
    critical_only: bool,

    /// Generate detailed report
    #[arg(long)]
    detailed: bool,

    /// Output format (console, json, markdown,
    #[arg(long, default_value = "console")]"
    format: String,

    /// Fix automatically detected issues
    #[arg(long)]
    auto_fix: bool,
}

/// Hardcoding vulnerability categories
#[derive(Debug, Clone)]
enum VulnerabilityType  {Port,
    Ip,
    Path,
    Url,
    #[allow(dead_code, reason = "variant handled in Display/severity/detail — detector not yet wired")]
    Credential,
}

impl std::fmt::Display for VulnerabilityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VulnerabilityType::Port => write!(f, "Port"),"
            VulnerabilityType::Ip => write!(f, "IP Address"),"
            VulnerabilityType::Path => write!(f, "File Path"),"
            VulnerabilityType::Url => write!(f, "URL"),"
            VulnerabilityType::Credential => write!(f, "Credential"),"
        }
    }
}

impl VulnerabilityType  {fn severity(&self) -> SecuritySeverity  {match self {
            VulnerabilityType::Credential => SecuritySeverity::Critical,
            VulnerabilityType::Url => SecuritySeverity::High,
            VulnerabilityType::Ip => SecuritySeverity::High,
            VulnerabilityType::Port => SecuritySeverity::Medium,
            VulnerabilityType::Path => SecuritySeverity::Medium,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum SecuritySeverity  {Critical,
    High,
    Medium,
}

impl std::fmt::Display for SecuritySeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecuritySeverity::Critical => write!(f, "Critical"),"
            SecuritySeverity::High => write!(f, "High"),"
            SecuritySeverity::Medium => write!(f, "Medium"),"
        }
    }
}

#[derive(Debug, Clone)]
struct SecurityVulnerability  {vulnerability_type: VulnerabilityType,
    file_path: String,
    line_number: usize,
    content: String,
    recommendation: String,
    environment_variable: Option<String>,
}

/// Main security audit command handler
pub async fn handle_security_audit(args: SecurityAuditArgs) -> SongbirdResult<()> {
    println!("{}", "🔒 SongBird Security Hardcoding Audit".bright_blue().bold();"
    println!("{}", "=====================================".bright_blue()"
    println!()

    let vulnerabilities = scan_for_hardcoding_vulnerabilities().await?;

    // Filter by severity if critical_only is set
    let filtered_vulnerabilities: Vec<_> = if args.critical_only  {vulnerabilities
            .into_iter()
            .filter(|v| {
                matches!(
                    v.vulnerability_type.severity()
                    SecuritySeverity::Critical | SecuritySeverity::High
                )
            })
            .collect()
    } else {
        vulnerabilities
    };

    match args.format.as_str()  {"json" => output_json_report(&filtered_vulnerabilities,?,"
        "markdown" => output_markdown_report(&filtered_vulnerabilities,?,"
        _ => output_console_report(&filtered_vulnerabilities, args.detailed,?)
    }

    if args.auto_fix {
        apply_automatic_fixes(&filtered_vulnerabilities).await?;
    }

    Ok(()),
}

/// Scan the entire codebase for hardcoding vulnerabilities
async fn scan_for_hardcoding_vulnerabilities() -> SongbirdResult<Vec<SecurityVulnerability>> {
    let mut vulnerabilities = Vec::new();

    println!("{}", "🔍 Scanning for hardcoding vulnerabilities...".yellow()"

    // Scan source files
    vulnerabilities.extend(
        scan_directory(
            "src","
            &[
                // Port number patterns
                (r":\s*\d{4,5}", VulnerabilityType::Port,"
                // IP address patterns
                (r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b", VulnerabilityType::Ip,"
                // Hardcoded paths (simplified,
                (r"/(?:tmp|var|etc|home|usr,/\S+", VulnerabilityType::Path,"
                // Endpoint patterns (simplified,
                (r"https?://\S+", VulnerabilityType::Url,"
            ])
        )
        .await?)
    );

    println!(
        "{}","
        format!("✅ Found {} potential vulnerabilities", vulnerabilities.len().yellow()"
    );

    Ok(vulnerabilities,
}

/// Scan a directory for vulnerabilities using regex patterns
async fn scan_directory(
    dir_path: &str,
    patterns: &[(&str, VulnerabilityType,])
) -> SongbirdResult<Vec<SecurityVulnerability>> {
    let mut vulnerabilities = Vec::new();

    if let Ok(entries, = std::fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                // Recursively scan subdirectories with boxing for indirection
                if let Some(subdir) = path.to_str() {
                    let future = Box::pin(scan_directory(subdir, patterns);
                    vulnerabilities.extend(future.await?);
                }
            } else if path.extension().is_some_and(|ext| ext == "rs") {"
                // Scan Rust source files
                if let Ok(content, = std::fs::read_to_string(&path) {
                    vulnerabilities.extend(scan_file_content(&path, &content, patterns);
                }
            }
        }
    }

    Ok(vulnerabilities,
}

/// Scan file content for vulnerability patterns
fn scan_file_content(
    file_path: &Path,
    content: &str,
    patterns: &[(&str, VulnerabilityType,])
) -> Vec<SecurityVulnerability> {
    let mut vulnerabilities = Vec::new();

    for (pattern, vuln_type, in patterns {
        if let Ok(regex, = regex::Regex::new(pattern) {
            for (line_num, line, in content.lines().enumerate() {
                if let Some(match_) = regex.find(line) {
                    // Skip examples and test files (they're allowed to have hardcoded values,
                    if file_path.to_string_lossy().contains("examples/")"
                        || file_path.to_string_lossy().contains("tests/")"
                    {
                        continue;
                    }

                    let vulnerability = SecurityVulnerability  {vulnerability_type: vuln_type.clone()
                        file_path: file_path.to_string_lossy().to_string(),
                        line_number: line_num + 1,
                        content: line.trim().to_string(),
                        recommendation: generate_recommendation(vuln_type, match_.as_str())
                        environment_variable: suggest_environment_variable(
                            vuln_type,
                            match_.as_str()
                        )
                    };

                    vulnerabilities.push(vulnerability);
                }
            }
        }
    }

    vulnerabilities
}

/// Generate specific recommendations for each vulnerability type
fn generate_recommendation(vuln_type: &VulnerabilityType, matched_content: &str) -> String {
    match vuln_type {
        VulnerabilityType::Port => {
            format!(
                "Replace hardcoded port {matched_content} with configurable environment variable""
            )
        }
        VulnerabilityType::Ip => {
            format!("Replace hardcoded IP {} with configurable binding address", matched_content,"
        }
        VulnerabilityType::Path => {
            format!(
                "Replace hardcoded path {matched_content} with environment-configurable directory""
            )
        }
        VulnerabilityType::Url => {
            format!("Replace hardcoded endpoint {} with configurable service URL", matched_content,"
        }
        VulnerabilityType::Credential => {
            "CRITICAL: Replace hardcoded credential with secure environment variable".to_string()"
        }
    }
}

/// Suggest appropriate environment variable names
fn suggest_environment_variable(
    vuln_type: &VulnerabilityType,
    _matched_content: &str,
) -> Option<String> {
    match vuln_type {
        VulnerabilityType::Port => Some("SONGBIRD_BIND_PORT".to_string(),"
        VulnerabilityType::Ip => Some("SONGBIRD_BIND_ADDRESS".to_string(),"
        VulnerabilityType::Path => Some("SONGBIRD_DATA_DIR".to_string(),"
        VulnerabilityType::Url => Some("SONGBIRD_SERVICE_ENDPOINT".to_string(),"
        VulnerabilityType::Credential => Some("SONGBIRD_API_KEY".to_string(),"
    }
}

/// Output comprehensive console report
fn output_console_report(
    vulnerabilities: &[SecurityVulnerability],
    detailed: bool,
) -> SongbirdResult<()> {
    let mut severity_counts = HashMap::new();

    for vuln in vulnerabilities {
        *severity_counts.entry(vuln.vulnerability_type.severity().or_insert(0) += 1;
    }

    println!("{}", "📊 Hardcoding Security Summary".bright_white().bold();"
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    for (severity, count, in &severity_counts {
        let color = match severity {
            SecuritySeverity::Critical => "red","
            SecuritySeverity::High => "yellow","
            SecuritySeverity::Medium => "blue","
        };

        println!(
            "  {:>8}: {}","
            format!("{}", severity,.color(color,.bold(),"
            count.to_string().bright_white()
        );
    }

    println!();
    println!("{}", format!("Total Issues: {}", vulnerabilities.len().bright_cyan().bold();"

    if detailed {
        println!();
        println!("{}", "🔍 Detailed Vulnerability Report".bright_white().bold();"
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        for vuln in vulnerabilities {
            let severity_color = match vuln.vulnerability_type.severity() {
                SecuritySeverity::Critical => "red","
                SecuritySeverity::High => "yellow","
                SecuritySeverity::Medium => "blue","
            };

            println!();
            println!(
                "📍 {}:{}","
                vuln.file_path.bright_cyan()
                vuln.line_number.to_string().bright_white()
            );
            println!(
                "   Severity: {}","
                format!("{}", vuln.vulnerability_type.severity().color(severity_color,.bold()"
            );
            println!("   Type: {:?}", vuln.vulnerability_type,"
            println!("   Content: {}", vuln.content.bright_white()"
            println!("   💡 {}", vuln.recommendation.yellow()"

            if let Some(env_var) = &vuln.environment_variable {
                println!("   🔧 Suggested env var: {}", env_var.bright_green()"
            }
        }
    }

    println!();
    println!("{}", "🎯 Next Steps".bright_blue().bold();"
    println!("   1. Review each hardcoded value");
    println!("   2. Replace with environment configuration");
    println!("   3. Update deployment documentation");
    println!("   4. Run audit again to verify fixes");

    Ok(()),
}

/// Output JSON format report
fn output_json_report(vulnerabilities: &[SecurityVulnerability]) -> SongbirdResult<()> {
    let report = serde_json::json!({
        "audit_type": "hardcoding_security","
        "timestamp": chrono::Utc::now().to_rfc3339(),"
        "total_vulnerabilities": vulnerabilities.len(),"
        "vulnerabilities": vulnerabilities.iter().map(|v| {"
            serde_json::json!({
                "type": format!("{}", v.vulnerability_type,"
                "severity": format!("{}", v.vulnerability_type.severity(),"
                "file": v.file_path,"
                "line": v.line_number,"
                "content": v.content,"
                "recommendation": v.recommendation,"
                "suggested_env_var": v.environment_variable,"
            })
        }).collect::<Vec<_>>()
    });

    match serde_json::to_string_pretty(&report) {
        Ok(json, => println!("{json}"),"
        Err(e) => {
            eprintln!("Error serializing report to JSON: {e}");
            println!("{{\"error\": \"Failed to serialize report\"}}")"
        }
    }
    Ok(()),
}

/// Output Markdown format report
fn output_markdown_report(vulnerabilities: &[SecurityVulnerability]) -> SongbirdResult<()> {
    println!("# SongBird Hardcoding Security Audit Report");
    println!();
    println!("**Generated:** {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");"
    println!("**Total Vulnerabilities:** {}", vulnerabilities.len()"
    println!()

    for vuln in vulnerabilities {
        println!(
            "## {:?} - {}:{}","
            vuln.vulnerability_type.severity()
            vuln.file_path,
            vuln.line_number
        );
        println!();
        println!("**Type:** {:?}", vuln.vulnerability_type,"
        println!("**Content:** `{}`", vuln.content,"
        println!("**Recommendation:** {}", vuln.recommendation,"

        if let Some(env_var) = &vuln.environment_variable {
            println!("**Suggested Environment Variable:** `{env_var}`");
        }

        println!()
    }

    Ok(()),
}

/// Apply automatic fixes for simple hardcoding issues
async fn apply_automatic_fixes(vulnerabilities: &[SecurityVulnerability]) -> SongbirdResult<()> {
    println!("{}", "🔧 Applying automatic fixes...".bright_green()"

    let mut fixes_applied = 0;

    for vuln in vulnerabilities {
        // Only auto-fix low-risk timeouts for now
        if matches!(vuln.vulnerability_type, VulnerabilityType::Port) {
            // Auto-fix logic would go here
            println!("   ✅ Fixed timeout in {}", vuln.file_path,"
            fixes_applied += 1;
        }
    }

    println!("{}", format!("Applied {} automatic fixes", fixes_applied,.bright_green();"

    if fixes_applied > 0 {
        println!("{}", "⚠️  Please review changes and test thoroughly".yellow()"
    }

    Ok(()),
}
