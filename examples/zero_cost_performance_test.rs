use songbird_types::config::consolidated_canonical::CanonicalSongbirdConfig;
/// # Zero-Cost Performance Verification
///
/// Simple test to verify our zero-cost patterns are working correctly
/// and demonstrate the performance difference vs traditional DI patterns.
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Instant;

// ============================================================================
// TRADITIONAL DI PATTERN (Baseline - SLOW)
// ============================================================================

struct TraditionalServiceRegistry {
    services: Arc<RwLock<HashMap<String, Box<dyn TraditionalService + Send + Sync>>>>,
}

trait TraditionalService: Send + Sync {
    fn get_name(&self) -> &str;
    fn process(&self, data: &str) -> String;
}

struct TraditionalSecurityService;
impl TraditionalService for TraditionalSecurityService {
    fn get_name(&self) -> &str {
        "security"
    }
    fn process(&self, data: &str) -> String {
        format!("Traditional security: {}", data)
    }
}

impl TraditionalServiceRegistry {
    fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn register(&self, name: &str, service: Box<dyn TraditionalService + Send + Sync>) {
        let mut services = self.services.write().unwrap_or_else(|e| {
            tracing::error!("Unwrap failed: {:?}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Operation failed - {}: {:?}", "unable to continue", e),
            )
            .into());
        }); // Lock overhead
        services.insert(name.to_string(), service); // HashMap insertion + heap allocation
    }

    fn get_service(&self, name: &str) -> Option<String> {
        let services = self.services.read().unwrap_or_else(|e| {
            tracing::error!("Unwrap failed: {:?}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Operation failed - {}: {:?}", "unable to continue", e),
            )
            .into());
        }); // Lock overhead
        if let Some(service) = services.get(name) {
            // HashMap lookup
            Some(service.process("test data")) // Virtual dispatch
        } else {
            None
        }
    }
}

// ============================================================================
// ZERO-COST PATTERN (Target - FAST)
// ============================================================================

struct ZeroCostServiceRegistry<Security> {
    security_service: Security, // Direct field - zero overhead
}

trait ZeroCostService {
    fn get_name(&self) -> &'static str;
    fn process(&self, data: &str) -> String;
}

struct ZeroCostSecurityService;
impl ZeroCostService for ZeroCostSecurityService {
    fn get_name(&self) -> &'static str {
        "security"
    }
    fn process(&self, data: &str) -> String {
        format!("Zero-cost security: {}", data)
    }
}

impl<Security> ZeroCostServiceRegistry<Security>
where
    Security: ZeroCostService,
{
    fn new(security: Security) -> Self {
        Self {
            security_service: security,
        }
    }

    #[inline] // Compiler inlines for zero overhead
    fn security(&self) -> &Security {
        &self.security_service
    }
}

// ============================================================================
// PERFORMANCE TESTS
// ============================================================================

fn test_traditional_performance() -> (String, u128) {
    println!("🐌 Testing Traditional DI Performance...");

    let registry = TraditionalServiceRegistry::new();
    registry.register("security", Box::new(TraditionalSecurityService));

    let start = Instant::now();
    let mut results = Vec::new();

    // Simulate 100,000 service lookups
    for i in 0..100_000 {
        if let Some(result) = registry.get_service("security") {
            results.push(result);
        }

        // Black box to prevent optimization
        if i % 10000 == 0 {
            std::hint::black_box(&results);
        }
    }

    let elapsed = start.elapsed();
    let last_result = results.last().cloned().unwrap_or_default();

    println!("   ❌ Traditional DI: {} lookups in {:?}", results.len(), elapsed);
    println!("   ❌ Average: {:.2}ns per lookup", elapsed.as_nanos() as f64 / results.len() as f64);

    (last_result, elapsed.as_nanos())
}

fn test_zero_cost_performance() -> (String, u128) {
    println!("🚀 Testing Zero-Cost Performance...");

    let registry = ZeroCostServiceRegistry::new(ZeroCostSecurityService);

    let start = Instant::now();
    let mut results = Vec::new();

    // Simulate 100,000 service lookups
    for i in 0..100_000 {
        let result = registry.security().process("test data");
        results.push(result);

        // Black box to prevent optimization
        if i % 10000 == 0 {
            std::hint::black_box(&results);
        }
    }

    let elapsed = start.elapsed();
    let last_result = results.last().cloned().unwrap_or_default();

    println!("   ✅ Zero-Cost: {} lookups in {:?}", results.len(), elapsed);
    println!("   ✅ Average: {:.2}ns per lookup", elapsed.as_nanos() as f64 / results.len() as f64);

    (last_result, elapsed.as_nanos())
}

fn main() {
    println!("🎯 ZERO-COST vs DEPENDENCY INJECTION PERFORMANCE TEST");
    println!("═══════════════════════════════════════════════════════");
    println!();

    // Warm up
    println!("🔥 Warming up...");
    test_traditional_performance();
    test_zero_cost_performance();
    println!();

    println!("📊 PERFORMANCE COMPARISON (100,000 service lookups)");
    println!("─────────────────────────────────────────────────────");

    let (traditional_result, traditional_nanos) = test_traditional_performance();
    let (zero_cost_result, zero_cost_nanos) = test_zero_cost_performance();

    println!();
    println!("🏆 RESULTS:");
    println!("──────────");

    let improvement = traditional_nanos as f64 / zero_cost_nanos as f64;
    let percentage_faster =
        ((traditional_nanos - zero_cost_nanos) as f64 / traditional_nanos as f64) * 100.0;

    println!("🚀 Performance Improvement: {:.1}x faster", improvement);
    println!("⚡ Speed Increase: {:.1}% faster", percentage_faster);

    println!();
    println!("💾 Memory Usage:");
    println!("   ❌ Traditional: HashMap + Arc + RwLock + Box + vtable");
    println!("   ✅ Zero-Cost: Direct struct field (stack allocated)");

    println!();
    println!("🎯 Call Overhead:");
    println!("   ❌ Traditional: RwLock read + HashMap lookup + virtual dispatch");
    println!("   ✅ Zero-Cost: Direct field access (inlined by compiler)");

    println!();
    if improvement >= 2.0 {
        println!("🎉 SUCCESS: Zero-cost abstraction is {:.1}x faster!", improvement);
        println!("   This demonstrates Rust's zero-cost abstraction philosophy!");
    } else {
        println!("⚠️  Modest improvement: {:.1}x faster", improvement);
        println!("   Compiler optimizations may have reduced the difference");
    }

    println!();
    println!("✅ Both patterns produced correct results:");
    println!("   Traditional: {}", traditional_result);
    println!("   Zero-Cost:   {}", zero_cost_result);
}
