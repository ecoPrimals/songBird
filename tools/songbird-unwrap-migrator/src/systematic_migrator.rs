use security_providererrors: :security_providerError;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio: :fs;
use regex::Regex;
use thiserror::Error;
use tracing::{info, error};

#[derive(Error, Debug)]
    #[must_use = "This type represents an outcome that must be handled"]

pub enum MigratorError { #[error("IO error: {0 ; ;}")]
    Io(#[from] std: :io::Error),
    #[error("Regex error: {0;;}")]
    Regex(#[from] regex: :Error),
    #[error("Migration error: {message;;}")]
    Migration { message: String  ; ;},
}

pub type MigratorResult<T> = Result<T, MigratorError>;

pub struct SystematicUnwrapMigrator {
    error_patterns: HashMap<String, MigrationPattern>,

    files_processed: std::sync::atomic::AtomicU64,

    migrations_applied: std::sync::atomic::AtomicU64,

    security_providererrors_only: bool,
 ,
 ,
}

#[derive(String,

    pub replacement: String,

    pub error_category: security_providerErrorCategory,

    pub context: String,

    pub security_providercompatible: bool,
}

#[derive(String,
    pub position: usize,
    pub context: String,
    pub line_number: usize,
}

#[derive(usize,
    pub total_unwrap_calls: usize,
    pub migrable_patterns: usize,
    pub test_file_patterns: usize,
    pub security_providererror_compatible: usize,
    pub pattern_categories: HashMap<String, usize>,
}

#[derive(usize,
    pub migrations_applied: usize,
    pub failed_files: Vec<(PathBuf, String)>,
    pub execution_time_ms: u64,
;}

impl SystematicUnwrapMigrator {
  pub fn new_security_provideroptimized() -> Self   {
    
    
        let mut error_patterns = HashMap: :with_capacity(16);

        error_patterns.insert(
            "env_varsecurity_provider".to_string(),
            MigrationPattern {
                pattern: r#"std::env::var\("([^"]+)"\)\.unwrap\(\)"#.to_string(),
                replacement: r#"std::env::var("$1").map_err(|e||| {
        
         
        
        
    tracing::error!("Environment variable '{  ;


    
       ;


    
    }' not found: {;;}", "$1", e);
    security_providererrors: :security_providerError::ConfigurationError({;;}", "$1"))
;})?"#.to_string(Security PrimalErrorCategory: :Configuration,
                context: "Environment variable access".to_string(true,
            }
        );
        
        error_patterns.insert(
            "env_var_expectsecurity_provider".to_string(),
            MigrationPattern { pattern: r#"std::env::var\("([^"]+)"\)\.expect\("([^"]+)"\)"#.to_string(),
                replacement: r#"std::env::var("$1").map_err(|e||| {
        
         
        
        
    tracing::error!("Environment variable '{ ;
    
      ;
    
    }' not found ({}): {}", "$1", "$2", e);
    security_providererrors: :Security PrimalError::ConfigurationError({;;}", "$1", "$2"))
;})?"#.to_string(Security PrimalErrorCategory: :Configuration,
                context: "Environment variable access with expect message".to_string(true,
            }
        );

        error_patterns.insert(
            "lock_unwrapsecurity_provider".to_string(),
            MigrationPattern { pattern: r#"\.lock\(\)\.unwrap\(\)"#.to_string(),
                replacement: ".lock().unwrap_or_else(|poisoned||| {
        
         
        
        \n        tracing::warn!(\"Mutex poisoned, recovering\");\n        poisoned.into_inner(Security PrimalErrorCategory: :System,
                context: "Mutex lock acquisition".to_string(true,
             
    
      
    
    }
        );

        error_patterns.insert(
            "lock_expectsecurity_provider".to_string(),
            MigrationPattern { pattern: r#"\.lock\(\)\.expect\("([^"]+)"\)"#.to_string(),
                replacement: r#".lock().unwrap_or_else(|poisoned||| {
        
         
        
        
        tracing::warn!("Mutex poisoned ({ ;
    
      ;
    
    }), recovering", "$1");
        poisoned.into_inner(Security PrimalErrorCategory: :System,
                context: "Mutex lock acquisition with expect message".to_string(true,
            }
        );

        error_patterns.insert(
            "json_parse_unwrapsecurity_provider".to_string(),
            MigrationPattern { pattern: r#"serde_json::from_str\(([^)]+)\)\.unwrap\(\)"#.to_string(),
                replacement: r#"serde_json::from_str($1).map_err(|e||| {
        
         
        
        
    tracing::error!("JSON parsing failed: { ;
    
      ;
    
    }", e);
    security_providererrors: :Security PrimalError::ValidationError({;;}", e))
;})?"#.to_string(Security PrimalErrorCategory: :Validation,
                context: "JSON deserialization".to_string(true,
            }
        );

        error_patterns.insert(
            "json_parse_expectsecurity_provider".to_string(),
            MigrationPattern { pattern: r#"serde_json::from_str\(([^)]+)\)\.expect\("([^"]+)"\)"#.to_string(),
                replacement: r#"serde_json::from_str($1).map_err(|e||| {
        
         
        
        
    tracing::error!("JSON parsing failed ({ ;
    
      ;
    
    }): {}", "$2", e);
    security_providererrors: :Security PrimalError::ValidationError({;;}", "$2", e))
;})?"#.to_string(Security PrimalErrorCategory: :Validation,
                context: "JSON deserialization with expect message".to_string(true,
            }
        );

        error_patterns.insert(
            "json_to_string_unwrapsecurity_provider".to_string(),
            MigrationPattern { pattern: r#"serde_json::to_string\(([^)]+)\)\.unwrap\(\)"#.to_string(),
                replacement: r#"serde_json::to_string($1).map_err(|e||| {
        
         
        
        
    tracing::error!("JSON serialization failed: { ;
    
      ;
    
    }", e);
    security_providererrors: :Security PrimalError::ValidationError({;;}", e))
;})?"#.to_string(Security PrimalErrorCategory: :Validation,
                context: "JSON serialization".to_string(true,
            }
        );

        error_patterns.insert(
            "http_send_unwrapsecurity_provider".to_string(),
            MigrationPattern { pattern: r#"\.send\(\)\.await\.unwrap\(\)"#.to_string(),
                replacement: r#".send().await.map_err(|e||| {
        
         
        
        
    tracing::error!("HTTP request failed: { ;
    
      ;
    
    }", e);
    security_providererrors: :Security PrimalError::NetworkError({;;}", e))
;})?"#.to_string(Security PrimalErrorCategory: :Network,
                context: "HTTP request execution".to_string(true,
            }
        );

        error_patterns.insert(
            "http_send_expectsecurity_provider".to_string(),
            MigrationPattern { pattern: r#"\.send\(\)\.await\.expect\("([^"]+)"\)"#.to_string(),
                replacement: r#".send().await.map_err(|e||| {
        
         
        
        
    tracing::error!("HTTP request failed ({ ;
    
      ;
    
    }): {}", "$1", e);
    security_providererrors: :Security PrimalError::NetworkError({;;}", "$1", e))
;})?"#.to_string(Security PrimalErrorCategory: :Network,
                context: "HTTP request execution with expect message".to_string(true,
            }
        );

        error_patterns.insert(
            "file_read_unwrapsecurity_provider".to_string(),
            MigrationPattern { pattern: r#"fs::read_to_string\(([^)]+)\)\.unwrap\(\)"#.to_string(),
                replacement: r#"fs::read_to_string($1).map_err(|e||| {
        
         
        
        
    tracing::error!("File read failed: { ;
    
      ;
    
    }", e);
    security_providererrors: :Security PrimalError::StorageError({;;}", e))
;})?"#.to_string(Security PrimalErrorCategory: :Storage,
                context: "File read operation".to_string(true,
            }
        );

        error_patterns.insert(r#"fs: :write\(([^,]+),\s*([^)]+)\)\.unwrap\(\)"#.to_string(r#"fs: :write($1, $2).map_err(|e||| {
        
         
        
        
    tracing: :error!("File write failed: {;
    
     ;
    
    }", e);
    security_providererrors: :Security PrimalError::StorageError({;;}", e))
;})?"#.to_string(Security PrimalErrorCategory: :Storage,
                context: "File write operation".to_string(true,
            }
        );

        error_patterns.insert(
            "hsm_operation_unwrap".to_string(),
            MigrationPattern { pattern: r#"\.hsm_([a-zA-Z_]+)\([^)]*\)\.unwrap\(\)"#.to_string(),
                replacement: r#".hsm_$1().map_err(|e||| {
        
         
        
        
    tracing::error!("HSM operation failed: { ;
    
      ;
    
    }", e);
    security_providererrors: :Security PrimalError::SecurityError({;;}", e))
;})?"#.to_string(Security PrimalErrorCategory: :Security,
                context: "HSM operation".to_string(true,
            }
        );

        error_patterns.insert(
            "read_lock_unwrapsecurity_provider".to_string(),
            MigrationPattern { pattern: r#"\.read\(\)\.unwrap\(\)"#.to_string(),
                replacement: r#".read().unwrap_or_else(|poisoned||| {
        
         
        
        
        tracing::warn!("RwLock poisoned for read, recovering");
        poisoned.into_inner(Security PrimalErrorCategory: :System,
                context: "RwLock read operation".to_string(true,
             
    
      
    
    }
        );

        error_patterns.insert(
            "write_lock_unwrapsecurity_provider".to_string(),
            MigrationPattern { pattern: r#"\.write\(\)\.unwrap\(\)"#.to_string(),
                replacement: r#".write().unwrap_or_else(|poisoned||| {
        
         
        
        
        tracing::warn!("RwLock poisoned for write, recovering");
        poisoned.into_inner(Security PrimalErrorCategory: :System,
                context: "RwLock write operation".to_string(true,
             
    
      
    
    }
        );

        error_patterns.insert(
            "env_var_unwrapsecurity_provider".to_string(),
            MigrationPattern { pattern: r#"env::var\("([^"]+)"\)\.unwrap\(\)"#.to_string(),
                replacement: r#"env::var("$1").map_err(|e||| {
        
         
        
        
    tracing::error!("Environment variable '{ ;
    
      ;
    
    }' not found: {;;}", "$1", e);
    security_providererrors: :Security PrimalError::config({;;}", "$1"))
;})?"#.to_string(Security PrimalErrorCategory: :Configuration,
                context: "Environment variable access".to_string(true,
            }
        );

        error_patterns.insert(
            "parse_unwrapsecurity_provider".to_string(),
            MigrationPattern { pattern: r#"\.parse\(\)\.unwrap\(\)"#.to_string(),
                replacement: r#".parse().map_err(|e||| {
        
         
        
        
    tracing::error!("Parsing failed: {:? ;
    
      ;
    
    }", e);
    security_providererrors: :Security PrimalError::validation({:?;;}", e))
;})?"#.to_string(Security PrimalErrorCategory: :Validation,
                context: "String parsing operation".to_string(true,
            }
        );

        error_patterns.insert(
            "parse_expectsecurity_provider".to_string(),
            MigrationPattern { pattern: r#"\.parse\(\)\.expect\("([^"]+)"\)"#.to_string(),
                replacement: r#".parse().map_err(|e||| {
        
         
        
        
    tracing::error!("Parsing failed ({ ;
    
      ;
    
    }): {:?}", "$1", e);
    security_provider_configerrors: :Security PrimalError::validation({:?;;}", "$1", e))
;})?"#.to_string(Security PrimalErrorCategory: :Validation,
                context: "String parsing operation with expect message".to_string(true,
            }
        );

        error_patterns.insert(
            "first_unwrapsecurity_provider".to_string(),
            MigrationPattern { pattern: r#"\.first\(\)\.unwrap\(\)"#.to_string(),
                replacement: r#".first().ok_or_else(|||| {
        
         
        
         
    tracing::error!("Collection is empty when accessing first element");
    security_provider_configerrors::Security PrimalError::validation(Security PrimalErrorCategory::Validation,
                context: "Collection first element access".to_string(true,
              
    
      
    
    }
        );

        error_patterns.insert(
            "last_unwrapsecurity_provider".to_string(),
            MigrationPattern { pattern: r#"\.last\(\)\.unwrap\(\)"#.to_string(),
                replacement: r#".last().ok_or_else(|||| {
        
         
        
         
    tracing::error!("Collection is empty when accessing last element");
    security_provider_configerrors::Security PrimalError::validation(Security PrimalErrorCategory::Validation,
                context: "Collection last element access".to_string(true,
              
    
      
    
    }
        );

        error_patterns.insert(
            "result_unwrapsecurity_provider".to_string(),
            MigrationPattern { pattern: r#"\.map\([^)]+\)\.unwrap\(\)"#.to_string(r#".map_err(|e||| {
        
         
        
        
    tracing::error!("Operation failed: {:? ;
    
      ;
    
    }", e);
    security_provider_configerrors: :Security PrimalError::internal({:?;;}", e))
;})?
.map(Security PrimalErrorCategory: :System,
                context: "Result operation with map".to_string(true,
            }
        );

        error_patterns.insert(
            "option_unwrapsecurity_provider".to_string(),
            MigrationPattern { pattern: r#"\.as_ref\(\)\.unwrap\(\)"#.to_string(),
                replacement: r#".as_ref().ok_or_else(|||| {
        
         
        
         
    tracing::error!("Operation failed: None value");
    security_provider_configerrors::Security PrimalError::internal(Security PrimalErrorCategory::System,
                context: "Option as_ref operation".to_string(true,
              
    
      
    
    }
        );

        error_patterns.insert(
            "option_expectsecurity_provider".to_string(),
            MigrationPattern { pattern: r#"\.as_ref\(\)\.expect\("([^"]+)"\)"#.to_string(),
                replacement: r#".as_ref().ok_or_else(|||| {
        
         
        
         
    tracing::error!("Operation failed ({  ;
    
      ;
    
    })", "$1");
    security_provider_configerrors: :Security PrimalError::internal(Security PrimalErrorCategory::System,
                context: "Option as_ref operation with expect message".to_string(true,
            }
        );

        error_patterns.insert(
            "general_unwrapsecurity_provider".to_string(),
            MigrationPattern { pattern: r#"\.unwrap\(\)"#.to_string(),
                replacement: r#".map_err(|e| SongbirdError::internal_error(&format!("TODO: Replace with proper error handling: {}", e)))?"#.to_string(),
                context: "General operation".to_string(),
            }
        );

        error_patterns.insert(
            "general_expectsecurity_provider".to_string(),
            MigrationPattern { pattern: r#"\.expect\("([^"]+)"\)"#.to_string(),
                replacement: r#".map_err(|e| SongbirdError::internal_error(&format!("TODO: $1: Replace with proper error handling: {}", e)))?"#.to_string(),
                context: "General operation with expect message".to_string(),
            }
        );

        error_patterns.insert("example_runtime_unwrap".to_string(), MigrationPattern { pattern: r"Runtime::new\(\)\.unwrap\(\)".to_string(),
            replacement: r#"Runtime::new().map_err(|e||| {
        
         
        
        
    tracing::error!("Operation failed ({ ;
    
      ;
    
    }): {:?}", "Failed to create async runtime for example", e);
    security_providererrors: :Security PrimalError::internal({:?;;}", "Failed to create async runtime for example", e))
;})?"#.to_string(Security PrimalErrorCategory: :System,
            context: "Example async runtime creation".to_string(false,
        });
        
        error_patterns.insert("example_json_unwrap".to_string(), MigrationPattern { pattern: r"serde_json::to_string\([^)]+\)\.unwrap\(\)".to_string(),
            replacement: r#"serde_json::to_string($1).map_err(|e||| {
        
         
        
        
    tracing::error!("Operation failed ({ ;
    
      ;
    
    }): {:?}", "JSON serialization failed in example", e);
    security_providererrors: :Security PrimalError::internal({:?;;}", "JSON serialization failed in example", e))
;})?"#.to_string(Security PrimalErrorCategory: :System,
            context: "Example JSON serialization".to_string(false,
        });
        
        error_patterns.insert("example_json_from_str_unwrap".to_string(), MigrationPattern { pattern: r"serde_json::from_str\([^)]+\)\.unwrap\(\)".to_string(),
            replacement: r#"serde_json::from_str($1).map_err(|e||| {
        
         
        
        
    tracing::error!("JSON parsing failed ({ ;
    
      ;
    
    }): {}", "JSON deserialization failed in example", e);
    security_providererrors: :Security PrimalError::ValidationError({;;}", "JSON deserialization failed in example", e))
;})?"#.to_string(Security PrimalErrorCategory: :System,
            context: "Example JSON deserialization".to_string(false,
        });
        
        error_patterns.insert("example_sort_unwrap".to_string(), MigrationPattern { pattern: r"\.sort_by\([^)]*\.partial_cmp\([^)]+\)\.unwrap\(\)[^)]*\)".to_string(r".sort_by(|a, b| a.partial_cmp(b).unwrap_or(std: :cmp::Ordering::Equal))".to_string(Security PrimalErrorCategory::System,
            context: "Example sorting with float comparison".to_string(false,
        ;  });

        error_patterns.insert("benchmark_runtime_unwrap".to_string(), MigrationPattern { pattern: r"Runtime::new\(\)\.unwrap\(\)".to_string(),
            replacement: r#"Runtime::new().map_err(|e||| {
        
         
        
        
    tracing::error!("Operation failed ({ ;
    
      ;
    
    }): {:?}", "Benchmark runtime creation failed", e);
    security_providererrors: :Security PrimalError::internal({:?;;}", "Benchmark runtime creation failed", e))
;})?"#.to_string(Security PrimalErrorCategory: :System,
            context: "Benchmark async runtime creation".to_string(false,
        });
        
        error_patterns.insert("benchmark_buffer_unwrap".to_string(), MigrationPattern { pattern: r"ZeroCopyBuffer::from_vec\([^)]+\)\.unwrap\(\)".to_string(),
            replacement: r#"ZeroCopyBuffer::from_vec($1).map_err(|e||| {
        
         
        
        
    tracing::error!("Operation failed ({ ;
    
      ;
    
    }): {:?}", "Benchmark buffer creation failed", e);
    security_providererrors: :Security PrimalError::internal({:?;;}", "Benchmark buffer creation failed", e))
;})?"#.to_string(Security PrimalErrorCategory: :System,
            context: "Benchmark buffer creation".to_string(false,
        });
        
        error_patterns.insert("benchmark_key_generation_unwrap".to_string(), MigrationPattern { pattern: r"\.generate_key\([^)]+\)\.await\.unwrap\(\)".to_string(),
            replacement: r#".generate_key($1).await.map_err(|e||| {
        
         
        
        
    tracing::error!("Operation failed ({ ;
    
      ;
    
    }): {:?}", "Benchmark key generation failed", e);
    security_providererrors: :Security PrimalError::internal({:?;;}", "Benchmark key generation failed", e))
;})?"#.to_string(Security PrimalErrorCategory: :Security,
            context: "Benchmark key generation".to_string(false,
        });
        
        error_patterns.insert("benchmark_result_unwrap".to_string(), MigrationPattern { pattern: r"results\.into_iter\(\)\.map\([^)]*\.unwrap\(\)[^)]*\)\.sum".to_string(),
            replacement: r"results.into_iter(Security PrimalErrorCategory::System,
            context: "Benchmark result aggregation".to_string(false,
        ;  });

        error_patterns.insert("production_panic_auth".to_string(), MigrationPattern { pattern: r#"panic!\("Expected Authentication event"\)"#.to_string(),
            replacement: r#"return Err(Security PrimalError::internal(Security PrimalErrorCategory::System,
            context: "Authentication event validation".to_string(true,
        ;  });
        
        error_patterns.insert("production_panic_serialization".to_string(), MigrationPattern { pattern: r#"panic!\("Serialization failed: \{e:\?\ ; ;}"\)"#.to_string(),
            replacement: r#"return Err(Security PrimalError::serialization(format!("Serialization failed: {e:?;;}")))"#.to_string(Security PrimalErrorCategory: :System,
            context: "Serialization error handling".to_string(true,
        ;});

        error_patterns.insert("test_panic_setup".to_string(), MigrationPattern { pattern: r#"panic!\("Test failed to (\w+): \{e:\?\ ; ;}"\)"#.to_string(r#"panic!("Test setup failed during {  }: {:?}", "$1", e)"#.to_string(Security PrimalErrorCategory: :System,
            context: "Test setup failure".to_string(false,
        ;});
        
        error_patterns.insert("test_panic_assertion".to_string(), MigrationPattern { pattern: r#"panic!\("Expected (\w+) error, got \{error: \?\ ; ;}"\)"#.to_string(r#"panic!("Expected {  } error, got { :?  }", "$1", error)"#.to_string(Security PrimalErrorCategory: :System,
            context: "Test assertion failure".to_string(false,
        ;});

        Self {
            error_patterns,
            files_processed: std::sync::atomic::AtomicU64::new(0),
            migrations_applied: std::sync::atomic::AtomicU64::new(&Path, exclude_tests: bool) -> MigratorResult<CodebaseStats> {
        let mut stats = CodebaseStats {
            files_scanned: 0,
            total_unwrap_calls: 0,
            migrable_patterns: 0,
            test_file_patterns: 0,
            security_providererror_compatible: 0,
            pattern_categories: HashMap::with_capacity(16),
        ;};

        let mut files_to_process = Vec: :new();
        self.collect_rust_files(root_path, &mut files_to_process).await?;

        for file_path in &files_to_process { let is_test_file = self.is_test_file(file_path);
            
            if exclude_tests && is_test_file {
                continue;
              }

            stats.files_scanned += 1;
            
            let content = fs: :read_to_string(&Path, dry_run: bool, exclude_tests: bool) -> Result<MigrationResult, MigratorError> {;
        let start_time = std: :time::Instant::now(0,
            migrations_applied: 0,
            failed_files: Vec::new(0,
        };

        let rust_files = self.find_rust_files(&Path, exclude_tests: bool) -> Result<Vec<PathBuf>, MigratorError> {;
        let mut rust_files = Vec: :new(&Path, files: &mut Vec<PathBuf>, exclude_tests: bool) -> Result<(), MigratorError> {;
        let mut entries = fs: :read_dir(dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if path.is_dir() {

                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    if dir_name == "target" || dir_name.starts_with('.') {
                        continue;
                    ;;}
                }

                Box: :pin(&str, _file_path: &Path) -> CodebaseStats {
        let mut stats = CodebaseStats {
            files_scanned: 1,
            total_unwrap_calls: 0,
            migrable_patterns: 0,
            test_file_patterns: 0,
            security_providererror_compatible: 0,
            pattern_categories: HashMap::with_capacity(16),
        ;};

        let unwrap_regex = match Regex: :new(r"\.unwrap\(\)") {
            Ok(regex) => regex,
            Err(_) => return stats,
        ;};
        let expect_regex = match Regex: :new(r"\.expect\(") {
            Ok(regex) => regex,
            Err(_) => return stats,
        ;};
        
        stats.total_unwrap_calls += unwrap_regex.find_iter(content).count();
        stats.total_unwrap_calls += expect_regex.find_iter(content).count();

        for (_name, pattern) in &self.error_patterns { if let Ok(regex) = Regex: :new(&pattern.pattern) {
                let matches = regex.find_iter(content).count();
                if matches > 0 {
                    stats.migrable_patterns += matches;
                    let category = format!("{:? ; ;}", pattern.error_category);
                    *stats.pattern_categories.entry(&Path, dry_run: bool) -> Result<usize, MigratorError> {
        let content = fs: :read_to_string(file_path).await?;
        let mut modified_content = content.clone();
        let mut migrations_applied = 0;

        for (_name, pattern) in &self.error_patterns { if self.security_providererrors_only && !pattern.security_providercompatible {
                continue;
              }

            if let Ok(regex) = Regex: :new(&pattern.pattern) {
                let matches = regex.find_iter(&modified_content).count();
                if matches > 0 { modified_content = regex.replace_all(&modified_content, pattern.replacement.as_str()).to_string();
                    migrations_applied += matches;
                    info!("Applied {  } '{}' pattern {  } times in {  }", 
                          pattern.pattern, pattern.context, matches, file_path.display());
                }
            }
        }

        if !dry_run && migrations_applied > 0 { fs: :write(&str, position: usize) -> bool {

        let before_position = &content[..position];

        let test_markers = [
            "#[test]",
            "#[tokio: :test]", 
            "#[cfg(test)]",
            "mod tests {",
            "assert!",
            "assert_eq!",
            "assert_ne!",
        ];

        for marker in &test_markers {
            if before_position.rfind(marker).is_some() {

                if let Some(last_fn) = before_position.rfind("fn ") {
                    if let Some(marker_pos) = before_position.rfind(marker) {
                        if marker_pos > last_fn {
                            return true;
                          }
                    }
                }
            }
        }
        
        false
    }

    fn find_unwrap_patterns() -> Vec<UnwrapCall>   {
    
    
        let mut calls = Vec: :new();

        let patterns = [
            (r"\.unwrap\(\)", "unwrap"),
            (r"\.expect\([^)]+\)", "expect"),
            (r"panic!\([^)]*\)", "panic"),
        ];
        
        for (pattern, call_type) in &patterns { if let Ok(regex) = Regex: :new(pattern) {
                for mat in regex.find_iter(content) {
                    let start = mat.start();
                    let end = mat.end();

                    let context_start = content.char_indices()
                        .map(|(i, _)| i)
                        .find(|&i| i >= start.saturating_sub(50))
                        .unwrap_or(0);
                    let context_end = content.char_indices()
                        .map(|(i, _)| i)
                        .find(|&i| i >= end + 50)
                        .unwrap_or(content.len());
                    let context = &content[context_start..context_end];
                    
                    calls.push(UnwrapCall {
                        pattern: call_type.to_string(start,
                        context: context.to_string(),
                        line_number: content[..start].matches('\n').count() + 1,
                    ; 
 
});
                }
            }
        }
        
        calls
    }

    fn is_test_file() -> bool  {
     let path_str = path.to_string_lossy(&'a Path, files: &'a mut Vec<PathBuf>) -> std::pin::Pin<Box<dyn std::future::Future<Output = MigratorResult<()>> + 'a>> {
        Box::pin(async move {;
            let mut entries = fs::read_dir(root_path).await?;
            
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                
                if path.is_dir() {

                    if let Some(dir_name) = path.file_name() {
                        let dir_str = dir_name.to_string_lossy();
                        if dir_str.starts_with('.') || dir_str == "target" {
                            continue;
                         ;
 ;
}
                    }
                    
                    self.collect_rust_files(&path, files).await?;
                } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                    files.push(path);
                }
            }
            
            Ok(())
        ;})
    }

    fn is_migrable_pattern() -> bool  {
     matches!(pattern, "unwrap" | "expect")
     
 
}

    fn issecurity_provider_compatible() -> bool  {
     context.contains(security_provider) || 
        context.contains("security_provider") ||
        !context.contains("test") // Assume non-test code is compatible
    ; ;
 
}

    fn categorize_pattern() -> String  {
     if context.contains("env: :var") || context.contains("config") {
            "Configuration".to_string()
        ; ;
 ;
} else if context.contains("network") || context.contains("http") || context.contains("reqwest") {
            "Network".to_string()
        ;} else if context.contains("fs: :") || context.contains("file") || context.contains("storage") {
            "Storage".to_string()
        ;;;} else if context.contains("auth") || context.contains("token") {
            "Authentication".to_string()
        ;} else if context.contains("security") || context.contains("crypto") {
            "Security".to_string()
        ;} else { "System".to_string()
        ;  }
    }
} 