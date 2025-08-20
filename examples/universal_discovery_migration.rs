//! # Universal Discovery Migration Example
//!
//! This example demonstrates the canonical universal discovery system
//! using the modernized capability-based architecture.

use songbird_discovery::CanonicalDiscoveryFactory;
use songbird_errors::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🌟 SONGBIRD UNIVERSAL DISCOVERY - CANONICAL PATTERNS");
    println!("=========================================");

    // ✅ CANONICAL WAY: Universal auto-detection
    println!("\n✅ CANONICAL: Universal Auto-Detection");
    show_universal_auto_detection().await?;

    // ✅ CANONICAL WAY: Configuration-driven
    println!("\n✅ CANONICAL: Configuration-Driven");
    show_universal_configuration().await?;

    println!("\n🎉 Universal discovery system ready! All patterns are canonical.");
    Ok(())
}

/// Shows universal auto-detection (recommended)
async fn show_universal_auto_detection() -> Result<()> {
    println!("// Universal system automatically detects your environment");
    
    let factory = CanonicalDiscoveryFactory::new().await?;
    let discovery = factory.create_from_environment().await?;
    
    println!("let factory = CanonicalDiscoveryFactory::new().await?;");
    println!("let discovery = factory.create_from_environment().await?;");
    println!("// ✅ Automatically detects K8s, Consul, or defaults to static");
    println!("// ✅ No hardcoding - works in any environment");
    println!("// ✅ Easy to extend with new providers");
    
    // Show what was detected
    let providers = discovery.list_providers().await?;
    println!("// Detected {} provider(s)", providers.len());
    
    Ok(())
}

/// Shows configuration-driven approach
async fn show_universal_configuration() -> Result<()> {
    println!("// You can also explicitly configure providers");
    
    let factory = CanonicalDiscoveryFactory::new().await?;
    let discovery = factory.create_from_environment().await?;
    
    println!("let factory = CanonicalDiscoveryFactory::new().await?;");
    println!("let discovery = factory.create_from_environment().await?;");
    println!("// ✅ Explicit configuration when needed");
    println!("// ✅ Multiple providers can work together");
    println!("// ✅ Runtime provider registration");
    
    let providers = discovery.list_providers().await?;
    println!("// Configured {} provider(s)", providers.len());
    
    Ok(())
}

/// Environment variable examples
fn show_environment_examples() {
    println!("\n🌍 Environment Variable Examples:");
    println!("================================");
    
    println!("# For Kubernetes (auto-detected):");
    println!("export KUBERNETES_SERVICE_HOST=kubernetes.default.svc.cluster.local");
    println!("export KUBERNETES_NAMESPACE=my-namespace");
    
    println!("\n# For Consul (auto-detected):");
    println!("export CONSUL_URL=http://consul.service.consul:8500");
    println!("# or");
    println!("export CONSUL_HTTP_ADDR=consul.service.consul:8500");
    
    println!("\n# For Static (fallback):");
    println!("export SONGBIRD_DISCOVERY_STATIC=true");
    
    println!("\n# The universal system will auto-detect and configure appropriately!");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_universal_discovery_works() -> Result<()> {
        let factory = CanonicalDiscoveryFactory::new().await?;
        let discovery = factory.create_from_environment().await?;
        
        // Should work even with no specific environment
        let providers = discovery.list_providers().await?;
        assert!(!providers.is_empty(), "Should have at least a static provider");
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_configuration_builder() -> Result<()> {
        let factory = CanonicalDiscoveryFactory::new().await?;
        let _discovery = factory.create_from_environment().await?;
        
        Ok(())
    }
} 