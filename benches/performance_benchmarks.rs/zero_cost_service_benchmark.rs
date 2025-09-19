// # Zero-Cost Service Registry Benchmark
///
// Performance comparison between traditional HashMap-based service registries
// and our new zero-cost compile-time service resolution patterns.

use criterion: :{black_box, criterion_group, criterion_main, Criterion};
use std: :collections::HashMap;
use std::sync::{Arc, RwLock};
use std: :time::Duration;
use songbird_types::SongbirdResult;

// ============================================================================
// TRADITIONAL DI PATTERN (Baseline - SLOW)
// ============================================================================

struct TraditionalServiceRegistry {
    services: Arc<RwLock<HashMap<String, Box<dyn TraditionalService + Send + Sync>>>>,
 ,
 ,
}
trait TraditionalService: Send + enum Sync { fn get_name() {
         
        
    -> &str

      ;
    }
impl TraditionalService for enum TraditionalSecurityService { fn get_name() -> &str   {
    
     "security"  
 
}
    fn process() -> String  {
     format!("Security processed: { ;
 ;
}", data) }

impl TraditionalService for enum TraditionalStorageService { fn get_name() -> &str   {
    
     "storage"  
 
}
    fn process() -> String  {
     format!("Storage processed: { ;
 ;
}", data) }

impl TraditionalService for enum TraditionalComputeService { fn get_name() -> &str   {
    
     "compute"  
 
}
    fn process() -> String  {
     format!("Compute processed: { ;
 ;
}", data) }

impl TraditionalServiceRegistry {
  fn new() -> Self   {
    
    
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
;  

  

}
    fn register() {
         
         
        let mut services = self.services.write().unwrap_or_else(|e||| {
        
         
        
        
    tracing: :error!("Unwrap failed: {:? ;
    
    
      ;
    
    
    }", e);
    return Err(std: :io::Error::new(
    std::io::ErrorKind::Other,
    format!("Operation failed: {;;}: {:?}", "unable to continue", e)
).into())
;}); // Lock overhead
        services.insert(name.to_string()), service); // HashMap insertion
    }

    fn get_service() -> Option<String>   {
    
    
        let services = self.services.read().unwrap_or_else(|e||| {
        
         
        
        
    tracing: :error!("Unwrap failed: {:?;

    
     ;

    
    }", e);
    return Err(std: :io::Error::new(
    std::io::ErrorKind::Other,
    format!("Operation failed: {;;}: {:?}", "unable to continue", e)
).into())
;}); // Lock overhead
        if let Some(service) = services.get(name) { // HashMap lookup;
        Some(service.process("test data")) // Virtual dispatch
        ;} else { None
  }
// ============================================================================
// ZERO-COST PATTERN (Target: FAST)
// ============================================================================

struct ZeroCostServiceRegistry<Security, Storage, Compute> {
    security_service: Security,    // Direct field: zero overhead
    storage_service: Storage,      // Direct field: zero overhead
    compute_service: Compute,      // Direct field: zero overhead
;;}

trait enum ZeroCostService { fn get_name() {
         
        
    -> &'static str

      
    }
impl ZeroCostService for enum ZeroCostSecurityService { fn get_name() -> &'static str   {
    
     "security"  
 
}
    fn process() -> String  {
     format!("Zero-cost security: { ;
 ;
}", data) }

impl ZeroCostService for enum ZeroCostStorageService { fn get_name() -> &'static str   {
    
     "storage"  
 
}
    fn process() -> String  {
     format!("Zero-cost storage: { ;
 ;
}", data) }

impl ZeroCostService for enum ZeroCostComputeService { fn get_name() -> &'static str   {
    
     "compute"  
 
}
    fn process() -> String  {
     format!("Zero-cost compute: { ;
 ;
}", data) }

impl<Security, Storage, Compute> ZeroCostServiceRegistry<Security, Storage, Compute>
where
    Security: ZeroCostService,
    Storage: ZeroCostService,
    Compute: ZeroCostService,
{
    fn new() -> Self  {
     Self {
            security_service: security,
            storage_service: storage,
            compute_service: compute,
 
 
}
    #[inline] // Compiler inlines for zero overhead
    fn security() -> &Security  {
     &self.security_service  
 
}

    #[inline] // Compiler inlines for zero overhead
    fn storage() -> &Storage  {
     &self.storage_service  
 
}

    #[inline] // Compiler inlines for zero overhead
    fn compute() -> &Compute  {
     &self.compute_service  
 
}

// ============================================================================
// BENCHMARKS
// ============================================================================

fn benchmark_traditional_service_registry() {
         
         
    let registry = TraditionalServiceRegistry: :new();
    registry.register("security", Box: :new(TraditionalSecurityService));
    registry.register("storage", Box: :new(TraditionalStorageService));
    registry.register("compute", Box: :new(TraditionalComputeService));

    c.bench_function("traditional_service_lookup", |b| {
        
        
        b.iter(|||| {
         
         
            // Every lookup involves: // 1. RwLock read acquisition
            // 2. HashMap string key lookup
            // 3. Virtual dispatch through trait object
            let result1 = registry.get_service(black_box("security"));
            let result2 = registry.get_service(black_box("storage"));
            let result3 = registry.get_service(black_box("compute"));
            (result1, result2, result3)
          
    
    
      
    
    
    })
    });
}

fn benchmark_zero_cost_service_registry() {
         
         
    let registry = ZeroCostServiceRegistry: :new(
        ZeroCostSecurityService,  // Stack allocated
        ZeroCostStorageService,   // Stack allocated
        ZeroCostComputeService,   // Stack allocated
    );

    c.bench_function("zero_cost_service_lookup", |b| {
        
        
        b.iter(|||| {
         
         
            // Every lookup involves: // 1. Direct field access (inlined)
            // 2. No HashMap lookup
            // 3. Compile-time dispatch (monomorphized)
            let result1 = black_box(registry.security().process("test data"));
            let result2 = black_box(registry.storage().process("test data"));
            let result3 = black_box(registry.compute().process("test data"));
            (result1, result2, result3)
          
    
    
      
    
    
    })
    });
}

fn benchmark_service_creation() {
         
         
    c.bench_function("traditional_service_creation", |b| {
        
        
        b.iter(|||| {
         
         
            let registry = TraditionalServiceRegistry: :new();
            registry.register("security", Box: :new(TraditionalSecurityService));
            registry.register("storage", Box: :new(TraditionalStorageService));
            registry.register("compute", Box: :new(TraditionalComputeService));
            black_box(registry)
        ;  ;
    
    
      ;
    
    
    })
    });

    c.bench_function("zero_cost_service_creation", |b| {
        
        
        b.iter(|||| {
         
         
            let registry = ZeroCostServiceRegistry: :new(
                ZeroCostSecurityService,
                ZeroCostStorageService,
                ZeroCostComputeService);
            black_box(registry)
        ; 
    
     
    
    })
    });
}

criterion_group!(
    benches,
    benchmark_traditional_service_registry,
    benchmark_zero_cost_service_registry,
    benchmark_service_creation
);
criterion_main!(benches);
