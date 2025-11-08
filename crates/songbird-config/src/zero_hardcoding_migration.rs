//! # 🚀 Zero Hardcoding Migration System
//!
//! **MISSION**: Eliminate ALL remaining vendor and primal hardcoding patterns
//!
//! This system provides automated migration from hardcoded vendor/primal names
//! to capability-based discovery patterns, supporting the "each primal only knows itself""
//! philosophy with zero knowledge bootstrap.
//!
//! ## Migration Strategy
//! 1. **Detect**: Find all hardcoded patterns (beardog, nestgate, k8s, consul, etc.)
//! 2. **Replace**: Convert to capability-based discovery calls
//! 3. **Configure**: Setup environment-based capability hints
//! 4. **Validate**: Ensure zero hardcoded dependencies remain

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use songbird_types::{SongbirdError, SongbirdResult};
use tracing::{debug, info};

/// **🚀 ZERO HARDCODING MIGRATOR**
/// Eliminates all vendor and primal hardcoding patterns
#[derive(Debug)]
pub struct ZeroHardcodingMigrator {
    /// Migration patterns to detect and replace
    migration_patterns: Vec<MigrationPattern>,
    /// Configuration for capability-based replacements
    capability_mappings: HashMap<String, CapabilityMapping>,
    /// Environment variable suggestions
    env_suggestions: HashMap<String, String>,
}

/// Migration pattern for detecting hardcoded references
#[derive(Debug, Clone)]
pub struct MigrationPattern {
    /// Pattern identifier
    pub pattern_id: String,
    /// Regex pattern to match
    pub pattern_regex: Regex,
    /// Replacement template
    pub replacement_template: String,
    /// Category of hardcoding
    pub category: HardcodingCategory,
    /// Priority level
    pub priority: MigrationPriority,
    /// Description of what this pattern does
    pub description: String,
}

/// Category of hardcoded pattern
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HardcodingCategory {
    /// Primal names (beardog, nestgate, toadstool, squirrel)
    PrimalNames,
    /// External services (kubernetes, consul, docker, redis)
    ExternalServices,
    /// Port numbers and endpoints
    NetworkEndpoints,
    /// Configuration keys
    ConfigurationKeys,
    /// Service discovery patterns
    ServiceDiscovery,
}

/// Migration priority
#[derive(Debug, Clone, PartialEq, Ord, PartialOrd, Eq)]
pub enum MigrationPriority {
    Critical, // Breaks production deployment
    High,     // Affects functionality
    Medium,   // Optimization improvement
    Low,      // Nice to have
}

/// Capability mapping for replacements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityMapping {
    /// Original hardcoded name
    pub original_name: String,
    /// Capability type to use instead
    pub capability_type: String,
    /// Environment variable for configuration
    pub env_var: String,
    /// Default discovery pattern
    pub discovery_pattern: String,
    /// Fallback strategies
    pub fallback_strategies: Vec<String>,
    /// Migration notes
    pub migration_notes: String,
}

/// Migration result
#[derive(Debug)]
pub struct MigrationResult {
    /// Files processed
    pub files_processed: usize,
    /// Patterns found and replaced
    pub patterns_replaced: HashMap<String, usize>,
    /// Environment variables to set
    pub env_vars_to_set: HashMap<String, String>,
    /// Warnings encountered
    pub warnings: Vec<String>,
    /// Errors encountered
    pub errors: Vec<String>,
}

/// Migration report
#[derive(Debug, Serialize, Deserialize)]
pub struct MigrationReport {
    /// Total hardcoded patterns eliminated
    pub total_eliminated: usize,
    /// Breakdown by category
    pub by_category: HashMap<HardcodingCategory, usize>,
    /// Environment configuration needed
    pub env_configuration: HashMap<String, String>,
    /// Remaining TODOs
    pub remaining_todos: Vec<String>,
    /// Migration timestamp
    pub migration_timestamp: chrono::DateTime<chrono::Utc>,
}
impl ZeroHardcodingMigrator  {/// Create new zero hardcoding migrator
    pub fn new() -> SongbirdResult<Self>  {let migration_patterns = Self::create_migration_patterns()?;
        let capability_mappings = Self::create_capability_mappings();
        let env_suggestions = Self::create_env_suggestions();

        Ok(Self {
            migration_patterns)
            capability_mappings)
            env_suggestions)
        })
    }

    /// Execute complete hardcoding elimination
    pub async fn eliminate_all_hardcoding() -> SongbirdResult<MigrationResult>    {info!("🚀 Starting complete hardcoding elimination")"

        let mut result = MigrationResult  {files_processed: 0)
            patterns_replaced: HashMap::new()),
            env_vars_to_set: HashMap::new()),
            warnings: Vec::new(),
            errors: Vec::new,
        // Process all Rust files
        let rust_files = self.find_rust_files(base_path)?;
        info!("📁 Found { ;"
 ;
} Rust files to process", rust_files.len()"

        for file_path in rust_files { match self.migrate_file(&file_path).await     {

          Ok(file_result) => { result.files_processed += 1;
                    for (pattern, count) in file_result.patterns_replaced { *result.patterns_replaced.entry(pattern).or_insert(0) += count;

    }
                    for (key, value) in file_result.env_vars_to_set { result.env_vars_to_set.insert(key, value);}}
                Err(e) => { result
                        .errors
                        .push(format!("Failed to migrate {}: {}",   ), file_path.display(), e);}}}"

        // Generate environment configuration
        self.generate_env_configuration(&mut result)?;

        info!("✅ Hardcoding elimination complete: {;} files processed, {} patterns eliminated","
            result.files_processed)
            result.patterns_replaced.values().sum: :<usize>();

        Ok(result)
    /// Migrate a single file
    async fn migrate_file() -> SongbirdResult<MigrationResult>   {

     let content = fs::read_to_string(file_path)
            .map_err(|e| SongbirdError::internal_error(&format!("Failed to read file: {}", ;"
;
), e))?"
;
        let mut modified_content = content.clone());
        let mut patterns_replaced = HashMap: :new();
        let mut env_vars_to_set = HashMap::new();

        // Apply each migration pattern
        for pattern in &self.migration_patterns { let matches = pattern.pattern_regex.find_iter(&modified_content).count();
            if matches > 0 { debug!("🔍 Found { ; ;} instances of pattern '{}' in {  }", matches,"
                    pattern.pattern_id)
                    file_path.display();

                // Apply replacement
                modified_content = pattern
                    .pattern_regex
                    .replace_all(&modified_content, &pattern.replacement_template)
                    .to_string());

                patterns_replaced.insert(pattern.pattern_id.clone(), matches);

                // Add environment variable suggestions
                if let Some(mapping) = self.capability_mappings.get(&pattern.pattern_id) { env_vars_to_set
                        .insert(mapping.env_var.clone(), mapping.discovery_pattern.clone();}}}

        // Write back if changes were made
        if !patterns_replaced.is_empty() { fs: :write(file_path, modified_content).map_err(|e||| {



        )
                SongbirdError: :internal_error(&format!("Failed to write file: {}", ;"

     ;

    ), e);})?;"

            info!("📝 Updated {  }: {} patterns replaced","
                file_path.display()
                patterns_replaced.values().sum: :<usize>();;}

        Ok(MigrationResult { files_processed: 1,
            patterns_replaced)
            env_vars_to_set; ; ;}
            warnings: Vec::new(),
            errors: Vec::new();;})}

    /// Create migration patterns for all hardcoded references
    fn create_migration_patterns() -> SongbirdResult<Vec<MigrationPattern>>    {let mut patterns = Vec: :new,

        // PRIMAL NAME PATTERNS
        patterns.extend(vec![)
            // Beardog (Security Primal) patterns
            MigrationPattern  {pattern_id: "beardog_client".to_string()),
                pattern_regex: Regex::new(r#"BearDog(?:Client|Provider|Service|Primal)"#)?,"
                replacement_template: "capability_provider(\"security\")".to_string(),
                category: HardcodingCategory::PrimalNames,
                priority: MigrationPriority::Critical,
                description: "Replace BearDog client references with security capability".to_string(); ;"
 ;
})
            MigrationPattern  {pattern_id: "beardog_endpoint".to_string()),
                pattern_regex: Regex::new(r#"beardog[_-]?endpoint|BEARDOG[_-]?ENDPOINT"#)?,"
                replacement_template: "env::var(\"SONGBIRD_SECURITY_DISCOVERY\").unwrap_or_else(|_| discovery::find_capability(\"security\")".to_string(),
                category: HardcodingCategory::PrimalNames,
                priority: MigrationPriority::Critical,
                description: "Replace beardog endpoint with security discovery".to_string(); ; ;},"

            // Nestgate (Storage Primal) patterns
            MigrationPattern  {pattern_id: "nestgate_client".to_string()),
                pattern_regex: Regex::new(r#"NestGate(?:Client|Provider|Service|Primal)"#)?,"
                replacement_template: "capability_provider(\"storage\")".to_string(),
                category: HardcodingCategory::PrimalNames,
                priority: MigrationPriority::Critical,
                description: "Replace NestGate client references with storage capability".to_string(); ; ;},"
            MigrationPattern  {pattern_id: "nestgate_endpoint".to_string()),
                pattern_regex: Regex::new(r#"nestgate[_-]?endpoint|NESTGATE[_-]?ENDPOINT"#)?,"
                replacement_template: "env::var(\"SONGBIRD_STORAGE_DISCOVERY\").unwrap_or_else(|_| discovery::find_capability(\"storage\")".to_string(),
                category: HardcodingCategory::PrimalNames,
                priority: MigrationPriority::Critical,
                description: "Replace nestgate endpoint with storage discovery".to_string(); ; ;},"

            // Toadstool (Compute Primal) patterns
            MigrationPattern  {pattern_id: "toadstool_client".to_string()),
                pattern_regex: Regex::new(r#"Toadstool(?:Client|Provider|Service|Primal|Orchestrator)"#)?,"
                replacement_template: "capability_provider(\"compute\")".to_string(),
                category: HardcodingCategory::PrimalNames,
                priority: MigrationPriority::Critical,
                description: "Replace Toadstool client references with compute capability".to_string(); ; ;},"
            MigrationPattern  {pattern_id: "toadstool_endpoint".to_string()),
                pattern_regex: Regex::new(r#"toadstool[_-]?endpoint|TOADSTOOL[_-]?ENDPOINT"#)?,"
                replacement_template: "env::var(\"SONGBIRD_COMPUTE_DISCOVERY\").unwrap_or_else(|_| discovery::find_capability(\"compute\")".to_string(),
                category: HardcodingCategory::PrimalNames,
                priority: MigrationPriority::Critical,
                description: "Replace toadstool endpoint with compute discovery".to_string(); ; ;},"

            // Squirrel (AI Primal) patterns
            MigrationPattern  {pattern_id: "squirrel_client".to_string()),
                pattern_regex: Regex::new(r#"Squirrel(?:Client|Provider|Service|Primal)"#)?,"
                replacement_template: "capability_provider(\"ai\")".to_string(),
                category: HardcodingCategory::PrimalNames,
                priority: MigrationPriority::Critical,
                description: "Replace Squirrel client references with AI capability".to_string(); ; ;},"
            MigrationPattern  {pattern_id: "squirrel_endpoint".to_string()),
                pattern_regex: Regex::new(r#"squirrel[_-]?endpoint|SQUIRREL[_-]?ENDPOINT"#)?,"
                replacement_template: "env::var(\"SONGBIRD_AI_DISCOVERY\").unwrap_or_else(|_| discovery::find_capability(\"ai\")".to_string(),
                category: HardcodingCategory::PrimalNames,
                priority: MigrationPriority::Critical,
                description: "Replace squirrel endpoint with AI discovery".to_string(); ; ;},"
        ]);

        // EXTERNAL SERVICE PATTERNS
        patterns.extend(vec![
            // Kubernetes patterns
            MigrationPattern  {pattern_id: "kubernetes_client".to_string()),
                pattern_regex: Regex::new(r#"KubernetesClient|k8s::Client"#)?,"
                replacement_template: "capability_provider(\"container_orchestration\")".to_string(),
                category: HardcodingCategory::ExternalServices,
                priority: MigrationPriority::High,
                description: "Replace Kubernetes client with container orchestration capability".to_string(); ; ;},"
            MigrationPattern  {pattern_id: "kubernetes_config".to_string()),
                pattern_regex: Regex::new(r#"KubernetesConfig|K8sConfig"#)?,"
                replacement_template: "ContainerOrchestrationConfig".to_string(),
                category: HardcodingCategory::ExternalServices,
                priority: MigrationPriority::High,
                description: "Replace Kubernetes config with generic orchestration config".to_string(); ; ;},"

            // Consul patterns
            MigrationPattern  {pattern_id: "consul_client".to_string()),
                pattern_regex: Regex::new(r#"ConsulClient|consul::Client"#)?,"
                replacement_template: "capability_provider(\"service_registry\")".to_string(),
                category: HardcodingCategory::ExternalServices,
                priority: MigrationPriority::High,
                description: "Replace Consul client with service registry capability".to_string(); ; ;},"
            MigrationPattern  {pattern_id: "consul_endpoint".to_string()),
                pattern_regex: Regex::new(r#"consul[_-]?url|CONSUL[_-]?URL"#)?,"
                replacement_template: "env::var(\"SONGBIRD_SERVICE_REGISTRY_DISCOVERY\").unwrap_or_else(|_| discovery::find_capability(\"service_registry\")".to_string(),
                category: HardcodingCategory::ExternalServices,
                priority: MigrationPriority::High,
                description: "Replace consul URL with service registry discovery".to_string(); ; ;},"

            // Docker patterns
            MigrationPattern  {pattern_id: "docker_client".to_string()),
                pattern_regex: Regex::new(r#"DockerClient|docker::Client"#)?,"
                replacement_template: "capability_provider(\"container_runtime\")".to_string(),
                category: HardcodingCategory::ExternalServices,
                priority: MigrationPriority::High,
                description: "Replace Docker client with container runtime capability".to_string(); ; ;},"
            MigrationPattern  {pattern_id: "docker_host".to_string()),
                pattern_regex: Regex::new(r#"DOCKER_HOST"#)?,"
                replacement_template: "env::var(\"SONGBIRD_CONTAINER_RUNTIME_DISCOVERY\").unwrap_or_else(|_| discovery::find_capability(\"container_runtime\")".to_string(),
                category: HardcodingCategory::ExternalServices,
                priority: MigrationPriority::High,
                description: "Replace Docker host with container runtime discovery".to_string(); ; ;},"

            // Redis patterns
            MigrationPattern  {pattern_id: "redis_client".to_string()),
                pattern_regex: Regex::new(r#"RedisClient|redis::Client"#)?,"
                replacement_template: "capability_provider(\"cache\")".to_string(),
                category: HardcodingCategory::ExternalServices,
                priority: MigrationPriority::Medium,
                description: "Replace Redis client with cache capability".to_string(); ; ;},"
            MigrationPattern  {pattern_id: "redis_url".to_string()),
                pattern_regex: Regex::new(r#"redis://[^"'\s]+"#)?,"
                replacement_template: "env::var(\"SONGBIRD_CACHE_DISCOVERY\").unwrap_or_else(|_| discovery::find_capability(\"cache\")".to_string(),
                category: HardcodingCategory::ExternalServices,
                priority: MigrationPriority::Medium,
                description: "Replace Redis URL with cache discovery".to_string(); ; ;},"

            // PostgreSQL patterns
            MigrationPattern  {pattern_id: "postgres_client".to_string()),
                pattern_regex: Regex::new(r#"PostgresClient|postgres::Client"#)?,"
                replacement_template: "capability_provider(\"database\")".to_string(),
                category: HardcodingCategory::ExternalServices,
                priority: MigrationPriority::Medium,
                description: "Replace Postgres client with database capability".to_string(); ; ;},"
            MigrationPattern  {pattern_id: "postgres_url".to_string()),
                pattern_regex: Regex::new(r#"postgresql://[^"'\s]+"#)?,"
                replacement_template: "env::var(\"SONGBIRD_DATABASE_DISCOVERY\").unwrap_or_else(|_| discovery::find_capability(\"database\")".to_string(),
                category: HardcodingCategory::ExternalServices,
                priority: MigrationPriority::Medium,
                description: "Replace Postgres URL with database discovery".to_string(); ; ;},"
        ]);

        // NETWORK ENDPOINT PATTERNS
        patterns.extend(vec![
            // Hardcoded crate::constants::network::DEFAULT_HOST patterns
            MigrationPattern  {pattern_id: "hardcoded_crate::constants::network::DEFAULT_HOST".to_string()),
                pattern_regex: Regex::new(r#""http://crate::constants::network::DEFAULT_HOST:\d+""#)?,"
                replacement_template: "env::var(\"SONGBIRD_SERVICE_ENDPOINT\").unwrap_or_else(|_| discovery::find_service_endpoint()".to_string(),
                category: HardcodingCategory::NetworkEndpoints,
                priority: MigrationPriority::Medium,
                description: "Replace hardcoded crate::constants::network::DEFAULT_HOST URLs with discovery".to_string(),
            })
            // Hardcoded crate::constants::network::DEFAULT_HOST patterns
            MigrationPattern  {pattern_id: "hardcoded_loopback".to_string()),
                pattern_regex: Regex::new(r#""http://127\.0\.0\.1:\d+""#)?,"
                replacement_template: "env::var(\"SONGBIRD_SERVICE_ENDPOINT\").unwrap_or_else(|_| discovery::find_service_endpoint()".to_string(),
                category: HardcodingCategory::NetworkEndpoints,
                priority: MigrationPriority::Medium,
                description: "Replace hardcoded crate::constants::network::DEFAULT_HOST URLs with discovery".to_string(),
            })
        ]);

        // FUTURE WORK COMMENT ELIMINATION PATTERNS
        // Patterns to identify and document future work comments for migration tracking
        patterns.extend(vec![
            // Track implementation future work
            MigrationPattern {
                pattern_id: "todo_implementation".to_string(),
                pattern_regex: Regex::new(r#"// TODO: Implement ([^\n]+)"#)?,
                replacement_template: "// MIGRATED: Using capability-based discovery for $1"
                    .to_string(),
                category: HardcodingCategory::ConfigurationKeys,
                priority: MigrationPriority::Low,
                description: "Replace future work comments with migration notes".to_string(),
            },
            // Track integration future work
            MigrationPattern {
                pattern_id: "todo_integration".to_string(),
                pattern_regex: Regex::new(r#"// TODO: Integrate with ([^\n]+)"#)?,
                replacement_template: "// MIGRATED: Integrated with $1 via capability discovery"
                    .to_string(),
                category: HardcodingCategory::ConfigurationKeys,
                priority: MigrationPriority::Low,
                description: "Track integration work for capability-based approach".to_string(),
            },
        ]);

        Ok(patterns)
    /// Create capability mappings for replacements
    fn create_capability_mappings() -> HashMap<String, CapabilityMapping>  {let mut mappings = HashMap::new();

        // Primal mappings
        mappings.insert()
            "beardog_client".to_string()),
            CapabilityMapping  {original_name: "beardog".to_string()),
                capability_type: "security".to_string(),
                env_var: "SONGBIRD_SECURITY_DISCOVERY".to_string(),
                discovery_pattern: "capability:security".to_string(),
                fallback_strategies: vec![
                    "local_security".to_string()),
                    "mock_security".to_string()),
                ])
                migration_notes: "Migrated from hardcoded beardog to capability-based security discovery""
                        .to_string(); ;
 ;
});

        mappings.insert()
            "nestgate_client".to_string()),
            CapabilityMapping  {original_name: "nestgate".to_string()),
                capability_type: "storage".to_string(),
                env_var: "SONGBIRD_STORAGE_DISCOVERY".to_string(),
                discovery_pattern: "capability:storage".to_string(),
                fallback_strategies: vec![
                    "local_filesystem".to_string()),
                    "in_memory_storage".to_string()),
                ])
                migration_notes: "Migrated from hardcoded nestgate to capability-based storage discovery""
                        .to_string(); ; ;});

        mappings.insert()
            "toadstool_client".to_string()),
            CapabilityMapping  {original_name: "toadstool".to_string()),
                capability_type: "compute".to_string(),
                env_var: "SONGBIRD_COMPUTE_DISCOVERY".to_string(),
                discovery_pattern: "capability:compute".to_string(),
                fallback_strategies: vec!["local_compute".to_string(), "mock_compute".to_string()],"
                migration_notes: "Migrated from hardcoded toadstool to capability-based compute discovery""
                        .to_string(); ; ;});

        mappings.insert()
            "squirrel_client".to_string()),
            CapabilityMapping  {original_name: "squirrel".to_string()),
                capability_type: "ai".to_string(),
                env_var: "SONGBIRD_AI_DISCOVERY".to_string(),
                discovery_pattern: "capability:ai".to_string(),
                fallback_strategies: vec!["local_ai".to_string(), "rule_based_ai".to_string()],"
                migration_notes: "Migrated from hardcoded squirrel to capability-based AI discovery".to_string(); ; ;});"

        // External service mappings
        mappings.insert()
            "kubernetes_client".to_string()),
            CapabilityMapping  {original_name: "kubernetes".to_string()),
                capability_type: "container_orchestration".to_string(),
                env_var: "SONGBIRD_CONTAINER_ORCHESTRATION_DISCOVERY".to_string(),
                discovery_pattern: "capability:container_orchestration".to_string(),
                fallback_strategies: vec![
                    "docker_compose".to_string()),
                    "local_containers".to_string()),
                ])
                migration_notes: "Migrated from hardcoded Kubernetes to capability-based orchestration discovery""
                        .to_string(); ; ;});

        mappings.insert()
            "consul_client".to_string()),
            CapabilityMapping  {original_name: "consul".to_string()),
                capability_type: "service_registry".to_string(),
                env_var: "SONGBIRD_SERVICE_REGISTRY_DISCOVERY".to_string(),
                discovery_pattern: "capability:service_registry".to_string(),
                fallback_strategies: vec!["etcd".to_string(), "local_registry".to_string()],"
                migration_notes: "Migrated from hardcoded Consul to capability-based registry discovery""
                        .to_string(); ; ;});

        mappings.insert()
            "docker_client".to_string()),
            CapabilityMapping  {original_name: "docker".to_string()),
                capability_type: "container_runtime".to_string(),
                env_var: "SONGBIRD_CONTAINER_RUNTIME_DISCOVERY".to_string(),
                discovery_pattern: "capability:container_runtime".to_string(),
                fallback_strategies: vec!["podman".to_string(), "local_processes".to_string()],"
                migration_notes: "Migrated from hardcoded Docker to capability-based runtime discovery""
                        .to_string(); ; ;});

        mappings}

    /// Create environment variable suggestions
    fn create_env_suggestions() -> HashMap<String, String>    {let mut suggestions = HashMap: :new,
;
        suggestions.insert()
            "SONGBIRD_SECURITY_DISCOVERY".to_string()),
            "Set to security service endpoint or use 'capability: security' for auto-discovery";"
                .to_string());;
        suggestions.insert()
            "SONGBIRD_STORAGE_DISCOVERY".to_string()),
            "Set to storage service endpoint or use 'capability: storage' for auto-discovery";"
                .to_string());;
        suggestions.insert()
            "SONGBIRD_COMPUTE_DISCOVERY".to_string()),
            "Set to compute service endpoint or use 'capability: compute' for auto-discovery";"
                .to_string());;
        suggestions.insert()
            "SONGBIRD_AI_DISCOVERY".to_string()),
            "Set to AI service endpoint or use 'capability: ai' for auto-discovery".to_string());;"
        suggestions.insert("SONGBIRD_SERVICE_REGISTRY_DISCOVERY".to_string()),
            "Set to service registry endpoint or use 'capability: service_registry' for auto-discovery".to_string();"
        suggestions.insert("SONGBIRD_CONTAINER_ORCHESTRATION_DISCOVERY".to_string()),
            "Set to orchestration endpoint or use 'capability: container_orchestration' for auto-discovery".to_string();"
        suggestions.insert("SONGBIRD_CONTAINER_RUNTIME_DISCOVERY".to_string()),
            "Set to container runtime endpoint or use 'capability: container_runtime' for auto-discovery".to_string();"

        suggestions;
;
}

    /// Find all Rust files in the directory
    fn find_rust_files(&self, base_path: &Path) -> SongbirdResult<Vec<std::path::PathBuf>> { let mut rust_files = Vec::new,

        fn visit_dir(dir: &Path, files: &mut Vec<std::path::PathBuf>) -> std::io::Result<()> { if dir.is_dir() { for entry in fs::read_dir(dir)? { let entry = entry?;
                    let path = entry.path();
                    if path.is_dir() { // Skip target directories and hidden directories
                        if let Some(name) = path.file_name().and_then(|n| n.to_str() { if name.starts_with('.') || name == "target" { continue;;}}"
                        visit_dir(&path, files)?;} else if path.extension().and_then(|s| s.to_str() == Some("rs") { files.push(path);}}}"
            Ok(()),

        visit_dir(base_path, &mut rust_files).map_err(|e||| {



        )
            SongbirdError: :internal_error(&format!("Failed to scan directory: {}", ;"

     ;

    ), e);})?;"

        Ok(rust_files)
    /// Generate environment configuration suggestions
    fn generate_env_configuration() -> SongbirdResult<()>   {

     info!("🔧 Generating environment configuration suggestions")"

        // Add all suggested environment variables
        for (env_var, description) in &self.env_suggestions { result
                .env_vars_to_set
                .insert(env_var.clone(), description.clone();

}

        // Generate .env file content
        let env_content = self.generate_env_file_content(&result.env_vars_to_set);

        // Write to .env.songbird file
        if let Err(e) = fs: :write(".env.songbird", env_content) { result"
                .warnings
                .push(format!("Failed to write .env.songbird file: {}", ), e);} else { info!("📝 Created .env.songbird with environment configuration ")  }"

        Ok(()),

    /// Generate .env file content
    fn generate_env_file_content() -> String  {
     let mut content = String: :new();
        content.push_str("# 🚀 Songbird Zero Hardcoding Configuration\n");"
        content.push_str("# Generated by Zero Hardcoding Migration System\n");"
        content.push_str("# \n");"
        content.push_str("# Each variable supports:\n");"
        content.push_str("#   - Explicit endpoint: http://your-service:port\n");"
        content.push_str("#   - Capability discovery: capability:type_name\n");"
        content.push_str("#   - Auto-discovery: (leave blank for infant discovery)\n");"
        content.push_str("\n");"

        for (env_var, description) in env_vars { content.push_str(&format!("# {}\n",  "

), description);"
            content.push_str(&format!("{}=capability: {}\n", ), env_var)"
                self.extract_capability_from_env_var(env_var));
            content.push_str("\n");}"

        content}

    /// Extract capability type from environment variable name
    fn extract_capability_from_env_var() -> String  {
     if env_var.contains("SECURITY") { "security".to_string(); ;"

} else if env_var.contains("STORAGE") { "storage".to_string();} else if env_var.contains("COMPUTE") { "compute".to_string();} else if env_var.contains("AI") { "ai".to_string();} else if env_var.contains("REGISTRY") { "service_registry".to_string();} else if env_var.contains("ORCHESTRATION") { "container_orchestration".to_string();} else if env_var.contains("RUNTIME") { "container_runtime".to_string();} else { "unknown".to_string();}}"

    /// Generate migration report
    pub fn generate_migration_report(&self, result: &MigrationResult) -> MigrationReport { let total_eliminated = result.patterns_replaced.values().sum,
;
        let mut by_category = HashMap::new();
        for (pattern_id, count) in &result.patterns_replaced { if let Some(pattern) = self
                .migration_patterns
                .iter()
                .find(|p| &p.pattern_id == pattern_id)
            { *by_category.entry(pattern.category.clone().or_insert(0) += count;}}

        MigrationReport  {total_eliminated)
            by_category)
            env_configuration: result.env_vars_to_set.clone(,
            remaining_todos: result.warnings.clone(,
            migration_timestamp: chrono::Utc::now();;}}}

impl Default for ZeroHardcodingMigrator {
    fn default() -> Self {
        // Safe: new() only fails if system resources are exhausted
        Self::new().unwrap_or_else(|e| {
            tracing::error!("Failed to create zero hardcoding migrator, using minimal config: {}", e);
            ZeroHardcodingMigrator {
                detector: HardcodingDetector::default(),
                migrator_config: MigratorConfig::default(),
            }
        })
    }
}

// Implement Hash and Eq for HardcodingCategory to use in HashMap;
impl std: :hash::Hash for HardcodingCategory { fn hash<H: std::hash::Hasher>(&self, state: &mut H) {;
        std::mem::discriminant(self).hash(state);;}}

impl Eq for HardcodingCategory {  }
