//! Ecosystem Standalone Demo - Canonical Implementation
//!
//! This example demonstrates the canonical patterns for running a standalone
//! Songbird ecosystem with modern service discovery and capability matching.

use songbird_universal_primals::errors::PrimalResult;
use songbird_universal_primals::universal_registry::{
    HealthStatus, ServiceCategory, ServiceFilter,
};
use songbird_universal_primals::{
    MemoryServiceRegistry, ServiceCapability, UniversalServiceRegistry,
};

#[tokio::main]
async fn main() -> PrimalResult<()> {
    println!("🎵 Songbird Ecosystem Standalone Demo - Canonical Implementation");

    // Create the universal service registry
    let registry = MemoryServiceRegistry::new();

    // Demonstrate basic registry functionality
    demonstrate_registry_operations(&registry).await?;

    // Demonstrate service discovery patterns
    demonstrate_service_discovery(&registry).await?;

    println!("✅ Ecosystem standalone demo completed successfully!");
    Ok(())
}

/// Demonstrate basic registry operations
async fn demonstrate_registry_operations(registry: &MemoryServiceRegistry) -> PrimalResult<()> {
    println!("🔧 Demonstrating registry operations...");

    // Test basic registry functionality
    let service_count = registry.list_services(None).await?.len();
    println!("Current service count: {service_count}");

    println!("✅ Registry operations demonstrated");
    Ok(())
}

/// Demonstrate service discovery patterns
async fn demonstrate_service_discovery(registry: &MemoryServiceRegistry) -> PrimalResult<()> {
    println!("🔍 Demonstrating service discovery patterns...");

    // Create service filters for different categories
    let compute_filter = ServiceFilter {
        categories: Some(vec![ServiceCategory::Compute]),
        lifecycle_stages: None,
        compliance_levels: None,
        capabilities: None,
        health_status: None,
        tags: Some(vec![]),
    };

    let storage_filter = ServiceFilter {
        categories: Some(vec![ServiceCategory::Storage]),
        lifecycle_stages: None,
        compliance_levels: None,
        capabilities: None,
        health_status: Some(vec![HealthStatus::Healthy]),
        tags: Some(vec![]),
    };

    // Demonstrate filtering capabilities
    let compute_services = registry.list_services(Some(compute_filter)).await?;
    let storage_services = registry.list_services(Some(storage_filter)).await?;

    println!("Found {} compute services", compute_services.len());
    println!("Found {} healthy storage services", storage_services.len());

    // Demonstrate capability-based discovery
    let _compute_capability = ServiceCapability::Compute {
        cpu_cores: Some(4.0),
        memory_gb: Some(8.0),
        gpu_support: false,
        container_runtime: Some("docker".to_string()),
    };

    let _storage_capability = ServiceCapability::Storage {
        storage_gb: Some(500.0),
        storage_type: songbird_universal_primals::universal_registry::StorageType::FileSystem,
        backup_support: true,
        encryption_support: true,
    };

    println!("🧠 Capability patterns defined:");
    println!("  - Compute: 4+ cores, 8GB+ RAM, Docker runtime");
    println!("  - Storage: 500GB+, backup & encryption support");

    println!("✅ Service discovery demonstration completed");
    Ok(())
}
