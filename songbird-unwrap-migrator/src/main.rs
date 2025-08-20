//! Songbird Unwrap Migrator CLI - 2025 Edition
//!
//! Enhanced CLI tool for comprehensive codebase modernization including:
//! - Fixing compilation errors
//! - Eliminating unwrap/expect/panic patterns
//! - Unifying to canonical patterns
//! - Cleaning fragments and deprecations

use clap::{Arg, Command};
use std::path::PathBuf;
use tracing::{info, error, Level};
use tracing_subscriber;

mod modernized_migrator;
use modernized_migrator::ModernizedMigrator;

#[tokio::main]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    let matches = Command::new("songbird-modernizer")
        .version("2.0.0")
        .author("Songbird Team")
        .about("Comprehensive codebase modernization tool for Songbird")
        .arg(
            Arg::new("mode")
                .short('m')
                .long("mode")
                .value_name("MODE")
                .help("Migration mode: canonical, unwraps, or full")
                .default_value("full")
        )
        .arg(
            Arg::new("dry-run")
                .long("dry-run")
                .help("Show what would be changed without making actual changes")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("target")
                .short('t')
                .long("target")
                .value_name("DIRECTORY")
                .help("Target directory to process (default: current directory)")
                .action(clap::ArgAction::Append)
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enable verbose output")
                .action(clap::ArgAction::SetTrue)
        )
        .get_matches();

    if matches.get_flag("verbose") {
        tracing_subscriber::fmt()
            .with_max_level(Level::DEBUG)
            .init();
    }

    let mode = matches.get_one::<String>("mode").map_err(|e| SongbirdError::internal(format!("Operation failed: {:?}", e)))?;
    let dry_run = matches.get_flag("dry-run");
    let targets: Vec<PathBuf> = matches
        .get_many::<String>("target")
        .unwrap_or_default()
        .map(PathBuf::from)
        .collect();

    info!("🚀 Starting Songbird Modernization Tool v2.0.0");
    info!("📋 Mode: {}", mode);
    info!("🔍 Dry run: {}", dry_run);
    
    if !targets.is_empty() {
        info!("📂 Target directories: {:?}", targets);
    } else {
        info!("📂 Processing current directory");
    }

    let mut migrator = ModernizedMigrator::new()?;
    migrator.set_dry_run(dry_run);
    
    if !targets.is_empty() {
        migrator.set_target_dirs(targets);
    }

    let report = match mode.as_str() {
        "canonical" => {
            info!("🎯 Running canonical modernization...");
            migrator.run_canonical_modernization().await?
        },
        "unwraps" => {
            info!("🎯 Running unwrap migration...");
            migrator.run_migration().await?
        },
        "full" => {
            info!("🎯 Running full modernization (canonical + unwraps)...");
            
            // First run canonical modernization
            info!("📋 Phase 1: Canonical modernization...");
            let canonical_report = migrator.run_canonical_modernization().await?;
            
            // Then run unwrap migration
            info!("📋 Phase 2: Unwrap migration...");
            let unwrap_report = migrator.run_migration().await?;
            
            // Combine reports
            let mut combined_stats = canonical_report.stats;
            combined_stats.files_scanned += unwrap_report.stats.files_scanned;
            combined_stats.files_modified += unwrap_report.stats.files_modified;
            combined_stats.total_replacements += unwrap_report.stats.total_replacements;
            combined_stats.errors.extend(unwrap_report.stats.errors);
            combined_stats.warnings.extend(unwrap_report.stats.warnings);
            
            let mut combined_files = canonical_report.files_modified;
            combined_files.extend(unwrap_report.files_modified);
            
            let mut combined_unwraps = canonical_report.remaining_unwraps;
            combined_unwraps.extend(unwrap_report.remaining_unwraps);
            
            let mut combined_recommendations = canonical_report.recommendations;
            combined_recommendations.extend(unwrap_report.recommendations);
            
            modernized_migrator::MigrationReport {
                stats: combined_stats,
                files_modified: combined_files,
                remaining_unwraps: combined_unwraps,
                recommendations: combined_recommendations,
            }
        },
        _ => {
            error!("❌ Unknown mode: {}. Use 'canonical', 'unwraps', or 'full'", mode);
            std::process::exit(1);
        }
    };

    // Print summary
    print_migration_summary(&report, dry_run);

    // Write detailed report
    let report_path = "modernization_report.json";
    let report_json = serde_json::to_string_pretty(&report)?;
    tokio::fs::write(report_path, report_json).await?;
    info!("📄 Detailed report written to: {}", report_path);

    if !report.stats.errors.is_empty() {
        error!("⚠️  {} errors encountered during migration", report.stats.errors.len());
        for error in &report.stats.errors {
            error!("   - {}", error);
        }
        std::process::exit(1);
    }

    info!("✅ Modernization completed successfully!");
    Ok(())
}

fn print_migration_summary(report: &modernized_migrator::MigrationReport, dry_run: bool) {
    println!("\n🎉 MODERNIZATION SUMMARY");
    println!("========================");
    
    if dry_run {
        println!("🔍 DRY RUN MODE - No files were actually modified");
    }
    
    println!("📊 Statistics:");
    println!("   Files scanned: {}", report.stats.files_scanned);
    println!("   Files modified: {}", report.stats.files_modified);
    println!("   Total replacements: {}", report.stats.total_replacements);
    
    if !report.stats.patterns_applied.is_empty() {
        println!("\n🔧 Patterns Applied:");
        for (pattern, count) in &report.stats.patterns_applied {
            println!("   {} → {} times", pattern, count);
        }
    }
    
    if !report.stats.categories.is_empty() {
        println!("\n📋 By Category:");
        for (category, count) in &report.stats.categories {
            println!("   {} → {} replacements", category, count);
        }
    }
    
    if !report.stats.warnings.is_empty() {
        println!("\n⚠️  Warnings ({}):", report.stats.warnings.len());
        for warning in &report.stats.warnings {
            println!("   - {}", warning);
        }
    }
    
    if !report.remaining_unwraps.is_empty() {
        println!("\n🚨 Remaining Issues ({}):", report.remaining_unwraps.len());
        let mut by_file: std::collections::HashMap<&std::path::PathBuf, usize> = std::collections::HashMap::new();
        for unwrap in &report.remaining_unwraps {
            *by_file.entry(&unwrap.file).or_insert(0) += 1;
        }
        
        for (file, count) in by_file {
            println!("   {} → {} issues", file.display(), count);
        }
    }
    
    if !report.recommendations.is_empty() {
        println!("\n💡 Recommendations:");
        for recommendation in &report.recommendations {
            println!("   {}", recommendation);
        }
    }
    
    println!("\n🎯 Next Steps:");
    if dry_run {
        println!("   1. Review the changes above");
        println!("   2. Run without --dry-run to apply changes");
    } else {
        println!("   1. Run 'cargo fmt' to format code");
        println!("   2. Run 'cargo clippy' to check for issues");
        println!("   3. Run 'cargo test' to verify functionality");
        if !report.remaining_unwraps.is_empty() {
            println!("   4. Address remaining {} unwrap/panic issues", report.remaining_unwraps.len());
        }
    }
}
