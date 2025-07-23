//! Capability-Based AI Delegation Demo
//!
//! This example demonstrates how Songbird uses capability-based discovery
//! to delegate AI functionality to ANY primal that provides AI capabilities,
//! not just hardcoded to "squirrel".

use songbird_core::api::ai_workload_classification::{
    AIWorkloadClassificationDelegate, WorkloadRequest, WorkloadType
};
use std::collections::HashMap;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("🚀 Starting Capability-Based AI Delegation Demo");

    // Create the AI delegate (no hardcoded primal names!)
    let ai_delegate = AIWorkloadClassificationDelegate::new();

    // Example workload request
    let workload = WorkloadRequest {
        id: "demo-workload-001".to_string(),
        workload_type: "machine_learning".to_string(),
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("priority".to_string(), serde_json::Value::String("high".to_string()));
            meta.insert("model_type".to_string(), serde_json::Value::String("llm".to_string()));
            meta
        },
        payload: serde_json::json!({
            "prompt": "Classify this workload for optimal resource allocation",
            "max_tokens": 1000
        }),
    };

    info!("📝 Example workload: {}", workload.id);

    // Demonstrate capability-based AI delegation
    info!("🔍 Discovering AI-capable primals...");
    
    // The delegate will:
    // 1. Search for primals with "ai", "ml", "intelligence", or "model" capabilities
    // 2. Route the request to ANY available AI primal (squirrel, custom-ai, neural-net, etc.)
    // 3. Fall back to basic classification if no AI primals are available
    
    match ai_delegate.classify_workload(&workload).await {
        Ok(classification) => {
            info!("✅ AI Classification Results:");
            info!("   Workload Type: {:?}", classification.workload_type);
            info!("   Confidence: {:.2}", classification.confidence_score);
            info!("   CPU Cores: {}", classification.resource_requirements.cpu_cores);
            info!("   Memory: {} MB", classification.resource_requirements.memory_mb);
        }
        Err(e) => {
            info!("⚠️ AI classification failed, using fallback: {}", e);
        }
    }

    // Demonstrate resource prediction
    info!("📊 Predicting resource requirements...");
    match ai_delegate.predict_resources(&WorkloadType::MachineLearning).await {
        Ok(resources) => {
            info!("✅ Resource Predictions:");
            info!("   CPU Cores: {}", resources.cpu_cores);
            info!("   Memory: {} MB", resources.memory_mb);
            info!("   Storage: {} MB", resources.storage_mb);
            info!("   Priority: {:?}", resources.priority);
        }
        Err(e) => {
            info!("⚠️ Resource prediction failed: {}", e);
        }
    }

    // Demonstrate risk assessment
    info!("🛡️ Assessing workload risks...");
    match ai_delegate.assess_risks(&workload, &Default::default()).await {
        Ok(assessment) => {
            info!("✅ Risk Assessment:");
            info!("   Overall Risk Score: {:.2}", assessment.overall_risk_score);
            info!("   Risk Factors: {:?}", assessment.risk_factors);
            info!("   Confidence: {:.2}", assessment.confidence);
        }
        Err(e) => {
            info!("⚠️ Risk assessment failed: {}", e);
        }
    }

    info!("🎉 Demo completed!");
    info!("");
    info!("💡 Key Benefits of Capability-Based AI Delegation:");
    info!("   • Works with ANY AI primal (squirrel, custom-ai, neural-net, etc.)");
    info!("   • No hardcoded primal names in Songbird");
    info!("   • Automatic discovery and failover");
    info!("   • Respects ecosystem boundaries");
    info!("   • Graceful fallback when no AI primals available");

    Ok(())
}

/// Configuration examples for different AI primals
fn print_configuration_examples() {
    info!("📋 Configuration Examples:");
    info!("");
    info!("🐿️ For Squirrel (existing):");
    info!("   export SQUIRREL_ENDPOINT=http://localhost:8002");
    info!("");
    info!("🧠 For custom Neural AI:");
    info!("   export NEURAL_AI_ENDPOINT=http://localhost:8010");
    info!("   export PRIMAL_1_NAME=neural-ai");
    info!("   export PRIMAL_1_ENDPOINT=http://localhost:8010");
    info!("");
    info!("🤖 For OpenAI Bridge:");
    info!("   export OPENAI_BRIDGE_ENDPOINT=http://localhost:8020");
    info!("   export PRIMAL_2_NAME=openai-bridge");
    info!("   export PRIMAL_2_ENDPOINT=http://localhost:8020");
    info!("");
    info!("🔄 Songbird automatically discovers and uses any of these!");
} 