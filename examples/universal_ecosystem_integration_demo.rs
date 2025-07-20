//! Universal Ecosystem Integration Demo
//!
//! This demo shows how Songbird properly leverages the ecosystem primals using
//! universal capability-based integration rather than hardcoded assumptions.
//!
//! **Key Principles Demonstrated:**
//! 1. Universal Discovery - Auto-discover primals at ../beardog, ../nestgate, etc.
//! 2. Capability-Based Routing - Route by what services can do, not what they're called
//! 3. Responsibility Delegation - Let each primal handle their expertise
//! 4. Graceful Fallback - Songbird handles tasks when primals unavailable
//! 5. Zero Hardcoding - No assumptions about specific primal implementations

use songbird_universal_primals::{
    discovery::{discover_ecosystem_primals, create_universal_context, EcosystemDiscovery, EcosystemDiscoveryConfig},
    registry::UniversalPrimalRegistry,
    traits::{PrimalCapability, PrimalContext, SecurityLevel, NetworkLocation},
    router::UniversalPrimalRouter,
};
use std::collections::HashMap;
use tracing::{info, warn, error};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("🌟 Starting Universal Ecosystem Integration Demo");
    info!("🎯 Demonstrating capability-based primal integration");

    // Step 1: Initialize Universal Primal Registry
    let mut registry = UniversalPrimalRegistry::new();
    
    // Step 2: Discover Real Ecosystem Primals
    info!("🔍 Discovering ecosystem primals at ../beardog, ../nestgate, etc.");
    
    match discover_ecosystem_primals().await {
        Ok(discovered_primals) => {
            if discovered_primals.is_empty() {
                warn!("⚠️ No ecosystem primals discovered. Running in standalone mode.");
                demonstrate_standalone_mode().await?;
            } else {
                info!("✅ Discovered {} ecosystem primals", discovered_primals.len());
                
                for primal in &discovered_primals {
                    info!("  📡 {} [{}]: {} capabilities at {}", 
                        primal.primal_type.as_str(),
                        primal.metadata.get("primal_name").unwrap_or(&"unknown".to_string()),
                        primal.capabilities.len(),
                        primal.endpoint
                    );
                }
                
                demonstrate_ecosystem_mode(discovered_primals).await?;
            }
        }
        Err(e) => {
            error!("❌ Ecosystem discovery failed: {}", e);
            warn!("🔄 Falling back to standalone mode");
            demonstrate_standalone_mode().await?;
        }
    }

    info!("🎉 Universal Ecosystem Integration Demo completed successfully!");
    Ok(())
}

/// Demonstrate ecosystem mode with real primal integration
async fn demonstrate_ecosystem_mode(
    discovered_primals: Vec<songbird_universal_primals::discovery::types::DiscoveredPrimal>
) -> Result<(), Box<dyn std::error::Error>> {
    info!("🌌 Operating in ECOSYSTEM MODE - leveraging real primals");

    // Create user context for capability routing
    let user_context = create_universal_context("demo-user".to_string());
    
    // Group primals by their capabilities for demonstration
    let mut security_primals = Vec::new();
    let mut storage_primals = Vec::new();
    let mut compute_primals = Vec::new();
    let mut ai_primals = Vec::new();

    for primal in &discovered_primals {
        for capability in &primal.capabilities {
            match capability {
                PrimalCapability::Authentication { .. } 
                | PrimalCapability::Encryption { .. }
                | PrimalCapability::ThreatDetection { .. } => {
                    if !security_primals.contains(&primal.primal_id) {
                        security_primals.push(primal.primal_id.clone());
                    }
                }
                PrimalCapability::FileSystem { .. }
                | PrimalCapability::ObjectStorage { .. }
                | PrimalCapability::Backup { .. } => {
                    if !storage_primals.contains(&primal.primal_id) {
                        storage_primals.push(primal.primal_id.clone());
                    }
                }
                PrimalCapability::ContainerRuntime { .. }
                | PrimalCapability::ServerlessExecution { .. }
                | PrimalCapability::LoadBalancing { .. } => {
                    if !compute_primals.contains(&primal.primal_id) {
                        compute_primals.push(primal.primal_id.clone());
                    }
                }
                PrimalCapability::ModelInference { .. }
                | PrimalCapability::AgentFramework { .. }
                | PrimalCapability::MachineLearning { .. } => {
                    if !ai_primals.contains(&primal.primal_id) {
                        ai_primals.push(primal.primal_id.clone());
                    }
                }
                _ => {}
            }
        }
    }

    // Demonstrate Capability-Based Task Routing
    demonstrate_security_delegation(&security_primals, &discovered_primals).await?;
    demonstrate_storage_delegation(&storage_primals, &discovered_primals).await?;
    demonstrate_compute_delegation(&compute_primals, &discovered_primals).await?;
    demonstrate_ai_delegation(&ai_primals, &discovered_primals).await?;

    Ok(())
}

/// Demonstrate security delegation to BearDog (or any security-capable primal)
async fn demonstrate_security_delegation(
    security_primals: &[String],
    all_primals: &[songbird_universal_primals::discovery::types::DiscoveredPrimal]
) -> Result<(), Box<dyn std::error::Error>> {
    info!("🔐 SECURITY DELEGATION - Routing to security-capable primals");

    if security_primals.is_empty() {
        warn!("  ⚠️ No security primals available - using Songbird fallback");
        info!("  📱 Songbird: Handling authentication with built-in security");
        return Ok(());
    }

    for primal_id in security_primals {
        if let Some(primal) = all_primals.iter().find(|p| &p.primal_id == primal_id) {
            let primal_name = primal.metadata.get("primal_name").unwrap_or(&"unknown".to_string());
            
            info!("  🛡️ DELEGATING to {}: Authentication & Encryption", primal_name);
            
            // Show what capabilities this primal provides
            for capability in &primal.capabilities {
                match capability {
                    PrimalCapability::Authentication { methods } => {
                        info!("    🔑 Authentication: {:?}", methods);
                    }
                    PrimalCapability::Encryption { algorithms } => {
                        info!("    🔒 Encryption: {:?}", algorithms);
                    }
                    PrimalCapability::ThreatDetection { ml_enabled } => {
                        info!("    🚨 Threat Detection: ML={}", ml_enabled);
                    }
                    PrimalCapability::Authorization { rbac_support } => {
                        info!("    🎫 Authorization: RBAC={}", rbac_support);
                    }
                    _ => {}
                }
            }

            // In a real implementation, we would:
            // 1. Create PrimalRequest for authentication
            // 2. Send request to primal.endpoint
            // 3. Handle response with proper error handling
            // 4. Fall back to Songbird if primal fails
            
            info!("  ✅ Security operations delegated to {}", primal_name);
            break; // Use first available security primal
        }
    }

    Ok(())
}

/// Demonstrate storage delegation to NestGate (or any storage-capable primal)
async fn demonstrate_storage_delegation(
    storage_primals: &[String],
    all_primals: &[songbird_universal_primals::discovery::types::DiscoveredPrimal]
) -> Result<(), Box<dyn std::error::Error>> {
    info!("🗄️ STORAGE DELEGATION - Routing to storage-capable primals");

    if storage_primals.is_empty() {
        warn!("  ⚠️ No storage primals available - using Songbird fallback");
        info!("  📱 Songbird: Handling storage with local filesystem");
        return Ok(());
    }

    for primal_id in storage_primals {
        if let Some(primal) = all_primals.iter().find(|p| &p.primal_id == primal_id) {
            let primal_name = primal.metadata.get("primal_name").unwrap_or(&"unknown".to_string());
            
            info!("  💾 DELEGATING to {}: File System & Object Storage", primal_name);
            
            for capability in &primal.capabilities {
                match capability {
                    PrimalCapability::FileSystem { supports_zfs } => {
                        info!("    📁 File System: ZFS={}", supports_zfs);
                    }
                    PrimalCapability::ObjectStorage { backends } => {
                        info!("    🗃️ Object Storage: {:?}", backends);
                    }
                    PrimalCapability::DataReplication { consistency } => {
                        info!("    🔄 Replication: {}", consistency);
                    }
                    PrimalCapability::Backup { incremental } => {
                        info!("    💿 Backup: Incremental={}", incremental);
                    }
                    _ => {}
                }
            }
            
            info!("  ✅ Storage operations delegated to {}", primal_name);
            break;
        }
    }

    Ok(())
}

/// Demonstrate compute delegation to Toadstool (or any compute-capable primal)
async fn demonstrate_compute_delegation(
    compute_primals: &[String],
    all_primals: &[songbird_universal_primals::discovery::types::DiscoveredPrimal]
) -> Result<(), Box<dyn std::error::Error>> {
    info!("⚡ COMPUTE DELEGATION - Routing to compute-capable primals");

    if compute_primals.is_empty() {
        warn!("  ⚠️ No compute primals available - using Songbird fallback");
        info!("  📱 Songbird: Handling compute with local processes");
        return Ok(());
    }

    for primal_id in compute_primals {
        if let Some(primal) = all_primals.iter().find(|p| &p.primal_id == primal_id) {
            let primal_name = primal.metadata.get("primal_name").unwrap_or(&"unknown".to_string());
            
            info!("  🚀 DELEGATING to {}: Container Runtime & Serverless", primal_name);
            
            for capability in &primal.capabilities {
                match capability {
                    PrimalCapability::ContainerRuntime { orchestrators } => {
                        info!("    🐳 Container Runtime: {:?}", orchestrators);
                    }
                    PrimalCapability::ServerlessExecution { languages } => {
                        info!("    ⚡ Serverless: {:?}", languages);
                    }
                    PrimalCapability::LoadBalancing { algorithms } => {
                        info!("    ⚖️ Load Balancing: {:?}", algorithms);
                    }
                    PrimalCapability::AutoScaling { metrics } => {
                        info!("    📈 Auto-scaling: {:?}", metrics);
                    }
                    _ => {}
                }
            }
            
            info!("  ✅ Compute operations delegated to {}", primal_name);
            break;
        }
    }

    Ok(())
}

/// Demonstrate AI delegation to Squirrel (or any AI-capable primal)
async fn demonstrate_ai_delegation(
    ai_primals: &[String],
    all_primals: &[songbird_universal_primals::discovery::types::DiscoveredPrimal]
) -> Result<(), Box<dyn std::error::Error>> {
    info!("🧠 AI DELEGATION - Routing to AI-capable primals");

    if ai_primals.is_empty() {
        warn!("  ⚠️ No AI primals available - using Songbird fallback");
        info!("  📱 Songbird: AI operations not available in standalone mode");
        return Ok(());
    }

    for primal_id in ai_primals {
        if let Some(primal) = all_primals.iter().find(|p| &p.primal_id == primal_id) {
            let primal_name = primal.metadata.get("primal_name").unwrap_or(&"unknown".to_string());
            
            info!("  🤖 DELEGATING to {}: Model Inference & ML", primal_name);
            
            for capability in &primal.capabilities {
                match capability {
                    PrimalCapability::ModelInference { models } => {
                        info!("    🎯 Model Inference: {:?}", models);
                    }
                    PrimalCapability::AgentFramework { mcp_support } => {
                        info!("    🤝 Agent Framework: MCP={}", mcp_support);
                    }
                    PrimalCapability::MachineLearning { training_support } => {
                        info!("    🎓 Machine Learning: Training={}", training_support);
                    }
                    PrimalCapability::NaturalLanguage { languages } => {
                        info!("    💬 NLP: {:?}", languages);
                    }
                    _ => {}
                }
            }
            
            info!("  ✅ AI operations delegated to {}", primal_name);
            break;
        }
    }

    Ok(())
}

/// Demonstrate standalone mode when no ecosystem primals are available
async fn demonstrate_standalone_mode() -> Result<(), Box<dyn std::error::Error>> {
    info!("📱 Operating in STANDALONE MODE - Songbird handles all tasks");
    info!("  🔐 Security: Built-in authentication & AES-256-GCM encryption");
    info!("  🗄️ Storage: Local filesystem operations");
    info!("  ⚡ Compute: Local process management");
    info!("  🧠 AI: Basic rule-based intelligence only");
    info!("  🌐 Networking: Direct protocol handling");
    info!("  📊 Monitoring: Local metrics collection");
    
    info!("💡 ARCHITECTURAL STRENGTH: Songbird works perfectly alone!");
    info!("   When ecosystem primals become available, capabilities will automatically expand");
    
    Ok(())
} 