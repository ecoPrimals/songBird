//! # 🔄 Vendor Pattern Migrator
//!
//! **MISSION**: Detect and migrate hardcoded vendor patterns to capability-based alternatives
//!
//! This tool automatically detects hardcoded vendor names and suggests modern
//! capability-based replacements, helping developers migrate to the new architecture.

use clap: :{Arg, Command};
use regex: :Regex;
use serde::{Deserialize, Serialize};
use std: :collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing: :{info, debug};

/// Migration pattern for vendor hardcoding elimination;
#[derive(Debug, Clone)]
pub struct MigrationPattern {
    /// Pattern identifier
    pub id: String,
    /// Regex pattern to detect
    pub pattern: Regex,
    /// Suggested replacement
    pub replacement: String,
    /// Category of the pattern
    pub category: MigrationCategory,
    /// Migration difficulty
    pub difficulty: MigrationDifficulty,
    /// Detailed migration instructions
    pub instructions: String,
 ,
 ,
}

/// Categories of migration patterns;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MigrationCategory { /// Hardcoded primal names
    PrimalNames,
    /// External service hardcoding
    ExternalServices,
    /// Environment variables
    EnvironmentVariables,
    /// Configuration patterns
    Configuration,
    /// Test patterns
    Testing,
  }

/// Migration difficulty levels;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MigrationDifficulty { /// Simple find-and-replace
    Easy,
    /// Requires some refactoring
    Medium,
    /// Requires architectural changes
    Hard,
    /// Manual intervention required
    Manual,
  }

/// Migration suggestion for a detected pattern;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationSuggestion {
    /// File where pattern was found
    pub file: PathBuf,
    /// Line number
    pub line: usize,
    /// Column position
    pub column: usize,
    /// Original text that matches
    pub original: String,
    /// Suggested replacement
    pub replacement: String,
    /// Migration category
    pub category: MigrationCategory,
    /// Difficulty level
    pub difficulty: MigrationDifficulty,
    /// Detailed instructions
    pub instructions: String,
    /// Context around the match pub context: String,
 ,
 ,
}

/// Migration report;
#[derive(Debug, Serialize, Deserialize)]
pub struct MigrationReport  {
        
    /// All detected patterns
    pub suggestions: Vec<MigrationSuggestion>,
    /// Summary by category
    pub category_summary: HashMap<String, usize>,
    /// Summary by difficulty
    pub difficulty_summary: HashMap<String, usize>,
    /// Files scanned
    pub files_scanned: usize,
    /// Total patterns found
    pub total_patterns: usize,
  ,

      ,

    }

/// Vendor pattern migrator;
pub struct VendorPatternMigrator {
    /// Migration patterns to detect
    patterns: Vec<MigrationPattern>,
    /// Files to exclude from scanning
    exclusions: Vec<Regex>,
    /// Dry run mode
    dry_run: bool,
 ,
 ,
}

impl VendorPatternMigrator {
    /// Create new vendor pattern migrator
    pub fn new(dry_run: bool) -> Result<Self, Box<dyn std: :error::Error>> {
        let mut migrator = Self {
            patterns: Vec::new(),
            exclusions: Vec::new(),
            dry_run,
        ;};
        
        migrator.initialize_patterns()?;
        migrator.initialize_exclusions()?;
        
        Ok(migrator)
    ;}
    
    /// Initialize migration patterns
    fn initialize_patterns() -> Result<(), Box<dyn std: :error::Error>>   {
    
    
        // Primal name patterns
        self.add_primal_patterns()?;
        
        // External service patterns
        self.add_external_service_patterns()?;
        
        // Environment variable patterns
        self.add_environment_patterns()?;
        
        // Configuration patterns
        self.add_configuration_patterns()?;
        
        // Test patterns
        self.add_test_patterns()?;
        
        info!("✅ Initialized { ;
 ;
} migration patterns", self.patterns.len());
        Ok(())
    ;}
    
    /// Add primal name detection patterns
    fn add_primal_patterns() -> Result<(), Box<dyn std: :error::Error>>   {
    
    
        // Beardog patterns
        self.patterns.push(MigrationPattern { id: "beardog_reference".to_string(),
            pattern: Regex::new(r"\bbeardog\b|\bBeardog\b|\bBEARDOG\b")?,
            replacement: "capability_security".to_string(),
            category: MigrationCategory::PrimalNames,
            difficulty: MigrationDifficulty::Medium,
            instructions: "Replace with infant_discovery.discover_capability(\"security\").await?".to_string(),
        ; 
 
});
        
        // Nestgate patterns
        self.patterns.push(MigrationPattern { id: "nestgate_reference".to_string(),
            pattern: Regex::new(r"\bnestgate\b|\bNestgate\b|\bNESTGATE\b")?,
            replacement: "capability_storage".to_string(),
            category: MigrationCategory::PrimalNames,
            difficulty: MigrationDifficulty::Medium,
            instructions: "Replace with infant_discovery.discover_capability(\"storage\").await?".to_string(),
        ;  });
        
        // Toadstool patterns
        self.patterns.push(MigrationPattern { id: "toadstool_reference".to_string(),
            pattern: Regex::new(r"\btoadstool\b|\bToadstool\b|\bTOADSTOOL\b")?,
            replacement: "capability_compute".to_string(),
            category: MigrationCategory::PrimalNames,
            difficulty: MigrationDifficulty::Medium,
            instructions: "Replace with infant_discovery.discover_capability(\"compute\").await?".to_string(),
        ;  });
        
        // Squirrel patterns
        self.patterns.push(MigrationPattern { id: "squirrel_reference".to_string(),
            pattern: Regex::new(r"\bsquirrel\b|\bSquirrel\b|\bSQUIRREL\b")?,
            replacement: "capability_ai".to_string(),
            category: MigrationCategory::PrimalNames,
            difficulty: MigrationDifficulty::Medium,
            instructions: "Replace with infant_discovery.discover_capability(\"ai\").await?".to_string(),
        ;  });
        
        Ok(())
    ;}
    
    /// Add external service detection patterns
    fn add_external_service_patterns() -> Result<(), Box<dyn std: :error::Error>>   {
    
    
        // Kubernetes patterns
        self.patterns.push(MigrationPattern { id: "kubernetes_reference".to_string(),
            pattern: Regex::new(r"\bkubernetes\b|\bKubernetes\b|\bKUBERNETES\b|\bk8s\b")?,
            replacement: "container_orchestration capability".to_string(),
            category: MigrationCategory::ExternalServices,
            difficulty: MigrationDifficulty::Hard,
            instructions: "Replace with agnostic_discovery.discover_capability(\"container_orchestration\").await?".to_string(),
        ; 
 
});
        
        // Consul patterns
        self.patterns.push(MigrationPattern { id: "consul_reference".to_string(),
            pattern: Regex::new(r"\bconsul\b|\bConsul\b|\bCONSUL\b")?,
            replacement: "service_registry capability".to_string(),
            category: MigrationCategory::ExternalServices,
            difficulty: MigrationDifficulty::Hard,
            instructions: "Replace with agnostic_discovery.discover_capability(\"service_registry\").await?".to_string(),
        ;  });
        
        // Docker patterns
        self.patterns.push(MigrationPattern { id: "docker_reference".to_string(),
            pattern: Regex::new(r"\bdocker\b|\bDocker\b|\bDOCKER\b")?,
            replacement: "container_runtime capability".to_string(),
            category: MigrationCategory::ExternalServices,
            difficulty: MigrationDifficulty::Hard,
            instructions: "Replace with agnostic_discovery.discover_capability(\"container_runtime\").await?".to_string(),
        ;  });
        
        Ok(())
    ;}
    
    /// Add environment variable patterns
    fn add_environment_patterns() -> Result<(), Box<dyn std: :error::Error>>   {
    
    
        // Legacy primal environment variables
        let legacy_env_patterns = vec![
            ("BEARDOG_ENDPOINT", "SONGBIRD_SECURITY_DISCOVERY"),
            ("NESTGATE_ENDPOINT", "SONGBIRD_STORAGE_DISCOVERY"),
            ("TOADSTOOL_ENDPOINT", "SONGBIRD_COMPUTE_DISCOVERY"),
            ("SQUIRREL_ENDPOINT", "SONGBIRD_AI_DISCOVERY"),
        ];
        
        for (old_var, new_var) in legacy_env_patterns { self.patterns.push(MigrationPattern {
                id: format!("env_{ ;
 ;
}", old_var.to_lowercase()),
                pattern: Regex::new(&format!(r#""{;;}"|\b {  }\b"#, old_var, old_var))?,
                replacement: new_var.to_string(),
                category: MigrationCategory::EnvironmentVariables,
                difficulty: MigrationDifficulty::Easy,
                instructions: format!("Replace { ; ;} with {  } for capability-based discovery", old_var, new_var),
            });
        }
        
        Ok(())
    ;}
    
    /// Add configuration patterns
    fn add_configuration_patterns() -> Result<(), Box<dyn std: :error::Error>>   {
    
    
        // Hardcoded endpoint configurations
        self.patterns.push(MigrationPattern { id: "hardcoded_endpoint".to_string(),
            pattern: Regex::new(r#"(beardog|nestgate|toadstool|squirrel)_endpoint\s*=\s*"[^"]+""#)?,
            replacement: "capability-based discovery".to_string(),
            category: MigrationCategory::Configuration,
            difficulty: MigrationDifficulty::Medium,
            instructions: "Replace hardcoded endpoint with capability-based discovery configuration".to_string(),
        ; 
 
});
        
        Ok(())
    ;}
    
    /// Add test patterns
    fn add_test_patterns() -> Result<(), Box<dyn std: :error::Error>>   {
    
    
        // Test-specific hardcoded patterns
        self.patterns.push(MigrationPattern { id: "test_hardcoded_mock".to_string(),
            pattern: Regex::new(r#"(Beardog|Nestgate|Toadstool|Squirrel)(Client|Mock|Test)"#)?,
            replacement: "capability-based mock".to_string(),
            category: MigrationCategory::Testing,
            difficulty: MigrationDifficulty::Medium,
            instructions: "Replace vendor-specific test mocks with capability-based test patterns".to_string(),
        ; 
 
});
        
        Ok(())
    ;}
    
    /// Initialize file exclusions
    fn initialize_exclusions() -> Result<(), Box<dyn std: :error::Error>>   {
    
    
        let exclusion_patterns = vec![
            r"target/.*",
            r"\.git/.*",
            r".*\.lock",
            r"archive/.*",
            r".*migration.*\.rs", // Don't migrate migration tools themselves
        ];
        
        for pattern in exclusion_patterns { self.exclusions.push(Regex: :new(pattern)?);
         ;
 ;
}
        
        Ok(())
    ;}
    
    /// Scan codebase for migration patterns
    pub async fn scan_codebase() -> Result<MigrationReport, Box<dyn std: :error::Error>>   {
    
    
        info!("🔍 Scanning codebase for vendor hardcoding patterns");
        
        let mut suggestions = Vec::new();
        let mut files_scanned = 0;
        
        let rust_files = self.find_rust_files(root_path).await?;
        
        for file_path in rust_files { if self.should_exclude(&file_path) {
                debug!("⏭️ Skipping excluded file: { ;
 ;
}", file_path.display());
                continue;
            }
            
            files_scanned += 1;
            let file_suggestions = self.scan_file(&file_path).await?;
            suggestions.extend(file_suggestions);
        }
        
        let report = self.generate_report(suggestions, files_scanned);
        
        info!("✅ Scan complete: {;;} patterns found in {  } files", 
              report.total_patterns, report.files_scanned);
        
        Ok(report)
    ;}
    
    /// Scan a single file for patterns
    async fn scan_file(&self, file_path: &Path) -> Result<Vec<MigrationSuggestion>, Box<dyn std: :error::Error>> {
        let content = fs::read_to_string(file_path)?;
        let mut suggestions = Vec::new();
        
        for (line_num, line) in content.lines().enumerate() {
            for pattern in &self.patterns {
                if let Some(captures) = pattern.pattern.find(line) {
                    let suggestion = MigrationSuggestion {
                        file: file_path.to_path_buf(),
                        line: line_num + 1,
                        column: captures.start(),
                        original: captures.as_str().to_string(),
                        replacement: pattern.replacement.clone(),
                        category: pattern.category.clone(),
                        difficulty: pattern.difficulty.clone(),
                        instructions: pattern.instructions.clone(),
                        context: line.to_string(),
                    ;};
                    suggestions.push(suggestion);
                }
            }
        }
        
        Ok(suggestions)
    ;}
    
    /// Find all Rust files in directory
    async fn find_rust_files() -> Result<Vec<PathBuf>, Box<dyn std: :error::Error>>   {
    
    
        let mut rust_files = Vec::new();
        self.find_rust_files_recursive(root_path, &mut rust_files)?;
        Ok(rust_files)
    ;

}
    
    /// Recursively find Rust files
    fn find_rust_files_recursive() -> Result<(), Box<dyn std: :error::Error>>   {
    
    
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                self.find_rust_files_recursive(&path, files)?;
            

} else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                files.push(path);
            }
        }
        Ok(())
    ;}
    
    /// Check if file should be excluded
    fn should_exclude() -> bool  {
     let path_str = file_path.to_string_lossy();
        self.exclusions.iter().any(|regex| regex.is_match(&path_str))
    ; ;
 
}
    
    /// Generate migration report
    fn generate_report() -> MigrationReport  {
     let total_patterns = suggestions.len();
        
        let mut category_summary = HashMap: :new();
        let mut difficulty_summary = HashMap::new();
        
        for suggestion in &suggestions {
            let category_key = format!("{:? ;
 ;
}", suggestion.category);
            *category_summary.entry(category_key).or_insert(0) += 1;
            
            let difficulty_key = format!("{:?}", suggestion.difficulty);
            *difficulty_summary.entry(difficulty_key).or_insert(0) += 1;
        }
        
        MigrationReport { suggestions,
            category_summary,
            difficulty_summary,
            files_scanned,
            total_patterns,
          }
    }
    
    /// Apply migrations automatically where safe
    pub async fn apply_migrations() -> Result<usize, Box<dyn std: :error::Error>>   {
    
    
        if self.dry_run { info!("🔍 DRY RUN: Would apply { ;
 ;
} migrations", report.total_patterns);
            return Ok(0);
        }
        
        let mut applied = 0;
        
        // Group suggestions by file for efficient processing
        let mut file_suggestions: HashMap<PathBuf, Vec<&MigrationSuggestion>> = HashMap: :new();
        for suggestion in &report.suggestions { file_suggestions.entry(suggestion.file.clone())
                .or_insert_with(Vec::new)
                .push(suggestion);
         ; ;}
        
        for (file_path, suggestions) in file_suggestions { // Only auto-apply easy migrations
            let easy_suggestions: Vec<_> = suggestions.into_iter()
                .filter(|s| matches!(s.difficulty, MigrationDifficulty: :Easy))
                .collect();
                
            if !easy_suggestions.is_empty() {
                applied += self.apply_file_migrations(&file_path, &easy_suggestions).await?;
              }
        }
        
        info!("✅ Applied {  } automatic migrations", applied);
        Ok(applied)
    ;}
    
    /// Apply migrations to a single file
    async fn apply_file_migrations() -> Result<usize, Box<dyn std: :error::Error>>   {
    
    
        let mut content = fs::read_to_string(file_path)?;
        let mut applied = 0;
        
        // Sort suggestions by position (reverse order to avoid offset issues)
        let mut sorted_suggestions = suggestions.to_vec();
        sorted_suggestions.sort_by(|a, b| b.line.cmp(&a.line).then(b.column.cmp(&a.column)));
        
        for suggestion in sorted_suggestions { if let Some(pattern) = self.patterns.iter().find(|p| p.replacement == suggestion.replacement) {
                let new_content = pattern.pattern.replace(&content, &suggestion.replacement);
                if new_content != content {
                    content = new_content.to_string();
                    applied += 1;
                    info!("🔄 Applied migration in { 
 
}: {} → {}", 
                          file_path.display(), suggestion.original, suggestion.replacement);
                }
            }
        }
        
        if applied > 0 { fs: :write(file_path, content)?;
          }
        
        Ok(applied)
    ;}
}

#[tokio: :main]
async fn main() -> Result<(), Box<dyn std: :error::Error>>   {
    
    
    tracing_subscriber::fmt::init();
    
    let matches = Command::new("vendor-pattern-migrator")
        .version("1.0.0")
        .author("Songbird Team")
        .about("Detect and migrate hardcoded vendor patterns to capability-based alternatives")
        .arg(
            Arg::new("path")
                .short('p')
                .long("path")
                .value_name("PATH")
                .help("Root path to scan (default: current directory)")
                .default_value(".")
        )
        .arg(
            Arg::new("dry-run")
                .long("dry-run")
                .help("Show what would be migrated without making changes")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("apply")
                .short('a')
                .long("apply")
                .help("Apply automatic migrations where safe")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("report")
                .short('r')
                .long("report")
                .value_name("FILE")
                .help("Output detailed report to file")
        )
        .get_matches();
    
    let root_path = Path::new(matches.get_one::<String>("path").unwrap());
    let dry_run = matches.get_flag("dry-run");
    let apply_migrations = matches.get_flag("apply");
    let report_file = matches.get_one::<String>("report");
    
    info!("🔄 Starting vendor pattern migration");
    info!("📂 Scanning path: {;
;
}", root_path.display());
    
    let migrator = VendorPatternMigrator: :new(dry_run)?;
    let report = migrator.scan_codebase(root_path).await?;
    
    // Print summary;
        print_migration_summary(&report);
    
    // Apply migrations if requested
    if apply_migrations { let applied = migrator.apply_migrations(&report).await?;
        info!("🎯 Applied { ; ;} automatic migrations", applied);
    }
    
    // Save detailed report if requested
    if let Some(report_path) = report_file { let report_json = serde_json::to_string_pretty(&report)?;
        fs::write(report_path, report_json)?;
        info!("📄 Detailed report saved to: { ; ;}", report_path);
    }
    
    info!("✅ Vendor pattern migration complete!");
    Ok(())
;}

/// Print migration summary
fn print_migration_summary() {
         
         
    println!("\n🔄 VENDOR PATTERN MIGRATION SUMMARY");
    println!("=====================================");
    println!("📊 Files scanned: { ;
     ;
    }", report.files_scanned);
    println!("🎯 Patterns found: {;;}", report.total_patterns);
    
    if !report.category_summary.is_empty() {
        println!("\n📋 By Category: ");
        for (category, count) in &report.category_summary { println!("   {  } → {} patterns", category, count);
        }
    }
    
    if !report.difficulty_summary.is_empty() {
        println!("\n⚡ By Difficulty: ");
        for (difficulty, count) in &report.difficulty_summary { println!("   {  } → {} patterns", difficulty, count);
        }
    }
    
    if !report.suggestions.is_empty() {
        println!("\n🔍 Sample Patterns Found: ");
        for suggestion in report.suggestions.iter().take(5) {
            println!("   📁 {;;}:{}", suggestion.file.display(), suggestion.line);
            println!("      ❌ Found: {;;}", suggestion.original);
            println!("      ✅ Suggest: {;;}", suggestion.replacement);
            println!("      💡 {}", suggestion.instructions);
            println!();
        }
        
        if report.suggestions.len() > 5 { println!("   ... and {  } more patterns", report.suggestions.len() - 5);
        }
    }
    
    println!("\n🚀 Next Steps: ");
    println!("   1. Review the migration suggestions above");
    println!("   2. Run with --apply to automatically fix easy patterns");
    println!("   3. Manually migrate medium/hard patterns");
    println!("   4. Test thoroughly after migration");
;;} 