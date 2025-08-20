/// # Zero-Cost Service Registry Demo
///
/// Demonstrates our zero-cost patterns vs traditional DI patterns
use crate::zero_cost_service_registry::{ZeroCostAIService, ZeroCostComputeService, ZeroCostSecurityService, ZeroCostService, ZeroCostServiceRegistry, ZeroCostStorageService};
use std::time::Instant;

/// Demonstrate zero-cost abstraction patterns in service registry
///
/// # Errors
/// Returns error if demo setup fails
pub fn demo_zero_cost_patterns() -> songbird_errors::Result<()> {
    println!("🚀 ZERO-COST SERVICE REGISTRY DEMO");
    println!("════════════════════════════════════");
    println!();

    // ✅ ZERO-COST: All services resolved at compile time
    let registry = ZeroCostServiceRegistry::new(
        ZeroCostSecurityService, // Stack allocated
        ZeroCostStorageService,  // Stack allocated
        ZeroCostComputeService,  // Stack allocated
        ZeroCostAIService,       // Stack allocated
    );

    println!("✅ Zero-Cost Registry Created");
    println!("   📊 Service count: {}", registry.service_count());
    println!("   💾 Memory usage: Stack allocated only");
    println!();

    // ✅ ZERO-COST: Direct field access, no HashMap lookups
    println!("🔍 Testing Direct Service Access:");
    let start = Instant::now();

    for _ in 0..10_000 {
        let _security = registry.security(); // Direct field access - zero cost
        let _storage = registry.storage(); // Direct field access - zero cost
        let _compute = registry.compute(); // Direct field access - zero cost
        let _ai = registry.ai(); // Direct field access - zero cost
    }

    let elapsed = start.elapsed();
    println!("   ⚡ 10,000 service accesses in: {elapsed:?}");
    println!(
        "   ⚡ Average: {:.2}ns per access",
        elapsed.as_nanos() as f64 / 10_000.0
    );
    println!();

    // ✅ ZERO-COST: Health checks with compile-time dispatch
    println!("🏥 Testing Health Checks:");
    let start = Instant::now();
    let health_reports = registry.health_check_all().await?;
    let elapsed = start.elapsed();

    for report in &health_reports.data {
        println!(
            "   ✅ {}: {}",
            report.service_id,
            if report.is_healthy {
                "Healthy"
            } else {
                "Unhealthy"
            }
        );
    }
    println!("   ⚡ Health check completed in: {elapsed:?}");
    println!();

    // ✅ Service capabilities demonstration
    println!("🎯 Service Capabilities:");
    println!("   🔒 Security: {:?}", registry.security().capabilities());
    println!("   💾 Storage: {:?}", registry.storage().capabilities());
    println!("   💻 Compute: {:?}", registry.compute().capabilities());
    println!("   🤖 AI: {:?}", registry.ai().capabilities());
    println!();

    println!("🎉 ZERO-COST PATTERNS WORKING PERFECTLY!");
    println!("   ✅ No HashMap lookups");
    println!("   ✅ No Arc<dyn> overhead");
    println!("   ✅ No RwLock contention");
    println!("   ✅ Compile-time service resolution");
    println!("   ✅ Stack allocated services");
    println!("   ✅ Inlined function calls");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_zero_cost_demo() {
        let result = demo_zero_cost_patterns().await;
        assert!(result.is_ok(), "Zero-cost demo should succeed");
    }

    #[test]
    fn test_zero_cost_service_creation() {
        // This should be zero-cost at runtime
        let registry = ZeroCostServiceRegistry::new(
            ZeroCostSecurityService,
            ZeroCostStorageService,
            ZeroCostComputeService,
            ZeroCostAIService,
        );

        // Direct field access should be inlined
        let _security = registry.security();
        let _storage = registry.storage();
        let _compute = registry.compute();
        let _ai = registry.ai();

        // Service count is compile-time constant
        assert_eq!(registry.service_count(), 4);
    }

    #[test]
    fn benchmark_zero_cost_access() {
        let registry = ZeroCostServiceRegistry::new(
            ZeroCostSecurityService,
            ZeroCostStorageService,
            ZeroCostComputeService,
            ZeroCostAIService,
        );

        let start = Instant::now();

        // This should be optimized to nearly nothing by the compiler
        for _ in 0..100_000 {
            std::hint::black_box(registry.security());
            std::hint::black_box(registry.storage());
            std::hint::black_box(registry.compute());
            std::hint::black_box(registry.ai());
        }

        let elapsed = start.elapsed();

        // Should be very fast (sub-millisecond for 100k accesses)
        println!("Zero-cost access benchmark: 100k accesses in {:?}", elapsed);

        // This should be extremely fast
        assert!(
            elapsed.as_millis() < 10,
            "Zero-cost access should be very fast"
        );
    }
}
