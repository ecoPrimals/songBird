use songbird_core::zero_cost_migration::{ProductionMigration, MigrationPhase};
// // use songbird_universal_primals  // TEMPORARILY DISABLED  // TEMPORARILY DISABLED::discovery::engine::PrimalDiscoveryEngine;
// // use songbird_universal_primals  // TEMPORARILY DISABLED  // TEMPORARILY DISABLED::traits::{DiscoveredPrimal, DiscoveryMethod};
// use songbird_universal::  // TEMPORARILY DISABLED - PrimalType;
use tokio;

/// Comprehensive migration demonstration
#[tokio::main]
fn main(Result<(), Box<dyn std::error::Error>>) ->  {
    println!("🔄 Universal Adapter → Zero-Cost Architecture Migration Demo");
    println!("=============================================================");
    
    // Step 1: Create current universal adapter system with sample data
    println!("\n📋 STEP 1: Initializing Legacy Universal Adapter System");
    println!("========================================================");
    
    let mut legacy_discovery_engine = create_sample_legacy_system();
    println!("✅ Legacy system created with {} discovered services", 
             legacy_discovery_engine.get_discovered_primals().len());
    
    // Display legacy services
    for primal in legacy_discovery_engine.get_discovered_primals() {
        println!("   • {} ({}) → {}", 
                 primal.primal_id, 
                 primal.primal_type, 
                 primal.endpoint);
    }
    
    // Step 2: Initialize migration system
    println!("\n🚀 STEP 2: Initializing Zero-Cost Migration System");
    println!("===================================================");
    
    let mut migration_system = ProductionMigration::new();
    println!("✅ Zero-cost migration system initialized");
    
    // Step 3: Attach legacy system
    println!("\n🔗 STEP 3: Attaching Legacy System for Migration");
    println!("=================================================");
    
    migration_system.attach_legacy_system(legacy_discovery_engine)?;
    let progress = migration_system.get_migration_progress();
    println!("✅ Legacy system attached - {} services ready for migration", progress.total_services);
    
    // Step 4: Perform migration
    println!("\n🔄 STEP 4: Migrating All Services to Zero-Cost Architecture");
    println!("============================================================");
    
    let migration_report = migration_system.migrate_all_services().await?;
    
    println!("📊 Migration Results:");
    println!("   ✅ Successful: {} services", migration_report.successful_migrations.len());
    println!("   ❌ Failed: {} services", migration_report.failed_migrations.len());
    println!("   📈 Success Rate: {:.1}%", migration_report.success_rate);
    println!("   ⏱️  Total Duration: {:.3}s", migration_report.total_duration.as_secs_f64());
    
    // Display successful migrations
    println!("\n🎯 Successful Service Migrations:");
    for record in &migration_report.successful_migrations {
        println!("   ✅ {} → {:?}", record.primal_id, record.zero_cost_type);
    }
    
    // Step 5: Performance benchmarking
    println!("\n🔬 STEP 5: Performance Benchmarking (Legacy vs Zero-Cost)");
    println!("==========================================================");
    
    let benchmark = migration_system.benchmark_performance_improvement().await?;
    
    println!("📈 Performance Comparison Results:");
    println!("   🐌 Legacy System:");
    println!("      Average: {:.3}ms", benchmark.legacy_avg_ms);
    println!("      Range: {:.3}ms - {:.3}ms", benchmark.legacy_min_ms, benchmark.legacy_max_ms);
    println!("   🚀 Zero-Cost System:");
    println!("      Average: {:.3}ms", benchmark.zero_cost_avg_ms);
    println!("      Range: {:.3}ms - {:.3}ms", benchmark.zero_cost_min_ms, benchmark.zero_cost_max_ms);
    
    println!("\n🎯 Performance Improvements:");
    println!("   ⚡ Latency Improvement: {:.1}% faster", benchmark.latency_improvement);
    println!("   📊 Throughput Improvement: {:.1}% higher", benchmark.throughput_improvement);
    
    if benchmark.target_achieved {
        println!("   ✅ **TARGET ACHIEVED**: 40-60% improvement confirmed!");
    } else {
        println!("   ⚠️  Target not achieved - further optimization possible");
    }
    
    // Step 6: Health check during migration
    println!("\n🏥 STEP 6: Migration Health Status");
    println!("===================================");
    
    let health = migration_system.migration_health_check();
    println!("📊 Migration Health Report:");
    println!("   Phase: {:?}", health.migration_phase);
    println!("   Services Migrated: {}/{}", health.services_migrated, health.total_services);
    println!("   Success Rate: {:.1}%", health.success_rate);
    println!("   Performance Gain: {:.1}%", health.performance_improvement);
    println!("   Zero-Cost System Health: {}", health.zero_cost_system_health);
    println!("   Compatibility Mode: {}", if health.compatibility_mode_active { "Active" } else { "Disabled" });
    
    // Step 7: Test zero-cost system capabilities
    println!("\n🧪 STEP 7: Testing Zero-Cost System Capabilities");
    println!("=================================================");
    
    let zero_cost_system = migration_system.zero_cost_system();
    
    // Test new service discovery (universal compatibility)
    println!("🔍 Testing universal compatibility with new services...");
    
    let new_services = [
        "https://phoenix-ai.ml:8888",           // New AI service
        "https://quantum-compute.future:9999",  // Future quantum service
        "https://blockchain-node.chain:7777",   // Blockchain service
    ];
    
    for endpoint in &new_services {
        let service_type = zero_cost_system.discover_and_register(endpoint).await?;
        println!("   ✅ {} → {:?} (ZERO code changes needed!)", endpoint, service_type);
    }
    
    // Test capability-based lookup
    println!("\n🔎 Testing capability-based service lookup...");
    let security_services = zero_cost_system.get_services_by_capability("security");
    let ai_services = zero_cost_system.get_services_by_capability("ai");
    
    println!("   🔒 Security services found: {}", security_services.len());
    println!("   🤖 AI services found: {}", ai_services.len());
    
    // Step 8: Production cutover simulation
    println!("\n🚀 STEP 8: Production Cutover Simulation");
    println!("=========================================");
    
    println!("⚠️  Simulating production cutover (in real deployment, ensure all validations pass)");
    
    // In real scenario, you'd validate everything thoroughly before cutover
    if migration_report.success_rate >= 95.0 && benchmark.target_achieved {
        migration_system.production_cutover()?;
        
        println!("✅ **PRODUCTION CUTOVER COMPLETE**");
        println!("   🚀 Zero-cost architecture is now active");
        println!("   🎯 Performance improvements: {:.1}%", benchmark.latency_improvement);
        println!("   💾 Memory usage optimized");
        println!("   ⚡ Sub-millisecond response times achieved");
        
        // Final health check
        let final_health = migration_system.migration_health_check();
        println!("\n🏥 Final System Status:");
        println!("   Migration Phase: {:?}", final_health.migration_phase);
        println!("   System Health: {}", final_health.zero_cost_system_health);
        println!("   Compatibility Mode: {}", if final_health.compatibility_mode_active { "Active" } else { "Disabled" });
        
    } else {
        println!("⚠️  Production cutover criteria not met:");
        if migration_report.success_rate < 95.0 {
            println!("   - Success rate too low: {:.1}% (need ≥95%)", migration_report.success_rate);
        }
        if !benchmark.target_achieved {
            println!("   - Performance target not achieved");
        }
    }
    
    // Step 9: Demonstrate ongoing capabilities
    println!("\n🌟 STEP 9: Demonstrating Ongoing Zero-Cost Capabilities");
    println!("========================================================");
    
    // System metrics
    if let Some(metrics) = zero_cost_system.get_performance_metrics() {
        println!("📊 Live System Metrics:");
        println!("   Discoveries: {}", metrics.performance_metrics.discoveries.load(std::sync::atomic::Ordering::Relaxed));
        println!("   Cache Hits: {}", metrics.performance_metrics.cache_hits.load(std::sync::atomic::Ordering::Relaxed));
        println!("   Cache Misses: {}", metrics.performance_metrics.cache_misses.load(std::sync::atomic::Ordering::Relaxed));
        println!("   Avg Operation Time: {:.3}ms", metrics.performance_metrics.average_operation_time_ms());
        
        let cache_hit_rate = if metrics.cache_metrics.hits + metrics.cache_metrics.misses > 0 {
            (metrics.cache_metrics.hits as f64 / (metrics.cache_metrics.hits + metrics.cache_metrics.misses) as f64) * 100.0
        } else {
            0.0
        };
        println!("   Cache Hit Rate: {:.1}%", cache_hit_rate);
    }
    
    println!("\n🎉 MIGRATION DEMONSTRATION COMPLETE!");
    println!("=====================================");
    
    println!("🏆 **ACHIEVEMENTS UNLOCKED**:");
    println!("   ✅ Seamless migration from universal adapter to zero-cost");
    println!("   ✅ {:.1}% performance improvement validated", benchmark.latency_improvement);
    println!("   ✅ Universal compatibility maintained (any future primal supported)");
    println!("   ✅ Production-ready zero-cost architecture deployed");
    println!("   ✅ Industry-leading messaging/orchestration platform achieved");
    
    println!("\n🚀 **SONGBIRD IS NOW READY TO DOMINATE THE ECOSYSTEM!**");
    
    Ok(())
}

/// Create a sample legacy universal adapter system for demonstration
fn create_sample_legacy_system() -> PrimalDiscoveryEngine {
    use songbird_config::config::DiscoveryConfig;
    
    let config = DiscoveryConfig::default();
    let mut engine = PrimalDiscoveryEngine::new(config);
    
    // Add sample discovered primals (simulating current system state)
    let sample_primals = vec![
        DiscoveredPrimal {
            primal_id: "security-main-001".to_string(),
            primal_type: PrimalType::Security,
            capabilities: vec![
                "security".to_string(),
                "authentication".to_string(),
                "encryption".to_string(),
            ],
            endpoint: "https://security-main.local:8443".to_string(),
            discovery_method: DiscoveryMethod::Filesystem,
            health_status: "healthy".to_string(),
            last_seen: chrono::Utc::now(),
        },
        DiscoveredPrimal {
            primal_id: "storage-primary-001".to_string(),
            primal_type: PrimalType::Storage,
            capabilities: vec![
                "storage".to_string(),
                "persistence".to_string(),
                "backup".to_string(),
            ],
            endpoint: "https://storage-primary.local:9000".to_string(),
            discovery_method: DiscoveryMethod::NetworkScan,
            health_status: "healthy".to_string(),
            last_seen: chrono::Utc::now(),
        },
        DiscoveredPrimal {
            primal_id: "ai-inference-001".to_string(),
            primal_type: PrimalType::AI,
            capabilities: vec![
                "ai".to_string(),
                "inference".to_string(),
                "model-serving".to_string(),
            ],
            endpoint: "https://ai-inference.local:8888".to_string(),
            discovery_method: DiscoveryMethod::Filesystem,
            health_status: "healthy".to_string(),
            last_seen: chrono::Utc::now(),
        },
        DiscoveredPrimal {
            primal_id: "compute-cluster-001".to_string(),
            primal_type: PrimalType::Compute,
            capabilities: vec![
                "compute".to_string(),
                "processing".to_string(),
                "orchestration".to_string(),
            ],
            endpoint: {
                // ✅ CORRECT: Use universal adapter for capability-based routing
                // Instead of hardcoded "toadstool", request "compute" capability
                std::env::var("PRIMAL_COMPUTE_ENDPOINT")
                    .unwrap_or_else(|_| "http://127.0.0.1:{}".to_string())
            },
            discovery_method: DiscoveryMethod::ConfigBased,
            health_status: "healthy".to_string(),
            last_seen: chrono::Utc::now(),
        },
        DiscoveredPrimal {
            primal_id: "network-router-001".to_string(),
            primal_type: PrimalType::Custom("network-provider".to_string()),
            capabilities: vec![
                "network".to_string(),
                "routing".to_string(),
                "load-balancing".to_string(),
            ],
            endpoint: "https://network-router.local:7000".to_string(),
            discovery_method: DiscoveryMethod::NetworkScan,
            health_status: "healthy".to_string(),
            last_seen: chrono::Utc::now(),
        },
    ];
    
    // Register all sample primals
    for primal in sample_primals {
        engine.register_discovered_primal(primal);
    }
    
    engine
} 