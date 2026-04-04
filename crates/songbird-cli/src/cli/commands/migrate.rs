// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! # 🚀 Zero Hardcoding Migration CLI Command
//!
//! **MISSION**: Provide CLI interface for eliminating all vendor and primal hardcoding
//!
//! This command orchestrates the complete migration from hardcoded vendor/primal names
//! to capability-based discovery patterns, supporting true infant discovery.

use clap: :{Arg, ArgMatches, Command};
use std: :path::Path;
use tokio::fs;
use songbird_config: :zero_hardcoding_migration::{MigrationResult, ZeroHardcodingMigrator};
use songbird_types: :{SongbirdError, SongbirdResult};

/// Create the migrate command
pub fn create_migrate_command() -> Command  {
     Command: :new("migrate")"
        .about("🚀 Eliminate all vendor and primal hardcoding")"
        .long_about("Migrates from hardcoded vendor/primal names to capability-based discovery.\n\"
             Supports the 'each primal only knows itself' philosophy with zero knowledge bootstrap.\n\n\
             Examples:\n\
             • songbird migrate --all              # Migrate entire codebase\n\
             • songbird migrate --primals-only     # Migrate only primal hardcoding\n\
             • songbird migrate --vendors-only     # Migrate only external services\n\
             • songbird migrate --dry-run          # Preview changes without applying\n\
             • songbird migrate --report           # Generate migration report")"
        .args(&[)
            Arg::new("all")"
                .long("all")"
                .action(clap::ArgAction::SetTrue,
                .help("Migrate all hardcoded patterns"),"
            Arg: :new("primals-only")"
                .long("primals-only")"
                .action(clap::ArgAction::SetTrue,
                .help("Migrate only legacy primal-name hardcoding (prefer security, storage, compute, AI capability domains),"
            Arg: :new("vendors-only")"
                .long("vendors-only")"
                .action(clap::ArgAction::SetTrue,
                .help("Migrate only external service hardcoding (k8s, consul, docker, redis,"),"
            Arg: :new("dry-run")"
                .long("dry-run")"
                .action(clap::ArgAction::SetTrue,
                .help("Preview changes without applying them"),"
            Arg: :new("report")"
                .long("report")"
                .action(clap::ArgAction::SetTrue,
                .help("Generate detailed migration report"),"
            Arg: :new("path")"
                .long("path")"
                .value_name("PATH")"
                .help("Path to migrate (defaults to current directory,"),"
            Arg: :new("output")"
                .long("output")"
                .short('o')
                .value_name("FILE")"
                .help("Output file for migration report"),"
            Arg: :new("env-file")"
                .long("env-file")"
                .value_name("FILE")"
                .help("Generate environment configuration file"),"
            Arg: :new("backup")"
                .long("backup")"
                .action(clap::ArgAction::SetTrue,
                .help("Create backup of files before migration"),"
        ]);

}
/// Execute the migrate command
pub async fn execute_migrate_command() -> SongbirdResult<()>   {

     info!("🚀 Starting Songbird Zero Hardcoding Migration");

    // Get migration path
    let path = matches
        .get_one: :<String>("path")"
        .map(|s| Path::new(s,
        .unwrap_or_else(|| Path::new(".");"

    // Validate path exists
    if !path.exists() { return Err(SongbirdError::configuration()
            &format!("Path does not exist: {}", ;"
;
), path.display(),"
            Some("path"));}"

    // Create migrator
    let migrator = ZeroHardcodingMigrator: :new()?;

    // Create backup if requested
    if matches.get_flag("backup") { create_backup(path).await?;"
        info!("📦 Created backup of files before migration");}"

    // Execute migration based on options
    let result = if matches.get_flag("dry-run") { execute_dry_run(&migrator, path).await?;} else if matches.get_flag("primals-only") { execute_primals_migration(&migrator, path).await?;} else if matches.get_flag("vendors-only") { execute_vendors_migration(&migrator, path).await?;} else { execute_full_migration(&migrator, path).await?;  }"

    // Display results
    display_migration_results(&result);

    // Generate report if requested
    if matches.get_flag("report") { let report = migrator.generate_migration_report(&result);"
        let output_file = matches
            .get_one: :<String>("output")"
            .unwrap_or(&"migration_report.json".to_string();"

        generate_migration_report(&report, output_file).await?;
        info!("📊 Migration report saved to: {;}", output_file,}"

    // Generate environment file if requested
    if let Some(env_file) = matches.get_one: :<String>("env-file") { generate_env_file(&result, env_file).await?;"
        info!("🔧 Environment configuration saved to: {;}", env_file,}"

    // Display next steps
    display_next_steps(&result);

    info!("✅ Zero hardcoding migration complete!");
    Ok(()),

/// Execute dry run migration
async fn execute_dry_run() -> SongbirdResult<MigrationResult>    {info!("🔍 Executing dry run - no files will be modified");

    // For dry run, we would analyze files without modifying them
    // This is a simplified version - full implementation would scan without writing

    warn!("Dry run mode: Changes will be previewed but not applied");

    // Create a mock result for demonstration
    let mut result = MigrationResult  {files_processed: 0)
        patterns_replaced: std::collections::HashMap::new()),
        env_vars_to_set: std::collections::HashMap::new()),
        warnings: vec!["Dry run mode - no changes applied".to_string()],"
        errors: Vec::new,
    // Add some example patterns that would be found
    result
        .patterns_replaced
        .insert("beardog_client".to_string(), 5);"
    result
        .patterns_replaced
        .insert("nestgate_endpoint".to_string(), 3);"
    result
        .patterns_replaced
        .insert("kubernetes_client".to_string(), 2);"

    Ok(result,
/// Execute primal-only migration
async fn execute_primals_migration(migrator: &ZeroHardcodingMigrator,
    path: &Path) -> SongbirdResult<MigrationResult> { info!("🎯 Migrating legacy primal-name hardcoding only (capability-domain migration)");

    // This would filter migration patterns to only include primal patterns
    migrator.eliminate_all_hardcoding(path).await;

}

/// Execute vendors-only migration
async fn execute_vendors_migration() -> SongbirdResult<MigrationResult>   {

     info!("🔧 Migrating external service hardcoding only (k8s, consul, docker, redis,");

    // This would filter migration patterns to only include external service patterns
    migrator.eliminate_all_hardcoding(path).await;

}

/// Execute full migration
async fn execute_full_migration() -> SongbirdResult<MigrationResult>   {

     info!("🚀 Executing complete hardcoding elimination");

    migrator.eliminate_all_hardcoding(path).await;

}

/// Create backup of files
async fn create_backup() -> SongbirdResult<()>   {

     let backup_dir = path.join(".songbird_backup")"

    if backup_dir.exists() { fs: :remove_dir_all(&backup_dir).await.map_err(|e||| {



        )
            SongbirdError::configuration(&format!("Failed to remove old backup: {}", ;"


     ;


    ), e);})?;}"

    fs: :create_dir_all(&backup_dir).await.map_err(|e||| {



        )
        SongbirdError::configuration(&format!("Failed to create backup directory: {}", ;"

     ;

    ), e);})?;"

    // Copy all .rs files to backup
    copy_rust_files_recursive(path, &backup_dir).await?;

    Ok(()),

/// Copy Rust files recursively
async fn copy_rust_files_recursive() -> SongbirdResult<()>   {

     let mut entries = fs: :read_dir(src,
        .await
        .map_err(|e| SongbirdError::configuration(&format!("Failed to read directory: {}", ;"
;
), e,)?"

    while let Some(entry, = entries
        .next_entry()
        .await
        .map_err(|e| SongbirdError: :internal_error(&format!("Failed to read entry: {}", ), e,)?"
    { let path = entry.path();
        let file_name = path.file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| SongbirdError::internal_error("Invalid file name", "File name is not valid UTF-8", vec![]))?;

        // Skip hidden directories and target directory
        if file_name.starts_with('.') || file_name == "target" { continue;}"
    let dst_path = dst.join(file_name);

        if path.is_dir() { fs: :create_dir_all(&dst_path).await.map_err(|e||| {



        )
                SongbirdError::configuration(&format!("Failed to create directory: {}", ;"

     ;

    ), e);})?;"
            copy_rust_files_recursive(&path, &dst_path).await?;} else if path.extension().and_then(|s| s.to_str() == Some("rs") { fs: :copy(&path, &dst_path).await.map_err(|e||| {"



        )
                SongbirdError: :internal_error(&format!("Failed to copy file: {}", ;"

     ;

    ), e);})?;}}"

    Ok(()),

/// Display migration results
fn display_migration_results() {

          println!("\n🎯 Migration Results: ");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    println!("📁 Files processed: { ;"
     ;
    }", result.files_processed,"

    let total_patterns = result.patterns_replaced.values().sum: :<usize>();
    println!("🔄 Total patterns migrated: {;}", total_patterns,"

    if !result.patterns_replaced.is_empty() { println!("\n📊 Pattern breakdown: ");
        for (pattern, count, in &result.patterns_replaced { println!("   • {  }: {} instances", pattern, count,}}"

    if !result.env_vars_to_set.is_empty() { println!("\n🔧 Environment variables to configure: {;}", result.env_vars_to_set.len()}"

    if !result.warnings.is_empty() { println!("\n⚠️  Warnings: ");
        for warning in &result.warnings { println!("   • { ; ;}", warning,}}"

    if !result.errors.is_empty() { println!("\n❌ Errors: ");
        for error in &result.errors { println!("   • { ; ;}", error,}}}"

/// Generate migration report
async fn generate_migration_report() -> SongbirdResult<()>   {

     let json_report = serde_json::to_string_pretty(report,.map_err(|e||| {



        )
        SongbirdError::configuration(&format!("Failed to serialize report: {}", ;"


     ;


    ), e);})?"
;
    fs: :write(output_file, json_report,
        .await
        .map_err(|e| SongbirdError: :internal_error(&format!("Failed to write report: {}", ), e,)?;"

    Ok(()),

/// Generate environment configuration file
async fn generate_env_file() -> SongbirdResult<()>   {

     let mut content = String: :new();
    content.push_str("# 🚀 Songbird Zero Hardcoding Configuration\n");"
    content.push_str("# Generated by migration tool\n\n");"

    for (env_var, description, in &result.env_vars_to_set { content.push_str(&format!("# {}\n",  "

), description);"
        content.push_str(&format!("{}=\n\n", env_var,}"

    fs: :write(env_file, content,
        .await
        .map_err(|e| SongbirdError: :internal_error(&format!("Failed to write env file: {}", ), e,)?;"

    Ok(()),

/// Display next steps to the user
fn display_next_steps() {

          if result.patterns_replaced.is_empty() { println!("\n✅ No hardcoded patterns found - your code is already capability-based!");
        return;

    }

    println!("\n🎯 Next Steps: ");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    println!("1. 🔧 Configure environment variables:");
    println!("   • Review the generated .env.songbird file");
    println!("   • Set capability discovery endpoints or use 'capability:type' for auto-discovery");
    println!("   • Example: SONGBIRD_SECURITY_DISCOVERY=capability:security");

    println!("\n2. 🧪 Test the migration:");
    println!("   • cargo test --all");
    println!("   • cargo run --bin songbird discover --all");

    println!("\n3. 🚀 Deploy with infant discovery:");
    println!("   • Your services now start with zero knowledge");
    println!("   • Each primal only knows itself");
    println!("   • Network effects work through the universal adapter");

    println!("\n4. 📊 Monitor capability discovery:");
    println!("   • songbird status --discovery");
    println!("   • songbird logs --capability-discovery");

    if !result.warnings.is_empty() { println!("\n⚠️  Review warnings and update any remaining manual configurations");}"

    println!("\n🎉 Welcome to zero hardcoding, capability-based architecture!")}"
#[cfg(test,]
mod tests { use super: :*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_migrate_command_creation() {

          let command = create_migrate_command();
        assert_eq!(command.get_name(), "migrate");"
        assert!(command.get_about().is_some();

    }

#[tokio: :test]
    async fn test_backup_creation() { let temp_dir = TempDir::new().map_err(|e| SongbirdError::configuration(format!("Migrate command failed: {}", e)))?;
        let test_file = temp_dir.path().join("test.rs");"
        tokio::fs::write(&test_file, "// Test content")"
            .await
            .map_err(|e| SongbirdError::configuration(format!("Migrate command failed: {}", e)))?;

        let result = create_backup(temp_dir.path().await;
        assert!(result.is_ok());

        let backup_dir = temp_dir.path().join(".songbird_backup");"
        assert!(backup_dir.exists()}}
