//! Songbird Unwrap Migrator - Context-Aware Panic Elimination Tool
//!
//! Systematically migrates unwrap(), expect(), and panic! patterns to use
//! Songbird's graceful error handling with SongbirdError and SongbirdResult.

use clap::{Arg, Command};
use std::path::Path;
use tracing::info;
use tracing_subscriber;

mod systematic_migrator;

use systematic_migrator::SystematicUnwrapMigrator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    let matches = Command::new("songbird-unwrap-migrator")
        .version("3.1.0")
        .about("🔄 Songbird Enhanced Unwrap/Expect Migrator: Context-aware panic elimination")
        .arg(
            Arg::new("path")
                .short('p')
                .long("path")
                .value_name("PATH")
                .help("Root path to scan for Rust files (defaults to ./crates)")
                .default_value("./crates")
        )
        .arg(
            Arg::new("dry-run")
                .long("dry-run")
                .help("Show what would be changed without applying changes")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("apply")
                .long("apply")
                .help("Apply the migration changes to files")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("stats-only")
                .long("stats-only")
                .help("Show statistics without performing migration")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("exclude-tests")
                .long("exclude-tests")
                .help("Exclude test files from migration (tests may legitimately use unwrap)")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("file")
                .short('f')
                .long("file")
                .value_name("FILE")
                .help("Target a specific file instead of scanning directory")
        )
        .get_matches();

    let default_path = "./crates".to_string();
    let root_path = matches.get_one::<String>("path").unwrap_or(&default_path);
    let dry_run = matches.get_flag("dry-run");
    let apply_changes = matches.get_flag("apply");
    let stats_only = matches.get_flag("stats-only");
    let exclude_tests = matches.get_flag("exclude-tests");
    let target_file = matches.get_one::<String>("file");

    info!("🚀 Starting Songbird Unwrap Migration");
    info!("📁 Target path: {}", root_path);
    info!("🔍 Mode: {}", if stats_only { "Statistics Only" } 
                              else if dry_run { "Dry Run" } 
                              else if apply_changes { "Apply Changes" } 
                              else { "Preview" });

    let migrator = SystematicUnwrapMigrator::new_songbird_optimized();

    if let Some(file_path) = target_file {
        // Single file mode
        handle_single_file(&migrator, file_path, apply_changes).await?;
    } else if stats_only {
        // Analysis mode
        handle_stats_mode(&migrator, root_path, exclude_tests).await?;
    } else if dry_run || apply_changes {
        // Migration mode
        handle_migration_mode(&migrator, root_path, !apply_changes, exclude_tests).await?;
    } else {
        println!("Please specify --dry-run, --apply, or --stats-only");
        println!("\n💡 Helpful commands:");
        println!("   📊 Analysis: cargo run -- --stats-only");
        println!("   🧪 Test run: cargo run -- --dry-run");
        println!("   ⚡ Apply: cargo run -- --apply");
        println!("   📄 Single file: cargo run -- --file path/to/file.rs --dry-run");
    }

    Ok(())
}

async fn handle_single_file(
    migrator: &SystematicUnwrapMigrator,
    file_path: &str,
    apply: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("📄 Analyzing single file: {}", file_path);
    
    let path = Path::new(file_path);
    if !path.exists() {
        return Err(format!("File not found: {}", file_path).into());
    }

    let migrations = migrator.migrate_file(path, !apply).await?;
    
    println!("\n📋 File Analysis: {}", file_path);
    println!("   🔧 Migrations applied: {}", migrations);
    
    if !apply {
        println!("\n💡 Run with --apply to make the changes permanent");
    } else if migrations > 0 {
        println!("\n✅ Changes have been applied");
        println!("   🧪 Run tests to verify: cargo test --lib --package <crate>");
    } else {
        println!("\n✨ No unwrap/expect patterns found in this file");
    }
    
    Ok(())
}

async fn handle_stats_mode(
    migrator: &SystematicUnwrapMigrator,
    root_path: &str,
    exclude_tests: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("📊 Analyzing codebase for unwrap/expect patterns...");
    
    let stats = migrator.analyze_codebase(Path::new(root_path), exclude_tests).await?;
    
    println!("\n📊 Songbird Codebase Analysis:");
    println!("   📁 Files scanned: {}", stats.files_scanned);
    println!("   ⚠️  Total unwrap/expect calls: {}", stats.total_unwrap_calls);
    println!("   🔧 Migrable patterns: {}", stats.migrable_patterns);
    println!("   🧪 Test file patterns: {}", stats.test_file_patterns);
    println!("   ✅ Songbird compatible: {}", stats.songbird_compatible);
    
    println!("\n📋 Pattern Breakdown:");
    let mut categories: Vec<_> = stats.pattern_categories.iter().collect();
    categories.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
    
    for (category, count) in categories {
        let emoji = match category.as_str() {
            "Configuration" => "⚙️ ",
            "Network" => "🌐",
            "Storage" => "💾",
            "Security" => "🛡️ ",
            "Validation" => "✅",
            "Discovery" => "🔍",
            "Orchestration" => "🎼",
            _ => "📦",
        };
        println!("   {} {}: {}", emoji, category, count);
    }
    
    println!("\n💡 Next Steps:");
    println!("   1. Run with --dry-run to preview migrations");
    println!("   2. Run with --apply to execute migrations");
    println!("   3. Focus on high-priority crates first (orchestrator, discovery)");
    
    Ok(())
}

async fn handle_migration_mode(
    migrator: &SystematicUnwrapMigrator,
    root_path: &str,
    dry_run: bool,
    exclude_tests: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mode = if dry_run { "Dry Run" } else { "Apply" };
    info!("🔄 {} Mode: Processing codebase...", mode);
    
    let result = migrator.migrate_codebase(
        Path::new(root_path),
        dry_run,
        exclude_tests
    ).await?;
    
    println!("\n🎉 Songbird Migration Complete:");
    println!("   📁 Files processed: {}", result.files_processed);
    println!("   🔧 Migrations applied: {}", result.migrations_applied);
    println!("   ⏱️  Execution time: {}ms", result.execution_time_ms);
    
    if !result.failed_files.is_empty() {
        println!("\n⚠️  Files with issues:");
        for (file, error) in &result.failed_files {
            println!("   ❌ {}: {}", file.display(), error);
        }
    }
    
    if dry_run {
        println!("\n💡 This was a dry run. Changes were NOT applied.");
        println!("   Run with --apply to make changes permanent");
    } else if result.migrations_applied > 0 {
        println!("\n✅ Changes have been applied to your codebase");
        println!("   🧪 Next steps:");
        println!("      1. Run: cargo fmt");
        println!("      2. Run: cargo clippy --workspace");
        println!("      3. Run: cargo test --lib");
        println!("      4. Review changes with: git diff");
    } else {
        println!("\n✨ No unwrap/expect patterns found - codebase is clean!");
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_creation() {
        let cmd = Command::new("test-unwrap-migrator")
            .version("3.1.0")
            .about("Test CLI");
        
        assert_eq!(cmd.get_name(), "test-unwrap-migrator");
    }
}
